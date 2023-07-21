use charts_rs::RadarChart;
use pretty_assertions::assert_eq;

#[test]
fn radar_chart() {
    let radar_chart = RadarChart::from_json(
        r###"{
        "series_list": [
            {
                "name": "Allocated Budget",
                "data": [4200.0, 3000.0, 20000.0, 35000.0, 50000.0, 18000.0]
            },
            {
                "name": "Actual Spending",
                "data": [5000.0, 14000.0, 28000.0, 26000.0, 42000.0, 21000.0]
            }
        ],
        "indicators": [
            {
                "name": "Sales",
                "max": 6500
            },
            {
                "name": "Administration",
                "max": 16000
            },
            {
                "name": "Information Technology",
                "max": 30000
            },
            {
                "name": "Customer Support",
                "max": 38000
            },
            {
                "name": "Development",
                "max": 52000
            },
            {
                "name": "Marketing",
                "max": 25000
            }
        ]
    }"###,
    )
    .unwrap();

    assert_eq!(
        include_str!("../asset/radar_chart/basic_json.svg"),
        radar_chart.svg().unwrap()
    );
}
