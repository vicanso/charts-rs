use charts_rs::{BarChart, Chart, LineChart, PieChart};

#[test]
fn dyn_chart_render() {
    // Mixed chart types held and rendered through the shared trait.
    let charts: Vec<Box<dyn Chart>> = vec![
        Box::new(
            BarChart::from_json(
                r###"{
                    "series_list": [{"name": "s", "data": [1, 2, 3]}],
                    "x_axis_data": ["A", "B", "C"]
                }"###,
            )
            .unwrap(),
        ),
        Box::new(
            LineChart::from_json(
                r###"{
                    "series_list": [{"name": "s", "data": [3, 2, 1]}],
                    "x_axis_data": ["A", "B", "C"]
                }"###,
            )
            .unwrap(),
        ),
        Box::new(
            PieChart::from_json(
                r###"{
                    "series_list": [
                        {"name": "a", "data": [30]},
                        {"name": "b", "data": [70]}
                    ]
                }"###,
            )
            .unwrap(),
        ),
    ];
    for chart in charts.iter() {
        let svg = chart.svg().unwrap();
        assert!(svg.starts_with("<svg"));
    }
}
