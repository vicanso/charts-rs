use charts_rs::MultiChart;
use pretty_assertions::assert_eq;

#[test]
fn multi_chart() {
    let mut multi_chart = MultiChart::from_json(
        r###"{
        "child_charts": [
            {
            "quality": 80,
            "width": 600,
            "height": 400,
            "margin": {
                "left": 5,
                "top": 5,
                "right": 5,
                "bottom": 5
            },
            "font_family": "Roboto",
            "title_font_size": 18,
            "title_font_weight": "bold",
            "title_margin": {
                "left": 0,
                "top": 0,
                "right": 0,
                "bottom": 0
            },
            "title_align": "center",
            "title_height": 30,
            "sub_title_text": "Sub Title",
            "sub_title_font_size": 14,
            "sub_title_margin": {
                "left": 0,
                "top": 0,
                "right": 0,
                "bottom": 0
            },
            "sub_title_align": "center",
            "sub_title_height": 20,
            "legend_font_size": 14,
            "legend_align": "left",
            "legend_margin": {
                "left": 0,
                "top": 0,
                "right": 0,
                "bottom": 0
            },
            "legend_category": "normal",
            "legend_show": true,
            "x_axis_height": 30,
            "x_axis_font_size": 14,
            "x_axis_name_gap": 5,
            "x_axis_name_rotate": 0,
            "x_boundary_gap": true,
            "x_axis_margin": {
                "left": 0,
                "top": 0,
                "right": 0,
                "bottom": 0
            },
            "type": "bar",
            "title_text": "Bar Chart",
            "x_axis_data": [
                "Mon",
                "Tue",
                "Wed",
                "Thu",
                "Fri",
                "Sat",
                "Sun"
            ],
            "series_list": [
                {
                "name": "Email",
                "label_show": true,
                "data": [
                    120,
                    132,
                    101,
                    134,
                    90,
                    230,
                    210
                ]
                },
                {
                "name": "Union Ads",
                "label_show": true,
                "data": [
                    220,
                    182,
                    191,
                    234,
                    290,
                    330,
                    310
                ]
                }
            ]
            },
            {
            "quality": 80,
            "width": 600,
            "height": 400,
            "margin": {
                "left": 15,
                "top": 15,
                "right": 15,
                "bottom": 15
            },
            "font_family": "Roboto",
            "title_font_size": 18,
            "title_font_weight": "bold",
            "title_margin": {
                "left": 0,
                "top": 0,
                "right": 0,
                "bottom": 0
            },
            "title_align": "center",
            "title_height": 30,
            "sub_title_text": "Sub Title",
            "sub_title_font_size": 14,
            "sub_title_margin": {
                "left": 0,
                "top": 0,
                "right": 0,
                "bottom": 0
            },
            "sub_title_align": "center",
            "sub_title_height": 20,
            "legend_font_size": 14,
            "legend_align": "right",
            "legend_margin": {
                "left": 0,
                "top": 0,
                "right": 0,
                "bottom": 0
            },
            "legend_category": "round_rect",
            "legend_show": true,
            "x_axis_height": 30,
            "x_axis_font_size": 14,
            "x_axis_name_gap": 5,
            "x_axis_name_rotate": 0,
            "x_boundary_gap": false,
            "x_axis_margin": {
                "left": 0,
                "top": 0,
                "right": 0,
                "bottom": 0
            },
            "type": "line",
            "title_text": "Line Chart",
            "x_axis_data": [
                "Mon",
                "Tue",
                "Wed",
                "Thu",
                "Fri",
                "Sat",
                "Sun"
            ],
            "series_list": [
                {
                "name": "Email",
                "label_show": true,
                "data": [
                    120,
                    132,
                    101,
                    134,
                    90,
                    230,
                    210
                ]
                },
                {
                "name": "Union Ads",
                "label_show": true,
                "data": [
                    220,
                    182,
                    191,
                    234,
                    290,
                    330,
                    310
                ]
                }
            ]
            },
            {
            "quality": 80,
            "width": 600,
            "height": 400,
            "margin": {
                "left": 5,
                "top": 5,
                "right": 5,
                "bottom": 5
            },
            "font_family": "Roboto",
            "title_font_size": 18,
            "title_font_weight": "bold",
            "title_margin": {
                "left": 0,
                "top": 0,
                "right": 0,
                "bottom": 0
            },
            "title_align": "center",
            "title_height": 30,
            "sub_title_text": "Sub Title",
            "sub_title_font_size": 14,
            "sub_title_margin": {
                "left": 0,
                "top": 0,
                "right": 0,
                "bottom": 0
            },
            "sub_title_align": "center",
            "sub_title_height": 20,
            "legend_font_size": 14,
            "legend_align": "center",
            "legend_margin": {
                "top": 50
            },
            "legend_category": "normal",
            "legend_show": true,
            "x_axis_height": 30,
            "x_axis_font_size": 14,
            "x_axis_name_gap": 5,
            "x_axis_name_rotate": 0,
            "x_boundary_gap": true,
            "x_axis_margin": {
                "left": 0,
                "top": 0,
                "right": 0,
                "bottom": 0
            },
            "type": "pie",
            "title_text": "Nightingale Chart",
            "series_list": [
                {
                "name": "rose 1",
                "data": [
                    40
                ]
                },
                {
                "name": "rose 2",
                "data": [
                    38
                ]
                },
                {
                "name": "rose 3",
                "data": [
                    32
                ]
                },
                {
                "name": "rose 4",
                "data": [
                    30
                ]
                },
                {
                "name": "rose 5",
                "data": [
                    28
                ]
                },
                {
                "name": "rose 6",
                "data": [
                    26
                ]
                },
                {
                "name": "rose 7",
                "data": [
                    22
                ]
                },
                {
                "name": "rose 8",
                "data": [
                    18
                ]
                }
            ]
            }
        ],
        "theme": "grafana"
        }"###,
    )
    .unwrap();

    assert_eq!(
        include_str!("../asset/multi_chart/basic_json.svg"),
        multi_chart.svg().unwrap()
    );
}
