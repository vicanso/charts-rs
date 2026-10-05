//! Error bars on bars, lines and scatter points.
mod common;

use charts_rs::{BarChart, Error, LineChart, ScatterChart, Series, SeriesBand};

fn attr<'a>(tag: &'a str, name: &str) -> &'a str {
    let key = format!(" {name}=\"");
    let start = tag.find(&key).map(|i| i + key.len()).unwrap();
    let end = start + tag[start..].find('"').unwrap();
    &tag[start..end]
}

fn number(tag: &str, name: &str) -> f32 {
    attr(tag, name).parse().unwrap()
}

/// The lines of the error bars as `(x1, y1, x2, y2)`: three for each bar,
/// its stem and its two caps. They are the only lines 1.5 wide.
fn error_lines(svg: &str) -> Vec<(f32, f32, f32, f32)> {
    svg.split("<line")
        .skip(1)
        .filter(|t| t.contains(r#"stroke-width="1.5""#))
        .map(|t| {
            (
                number(t, "x1"),
                number(t, "y1"),
                number(t, "x2"),
                number(t, "y2"),
            )
        })
        .collect()
}

/// The numbers written in the svg.
fn numbers(svg: &str) -> Vec<f32> {
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

fn with_errors(data: Vec<f32>, lower: Vec<f32>, upper: Vec<f32>) -> Series {
    let mut series: Series = ("A", data).into();
    series.error_bar = Some(SeriesBand::new(lower, upper));
    series
}

#[test]
fn error_bar_snapshot() {
    let chart = BarChart::from_json(include_str!("../asset/bar_chart/error_bar.json")).unwrap();
    common::assert_snapshot!("bar_chart/error_bar_json.svg", chart.svg().unwrap());
}

#[test]
fn error_bars_on_bars() {
    let mut chart = BarChart::new(
        vec![with_errors(
            vec![20.0, 10.0],
            vec![10.0, 5.0],
            vec![30.0, 15.0],
        )],
        categories(2),
    );
    chart.legend.show = Some(false);
    chart.tooltip.show = true;
    chart.y_axis_configs[0].max = Some(40.0);
    chart.y_axis_configs[0].split_number = 4;
    let svg = chart.svg().unwrap();

    let lines = error_lines(&svg);
    assert_eq!(6, lines.len());
    let bars: Vec<&str> = svg
        .split("<rect")
        .skip(1)
        .filter(|t| t.contains("data-series="))
        .collect();
    let (bar_x, bar_top, bar_width, bar_height) = (
        number(bars[0], "x"),
        number(bars[0], "y"),
        number(bars[0], "width"),
        number(bars[0], "height"),
    );
    // The stem stands in the middle of its bar, from the lower bound to the
    // upper one: half the value below the top of the bar and above it.
    let stem = lines[0];
    assert_eq!(stem.0, stem.2);
    assert!((stem.0 - (bar_x + bar_width / 2.0)).abs() < 0.1);
    let (high, low) = (stem.1.min(stem.3), stem.1.max(stem.3));
    assert!((low - (bar_top + bar_height / 2.0)).abs() < 0.2, "{stem:?}");
    assert!((bar_top - high - bar_height / 2.0).abs() < 0.2, "{stem:?}");
    // A cap at each end, centered on the stem.
    for cap in &lines[1..3] {
        assert_eq!(cap.1, cap.3);
        assert!(cap.1 == stem.1 || cap.1 == stem.3);
        assert!(((cap.0 + cap.2) / 2.0 - stem.0).abs() < 0.1);
        assert!(cap.2 - cap.0 >= 4.0 && cap.2 - cap.0 <= 12.0);
    }

    // The bounds are told with the value.
    assert!(svg.contains("<title>A: 20 (10 – 30)</title>"));
    assert_eq!(
        ("10", "30"),
        (attr(bars[0], "data-lower"), attr(bars[0], "data-upper"))
    );
    // Over the bars, not under them.
    assert!(svg.rfind("<rect").unwrap() < svg.find(r#"stroke-width="1.5""#).unwrap());
}

#[test]
fn axis_makes_room_for_the_errors() {
    let plain = BarChart::new(vec![("A", vec![20.0, 10.0]).into()], categories(2));
    let max = |svg: &str| numbers(svg).into_iter().fold(f32::MIN, f32::max);
    assert!(max(&plain.svg().unwrap()) < 60.0);
    let wide = BarChart::new(
        vec![with_errors(
            vec![20.0, 10.0],
            vec![-15.0, 5.0],
            vec![90.0, 15.0],
        )],
        categories(2),
    );
    let svg = wide.svg().unwrap();
    assert!(max(&svg) >= 90.0);
    assert!(numbers(&svg).into_iter().fold(f32::MAX, f32::min) <= -15.0);
    // No end of an error bar is above the plot.
    assert!(error_lines(&svg).iter().all(|l| l.1 >= 0.0 && l.3 >= 0.0));
}

#[test]
fn error_bars_on_lines_and_points() {
    let nil = charts_rs::NIL_VALUE;
    let mut chart = LineChart::new(
        vec![with_errors(
            vec![20.0, 10.0, 30.0],
            vec![15.0, nil, 20.0],
            vec![25.0, 12.0, 35.0, 99.0],
        )],
        categories(3),
    );
    chart.legend.show = Some(false);
    chart.tooltip.show = true;
    let svg = chart.svg().unwrap();
    // A point without both of its bounds has no error bar; a bound without
    // a point neither.
    let lines = error_lines(&svg);
    assert_eq!(6, lines.len());
    // On the points of the line, in its color.
    let points: Vec<(f32, f32)> = svg
        .split("<circle")
        .skip(1)
        .filter(|t| t.contains("ct-trigger"))
        .map(|t| (number(t, "cx"), number(t, "cy")))
        .collect();
    assert_eq!((points[0].0, points[2].0), (lines[0].0, lines[3].0));
    assert!(lines[0].1.min(lines[0].3) < points[0].1 && points[0].1 < lines[0].1.max(lines[0].3));
    assert_eq!(6, svg.matches(r##"stroke-width="1.5" x1="##).count());
    assert!(svg.contains(r##"stroke="#5470C6"/>"##));
    assert!(svg.contains("<title>A: 20 (15 – 25)</title>"));
    assert!(svg.contains("<title>A: 10</title>"));

    // Scatter points: the bounds of the n-th point are the n-th of the lists.
    let mut series = with_errors(
        vec![1.0, 4.0, 2.0, 7.0, 3.0, 5.0],
        vec![3.0, 5.5, 4.0],
        vec![5.0, 8.0, 14.0],
    );
    series.name = "S".to_string();
    let mut chart = ScatterChart::new(vec![series]);
    chart.legend.show = Some(false);
    let svg = chart.svg().unwrap();
    let lines = error_lines(&svg);
    assert_eq!(9, lines.len());
    let points: Vec<&str> = svg
        .split("<circle")
        .skip(1)
        .filter(|t| t.contains("data-series="))
        .collect();
    assert_eq!(3, points.len());
    for (i, point) in points.iter().enumerate() {
        assert_eq!(number(point, "cx"), lines[i * 3].0);
    }
    assert_eq!(
        ("4", "14"),
        (attr(points[2], "data-lower"), attr(points[2], "data-upper"))
    );
    // The y axis reaches up to the highest bound.
    assert!(numbers(&svg).into_iter().fold(f32::MIN, f32::max) >= 14.0);
    // The bars are drawn under their points.
    assert!(svg.find(r#"stroke-width="1.5""#).unwrap() < svg.find("data-series=").unwrap());
}

#[test]
fn error_bar_from_json() {
    let json = |error: &str| {
        format!(
            r#"{{"x_axis_data": ["a", "b"], "series_list": [{{"name": "A", "data": [20, 10]{error}}}]}}"#
        )
    };
    let plain = BarChart::from_json(&json("")).unwrap();
    assert_eq!(None, plain.series_list[0].error_bar);
    let chart = BarChart::from_json(&json(
        r#", "error_bar": {"lower": [10, null], "upper": [30, 15]}"#,
    ))
    .unwrap();
    assert_eq!(
        Some(SeriesBand {
            lower: vec![Some(10.0), None],
            upper: vec![Some(30.0), Some(15.0)],
            ..Default::default()
        }),
        chart.series_list[0].error_bar
    );
    assert_eq!(3, error_lines(&chart.svg().unwrap()).len());
    // Bounds the wrong way round are put in order.
    let swapped = BarChart::from_json(&json(
        r#", "error_bar": {"lower": [30, null], "upper": [10, 15]}"#,
    ))
    .unwrap();
    assert_eq!(chart.svg().unwrap(), swapped.svg().unwrap());
    // `null` is no error bar.
    let none = BarChart::from_json(&json(r#", "error_bar": null"#)).unwrap();
    assert_eq!(plain.svg().unwrap(), none.svg().unwrap());

    let message = err(BarChart::from_json(&json(r#", "error_bar": {"low": [1]}"#)));
    assert!(
        message.contains("error_bar") && message.contains("low"),
        "{message}"
    );
    let message = err(BarChart::from_json(&json(r#", "error_bar": [1, 2]"#)));
    assert!(message.contains("error_bar"), "{message}");
}
