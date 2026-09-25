//! Compact output renders the same picture in fewer bytes.
mod common;

use charts_rs::{
    BarChart, CalendarChart, FunnelChart, GaugeChart, HeatmapChart, LineChart, MultiChart,
    PieChart, SankeyChart, ScatterChart, TableChart, TreemapChart, compact_svg,
};

/// One JSON per chart type, rendered plain and compact.
fn cases() -> Vec<(&'static str, String, String)> {
    let bar = r##"{"title_text": "Bars", "series_list": [{"name": "A", "data": [3, 5, 2], "label_show": true}, {"name": "B", "data": [1, 4, 6]}], "x_axis_data": ["a", "b", "c"], "tooltip_show": true}"##;
    let line = r##"{"title_text": "Lines", "series_smooth": true, "series_fill": true, "series_list": [{"name": "A", "data": [3, 5, 2, 6]}, {"name": "B", "data": [1, 4, 6, 2]}], "x_axis_data": ["a", "b", "c", "d"]}"##;
    let pie = r##"{"series_list": [{"name": "A", "data": [3]}, {"name": "B", "data": [5]}, {"name": "C", "data": [2]}], "rose_type": false}"##;
    let scatter = r##"{"series_list": [{"name": "A", "data": [1, 2, 3, 4, 5, 6]}, {"name": "B", "data": [2, 2, 4, 4, 6, 6]}]}"##;
    let heatmap = r##"{"x_axis_data": ["a", "b"], "y_axis_data": ["x", "y"], "series": {"data": [[0, 1], [1, 5], [2, 3], [3, 8]]}}"##;
    let sankey = r##"{"nodes": [{"name": "a"}, {"name": "b"}, {"name": "c"}], "links": [{"source": "a", "target": "b", "value": 3}, {"source": "a", "target": "c", "value": 1}]}"##;
    let treemap = r##"{"series_list": [{"name": "A", "data": [6]}, {"name": "B", "data": [3]}, {"name": "C", "data": [1]}]}"##;
    let funnel =
        r##"{"series_list": [{"name": "Visit", "data": [100]}, {"name": "Buy", "data": [20]}]}"##;
    let gauge = r##"{"min": 0, "max": 100, "series_list": [{"name": "Speed", "data": [42]}]}"##;
    let calendar = r##"{"start_date": "2024-01-01", "end_date": "2024-02-29", "data": [["2024-01-05", 3], ["2024-02-10", 7]]}"##;
    let table = r##"{"data": [["Name", "Price"], ["Apple", "1.5"], ["Pear", "2"]]}"##;
    let multi = r##"{"child_charts": [{"type": "bar", "series_list": [{"name": "A", "data": [1, 2]}], "x_axis_data": ["a", "b"]}, {"type": "pie", "series_list": [{"name": "A", "data": [1]}, {"name": "B", "data": [2]}]}]}"##;

    macro_rules! case {
        ($name:literal, $chart:ty, $json:expr) => {{
            let plain = <$chart>::from_json($json).unwrap().svg().unwrap();
            let json = $json.replacen('{', r#"{"compact": true, "#, 1);
            let compact = <$chart>::from_json(&json).unwrap().svg().unwrap();
            ($name, plain, compact)
        }};
    }
    vec![
        case!("bar", BarChart, bar),
        case!("line", LineChart, line),
        case!("pie", PieChart, pie),
        case!("scatter", ScatterChart, scatter),
        case!("heatmap", HeatmapChart, heatmap),
        case!("sankey", SankeyChart, sankey),
        case!("treemap", TreemapChart, treemap),
        case!("funnel", FunnelChart, funnel),
        case!("gauge", GaugeChart, gauge),
        case!("calendar", CalendarChart, calendar),
        case!("table", TableChart, table),
        case!("multi", MultiChart, multi),
    ]
}

#[test]
fn compact_is_smaller_and_equals_the_post_pass() {
    let (mut total_plain, mut total_compact) = (0, 0);
    for (name, plain, compact) in cases() {
        // Every chart shrinks; charts made of distinct rounded cells (the
        // calendar) only a little, the rest by a fifth or more.
        assert!(
            compact.len() < plain.len(),
            "{name}: {} -> {} bytes",
            plain.len(),
            compact.len()
        );
        eprintln!(
            "{name:9} {:6} -> {:6} bytes ({:.0}% smaller)",
            plain.len(),
            compact.len(),
            (plain.len() - compact.len()) as f32 * 100.0 / plain.len() as f32
        );
        total_plain += plain.len();
        total_compact += compact.len();
        assert_eq!(
            compact_svg(&plain),
            compact,
            "{name}: the flag is the post-pass"
        );
        assert!(!compact.contains('\n'), "{name}: no newlines");
        assert!(compact.starts_with("<svg ") && compact.ends_with("</svg>"));
        // Semantics that must survive: tooltips, data attributes, text.
        for needle in ["<title>", "data-series=", "class=\"ct-trigger\""] {
            assert_eq!(
                plain.matches(needle).count(),
                compact.matches(needle).count(),
                "{name}: {needle}"
            );
        }
    }
    assert!(
        total_compact * 100 < total_plain * 80,
        "overall {total_plain} -> {total_compact} bytes"
    );
}

#[cfg(feature = "png")]
#[test]
fn compact_renders_identically() {
    use charts_rs::svg_to_png;
    for (name, plain, compact) in cases() {
        let a = image::load_from_memory(&svg_to_png(&plain).unwrap())
            .unwrap()
            .into_rgba8();
        let b = image::load_from_memory(&svg_to_png(&compact).unwrap())
            .unwrap()
            .into_rgba8();
        assert_eq!(a.dimensions(), b.dimensions(), "{name}");
        // Coordinates are rewritten exactly (fixed-point decimals), so the
        // rasterization is the same; allow a hair of anti-aliasing slack.
        let differing = a
            .pixels()
            .zip(b.pixels())
            .filter(|(p, q)| p.0.iter().zip(q.0.iter()).any(|(x, y)| x.abs_diff(*y) > 2))
            .count();
        assert_eq!(0, differing, "{name}: {differing} pixels differ");
    }
}

#[test]
fn compact_snapshots() {
    let (_, _, line) = cases().into_iter().find(|c| c.0 == "line").unwrap();
    common::assert_snapshot!("line_chart/compact.svg", line);
    let (_, _, sankey) = cases().into_iter().find(|c| c.0 == "sankey").unwrap();
    common::assert_snapshot!("sankey_chart/compact.svg", sankey);
}
