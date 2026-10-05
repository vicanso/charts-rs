//! Color scales of several colors, and of classes, in heatmaps and calendars.
mod common;

use charts_rs::{CalendarChart, Error, HeatmapChart};

fn attr<'a>(tag: &'a str, name: &str) -> &'a str {
    let key = format!(" {name}=\"");
    let start = tag.find(&key).map(|i| i + key.len()).unwrap();
    let end = start + tag[start..].find('"').unwrap();
    &tag[start..end]
}

/// The fill of the cell that shows `value`.
fn fill_of<'a>(svg: &'a str, value: &str) -> &'a str {
    let cell = svg
        .split("<rect")
        .find(|t| t.contains(&format!("data-value=\"{value}\"")))
        .unwrap_or_else(|| panic!("no cell for {value}"));
    attr(cell, "fill")
}

fn err(result: Result<impl Sized, Error>) -> String {
    match result {
        Ok(_) => panic!("expected an error"),
        Err(e) => e.to_string(),
    }
}

/// A heatmap of the values 0, 25, 50, 75 and 100 on a scale from 0 to 100.
fn heatmap(scale: &str) -> String {
    let json = format!(
        r##"{{"x_axis_data": ["a", "b", "c", "d", "e"], "y_axis_data": ["y"],
            "series": {{"min": 0, "max": 100, "min_color": "#ffffff", "max_color": "#000000"{scale},
            "data": [[0, 0], [1, 25], [2, 50], [3, 75], [4, 100]]}}}}"##
    );
    HeatmapChart::from_json(&json).unwrap().svg().unwrap()
}

/// A calendar of the same values, one a day.
fn calendar(scale: &str) -> String {
    let json = format!(
        r##"{{"start_date": "2024-01-01", "end_date": "2024-01-07", "min": 0, "max": 100,
            "min_color": "#ffffff", "max_color": "#000000"{scale},
            "data": [["2024-01-01", 1], ["2024-01-02", 25], ["2024-01-03", 50],
                     ["2024-01-04", 75], ["2024-01-05", 100]]}}"##
    );
    CalendarChart::from_json(&json).unwrap().svg().unwrap()
}

#[test]
fn heatmap_scale_snapshot() {
    let chart = HeatmapChart::from_json(include_str!("../asset/heatmap_chart/scale.json")).unwrap();
    assert_eq!(5, chart.heatmap_series.colors.len());
    assert_eq!(
        vec![50.0, 100.0, 150.0, 200.0],
        chart.heatmap_series.thresholds
    );
    common::assert_snapshot!("heatmap_chart/scale_json.svg", chart.svg().unwrap());
}

#[test]
fn scale_goes_through_every_color() {
    let colors = r##", "colors": ["#ff0000", "#ffffff", "#0000ff"]"##;
    for svg in [heatmap(colors), calendar(colors)] {
        // The middle color is the middle of the scale; `min_color` and
        // `max_color` are not used.
        assert_eq!("#FFFFFF", fill_of(&svg, "50"));
        assert_eq!("#FF8080", fill_of(&svg, "25"));
        assert_eq!("#8080FF", fill_of(&svg, "75"));
        assert_eq!("#0000FF", fill_of(&svg, "100"));
        assert!(!svg.contains("#000000"));
    }
    assert_eq!("#FF0000", fill_of(&heatmap(colors), "0"));

    // A single color is not a scale: the two ends are used as before.
    let single = r##", "colors": ["#ff0000"]"##;
    assert_eq!(heatmap(""), heatmap(single));
    assert_eq!(calendar(""), calendar(single));
}

#[test]
fn classes_of_the_same_width() {
    let steps = r#", "steps": 2"#;
    for svg in [heatmap(steps), calendar(steps)] {
        // Two classes, each in the color of one end of the scale.
        assert_eq!("#FFFFFF", fill_of(&svg, "25"));
        assert_eq!("#000000", fill_of(&svg, "50"));
        assert_eq!("#000000", fill_of(&svg, "75"));
        assert_eq!("#000000", fill_of(&svg, "100"));
    }
    // As many colors as classes: each class has one of them.
    let four = r##", "steps": 4, "colors": ["#111111", "#222222", "#333333", "#444444"]"##;
    for svg in [heatmap(four), calendar(four)] {
        assert_eq!("#222222", fill_of(&svg, "25"));
        assert_eq!("#333333", fill_of(&svg, "50"));
        assert_eq!("#444444", fill_of(&svg, "75"));
        assert_eq!("#444444", fill_of(&svg, "100"));
    }
    assert_eq!("#111111", fill_of(&heatmap(four), "0"));
    // One class, or none, leaves the scale continuous.
    assert_eq!(heatmap(""), heatmap(r#", "steps": 1"#));
    assert_eq!(calendar(""), calendar(r#", "steps": 0"#));
}

#[test]
fn classes_between_thresholds() {
    // Three classes of different widths; `steps` gives way.
    let thresholds = r##", "steps": 9, "thresholds": [60, 20],
        "colors": ["#111111", "#222222", "#333333"]"##;
    for svg in [heatmap(thresholds), calendar(thresholds)] {
        assert_eq!("#222222", fill_of(&svg, "25"));
        assert_eq!("#222222", fill_of(&svg, "50"));
        assert_eq!("#333333", fill_of(&svg, "75"));
        assert_eq!("#333333", fill_of(&svg, "100"));
    }
    assert_eq!("#111111", fill_of(&heatmap(thresholds), "0"));
    assert_eq!("#111111", fill_of(&calendar(thresholds), "1"));
    // Without colors of their own the classes are spread from `min_color`
    // to `max_color`.
    let plain = r#", "thresholds": [20, 60]"#;
    assert_eq!("#808080", fill_of(&heatmap(plain), "50"));
    assert_eq!("#000000", fill_of(&heatmap(plain), "75"));
}

#[test]
fn heatmap_labels_stay_legible() {
    // Dark in the middle, light at both ends: the light font where the
    // cell is dark, not where the value is high.
    let svg = heatmap(
        r##", "colors": ["#ffffff", "#000000", "#ffffff"],
            "min_font_color": "#111111", "max_font_color": "#eeeeee""##,
    );
    let font = |value: &str| {
        let label = svg
            .split("<text")
            .find(|t| t.contains(&format!("\n{value}\n")))
            .unwrap();
        attr(label, "fill").to_string()
    };
    assert_eq!("#EEEEEE", font("50"));
    assert_eq!("#111111", font("100"));
    assert_eq!("#111111", font("0"));
}

#[test]
fn scale_from_json() {
    let message = err(HeatmapChart::from_json(
        r#"{"x_axis_data": ["a"], "y_axis_data": ["y"], "series": {"colors": ["red"]}}"#,
    ));
    assert!(message.contains("series.colors"), "{message}");
    let message = err(HeatmapChart::from_json(
        r#"{"x_axis_data": ["a"], "y_axis_data": ["y"], "series": {"steps": -1}}"#,
    ));
    assert!(message.contains("series.steps"), "{message}");
    let message = err(CalendarChart::from_json(r#"{"thresholds": "10"}"#));
    assert!(message.contains("thresholds"), "{message}");

    let chart = CalendarChart::from_json(
        r##"{"colors": ["#ebedf0", "#216e39"], "steps": 4, "thresholds": [1, 5, 10]}"##,
    )
    .unwrap();
    assert_eq!(
        (2, 4, vec![1.0, 5.0, 10.0]),
        (chart.colors.len(), chart.steps, chart.thresholds.clone())
    );
}
