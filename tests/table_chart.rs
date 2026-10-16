mod common;

use charts_rs::{Color, LineChart, NIL_VALUE, SparklineType, TableChart, TableSparkline};

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
    assert_eq!(Some("bold".to_string()), chart.body.font.weight);
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

fn attr<'a>(tag: &'a str, name: &str) -> Option<&'a str> {
    let key = format!(" {name}=\"");
    let start = tag.find(&key)? + key.len();
    Some(&tag[start..start + tag[start..].find('"')?])
}

fn number(tag: &str, name: &str) -> f32 {
    attr(tag, name).unwrap().parse().unwrap()
}

/// The tags of the given element.
fn tags<'a>(svg: &'a str, element: &str) -> Vec<&'a str> {
    svg.split(&format!("<{element}"))
        .skip(1)
        .map(|tag| &tag[..tag.find('>').unwrap()])
        .collect()
}

/// The points of every polyline.
fn lines(svg: &str) -> Vec<Vec<(f32, f32)>> {
    tags(svg, "polyline")
        .into_iter()
        .map(|tag| {
            attr(tag, "points")
                .unwrap()
                .split(' ')
                .map(|point| {
                    let (x, y) = point.split_once(',').unwrap();
                    (x.parse().unwrap(), y.parse().unwrap())
                })
                .collect()
        })
        .collect()
}

/// Where the text of a cell is: `(x, y)`.
fn text_at(svg: &str, text: &str) -> (f32, f32) {
    let ending = format!(">\n{text}\n</text>");
    let end = svg.find(&ending).unwrap_or_else(|| panic!("{text:?}"));
    let tag = &svg[svg[..end].rfind("<text").unwrap()..end];
    (number(tag, "x"), number(tag, "y"))
}

#[test]
fn table_sparkline_json() {
    let table = TableChart::from_json(include_str!("../asset/table_chart/sparkline.json")).unwrap();
    assert_eq!(4, table.sparklines.len());
    common::assert_snapshot!("table_chart/sparkline_json.svg", table.svg().unwrap());
}

#[test]
fn a_cell_of_numbers_is_a_sparkline() {
    let table = TableChart::from_json(
        r#"{"spans": [100, 100], "data": [
            ["Name", "Trend", "Count"],
            ["a", [1, 3, 2, 5], 12],
            ["b", [4, null, 6, 7], null]]}"#,
    )
    .unwrap();
    // The cell keeps its place in the row, as does every other: a number is
    // written as it is, `null` is an empty cell.
    assert_eq!(
        vec![
            vec!["Name", "Trend", "Count"],
            vec!["a", "", "12"],
            vec!["b", "", ""],
        ],
        table.data
    );
    assert_eq!(
        vec![
            TableSparkline {
                indexes: vec![1, 1],
                values: vec![1.0, 3.0, 2.0, 5.0],
            },
            TableSparkline {
                indexes: vec![2, 1],
                values: vec![4.0, NIL_VALUE, 6.0, 7.0],
            },
        ],
        table.sparklines
    );
    let svg = table.svg().unwrap();

    // A line across the cell, a little inside it as its text would be: the
    // second column starts at 100 and its text at 110, 10 from its edges.
    let found = lines(&svg);
    assert_eq!(2, found.len());
    let first = &found[0];
    assert_eq!(4, first.len());
    assert_eq!((110.0, 190.0), (first[0].0, first[3].0));
    // Evenly along it, from the lowest value at the bottom to the highest at
    // the top of the first line of the cell: 30 high, 15% in from each end.
    assert!((first[1].0 - first[0].0 - 80.0 / 3.0).abs() < 0.1);
    let (_, text_y) = text_at(&svg, "a");
    let (bottom, top) = (first[0].1, first[3].1);
    assert!((bottom - top - 30.0 * 0.7).abs() < 0.1, "{bottom} {top}");
    assert!(((bottom + top) / 2.0 - text_y).abs() < 16.0);
    // A gap where a value is missing: the line goes on after it.
    assert_eq!(2, found[1].len());
    // A dot on the last value of each, and on a value that stands alone.
    let dots = tags(&svg, "circle");
    assert_eq!(3, dots.len());
    assert_eq!((190.0, top), (number(dots[0], "cx"), number(dots[0], "cy")));
    // In the first color of the palette.
    assert!(
        tags(&svg, "polyline")
            .iter()
            .all(|t| attr(t, "stroke") == Some("#5470C6"))
    );

    // The text of the other cells is where it would be.
    let plain = TableChart::from_json(
        r#"{"spans": [100, 100], "data": [["Name", "Trend", "Count"], ["a", "", "12"], ["b", "", ""]]}"#,
    )
    .unwrap()
    .svg()
    .unwrap();
    for text in ["a", "b", "12", "Count"] {
        assert_eq!(text_at(&plain, text), text_at(&svg, text), "{text}");
    }
}

