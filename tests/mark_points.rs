//! The markers on the points of a line: on its smallest and its largest
//! value, and on any point that is asked for by its index.

use charts_rs::{
    BarChart, LineChart, MarkPoint, MarkPointCategory, Series, measure_text_width_family,
};

/// A day of half hours with a peak in the morning, at index 19, and a lower
/// one in the afternoon, at index 28.
const DAY: &str = "[55, 60, 20, 20, 12, 10, 22, 20, 30, 10, 12, 25, 20, 62, 75, 110, 170, 320, 1090, 1505, 1320, 1170, 910, 660, 490, 492, 660, 780, 932, 770, 705, 545, 440, 325, 290]";

fn day(marks: &str, options: &str) -> String {
    let labels: Vec<String> = (0..35)
        .map(|i| format!("\"{:02}:{:02}\"", i / 2, i % 2 * 30))
        .collect();
    format!(
        r##"{{"legend_show": false, "x_axis_data": [{}]{options},
            "series_list": [{{"name": "handled", "data": {DAY}, "mark_points": [{marks}]}}]}}"##,
        labels.join(",")
    )
}

fn line(marks: &str, options: &str) -> String {
    LineChart::from_json(&day(marks, options))
        .unwrap()
        .svg()
        .unwrap()
}

/// A marker as it is drawn: its middle, its radius, its tip, and the text
/// in it with what the text is written in.
#[derive(Debug)]
struct Marker {
    x: f32,
    y: f32,
    r: f32,
    tip: (f32, f32),
    text: String,
    tag: String,
}

fn markers(svg: &str) -> Vec<Marker> {
    let pair = |text: &str| -> (f32, f32) {
        let (x, y) = text.trim().split_once(',').unwrap();
        (x.parse().unwrap(), y.parse().unwrap())
    };
    svg.split("<path d=\"M ")
        .skip(1)
        .filter(|path| path.matches(" 0,0,1 ").count() == 3 && path.contains(" L "))
        .map(|path| {
            let d = &path[..path.find('"').unwrap()];
            // Three arcs: to the left of the circle, to its right, and down
            // to where the tip begins.
            let arcs: Vec<&str> = d.split(" A ").collect();
            let r = pair(arcs[1].split(' ').next().unwrap()).0;
            let left = pair(arcs[1].rsplit(' ').next().unwrap());
            let right = pair(arcs[2].rsplit(' ').next().unwrap());
            let tip = pair(d[d.find(" L ").unwrap() + 3..].trim_end_matches(" Z"));
            let text = &path[path.find("<text").unwrap()..];
            let tag = text[..text.find('>').unwrap()].to_string();
            let text = text[text.find('>').unwrap() + 1..text.find("</text>").unwrap()].trim();
            Marker {
                x: (left.0 + right.0) / 2.0,
                y: left.1,
                r,
                tip,
                text: text.to_string(),
                tag,
            }
        })
        .collect()
}

fn attr<'a>(tag: &'a str, name: &str) -> Option<&'a str> {
    let key = format!(" {name}=\"");
    let start = tag.find(&key)? + key.len();
    Some(&tag[start..start + tag[start..].find('"')?])
}

/// Where the points of the line are.
fn points(svg: &str) -> Vec<(f32, f32)> {
    let path = svg
        .split("<path d=\"M ")
        .skip(1)
        .find(|path| path[..path.find('>').unwrap()].contains("fill=\"none\""))
        .unwrap();
    path[..path.find('"').unwrap()]
        .split(" L ")
        .map(|point| {
            let (x, y) = point.trim().split_once(' ').unwrap();
            (x.parse().unwrap(), y.parse().unwrap())
        })
        .collect()
}

