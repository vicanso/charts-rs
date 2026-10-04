//! Maps: regions colored by their values.
mod common;

use charts_rs::{Error, MapChart, MapProjection, MapRegion, MultiChart, compact_svg};

fn attr<'a>(tag: &'a str, name: &str) -> &'a str {
    let key = format!(" {name}=\"");
    let start = tag.find(&key).map(|i| i + key.len()).unwrap();
    let end = start + tag[start..].find('"').unwrap();
    &tag[start..end]
}

/// The regions of the svg: the paths that carry a name.
fn regions(svg: &str) -> Vec<&str> {
    svg.split("<path")
        .skip(1)
        .filter(|t| t.contains("data-name="))
        .collect()
}

/// The outlines of a region, each a list of corners.
fn rings(tag: &str) -> Vec<Vec<(f32, f32)>> {
    attr(tag, "d")
        .split(" Z")
        .filter(|ring| !ring.trim().is_empty())
        .map(|ring| {
            ring.trim()
                .trim_start_matches("M ")
                .split(" L ")
                .map(|p| {
                    let (x, y) = p.split_once(' ').unwrap();
                    (x.parse().unwrap(), y.parse().unwrap())
                })
                .collect()
        })
        .collect()
}

/// The text nodes of the svg.
fn texts(svg: &str) -> Vec<&str> {
    svg.lines()
        .map(str::trim)
        .filter(|l| !l.is_empty() && !l.starts_with('<'))
        .collect()
}

fn err(result: Result<impl Sized, Error>) -> String {
    match result {
        Ok(_) => panic!("expected an error"),
        Err(e) => e.to_string(),
    }
}

/// A square of ten degrees with its lower left corner at `(lon, lat)`.
fn square(name: &str, lon: f64, lat: f64) -> MapRegion {
    MapRegion {
        name: name.to_string(),
        polygons: vec![vec![vec![
            (lon, lat),
            (lon + 10.0, lat),
            (lon + 10.0, lat + 10.0),
            (lon, lat + 10.0),
            (lon, lat),
        ]]],
    }
}

/// Two squares side by side on the equator, without a scale beside them,
/// in degrees as they are.
fn chart(data: Vec<(&str, f32)>) -> MapChart {
    let mut chart = MapChart::new(
        vec![square("west", 0.0, 0.0), square("east", 10.0, 0.0)],
        data.into_iter().map(|(n, v)| (n.to_string(), v)).collect(),
    );
    chart.projection = MapProjection::Equirectangular;
    chart.visual_map_show = Some(false);
    chart
}

#[test]
fn map_snapshot() {
    let chart = MapChart::from_json(include_str!("../asset/map_chart/basic.json")).unwrap();
    assert_eq!(11, chart.regions.len());
    assert_eq!(10, chart.data.len());
    common::assert_snapshot!("map_chart/basic_json.svg", chart.svg().unwrap());
}

#[test]
fn regions_fill_the_plot() {
    let svg = chart(vec![("west", 1.0), ("east", 3.0)]).svg().unwrap();
    let regions = regions(&svg);
    assert_eq!(2, regions.len());
    let (west, east) = (rings(regions[0]), rings(regions[1]));
    // Twice as wide as high, the map takes the width of the plot (590 of
    // the 600 pixels) and is centered in its height.
    let xs: Vec<f32> = west[0].iter().chain(east[0].iter()).map(|p| p.0).collect();
    let ys: Vec<f32> = west[0].iter().chain(east[0].iter()).map(|p| p.1).collect();
    let (left, right) = (
        xs.iter().cloned().fold(f32::MAX, f32::min),
        xs.iter().cloned().fold(f32::MIN, f32::max),
    );
    let (top, bottom) = (
        ys.iter().cloned().fold(f32::MAX, f32::min),
        ys.iter().cloned().fold(f32::MIN, f32::max),
    );
    assert!(
        (left - 5.0).abs() < 0.1 && (right - 595.0).abs() < 0.1,
        "{left} {right}"
    );
    assert!((bottom - top - 295.0).abs() < 0.2, "{top} {bottom}");
    assert!((top + bottom - 400.0).abs() < 0.5, "{top} {bottom}");
    // The closing corner of a ring is not written twice.
    assert_eq!(4, west[0].len());
    // North is up: the first corner (the south west) is at the bottom left.
    assert_eq!((left, bottom), west[0][0]);
    // The two regions meet in the middle.
    assert!((west[0][1].0 - east[0][0].0).abs() < 0.1);
}

