mod common;

use charts_rs::HeatmapChart;
use pretty_assertions::assert_eq;

#[test]
fn heatmap_chart() {
    let heatmap_chart = HeatmapChart::from_json(
        r###"{
            "theme": "grafana",
            "y_axis_data": [
                "Saturday",
                "Friday",
                "Thursday",
                "Wednesday",
                "Tuesday",
                "Monday",
                "Sunday"
            ],
            "x_axis_data": [
                "12a", "1a", "2a", "3a", "4a", "5a", "6a", "7a", "8a", "9a", "10a", "11a", "12p", "1p",
                "2p", "3p", "4p", "5p", "6p", "7p", "8p", "9p", "10p", "11p"
            ],
            "series": {
                "data": [
                    [0, 9.0],
                    [1, 3.0],
                    [7, 3.0],
                    [12, 3.0],
                    [24, 12.0],
                    [28, 10.0],
                    [31, 8.0],
                    [50, 4.0],
                    [63, 2.0]
                ]
            }
    }"###,
    )
    .unwrap();
    common::assert_snapshot!(
        "heatmap_chart/basic_grafana_json.svg",
        heatmap_chart.svg().unwrap()
    );
}

#[test]
fn heatmap_chart_cell_dataset() {
    let chart = HeatmapChart::from_json(
        r###"{
            "width": 300,
            "height": 200,
            "x_axis_data": ["a", "b"],
            "y_axis_data": ["r0", "r1"],
            "series": {
                "data": [[0, 1.0], [3, 4.0]]
            }
        }"###,
    )
    .unwrap();
    let svg = chart.svg().unwrap();

    // `data-index` is the flat index the series data is keyed by
    assert_eq!(4, svg.matches("data-index=").count());
    assert_eq!(2, svg.matches("data-value=").count());
    assert!(svg.contains(r#"data-index="0" data-x="a" data-y="r0" data-value="1""#));
    assert!(svg.contains(r#"data-index="3" data-x="b" data-y="r1" data-value="4""#));
    assert!(svg.contains(r#"data-index="1" data-x="b" data-y="r0"/>"#));
}

#[test]
fn heatmap_punch_card() {
    let chart =
        HeatmapChart::from_json(include_str!("../asset/heatmap_chart/punch_card.json")).unwrap();
    assert_eq!(
        charts_rs::HeatmapSymbol::Circle,
        chart.heatmap_series.symbol
    );
    common::assert_snapshot!("heatmap_chart/punch_card_json.svg", chart.svg().unwrap());

    let json = |symbol: &str| {
        format!(
            r##"{{"x_axis_data": ["a", "b", "c", "d"], "y_axis_data": ["y1", "y2"], "tooltip_show": true,
                "series": {{"min": 0, "max": 100{symbol}, "data": [[0, 100], [1, 25], [2, 0], [7, 50]]}}}}"##
        )
    };
    let number = |tag: &str, name: &str| -> f32 {
        let key = format!(" {name}=\"");
        let start = tag.find(&key).unwrap() + key.len();
        tag[start..start + tag[start..].find('"').unwrap()]
            .parse()
            .unwrap()
    };
    let svg = HeatmapChart::from_json(&json(r#", "symbol": "Circle""#))
        .unwrap()
        .svg()
        .unwrap();
    // A circle for every cell that has a value, none for the others; no
    // cell is filled.
    let circles: Vec<&str> = svg
        .split("<circle")
        .skip(1)
        .filter(|t| t.contains("data-value="))
        .collect();
    assert_eq!(4, circles.len());
    assert_eq!(1, svg.matches("<rect").count());
    // The area tells the value: a quarter of it is half the radius.
    let radius: Vec<f32> = circles.iter().map(|c| number(c, "r")).collect();
    assert!((radius[1] - radius[0] / 2.0).abs() < 0.2, "{radius:?}");
    assert!(
        (radius[3] - radius[0] * 0.5_f32.sqrt()).abs() < 0.2,
        "{radius:?}"
    );
    // Nothing is still a dot, so the cell is seen to have a value.
    assert_eq!(1.5, radius[2]);
    // The circles of a row lie on its line; the second row is above.
    assert_eq!(number(circles[0], "cy"), number(circles[1], "cy"));
    assert!(number(circles[3], "cy") < number(circles[0], "cy"));
    assert!(number(circles[0], "cx") < number(circles[1], "cx"));
    assert!(svg.contains("<title>a, y1: 100</title>"));
    // In the color of the value, without its number written on it.
    assert_ne!(
        circles[0].split(" fill=\"").nth(1).unwrap()[..7].to_string(),
        circles[1].split(" fill=\"").nth(1).unwrap()[..7].to_string()
    );
    assert!(!svg.lines().any(|l| l.trim() == "25"));

    // Cells by default, as before.
    let cells = HeatmapChart::from_json(&json("")).unwrap().svg().unwrap();
    assert_eq!(
        cells,
        HeatmapChart::from_json(&json(r#", "symbol": "rect""#))
            .unwrap()
            .svg()
            .unwrap()
    );
    assert_eq!(9, cells.matches("<rect").count());
    assert!(cells.lines().any(|l| l.trim() == "25"));

    let message = match HeatmapChart::from_json(&json(r#", "symbol": "square""#)) {
        Ok(_) => panic!("accepted"),
        Err(e) => e.to_string(),
    };
    assert!(message.contains("series.symbol"), "{message}");
}