#[test]
fn sparklines_as_bars_and_in_rust() {
    let svg = TableChart::from_json(
        r##"{"spans": [100, 100], "sparkline_type": "bar", "sparkline_color": "#123456",
            "data": [["Name", "Trend"], ["a", [2, -1, 4, null]]]}"##,
    )
    .unwrap()
    .svg()
    .unwrap();
    let bars: Vec<&str> = tags(&svg, "rect")
        .into_iter()
        .filter(|tag| attr(tag, "fill") == Some("#123456"))
        .collect();
    // A bar for every value there is, from 0: up for one above it, down for
    // one below, in four slots of 20 pixels.
    assert_eq!(3, bars.len());
    let zero = number(bars[0], "y") + number(bars[0], "height");
    assert!((number(bars[1], "y") - zero).abs() < 0.1);
    assert!(number(bars[2], "height") > number(bars[0], "height"));
    let (x0, x1) = (number(bars[0], "x"), number(bars[1], "x"));
    assert!((x1 - x0 - 20.0).abs() < 0.1 && (number(bars[0], "width") - 14.0).abs() < 0.1);
    assert!(lines(&svg).is_empty());

    // In Rust, and in a header cell as well.
    let mut table = TableChart::new(vec![
        vec!["Trend".to_string(), "Name".to_string()],
        vec!["".to_string(), "a".to_string()],
    ]);
    table.sparklines = vec![
        TableSparkline {
            indexes: vec![1, 0],
            values: vec![1.0, 2.0, 1.5],
        },
        TableSparkline {
            indexes: vec![0, 1],
            values: vec![1.0, f32::NAN, f32::INFINITY],
        },
    ];
    table.sparkline_color = Some(Color::from("#ff0000"));
    assert_eq!(SparklineType::Line, table.sparkline_type);
    let svg = table.svg().unwrap();
    // The one of the header has a single value: a dot, and no line, in place
    // of its text.
    assert_eq!(1, lines(&svg).len());
    assert_eq!(2, tags(&svg, "circle").len());
    assert!(!svg.contains(">\nName\n</text>"));
    assert!(!svg.contains("NaN") && !svg.contains("inf"));
    // A sparkline of nothing draws nothing, and one of a cell that is not
    // there is not drawn.
    table.sparklines = vec![
        TableSparkline {
            indexes: vec![1, 0],
            values: vec![],
        },
        TableSparkline {
            indexes: vec![5, 5],
            values: vec![1.0, 2.0],
        },
    ];
    let svg = table.svg().unwrap();
    assert!(lines(&svg).is_empty() && tags(&svg, "circle").is_empty());
}

#[test]
fn a_line_without_markers() {
    // A symbol of type `none` is no marker, as `null` is: a line chart that
    // is a sparkline of its own.
    let chart = |symbol: &str| {
        LineChart::from_json(&format!(
            r#"{{"series_symbol": {symbol}, "x_axis_data": ["a", "b", "c"],
                "series_list": [{{"name": "s", "data": [1, 3, 2]}}]}}"#
        ))
        .unwrap()
        .svg()
        .unwrap()
    };
    let none = chart(r#"{"type": "none"}"#);
    assert_eq!(chart("null"), none);
    // The legend has its circle; the points have none.
    assert_eq!(1, tags(&none, "circle").len());
    assert_eq!(4, tags(&chart(r#"{"type": "circle"}"#), "circle").len());
}
