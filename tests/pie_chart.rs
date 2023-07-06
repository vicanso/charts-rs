use charts_rs::PieChart;
use pretty_assertions::assert_eq;

#[test]
fn pie_chart() {
    let pie_chart = PieChart::from_json(
        r###"{
        "title_text": "Nightingale Chart",
        "sub_title_text": "Fake Data",
        "legend_show": false,
        "series_list": [
            {
                "name": "rose 1",
                "data": [40]
            },
            {
                "name": "rose 2",
                "data": [38]
            },
            {
                "name": "rose 3",
                "data": [32]
            },
            {
                "name": "rose 4",
                "data": [30]
            },
            {
                "name": "rose 5",
                "data": [28]
            },
            {
                "name": "rose 6",
                "data": [26]
            },
            {
                "name": "rose 7",
                "data": [22]
            },
            {
                "name": "rose 8",
                "data": [18]
            }
        ]
    }"###,
    )
    .unwrap();
    assert_eq!(
        include_str!("../asset/pie_chart/basic_json.svg"),
        pie_chart.svg().unwrap()
    );
}
