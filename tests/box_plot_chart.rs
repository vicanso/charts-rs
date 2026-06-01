use charts_rs::BoxPlotChart;
use pretty_assertions::assert_eq;

#[test]
fn box_plot_chart_basic_json() {
    let chart = BoxPlotChart::from_json(
        r##"{
            "title_text": "Box Plot",
            "x_axis_data": ["Cat A", "Cat B", "Cat C"],
            "box_series": [
                {"name": "Group 1", "data": [[3,10,18,28,40],[5,14,22,32,45],[1,8,15,24,35]]},
                {"name": "Group 2", "data": [[5,13,21,31,43],[2,9,17,26,38],[4,11,19,29,41]]}
            ]
        }"##,
    )
    .unwrap();
    assert_eq!(
        include_str!("../asset/box_plot_chart/basic_json.svg"),
        chart.svg().unwrap()
    );
}

#[test]
fn box_plot_chart_grafana_json() {
    let chart = BoxPlotChart::from_json(
        r##"{
            "theme": "grafana",
            "title_text": "Box Plot",
            "x_axis_data": ["Cat A", "Cat B", "Cat C"],
            "box_series": [
                {"name": "Group 1", "data": [[3,10,18,28,40],[5,14,22,32,45],[1,8,15,24,35]]},
                {"name": "Group 2", "data": [[5,13,21,31,43],[2,9,17,26,38],[4,11,19,29,41]]}
            ]
        }"##,
    )
    .unwrap();
    assert_eq!(
        include_str!("../asset/box_plot_chart/grafana_json.svg"),
        chart.svg().unwrap()
    );
}
