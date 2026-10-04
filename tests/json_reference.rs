//! The examples of the JSON reference (docs/json.md, docs/json-zh.md) are
//! real options: every one of them makes its chart.

use charts_rs::*;

/// The JSON below each `<!-- example: chart -->` marker of a page.
fn examples(page: &str) -> Vec<(&str, &str)> {
    page.split("<!-- example: ")
        .skip(1)
        .map(|part| {
            let (chart, rest) = part.split_once(" -->").unwrap();
            let (_, rest) = rest.split_once("```json\n").unwrap();
            let (json, _) = rest.split_once("```").unwrap();
            (chart, json)
        })
        .collect()
}

fn render(chart: &str, json: &str) -> Result<String> {
    match chart {
        "bar" => BarChart::from_json(json)?.svg(),
        "horizontal_bar" => HorizontalBarChart::from_json(json)?.svg(),
        "line" => LineChart::from_json(json)?.svg(),
        "pie" => PieChart::from_json(json)?.svg(),
        "radar" => RadarChart::from_json(json)?.svg(),
        "scatter" => ScatterChart::from_json(json)?.svg(),
        "candlestick" => CandlestickChart::from_json(json)?.svg(),
        "table" => TableChart::from_json(json)?.svg(),
        "heatmap" => HeatmapChart::from_json(json)?.svg(),
        "funnel" => FunnelChart::from_json(json)?.svg(),
        "waterfall" => WaterfallChart::from_json(json)?.svg(),
        "calendar" => CalendarChart::from_json(json)?.svg(),
        "gauge" => GaugeChart::from_json(json)?.svg(),
        "treemap" => TreemapChart::from_json(json)?.svg(),
        "box_plot" => BoxPlotChart::from_json(json)?.svg(),
        "sunburst" => SunburstChart::from_json(json)?.svg(),
        "sankey" => SankeyChart::from_json(json)?.svg(),
        "tree" => TreeChart::from_json(json)?.svg(),
        "graph" => GraphChart::from_json(json)?.svg(),
        "parallel" => ParallelChart::from_json(json)?.svg(),
        "theme_river" => ThemeRiverChart::from_json(json)?.svg(),
        "histogram" => HistogramChart::from_json(json)?.svg(),
        "polar_bar" => PolarBarChart::from_json(json)?.svg(),
        "chord" => ChordChart::from_json(json)?.svg(),
        "gantt" => GanttChart::from_json(json)?.svg(),
        "map" => MapChart::from_json(json)?.svg(),
        "multi" => MultiChart::from_json(json)?.svg(),
        other => panic!("no chart for the example `{other}`"),
    }
}

#[test]
fn reference_examples_render() {
    for (name, page) in [
        ("docs/json.md", include_str!("../docs/json.md")),
        ("docs/json-zh.md", include_str!("../docs/json-zh.md")),
    ] {
        let examples = examples(page);
        // One for each of the 27 chart types.
        let mut charts: Vec<&str> = examples.iter().map(|(chart, _)| *chart).collect();
        charts.sort_unstable();
        charts.dedup();
        assert_eq!(27, charts.len(), "{name}: {charts:?}");
        for (chart, json) in examples {
            let svg = render(chart, json).unwrap_or_else(|e| panic!("{name}: {chart}: {e}"));
            assert!(
                svg.starts_with("<svg") && svg.len() > 500,
                "{name}: {chart}"
            );
            assert!(
                !svg.contains("NaN") && !svg.contains("inf"),
                "{name}: {chart}"
            );
            // The example shows data, not an empty frame.
            let shapes = ["<rect", "<path", "<circle", "<polygon", "<polyline"]
                .iter()
                .map(|tag| svg.matches(tag).count())
                .sum::<usize>();
            assert!(shapes > 3, "{name}: {chart} draws {shapes} shapes");
        }
    }
}
