use charts_rs::{BarChart, Chart};
use pretty_assertions::assert_eq;
use serde_json::json;

#[test]
fn bar_chart() {
    let data = BarChart::from_json(
        r###"{
        "width": 800,
        "height": 600,
        "margin": {
            "left": 10.1
        },
        "series_list": [
            {
                "name": "Email",
                "data": [120.0, 132.0, 101.0, 134.0, 90.0, 230.0, 210.0]
            }
        ]
    }"###,
    )
    .unwrap();

    // let str = serde_json::to_string(&charts_rs::Series {
    //     category: Some(charts_rs::SeriesCategory::Line),
    //     ..Default::default()
    // })
    // .unwrap();

    assert_eq!("", "ab")
}