#[test]
fn regions_take_the_color_of_their_value() {
    let fills = |chart: &MapChart| -> Vec<String> {
        regions(&chart.svg().unwrap())
            .iter()
            .map(|r| attr(r, "fill").to_string())
            .collect()
    };
    let mut chart = chart(vec![("west", 0.0), ("east", 100.0), ("nowhere", 50.0)]);
    chart.min_color = "#ffffff".into();
    chart.max_color = "#000000".into();
    // From the smallest value to the largest.
    assert_eq!(vec!["#FFFFFF", "#000000"], fills(&chart));
    // On a scale of one's own.
    chart.min = -100.0;
    chart.max = 100.0;
    assert_eq!(vec!["#808080", "#000000"], fills(&chart));
    // Several colors, and classes: as in a heatmap.
    chart.colors = vec!["#ff0000".into(), "#00ff00".into(), "#0000ff".into()];
    assert_eq!(vec!["#00FF00", "#0000FF"], fills(&chart));
    chart.thresholds = vec![50.0, 200.0];
    assert_eq!(vec!["#FF0000", "#00FF00"], fills(&chart));

    // A region without a value, or with one that is no number, is drawn in
    // the color for it.
    chart.data = vec![("east".to_string(), 7.0), ("west".to_string(), f32::NAN)];
    chart.empty_color = "#123456".into();
    let svg = chart.svg().unwrap();
    assert_eq!("#123456", attr(regions(&svg)[0], "fill"));
    assert!(!regions(&svg)[0].contains("data-value="));
    assert_eq!("7", attr(regions(&svg)[1], "data-value"));
}

#[test]
fn holes_islands_and_borders() {
    let mut region = square("land", 0.0, 0.0);
    // A lake in it, and an island off it.
    region.polygons[0].push(vec![
        (4.0, 4.0),
        (6.0, 4.0),
        (6.0, 6.0),
        (4.0, 6.0),
        (4.0, 4.0),
    ]);
    region.polygons.push(vec![vec![
        (12.0, 0.0),
        (14.0, 0.0),
        (14.0, 2.0),
        (12.0, 0.0),
    ]]);
    // What is too small to show: a ring of two corners, and one of nothing.
    region
        .polygons
        .push(vec![vec![(20.0, 0.0), (20.0, 0.0)], vec![]]);
    let mut chart = MapChart::new(vec![region], vec![("land".to_string(), 1.0)]);
    chart.visual_map_show = Some(false);
    chart.border_color = "#ff0000".into();
    chart.border_width = 2.0;
    let svg = chart.svg().unwrap();
    let regions_of = regions(&svg);
    // One shape of three outlines; the lake is cut out of it.
    assert_eq!(1, regions_of.len());
    assert_eq!(
        vec![4, 4, 3],
        rings(regions_of[0])
            .iter()
            .map(Vec::len)
            .collect::<Vec<_>>()
    );
    assert_eq!("evenodd", attr(regions_of[0], "fill-rule"));
    assert_eq!(
        ("#FF0000", "2"),
        (
            attr(regions_of[0], "stroke"),
            attr(regions_of[0], "stroke-width")
        )
    );
    // Without a border width there is no stroke.
    chart.border_width = 0.0;
    assert!(!regions(&chart.svg().unwrap())[0].contains("stroke"));

    // Corners closer than a pixel can show are drawn as one.
    let mut fine = square("fine", 0.0, 0.0);
    fine.polygons[0][0] = (0..=5000)
        .map(|i| (i as f64 / 500.0, (i as f64 / 250.0).sin()))
        .chain([(10.0, 5.0), (0.0, 5.0)])
        .collect();
    let chart = MapChart::new(vec![fine], vec![]);
    let corners = rings(regions(&chart.svg().unwrap())[0])[0].len();
    assert!(corners > 100 && corners < 2500, "{corners}");
}

