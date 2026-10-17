//! Plots that line up: a plot further in than its axes put it
//! (`grid_left`, `grid_right`), and the children of a multi chart one above
//! the other with their plots from the same left edge to the same right
//! edge (`align_plots`).
mod common;

use charts_rs::{BarChart, CandlestickChart, ChildChart, LineChart, MultiChart, PieChart};

fn attr(tag: &str, name: &str) -> Option<f32> {
    let key = format!(" {name}=\"");
    let start = tag.find(&key)? + key.len();
    tag[start..start + tag[start..].find('"')?].parse().ok()
}

/// The left and the right edge of the plot of a chart: those of its grid,
/// the horizontal lines that most of the lines across it are.
fn plot_edges(svg: &str) -> (f32, f32) {
    let mut counts: Vec<((f32, f32), usize)> = vec![];
    for tag in svg.split("<line").skip(1) {
        let tag = &tag[..tag.find('>').unwrap()];
        let (Some(x1), Some(y1), Some(x2), Some(y2)) = (
            attr(tag, "x1"),
            attr(tag, "y1"),
            attr(tag, "x2"),
            attr(tag, "y2"),
        ) else {
            continue;
        };
        if y1 != y2 || x2 - x1 < 50.0 {
            continue;
        }
        match counts.iter_mut().find(|(edges, _)| *edges == (x1, x2)) {
            Some((_, count)) => *count += 1,
            None => counts.push(((x1, x2), 1)),
        }
    }
    counts
        .into_iter()
        .max_by_key(|(_, count)| *count)
        .unwrap()
        .0
}

/// The plot edges of each child of a multi chart, where the multi chart
/// puts them.
fn children(svg: &str) -> Vec<(f32, f32)> {
    svg.split("<svg")
        .skip(2)
        .map(|child| {
            let head = &child[..child.find('>').unwrap()];
            let x = attr(head, "x").unwrap();
            let (left, right) = plot_edges(child);
            (x + left, x + right)
        })
        .collect()
}

const DAYS: &str = r#""x_axis_data": ["a", "b", "c", "d", "e"]"#;

fn bar(options: &str) -> String {
    BarChart::from_json(&format!(
        r#"{{{DAYS}, "series_list": [{{"name": "s", "data": [1200000, 2600000, 900000, 1800000, 2100000]}}]{options}}}"#
    ))
    .unwrap()
    .svg()
    .unwrap()
}

#[test]
fn a_plot_further_in_than_its_axes_put_it() {
    let (left, right) = plot_edges(&bar(""));
    // The plot starts after the labels of the y axis, and ends 5 pixels
    // from the right edge, the margin of the chart.
    assert!(
        left > 30.0 && left < 60.0 && right == 595.0,
        "{left} {right}"
    );
    // Further in where it is asked for: 100 from the left, 40 from the
    // right of a chart of 600.
    let moved = plot_edges(&bar(r#", "grid_left": 100, "grid_right": 40"#));
    assert_eq!((100.0, 560.0), moved);
    // Where the axes need more it is where they put it: the option is how
    // far in it is at the least, not where it is.
    assert_eq!(bar(""), bar(r#", "grid_left": 10, "grid_right": 2"#));
    // The y axis is where it was, its labels against the plot as they were:
    // what is added is room to the left of them.
    let svg = bar(r#", "grid_left": 100"#);
    let label = |svg: &str| -> f32 {
        let end = svg.find(">\n3.1M\n").unwrap();
        let tag = &svg[svg[..end].rfind("<text").unwrap()..end];
        attr(tag, "x").unwrap()
    };
    assert!(label(&svg) - label(&bar("")) > 40.0);

    // A line, a candlestick and a histogram chart as well; on a right axis
    // the room is to the right of it.
    let line = LineChart::from_json(&format!(
        r#"{{{DAYS}, "grid_left": 120, "grid_right": 30, "series_list": [{{"name": "s", "data": [1, 3, 2, 5, 4]}}]}}"#
    ))
    .unwrap()
    .svg()
    .unwrap();
    assert_eq!((120.0, 570.0), plot_edges(&line));
    let candles = CandlestickChart::from_json(&format!(
        r#"{{{DAYS}, "grid_left": 90, "series_list": [{{"name": "k", "data": [10, 12, 9, 13, 12, 11, 10, 14, 11, 14, 10, 15, 14, 16, 13, 17, 16, 15, 14, 18]}}]}}"#
    ))
    .unwrap()
    .svg()
    .unwrap();
    assert_eq!(90.0, plot_edges(&candles).0);
    let two_axes = LineChart::from_json(&format!(
        r#"{{{DAYS}, "grid_right": 120, "series_list": [{{"name": "s", "data": [1, 3, 2, 5, 4]}},
            {{"name": "t", "data": [100, 300, 200, 500, 400], "y_axis_index": 1}}]}}"#
    ))
    .unwrap()
    .svg()
    .unwrap();
    assert_eq!(480.0, plot_edges(&two_axes).1);
}

fn stack(align: bool, extra: &str) -> MultiChart {
    MultiChart::from_json(&format!(
        r#"{{"margin": 10, "gap": 10, "align_plots": {align}, "child_charts": [
            {{"type": "candlestick", "height": 200, {DAYS},
              "series_list": [{{"name": "k", "data": [10, 12, 9, 13, 12, 11, 10, 14, 11, 14, 10, 15, 14, 16, 13, 17, 16, 15, 14, 18]}}]}},
            {{"type": "bar", "height": 150, {DAYS}, "legend_show": false,
              "series_list": [{{"name": "v", "data": [1200000, 2600000, 900000, 1800000, 2100000]}}]}},
            {{"type": "line", "height": 150, {DAYS}, "legend_show": false,
              "series_list": [{{"name": "m", "data": [0.2, -0.4, 0.1, 0.6, -0.2]}}]}}{extra}
        ]}}"#
    ))
    .unwrap()
}

