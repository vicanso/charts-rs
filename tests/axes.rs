//! Continuous (value / time) x axes, axis titles, bubble charts and data
//! label overlap handling.
mod common;

use charts_rs::{
    AxisType, BarChart, BoxPlotChart, Error, HeatmapChart, HorizontalBarChart, LineChart,
    ScatterChart, Series, SeriesCategory, WaterfallChart,
};

fn attr(tag: &str, name: &str) -> f32 {
    let key = format!(" {name}=\"");
    let start = tag.find(&key).map(|i| i + key.len()).unwrap();
    let end = start + tag[start..].find('"').unwrap();
    tag[start..end].parse().unwrap()
}

/// `cx` of every hover target of a line chart (one per data point).
fn point_xs(svg: &str) -> Vec<f32> {
    svg.split("<circle")
        .skip(1)
        .filter(|t| t.contains("ct-trigger"))
        .map(|t| attr(t, "cx"))
        .collect()
}

/// `(x, y, width, height)` of every `<rect>` after the background.
fn rects(svg: &str) -> Vec<(f32, f32, f32, f32)> {
    svg.split("<rect")
        .skip(2)
        .map(|t| {
            (
                attr(t, "x"),
                attr(t, "y"),
                attr(t, "width"),
                attr(t, "height"),
            )
        })
        .collect()
}

/// The text nodes of the svg, one per line of content.
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

const HOUR: f64 = 3600.0;
/// 2024-03-01 00:00 UTC.
const T0: f64 = 1_709_251_200.0;

#[test]
fn time_axis_keeps_the_real_spacing() {
    let mut chart = LineChart::new(vec![("A", vec![1.0, 2.0, 3.0, 4.0]).into()], vec![]);
    chart.x_axis.kind = AxisType::Time;
    chart.x_axis.values = vec![T0, T0 + HOUR, T0 + 3.0 * HOUR, T0 + 9.0 * HOUR];
    chart.tooltip.show = true;
    let svg = chart.svg().unwrap();
    let xs = point_xs(&svg);
    assert_eq!(4, xs.len());
    // Gaps of 1h, 2h and 6h: the pixels follow the time, not the index.
    let unit = xs[1] - xs[0];
    assert!(((xs[2] - xs[1]) - 2.0 * unit).abs() < 0.3, "{xs:?}");
    assert!(((xs[3] - xs[2]) - 6.0 * unit).abs() < 0.3, "{xs:?}");
    // Ticks at round times, the first one naming the day.
    let labels = texts(&svg);
    for label in ["03-01", "02:00", "04:00", "06:00", "08:00"] {
        assert!(labels.contains(&label), "{label} in {labels:?}");
    }
    // The point's time is its category in tooltips and data attributes.
    assert!(svg.contains("data-category=\"2024-03-01 03:00\""), "{svg}");

    // A display offset shifts the labels, not the points.
    chart.x_axis.time_offset = 8 * 60;
    let shifted = chart.svg().unwrap();
    assert_eq!(xs, point_xs(&shifted));
    assert!(shifted.contains("data-category=\"2024-03-01 11:00\""));
    // A custom pattern.
    chart.x_axis.formatter = Some("%H:%M".to_string());
    assert!(texts(&chart.svg().unwrap()).contains(&"10:00"));
}

#[test]
fn value_axis_and_per_series_x() {
    let mut own: Series = ("B", vec![5.0, 6.0]).into();
    own.x_values = Some(vec![2.0, 10.0]);
    let mut chart = LineChart::new(vec![("A", vec![1.0, 2.0, 3.0]).into(), own], vec![]);
    chart.x_axis.values = vec![0.0, 5.0, 20.0];
    chart.tooltip.show = true;
    let svg = chart.svg().unwrap();
    let xs = point_xs(&svg);
    assert_eq!(5, xs.len());
    let width = xs[2] - xs[0];
    // Series A at 0, 5, 20; series B at its own 2 and 10.
    assert!((xs[1] - xs[0] - width * 0.25).abs() < 0.3, "{xs:?}");
    assert!((xs[3] - xs[0] - width * 0.1).abs() < 0.3, "{xs:?}");
    assert!((xs[4] - xs[0] - width * 0.5).abs() < 0.3, "{xs:?}");
    let labels = texts(&svg);
    for label in ["0", "5", "10", "15", "20"] {
        assert!(labels.contains(&label), "{label} in {labels:?}");
    }
    // A fixed range.
    chart.x_axis.min = Some(-20.0);
    chart.x_axis.max = Some(20.0);
    let xs = point_xs(&chart.svg().unwrap());
    let width = xs[2] - xs[0];
    assert!((xs[1] - xs[0] - width * 0.25).abs() < 0.3);
    assert!(texts(&chart.svg().unwrap()).contains(&"-20"));

    // A point without an x value breaks the line instead of being guessed.
    let mut chart = LineChart::new(vec![("A", vec![1.0, 2.0, 3.0]).into()], vec![]);
    chart.x_axis.values = vec![0.0, f64::NAN, 2.0];
    chart.tooltip.show = true;
    assert_eq!(2, point_xs(&chart.svg().unwrap()).len());

    // Without x values the axis stays a category axis.
    let mut chart = LineChart::new(
        vec![("A", vec![1.0, 2.0]).into()],
        vec!["a".into(), "b".into()],
    );
    let plain = chart.svg().unwrap();
    chart.x_axis.kind = AxisType::Time;
    assert_eq!(plain, chart.svg().unwrap());
}

