use charts_rs::FunnelChart;
use pretty_assertions::assert_eq;

#[test]
fn funnel_chart_basic_json() {
    let chart = FunnelChart::from_json(
        r##"{
            "title_text": "Funnel Chart",
            "series_label_position": "inside",
            "funnel_gap": 4,
            "series_list": [
                {"name": "Impression", "data": [60000]},
                {"name": "Click",      "data": [40000]},
                {"name": "Inquiry",    "data": [20000]},
                {"name": "Order",      "data": [8000]},
                {"name": "Re-order",   "data": [2000]}
            ]
        }"##,
    )
    .unwrap();
    assert_eq!(
        include_str!("../asset/funnel_chart/basic_json.svg"),
        chart.svg().unwrap()
    );
}

#[test]
fn funnel_chart_grafana_json() {
    let chart = FunnelChart::from_json(
        r##"{
            "theme": "grafana",
            "title_text": "Funnel Chart",
            "series_label_position": "inside",
            "funnel_gap": 4,
            "series_list": [
                {"name": "Impression", "data": [60000]},
                {"name": "Click",      "data": [40000]},
                {"name": "Inquiry",    "data": [20000]},
                {"name": "Order",      "data": [8000]},
                {"name": "Re-order",   "data": [2000]}
            ]
        }"##,
    )
    .unwrap();
    assert_eq!(
        include_str!("../asset/funnel_chart/grafana_json.svg"),
        chart.svg().unwrap()
    );
}