#[test]
fn the_plots_of_a_multi_chart_line_up() {
    // Each where its own axes put it: the volumes need the widest axis.
    let apart = children(&stack(false, "").svg().unwrap());
    assert_eq!(3, apart.len());
    assert!(
        apart[1].0 > apart[0].0 && apart[1].0 > apart[2].0,
        "{apart:?}"
    );

    // Lined up: all of them from the left edge of the one that starts
    // furthest in, to the right edge of the one that ends furthest in.
    let svg = stack(true, "").svg().unwrap();
    let lined = children(&svg);
    let left = apart.iter().map(|e| e.0).fold(f32::MIN, f32::max);
    let right = apart.iter().map(|e| e.1).fold(f32::MAX, f32::min);
    for edges in lined.iter() {
        assert!(
            (edges.0 - left).abs() < 0.1 && (edges.1 - right).abs() < 0.1,
            "{lined:?}"
        );
    }
    // And so the days are one above the other: the middle of the third
    // candle is where the third bar is.
    let size = |svg: &str| attr(&svg[..svg.find('>').unwrap()], "height").unwrap();
    assert_eq!(size(&stack(false, "").svg().unwrap()), size(&svg));
    common::assert_snapshot!(
        "multi_chart/aligned_json.svg",
        MultiChart::from_json(include_str!("../asset/multi_chart/aligned.json"))
            .unwrap()
            .svg()
            .unwrap()
    );
}

#[test]
fn what_lines_up_and_what_does_not() {
    // A child that wants its plot further in has it there, and the others
    // with it.
    let svg = stack(
        true,
        r#", {"type": "line", "height": 150, "grid_left": 150, "x_axis_data": ["a", "b"],
            "series_list": [{"name": "w", "data": [1, 2]}]}"#,
    )
    .svg()
    .unwrap();
    assert!(
        children(&svg).iter().all(|edges| edges.0 == 160.0),
        "{:?}",
        children(&svg)
    );

    // A child placed by hand keeps its own, and is not measured: it is
    // where it is without the others.
    let placed = r#", {"type": "line", "height": 150, "x": 700, "y": 10, "grid_left": 200,
        "x_axis_data": ["a", "b"], "series_list": [{"name": "w", "data": [1, 2]}]}"#;
    let lined = children(&stack(true, placed).svg().unwrap());
    let alone = children(&stack(true, "").svg().unwrap());
    assert_eq!(alone[..], lined[..3]);
    assert_eq!(900.0, lined[3].0);

    // Charts of other kinds are drawn as they would be.
    let mut multi = MultiChart::new();
    multi.align_plots = true;
    let pie = PieChart::from_json(
        r#"{"series_list": [{"name": "a", "data": [1]}, {"name": "b", "data": [2]}]}"#,
    )
    .unwrap();
    multi.add(ChildChart::Pie(pie.clone(), None));
    let aligned = multi.svg().unwrap();
    multi.align_plots = false;
    assert_eq!(multi.svg().unwrap(), aligned);

    // Children of other widths line up where they are on the picture.
    let mut multi = MultiChart::new();
    multi.align_plots = true;
    for width in [600.0, 400.0] {
        let mut chart = BarChart::from_json(&format!(
            r#"{{{DAYS}, "series_list": [{{"name": "s", "data": [1, 2, 3, 4, 5]}}]}}"#
        ))
        .unwrap();
        chart.width = width;
        multi.add(ChildChart::Bar(chart, None));
    }
    let edges = children(&multi.svg().unwrap());
    assert_eq!(edges[0], edges[1]);
    assert!(edges[0].1 < 410.0 + 0.1, "{edges:?}");
}

const POINTS: &str =
    r#""series_list": [{"name": "s", "data": [1, 120000, 2, 380000, 3, 250000, 4, 90000]}]"#;
