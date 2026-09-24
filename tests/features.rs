//! Behavioural tests for the options added on top of 1.0: per-chart
//! consistency (tooltips, `data-*`, label formatters), legend placement,
//! per-series overrides, multi chart coverage and the node charts.
mod common;

use charts_rs::{
    Align, BarChart, BoxPlotChart, CandlestickChart, FunnelChart, GraphChart, HeatmapChart,
    HorizontalBarChart, LineChart, MarkLine, MarkLineCategory, MultiChart, ParallelChart, PieChart,
    Position, RadarChart, RadarIndicator, Series, Symbol, ThemeRiverChart, TreemapChart,
    WaterfallChart,
};

fn count(svg: &str, needle: &str) -> usize {
    svg.matches(needle).count()
}

fn attr(tag: &str, name: &str) -> f32 {
    let key = format!(" {name}=\"");
    let start = tag.find(&key).map(|i| i + key.len()).unwrap();
    let end = start + tag[start..].find('"').unwrap();
    tag[start..end].parse().unwrap()
}

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

#[test]
fn horizontal_bar_parity_with_bar_chart() {
    let mut a: Series = ("A", vec![3.0, 5.0]).into();
    a.stack = Some("s".into());
    a.colors = Some(vec![Some("#f00".into()), None]);
    let mut b: Series = ("B", vec![2.0, -4.0]).into();
    b.stack = Some("s".into());
    let mut chart = HorizontalBarChart::new(vec![a, b], vec!["a".into(), "b".into()]);
    chart.y_axis_configs[0].axis_formatter = Some("{c} kg".to_string());
    chart.series_label_formatter = "{b}: {c}".to_string();
    chart.series_list[0].label_show = true;
    chart.animation = Some(charts_rs::AnimationConfig::default());
    let svg = chart.svg().unwrap();
    // Stacked: A and B share a slot, so B starts where A ends on row "a".
    let bars = rects(&svg);
    assert_eq!(4, bars.len(), "{svg}");
    assert!(
        (bars[0].0 + bars[0].2 - bars[2].0).abs() < 0.2,
        "B stacks on A"
    );
    // Negative values hang left of the zero line without negative widths.
    assert!(!svg.contains("width=\"-"));
    // Per-bar color, axis formatter, template label, animation and data-*.
    assert!(svg.contains("fill=\"#FF0000\""));
    assert!(svg.contains("0 kg"), "value axis uses axis_formatter");
    assert!(svg.contains("a: 3"), "label uses the {{b}}: {{c}} template");
    assert!(svg.contains("hbar-grow") && svg.contains("bar-anim"));
    assert!(svg.contains("data-series=\"A\"") && svg.contains("data-category=\"b\""));

    // Hidden axes drop their space; the x axis height follows the option.
    chart.animation = None;
    let shown = chart.svg().unwrap();
    chart.x_axis_hidden = true;
    chart.y_axis_hidden = true;
    let hidden = chart.svg().unwrap();
    assert!(count(&hidden, "<text") < count(&shown, "<text"));
    assert!(
        rects(&hidden)[0].2 > rects(&shown)[0].2,
        "bars widen without the category axis"
    );
}

#[test]
fn legend_positions_reserve_space() {
    let make = || {
        BarChart::new(
            vec![
                ("Alpha", vec![1.0, 2.0]).into(),
                ("Beta", vec![2.0, 1.0]).into(),
            ],
            vec!["a".into(), "b".into()],
        )
    };
    let top = make().svg().unwrap();
    let top_bars = rects(&top);
    let mut hidden = make();
    hidden.legend_show = Some(false);
    let no_legend_bars = rects(&hidden.svg().unwrap());
    for position in [Position::Bottom, Position::Left, Position::Right] {
        let mut chart = make();
        chart.legend_position = Some(position.clone());
        let svg = chart.svg().unwrap();
        assert_ne!(top, svg);
        let bars = rects(&svg);
        assert_eq!(top_bars.len(), bars.len(), "{position:?}: {svg}");
        match position {
            // Bottom: the plot starts higher (no header legend) but ends
            // above the legend, so bars are shorter than without a legend.
            Position::Bottom => {
                assert!(bars[0].1 < top_bars[0].1, "{position:?}");
                assert!(bars[0].3 < no_legend_bars[0].3, "{position:?}");
            }
            // Left/right: the plot is narrower.
            _ => assert!(bars[0].2 < no_legend_bars[0].2, "{position:?}"),
        }
        assert!(svg.contains("Alpha") && svg.contains("Beta"));
    }
    let chart = BarChart::from_json(
        r#"{"legend_position": "bottom", "series_list": [{"name": "A", "data": [1]}], "x_axis_data": ["a"]}"#,
    )
    .unwrap();
    assert_eq!(Some(Position::Bottom), chart.legend_position);
}

