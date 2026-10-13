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

struct ContourLine {
    level: usize,
    closed: bool,
    points: Vec<(f32, f32)>,
    opacity: String,
}

fn attr_str(tag: &str, name: &str) -> String {
    let key = format!(" {name}=\"");
    let start = tag.find(&key).unwrap() + key.len();
    tag[start..start + tag[start..].find('"').unwrap()].to_string()
}

/// The density lines of the svg, in the order they were drawn.
fn contour_lines(svg: &str) -> Vec<ContourLine> {
    svg.split("<path")
        .skip(1)
        .filter(|tag| tag[..tag.find("/>").unwrap()].contains("ct-contour"))
        .map(|tag| {
            let head = &tag[..tag.find("/>").unwrap()];
            let d = attr_str(head, "d");
            let closed = d.ends_with(" Z");
            let mut points = Vec::new();
            let mut tokens = d.trim_end_matches(" Z").split_whitespace();
            while let Some(command) = tokens.next() {
                let (Some(x), Some(y)) = (tokens.next(), tokens.next()) else {
                    break;
                };
                assert!(command == "M" || command == "L", "{command}");
                points.push((x.parse().unwrap(), y.parse().unwrap()));
            }
            let class = attr_str(head, "class");
            let level = class
                .split_whitespace()
                .find_map(|token| token.strip_prefix('l'))
                .unwrap()
                .parse()
                .unwrap();
            ContourLine {
                level,
                closed,
                points,
                opacity: attr_str(head, "stroke-opacity"),
            }
        })
        .collect()
}

fn contour_paths(svg: &str) -> Vec<String> {
    contour_lines(svg)
        .into_iter()
        .map(|line| {
            line.points
                .iter()
                .map(|(x, y)| format!("{x},{y}"))
                .collect()
        })
        .collect()
}

/// Even-odd test. The line is closed by the caller, the way the svg `Z` is.
fn encloses(points: &[(f32, f32)], x: f32, y: f32) -> bool {
    let mut crossings = 0;
    for index in 0..points.len() {
        let (x1, y1) = points[index];
        let (x2, y2) = points[(index + 1) % points.len()];
        if (y1 > y) != (y2 > y) {
            let at = x1 + (y - y1) * (x2 - x1) / (y2 - y1);
            if x < at {
                crossings += 1;
            }
        }
    }
    crossings % 2 == 1
}

fn contour_chart(extra: &str, data: &str) -> String {
    ScatterChart::from_json(&format!(
        r#"{{
            "width": 420,
            "height": 320,
            "x_axis_config": {{"axis_min": 0, "axis_max": 10}},
            "y_axis_configs": [{{"axis_min": 0, "axis_max": 10}}],
            "series_symbol_sizes": [4, 4],
            "series_symbols": ["circle", "circle"],
            {extra}
            "series_list": [{data}]
        }}"#
    ))
    .unwrap()
    .svg()
    .unwrap()
}

const CLUSTER: &str = "4.6, 5.2, 5.1, 4.7, 5.4, 5.0, 4.8, 5.5, 5.2, 4.4, 4.9, 5.3, 5.5, 4.8, 4.7, 5.1, 5.0, 4.6, 5.3, 5.4";

