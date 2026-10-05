//! The options of a chart are the same in Rust and in JSON: a JSON key is
//! the path of the Rust field, joined with `_`.

use charts_rs::{
    Align, AxisType, BarChart, LineChart, PieChart, Position, Series, Symbol, TableChart, Theme,
    add_theme, get_theme,
};

fn labels() -> Vec<String> {
    ["a", "b", "c"].iter().map(|s| s.to_string()).collect()
}

fn series() -> Vec<Series> {
    let mut series: Series = ("s", vec![1.0, 2.0, 3.0]).into();
    series.label_show = true;
    vec![series]
}

#[test]
fn json_keys_are_field_paths() {
    let from_json = LineChart::from_json(
        r##"{
            "title_text": "Title",
            "title_font_size": 20,
            "title_font_color": "#123456",
            "title_font_weight": "bold",
            "title_align": "left",
            "title_height": 36,
            "sub_title_text": "Sub-title",
            "sub_title_font_size": 11,
            "sub_title_align": "left",
            "legend_position": "bottom",
            "legend_font_size": 11,
            "legend_font_color": "#444444",
            "legend_show": true,
            "x_axis_data": ["a", "b", "c"],
            "x_axis_font_size": 12,
            "x_axis_font_color": "#654321",
            "x_axis_font_weight": "bold",
            "x_axis_stroke_color": "#0000ff",
            "x_axis_stroke_width": 2,
            "x_axis_title": "letters",
            "x_boundary_gap": false,
            "grid_stroke_color": "#ff0000",
            "grid_stroke_width": 2,
            "grid_stroke_dash_array": "4,2",
            "series_colors": ["#00aa00"],
            "series_stroke_width": 3,
            "series_smooth": true,
            "series_fill": true,
            "series_fill_opacity": 0.5,
            "series_label_font_size": 9,
            "series_label_font_color": "#aa00aa",
            "series_label_formatter": "{c}!",
            "tooltip_show": true,
            "tooltip_font_size": 13,
            "tooltip_font_color": "#101010",
            "tooltip_font_weight": "bold",
            "y_axis_configs": [{
                "axis_stroke_color": "#00aaaa",
                "axis_stroke_width": 2,
                "axis_min": 0,
                "axis_max": 10,
                "axis_split_number": 5,
                "axis_font_size": 10,
                "axis_font_color": "#00aaaa",
                "axis_formatter": "{c}u",
                "axis_title": "units"
            }],
            "series_list": [{"name": "s", "data": [1, 2, 3], "label_show": true}]
        }"##,
    )
    .unwrap();

    let mut chart = LineChart::new(series(), labels());
    chart.title.text = "Title".to_string();
    chart.title.font.size = 20.0;
    chart.title.font.color = "#123456".into();
    chart.title.font.weight = Some("bold".to_string());
    chart.title.align = Align::Left;
    chart.title.height = 36.0;
    chart.sub_title.text = "Sub-title".to_string();
    chart.sub_title.font.size = 11.0;
    chart.sub_title.align = Align::Left;
    chart.legend.position = Some(Position::Bottom);
    chart.legend.font.size = 11.0;
    chart.legend.font.color = "#444444".into();
    chart.legend.show = Some(true);
    chart.x_axis.font.size = 12.0;
    chart.x_axis.font.color = "#654321".into();
    chart.x_axis.font.weight = Some("bold".to_string());
    chart.x_axis.stroke_color = "#0000ff".into();
    chart.x_axis.stroke_width = Some(2.0);
    chart.x_axis.title = "letters".to_string();
    chart.x_axis.boundary_gap = Some(false);
    chart.grid.stroke_color = "#ff0000".into();
    chart.grid.stroke_width = 2.0;
    chart.grid.stroke_dash_array = Some("4,2".to_string());
    chart.series.colors = vec!["#00aa00".into()];
    chart.series.stroke_width = 3.0;
    chart.series.smooth = true;
    chart.series.fill = true;
    chart.series.fill_opacity = Some(0.5);
    chart.series.label.font.size = 9.0;
    chart.series.label.font.color = "#aa00aa".into();
    chart.series.label.formatter = "{c}!".to_string();
    chart.tooltip.show = true;
    chart.tooltip.font.size = 13.0;
    chart.tooltip.font.color = "#101010".into();
    chart.tooltip.font.weight = Some("bold".to_string());
    let y_axis = &mut chart.y_axis_configs[0];
    y_axis.stroke_color = "#00aaaa".into();
    y_axis.stroke_width = Some(2.0);
    y_axis.min = Some(0.0);
    y_axis.max = Some(10.0);
    y_axis.split_number = 5;
    y_axis.font.size = 10.0;
    y_axis.font.color = "#00aaaa".into();
    y_axis.formatter = Some("{c}u".to_string());
    y_axis.title = Some("units".to_string());

    assert_eq!(from_json.title, chart.title);
    assert_eq!(from_json.sub_title, chart.sub_title);
    assert_eq!(from_json.legend, chart.legend);
    assert_eq!(from_json.x_axis, chart.x_axis);
    assert_eq!(from_json.grid, chart.grid);
    assert_eq!(from_json.series, chart.series);
    assert_eq!(from_json.tooltip, chart.tooltip);
    assert_eq!(from_json.y_axis_configs, chart.y_axis_configs);
    assert_eq!(from_json.svg().unwrap(), chart.svg().unwrap());

    // The one key that is not the name of its field.
    let timed = LineChart::from_json(r#"{"x_axis_type": "time"}"#).unwrap();
    assert_eq!(AxisType::Time, timed.x_axis.kind);
}