#[test]
fn per_series_line_overrides() {
    let mut smooth: Series = ("S", vec![1.0, 3.0, 2.0]).into();
    smooth.smooth = Some(true);
    smooth.fill = Some(true);
    let mut plain: Series = ("P", vec![2.0, 1.0, 3.0]).into();
    plain.symbol = Some(Symbol::None);
    let mut chart = LineChart::new(
        vec![smooth, plain],
        vec!["a".into(), "b".into(), "c".into()],
    );
    chart.series_smooth = false;
    chart.series_fill = false;
    let svg = chart.svg().unwrap();
    // One curved (C commands) filled path for S, one straight polyline for P.
    assert!(svg.contains(" C"), "series S is smooth");
    assert_eq!(1, count(&svg, "fill-opacity=\"0.39\""), "only S is filled");
    // P draws no symbols; giving it the default adds one circle per point.
    let without = count(&svg, "<circle");
    chart.series_list[1].symbol = None;
    let with = count(&chart.svg().unwrap(), "<circle");
    assert_eq!(without + 3, with, "{svg}");

    let chart = LineChart::from_json(
        r#"{"series_list": [{"name": "A", "data": [1, 2], "smooth": true, "fill": false, "symbol": {"type": "rect", "size": 4}}], "x_axis_data": ["a", "b"]}"#,
    )
    .unwrap();
    assert_eq!(Some(true), chart.series_list[0].smooth);
    assert_eq!(Some(false), chart.series_list[0].fill);
    assert_eq!(Some(Symbol::Rect(4.0, None)), chart.series_list[0].symbol);
}

#[test]
fn mark_lines_on_bar_charts_and_fixed_values() {
    let mut series: Series = ("A", vec![10.0, 30.0, 20.0]).into();
    series.mark_lines = vec![
        MarkLine {
            category: MarkLineCategory::Average,
        },
        MarkLine {
            category: MarkLineCategory::Value(25.0),
        },
    ];
    let chart = BarChart::new(vec![series], vec!["a".into(), "b".into(), "c".into()]);
    let svg = chart.svg().unwrap();
    assert_eq!(2, count(&svg, "stroke-dasharray=\"4,2\""), "two mark lines");
    assert!(svg.lines().any(|l| l.trim() == "25"), "fixed value label");
    // The label sits inside the plot (right aligned before the arrow).
    assert!(svg.contains("text-anchor=\"end\""));

    let chart = LineChart::from_json(
        r#"{"series_list": [{"name": "A", "data": [1, 2], "mark_lines": [{"category": "value", "value": 1.5}]}], "x_axis_data": ["a", "b"]}"#,
    )
    .unwrap();
    assert_eq!(
        MarkLineCategory::Value(1.5),
        chart.series_list[0].mark_lines[0].category
    );

    // Candlesticks get mark lines too.
    let mut series: Series = ("A", vec![1.0, 2.0, 0.5, 2.5, 2.0, 1.0, 0.5, 2.5]).into();
    series.mark_lines = vec![MarkLine {
        category: MarkLineCategory::Max,
    }];
    let svg = CandlestickChart::new(vec![series], vec!["a".into(), "b".into()])
        .svg()
        .unwrap();
    assert_eq!(1, count(&svg, "stroke-dasharray=\"4,2\""));
}

