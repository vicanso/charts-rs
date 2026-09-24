mod common;

use charts_rs::TreeChart;

#[test]
fn tree_chart_json() {
    let chart = TreeChart::from_json(include_str!("../asset/tree_chart/integration.json")).unwrap();
    common::assert_snapshot!("tree_chart/integration_json.svg", chart.svg().unwrap());
}
