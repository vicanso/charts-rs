use charts_rs::BarChart;
use pretty_assertions::assert_eq;

#[test]
fn bar_chart() {
    let bar_chart = BarChart::from_json(
        r###"{
            "width": 630,
            "height": 410,
            "margin": {
                "left": 10,
                "top": 5,
                "right": 10
            },
            "title_text": "Bar Chart",
            "title_font_color": "#345",
            "title_align": "right",
            "sub_title_text": "demo",
            "legend_align": "left",
            "series_list": [
                {
                    "name": "Email",
                    "label_show": true,
                    "data": [120.0, 132.0, 101.0, 134.0, 90.0, 230.0, 210.0]
                },
                {
                    "name": "Union Ads",
                    "data": [220.0, 182.0, 191.0, 234.0, 290.0, 330.0, 310.0]
                },
                {
                    "name": "Direct",
                    "data": [320.0, 332.0, 301.0, 334.0, 390.0, 330.0, 320.0]
                },
                {
                    "name": "Search Engine",
                    "data": [820.0, 932.0, 901.0, 934.0, 1290.0, 1330.0, 1320.0]
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
        include_str!("../asset/bar_chart/basic_json.svg"),
        bar_chart.svg().unwrap()
    );
}

#[test]
fn bar_chart_mixin() {
    let bar_chart = BarChart::from_json(
        r###"{
            "width": 630,
            "height": 410,
            "margin": {
                "left": 10,
                "top": 5,
                "right": 10
            },
            "title_text": "Bar Chart",
            "title_font_color": "#345",
            "title_align": "right",
            "sub_title_text": "demo",
            "legend_align": "left",
            "series_list": [
                {
                    "name": "Email",
                    "label_show": true,
                    "data": [120.0, 132.0, 101.0, 134.0, 90.0, 230.0, 210.0]
                },
                {
                    "name": "Union Ads",
                    "data": [220.0, 182.0, 191.0, 234.0, 290.0, 330.0, 310.0]
                },
                {
                    "name": "Direct",
                    "data": [320.0, 332.0, 301.0, 334.0, 390.0, 330.0, 320.0]
                },
                {
                    "name": "Search Engine",
                    "data": [820.0, 932.0, 901.0, 934.0, 1290.0, 1330.0, 1320.0],
                    "category": "line"
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
        include_str!("../asset/bar_chart/line_mixin_json.svg"),
        bar_chart.svg().unwrap()
    );
}