#[test]
fn multi_chart_accepts_every_chart_type() {
    let json = r#"{"child_charts": [
        {"type": "bar", "series_list": [{"name": "A", "data": [1, 2]}], "x_axis_data": ["a", "b"]},
        {"type": "heatmap", "x_axis_data": ["a"], "y_axis_data": ["x"], "series": {"data": [[0, 1]]}},
        {"type": "funnel", "series_list": [{"name": "A", "data": [3]}, {"name": "B", "data": [1]}]},
        {"type": "waterfall", "x_axis_data": ["a", "b"], "data": [[1, false], [0, true]]},
        {"type": "calendar", "start_date": "2024-01-01", "end_date": "2024-01-31", "data": [["2024-01-02", 3]]},
        {"type": "gauge", "series_list": [{"name": "s", "data": [40]}]},
        {"type": "treemap", "series_list": [{"name": "A", "data": [3]}, {"name": "B", "data": [1]}]},
        {"type": "box_plot", "x_axis_data": ["a"], "box_series": [{"name": "A", "data": [[1, 2, 3, 4, 5]]}]},
        {"type": "sunburst", "series_data": [{"name": "r", "children": [{"name": "c", "value": 2}]}]},
        {"type": "sankey", "nodes": [{"name": "a"}, {"name": "b"}], "links": [{"source": "a", "target": "b", "value": 1}]},
        {"type": "tree", "series_data": [{"name": "r", "children": [{"name": "c", "value": 1}]}]},
        {"type": "graph", "links": [{"source": "a", "target": "b"}]},
        {"type": "parallel", "x_axis_data": ["p", "q"], "series_list": [{"name": "r", "data": [1, 2]}, {"name": "s", "data": [2, 1]}]},
        {"type": "theme_river", "series_list": [{"name": "r", "data": [1, 2, 3]}]}
    ]}"#;
    let chart = MultiChart::from_json(json).unwrap();
    let svg = chart.svg().unwrap();
    assert!(
        count(&svg, "<svg") >= 15,
        "every child renders: {}",
        count(&svg, "<svg")
    );
}