#[test]
fn a_point_is_marked_by_its_index() {
    // Two peaks: the largest value, and the one of the afternoon, which is
    // the largest of nothing.
    let svg = line(
        r#"{"category": "max"}, {"category": "index", "index": 28}"#,
        "",
    );
    let found = markers(&svg);
    assert_eq!(
        vec!["1505", "932"],
        found.iter().map(|m| m.text.as_str()).collect::<Vec<_>>()
    );
    // Each above its point: the tip 7.5 pixels over it.
    let points = points(&svg);
    assert_eq!(35, points.len());
    for (marker, index) in found.iter().zip([19, 28]) {
        let (x, y) = points[index];
        assert!((marker.tip.0 - x).abs() < 0.2, "{marker:?} {x}");
        assert!((marker.tip.1 - (y - 7.5)).abs() < 0.2, "{marker:?} {y}");
        assert!((marker.x - x).abs() < 0.2 && marker.y < marker.tip.1);
    }

    // Any point, the first and the last as well, and as many as are wanted.
    let svg = line(
        r#"{"category": "index", "index": 0}, {"category": "index", "index": 34},
           {"category": "index", "index": 24}, {"category": "min"}"#,
        "",
    );
    let texts: Vec<String> = markers(&svg).into_iter().map(|m| m.text).collect();
    assert_eq!(vec!["55", "290", "490", "10"], texts);

    // A point has one marker, however often it is asked for.
    let svg = line(
        r#"{"category": "max"}, {"category": "index", "index": 19}, {"category": "index", "index": 19}"#,
        "",
    );
    assert_eq!(1, markers(&svg).len());
    // There is none for an index the series has no value at,
    assert!(markers(&line(r#"{"category": "index", "index": 35}"#, "")).is_empty());
    let gap = LineChart::from_json(
        r#"{"x_axis_data": ["a", "b", "c"], "series_list": [{"name": "s", "data": [1, null, 3],
            "mark_points": [{"category": "index", "index": 1}, {"category": "index", "index": 2}]}]}"#,
    )
    .unwrap()
    .svg()
    .unwrap();
    let texts: Vec<String> = markers(&gap).into_iter().map(|m| m.text).collect();
    assert_eq!(vec!["3"], texts);
    // and none for a mark point that does not say which index.
    let chart = LineChart::from_json(&day(r#"{"category": "index"}"#, "")).unwrap();
    assert!(chart.series_list[0].mark_points.is_empty());
    // An index is a whole number that is not negative.
    for index in ["-1", "1.5", "\"19\""] {
        let json = day(&format!(r#"{{"category": "index", "index": {index}}}"#), "");
        let message = match LineChart::from_json(&json) {
            Ok(_) => panic!("{index} is taken"),
            Err(e) => e.to_string(),
        };
        assert!(message.contains("index"), "{message}");
    }

    // The same in Rust.
    let chart = LineChart::from_json(&day(
        r#"{"category": "max"}, {"category": "Index", "index": 28}"#,
        "",
    ))
    .unwrap();
    assert_eq!(
        vec![
            MarkPoint {
                category: MarkPointCategory::Max
            },
            MarkPoint {
                category: MarkPointCategory::Index(28)
            },
        ],
        chart.series_list[0].mark_points
    );
    let mut series: Series = ("s", vec![1.0, 5.0, 2.0, 4.0, 1.0]).into();
    series.mark_points = vec![MarkPoint {
        category: MarkPointCategory::Index(3),
    }];
    let labels = ["a", "b", "c", "d", "e"].map(String::from).to_vec();
    let svg = LineChart::new(vec![series], labels).svg().unwrap();
    assert_eq!("4", markers(&svg)[0].text);
}

#[test]
fn a_marker_is_as_large_as_its_value_and_read_on_its_color() {
    let width = |text: &str| {
        measure_text_width_family("Roboto", 14.0, text)
            .unwrap()
            .width()
    };
    let svg = line(
        r#"{"category": "max"}, {"category": "index", "index": 28}, {"category": "min"}"#,
        "",
    );
    let found = markers(&svg);
    assert_eq!(3, found.len());
    for marker in found.iter() {
        // The corners of the text are 3 pixels inside of the rim: a value
        // of four figures was wider than its marker.
        let reach = (width(&marker.text) / 2.0).hypot(14.0 * 0.36) + 3.0;
        assert!(marker.r >= reach - 0.1, "{marker:?} {reach}");
        // No marker is smaller than those of two figures were.
        assert!(marker.r >= 15.0);
        // The text is in the middle of it, wherever its font puts it.
        assert_eq!(Some("middle"), attr(&marker.tag, "text-anchor"));
        assert_eq!(Some("central"), attr(&marker.tag, "dominant-baseline"));
        let at = |name: &str| attr(&marker.tag, name).unwrap().parse::<f32>().unwrap();
        assert!((at("x") - marker.x).abs() < 0.2 && (at("y") - marker.y).abs() < 0.2);
        assert_eq!(None, attr(&marker.tag, "dx"));
        // Its tip is as far below its middle as it was: one and a half
        // times its radius.
        assert!((marker.tip.1 - marker.y - marker.r * 1.5).abs() < 0.2);
    }
    let radius = |text: &str| found.iter().find(|m| m.text == text).unwrap().r;
    assert!(radius("1505") > 19.0 && radius("1505") < 22.0);
    assert!(radius("932") > 15.0 && radius("932") < radius("1505"));
    assert_eq!(15.0, radius("10"));

    // White on a dark marker, dark on a light one.
    let color = |series_color: &str| -> String {
        let svg = line(
            r#"{"category": "max"}"#,
            &format!(r#", "series_colors": ["{series_color}"]"#),
        );
        let marker = &markers(&svg)[0];
        assert!(svg.contains(&format!("Z\" fill=\"{}\"", series_color.to_uppercase())));
        attr(&marker.tag, "fill").unwrap().to_string()
    };
    assert_eq!("#FFFFFF", color("#3b78d8"));
    assert_eq!("#FFFFFF", color("#5470c6"));
    assert_eq!("#FFFFFF", color("#000000"));
    assert_eq!("#464646", color("#fac858"));
    assert_eq!("#464646", color("#91cc75"));

    // What is in it is the label of the point: as it is formatted, and in
    // the weight of the labels.
    let svg = line(
        r#"{"category": "max"}"#,
        r#", "series_label_formatter": "{t}", "series_label_font_weight": "bold""#,
    );
    let marker = &markers(&svg)[0];
    assert_eq!("1,505", marker.text);
    assert_eq!(Some("bold"), attr(&marker.tag, "font-weight"));
    assert!(marker.r > radius("1505"));
}

/// The y of the highest line of the grid: where the plot starts.
fn plot_top(svg: &str) -> f32 {
    svg.split("<line")
        .skip(1)
        .filter_map(|tag| {
            let (y1, y2) = (attr(tag, "y1")?, attr(tag, "y2")?);
            (y1 == y2).then(|| y1.parse::<f32>().unwrap())
        })
        .fold(f32::MAX, f32::min)
}

#[test]
fn a_plot_leaves_room_for_the_markers_above_it() {
    let both = r#"{"category": "max"}, {"category": "index", "index": 28}"#;
    let titled = r#", "title_text": "Handled", "sub_title_text": "Two peaks""#;
    // The largest value is near the top of its axis, and its marker above
    // the plot: over the titles, or off the chart where there are none.
    for (options, header) in [("", 5.0), (titled, 60.0), (r#", "margin": 0"#, 0.0)] {
        let plain = line("", options);
        let marked = line(both, options);
        let top = markers(&marked)
            .iter()
            .map(|marker| marker.y - marker.r)
            .fold(f32::MAX, f32::min);
        // The plot starts lower by what the marker needs, and no more: the
        // marker begins where the plot did.
        assert!(top >= header - 0.2, "{options}: {top}");
        assert!((top - plot_top(&plain)).abs() < 0.6, "{options}: {top}");
        assert!(plot_top(&marked) > plot_top(&plain) + 20.0, "{options}");
        // It ends where it did.
        let bottom = |svg: &str| points(svg).iter().map(|p| p.1).fold(f32::MIN, f32::max);
        assert!(bottom(&marked) > bottom(&plain));
    }

    // With room on the plot, nothing moves: a value far below the top of
    // its axis, and a marker on the smallest one.
    for (marks, options) in [
        (both, r#", "y_axis_configs": [{"axis_max": 3000}]"#),
        (
            r#"{"category": "min"}, {"category": "index", "index": 24}"#,
            "",
        ),
    ] {
        let (plain, marked) = (line("", options), line(marks, options));
        assert_eq!(plot_top(&plain), plot_top(&marked), "{marks}");
        assert_eq!(points(&plain), points(&marked), "{marks}");
        assert!(!markers(&marked).is_empty());
    }

    // The lines of a bar chart have their markers too, and its bars none.
    let bar = |marks: &str, category: &str| -> String {
        BarChart::from_json(&format!(
            r#"{{"legend_show": false, "x_axis_data": ["a", "b", "c"],
                "y_axis_configs": [{{"axis_max": 10}}], "series_list": [
                {{"name": "bars", "data": [5, 9, 7]}},
                {{"name": "s", "data": [4, 9.8, 6]{category}, "mark_points": [{marks}]}}]}}"#
        ))
        .unwrap()
        .svg()
        .unwrap()
    };
    let line_series = r#", "category": "line""#;
    let (plain, marked) = (
        bar("", line_series),
        bar(r#"{"category": "max"}"#, line_series),
    );
    let found = markers(&marked);
    assert_eq!(
        vec!["9.8"],
        found.iter().map(|m| m.text.as_str()).collect::<Vec<_>>()
    );
    assert!(found[0].y - found[0].r >= 4.8);
    assert!(plot_top(&marked) > plot_top(&plain));
    let (plain, marked) = (bar("", ""), bar(r#"{"category": "max"}"#, ""));
    assert!(markers(&marked).is_empty());
    assert_eq!(plain, marked);

    // The markers of stacked lines are on the chart as well.
    let stacked = LineChart::from_json(
        r#"{"legend_show": false, "x_axis_data": ["a", "b", "c"], "series_list": [
            {"name": "s", "data": [4, 9, 6], "stack": "all"},
            {"name": "t", "data": [5, 11, 7], "stack": "all", "mark_points": [{"category": "max"}]}]}"#,
    )
    .unwrap()
    .svg()
    .unwrap();
    let found = markers(&stacked);
    assert_eq!(1, found.len());
    assert!(found[0].y - found[0].r >= 4.8, "{:?}", found[0]);
}