#[test]
fn mercator_stretches_the_north() {
    let stack = |projection| {
        let mut chart = MapChart::new(
            vec![square("south", 0.0, 0.0), square("north", 0.0, 60.0)],
            vec![],
        );
        chart.projection = projection;
        let svg = chart.svg().unwrap();
        let height = |tag: &str| {
            let ring = &rings(tag)[0];
            let ys: Vec<f32> = ring.iter().map(|p| p.1).collect();
            ys.iter().cloned().fold(f32::MIN, f32::max)
                - ys.iter().cloned().fold(f32::MAX, f32::min)
        };
        let regions = regions(&svg);
        (height(regions[0]), height(regions[1]))
    };
    // As they are, ten degrees are as high in the north as on the equator.
    let (south, north) = stack(MapProjection::Equirectangular);
    assert!((south - north).abs() < 0.2, "{south} {north}");
    // On a Mercator map they are more than twice as high at 60 to 70 north.
    let (south, north) = stack(MapProjection::Mercator);
    assert!(
        north > south * 2.0 && north < south * 3.0,
        "{south} {north}"
    );
}

#[test]
fn names_tooltips_and_the_scale() {
    let mut chart = chart(vec![("west", 20.0), ("east", 80.0)]);
    chart.label_show = true;
    chart.tooltip_show = true;
    chart.min_color = "#ffffff".into();
    chart.max_color = "#000000".into();
    let svg = chart.svg().unwrap();
    let texts_of = texts(&svg);
    assert!(
        texts_of.contains(&"west") && texts_of.contains(&"east"),
        "{texts_of:?}"
    );
    assert!(svg.contains("<title>west: 20</title>") && svg.contains(".ct-trigger:hover+.ct-tip"));
    assert_eq!("west", attr(regions(&svg)[0], "data-name"));
    // The name in the middle of its region, dark on the light one and
    // light on the dark one.
    let label = |name: &str| {
        svg.split("<text")
            .find(|t| t.contains(&format!("\n{name}\n")) && !t.contains("ct-tip"))
            .unwrap()
    };
    assert_eq!("152.5", attr(label("west"), "x"));
    assert_eq!("#000000", attr(label("west"), "fill"));
    assert_eq!("#FFFFFF", attr(label("east"), "fill"));

    // The scale beside the map: a bar from the smallest to the largest
    // value, and the map moves over for it.
    chart.label_show = false;
    chart.tooltip_show = false;
    chart.visual_map_show = None;
    let svg = chart.svg().unwrap();
    assert_eq!(vec!["80", "20"], texts(&svg));
    assert_eq!(22, svg.matches("<rect").count() - 1);
    assert!(rings(regions(&svg)[0])[0][0].0 > 30.0);
    // With classes: a swatch and a range for each of them.
    chart.thresholds = vec![30.0, 60.0];
    let svg = chart.svg().unwrap();
    assert_eq!(vec!["&lt; 30", "30 – 60", "≥ 60"], texts(&svg));
    assert_eq!(3, svg.matches("<rect").count() - 1);
    chart.thresholds = vec![];
    chart.steps = 4;
    assert_eq!(
        vec!["&lt; 35", "35 – 50", "50 – 65", "≥ 65"],
        texts(&chart.svg().unwrap())
    );
    // Without values there is nothing to tell a scale of.
    chart.data = vec![];
    assert!(texts(&chart.svg().unwrap()).is_empty());
}

