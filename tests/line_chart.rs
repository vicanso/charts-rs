use charts_rs::LineChart;
use pretty_assertions::assert_eq;

#[test]
fn line_chart() {
    let line_chart = LineChart::from_json(
        r###"{
        "title_text": "Stacked Area Chart",
        "sub_title_text": "Hello World",
        "legend_margin": {
            "top": 50,
            "bottom": 10
        },
        "margin": {
            "top": 10,
            "right": 50,
            "bottom": 10,
            "left": 10
        },
        "series_list": [
            {
                "name": "Email",
                "data": [120.0, 132.0, 101.0, 134.0, 90.0, 230.0, 210.0]
            },
            {
                "name": "Union Ads",
                "data": [220.0, 182.0, 191.0, 234.0, 290.0, 330.0, 310.0]
            },
            {
                "name": "Direct",
                "mark_points": [
                    {
                        "category": "max"
                    },
                    {
                        "category": "min"
                    }
                ],
                "data": [320.0, 332.0, 301.0, 334.0, 390.0, 330.0, 320.0]
            },
            {
                "name": "Search Engine",
                "label_show": true,
                "mark_lines": [
                    {
                        "category": "average"
                    }
                ],
                "data": [820.0, 932.0, 901.0, 934.0, 1290.0, 1330.0, 1320.0]
            }
        ],
        "x_axis_name_rotate": 0.393,
        "x_axis_data": [
            "Mon",
            "Tue",
            "Wed",
            "Thu",
            "Fri",
            "Sat",
            "Sun"
        ]
    }"###,
    )
    .unwrap();

    assert_eq!(
        include_str!("../asset/line_chart/basic_json.svg"),
        line_chart.svg().unwrap()
    );
}

#[test]
fn line_chart_nil_value() {
    let line_chart = LineChart::from_json(
        r###"{
        "title_text": "Stacked Area Chart",
        "sub_title_text": "Hello World",
        "legend_margin": {
            "top": 50,
            "bottom": 10
        },
        "series_list": [
            {
                "name": "Email",
                "data": [120.0, null, 101.0, 134.0, 90.0, 230.0, 210.0]
            },
            {
                "name": "Union Ads",
                "data": [220.0, 182.0, null, 234.0, 290.0, 330.0, 310.0]
            },
            {
                "name": "Direct",
                "data": [320.0, 332.0, 301.0, 334.0, null, 330.0, 320.0]
            },
            {
                "name": "Search Engine",
                "label_show": true,
                "data": [820.0, 932.0, null, 934.0, 1290.0, 1330.0, 1320.0]
            }
        ],
        "x_axis_data": [
            "Mon",
            "Tue",
            "Wed",
            "Thu",
            "Fri",
            "Sat",
            "Sun"
        ]
    }"###,
    )
    .unwrap();

    assert_eq!(
        include_str!("../asset/line_chart/nil_value_json.svg"),
        line_chart.svg().unwrap()
    );
}