const BOXES: &str = r#""x_axis_data": ["a", "b"],
    "box_series": [{"name": "b", "data": [[1, 2, 3, 4, 5], [2, 3, 4, 5, 6]]}]"#;
const STEPS: &str = r#""x_axis_data": ["a", "b", "c"], "data": [120000, -30000, 45000]"#;

fn scatter(options: &str) -> String {
    charts_rs::ScatterChart::from_json(&format!("{{{POINTS}{options}}}"))
        .unwrap()
        .svg()
        .unwrap()
}

fn box_plot(options: &str) -> String {
    charts_rs::BoxPlotChart::from_json(&format!("{{{BOXES}{options}}}"))
        .unwrap()
        .svg()
        .unwrap()
}

fn waterfall(options: &str) -> String {
    charts_rs::WaterfallChart::from_json(&format!("{{{STEPS}{options}}}"))
        .unwrap()
        .svg()
        .unwrap()
}

#[test]
fn scatter_box_plot_and_waterfall_charts_have_their_plots_further_in() {
    let further = r#", "grid_left": 110, "grid_right": 45"#;
    for (name, plain, moved) in [
        ("scatter", scatter(""), scatter(further)),
        ("box plot", box_plot(""), box_plot(further)),
        ("waterfall", waterfall(""), waterfall(further)),
    ] {
        let (left, right) = plot_edges(&plain);
        assert!(left < 110.0 && right > 555.0, "{name}: {left} {right}");
        assert_eq!((110.0, 555.0), plot_edges(&moved), "{name}");
        // Less than the axes need is what they need.
        let less = r#", "grid_left": 5, "grid_right": 1"#;
        let again = match name {
            "scatter" => scatter(less),
            "box plot" => box_plot(less),
            _ => waterfall(less),
        };
        assert_eq!(plain, again, "{name}");
    }

    // The histogram of a scatter chart to the right of its plot stays
    // against the plot: what is added is beyond it.
    let marginal = r#", "marginal": {"size": 40}"#;
    let (_, right) = plot_edges(&scatter(marginal));
    assert!((right - (600.0 - 5.0 - 40.0)).abs() < 1.0, "{right}");
    let svg = scatter(&format!(r#"{marginal}, "grid_right": 120"#));
    assert_eq!(480.0, plot_edges(&svg).1);
    let bars: Vec<f32> = svg
        .split("<rect")
        .skip(2)
        .filter_map(|tag| attr(&tag[..tag.find('>').unwrap()], "x"))
        .filter(|x| *x >= 480.0)
        .collect();
    assert!(
        !bars.is_empty() && bars.iter().all(|x| *x < 480.0 + 40.0),
        "{bars:?}"
    );

    // A hidden y axis stays hidden, however far in its plot starts: no text
    // is to the left of the plot, where the labels of the axis would be.
    let text_xs = |svg: &str| -> Vec<f32> {
        svg.split("<text")
            .skip(1)
            .filter_map(|tag| attr(&tag[..tag.find('>').unwrap()], "x"))
            .collect()
    };
    let hidden = waterfall(r#", "y_axis_hidden": true, "grid_left": 80"#);
    assert_eq!(80.0, plot_edges(&hidden).0);
    assert!(
        text_xs(&hidden).iter().all(|x| *x >= 79.0),
        "{:?}",
        text_xs(&hidden)
    );
    let shown = waterfall(r#", "grid_left": 80"#);
    assert!(text_xs(&shown).iter().any(|x| *x < 79.0));
}

#[test]
fn every_kind_of_plot_lines_up_in_a_multi_chart() {
    let child = |kind: &str, data: &str| {
        format!(r#"{{"type": "{kind}", "height": 160, "legend_show": false, {data}}}"#)
    };
    let json = |align: bool| {
        format!(
            r#"{{"align_plots": {align}, "child_charts": [{}, {}, {}, {}]}}"#,
            child(
                "bar",
                r#""x_axis_data": ["a", "b"], "series_list": [{"name": "s", "data": [1, 2]}]"#
            ),
            child("scatter", POINTS),
            child("box_plot", BOXES),
            child("waterfall", STEPS),
        )
    };
    let apart = children(&MultiChart::from_json(&json(false)).unwrap().svg().unwrap());
    assert_eq!(4, apart.len());
    assert!(
        apart.windows(2).any(|pair| pair[0].0 != pair[1].0),
        "{apart:?}"
    );
    let lined = children(&MultiChart::from_json(&json(true)).unwrap().svg().unwrap());
    let left = apart.iter().map(|e| e.0).fold(f32::MIN, f32::max);
    let right = apart.iter().map(|e| e.1).fold(f32::MAX, f32::min);
    for edges in lined.iter() {
        assert!(
            (edges.0 - left).abs() < 0.1 && (edges.1 - right).abs() < 0.1,
            "{lined:?}"
        );
    }
}