#[test]
fn map_from_json() {
    let json = r##"{
        "name_property": "code",
        "projection": "Equirectangular",
        "min": 0,
        "max": 10,
        "min_color": "#ffffff",
        "max_color": "#000000",
        "empty_color": "#123456",
        "border_color": "#654321",
        "border_width": 2,
        "label_show": true,
        "visual_map_show": false,
        "steps": 2,
        "data": [["W", 2], ["E", 9], ["no value"], [7, 1]],
        "geo_json": {"type": "FeatureCollection", "features": [
            {"type": "Feature", "properties": {"code": "W", "name": "West"},
             "geometry": {"type": "Polygon", "coordinates": [[[0, 0], [10, 0], [10, 10], [0, 10], [0, 0]]]}},
            {"type": "Feature", "properties": {"code": "E"},
             "geometry": {"type": "Polygon", "coordinates": [[[10, 0], [20, 0], [20, 10], [10, 10], [10, 0]]]}},
            {"type": "Feature", "properties": {"code": "X"},
             "geometry": {"type": "Polygon", "coordinates": [[[20, 0], [30, 0], [30, 10], [20, 10], [20, 0]]]}}
        ]}
    }"##;
    let chart = MapChart::from_json(json).unwrap();
    assert_eq!(
        vec!["W", "E", "X"],
        chart
            .regions
            .iter()
            .map(|r| r.name.as_str())
            .collect::<Vec<_>>()
    );
    assert_eq!(
        vec![("W".to_string(), 2.0), ("E".to_string(), 9.0)],
        chart.data
    );
    assert_eq!(MapProjection::Equirectangular, chart.projection);
    let svg = chart.svg().unwrap();
    let fills: Vec<&str> = regions(&svg).iter().map(|r| attr(r, "fill")).collect();
    assert_eq!(vec!["#FFFFFF", "#000000", "#123456"], fills);
    assert_eq!(
        ("#654321", "2"),
        (
            attr(regions(&svg)[0], "stroke"),
            attr(regions(&svg)[0], "stroke-width")
        )
    );

    // The same chart, built from the GeoJSON in code.
    let geo_json = serde_json_text(json);
    let mut built = MapChart::new(
        MapRegion::from_geo_json(&geo_json, "code").unwrap(),
        vec![("W".to_string(), 2.0), ("E".to_string(), 9.0)],
    );
    built.projection = MapProjection::Equirectangular;
    built.max = 10.0;
    built.min_color = "#ffffff".into();
    built.max_color = "#000000".into();
    built.empty_color = "#123456".into();
    built.border_color = "#654321".into();
    built.border_width = 2.0;
    built.label_show = true;
    built.visual_map_show = Some(false);
    built.steps = 2;
    assert_eq!(built.svg().unwrap(), svg);

    // Nothing to draw without regions.
    let mut empty = MapChart::from_json(r#"{"data": [["a", 1]], "empty_text": "No map"}"#).unwrap();
    assert_eq!(vec!["No map"], texts(&empty.svg().unwrap()));
    empty.width = 10.0;
    empty.height = 8.0;
    assert!(empty.svg().unwrap().starts_with("<svg"));

    let message = err(MapChart::from_json(r#"{"projection": "globe"}"#));
    assert!(message.contains("mercator, equirectangular"), "{message}");
    let message = err(MapChart::from_json(r#"{"geojson": {}}"#));
    assert!(message.contains("geo_json"), "{message}");
    let message = err(MapChart::from_json(r#"{"data": {"a": 1}}"#));
    assert!(message.contains("data"), "{message}");
}

/// The `geo_json` of a chart's options, as a document of its own.
fn serde_json_text(json: &str) -> String {
    let start = json.find("\"geo_json\":").unwrap() + 11;
    let end = json.rfind('}').unwrap();
    json[start..end].trim().to_string()
}

#[test]
fn map_in_a_multi_chart_and_compact() {
    let json = include_str!("../asset/map_chart/basic.json");
    let multi = format!(
        r#"{{"child_charts": [{}]}}"#,
        json.replacen('{', r#"{"type": "map", "#, 1)
    );
    let svg = MultiChart::from_json(&multi).unwrap().svg().unwrap();
    assert_eq!(11, regions(&svg).len());

    let plain = MapChart::from_json(json).unwrap().svg().unwrap();
    let compact = compact_svg(&plain);
    assert!(compact.len() < plain.len());
    for needle in [
        "data-name=",
        "<title>",
        "class=\"ct-trigger\"",
        "fill-rule=\"evenodd\"",
    ] {
        assert_eq!(
            plain.matches(needle).count(),
            compact.matches(needle).count(),
            "{needle}"
        );
    }
}
