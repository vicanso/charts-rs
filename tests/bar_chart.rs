use charts_rs::BarChart;
use pretty_assertions::assert_eq;

mod common;

#[test]
fn bar_chart_stacked() {
    let bar_chart = BarChart::from_json(
        r###"{
            "width": 630, "height": 410,
             "margin": {
                "left": 10,
                "top": 5,
                "right": 10
            },
            "title_text": "Stacked Bar Chart",
            "series_list": [
                {"name": "Direct",        "label_show": true, "data": [320.0, 332.0, 301.0, 334.0, 390.0, 330.0, 320.0], "stack": "total"},
                {"name": "Mail Ad",                           "data": [120.0, 132.0, 101.0, 134.0,  90.0, 230.0, 210.0], "stack": "total"},
                {"name": "Affiliate Ad",                      "data": [220.0, 182.0, 191.0, 234.0, 290.0, 330.0, 310.0], "stack": "total"},
                {"name": "Video Ad",                          "data": [150.0, 232.0, 201.0, 154.0, 190.0, 330.0, 410.0], "stack": "total"},
                {"name": "Search Engine",                     "data": [820.0, 932.0, 901.0, 934.0,1290.0,1330.0,1320.0], "stack": "total"}
            ],
            "x_axis_data": ["Mon","Tue","Wed","Thu","Fri","Sat","Sun"]
        }"###,
    )
    .unwrap();
    assert_eq!(
        include_str!("../asset/bar_chart/stacked_json.svg"),
        bar_chart.svg().unwrap()
    );
}

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
            "sub_title_align": "right",
            "sub_title_font_weight": "bold",
            "legend_align": "left",
            "legend_font_weight": "bold",
            "y_axis_configs": [
                {
                    "axis_font_weight": "bold"
                }
            ],
            "series_label_font_weight": "bold",
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
                    "data": [320.0, 332.0, 301.0, 334.0, 390.0, 330.0, 320.0],
                    "colors": [null, "#a90000"]
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
            ],
            "x_axis_margin": {
                "left": 1,
                "top": 0,
                "right": 0,
                "bottom": 0
            },
            "x_axis_font_weight": "bold"
        }"###,
    )
    .unwrap();

    assert_eq!(
        include_str!("../asset/bar_chart/basic_json.svg"),
        bar_chart.svg().unwrap()
    );
}

#[test]
fn bar_chart_nil_value() {
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
            "sub_title_align": "right",
            "sub_title_font_weight": "bold",
            "legend_align": "left",
            "legend_font_weight": "bold",
            "y_axis_configs": [
                {
                    "axis_font_weight": "bold"
                }
            ],
            "series_label_font_weight": "bold",
            "series_list": [
                {
                    "name": "Email",
                    "label_show": true,
                    "data": [120.0, null, 101.0, 134.0, 90.0, 230.0, 210.0]
                },
                {
                    "name": "Union Ads",
                    "data": [220.0, 182.0, 191.0, null, 290.0, 330.0, 310.0]
                },
                {
                    "name": "Direct",
                    "data": [320.0, null, 301.0, 334.0, 390.0, 330.0, 320.0]
                },
                {
                    "name": "Search Engine",
                    "data": [820.0, 932.0, 901.0, 934.0, 1290.0, null, 1320.0]
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
            ],
            "x_axis_margin": {
                "left": 1,
                "top": 0,
                "right": 0,
                "bottom": 0
            },
            "x_axis_font_weight": "bold"
        }"###,
    )
    .unwrap();
    assert_eq!(
        include_str!("../asset/bar_chart/nil_value_json.svg"),
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
            "sub_title_align": "right",
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

#[test]
fn bar_chart_narrow_bands() {
    // 24 categories in a 300px canvas: each band is narrower than the
    // per-band margins plus the inter-series gap.
    let bar_chart = BarChart::from_json(
        r###"{
        "width": 300,
        "height": 200,
        "series_list": [
            {
                "name": "Mon",
                "data": [12.0, 18.0, 9.0, 14.0, 22.0, 31.0, 27.0, 19.0, 24.0, 33.0, 28.0, 21.0,
                         17.0, 25.0, 30.0, 26.0, 20.0, 15.0, 23.0, 29.0, 34.0, 22.0, 16.0, 11.0]
            },
            {
                "name": "Tue",
                "data": [14.0, 20.0, 11.0, 16.0, 25.0, 34.0, 30.0, 22.0, 27.0, 36.0, 31.0, 24.0,
                         19.0, 28.0, 33.0, 29.0, 23.0, 17.0, 26.0, 32.0, 37.0, 25.0, 18.0, 13.0]
            }
        ],
        "x_axis_data": [
            "00", "01", "02", "03", "04", "05", "06", "07",
            "08", "09", "10", "11", "12", "13", "14", "15",
            "16", "17", "18", "19", "20", "21", "22", "23"
        ]
    }"###,
    )
    .unwrap();

    let svg = bar_chart.svg().unwrap();
    assert!(!svg.contains(r#"width="-"#), "negative bar width");
    common::assert_bars_in_x_bands(&svg, 24);
    assert_eq!(include_str!("../asset/bar_chart/narrow_bands.svg"), svg);
}

#[test]
fn bar_chart_dense_single_series() {
    // Bands of 4.7px and 2.1px: narrower than the margins alone, so the bars
    // only stay on their own category if the margins shrink with the band.
    for (width, categories) in [(600, 120), (800, 365)] {
        let svg = BarChart::from_json(&common::dense_json(width, 300, categories, 1))
            .unwrap()
            .svg()
            .unwrap();
        common::assert_bars_in_x_bands(&svg, categories);
    }
}
