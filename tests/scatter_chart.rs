mod common;

use charts_rs::ScatterChart;

#[test]
fn scatter_chart() {
    let scatter_chart = ScatterChart::from_json(
        r###"{
            "width": 630,
            "height": 410,
            "margin": {
                "left": 10,
                "top": 5,
                "right": 20
            },
            "title_text": "Male and female height and weight distribution",
            "title_align": "left",
            "sub_title_text": "Data from: Heinz 2003",
            "sub_title_align": "left",
            "legend_align": "right",
            "y_axis_configs": [
                {
                    "axis_min": 40,
                    "axis_max": 130,
                    "axis_formatter": "{c} kg"
                }
            ],
            "x_axis_config": {
                "axis_min": 140,
                "axis_max": 230,
                "axis_formatter": "{c} cm"
            },
            "series_list": [
                {
                    "name": "Female",
                    "data": [
                        161.2, 51.6, 167.5, 59.0, 159.5, 49.2, 157.0, 63.0, 155.8, 53.6, 170.0, 59.0,
                        159.1, 47.6, 166.0, 69.8, 176.2, 66.8, 160.2, 75.2, 172.5, 55.2, 170.9, 54.2,
                        172.9, 62.5, 153.4, 42.0, 160.0, 50.0, 147.2, 49.8, 168.2, 49.2, 175.0, 73.2,
                        157.0, 47.8, 167.6, 68.8, 159.5, 50.6, 175.0, 82.5, 166.8, 57.2, 176.5, 87.8,
                        170.2, 72.8
                    ]
                },
                {
                    "name": "Male",
                    "data": [
                        174.0, 65.6, 175.3, 71.8, 193.5, 80.7, 186.5, 72.6, 187.2, 78.8, 181.5, 74.8,
                        184.0, 86.4, 184.5, 78.4, 175.0, 62.0, 184.0, 81.6, 180.0, 76.6, 177.8, 83.6,
                        192.0, 90.0, 176.0, 74.6, 174.0, 71.0, 184.0, 79.6, 192.7, 93.8, 171.5, 70.0,
                        173.0, 72.4, 176.0, 85.9, 176.0, 78.8, 180.5, 77.8, 172.7, 66.2, 176.0, 86.4,
                        173.5, 81.8
                    ]
                }
            ],
            "series_symbol_sizes": [6, 6]
        }"###).unwrap();

    common::assert_snapshot!("scatter_chart/basic_json.svg", scatter_chart.svg().unwrap());
}

