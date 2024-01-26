use charts_rs::HeatmapChart;
use pretty_assertions::assert_eq;

#[test]
fn heatmap_chart() {
    let heatmap_chart = HeatmapChart::from_json(
        r###"{
            "theme": "grafana",
            "y_axis_data": [
                "Saturday",
                "Friday",
                "Thursday",
                "Wednesday",
                "Tuesday",
                "Monday",
                "Sunday"
            ],
            "x_axis_data": [
                "12a", "1a", "2a", "3a", "4a", "5a", "6a", "7a", "8a", "9a", "10a", "11a", "12p", "1p",
                "2p", "3p", "4p", "5p", "6p", "7p", "8p", "9p", "10p", "11p"
            ],
            "series": {
                "data": [
                    [0, 9.0],
                    [1, 3.0],
                    [7, 3.0],
                    [12, 3.0],
                    [24, 12.0],
                    [28, 10.0],
                    [31, 8.0],
                    [50, 4.0],
                    [63, 2.0]
                ]
            }
    }"###,
    )
    .unwrap();
    assert_eq!(
        include_str!("../asset/heatmap_chart/basic_grafana_json.svg"),
        heatmap_chart.svg().unwrap()
    );
}
