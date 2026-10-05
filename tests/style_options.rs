//! The style options that used to be fixed values: each of them shows in
//! the SVG, and none of them when it is left out.

use charts_rs::{
    BarChart, GanttChart, HorizontalBarChart, LineChart, MarkArea, MarkLine, MarkLineCategory,
    PieChart, PolarBarChart, RadarChart, ScatterChart, Series, SeriesBand, TableChart,
    WaterfallChart,
};

const DATA: &str = r#""x_axis_data": ["a", "b", "c"],
    "series_list": [{"name": "s", "data": [1, 3, 2]}]"#;

fn json(options: &str) -> String {
    format!("{{{DATA}{options}}}")
}

/// The tags of an SVG that have the given text in them.
fn tags<'a>(svg: &'a str, with: &str) -> Vec<&'a str> {
    svg.split('<').filter(|tag| tag.contains(with)).collect()
}

#[test]
fn grid_lines_are_dashed() {
    let dashed = r#", "grid_stroke_dash_array": "4,2""#;
    // A scatter chart draws the lines across and the lines down apart.
    for (name, groups_expected, plain, with) in [
        (
            "bar",
            1,
            BarChart::from_json(&json("")).unwrap().svg().unwrap(),
            BarChart::from_json(&json(dashed)).unwrap().svg().unwrap(),
        ),
        (
            "line",
            1,
            LineChart::from_json(&json("")).unwrap().svg().unwrap(),
            LineChart::from_json(&json(dashed)).unwrap().svg().unwrap(),
        ),
        (
            "horizontal bar",
            1,
            HorizontalBarChart::from_json(&json(""))
                .unwrap()
                .svg()
                .unwrap(),
            HorizontalBarChart::from_json(&json(dashed))
                .unwrap()
                .svg()
                .unwrap(),
        ),
        (
            "scatter",
            2,
            ScatterChart::from_json(&json("")).unwrap().svg().unwrap(),
            ScatterChart::from_json(&json(dashed))
                .unwrap()
                .svg()
                .unwrap(),
        ),
    ] {
        assert!(!plain.contains("stroke-dasharray"), "{name}");
        // The dashes are on the group of the grid.
        let groups = tags(&with, "stroke-dasharray=\"4,2\"");
        assert_eq!(groups_expected, groups.len(), "{name}: {groups:?}");
        assert!(groups.iter().all(|g| g.starts_with("g ")), "{name}");
        assert_eq!(
            plain,
            with.replace(" stroke-dasharray=\"4,2\"", ""),
            "{name}"
        );
    }
    // In Rust, and in compact output.
    let mut chart = BarChart::from_json(&json("")).unwrap();
    chart.grid.stroke_dash_array = Some("1,3".to_string());
    assert!(chart.svg().unwrap().contains("stroke-dasharray=\"1,3\""));
    chart.compact = true;
    assert!(chart.svg().unwrap().contains("stroke-dasharray=\"1,3\""));
    // No dashes are no attribute.
    chart.grid.stroke_dash_array = Some(String::new());
    assert!(!chart.svg().unwrap().contains("stroke-dasharray"));

    // The grid of a gantt chart is lines of its own.
    let gantt = |extra: &str| {
        GanttChart::from_json(&format!(
            r#"{{"x_axis_type": "value", "tasks": [{{"name": "t", "start": 1, "end": 5}}]{extra}}}"#
        ))
        .unwrap()
        .svg()
        .unwrap()
    };
    assert!(!gantt("").contains("stroke-dasharray=\"4,2\""));
    assert!(gantt(dashed).contains("stroke-dasharray=\"4,2\""));

    // So are the web of a radar chart, and the rings and spokes of a polar
    // bar chart.
    let radar = |extra: &str| {
        RadarChart::from_json(&format!(
            r#"{{"indicators": [{{"name": "a", "max": 5}}, {{"name": "b", "max": 5}},
                {{"name": "c", "max": 5}}],
                "series_list": [{{"name": "s", "data": [1, 3, 2]}}]{extra}}}"#
        ))
        .unwrap()
        .svg()
        .unwrap()
    };
    let polar = |extra: &str| {
        PolarBarChart::from_json(&json(extra))
            .unwrap()
            .svg()
            .unwrap()
    };
    for (name, plain, with) in [
        ("radar", radar(""), radar(dashed)),
        ("polar bar", polar(""), polar(dashed)),
    ] {
        assert!(!plain.contains("stroke-dasharray"), "{name}");
        assert!(tags(&with, "stroke-dasharray=\"4,2\"").len() >= 4, "{name}");
        assert_eq!(
            plain,
            with.replace(" stroke-dasharray=\"4,2\"", ""),
            "{name}"
        );
    }
}

