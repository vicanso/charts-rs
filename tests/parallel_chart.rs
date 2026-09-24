mod common;

use charts_rs::ParallelChart;

#[test]
fn parallel_chart_json() {
    let chart =
        ParallelChart::from_json(include_str!("../asset/parallel_chart/integration.json")).unwrap();
    common::assert_snapshot!("parallel_chart/integration_json.svg", chart.svg().unwrap());
}
