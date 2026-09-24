//! Regression tests for edge cases that used to panic, emit invalid SVG
//! (`NaN`/`inf`/negative sizes) or silently draw the wrong thing.
mod common;

use charts_rs::{
    Align, BarChart, CandlestickChart, Color, GaugeChart, HorizontalBarChart, LineChart, MarkPoint,
    MarkPointCategory, MultiChart, NIL_VALUE, PieChart, SankeyChart, Series, SunburstChart,
    TableChart, TreemapChart,
};

fn attr(tag: &str, name: &str) -> f32 {
    let key = format!(" {name}=\"");
    let start = tag.find(&key).map(|i| i + key.len()).unwrap();
    let end = start + tag[start..].find('"').unwrap();
    tag[start..end].parse().unwrap()
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

fn assert_valid_numbers(svg: &str) {
    for bad in ["NaN", "inf", "width=\"-", "height=\"-", " r=\"-"] {
        assert!(!svg.contains(bad), "svg contains {bad:?}");
    }
}

const EPSILON: f32 = 0.15;

#[test]
fn bar_negative_values_hang_from_zero_line() {
    let chart = BarChart::new(
        vec![("A", vec![-7.0, 10.0, -3.0, 5.0]).into()],
        vec!["a".into(), "b".into(), "c".into(), "d".into()],
    );
    let svg = chart.svg().unwrap();
    assert_valid_numbers(&svg);
    assert!(
        svg.lines().any(|l| l.trim() == "0"),
        "0 must be an axis tick"
    );
    let bars = rects(&svg);
    assert_eq!(4, bars.len());
    for (i, (_, _, w, h)) in bars.iter().enumerate() {
        assert!(*w > 0.0 && *h > 0.0, "bar {i} has size {w}x{h}");
    }
    // Positive bars end where negative bars start: the zero line.
    let zero = bars[1].1 + bars[1].3;
    assert!((bars[0].1 - zero).abs() < EPSILON, "-7 starts at zero line");
    assert!((bars[2].1 - zero).abs() < EPSILON, "-3 starts at zero line");
    assert!(
        (bars[3].1 + bars[3].3 - zero).abs() < EPSILON,
        "5 ends at zero line"
    );
    assert!(bars[0].3 > bars[2].3, "-7 is taller than -3");
}

#[test]
fn bar_stacked_negative_values() {
    let mut a: Series = ("A", vec![-5.0, 3.0]).into();
    a.stack = Some("s".into());
    let mut b: Series = ("B", vec![10.0, -4.0]).into();
    b.stack = Some("s".into());
    let chart = BarChart::new(vec![a, b], vec!["a".into(), "b".into()]);
    let svg = chart.svg().unwrap();
    assert_valid_numbers(&svg);
    let bars = rects(&svg);
    assert_eq!(4, bars.len());
    assert!(bars.iter().all(|r| r.2 > 0.0 && r.3 > 0.0));
    // Column "a": -5 hangs below zero, +10 stands above it, sharing the line.
    assert!((bars[0].1 - (bars[2].1 + bars[2].3)).abs() < EPSILON);
    // Column "b": +3 above, -4 below.
    assert!(((bars[1].1 + bars[1].3) - bars[3].1).abs() < EPSILON);
}

#[test]
fn horizontal_bar_negative_values_and_unequal_series() {
    let chart = HorizontalBarChart::new(
        vec![("A", vec![-5.0, 3.0]).into()],
        vec!["a".into(), "b".into()],
    );
    let svg = chart.svg().unwrap();
    assert_valid_numbers(&svg);
    let bars = rects(&svg);
    assert_eq!(2, bars.len());
    assert!(bars.iter().all(|r| r.2 > 0.0));
    // The negative bar ends where the positive one starts.
    assert!((bars[0].0 + bars[0].2 - bars[1].0).abs() < EPSILON);

    // A shorter series keeps its bars on the rows of its own categories.
    let chart = HorizontalBarChart::new(
        vec![
            ("A", vec![1.0, 2.0, 3.0]).into(),
            ("B", vec![1.0, 2.0]).into(),
        ],
        vec!["a".into(), "b".into(), "c".into()],
    );
    let svg = chart.svg().unwrap();
    let bars = rects(&svg);
    assert_eq!(5, bars.len());
    let (a0, a1, b0) = (bars[0].1, bars[1].1, bars[3].1);
    let row = a0 - a1;
    assert!(
        b0 > a0 && b0 - a0 < row / 2.0,
        "B[0] shares the row of A[0]"
    );
}

#[test]
fn animation_easing_is_sanitized_in_every_chart() {
    let easing = r#""easing": "ease}} </style><script>alert(1)</script>""#;
    let anim = format!(r#""animation": {{"duration": 500, {easing}, "delay": 10}}"#);
    let svgs = [
        TreemapChart::from_json(&format!(
            r#"{{"series_list": [{{"name": "a", "data": [3]}}, {{"name": "b", "data": [1]}}], {anim}}}"#
        ))
        .unwrap()
        .svg()
        .unwrap(),
        SunburstChart::from_json(&format!(
            r#"{{"series_data": [{{"name": "p", "children": [{{"name": "c", "value": 3}}]}}], {anim}}}"#
        ))
        .unwrap()
        .svg()
        .unwrap(),
        SankeyChart::from_json(&format!(
            r#"{{"nodes": [{{"name": "a"}}, {{"name": "b"}}], "links": [{{"source": "a", "target": "b", "value": 3}}], {anim}}}"#
        ))
        .unwrap()
        .svg()
        .unwrap(),
    ];
    for svg in svgs {
        assert!(
            svg.contains("500ms ease both"),
            "easing falls back to the default"
        );
        assert!(!svg.contains("<script") && !svg.contains("alert(1)"));
    }
}

#[test]
fn multi_chart_json_theme_handling() {
    let child = r#"{"type": "bar", "series_list": [{"name": "A", "data": [1, 2]}], "x_axis_data": ["a", "b"]}"#;
    // Neither the top level nor the child names a theme.
    let chart = MultiChart::from_json(&format!(r#"{{"child_charts": [{child}]}}"#)).unwrap();
    chart.svg().unwrap();

    // The top-level theme is inherited by the child.
    let inherited = MultiChart::from_json(&format!(
        r#"{{"theme": "dark", "child_charts": [{child}]}}"#
    ))
    .unwrap()
    .svg()
    .unwrap();
    let explicit = MultiChart::from_json(&format!(
        r#"{{"child_charts": [{}]}}"#,
        child.replacen('{', r#"{"theme": "dark", "#, 1)
    ))
    .unwrap()
    .svg()
    .unwrap();
    assert_eq!(explicit, inherited);

    let err = match MultiChart::from_json(r#"{"child_charts": [{"type": "nope"}]}"#) {
        Err(err) => err,
        Ok(_) => panic!("an unknown child type must be rejected"),
    };
    assert!(err.to_string().contains("nope"), "{err}");
}

#[test]
fn extreme_inputs_do_not_panic() {
    // Values far beyond i32 used to overflow the tick rounding.
    let svg = BarChart::from_json(
        r#"{"series_list": [{"name": "A", "data": [1e10, 2e10]}], "x_axis_data": ["a", "b"]}"#,
    )
    .unwrap()
    .svg()
    .unwrap();
    assert_valid_numbers(&svg);

    // A huge start index must not allocate a vector that long (JSON rejects
    // it up front; the builder API still has to cope).
    assert!(
        BarChart::from_json(
            r#"{"series_list": [{"name": "A", "data": [1], "stack": "s", "start_index": 4000000000000000000}], "x_axis_data": ["a"]}"#,
        )
        .is_err()
    );
    let mut series: Series = ("A", vec![1.0]).into();
    series.stack = Some("s".into());
    series.start_index = 4_000_000_000_000_000_000;
    BarChart::new(vec![series], vec!["a".into()]).svg().unwrap();

    // No categories with `x_boundary_gap = false` used to underflow.
    let mut chart = LineChart::new(vec![("A", Vec::<f32>::new()).into()], vec![]);
    chart.x_boundary_gap = Some(false);
    assert_valid_numbers(&chart.svg().unwrap());

    // A single point without boundary gap divided by zero.
    let mut chart = LineChart::new(vec![("A", vec![3.0]).into()], vec!["a".into()]);
    chart.x_boundary_gap = Some(false);
    assert_valid_numbers(&chart.svg().unwrap());

    // An empty row palette used to divide by zero.
    TableChart::from_json(r#"{"data": [["h"], ["r"]], "body_background_colors": []}"#)
        .unwrap()
        .svg()
        .unwrap();
}

#[test]
fn non_finite_values_are_treated_as_missing() {
    let svg = PieChart::new(vec![
        ("A", vec![10.0]).into(),
        Series::new_nullable("B".into(), vec![None]),
    ])
    .svg()
    .unwrap();
    assert_valid_numbers(&svg);

    let svg = CandlestickChart::new(
        vec![("A", vec![1.0, NIL_VALUE, 0.5, 2.0, 1.0, 1.5, 0.5, 2.0]).into()],
        vec!["a".into(), "b".into()],
    )
    .svg()
    .unwrap();
    assert_valid_numbers(&svg);

    let svg = LineChart::new(
        vec![("A", vec![f32::NAN, f32::INFINITY, 2.0, 3.0]).into()],
        vec!["a".into(), "b".into(), "c".into(), "d".into()],
    )
    .svg()
    .unwrap();
    assert_valid_numbers(&svg);
}

#[test]
fn mark_point_skips_missing_points() {
    let mut series: Series = ("A", vec![NIL_VALUE, 11.0, 47.0, 23.0]).into();
    series.mark_points = vec![MarkPoint {
        category: MarkPointCategory::Max,
    }];
    let chart = LineChart::new(
        vec![series],
        vec!["a".into(), "b".into(), "c".into(), "d".into()],
    );
    let svg = chart.svg().unwrap();
    assert!(svg.lines().any(|l| l.trim() == "47"), "max marker shows 47");
    assert!(!svg.lines().any(|l| l.trim() == "23"), "23 is not the max");
}

#[test]
fn bar_hidden_y_axis_keeps_right_axis_scale() {
    let mut b: Series = ("B", vec![100.0, 200.0]).into();
    b.y_axis_index = 1;
    let mut chart = BarChart::new(
        vec![("A", vec![1.0, 2.0]).into(), b],
        vec!["a".into(), "b".into()],
    );
    chart.y_axis_configs.push(chart.y_axis_configs[0].clone());
    chart.y_axis_hidden = true;
    let svg = chart.svg().unwrap();
    let bars = rects(&svg);
    assert_eq!(4, bars.len());
    assert!(
        bars.iter().all(|r| r.3 > 0.0),
        "every bar has a height: {bars:?}"
    );
}

#[test]
fn table_sub_title_uses_its_own_alignment() {
    let mut chart = TableChart::new(vec![vec!["h".into()], vec!["r".into()]]);
    chart.title_text = "Title".into();
    chart.sub_title_text = "Sub".into();
    chart.sub_title_align = Align::Right;
    let right = chart.svg().unwrap();
    chart.sub_title_align = Align::Center;
    let center = chart.svg().unwrap();
    assert_ne!(right, center);
}

#[test]
fn gauge_label_shows_out_of_range_value() {
    let svg = GaugeChart::from_json(
        r#"{"min": 0, "max": 100, "series_list": [{"name": "Speed", "data": [250]}]}"#,
    )
    .unwrap()
    .svg()
    .unwrap();
    assert!(
        svg.lines().any(|l| l.trim() == "250"),
        "label keeps the raw value"
    );
}

#[test]
fn json_colors_are_validated() {
    // A bad color is an error rather than a silently transparent title.
    assert!(BarChart::from_json(r#"{"title_font_color": "red"}"#).is_err());
    assert!(BarChart::from_json(r##"{"series_colors": ["#f00", 5]}"##).is_err());
    // `null` entries keep their position (and the default color).
    let chart = BarChart::from_json(
        r##"{
            "series_colors": ["#f00", null, "#0f0"],
            "series_list": [{"name": "A", "data": [1, 2, 3], "colors": ["#f00", null, "#0f0"]}],
            "x_axis_data": ["a", "b", "c"]
        }"##,
    )
    .unwrap();
    assert_eq!(3, chart.series_colors.len());
    assert_eq!(Color::from("#0f0"), chart.series_colors[2]);
    assert_eq!(
        Some(vec![
            Some(Color::from("#f00")),
            None,
            Some(Color::from("#0f0"))
        ]),
        chart.series_list[0].colors
    );
}
