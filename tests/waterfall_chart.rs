use charts_rs::WaterfallChart;
use pretty_assertions::assert_eq;

#[test]
fn waterfall_chart_basic_json() {
    let chart = WaterfallChart::from_json(
        r#"{
            "title_text": "Waterfall Chart",
            "x_axis_data": ["Initial","Revenue","Services","Purchases","Marketing","Profit"],
            "data": [
                [900, false],
                [345, false],
                [393, false],
                [-108, false],
                [-154, false],
                [0, true]
            ]
        }"#,
    )
    .unwrap();
    assert_eq!(
        include_str!("../asset/waterfall_chart/basic_json.svg"),
        chart.svg().unwrap()
    );
}

#[test]
fn waterfall_chart_grafana_json() {
    let chart = WaterfallChart::from_json(
        r#"{
            "theme": "grafana",
            "title_text": "Waterfall Chart",
            "x_axis_data": ["Initial","Revenue","Services","Purchases","Marketing","Profit"],
            "data": [
                [900, false],
                [345, false],
                [393, false],
                [-108, false],
                [-154, false],
                [0, true]
            ]
        }"#,
    )
    .unwrap();
    assert_eq!(
        include_str!("../asset/waterfall_chart/grafana_json.svg"),
        chart.svg().unwrap()
    );
}
