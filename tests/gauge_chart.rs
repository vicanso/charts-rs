mod common;

use charts_rs::GaugeChart;

#[test]
fn gauge_chart_basic_json() {
    let chart = GaugeChart::from_json(
        r##"{
            "title_text": "Gauge",
            "min": 0,
            "max": 200,
            "series_list": [{"name": "Speed", "data": [120]}]
        }"##,
    )
    .unwrap();
    common::assert_snapshot!("gauge_chart/basic_json.svg", chart.svg().unwrap());
}

#[test]
fn gauge_chart_grafana_json() {
    let chart = GaugeChart::from_json(
        r##"{
            "theme": "grafana",
            "title_text": "Gauge",
            "min": 0,
            "max": 200,
            "series_list": [{"name": "Speed", "data": [120]}]
        }"##,
    )
    .unwrap();
    common::assert_snapshot!("gauge_chart/grafana_json.svg", chart.svg().unwrap());
}