#[test]
fn bars_on_a_continuous_axis() {
    let day = 86400.0;
    let mut line: Series = ("Rate", vec![1.0, 2.0, 3.0, 4.0]).into();
    line.category = Some(SeriesCategory::Line);
    let mut chart = BarChart::new(
        vec![("Orders", vec![10.0, 20.0, 30.0, 40.0]).into(), line],
        vec![],
    );
    chart.x_axis.kind = AxisType::Time;
    chart.x_axis.values = vec![T0, T0 + day, T0 + 4.0 * day, T0 + 5.0 * day];
    let svg = chart.svg().unwrap();
    let bars = rects(&svg);
    assert_eq!(4, bars.len(), "{svg}");
    // Every bar is as wide as a day allows, and none sticks out of the plot.
    assert!(
        bars.iter()
            .all(|b| (b.2 - bars[0].2).abs() < 0.1 && b.2 > 10.0)
    );
    let centre = |b: &(f32, f32, f32, f32)| b.0 + b.2 / 2.0;
    let unit = centre(&bars[1]) - centre(&bars[0]);
    assert!(((centre(&bars[2]) - centre(&bars[1])) - 3.0 * unit).abs() < 0.5);
    assert!(bars[0].0 > 0.0 && bars[3].0 + bars[3].2 < 600.0);
    assert!(!svg.contains("width=\"-"));

    // A single x value still gets a bar of a sensible width.
    let mut chart = BarChart::new(vec![("A", vec![3.0]).into()], vec![]);
    chart.x_axis.values = vec![7.0];
    let bars = rects(&chart.svg().unwrap());
    assert_eq!(1, bars.len());
    assert!(bars[0].2 > 50.0 && bars[0].2 < 400.0, "{bars:?}");
}