#[test]
fn scatter_contour_lines_sit_under_the_points() {
    let svg = contour_chart(
        r#""contour": {},"#,
        &format!(r#"{{"name": "A", "data": [{CLUSTER}]}}"#),
    );
    let lines = contour_lines(&svg);
    let tag = svg
        .split("<path")
        .find(|tag| tag.contains("ct-contour"))
        .unwrap();
    assert!(tag.contains("stroke-width=\"1.5\""));
    assert!(tag.contains("fill=\"none\""));
    assert!(lines.iter().all(|line| line.opacity == "0.82"));
    let mut levels: Vec<_> = lines.iter().map(|line| line.level).collect();
    levels.sort_unstable();
    levels.dedup();
    assert_eq!(vec![1, 2, 3, 4], levels);
    // The lines are written before the points.
    assert!(svg.find("ct-contour").unwrap() < svg.find("data-series=\"").unwrap());
    let outer = lines
        .iter()
        .find(|line| line.level == 1 && line.closed)
        .unwrap();
    for point in circles(&svg, "A") {
        assert!(encloses(&outer.points, point.0, point.1), "{point:?}");
    }

    let overlaid = contour_chart(
        r#""contour": {},"#,
        &format!(
            r#"{{"name": "A", "data": [{CLUSTER}]}}, {{"name": "B", "data": [1.2, 1.4, 1.5, 1.1, 1.8, 1.6, 1.3, 1.9, 1.6, 1.2, 1.4, 1.7, 1.1, 1.5, 1.7, 1.3, 1.9, 1.8, 1.2, 1.6]}}"#
        ),
    );
    assert!(
        contour_lines(&overlaid)
            .iter()
            .all(|line| line.opacity == "0.55")
    );

    // A second series too small to draw leaves the first at full strength.
    let single = contour_chart(
        r#""contour": {},"#,
        &format!(r#"{{"name": "A", "data": [{CLUSTER}]}}, {{"name": "B", "data": [1, 1, 2, 2]}}"#),
    );
    assert!(
        contour_lines(&single)
            .iter()
            .all(|line| line.opacity == "0.82")
    );
}

#[test]
fn scatter_contour_skips_what_is_not_on_the_axes() {
    let bare = contour_chart("", &format!(r#"{{"name": "A", "data": [{CLUSTER}]}}"#));
    assert!(!bare.contains("ct-contour"));

    // Seven points on the axes, plus a missing one and one outside it.
    let seven = "4.6, 5.2, 5.1, 4.7, 5.4, 5.0, 4.8, 5.5, 5.2, 4.4, 4.9, 5.3, 5.5, 4.8";
    let short = contour_chart(
        r#""contour": {},"#,
        &format!(r#"{{"name": "A", "data": [{seven}, null, null, 30, 30]}}"#),
    );
    assert!(!short.contains("ct-contour"));

    let plain = contour_chart(
        r#""contour": {"grid_size": 32},"#,
        &format!(r#"{{"name": "A", "data": [{CLUSTER}]}}"#),
    );
    let junk = contour_chart(
        r#""contour": {"grid_size": 32},"#,
        &format!(r#"{{"name": "A", "data": [{CLUSTER}, null, 1, 30, 30, null, null]}}"#),
    );
    assert_eq!(contour_paths(&plain), contour_paths(&junk));

    let triples: Vec<String> = CLUSTER
        .split(", ")
        .collect::<Vec<_>>()
        .chunks(2)
        .enumerate()
        .map(|(i, pair)| format!("{}, {}, {}", pair[0], pair[1], 10 + i * 17))
        .collect();
    let bubbles = contour_chart(
        r#""bubble": true, "contour": {"grid_size": 32},"#,
        &format!(r#"{{"name": "A", "data": [{}]}}"#, triples.join(", ")),
    );
    assert_eq!(contour_paths(&plain), contour_paths(&bubbles));
}

#[test]
fn scatter_contour_follows_the_scale() {
    let data = "8, 9, 9, 11, 10, 10, 12, 8, 11, 12, 8, 10, 10, 9, 12, 11, 9, 8, 11, 10";
    for (scale, inverse, min, max) in [
        ("linear", true, 0, 20),
        ("log", false, 1, 100),
        ("log", true, 1, 100),
    ] {
        let svg = ScatterChart::from_json(&format!(
            r#"{{
                "width": 420,
                "height": 320,
                "x_axis_config": {{"axis_min": {min}, "axis_max": {max}, "axis_scale": "{scale}", "axis_inverse": {inverse}}},
                "y_axis_configs": [{{"axis_min": {min}, "axis_max": {max}, "axis_scale": "{scale}", "axis_inverse": {inverse}}}],
                "contour": {{}},
                "series_symbol_sizes": [4],
                "series_list": [{{"name": "A", "data": [{data}]}}]
            }}"#
        ))
        .unwrap()
        .svg()
        .unwrap();
        let outer = contour_lines(&svg)
            .into_iter()
            .find(|line| line.level == 1 && line.closed)
            .unwrap_or_else(|| panic!("{scale} inverse={inverse} has no outer line"));
        let point = svg
            .split("<circle")
            .skip(1)
            .find(|tag| tag.contains("data-x=\"10\"") && tag.contains("data-y=\"10\""))
            .unwrap();
        let (x, y): (f32, f32) = (
            attr_str(point, "cx").parse().unwrap(),
            attr_str(point, "cy").parse().unwrap(),
        );
        assert!(
            encloses(&outer.points, x, y),
            "{scale} inverse={inverse}: (10, 10) at ({x}, {y}) is outside the contour"
        );
    }
}

#[test]
fn scatter_contour_options_and_unknown_keys() {
    let chart =
        ScatterChart::from_json(r#"{"contour": {"levels": 0, "grid_size": 0}, "series_list": []}"#)
            .unwrap();
    let contour = chart.contour.unwrap();
    assert_eq!(4, contour.levels);
    assert_eq!(64, contour.grid_size);

    let chart = ScatterChart::from_json(
        r#"{"contour": {"levels": 100, "grid_size": 3}, "series_list": []}"#,
    )
    .unwrap();
    let contour = chart.contour.unwrap();
    assert_eq!(8, contour.levels);
    assert_eq!(16, contour.grid_size);

    let message = ScatterChart::from_json(r#"{"contour": {"bandwidth": 1}, "series_list": []}"#)
        .unwrap_err()
        .to_string();
    assert!(message.contains("contour.bandwidth"), "{message}");
}

#[test]
fn scatter_contour_snapshot() {
    let chart =
        ScatterChart::from_json(include_str!("../asset/scatter_chart/contour.json")).unwrap();
    common::assert_snapshot!("scatter_chart/contour.svg", chart.svg().unwrap());
}