#[test]
fn scatter_series_symbols() {
    let chart = |symbols: &str| {
        ScatterChart::from_json(&format!(
            r##"{{"series_symbols": {symbols}, "series_list": [
                {{"name": "a", "data": [1, 2, 3, 4]}},
                {{"name": "b", "data": [2, 3, 4, 5]}}
            ]}}"##
        ))
        .unwrap()
        .svg()
        .unwrap()
    };
    // By its type alone, or as an object with a size and a color: the same
    // shapes either way.
    let names = chart(r#"["triangle", "diamond"]"#);
    let objects = chart(r#"[{"type": "triangle"}, {"type": "Diamond"}]"#);
    assert_eq!(names, objects);
    // Two triangles and two diamonds, no circle for the points.
    assert_eq!(4, names.matches("<polygon").count());
    assert_eq!(0, names.matches(r#" r="3""#).count());

    // The size is `series_symbol_sizes`, not the one of the symbol; an
    // entry that is no symbol is skipped.
    let plain = chart(r#"["rect", "diamond"]"#);
    let sized = chart(r#"[{"type": "rect", "size": 7}, null, 5, "diamond"]"#);
    assert_eq!(plain, sized);
    assert_ne!(plain, names);
    // A color of its own instead of the color of the series.
    let colored = chart(r##"[{"type": "rect", "color": "#123456"}, "diamond"]"##);
    assert_eq!(2, colored.matches(r##"fill="#123456""##).count());
    assert!(!plain.contains("#123456"));
}

/// `(cx, cy)` of the circles of a series.
fn circles(svg: &str, series: &str) -> Vec<(f32, f32)> {
    let number = |tag: &str, name: &str| -> f32 {
        let key = format!(" {name}=\"");
        let start = tag.find(&key).unwrap() + key.len();
        tag[start..start + tag[start..].find('"').unwrap()]
            .parse()
            .unwrap()
    };
    svg.split("<circle")
        .skip(1)
        .filter(|t| t.contains(&format!("data-series=\"{series}\"")))
        .map(|t| (number(t, "cx"), number(t, "cy")))
        .collect()
}

/// The points of every fitted curve of the svg.
fn curves(svg: &str) -> Vec<Vec<(f32, f32)>> {
    svg.split("<path d=\"M ")
        .skip(1)
        .filter(|t| t[..t.find("/>").unwrap()].contains("ct-regression"))
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

#[test]
fn scatter_regression() {
    let chart =
        ScatterChart::from_json(include_str!("../asset/scatter_chart/regression.json")).unwrap();
    assert_eq!(Some(charts_rs::Regression::Polynomial), chart.regression);
    common::assert_snapshot!("scatter_chart/regression_json.svg", chart.svg().unwrap());

    let json = |extra: &str| {
        format!(
            r##"{{"legend_show": false, "series_symbols": ["circle", "circle"], "series_list": [
                {{"name": "a", "data": [1, 2, 2, 4, 4, 8, 6, 12]}},
                {{"name": "b", "data": [1, 9, 3, 7, 5, 5]}}
            ]{extra}}}"##
        )
    };
    let plain = ScatterChart::from_json(&json("")).unwrap().svg().unwrap();
    assert!(curves(&plain).is_empty());

    // Points on a line: the fitted line runs from the first to the last.
    let svg = ScatterChart::from_json(&json(
        r#", "regression": "linear", "regression_label_show": true"#,
    ))
    .unwrap()
    .svg()
    .unwrap();
    let lines = curves(&svg);
    assert_eq!(2, lines.len());
    for (line, series) in lines.iter().zip(["a", "b"]) {
        let points = circles(&svg, series);
        assert_eq!(2, line.len());
        let (first, last) = (points[0], points[points.len() - 1]);
        assert!(
            (line[0].0 - first.0).abs() < 0.2 && (line[0].1 - first.1).abs() < 0.2,
            "{line:?}"
        );
        assert!(
            (line[1].0 - last.0).abs() < 0.2 && (line[1].1 - last.1).abs() < 0.2,
            "{line:?}"
        );
    }
    // In the color of its series, with its formula.
    assert!(svg.contains(r##"stroke="#5470C6" class="ct-regression""##));
    assert!(
        svg.contains("\ny = 2x\n") || svg.contains("\ny = 2x + 0\n"),
        "{svg}"
    );
    assert!(svg.contains("\ny = -x + 10\n"));
    // The points are where they were.
    assert_eq!(circles(&plain, "a"), circles(&svg, "a"));

    // A curve is drawn in short pieces, and stays on the plot.
    let svg = ScatterChart::from_json(&json(
        r#", "regression": "Exponential", "y_axis_configs": [{"axis_max": 6}]"#,
    ))
    .unwrap()
    .svg()
    .unwrap();
    let curve = &curves(&svg)[0];
    assert!(curve.len() > 10 && curve.len() < 65, "{}", curve.len());
    let top = curve.iter().map(|p| p.1).fold(f32::MAX, f32::min);
    assert!(top >= 0.0, "{top}");
    // A line that leaves the plot is cut where it does.
    let svg = ScatterChart::from_json(&json(
        r#", "regression": "linear", "y_axis_configs": [{"axis_max": 6}]"#,
    ))
    .unwrap()
    .svg()
    .unwrap();
    let line = &curves(&svg)[0];
    assert_eq!(2, line.len());
    let lowest = circles(&svg, "a")[0];
    assert!((line[0].1 - lowest.1).abs() < 0.2);
    assert!(line[1].1 < 50.0 && line[1].1 >= 0.0, "{line:?}");

    // A series of one point has no curve; the others keep theirs.
    let svg = ScatterChart::from_json(
        r#"{"regression": "polynomial", "regression_order": 3, "series_list": [
            {"name": "a", "data": [1, 2]},
            {"name": "b", "data": [1, 1, 2, 8, 3, 27, 4, 64, 5, 125]}
        ]}"#,
    )
    .unwrap()
    .svg()
    .unwrap();
    assert_eq!(1, curves(&svg).len());
    assert_eq!(65, curves(&svg)[0].len());

    let message = match ScatterChart::from_json(&json(r#", "regression": "cubic""#)) {
        Ok(_) => panic!("accepted"),
        Err(e) => e.to_string(),
    };
    assert!(
        message.contains("linear, exponential, logarithmic, polynomial"),
        "{message}"
    );
}

#[derive(Debug)]
struct MarginalBar {
    series: String,
    axis: String,
    from: f32,
    to: f32,
    value: u32,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
}

fn attr(tag: &str, name: &str) -> String {
    let key = format!(" {name}=\"");
    let start = tag.find(&key).unwrap() + key.len();
    tag[start..start + tag[start..].find('"').unwrap()].to_string()
}

/// The histogram bars, and the points as `(series, x, y, cx, cy)`.
fn marginal_bars(svg: &str) -> Vec<MarginalBar> {
    svg.split("<rect")
        .skip(1)
        .filter(|tag| tag.contains("ct-marginal"))
        .map(|tag| MarginalBar {
            series: attr(tag, "data-series"),
            axis: attr(tag, "data-axis"),
            from: attr(tag, "data-from").parse().unwrap(),
            to: attr(tag, "data-to").parse().unwrap(),
            value: attr(tag, "data-value").parse().unwrap(),
            x: attr(tag, "x").parse().unwrap(),
            y: attr(tag, "y").parse().unwrap(),
            w: attr(tag, "width").parse().unwrap(),
            h: attr(tag, "height").parse().unwrap(),
        })
        .collect()
}

fn scatter_points(svg: &str) -> Vec<(String, f32, f32, f32, f32)> {
    svg.split("<circle")
        .skip(1)
        .filter(|tag| tag.contains("data-series="))
        .map(|tag| {
            (
                attr(tag, "data-series"),
                attr(tag, "data-x").parse().unwrap(),
                attr(tag, "data-y").parse().unwrap(),
                attr(tag, "cx").parse().unwrap(),
                attr(tag, "cy").parse().unwrap(),
            )
        })
        .collect()
}

fn marginal_svg() -> String {
    ScatterChart::from_json(
        r#"{
            "width": 420,
            "height": 320,
            "x_axis_config": {"axis_min": 0, "axis_max": 4},
            "y_axis_configs": [{"axis_min": 0, "axis_max": 4}],
            "marginal": {"bin_count": 4, "bar_gap": 0, "size": 60},
            "series_symbols": ["circle", "circle"],
            "series_symbol_sizes": [4, 4],
            "series_list": [
                {"name": "A", "data": [0.5, 0.5, 0.6, 0.7, 1.5, 1.5, 2.5, 2.5, 3.5, 3.5]},
                {"name": "B", "data": [0.5, 3.5]}
            ]
        }"#,
    )
    .unwrap()
    .svg()
    .unwrap()
}

/// The bar of `series` on `axis` whose bin holds `value`.
fn bar_over<'a>(bars: &'a [MarginalBar], series: &str, axis: &str, value: f32) -> &'a MarginalBar {
    bars.iter()
        .find(|bar| bar.series == series && bar.axis == axis && bar.from < value && value < bar.to)
        .unwrap_or_else(|| panic!("{series} {axis} has no bar over {value}: {bars:?}"))
}

#[test]
fn scatter_marginal_histogram() {
    let svg = marginal_svg();
    let bars = marginal_bars(&svg);
    let points = scatter_points(&svg);
    assert_eq!(6, points.len(), "{points:?}");

    // Each point sits in the bar that counts it, on both axes.
    for (series, x, y, cx, cy) in &points {
        let across = bar_over(&bars, series, "x", *x);
        let up = bar_over(&bars, series, "y", *y);
        assert!(
            cx + 0.6 >= across.x && *cx <= across.x + across.w + 0.6,
            "{series} ({x}, {y}) at {cx} is outside its x bar {across:?}"
        );
        assert!(
            cy + 0.6 >= up.y && *cy <= up.y + up.h + 0.6,
            "{series} ({x}, {y}) at {cy} is outside its y bar {up:?}"
        );
    }

    // The x bars are above the plot and the y bars are to its right.
    let above = bars
        .iter()
        .filter(|bar| bar.axis == "x")
        .map(|bar| bar.y + bar.h)
        .fold(0.0_f32, f32::max);
    let beside = bars
        .iter()
        .filter(|bar| bar.axis == "y")
        .map(|bar| bar.x)
        .fold(f32::MAX, f32::min);
    for (_, _, _, cx, cy) in &points {
        assert!(
            *cy + 0.6 >= above,
            "point {cy} is above the histogram {above}"
        );
        assert!(
            *cx <= beside + 0.6,
            "point {cx} is right of the histogram {beside}"
        );
    }

    // A's first x bin holds two points, B's holds one, and the bars share
    // a scale, so A's bar is twice as tall.
    let tall = bars
        .iter()
        .find(|bar| bar.series == "A" && bar.axis == "x" && bar.value == 2)
        .unwrap();
    let short = bars
        .iter()
        .find(|bar| bar.series == "B" && bar.axis == "x" && bar.value == 1)
        .unwrap();
    assert!(
        (tall.h / short.h - 2.0).abs() < 0.05,
        "{} {}",
        tall.h,
        short.h
    );
    // Two series are drawn over each other.
    assert!(svg.contains(r#"fill-opacity="0.55""#), "{svg}");
}

#[test]
fn scatter_marginal_ignores_what_is_not_a_point() {
    // A coordinate past the axis, a missing one, and a bubble's size.
    let outside = ScatterChart::from_json(
        r#"{
            "x_axis_config": {"axis_min": 0, "axis_max": 4},
            "y_axis_configs": [{"axis_min": 0, "axis_max": 4}],
            "marginal": {"bin_count": 4, "bar_gap": 0},
            "series_list": [
                {"name": "C", "data": [10, 0.5, 1.5, null]},
                {"name": "D", "data": [null, 1, 2.5, 2.5]}
            ]
        }"#,
    )
    .unwrap()
    .svg()
    .unwrap();
    let bars = marginal_bars(&outside);
    let values = |series: &str, axis: &str| -> u32 {
        bars.iter()
            .filter(|bar| bar.series == series && bar.axis == axis)
            .map(|bar| bar.value)
            .sum()
    };
    // 10 is outside the x axis; the following pair is missing its y.
    assert_eq!(0, values("C", "x"));
    assert_eq!(1, values("C", "y"));
    // The missing x leaves one point.
    assert_eq!(1, values("D", "x"));
    assert_eq!(1, values("D", "y"));

    let bubbles = ScatterChart::from_json(
        r#"{
            "bubble": true,
            "x_axis_config": {"axis_min": 0, "axis_max": 4},
            "y_axis_configs": [{"axis_min": 0, "axis_max": 4}],
            "marginal": {"bin_count": 4, "bar_gap": 0},
            "series_list": [{"name": "A", "data": [0.5, 0.5, 100, 1.5, 1.5, 1]}]
        }"#,
    )
    .unwrap()
    .svg()
    .unwrap();
    let bars = marginal_bars(&bubbles);
    let counted: u32 = bars
        .iter()
        .filter(|bar| bar.axis == "x")
        .map(|bar| bar.value)
        .sum();
    assert_eq!(2, counted, "{bars:?}");
}

#[test]
fn scatter_marginal_follows_the_scale() {
    for (scale, inverse) in [("linear", true), ("log", false)] {
        let svg = ScatterChart::from_json(&format!(
            r#"{{
                "x_axis_config": {{"axis_min": 1, "axis_max": 100, "axis_scale": "{scale}", "axis_inverse": {inverse}}},
                "y_axis_configs": [{{"axis_min": 1, "axis_max": 100, "axis_scale": "{scale}", "axis_inverse": {inverse}}}],
                "marginal": {{"bin_count": 2, "bar_gap": 0, "size": 50}},
                "series_symbol_sizes": [4],
                "series_list": [{{"name": "A", "data": [2, 3, 80, 60]}}]
            }}"#
        ))
        .unwrap()
        .svg()
        .unwrap();
        let bars = marginal_bars(&svg);
        for (series, x, y, cx, cy) in scatter_points(&svg) {
            let across = bar_over(&bars, &series, "x", x);
            let up = bar_over(&bars, &series, "y", y);
            assert!(
                cx + 0.6 >= across.x && cx <= across.x + across.w + 0.6,
                "{scale} inverse={inverse}: {x} at {cx} outside {across:?}"
            );
            assert!(
                cy + 0.6 >= up.y && cy <= up.y + up.h + 0.6,
                "{scale} inverse={inverse}: {y} at {cy} outside {up:?}"
            );
        }
    }
}

#[test]
fn scatter_marginal_small_chart_draws_none() {
    let svg = ScatterChart::from_json(
        r#"{"width": 30, "height": 30, "marginal": {}, "series_list": [{"name": "a", "data": [1, 2, 3, 4]}]}"#,
    )
    .unwrap()
    .svg()
    .unwrap();
    assert!(!svg.contains("ct-marginal"), "{svg}");
    assert!(svg.starts_with("<svg"));

    let message = ScatterChart::from_json(r#"{"marginal": {"bins": 4}, "series_list": []}"#)
        .unwrap_err()
        .to_string();
    assert!(message.contains("marginal.bins"), "{message}");
}

#[test]
fn scatter_marginal_snapshot() {
    let chart =
        ScatterChart::from_json(include_str!("../asset/scatter_chart/marginal.json")).unwrap();
    common::assert_snapshot!("scatter_chart/marginal.svg", chart.svg().unwrap());
}
