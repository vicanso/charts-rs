use charts_rs::HorizontalBarChart;
use pretty_assertions::assert_eq;

#[test]
fn horizontal_bar_chart() {
    let horizontal_bar_chart =  HorizontalBarChart::from_json(r###"{
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
    }"###).unwrap();

    assert_eq!(
        include_str!("../asset/horizontal_bar_chart/basic_json.svg"),
        horizontal_bar_chart.svg().unwrap()
    );
}