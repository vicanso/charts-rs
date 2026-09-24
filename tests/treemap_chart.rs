mod common;

use charts_rs::TreemapChart;

#[test]
fn treemap_chart_basic_json() {
    let chart = TreemapChart::from_json(
        r##"{
            "title_text": "Disk Usage",
            "item_gap": 3,
            "series_list": [
                {"name": "nodeExcel", "data": [600]},
                {"name": "nodePPT",   "data": [500]},
                {"name": "nodeDoc",   "data": [400]},
                {"name": "nodeWeb",   "data": [300]},
                {"name": "nodeWord",  "data": [200]},
                {"name": "nodeOther", "data": [100]}
            ]
        }"##,
    )
    .unwrap();
    common::assert_snapshot!("treemap_chart/basic_json.svg", chart.svg().unwrap());
}

#[test]
fn treemap_chart_grafana_json() {
    let chart = TreemapChart::from_json(
        r##"{
            "theme": "grafana",
            "title_text": "Disk Usage",
            "item_gap": 3,
            "series_list": [
                {"name": "nodeExcel", "data": [600]},
                {"name": "nodePPT",   "data": [500]},
                {"name": "nodeDoc",   "data": [400]},
                {"name": "nodeWeb",   "data": [300]},
                {"name": "nodeWord",  "data": [200]},
                {"name": "nodeOther", "data": [100]}
            ]
        }"##,
    )
    .unwrap();
    common::assert_snapshot!("treemap_chart/grafana_json.svg", chart.svg().unwrap());
}