#[test]
fn x_values_from_json() {
    let chart = LineChart::from_json(
        r#"{"x_axis_values": ["2024-03-01", "2024-03-02 12:00", "2024-03-05T00:00:00+08:00", null],
            "x_axis_min": "2024-02-28", "x_axis_formatter": "%m/%d", "x_axis_time_offset": 480,
            "series_list": [{"name": "A", "data": [1, 2, 3, 4]}, {"name": "B", "x_values": [1709337600, 1709424000], "data": [5, 6]}]}"#,
    )
    .unwrap();
    // Date strings make it a time axis; numbers are unix seconds.
    assert_eq!(AxisType::Time, chart.x_axis.kind);
    assert_eq!(T0, chart.x_axis.values[0]);
    assert_eq!(T0 + 36.0 * HOUR, chart.x_axis.values[1]);
    assert_eq!(T0 + 4.0 * 86400.0 - 8.0 * HOUR, chart.x_axis.values[2]);
    assert!(chart.x_axis.values[3].is_nan());
    assert_eq!(Some(T0 - 2.0 * 86400.0), chart.x_axis.min);
    assert_eq!(480, chart.x_axis.time_offset);
    assert_eq!(
        Some(vec![1_709_337_600.0, 1_709_424_000.0]),
        chart.series_list[1].x_values
    );
    chart.svg().unwrap();

    let chart = LineChart::from_json(
        r#"{"x_axis_type": "value", "x_axis_values": [1, 2.5], "series_list": [{"name": "A", "data": [1, 2]}]}"#,
    )
    .unwrap();
    assert_eq!(AxisType::Value, chart.x_axis.kind);

    assert!(
        err(LineChart::from_json(r#"{"x_axis_values": ["yesterday"]}"#)).contains("x_axis_values")
    );
    assert!(
        err(LineChart::from_json(r#"{"x_axis_type": "log"}"#)).contains("category, value, time")
    );
    assert!(
        err(LineChart::from_json(
            r#"{"series_list": [{"name": "A", "data": [1], "x_values": "2024"}]}"#
        ))
        .contains("x_values")
    );
    assert!(err(LineChart::from_json(r#"{"x_axis_max": true}"#)).contains("x_axis_max"));
}

#[test]
fn axis_titles() {
    let make = || {
        let mut right: Series = ("B", vec![10.0, 30.0]).into();
        right.y_axis_index = 1;
        let mut chart = BarChart::new(
            vec![("A", vec![1.0, 2.0]).into(), right],
            vec!["a".into(), "b".into()],
        );
        chart.y_axis_configs.push(chart.y_axis_configs[0].clone());
        chart
    };
    let plain = make().svg().unwrap();
    let mut chart = make();
    chart.x_axis.title = "Weekday".to_string();
    chart.y_axis_configs[0].title = Some("Orders".to_string());
    chart.y_axis_configs[1].title = Some("Rate <%>".to_string());
    let svg = chart.svg().unwrap();
    let labels = texts(&svg);
    for title in ["Weekday", "Orders", "Rate &lt;%&gt;"] {
        assert!(labels.contains(&title), "{title} in {labels:?}");
    }
    // The y titles are rotated to read along their axes.
    assert_eq!(1, svg.matches("rotate(-90)").count());
    assert_eq!(1, svg.matches("rotate(90)").count());
    // The plot gives up the room the titles take.
    let (before, after) = (rects(&plain), rects(&svg));
    assert!(after[0].2 < before[0].2, "narrower bars");
    assert!(after[1].3 < before[1].3, "shorter bars");
    assert!(after[0].0 > before[0].0, "shifted right of the y title");

    // A hidden axis has no title.
    chart.x_axis.hidden = true;
    chart.y_axis_hidden = true;
    let hidden = chart.svg().unwrap();
    assert!(!hidden.contains("Weekday") && !hidden.contains("rotate("));

    let chart = BarChart::from_json(
        r#"{"x_axis_title": "Day", "y_axis_configs": [{"axis_title": "Count"}],
            "series_list": [{"name": "A", "data": [1, 2]}], "x_axis_data": ["a", "b"]}"#,
    )
    .unwrap();
    assert_eq!("Day", chart.x_axis.title);
    assert_eq!(Some("Count".to_string()), chart.y_axis_configs[0].title);
}

#[test]
fn axis_titles_on_every_cartesian_chart() {
    let titles = r#""x_axis_title": "The X", "y_axis_configs": [{"axis_title": "The Y"}]"#;
    let svgs = [
        LineChart::from_json(&format!(
            r#"{{{titles}, "series_list": [{{"name": "A", "data": [1, 2]}}], "x_axis_data": ["a", "b"]}}"#
        ))
        .unwrap()
        .svg()
        .unwrap(),
        ScatterChart::from_json(&format!(
            r#"{{{titles}, "series_list": [{{"name": "A", "data": [1, 2, 3, 4]}}]}}"#
        ))
        .unwrap()
        .svg()
        .unwrap(),
        HorizontalBarChart::from_json(&format!(
            r#"{{{titles}, "series_list": [{{"name": "A", "data": [1, 2]}}], "x_axis_data": ["a", "b"]}}"#
        ))
        .unwrap()
        .svg()
        .unwrap(),
        WaterfallChart::from_json(&format!(
            r#"{{{titles}, "x_axis_data": ["a", "b"], "data": [[5, false], [0, true]]}}"#
        ))
        .unwrap()
        .svg()
        .unwrap(),
        BoxPlotChart::from_json(&format!(
            r#"{{{titles}, "x_axis_data": ["a"], "box_series": [{{"name": "A", "data": [[1, 2, 3, 4, 5]]}}]}}"#
        ))
        .unwrap()
        .svg()
        .unwrap(),
        HeatmapChart::from_json(&format!(
            r#"{{{titles}, "x_axis_data": ["a", "b"], "y_axis_data": ["x"], "series": {{"data": [[0, 1], [1, 5]]}}}}"#
        ))
        .unwrap()
        .svg()
        .unwrap(),
    ];
    for (index, svg) in svgs.iter().enumerate() {
        let labels = texts(svg);
        assert_eq!(
            1,
            labels.iter().filter(|l| **l == "The X").count(),
            "chart {index}"
        );
        assert_eq!(
            1,
            labels.iter().filter(|l| **l == "The Y").count(),
            "chart {index}"
        );
        assert!(svg.contains("rotate(-90)"), "chart {index}");
    }
}

#[test]
fn bubble_chart() {
    // (x, y, size) triples.
    let mut chart = ScatterChart::new(vec![
        ("A", vec![1.0, 1.0, 100.0, 2.0, 2.0, 400.0, 3.0, 3.0, 25.0]).into(),
        ("B", vec![2.0, 1.0, 100.0]).into(),
    ]);
    chart.bubble = true;
    chart.bubble_min_size = 5.0;
    chart.bubble_max_size = 25.0;
    chart.tooltip.show = true;
    let svg = chart.svg().unwrap();
    let radii: Vec<f32> = svg
        .split("<circle")
        .skip(1)
        .filter(|t| t.contains("data-size"))
        .map(|t| attr(t, "r"))
        .collect();
    // Drawn large to small within a series; the area follows the size, so
    // 100 (a fifth of the 25..400 range) is at the root of that fraction.
    assert_eq!(4, radii.len(), "{svg}");
    assert_eq!(25.0, radii[0]);
    assert_eq!(5.0, radii[2]);
    let expected = 5.0 + 20.0 * (75.0_f32 / 375.0).sqrt();
    assert!((radii[1] - expected).abs() < 0.1, "{radii:?}");
    assert!(
        (radii[3] - expected).abs() < 0.1,
        "every series is drawn as circles"
    );
    assert!(svg.contains("data-size=\"400\"") && svg.contains("<title>A: (2, 2, 400)</title>"));
    // The size is not mistaken for a coordinate.
    assert!(!texts(&svg).contains(&"400"));

    // Without the flag the same data are (x, y) pairs as before.
    chart.bubble = false;
    assert!(!chart.svg().unwrap().contains("data-size"));

    let chart = ScatterChart::from_json(
        r#"{"bubble": true, "bubble_max_size": 40, "series_list": [{"name": "A", "data": [1, 2, 3, 4, 5, null]}]}"#,
    )
    .unwrap();
    assert!(chart.bubble);
    assert_eq!((4.0, 40.0), (chart.bubble_min_size, chart.bubble_max_size));
    // A missing size falls back to the series' symbol size.
    assert!(chart.svg().unwrap().contains(" r=\"10\""));
}

#[test]
fn overlapping_data_labels_can_be_hidden() {
    let data: Vec<f32> = (0..60).map(|i| 1000.0 + (i * 37 % 250) as f32).collect();
    let categories: Vec<String> = (0..60).map(|i| format!("d{i}")).collect();
    let mut series: Series = ("A", data).into();
    series.label_show = true;
    let mut chart = LineChart::new(vec![series], categories);
    chart.legend.show = Some(false);
    // The data labels are the four-digit values (axis labels read "1k").
    let count = |svg: &str| {
        texts(svg)
            .iter()
            .filter(|t| t.parse::<u32>().is_ok_and(|v| (1000..1300).contains(&v)))
            .count()
    };
    let all = chart.svg().unwrap();
    assert_eq!(60, count(&all), "every label by default");
    chart.series.label.hide_overlap = true;
    let hidden = chart.svg().unwrap();
    let kept = count(&hidden);
    assert!(kept > 5 && kept < 40, "{kept} labels kept");
    // The data itself is untouched.
    assert_eq!(
        all.matches("<circle").count(),
        hidden.matches("<circle").count()
    );

    let chart = LineChart::from_json(r#"{"series_label_hide_overlap": true}"#).unwrap();
    assert!(chart.series.label.hide_overlap);
}

#[test]
fn a_label_on_the_edge_stays_on_the_canvas() {
    let mut series: Series = ("A", vec![1.0, 123456.0]).into();
    series.label_show = true;
    let mut chart = LineChart::new(vec![series], vec![]);
    chart.x_axis.values = vec![0.0, 10.0];
    chart.series.label.formatter = "{c} units".to_string();
    let svg = chart.svg().unwrap();
    // The last point sits on the right edge of the plot; its label is
    // moved left instead of being cut off by the canvas.
    let label = svg
        .split("<text")
        .find(|t| t.contains("123456 units"))
        .expect("label");
    let right = attr(label, "x") + attr(label, "dx") + 85.0;
    assert!(right <= 600.0 + 1.0, "label ends at {right}");
}

#[test]
fn snapshots() {
    let line = LineChart::from_json(include_str!("../asset/line_chart/time_axis.json")).unwrap();
    common::assert_snapshot!("line_chart/time_axis_json.svg", line.svg().unwrap());
    let bar = BarChart::from_json(include_str!("../asset/bar_chart/time_axis.json")).unwrap();
    common::assert_snapshot!("bar_chart/time_axis_json.svg", bar.svg().unwrap());
    let bubble =
        ScatterChart::from_json(include_str!("../asset/scatter_chart/bubble.json")).unwrap();
    common::assert_snapshot!("scatter_chart/bubble_json.svg", bubble.svg().unwrap());
}