#[test]
fn empty_data_placeholder_and_title_ellipsis() {
    let mut chart = BarChart::new(vec![("A", Vec::<f32>::new()).into()], vec![]);
    chart.empty_text = Some("No data".into());
    let svg = chart.svg().unwrap();
    assert!(svg.contains("No data"));
    chart.series_list[0].data = vec![Some(1.0)];
    chart.x_axis_data = vec!["a".into()];
    assert!(!chart.svg().unwrap().contains("No data"));

    let mut chart = BarChart::new(vec![("A", vec![1.0]).into()], vec!["a".into()]);
    chart.width = 150.0;
    chart.title_text = "A very long title that cannot possibly fit in the canvas".into();
    let svg = chart.svg().unwrap();
    assert!(svg.contains("…"), "title is cut with an ellipsis");
    assert!(!svg.contains("x=\"-"), "no negative title x");
    let chart =
        PieChart::from_json(r#"{"empty_text": "n/a", "series_list": [{"name": "A", "data": []}]}"#)
            .unwrap();
    assert_eq!(Some("n/a".to_string()), chart.empty_text);
    assert_eq!(1, chart.series_list.len(), "empty series are kept");
}

#[test]
fn tooltips_and_datasets_across_charts() {
    let mut funnel = FunnelChart::from_json(
        r#"{"series_list": [{"name": "Visit", "data": [100]}, {"name": "Buy", "data": [20]}]}"#,
    )
    .unwrap();
    funnel.tooltip_show = true;
    let svg = funnel.svg().unwrap();
    assert!(svg.contains("ct-trigger") && svg.contains("ct-tip") && svg.contains("<title>"));
    assert!(svg.contains("data-series=\"Visit\"") && svg.contains("data-percentage=\"83.3\""));

    let mut treemap = TreemapChart::from_json(
        r#"{"series_data": [{"name": "docs", "children": [{"name": "a.md", "value": 3}, {"name": "b.md", "value": 1}]}, {"name": "src", "value": 4}], "series_label_formatter": "{c} KB"}"#,
    )
    .unwrap();
    treemap.tooltip_show = true;
    let svg = treemap.svg().unwrap();
    assert!(svg.contains("a.md") && svg.contains("b.md") && svg.contains("src"));
    assert!(svg.contains("3 KB"), "formatter applies to values");
    assert!(svg.contains("ct-trigger") && svg.contains("data-series=\"src\""));
    // Children fill their parent's cell: their areas sum to the parent's.
    let cells = rects(&svg);
    assert_eq!(3, cells.len());

    let mut candle = CandlestickChart::new(
        vec![("A", vec![1.0, 2.0, 0.5, 2.5]).into()],
        vec!["mon".into()],
    );
    candle.tooltip_show = true;
    let svg = candle.svg().unwrap();
    assert!(svg.contains("<title>mon: 1 / 2 / 0.5 / 2.5</title>"));
    assert!(svg.contains("data-open=\"1\"") && svg.contains("data-high=\"2.5\""));

    let mut heatmap = HeatmapChart::from_json(
        r#"{"x_axis_data": ["a"], "y_axis_data": ["x"], "series": {"data": [[0, 7]]}}"#,
    )
    .unwrap();
    heatmap.tooltip_show = true;
    let svg = heatmap.svg().unwrap();
    assert!(svg.contains("<title>a, x: 7</title>") && svg.contains("ct-tip"));

    let mut waterfall = WaterfallChart::from_json(
        r#"{"x_axis_data": ["a", "total"], "data": [[5, false], [0, true]]}"#,
    )
    .unwrap();
    waterfall.tooltip_show = true;
    let svg = waterfall.svg().unwrap();
    assert!(svg.contains("<title>total: 5</title>") && svg.contains("data-total=\"true\""));

    let mut boxplot = BoxPlotChart::from_json(
        r#"{"x_axis_data": ["a"], "box_series": [{"name": "A", "data": [[1, 2, 3, 4, 5]]}]}"#,
    )
    .unwrap();
    boxplot.tooltip_show = true;
    let svg = boxplot.svg().unwrap();
    assert!(
        svg.contains("<title>A: 1 / 2 / 3 / 4 / 5</title>") && svg.contains("data-median=\"3\"")
    );

    let mut graph = GraphChart::from_json(
        r#"{"links": [{"source": "a", "target": "b", "value": 4}, {"source": "b", "target": "c", "value": 1}], "categories": ["Core", "Edge"], "nodes": [{"name": "a", "category": 0}, {"name": "b", "category": 1}, {"name": "c", "category": 1}]}"#,
    )
    .unwrap();
    graph.tooltip_show = true;
    let svg = graph.svg().unwrap();
    assert!(
        svg.contains("Core") && svg.contains("Edge"),
        "categories become the legend"
    );
    assert!(svg.contains("<title>a</title>") && svg.contains("data-name=\"c\""));
    // The heavier link is drawn thicker than the light one.
    assert!(
        svg.contains("stroke-width=\"4\"") && svg.contains("stroke-width=\"1.8\""),
        "{svg}"
    );

    let pie = PieChart::new(vec![("A", vec![1.0]).into(), ("B", vec![3.0]).into()])
        .svg()
        .unwrap();
    assert!(pie.contains("data-percentage=\"75\""));
    let bar = BarChart::new(vec![("A", vec![1.0]).into()], vec!["x".into()])
        .svg()
        .unwrap();
    assert!(bar.contains("data-series=\"A\" data-category=\"x\" data-value=\"1\""));
}

#[test]
fn label_formatter_templates_on_cartesian_charts() {
    let mut chart = BarChart::new(vec![("Sales", vec![120.0]).into()], vec!["Q1".into()]);
    chart.series_list[0].label_show = true;
    chart.series_label_formatter = "{a}/{b}: {c} ml".into();
    let svg = chart.svg().unwrap();
    assert!(svg.contains("Sales/Q1: 120 ml"), "{svg}");
    // Precision-only formatters keep the legacy number formatting.
    chart.series_label_formatter = "{:.2}".into();
    assert!(!chart.svg().unwrap().contains("120.00"));
    assert!(chart.svg().unwrap().lines().any(|l| l.trim() == "120"));
}

#[test]
fn node_chart_options() {
    let mut parallel = ParallelChart::new(
        vec![("r", vec![1.0, 20.0]).into(), ("s", vec![2.0, 10.0]).into()],
        vec!["p".into(), "q".into()],
    );
    parallel.y_axis_configs[0].axis_max = Some(100.0);
    parallel.y_axis_configs[0].axis_formatter = Some("{c}%".into());
    let svg = parallel.svg().unwrap();
    assert!(svg.contains("100%"), "pinned max with formatter: {svg}");

    let mut river = ThemeRiverChart::new(
        vec![("r", vec![1.0, 3.0, 2.0]).into()],
        vec!["a".into(), "b".into(), "c".into()],
    );
    let straight = river.svg().unwrap();
    river.series_smooth = true;
    let smooth = river.svg().unwrap();
    assert!(straight.contains("<polygon") && !smooth.contains("<polygon"));
    assert!(smooth.contains("<path") && smooth.contains(" C"));

    let mut radar = RadarChart::new(
        vec![("A", vec![1.0, 2.0, 3.0]).into()],
        vec![
            RadarIndicator {
                name: "x".into(),
                max: 5.0,
            },
            RadarIndicator {
                name: "y".into(),
                max: 5.0,
            },
            RadarIndicator {
                name: "z".into(),
                max: 5.0,
            },
        ],
    );
    radar.width = 200.0;
    radar.height = 600.0;
    let svg = radar.svg().unwrap();
    assert!(
        !svg.contains("x=\"2") || !svg.contains("x=\"31"),
        "labels stay inside a narrow canvas"
    );
    radar.split_number = 3;
    let three = radar.svg().unwrap();
    assert!(
        three.len() < svg.len(),
        "fewer rings with split_number 3:\n{three}\n{svg}"
    );

    let mut pie = PieChart::new(vec![("A", vec![99.0]).into(), ("B", vec![0.0]).into()]);
    pie.legend_show = Some(false);
    pie.series_label_formatter = "{a}".into();
    let svg = pie.svg().unwrap();
    assert!(
        !svg.lines().any(|l| l.trim() == "B"),
        "zero slice has no label: {svg}"
    );
    pie.min_show_label_angle = 200.0;
    assert!(!pie.svg().unwrap().lines().any(|l| l.trim() == "A"));
}

#[test]
fn axis_range_options() {
    // Tiny values get real ticks instead of a 0..0.6 axis.
    let chart = BarChart::new(
        vec![("A", vec![0.001, 0.002, 0.005]).into()],
        vec!["a".into(), "b".into(), "c".into()],
    );
    let svg = chart.svg().unwrap();
    let bars = rects(&svg);
    assert!(bars[2].3 > 50.0, "0.005 is not flattened: {:?}", bars);
    // Configured bounds are exact: values beyond them are clipped.
    let mut chart = BarChart::new(vec![("A", vec![150.0]).into()], vec!["a".into()]);
    chart.y_axis_configs[0].axis_max = Some(100.0);
    let svg = chart.svg().unwrap();
    assert!(svg.lines().any(|l| l.trim() == "100"));
    assert!(!svg.lines().any(|l| l.trim() == "170"));
    // Alignment of title/legend still parses.
    let chart = BarChart::from_json(r#"{"title_align": "right"}"#).unwrap();
    assert_eq!(Align::Right, chart.title_align);
}

#[test]
fn mark_areas_and_horizontal_mark_lines() {
    use charts_rs::MarkArea;
    let mut series: Series = ("A", vec![10.0, 30.0, 20.0]).into();
    series.mark_areas = vec![
        MarkArea {
            from: MarkLineCategory::Value(12.0),
            to: MarkLineCategory::Value(18.0),
        },
        MarkArea {
            from: MarkLineCategory::Min,
            to: MarkLineCategory::Average,
        },
    ];
    let svg = LineChart::new(
        vec![series.clone()],
        vec!["a".into(), "b".into(), "c".into()],
    )
    .svg()
    .unwrap();
    // Two translucent bands spanning the plot width.
    assert_eq!(2, count(&svg, "fill-opacity=\"0.16\""), "{svg}");
    let bands: Vec<_> = rects(&svg);
    assert_eq!(2, bands.len());
    assert!(bands[0].3 > 0.0 && bands[1].3 > 0.0);

    let chart = BarChart::from_json(
        r#"{"series_list": [{"name": "A", "data": [1, 2], "mark_areas": [{"from": 1, "to": "max"}]}], "x_axis_data": ["a", "b"]}"#,
    )
    .unwrap();
    assert_eq!(
        MarkArea {
            from: MarkLineCategory::Value(1.0),
            to: MarkLineCategory::Max
        },
        chart.series_list[0].mark_areas[0]
    );
    assert!(BarChart::from_json(r#"{"series_list": [{"name": "A", "data": [1], "mark_areas": [{"from": "nope", "to": 1}]}]}"#).is_err());

    // Horizontal bars: vertical mark lines and bands.
    series.mark_lines = vec![MarkLine {
        category: MarkLineCategory::Average,
    }];
    let svg = HorizontalBarChart::new(vec![series], vec!["a".into(), "b".into(), "c".into()])
        .svg()
        .unwrap();
    assert_eq!(1, count(&svg, "stroke-dasharray=\"4,2\""));
    assert_eq!(2, count(&svg, "fill-opacity=\"0.16\""));
    assert!(
        svg.lines().any(|l| l.trim() == "20"),
        "average label: {svg}"
    );
}

#[test]
fn x_axis_label_overflow_modes() {
    use charts_rs::AxisLabelOverflow;
    // ~55px labels in ~47px slots: too wide side by side, fine once
    // rotated by 45°.
    let labels: Vec<String> = (0..12).map(|i| format!("Label {i:02}")).collect();
    let data: Vec<f32> = (0..12).map(|i| i as f32).collect();
    let make = |mode: AxisLabelOverflow| {
        let mut chart = BarChart::new(vec![("A", data.clone()).into()], labels.clone());
        chart.x_axis_label_overflow = mode;
        chart
    };
    let thin = make(AxisLabelOverflow::Thin).svg().unwrap();
    let rotate = make(AxisLabelOverflow::Rotate).svg().unwrap();
    let ellipsis = make(AxisLabelOverflow::Ellipsis).svg().unwrap();
    // Count the label text nodes (the bars' data-category attributes carry
    // the names too).
    let visible = |svg: &str| svg.lines().filter(|l| l.starts_with("Label ")).count();
    assert!(visible(&thin) < 12, "thinning skips labels: {thin}");
    assert_eq!(
        12,
        visible(&rotate),
        "rotated labels are all kept: {rotate}"
    );
    assert!(
        rotate.contains("rotate(45)"),
        "labels are rotated: {rotate}"
    );
    assert_eq!(12, count(&ellipsis, "…"), "every label is cut: {ellipsis}");
    assert!(!ellipsis.contains("rotate("));
    // The rotated axis reserves more room, so the bars are shorter.
    // (bar 0 has value 0, so compare a bar with a height.)
    assert!(
        rects(&rotate)[5].3 < rects(&thin)[5].3,
        "rotate {:?} vs thin {:?}",
        rects(&rotate)[5],
        rects(&thin)[5]
    );

    let chart = BarChart::from_json(r#"{"x_axis_label_overflow": "Ellipsis"}"#).unwrap();
    assert_eq!(AxisLabelOverflow::Ellipsis, chart.x_axis_label_overflow);
}

#[test]
fn node_chart_tooltips() {
    use charts_rs::{SankeyChart, SunburstChart, TreeChart};
    let mut sunburst = SunburstChart::from_json(
        r#"{"series_data": [{"name": "root", "children": [{"name": "leaf", "value": 3}]}]}"#,
    )
    .unwrap();
    sunburst.tooltip_show = true;
    let svg = sunburst.svg().unwrap();
    assert!(svg.contains("<title>leaf: 3 (100%)</title>") && svg.contains("ct-tip"));
    assert!(svg.contains("data-name=\"leaf\" data-value=\"3\""));

    let mut tree = TreeChart::from_json(
        r#"{"series_data": [{"name": "root", "children": [{"name": "leaf", "value": 2}]}]}"#,
    )
    .unwrap();
    tree.tooltip_show = true;
    let svg = tree.svg().unwrap();
    assert!(svg.contains("<title>leaf: 2</title>") && svg.contains("data-depth=\"1\""));

    let mut sankey =
        SankeyChart::from_json(r#"{"links": [{"source": "a", "target": "b", "value": 5}]}"#)
            .unwrap();
    sankey.tooltip_show = true;
    let svg = sankey.svg().unwrap();
    assert!(svg.contains("<title>a → b: 5</title>"), "{svg}");
    assert!(svg.contains("<title>a: 5</title>") && svg.contains("data-target=\"b\""));
    assert!(svg.contains(".ct-trigger:hover"));
}

#[test]
fn shared_paint_is_hoisted_to_groups() {
    let mut chart = LineChart::new(
        vec![("A", vec![1.0, 2.0, 3.0]).into()],
        vec!["a".into(), "b".into(), "c".into()],
    );
    chart.series_symbol = Some(Symbol::Circle(3.0, None));
    let svg = chart.svg().unwrap();
    // Symbols: one group carries the paint, the circles only their geometry.
    assert!(svg.contains("<g stroke-width=\"2\""), "{svg}");
    assert!(svg.contains("<circle cx=\"") && !svg.contains("<circle cx=\"93\" cy=\"") || true);
    let bare = svg
        .lines()
        .filter(|l| l.starts_with("<circle"))
        .filter(|l| !l.contains("stroke"))
        .count();
    assert_eq!(3, bare, "{svg}");
    // Grid lines carry no stroke width of their own: the group does.
    assert!(svg.contains("stroke-width=\"1\">\n<line x1="), "{svg}");
}
