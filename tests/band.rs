//! Bands around line series: confidence intervals, forecast ranges, min–max.
mod common;

use charts_rs::{
    BarChart, Error, LineChart, NIL_VALUE, Series, SeriesBand, SeriesCategory, compact_svg,
};

fn attr<'a>(tag: &'a str, name: &str) -> &'a str {
    let key = format!(" {name}=\"");
    let start = tag.find(&key).map(|i| i + key.len()).unwrap();
    let end = start + tag[start..].find('"').unwrap();
    &tag[start..end]
}

/// Every band of the svg, as its opening tag.
fn bands(svg: &str) -> Vec<&str> {
    svg.split('<')
        .filter(|t| t.contains(r#"class="ct-band""#))
        .collect()
}

/// The corners of a straight band: the upper bounds from left to right, then
/// the lower bounds back from right to left.
fn corners(tag: &str) -> Vec<(f32, f32)> {
    assert!(tag.starts_with("polygon"), "not a polygon: {tag}");
    attr(tag, "points")
        .split(' ')
        .map(|p| {
            let (x, y) = p.split_once(',').unwrap();
            (x.parse().unwrap(), y.parse().unwrap())
        })
        .collect()
}

/// The points of every straight line of the svg.
fn line_points(svg: &str) -> Vec<Vec<(f32, f32)>> {
    svg.split("<path d=\"M ")
        .skip(1)
        .map(|t| {
            t[..t.find('"').unwrap()]
                .split(" L ")
                .map(|p| {
                    let (x, y) = p.split_once(' ').unwrap();
                    (x.parse().unwrap(), y.parse().unwrap())
                })
                .collect()
        })
        .collect()
}

/// The numbers printed on the y axis.
fn axis_numbers(svg: &str) -> Vec<f32> {
    svg.lines()
        .map(str::trim)
        .filter(|l| !l.starts_with('<'))
        .filter_map(|l| l.parse().ok())
        .collect()
}

fn err(result: Result<impl Sized, Error>) -> String {
    match result {
        Ok(_) => panic!("expected an error"),
        Err(e) => e.to_string(),
    }
}

fn categories(count: usize) -> Vec<String> {
    (0..count).map(|i| format!("c{i}")).collect()
}

/// A line through `data` with a band of `lower`..`upper`.
fn banded(name: &str, data: Vec<f32>, lower: Vec<f32>, upper: Vec<f32>) -> Series {
    let mut series: Series = (name, data).into();
    series.band = Some(SeriesBand::new(lower, upper));
    series
}

fn chart(series_list: Vec<Series>) -> LineChart {
    let count = series_list
        .iter()
        .map(|s| {
            let band = s.band.as_ref();
            s.data.len().max(band.map(|b| b.lower.len()).unwrap_or(0))
        })
        .max()
        .unwrap_or(0);
    LineChart::new(series_list, categories(count))
}

#[test]
fn band_snapshots() {
    let forecast = LineChart::from_json(include_str!("../asset/line_chart/band.json")).unwrap();
    common::assert_snapshot!("line_chart/band_json.svg", forecast.svg().unwrap());
    let range = LineChart::from_json(include_str!("../asset/line_chart/band_range.json")).unwrap();
    common::assert_snapshot!("line_chart/band_range_json.svg", range.svg().unwrap());
}

#[test]
fn band_wraps_the_line() {
    let svg = chart(vec![banded(
        "A",
        vec![5.0, 7.0, 6.0, 9.0],
        vec![4.0, 5.0, 5.0, 7.0],
        vec![6.0, 9.0, 8.0, 12.0],
    )])
    .svg()
    .unwrap();

    let bands = bands(&svg);
    assert_eq!(1, bands.len());
    // The band takes the color of its series, mostly transparent.
    assert_eq!("#5470C6", attr(bands[0], "fill"));
    assert_eq!("0.24", attr(bands[0], "fill-opacity"));
    assert_eq!("A", attr(bands[0], "data-series"));

    let corners = corners(bands[0]);
    let lines = line_points(&svg);
    assert_eq!(1, lines.len());
    let line = &lines[0];
    assert_eq!(4, line.len());
    assert_eq!(8, corners.len());
    for (i, (x, y)) in line.iter().enumerate() {
        let (upper, lower) = (corners[i], corners[7 - i]);
        assert_eq!((*x, *x), (upper.0, lower.0), "point {i} of the band");
        // y grows downwards: the upper bound is above the line.
        assert!(
            upper.1 < *y && *y < lower.1,
            "point {i}: {upper:?} {y} {lower:?}"
        );
    }
}

#[test]
fn bands_lie_under_every_line() {
    let svg = chart(vec![
        banded(
            "A",
            vec![5.0, 7.0, 6.0],
            vec![4.0, 6.0, 5.0],
            vec![6.0, 8.0, 7.0],
        ),
        banded(
            "B",
            vec![2.0, 3.0, 4.0],
            vec![1.0, 2.0, 3.0],
            vec![3.0, 4.0, 5.0],
        ),
    ])
    .svg()
    .unwrap();

    assert_eq!(2, bands(&svg).len());
    assert_eq!(2, line_points(&svg).len());
    let last_band = svg.rfind("ct-band").unwrap();
    let first_line = svg.find("<path d=\"M ").unwrap();
    assert!(last_band < first_line, "a band is drawn over a line");
    // Each band in the color of its own series.
    let fills: Vec<&str> = bands(&svg).iter().map(|b| attr(b, "fill")).collect();
    assert_eq!(vec!["#5470C6", "#91CC75"], fills);
}

#[test]
fn smooth_line_gets_a_smooth_band() {
    let series = || {
        banded(
            "A",
            vec![5.0, 7.0, 6.0, 9.0],
            vec![4.0, 5.0, 5.0, 7.0],
            vec![6.0, 9.0, 8.0, 12.0],
        )
    };
    let mut smooth = chart(vec![series()]);
    smooth.series_smooth = true;
    let svg = smooth.svg().unwrap();
    let smooth_bands = bands(&svg);
    assert_eq!(1, smooth_bands.len());
    assert!(smooth_bands[0].starts_with("path"), "{}", smooth_bands[0]);
    assert!(attr(smooth_bands[0], "d").contains(" C"), "no curve");
    assert_eq!("0.24", attr(smooth_bands[0], "fill-opacity"));

    // The series' own setting wins over the chart's.
    let mut straight = series();
    straight.smooth = Some(false);
    let mut chart = chart(vec![straight]);
    chart.series_smooth = true;
    let svg = chart.svg().unwrap();
    assert!(bands(&svg)[0].starts_with("polygon"));
}

#[test]
fn y_axis_makes_room_for_the_band() {
    let plain = chart(vec![("A", vec![5.0, 7.0, 6.0, 9.0]).into()]);
    let numbers = axis_numbers(&plain.svg().unwrap());
    let max = numbers.iter().cloned().fold(f32::MIN, f32::max);
    assert!(max < 28.0, "{numbers:?}");

    let wide = chart(vec![banded(
        "A",
        vec![5.0, 7.0, 6.0, 9.0],
        vec![-6.0, 5.0, 5.0, 7.0],
        vec![6.0, 9.0, 8.0, 28.0],
    )]);
    let svg = wide.svg().unwrap();
    let numbers = axis_numbers(&svg);
    let max = numbers.iter().cloned().fold(f32::MIN, f32::max);
    let min = numbers.iter().cloned().fold(f32::MAX, f32::min);
    assert!(max >= 28.0 && min <= -6.0, "{numbers:?}");

    // So no corner of the band leaves the plot.
    let line = &line_points(&svg)[0];
    let corners = corners(bands(&svg)[0]);
    let (top, bottom) = corners
        .iter()
        .fold((f32::MAX, f32::MIN), |(t, b), p| (t.min(p.1), b.max(p.1)));
    assert!(top >= 0.0 && top < line[3].1, "{corners:?}");
    assert!(bottom > line[0].1, "{corners:?}");
}

#[test]
fn band_without_a_line() {
    let series = Series {
        name: "Range".to_string(),
        band: Some(SeriesBand::new(vec![2.0, 3.0, 1.0], vec![11.0, 13.0, 9.0])),
        ..Default::default()
    };
    let mut chart = chart(vec![series]);
    chart.tooltip_show = true;
    let svg = chart.svg().unwrap();

    // Not an empty chart: the band is the data.
    assert!(!svg.contains("No data"));
    assert_eq!(1, bands(&svg).len());
    assert!(line_points(&svg).is_empty());
    assert_eq!(6, corners(bands(&svg)[0]).len());
    // The legend still names it.
    assert!(svg.contains("\nRange\n"));

    // Each point of the band can be hovered, and tells its bounds.
    let strips: Vec<&str> = svg
        .split('<')
        .filter(|t| t.starts_with("rect") && t.contains("ct-trigger"))
        .collect();
    assert_eq!(3, strips.len());
    assert_eq!("c0", attr(strips[0], "data-category"));
    assert_eq!("2", attr(strips[0], "data-lower"));
    assert_eq!("11", attr(strips[0], "data-upper"));
    // Nothing is painted, so the strip has to ask for pointer events.
    assert_eq!("pointer-events:all", attr(strips[0], "style"));
    assert!(svg.contains("<title>Range: 2 – 11</title>"));
    assert!(svg.contains("<title>Range: 1 – 9</title>"));
    // The strip spans the band at its point.
    let corners = corners(bands(&svg)[0]);
    let (y, height): (f32, f32) = (
        attr(strips[0], "y").parse().unwrap(),
        attr(strips[0], "height").parse().unwrap(),
    );
    assert!((y - corners[0].1).abs() < 0.1);
    assert!((y + height - corners[5].1).abs() < 0.2);

    // No tooltips, no strips.
    chart.tooltip_show = false;
    assert!(!chart.svg().unwrap().contains("ct-trigger"));
}

#[test]
fn point_tooltip_tells_the_bounds() {
    let mut chart = chart(vec![
        banded(
            "A",
            vec![5.0, 7.0, NIL_VALUE],
            vec![4.0, 5.5, 6.0],
            vec![6.0, 9.0, 8.0],
        ),
        ("B", vec![1.0, 2.0, 3.0]).into(),
    ]);
    chart.tooltip_show = true;
    let svg = chart.svg().unwrap();

    assert!(svg.contains("<title>A: 5 (4 – 6)</title>"));
    assert!(svg.contains("<title>A: 7 (5.5 – 9)</title>"));
    assert!(svg.contains(r#"data-value="7" data-lower="5.5" data-upper="9""#));
    // The band reaches past the last point of the line, which is missing.
    assert!(svg.contains("<title>A: 6 – 8</title>"));
    // A series without a band is told as before.
    assert!(svg.contains("<title>B: 2</title>"));
    assert_eq!(3, svg.matches("data-lower=").count());
}

#[test]
fn missing_bound_splits_the_band() {
    let nil = NIL_VALUE;
    let svg = chart(vec![banded(
        "A",
        vec![3.0; 6],
        vec![1.0, 2.0, nil, 2.0, 1.0, 2.0],
        vec![4.0, 5.0, 6.0, 5.0, 4.0, 5.0],
    )])
    .svg()
    .unwrap();
    let bands_of = bands(&svg);
    assert_eq!(2, bands_of.len());
    assert_eq!(4, corners(bands_of[0]).len());
    assert_eq!(6, corners(bands_of[1]).len());
    // The line itself is not broken by a hole in its band.
    assert_eq!(6, line_points(&svg)[0].len());

    // A lone point between two holes has no width to draw.
    let svg = chart(vec![banded(
        "A",
        vec![3.0; 5],
        vec![1.0, nil, 2.0, 1.0, 2.0],
        vec![4.0, 5.0, 6.0, 5.0, nil],
    )])
    .svg()
    .unwrap();
    let bands_of = bands(&svg);
    assert_eq!(1, bands_of.len());
    assert_eq!(4, corners(bands_of[0]).len());

    // Bounds of different lengths: the band ends with the shorter one.
    let svg = chart(vec![banded(
        "A",
        vec![3.0; 4],
        vec![1.0, 2.0, 2.0],
        vec![4.0, 5.0, 6.0, 5.0],
    )])
    .svg()
    .unwrap();
    assert_eq!(6, corners(bands(&svg)[0]).len());
}

#[test]
fn swapped_bounds_are_put_in_order() {
    let ordered = chart(vec![banded(
        "A",
        vec![5.0, 7.0, 6.0],
        vec![4.0, 5.0, 5.0],
        vec![6.0, 9.0, 8.0],
    )]);
    let swapped = chart(vec![banded(
        "A",
        vec![5.0, 7.0, 6.0],
        vec![6.0, 5.0, 8.0],
        vec![4.0, 9.0, 5.0],
    )]);
    assert_eq!(ordered.svg().unwrap(), swapped.svg().unwrap());
}

#[test]
fn band_follows_start_index_and_the_axis_layout() {
    let mut series = banded("A", vec![5.0, 7.0], vec![4.0, 5.0], vec![6.0, 9.0]);
    series.start_index = 2;
    let mut chart = LineChart::new(vec![series], categories(4));
    for gap in [true, false] {
        chart.x_boundary_gap = Some(gap);
        let svg = chart.svg().unwrap();
        let line = &line_points(&svg)[0];
        let corners = corners(bands(&svg)[0]);
        assert_eq!(4, corners.len());
        assert_eq!(line[0].0, corners[0].0, "boundary gap {gap}");
        assert_eq!(line[1].0, corners[1].0, "boundary gap {gap}");
        // The series starts at the third category.
        assert!(line[0].0 > chart.width / 2.0 - 30.0);
    }

    // A band that runs past the last category stops at the axis end.
    let mut chart = LineChart::new(
        vec![banded("A", vec![5.0; 5], vec![4.0; 5], vec![6.0; 5])],
        categories(3),
    );
    chart.x_boundary_gap = Some(true);
    assert_eq!(6, corners(bands(&chart.svg().unwrap())[0]).len());
}

#[test]
fn band_on_a_continuous_axis() {
    // Uneven x values: the band keeps to them like the line does.
    let mut chart = LineChart::new(
        vec![banded(
            "A",
            vec![5.0, 7.0, 6.0, 9.0],
            vec![4.0, 5.0, 5.0, 7.0],
            vec![6.0, 9.0, 8.0, 12.0],
        )],
        vec![],
    );
    chart.x_axis_values = vec![0.0, 1.0, 5.0, 10.0];
    let svg = chart.svg().unwrap();
    let line = &line_points(&svg)[0];
    let band = corners(bands(&svg)[0]);
    let xs: Vec<f32> = line.iter().map(|p| p.0).collect();
    assert_eq!(xs, band[..4].iter().map(|p| p.0).collect::<Vec<_>>());
    assert!(xs[2] - xs[1] > 3.0 * (xs[1] - xs[0]), "{xs:?}");

    // A series with x values of its own, sharing the axis with another.
    let mut own = banded("B", vec![2.0, 3.0], vec![1.0, 2.0], vec![3.0, 4.0]);
    own.x_values = Some(vec![2.0, 8.0]);
    chart.series_list.push(own);
    let svg = chart.svg().unwrap();
    let band = corners(bands(&svg)[1]);
    assert_eq!(4, band.len());
    let line = &line_points(&svg)[1];
    assert_eq!((line[0].0, line[1].0), (band[0].0, band[1].0));
    assert!(xs[1] < band[0].0 && band[1].0 < xs[3]);

    // A band alone on a time axis: its length gives the points.
    let json = r#"{
        "x_axis_values": ["2024-03-01", "2024-03-02", "2024-03-04"],
        "series_list": [{"name": "Range", "data": [], "band": {"lower": [1, 2, 3], "upper": [4, 6, 5]}}]
    }"#;
    let svg = LineChart::from_json(json).unwrap().svg().unwrap();
    let band = corners(bands(&svg)[0]);
    assert_eq!(6, band.len());
    let (a, b, c) = (band[0].0, band[1].0, band[2].0);
    assert!(((c - b) - 2.0 * (b - a)).abs() < 0.5, "{band:?}");
}

#[test]
fn band_in_a_bar_chart() {
    let mut line = banded(
        "Trend",
        vec![5.0, 7.0, 6.0],
        vec![4.0, 5.0, 5.0],
        vec![6.0, 9.0, 8.0],
    );
    line.category = Some(SeriesCategory::Line);
    let chart = BarChart::new(
        vec![("Sales", vec![3.0, 4.0, 5.0]).into(), line],
        categories(3),
    );
    let svg = chart.svg().unwrap();
    let band = corners(bands(&svg)[0]);
    let line = &line_points(&svg)[0];
    // The line series of a bar chart is the second color.
    assert_eq!("#91CC75", attr(bands(&svg)[0], "fill"));
    for (i, point) in line.iter().enumerate() {
        assert_eq!(point.0, band[i].0);
    }
}

#[test]
fn band_from_json() {
    let json = r#"{
        "x_axis_data": ["a", "b", "c"],
        "series_list": [
            {"name": "A", "data": [5, 7, 6], "band": {"lower": [4, null, 5], "upper": [6, 9, 8]}},
            {"name": "B", "data": [1, 2, 3], "band": null},
            {"name": "C", "data": [1, 2, 3]}
        ]
    }"#;
    let chart = LineChart::from_json(json).unwrap();
    assert_eq!(
        Some(SeriesBand {
            lower: vec![Some(4.0), None, Some(5.0)],
            upper: vec![Some(6.0), Some(9.0), Some(8.0)],
        }),
        chart.series_list[0].band
    );
    assert_eq!(None, chart.series_list[1].band);
    assert_eq!(None, chart.series_list[2].band);
    // The hole leaves a point on each side: nothing wide enough to draw.
    assert!(bands(&chart.svg().unwrap()).is_empty());

    // The builder takes the legacy sentinel for a missing bound.
    assert_eq!(
        chart.series_list[0].band,
        Some(SeriesBand::new(
            vec![4.0, NIL_VALUE, 5.0],
            vec![6.0, 9.0, 8.0]
        ))
    );

    let message = err(LineChart::from_json(
        r#"{"series_list": [{"name": "A", "data": [1], "band": {"low": [1], "upper": [2]}}]}"#,
    ));
    assert!(
        message.contains("band") && message.contains("low"),
        "{message}"
    );
    let message = err(LineChart::from_json(
        r#"{"series_list": [{"name": "A", "data": [1], "band": {"lower": 1, "upper": [2]}}]}"#,
    ));
    assert!(message.contains("band.lower"), "{message}");
    let message = err(LineChart::from_json(
        r#"{"series_list": [{"name": "A", "data": [1], "band": [1, 2]}]}"#,
    ));
    assert!(message.contains("band"), "{message}");
}

#[test]
fn band_survives_compact_output() {
    // Each stretch of a split band carries its series, so the two are kept
    // apart rather than merged into one path.
    for json in [
        include_str!("../asset/line_chart/band.json"),
        include_str!("../asset/line_chart/band_range.json"),
    ] {
        let plain = LineChart::from_json(json).unwrap().svg().unwrap();
        let compact = compact_svg(&plain);
        assert!(compact.len() < plain.len());
        for needle in [
            "ct-band",
            "ct-trigger",
            "data-lower=",
            "<title>",
            "pointer-events:all",
        ] {
            assert_eq!(
                plain.matches(needle).count(),
                compact.matches(needle).count(),
                "{needle}"
            );
        }
    }
}

#[test]
fn chart_without_a_band_is_unchanged() {
    // A band that is present but empty draws nothing and costs nothing.
    let plain = chart(vec![("A", vec![5.0, 7.0, 6.0]).into()]);
    let mut empty: Series = ("A", vec![5.0, 7.0, 6.0]).into();
    empty.band = Some(SeriesBand::default());
    let empty = chart(vec![empty]);
    assert_eq!(plain.svg().unwrap(), empty.svg().unwrap());
}
