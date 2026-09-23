use charts_rs::HorizontalBarChart;
use pretty_assertions::assert_eq;

mod common;

#[test]
fn horizontal_bar_chart() {
    let horizontal_bar_chart = HorizontalBarChart::from_json(
        r###"{
        "title_text": "World Population",
        "title_align": "left",
        "margin": {
            "left": 10,
            "top": 10,
            "right": 20,
            "bottom": 10
        },
        "series_list": [
            {
                "name": "2011",
                "data": [18203.0, 23489.0, 29034.0, 104970.0, 131744.0, 630230.0]
            },
            {
                "name": "2012",
                "data": [19325.0, 23438.0, 31000.0, 121594.0, 134141.0, 681807.0]
            }
        ],
        "x_axis_data": [
            "Brazil",
            "Indonesia",
            "USA",
            "India",
            "China",
            "World"
        ]
    }"###,
    )
    .unwrap();

    assert_eq!(
        include_str!("../asset/horizontal_bar_chart/basic_json.svg"),
        horizontal_bar_chart.svg().unwrap()
    );
}

#[test]
fn horizontal_bar_chart_label_inside() {
    let horizontal_bar_chart = HorizontalBarChart::from_json(
        r###"{
        "title_text": "World Population",
        "title_align": "left",
        "margin": {
            "left": 10,
            "top": 10,
            "right": 20,
            "bottom": 10
        },
        "series_label_position": "left",
        "series_list": [
            {
                "name": "2011",
                "label_show": true,
                "data": [18203.0, 23489.0, 29034.0, 104970.0, 131744.0, 630230.0]
            },
            {
                "name": "2012",
                "label_show": true,
                "data": [19325.0, 23438.0, 31000.0, 121594.0, 134141.0, 681807.0]
            }
        ],
        "x_axis_data": [
            "Brazil",
            "Indonesia",
            "USA",
            "India",
            "China",
            "World"
        ]
    }"###,
    )
    .unwrap();

    assert_eq!(
        include_str!("../asset/horizontal_bar_chart/basic_json_label_left.svg"),
        horizontal_bar_chart.svg().unwrap()
    );
}

#[test]
fn horizontal_bar_chart_nil_value() {
    let horizontal_bar_chart = HorizontalBarChart::from_json(
        r###"{
        "title_text": "World Population",
        "title_align": "left",
        "margin": {
            "left": 10,
            "top": 10,
            "right": 20,
            "bottom": 10
        },
        "series_list": [
            {
                "name": "2011",
                "data": [18203.0, null, 29034.0, 104970.0, 131744.0, 630230.0]
            },
            {
                "name": "2012",
                "data": [19325.0, 23438.0, 31000.0, null, 134141.0, 681807.0]
            }
        ],
        "x_axis_data": [
            "Brazil",
            "Indonesia",
            "USA",
            "India",
            "China",
            "World"
        ]
    }"###,
    )
    .unwrap();

    assert_eq!(
        include_str!("../asset/horizontal_bar_chart/nil_value_json.svg"),
        horizontal_bar_chart.svg().unwrap()
    );
}

#[test]
fn horizontal_bar_chart_narrow_bands() {
    // 24 categories in a 200px canvas: each band is shorter than the
    // per-band margins plus the inter-series gap.
    let horizontal_bar_chart = HorizontalBarChart::from_json(
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

    let svg = horizontal_bar_chart.svg().unwrap();
    assert!(!svg.contains(r#"height="-"#), "negative bar height");
    common::assert_bars_in_y_bands(&svg, 24);
    assert_eq!(
        include_str!("../asset/horizontal_bar_chart/narrow_bands.svg"),
        svg
    );
}

#[test]
fn horizontal_bar_chart_dense_single_series() {
    // Bands of about 1.9px: narrower than the margins alone.
    let svg = HorizontalBarChart::from_json(&common::dense_json(300, 300, 120, 1))
        .unwrap()
        .svg()
        .unwrap();
    common::assert_bars_in_y_bands(&svg, 120);
}
