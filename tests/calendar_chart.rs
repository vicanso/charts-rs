use charts_rs::CalendarChart;
use pretty_assertions::assert_eq;

#[test]
fn calendar_chart_basic_json() {
    let chart = CalendarChart::from_json(
        r##"{
            "start_date": "2024-01-01",
            "end_date": "2024-12-31",
            "title_text": "2024 Contributions",
            "cell_size": 11,
            "cell_gap": 2,
            "max_color": "#216e39",
            "min_color": "#ebedf0",
            "data": [
                ["2024-01-05", 2],
                ["2024-02-14", 8],
                ["2024-06-15", 9],
                ["2024-09-01", 4],
                ["2024-12-25", 10]
            ]
        }"##,
    )
    .unwrap();
    assert_eq!(
        include_str!("../asset/calendar_chart/basic_json.svg"),
        chart.svg().unwrap()
    );
}

#[test]
fn calendar_chart_grafana_json() {
    let chart = CalendarChart::from_json(
        r##"{
            "theme": "grafana",
            "start_date": "2024-01-01",
            "end_date": "2024-12-31",
            "title_text": "2024 Contributions",
            "cell_size": 11,
            "cell_gap": 2,
            "max_color": "#216e39",
            "min_color": "#ebedf0",
            "data": [
                ["2024-01-05", 2],
                ["2024-02-14", 8],
                ["2024-06-15", 9],
                ["2024-09-01", 4],
                ["2024-12-25", 10]
            ]
        }"##,
    )
    .unwrap();
    assert_eq!(
        include_str!("../asset/calendar_chart/grafana_json.svg"),
        chart.svg().unwrap()
    );
}