#[test]
fn table_keys_are_field_paths() {
    let data = r#"[["Name", "Price"], ["A", "1"], ["B", "2"]]"#;
    let from_json = TableChart::from_json(&format!(
        r##"{{
            "data": {data},
            "header_row_height": 40,
            "header_font_size": 15,
            "header_font_color": "#111111",
            "header_font_weight": "bold",
            "header_background_color": "#eeeeee",
            "body_row_height": 28,
            "body_font_size": 12,
            "body_font_color": "#222222",
            "body_background_colors": ["#ffffff", "#f5f5f5"]
        }}"##
    ))
    .unwrap();

    let mut chart = TableChart::new(serde_json::from_str(data).unwrap());
    chart.header.row_height = 40.0;
    chart.header.font.size = 15.0;
    chart.header.font.color = "#111111".into();
    chart.header.font.weight = Some("bold".to_string());
    chart.header.background_color = "#eeeeee".into();
    chart.body.row_height = 28.0;
    chart.body.font.size = 12.0;
    chart.body.font.color = "#222222".into();
    chart.body.background_colors = vec!["#ffffff".into(), "#f5f5f5".into()];

    assert_eq!(from_json.header, chart.header);
    assert_eq!(from_json.body, chart.body);
    assert_eq!(from_json.svg().unwrap(), chart.svg().unwrap());
}

fn theme_from(name: &str) -> Theme {
    (*get_theme(name)).clone()
}

#[test]
fn a_theme_holds_any_shared_option() {
    let mut theme = theme_from("light");
    theme.title.align = Align::Left;
    theme.legend.position = Some(Position::Bottom);
    theme.x_axis.font.weight = Some("bold".to_string());
    theme.y_axis.split_number = 3;
    theme.series.smooth = true;
    theme.series.symbol = Some(Symbol::None);
    theme.series.label.formatter = "{c}%".to_string();
    // The labels of an axis are those of a chart, never those of a theme.
    theme.x_axis.data = vec!["not".to_string(), "these".to_string()];
    add_theme("options-test-any", theme);

    let chart = BarChart::new_with_theme(series(), labels(), "options-test-any");
    assert_eq!(Align::Left, chart.title.align);
    assert_eq!(Some(Position::Bottom), chart.legend.position);
    assert_eq!(Some("bold".to_string()), chart.x_axis.font.weight);
    assert_eq!(3, chart.y_axis_configs[0].split_number);
    assert!(chart.series.smooth);
    assert_eq!(Some(Symbol::None), chart.series.symbol);
    assert_eq!("{c}%", chart.series.label.formatter);
    assert_eq!(labels(), chart.x_axis.data);
    let svg = chart.svg().unwrap();
    assert!(svg.contains("font-weight=\"bold\"") && svg.contains("3%"));

    // From JSON the theme comes first and the keys after it.
    let chart = BarChart::from_json(
        r#"{"theme": "options-test-any", "legend_position": "top", "x_axis_data": ["a"],
            "series_list": [{"name": "s", "data": [1]}]}"#,
    )
    .unwrap();
    assert_eq!(Some(Position::Top), chart.legend.position);
    assert_eq!(Align::Left, chart.title.align);
    assert_eq!(vec!["a".to_string()], chart.x_axis.data);

    // Without a symbol of the theme, the points of a line are circles of
    // the stroke width on the background.
    let plain = LineChart::new(series(), labels());
    assert_eq!(
        Some(Symbol::Circle(
            plain.series.stroke_width,
            Some(plain.background_color)
        )),
        plain.series.symbol
    );
}

#[test]
fn a_pie_has_no_legend_until_told() {
    let slices = || -> Vec<Series> { vec![("a", vec![1.0]).into(), ("b", vec![2.0]).into()] };
    assert_eq!(Some(false), PieChart::new(slices()).legend.show);
    let json = |extra: &str| {
        format!(
            r#"{{"series_list": [{{"name": "a", "data": [1]}}, {{"name": "b", "data": [2]}}]{extra}}}"#
        )
    };
    assert_eq!(
        Some(false),
        PieChart::from_json(&json("")).unwrap().legend.show
    );
    assert_eq!(
        Some(true),
        PieChart::from_json(&json(r#", "legend_show": true"#))
            .unwrap()
            .legend
            .show
    );

    // A theme may say so as well, and the options of a chart still decide.
    let mut theme = theme_from("light");
    theme.legend.show = Some(true);
    add_theme("options-test-pie", theme);
    assert_eq!(
        Some(true),
        PieChart::new_with_theme(slices(), "options-test-pie")
            .legend
            .show
    );
    assert_eq!(
        Some(false),
        PieChart::from_json(&json(
            r#", "theme": "options-test-pie", "legend_show": false"#
        ))
        .unwrap()
        .legend
        .show
    );
}

#[test]
fn a_theme_is_written_as_it_is_grouped() {
    let theme = theme_from("light");
    let value = serde_json::to_value(&theme).unwrap();
    assert_eq!(18.0, value["title"]["font"]["size"]);
    assert_eq!("bold", value["title"]["font"]["weight"]);
    assert_eq!(1.0, value["grid"]["stroke_width"]);
    assert!(value["series"]["label"]["font"]["color"].is_string());
    assert!(value["y_axis"]["split_number"].is_u64());
    // And read back the same.
    let read: Theme = serde_json::from_value(value).unwrap();
    assert_eq!(theme.title, read.title);
    assert_eq!(theme.series, read.series);
    assert_eq!(theme.x_axis, read.x_axis);

    // A group may be given in part: what it leaves out is the default.
    let title: charts_rs::TitleConfig =
        serde_json::from_str(r#"{"font": {"size": 20}, "align": "Left"}"#).unwrap();
    assert_eq!(20.0, title.font.size);
    assert_eq!(Align::Left, title.align);
    assert!(title.text.is_empty() && title.font.weight.is_none());
}
