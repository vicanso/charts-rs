mod common;

use charts_rs::SankeyChart;

#[test]
fn sankey_chart_json() {
    let chart =
        SankeyChart::from_json(include_str!("../asset/sankey_chart/integration.json")).unwrap();
    common::assert_snapshot!("sankey_chart/integration_json.svg", chart.svg().unwrap());
}