#[test]
fn axis_lines_have_a_width() {
    // The y axis has no line until it has a color.
    let axes = |extra: &str, y: &str| {
        format!(
            r##"{{{DATA}{extra}, "x_axis_stroke_color": "#111111",
                "y_axis_configs": [{{"axis_stroke_color": "#222222"{y}}}]}}"##
        )
    };
    let width_of = |svg: &str, color: &str| -> String {
        let group = tags(svg, &format!("stroke=\"{color}\""))
            .into_iter()
            .find(|tag| tag.starts_with("g ") && tag.contains("stroke-width"))
            .unwrap_or_else(|| panic!("no axis of {color}"));
        let start = group.find("stroke-width=\"").unwrap() + 14;
        group[start..start + group[start..].find('"').unwrap()].to_string()
    };
    let plain = BarChart::from_json(&axes("", "")).unwrap().svg().unwrap();
    assert_eq!("1", width_of(&plain, "#111111"));
    assert_eq!("1", width_of(&plain, "#222222"));

    let wide = BarChart::from_json(&axes(
        r#", "x_axis_stroke_width": 2.5"#,
        r#", "axis_stroke_width": 3"#,
    ))
    .unwrap();
    assert_eq!(Some(2.5), wide.x_axis.stroke_width);
    assert_eq!(Some(3.0), wide.y_axis_configs[0].stroke_width);
    let svg = wide.svg().unwrap();
    assert_eq!("2.5", width_of(&svg, "#111111"));
    assert_eq!("3", width_of(&svg, "#222222"));

    // A line chart on a continuous axis, and a horizontal bar chart, whose
    // categories are on the left and whose values are at the bottom.
    let continuous = LineChart::from_json(&axes(
        r#", "x_axis_values": [1, 2, 4], "x_axis_stroke_width": 4"#,
        "",
    ))
    .unwrap()
    .svg()
    .unwrap();
    assert_eq!("4", width_of(&continuous, "#111111"));
    let horizontal = HorizontalBarChart::from_json(&axes(
        r#", "x_axis_stroke_width": 2"#,
        r#", "axis_stroke_width": 5"#,
    ))
    .unwrap()
    .svg()
    .unwrap();
    assert_eq!("2", width_of(&horizontal, "#111111"));
    assert_eq!("5", width_of(&horizontal, "#222222"));
}

