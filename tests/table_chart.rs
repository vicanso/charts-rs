mod common;

use charts_rs::TableChart;

#[test]
fn table_chart() {
    let table_chart = TableChart::from_json(
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
    common::assert_snapshot!("table_chart/basic_json.svg", table_chart.svg().unwrap());
}

#[test]
fn table_body_font_weight_and_background() {
    let json = |extra: &str| {
        format!(
            r##"{{"data": [["Name", "Price"], ["Datadog Inc", "97.32"], ["Gitlab Inc", "51.63"]]{extra}}}"##
        )
    };
    let plain = TableChart::from_json(&json("")).unwrap().svg().unwrap();
    assert_eq!(0, plain.matches(r#"font-weight="bold""#).count());

    // The weight of the body rows, as the header has one of its own.
    let chart = TableChart::from_json(&json(r#", "body_font_weight": "bold""#)).unwrap();
    assert_eq!(Some("bold".to_string()), chart.body_font_weight);
    let svg = chart.svg().unwrap();
    assert_eq!(4, svg.matches(r#"font-weight="bold""#).count());
    // A header weight wins in the header, a cell style in its cell.
    let svg = TableChart::from_json(&json(
        r#", "body_font_weight": "bold", "header_font_weight": "lighter",
            "cell_styles": [{"indexes": [1, 0], "font_weight": "normal"}]"#,
    ))
    .unwrap()
    .svg()
    .unwrap();
    assert_eq!(3, svg.matches(r#"font-weight="bold""#).count());
    assert_eq!(2, svg.matches(r#"font-weight="lighter""#).count());
    assert_eq!(1, svg.matches(r#"font-weight="normal""#).count());

    // The background behind the title; the rows have colors of their own.
    let chart = TableChart::from_json(&json(
        r##", "title_text": "Stocks", "background_color": "#123456""##,
    ))
    .unwrap();
    let svg = chart.svg().unwrap();
    assert!(!plain.contains("#123456"));
    assert_eq!(1, svg.matches(r##"fill="#123456""##).count());

    // The keys of a cell style belong in `cell_styles`, not next to it.
    for key in [
        r#""font_weight": "bold""#,
        r##""font_color": "#fff""##,
        r#""indexes": [1, 0]"#,
    ] {
        let message = match TableChart::from_json(&json(&format!(", {key}"))) {
            Ok(_) => panic!("{key} is accepted"),
            Err(e) => e.to_string(),
        };
        assert!(message.contains("unknown field"), "{message}");
    }
}
