use charts_rs::TableChart;
use pretty_assertions::assert_eq;

#[test]
fn table_chart() {
    let mut table_chart = TableChart::from_json(
        r###"{
        "theme": "grafana",
        "title_text": "NASDAQ",
        "data": [
            [
                "Name",
                "Price",
                "Change"
            ],
            [
                "Datadog Inc",
                "97.32",
                "-7.49%"
            ],
            [
                "Hashicorp Inc",
                "28.66",
                "-9.25%"
            ],
            [
                "Gitlab Inc",
                "51.63",
                "+4.32%"
            ]
        ],
        "header_font_weight": "bold",
        "text_aligns": ["left", "center", "right"],
        "cell_styles": [
            {
                "font_color": "#fff", 
                "font_weight": "bold", 
                "background_color": "#2d7c2b",
                "indexes": [1, 2] 
            }
        ]
    }"###,
    )
    .unwrap();
    assert_eq!(
        include_str!("../asset/table_chart/basic_json.svg"),
        table_chart.svg().unwrap()
    );
}
