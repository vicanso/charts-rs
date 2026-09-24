//! `from_json` rejects malformed options instead of silently ignoring them.
use charts_rs::{BarChart, Error, MultiChart, PieChart, TableChart};

fn err(result: Result<impl Sized, Error>) -> String {
    match result {
        Ok(_) => panic!("expected an error"),
        Err(e) => e.to_string(),
    }
}

#[test]
fn unknown_keys_are_rejected_with_a_hint() {
    let e = err(BarChart::from_json(r#"{"tittle_text": "Hello"}"#));
    assert!(e.contains("unknown field `tittle_text`"), "{e}");
    assert!(e.contains("did you mean `title_text`"), "{e}");

    let e = err(BarChart::from_json(
        r#"{"series_list": [{"name": "a", "data": [1], "lable_show": true}]}"#,
    ));
    assert!(e.contains("series_list[0].lable_show"), "{e}");

    let e = err(TableChart::from_json(
        r#"{"data": [["a"]], "boddy_font_size": 12}"#,
    ));
    assert!(e.contains("body_font_size"), "{e}");
}

#[test]
fn wrong_types_and_values_are_rejected() {
    assert!(err(BarChart::from_json(r#"{"width": "600"}"#)).contains("`width`"));
    assert!(err(BarChart::from_json(r#"{"width": 0}"#)).contains("positive"));
    assert!(err(BarChart::from_json(r#"{"width": -100}"#)).contains("positive"));
    assert!(
        err(BarChart::from_json(r#"{"title_align": "lefft"}"#)).contains("left, center, right")
    );
    assert!(err(BarChart::from_json(r#"{"title_font_color": "red"}"#)).contains("color"));
    assert!(
        err(BarChart::from_json(
            r#"{"y_axis_configs": [{"axis_split_number": 2000000}]}"#
        ))
        .contains("axis_split_number")
    );
    assert!(
        err(BarChart::from_json(
            r#"{"series_list": [{"name": "a", "data": [1], "start_index": 4000000000000000000}]}"#
        ))
        .contains("start_index")
    );
    assert!(err(BarChart::from_json(r#"[1, 2]"#)).contains("JSON object"));
}

#[test]
fn unknown_theme_is_rejected() {
    let e = err(BarChart::from_json(r#"{"theme": "nope"}"#));
    assert!(e.contains("unknown theme `nope`"), "{e}");
    assert!(BarChart::from_json(r#"{"theme": "grafana"}"#).is_ok());
    assert!(BarChart::from_json(r#"{"theme": ""}"#).is_ok());
    assert!(TableChart::from_json(r#"{"theme": "nope", "data": [["a"]]}"#).is_err());
}

#[test]
fn enum_values_are_case_insensitive() {
    let chart = BarChart::from_json(
        r#"{"title_align": "Right", "series_list": [{"name": "a", "data": [1], "mark_lines": [{"category": "Max"}]}]}"#,
    )
    .unwrap();
    assert_eq!(charts_rs::Align::Right, chart.title_align);
    assert_eq!(
        charts_rs::MarkLineCategory::Max,
        chart.series_list[0].mark_lines[0].category
    );
    let chart = PieChart::from_json(
        r#"{"series_list": [{"name": "a", "data": [1]}], "start_angle": 90, "series_label_position": "Inside"}"#,
    )
    .unwrap();
    assert_eq!(90.0, chart.start_angle);
    assert_eq!(Some("inside".to_string()), chart.series_label_position);
}

#[test]
fn null_and_numeric_margins_are_accepted() {
    let chart = BarChart::from_json(
        r#"{"width": null, "margin": 12, "title_margin": {"top": 3}, "series_list": [{"name": "a", "data": [1]}]}"#,
    )
    .unwrap();
    assert_eq!(12.0, chart.margin.left);
    assert_eq!(12.0, chart.margin.bottom);
    assert_eq!(3.0, chart.title_margin.as_ref().unwrap().top);
}

#[test]
fn multi_chart_children_keep_placement_keys() {
    let chart = MultiChart::from_json(
        r#"{"gap": 5, "child_charts": [
            {"type": "line", "x": 10, "y": 20, "series_list": [{"name": "a", "data": [1, 2]}], "x_axis_data": ["a", "b"]},
            {"type": "pie", "series_list": [{"name": "a", "data": [1]}]}
        ]}"#,
    )
    .unwrap();
    chart.svg().unwrap();
    let e = err(MultiChart::from_json(r#"{"gapp": 5, "child_charts": []}"#));
    assert!(e.contains("did you mean `gap`"), "{e}");
    let e = err(MultiChart::from_json(
        r#"{"child_charts": [{"type": "line", "seris_list": []}]}"#,
    ));
    assert!(e.contains("seris_list"), "{e}");
}