#[test]
fn areas_are_as_opaque_as_told() {
    let line = |extra: &str| {
        LineChart::from_json(&json(&format!(r#", "series_fill": true{extra}"#)))
            .unwrap()
            .svg()
            .unwrap()
    };
    assert!(line("").contains("fill-opacity=\"0.39\""));
    let half = line(r#", "series_fill_opacity": 0.5"#);
    assert!(half.contains("fill-opacity=\"0.5\"") && !half.contains("fill-opacity=\"0.39\""));
    // Out of range is the nearest end of it.
    assert!(!line(r#", "series_fill_opacity": 7"#).contains("fill-opacity=\"0.39\""));
    assert_eq!(
        line(r#", "series_fill_opacity": 0"#),
        line(r#", "series_fill_opacity": -3"#)
    );

    let radar = |extra: &str| {
        RadarChart::from_json(&format!(
            r#"{{"indicators": [{{"name": "a", "max": 5}}, {{"name": "b", "max": 5}},
                {{"name": "c", "max": 5}}],
                "series_list": [{{"name": "s", "data": [1, 3, 2]}}]{extra}}}"#
        ))
        .unwrap()
        .svg()
        .unwrap()
    };
    assert!(radar("").contains("fill-opacity=\"0.2\""));
    let radar = radar(r#", "series_fill_opacity": 0.6"#);
    assert!(radar.contains("fill-opacity=\"0.6\"") && !radar.contains("fill-opacity=\"0.2\""));
}

#[test]
fn tooltips_have_a_font_of_their_own() {
    let font = r##", "tooltip_font_size": 21, "tooltip_font_color": "#abcdef",
        "tooltip_font_weight": "bold""##;
    let tips = |svg: &str| -> Vec<String> {
        tags(svg, "class=\"ct-tip\"")
            .into_iter()
            .map(|tag| tag.split('>').next().unwrap().to_string())
            .collect()
    };
    for (name, plain, with) in [(
        "bar",
        BarChart::from_json(&json(r#", "tooltip_show": true"#)),
        BarChart::from_json(&json(&format!(r#", "tooltip_show": true{font}"#))),
    )] {
        let (plain, with) = (plain.unwrap().svg().unwrap(), with.unwrap().svg().unwrap());
        let (plain, with) = (tips(&plain), tips(&with));
        assert!(!plain.is_empty() && plain.len() == with.len(), "{name}");
        for tip in plain.iter() {
            assert!(
                tip.contains("font-size=\"14\"") && !tip.contains("font-weight"),
                "{tip}"
            );
        }
        for tip in with.iter() {
            assert!(
                tip.contains("font-size=\"21\"")
                    && tip.contains("fill=\"#ABCDEF\"")
                    && tip.contains("font-weight=\"bold\""),
                "{name}: {tip}"
            );
        }
    }
    // Every chart with tooltips writes them in it.
    let options = format!(r#", "tooltip_show": true{font}"#);
    let svgs = [
        LineChart::from_json(&json(&options))
            .unwrap()
            .svg()
            .unwrap(),
        HorizontalBarChart::from_json(&json(&options))
            .unwrap()
            .svg()
            .unwrap(),
        ScatterChart::from_json(&json(&options))
            .unwrap()
            .svg()
            .unwrap(),
        PieChart::from_json(&json(&options)).unwrap().svg().unwrap(),
        WaterfallChart::from_json(&format!(
            r#"{{"x_axis_data": ["a", "b"], "data": [3, 4]{options}}}"#
        ))
        .unwrap()
        .svg()
        .unwrap(),
    ];
    for svg in svgs.iter() {
        let tips = tips(svg);
        assert!(!tips.is_empty());
        for tip in tips {
            assert!(
                tip.contains("font-size=\"21\"") && tip.contains("font-weight=\"bold\""),
                "{tip}"
            );
        }
    }
    // A scatter chart leaves the font of its tooltips to the viewer until
    // it is given one.
    let scatter = ScatterChart::from_json(&json(r#", "tooltip_show": true"#))
        .unwrap()
        .svg()
        .unwrap();
    assert!(tips(&scatter).iter().all(|tip| !tip.contains("font-size")));

    // Only the size, in Rust: the color stays that of the labels.
    let mut chart = BarChart::from_json(&json(r#", "tooltip_show": true"#)).unwrap();
    chart.tooltip.font.size = 9.0;
    let svg = chart.svg().unwrap();
    for tip in tips(&svg) {
        assert!(
            tip.contains("font-size=\"9\"") && tip.contains("fill=\"#464646\""),
            "{tip}"
        );
    }
}

#[test]
fn mark_lines_and_areas_have_their_style() {
    let marked = |lines: &str, areas: &str| {
        format!(
            r##"{{"x_axis_data": ["a", "b", "c"], "series_list": [{{"name": "s", "data": [1, 3, 2],
                "mark_lines": [{{"category": "average"{lines}}}],
                "mark_areas": [{{"from": 1, "to": 2{areas}}}]}}]}}"##
        )
    };
    let style = r##", "color": "#ff0000", "stroke_width": 3, "stroke_dash_array": "1,1""##;
    let area = r##", "color": "#00ff00", "opacity": 0.5"##;
    for (name, plain, with) in [
        (
            "bar",
            BarChart::from_json(&marked("", "")).unwrap().svg().unwrap(),
            BarChart::from_json(&marked(style, area))
                .unwrap()
                .svg()
                .unwrap(),
        ),
        (
            "line",
            LineChart::from_json(&marked("", ""))
                .unwrap()
                .svg()
                .unwrap(),
            LineChart::from_json(&marked(style, area))
                .unwrap()
                .svg()
                .unwrap(),
        ),
        (
            "horizontal bar",
            HorizontalBarChart::from_json(&marked("", ""))
                .unwrap()
                .svg()
                .unwrap(),
            HorizontalBarChart::from_json(&marked(style, area))
                .unwrap()
                .svg()
                .unwrap(),
        ),
    ] {
        // As it always was: dashes of 4 and 2, in the color of the series.
        let line = tags(&plain, "stroke-dasharray=\"4,2\"");
        assert_eq!(1, line.len(), "{name}");
        assert!(
            line[0].contains("stroke-width=\"1\"") && line[0].contains("#5470C6"),
            "{name}"
        );
        assert_eq!(1, tags(&plain, "fill-opacity=\"0.16\"").len(), "{name}");

        let line = tags(&with, "stroke-dasharray=\"1,1\"");
        assert_eq!(1, line.len(), "{name}");
        assert!(
            line[0].contains("stroke-width=\"3\"") && line[0].contains("stroke=\"#FF0000\""),
            "{name}: {}",
            line[0]
        );
        // The dot of the line is of its color, and its arrow where it has one.
        assert!(tags(&with, "#FF0000").len() >= 2, "{name}");
        let band = tags(&with, "fill=\"#00FF00\"");
        assert_eq!(1, band.len(), "{name}");
        assert!(
            band[0].contains("fill-opacity=\"0.5\""),
            "{name}: {}",
            band[0]
        );
    }
    // No dashes: a solid line.
    let solid = BarChart::from_json(&marked(r#", "stroke_dash_array": """#, ""))
        .unwrap()
        .svg()
        .unwrap();
    assert!(!solid.contains("stroke-dasharray"));

    // The same in Rust.
    let mut series: Series = ("s", vec![1.0, 3.0, 2.0]).into();
    series.mark_lines = vec![MarkLine {
        category: MarkLineCategory::Average,
        color: Some("#ff0000".into()),
        stroke_width: Some(3.0),
        stroke_dash_array: Some("1,1".to_string()),
    }];
    series.mark_areas = vec![MarkArea {
        from: MarkLineCategory::Value(1.0),
        to: MarkLineCategory::Value(2.0),
        color: Some("#00ff00".into()),
        opacity: Some(0.5),
    }];
    let labels = vec!["a".to_string(), "b".to_string(), "c".to_string()];
    assert_eq!(
        BarChart::from_json(&marked(style, area))
            .unwrap()
            .svg()
            .unwrap(),
        BarChart::new(vec![series], labels).svg().unwrap()
    );
}

#[test]
fn error_bars_have_a_width() {
    let errors = |extra: &str| {
        format!(
            r#"{{"x_axis_data": ["a", "b"], "series_list": [{{"name": "s", "data": [2, 3],
                "error_bar": {{"lower": [1, 2], "upper": [3, 4]{extra}}}}}]}}"#
        )
    };
    let wide = r#", "stroke_width": 4"#;
    // A line down the middle and a cap at both ends, for each of two values.
    for (name, plain, with) in [
        (
            "bar",
            BarChart::from_json(&errors("")).unwrap().svg().unwrap(),
            BarChart::from_json(&errors(wide)).unwrap().svg().unwrap(),
        ),
        (
            "line",
            LineChart::from_json(&errors("")).unwrap().svg().unwrap(),
            LineChart::from_json(&errors(wide)).unwrap().svg().unwrap(),
        ),
    ] {
        assert_eq!(6, tags(&plain, "stroke-width=\"1.5\"").len(), "{name}");
        assert_eq!(0, tags(&with, "stroke-width=\"1.5\"").len(), "{name}");
        assert_eq!(
            plain,
            with.replace("stroke-width=\"4\"", "stroke-width=\"1.5\""),
            "{name}"
        );
    }
    let scatter = |extra: &str| {
        ScatterChart::from_json(&format!(
            r#"{{"series_list": [{{"name": "s", "data": [1, 2, 3, 4],
                "error_bar": {{"lower": [1, 3], "upper": [3, 5]{extra}}}}}]}}"#
        ))
        .unwrap()
        .svg()
        .unwrap()
    };
    assert_eq!(6, tags(&scatter(""), "stroke-width=\"1.5\"").len());
    assert_eq!(6, tags(&scatter(wide), "stroke-width=\"4\"").len());

    // In Rust the width is on the bounds.
    let mut series: Series = ("s", vec![2.0, 3.0]).into();
    series.error_bar = Some(SeriesBand {
        stroke_width: Some(4.0),
        ..SeriesBand::new(vec![1.0, 2.0], vec![3.0, 4.0])
    });
    let chart = BarChart::new(vec![series], vec!["a".to_string(), "b".to_string()]);
    assert_eq!(
        BarChart::from_json(&errors(wide)).unwrap().svg().unwrap(),
        chart.svg().unwrap()
    );
}

/// The radii of the arcs of the slices of a ring.
fn ring_radii(svg: &str, ring: &str) -> Vec<f32> {
    let mut radii: Vec<f32> = tags(svg, &format!("data-ring=\"{ring}\""))
        .into_iter()
        .flat_map(|tag| {
            let start = tag.find(" d=\"").unwrap() + 4;
            tag[start..start + tag[start..].find('"').unwrap()]
                .split(' ')
                .filter_map(|token| token.strip_prefix('A')?.parse().ok())
                .filter(|radius: &f32| *radius > 0.0)
                .collect::<Vec<f32>>()
        })
        .collect();
    radii.sort_by(f32::total_cmp);
    radii.dedup();
    radii
}

#[test]
fn rings_of_a_pie_are_as_far_apart_as_told() {
    let pie = |extra: &str| {
        PieChart::from_json(&format!(
            r#"{{"rose_type": false, "radius": 100, "inner_radius": 20, "border_radius": 0{extra},
                "series_list": [
                    {{"name": "in", "data": [1], "ring": 0}},
                    {{"name": "in too", "data": [2], "ring": 0}},
                    {{"name": "out", "data": [1], "ring": 1}},
                    {{"name": "out too", "data": [2], "ring": 1}}
                ]}}"#
        ))
        .unwrap()
    };
    let gap = |extra: &str| -> f32 {
        let svg = pie(extra).svg().unwrap();
        let (inner, outer) = (ring_radii(&svg, "0"), ring_radii(&svg, "1"));
        assert_eq!((20.0, 100.0), (inner[0], outer[1]), "{extra}");
        outer[0] - inner[1]
    };
    assert_eq!(None, pie("").ring_gap);
    assert_eq!(Some(10.0), pie(r#", "ring_gap": 10"#).ring_gap);
    assert_eq!(6.0, gap(""));
    assert_eq!(0.0, gap(r#", "ring_gap": 0"#));
    assert_eq!(10.0, gap(r#", "ring_gap": 10"#));
    // Less than nothing is nothing, and never more than a third of what
    // each ring would have without it.
    assert_eq!(0.0, gap(r#", "ring_gap": -5"#));
    assert!(gap(r#", "ring_gap": 500"#) < 14.0);
    // A pie of one ring has no use for it.
    let one = |extra: &str| {
        PieChart::from_json(&format!(
            r#"{{"series_list": [{{"name": "a", "data": [1]}}, {{"name": "b", "data": [2]}}]{extra}}}"#
        ))
        .unwrap()
        .svg()
        .unwrap()
    };
    assert_eq!(one(""), one(r#", "ring_gap": 30"#));
}

#[test]
fn table_borders_have_a_width() {
    let table = |extra: &str| {
        TableChart::from_json(&format!(
            r##"{{"data": [["Name", "Price"], ["A", "1"], ["B", "2"]],
                "border_color": "#123456", "outlined": true{extra}}}"##
        ))
        .unwrap()
    };
    assert_eq!(None, table("").border_width);
    assert_eq!(Some(3.0), table(r#", "border_width": 3"#).border_width);
    let plain = table("").svg().unwrap();
    let wide = table(r#", "border_width": 3"#).svg().unwrap();
    let lines = |svg: &str| -> Vec<String> {
        tags(svg, "stroke=\"#123456\"")
            .into_iter()
            .map(str::to_string)
            .collect()
    };
    let (plain, wide) = (lines(&plain), lines(&wide));
    // The lines between the rows, and the border around them.
    assert!(plain.len() >= 3 && plain.len() == wide.len());
    assert!(
        plain
            .iter()
            .all(|line| !line.contains("stroke-width=\"3\""))
    );
    assert!(
        wide.iter().all(|line| line.contains("stroke-width=\"3\"")),
        "{wide:?}"
    );
    assert!(wide.iter().any(|line| line.starts_with("rect ")));
}

#[test]
fn waterfall_connectors_are_dashed_as_told() {
    let waterfall = |extra: &str| {
        WaterfallChart::from_json(&format!(
            r#"{{"x_axis_data": ["a", "b", "c"], "data": [3, 4, -2]{extra}}}"#
        ))
        .unwrap()
        .svg()
        .unwrap()
    };
    assert_eq!(2, tags(&waterfall(""), "stroke-dasharray=\"4,4\"").len());
    let dotted = waterfall(r#", "connector_line_dash_array": "1,2""#);
    assert_eq!(2, tags(&dotted, "stroke-dasharray=\"1,2\"").len());
    assert_eq!(
        waterfall(""),
        dotted.replace("stroke-dasharray=\"1,2\"", "stroke-dasharray=\"4,4\"")
    );
    assert!(!waterfall(r#", "connector_line_dash_array": """#).contains("stroke-dasharray"));
    assert!(
        !waterfall(r#", "connector_line_show": false, "connector_line_dash_array": "1,2""#)
            .contains("stroke-dasharray")
    );
}

#[test]
fn lines_drawn_as_a_grid_are_dashed_too() {
    use charts_rs::{HeatmapChart, ParallelChart};
    let dashed = r#", "grid_stroke_dash_array": "4,2""#;
    // The rows of a punch card, and the axes of a parallel chart.
    let punch_card = |extra: &str| {
        HeatmapChart::from_json(&format!(
            r#"{{"x_axis_data": ["a", "b"], "y_axis_data": ["p", "q"],
                "series": {{"symbol": "circle", "data": [[0, 1], [1, 5], [2, 3], [3, 2]]}}{extra}}}"#
        ))
        .unwrap()
        .svg()
        .unwrap()
    };
    let parallel = |extra: &str| {
        ParallelChart::from_json(&json(extra))
            .unwrap()
            .svg()
            .unwrap()
    };
    for (name, lines, plain, with) in [
        ("punch card", 2, punch_card(""), punch_card(dashed)),
        ("parallel", 3, parallel(""), parallel(dashed)),
    ] {
        assert!(!plain.contains("stroke-dasharray"), "{name}");
        assert_eq!(
            lines,
            tags(&with, "stroke-dasharray=\"4,2\"").len(),
            "{name}"
        );
        assert_eq!(
            plain,
            with.replace(" stroke-dasharray=\"4,2\"", ""),
            "{name}"
        );
    }
}

#[test]
fn the_zero_line_of_a_waterfall_is_as_wide_as_its_axis() {
    let waterfall = |extra: &str| {
        WaterfallChart::from_json(&format!(
            r##"{{"x_axis_data": ["a", "b", "c"], "data": [3, -5, 1],
                "x_axis_stroke_color": "#123456", "connector_line_show": false{extra}}}"##
        ))
        .unwrap()
        .svg()
        .unwrap()
    };
    let widths = |svg: &str| -> Vec<String> {
        tags(svg, "stroke=\"#123456\"")
            .into_iter()
            .map(|tag| {
                let start = tag.find("stroke-width=\"").unwrap() + 14;
                tag[start..start + tag[start..].find('"').unwrap()].to_string()
            })
            .collect()
    };
    // The axis, and the line of 0 across the bars.
    assert_eq!(vec!["1", "1"], widths(&waterfall("")));
    assert_eq!(
        vec!["4", "4"],
        widths(&waterfall(r#", "x_axis_stroke_width": 4"#))
    );
}

#[test]
fn a_mark_area_keeps_the_alpha_of_its_color() {
    let area = |style: &str| {
        BarChart::from_json(&format!(
            r##"{{"x_axis_data": ["a", "b"], "series_list": [{{"name": "s", "data": [1, 3],
                "mark_areas": [{{"from": 1, "to": 2{style}}}]}}]}}"##
        ))
        .unwrap()
        .svg()
        .unwrap()
    };
    let opacity = |svg: &str| -> String {
        let band = tags(svg, "fill=\"#FF0000\"")[0];
        let start = band.find("fill-opacity=\"").unwrap() + 14;
        band[start..start + band[start..].find('"').unwrap()].to_string()
    };
    // A plain color is as faint as the band always was.
    assert_eq!("0.16", opacity(&area(r##", "color": "#ff0000""##)));
    // One with an alpha keeps it, unless an opacity is given.
    assert_eq!("0.5", opacity(&area(r##", "color": "#ff000080""##)));
    assert_eq!(
        "0.25",
        opacity(&area(r##", "color": "#ff000080", "opacity": 0.25"##))
    );
}
