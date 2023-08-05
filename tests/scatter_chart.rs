use charts_rs::ScatterChart;
use pretty_assertions::assert_eq;

#[test]
fn scatter_chart() {
    let scatter_chart = ScatterChart::from_json(
        r###"{
            "width": 630,
            "height": 410,
            "margin": {
                "left": 10,
                "top": 5,
                "right": 20
            },
            "title_text": "Male and female height and weight distribution",
            "title_align": "left",
            "sub_title_text": "Data from: Heinz 2003",
            "sub_title_align": "left",
            "legend_align": "right",
            "y_axis_configs": [
                {
                    "axis_min": 40,
                    "axis_max": 130,
                    "axis_formatter": "{c} kg"
                }
            ],
            "x_axis_config": {
                "axis_min": 140,
                "axis_max": 230,
                "axis_formatter": "{c} cm"
            },
            "series_list": [
                {
                    "name": "Female",
                    "data": [
                        161.2, 51.6, 167.5, 59.0, 159.5, 49.2, 157.0, 63.0, 155.8, 53.6, 170.0, 59.0,
                        159.1, 47.6, 166.0, 69.8, 176.2, 66.8, 160.2, 75.2, 172.5, 55.2, 170.9, 54.2,
                        172.9, 62.5, 153.4, 42.0, 160.0, 50.0, 147.2, 49.8, 168.2, 49.2, 175.0, 73.2,
                        157.0, 47.8, 167.6, 68.8, 159.5, 50.6, 175.0, 82.5, 166.8, 57.2, 176.5, 87.8,
                        170.2, 72.8
                    ]
                },
                {
                    "name": "Male",
                    "data": [
                        174.0, 65.6, 175.3, 71.8, 193.5, 80.7, 186.5, 72.6, 187.2, 78.8, 181.5, 74.8,
                        184.0, 86.4, 184.5, 78.4, 175.0, 62.0, 184.0, 81.6, 180.0, 76.6, 177.8, 83.6,
                        192.0, 90.0, 176.0, 74.6, 174.0, 71.0, 184.0, 79.6, 192.7, 93.8, 171.5, 70.0,
                        173.0, 72.4, 176.0, 85.9, 176.0, 78.8, 180.5, 77.8, 172.7, 66.2, 176.0, 86.4,
                        173.5, 81.8
                    ]
                }
            ],
            "series_symbol_sizes": [6, 6]
        }"###).unwrap();

    assert_eq!(
        include_str!("../asset/scatter_chart/basic_json.svg"),
        scatter_chart.svg().unwrap()
    );
}
