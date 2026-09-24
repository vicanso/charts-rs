mod common;

use charts_rs::GraphChart;

#[test]
fn graph_chart_json() {
    let chart =
        GraphChart::from_json(include_str!("../asset/graph_chart/integration.json")).unwrap();
    common::assert_snapshot!("graph_chart/integration_json.svg", chart.svg().unwrap());
}
