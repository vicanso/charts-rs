//! Texts written over a chart wherever they are asked for: a copyright line,
//! a watermark.
mod common;

use charts_rs::{
    Align, BarChart, BoxPlotChart, CalendarChart, CandlestickChart, ChildChart, ChordChart,
    FontConfig, FunnelChart, GanttChart, GaugeChart, GraphChart, HeatmapChart, HistogramChart,
    HorizontalBarChart, LineChart, MapChart, MultiChart, ParallelChart, PieChart, PolarBarChart,
    RadarChart, SankeyChart, ScatterChart, Series, SunburstChart, TableChart, TextConfig,
    ThemeRiverChart, TreeChart, TreemapChart, WaterfallChart, add_theme, get_theme,
};

/// The `<text>` tag the given text is written in, without its content.
fn tag_of<'a>(svg: &'a str, text: &str) -> &'a str {
    let ending = format!(">\n{text}\n</text>");
    let end = svg
        .find(&ending)
        .unwrap_or_else(|| panic!("{text:?} is not in {svg}"));
    let start = svg[..end].rfind("<text").unwrap();
    &svg[start..end]
}

fn attr<'a>(tag: &'a str, name: &str) -> Option<&'a str> {
    let key = format!(" {name}=\"");
    let start = tag.find(&key)? + key.len();
    Some(&tag[start..start + tag[start..].find('"')?])
}

fn number(tag: &str, name: &str) -> f32 {
    attr(tag, name).unwrap().parse().unwrap()
}

const DATA: &str = r#""x_axis_data": ["a", "b", "c"],
    "series_list": [{"name": "s", "data": [1, 3, 2]}]"#;

fn bar(texts: &str) -> String {
    BarChart::from_json(&format!(r#"{{{DATA}, "texts": {texts}}}"#))
        .unwrap()
        .svg()
        .unwrap()
}

#[test]
fn texts_snapshot() {
    let chart = BarChart::from_json(include_str!("../asset/bar_chart/texts.json")).unwrap();
    assert_eq!(3, chart.texts.len());
    common::assert_snapshot!("bar_chart/texts_json.svg", chart.svg().unwrap());
}

#[test]
fn a_text_is_placed_by_its_distances_from_the_edges() {
    // On the 600 by 400 pixels of a chart, 12 pixels high.
    for (place, x, anchor, y) in [
        (r#""left": 10, "top": 8"#, 10.0, "start", 14.0),
        (r#""top": 8"#, 300.0, "middle", 14.0),
        (r#""right": 10, "top": 8"#, 590.0, "end", 14.0),
        (r#""left": 10"#, 10.0, "start", 200.0),
        ("", 300.0, "middle", 200.0),
        (r#""right": 10"#, 590.0, "end", 200.0),
        (r#""left": 10, "bottom": 6"#, 10.0, "start", 388.0),
        (r#""bottom": 6"#, 300.0, "middle", 388.0),
        (r#""right": 10, "bottom": 6"#, 590.0, "end", 388.0),
        // The left and the top win over the right and the bottom.
        (
            r#""left": 10, "right": 20, "top": 8, "bottom": 30"#,
            10.0,
            "start",
            14.0,
        ),
        // From outside the chart, if it is asked for.
        (r#""left": -5, "top": 0"#, -5.0, "start", 6.0),
    ] {
        let comma = if place.is_empty() { "" } else { ", " };
        let svg = bar(&format!(r#"[{{"text": "placed"{comma}{place}}}]"#));
        let tag = tag_of(&svg, "placed");
        assert_eq!(x, number(tag, "x"), "{place}");
        assert_eq!(y, number(tag, "y"), "{place}");
        assert_eq!(Some(anchor), attr(tag, "text-anchor"), "{place}");
        assert_eq!(Some("central"), attr(tag, "dominant-baseline"), "{place}");
        assert_eq!(None, attr(tag, "transform"), "{place}");
    }
    // A larger text is placed by its own height.
    let svg = bar(
        r#"[{"text": "large", "font_size": 40, "top": 8}, {"text": "low", "font_size": 40, "bottom": 6}]"#,
    );
    assert_eq!(28.0, number(tag_of(&svg, "large"), "y"));
    assert_eq!(374.0, number(tag_of(&svg, "low"), "y"));
}

#[test]
fn a_text_has_its_font_and_its_turn() {
    // Left to itself: 12 pixels, in the color of the sub-title, upright.
    let svg = bar(r#"[{"text": "plain", "right": 10, "bottom": 6}]"#);
    let tag = tag_of(&svg, "plain");
    assert_eq!(12.0, number(tag, "font-size"));
    assert_eq!(Some("#464646"), attr(tag, "fill"));
    assert_eq!(Some("Roboto"), attr(tag, "font-family"));
    assert_eq!(None, attr(tag, "font-weight"));
    assert_eq!(None, attr(tag, "fill-opacity"));

    // A watermark: large, faint, bold and turned about its middle.
    let svg = bar(
        r##"[{"text": "DRAFT", "font_size": 56, "font_color": "#00000020", "font_weight": "bold", "rotate": -25}]"##,
    );
    let tag = tag_of(&svg, "DRAFT");
    assert_eq!(56.0, number(tag, "font-size"));
    assert_eq!(Some("#000000"), attr(tag, "fill"));
    assert_eq!(Some("0.13"), attr(tag, "fill-opacity"));
    assert_eq!(Some("bold"), attr(tag, "font-weight"));
    assert_eq!(Some("rotate(-25 300 200)"), attr(tag, "transform"));
    // In a corner it turns about that end of it.
    let svg = bar(r#"[{"text": "turned", "right": 10, "bottom": 6, "rotate": 90}]"#);
    assert_eq!(
        Some("rotate(90 590 388)"),
        attr(tag_of(&svg, "turned"), "transform")
    );

    // On a dark theme the text is light, and in the font of the chart.
    let dark = BarChart::from_json(&format!(
        r#"{{{DATA}, "theme": "dark", "font_family": "Arial", "texts": [{{"text": "light"}}]}}"#
    ))
    .unwrap()
    .svg()
    .unwrap();
    let tag = tag_of(&dark, "light");
    assert_eq!(Some("#EEEEEE"), attr(tag, "fill"));
    assert_eq!(Some("Arial"), attr(tag, "font-family"));
}

#[test]
fn texts_are_written_last_and_as_text() {
    let plain = BarChart::from_json(&format!("{{{DATA}}}"))
        .unwrap()
        .svg()
        .unwrap();
    // No texts: the chart as it always was.
    assert_eq!(plain, bar("[]"));
    assert_eq!(plain, bar(r#"[{"text": ""}, {"left": 10}]"#));

    // Over everything else, in the order they are given.
    let svg = bar(r#"[{"text": "first", "left": 10}, {"text": "second", "right": 10}]"#);
    let (first, second) = (
        svg.find(">\nfirst\n").unwrap(),
        svg.find(">\nsecond\n").unwrap(),
    );
    assert!(first < second);
    let before = &svg[..svg[..first].rfind("<text").unwrap()];
    assert_eq!(
        plain.trim_end_matches("</svg>").trim_end(),
        before.trim_end()
    );
    assert!(svg.trim_end().ends_with("</text>\n</svg>"));

    // What is written is text, not markup.
    let svg = bar(r#"[{"text": "<b>a & b</b> \"q\""}]"#);
    assert!(svg.contains("&lt;b&gt;a &amp; b&lt;/b&gt;") && !svg.contains("<b>"));

    // Compact output has them too.
    let compact = BarChart::from_json(&format!(
        r#"{{{DATA}, "compact": true, "texts": [{{"text": "kept", "right": 10, "bottom": 6}}]}}"#
    ))
    .unwrap()
    .svg()
    .unwrap();
    assert!(compact.contains(">kept</text>"), "{compact}");

    // A wrong key or value is an error, as everywhere.
    for (json, key) in [
        (r#"{"texts": [{"txt": "a"}]}"#, "texts[0].txt"),
        (
            r#"{"texts": [{"text": "a", "left": "10"}]}"#,
            "texts[0].left",
        ),
        (r#"{"texts": {"text": "a"}}"#, "texts"),
    ] {
        let message = match BarChart::from_json(json) {
            Ok(_) => panic!("{json} is accepted"),
            Err(e) => e.to_string(),
        };
        assert!(message.contains(key), "{message}");
    }
}

#[test]
fn every_chart_writes_its_texts() {
    let texts = r#""texts": [{"text": "© ACME", "right": 10, "bottom": 6}]"#;
    let with = |json: &str| -> String {
        if json.is_empty() {
            format!("{{{texts}}}")
        } else {
            format!("{{{json}, {texts}}}")
        }
    };
    // With data and without: a chart of nothing still says whose it is.
    let series = r#""x_axis_data": ["a", "b"], "series_list": [{"name": "s", "data": [1, 2]}]"#;
    let mut svgs: Vec<(&str, String)> = vec![];
    // Some charts are none without their data: what counts is that every
    // chart that is drawn has the text.
    macro_rules! chart {
        ($name:literal, $chart:ty, $($json:expr),+) => {
            $(match <$chart>::from_json(&with($json)).unwrap().svg() {
                Ok(svg) => svgs.push(($name, svg)),
                Err(e) => assert!($json.is_empty(), "{}: {e}", $name),
            })+
        };
    }
    chart!("bar", BarChart, "", series);
    chart!("horizontal bar", HorizontalBarChart, "", series);
    chart!("line", LineChart, "", series);
    chart!("pie", PieChart, "", series);
    chart!(
        "radar",
        RadarChart,
        "",
        r#""indicators": [{"name": "a", "max": 5}, {"name": "b", "max": 5}, {"name": "c", "max": 5}], "series_list": [{"name": "s", "data": [1, 2, 3]}]"#
    );
    chart!(
        "scatter",
        ScatterChart,
        "",
        r#""series_list": [{"name": "s", "data": [1, 2, 3, 4]}]"#
    );
    chart!(
        "candlestick",
        CandlestickChart,
        "",
        r#""x_axis_data": ["a"], "series_list": [{"name": "k", "data": [1, 3, 0, 4]}]"#
    );
    chart!(
        "heatmap",
        HeatmapChart,
        "",
        r#""x_axis_data": ["a", "b"], "y_axis_data": ["p"], "series": {"data": [[0, 1], [1, 5]]}"#
    );
    chart!("funnel", FunnelChart, "", series);
    chart!(
        "waterfall",
        WaterfallChart,
        "",
        r#""x_axis_data": ["a", "b"], "data": [3, -1]"#
    );
    chart!(
        "calendar",
        CalendarChart,
        "",
        r#""data": [["2024-02-05", 3]]"#
    );
    chart!(
        "gauge",
        GaugeChart,
        "",
        r#""series_list": [{"name": "s", "data": [40]}]"#
    );
    chart!(
        "treemap",
        TreemapChart,
        "",
        r#""series_data": [{"name": "n", "value": 1}]"#
    );
    chart!(
        "box plot",
        BoxPlotChart,
        "",
        r#""x_axis_data": ["a"], "box_series": [{"name": "b", "data": [[1, 2, 3, 4, 5]]}]"#
    );
    chart!(
        "sunburst",
        SunburstChart,
        "",
        r#""series_data": [{"name": "n", "value": 1}]"#
    );
    chart!(
        "sankey",
        SankeyChart,
        "",
        r#""links": [{"source": "a", "target": "b", "value": 1}]"#
    );
    chart!(
        "tree",
        TreeChart,
        "",
        r#""series_data": [{"name": "n", "children": [{"name": "m"}]}]"#
    );
    chart!(
        "graph",
        GraphChart,
        "",
        r#""nodes": [{"name": "a"}, {"name": "b"}], "links": [{"source": "a", "target": "b"}]"#
    );
    chart!("parallel", ParallelChart, "", series);
    chart!("theme river", ThemeRiverChart, "", series);
    chart!(
        "histogram",
        HistogramChart,
        "",
        r#""series_list": [{"name": "s", "data": [1, 2, 2, 3]}]"#
    );
    chart!("polar bar", PolarBarChart, "", series);
    chart!(
        "chord",
        ChordChart,
        "",
        r#""links": [{"source": "a", "target": "b", "value": 1}]"#
    );
    chart!(
        "gantt",
        GanttChart,
        "",
        r#""x_axis_type": "value", "tasks": [{"name": "t", "start": 1, "end": 5}]"#
    );
    chart!(
        "map",
        MapChart,
        "",
        r#""data": [["a", 3]], "geo_json": {"type": "Feature", "properties": {"name": "a"}, "geometry": {"type": "Polygon", "coordinates": [[[0, 0], [1, 0], [1, 1]]]}}"#
    );
    chart!("table", TableChart, r#""data": [["Name"], ["A"]]"#);
    // All 26 of them, most of them empty as well.
    let mut names: Vec<&str> = svgs.iter().map(|(name, _)| *name).collect();
    names.dedup();
    assert_eq!(26, names.len(), "{names:?}");
    assert!(svgs.len() > 40, "{}", svgs.len());

    for (name, svg) in svgs.iter() {
        let tag = tag_of(svg, "© ACME");
        assert_eq!(Some("end"), attr(tag, "text-anchor"), "{name}");
        // In the corner of the picture, however large that is.
        let size = |key: &str| -> f32 {
            let head = &svg[..svg.find('>').unwrap()];
            attr(head, key).unwrap().parse().unwrap()
        };
        assert_eq!(size("width") - 10.0, number(tag, "x"), "{name}");
        assert_eq!(size("height") - 12.0, number(tag, "y"), "{name}");
        // The last thing in it.
        assert!(
            svg.trim_end().ends_with("© ACME\n</text>\n</svg>"),
            "{name}"
        );
    }
    // A table is as high as its rows: the text is at the bottom of those.
    let table = &svgs.last().unwrap().1;
    let height: f32 = attr(&table[..table.find('>').unwrap()], "height")
        .unwrap()
        .parse()
        .unwrap();
    assert!(height < 200.0, "{height}");
}

#[test]
fn a_multi_chart_has_texts_over_all_of_it() {
    let multi = MultiChart::from_json(&format!(
        r##"{{"theme": "dark", "margin": 0, "gap": 0,
            "texts": [{{"text": "over all", "right": 10, "bottom": 6}},
                      {{"text": "MARK", "font_size": 40, "font_color": "#ffffff20", "rotate": -30}}],
            "child_charts": [
                {{{DATA}, "texts": [{{"text": "over one", "left": 10, "top": 8}}]}},
                {{"type": "pie", "series_list": [{{"name": "a", "data": [1]}}, {{"name": "b", "data": [2]}}]}}
            ]}}"##
    ))
    .unwrap();
    assert_eq!(2, multi.texts.len());
    let svg = multi.svg().unwrap();
    // Two charts of 600 by 400, one below the other.
    let all = tag_of(&svg, "over all");
    assert_eq!((590.0, 788.0), (number(all, "x"), number(all, "y")));
    // In the color of the sub-title of the theme, as in a chart.
    assert_eq!(Some("#EEEEEE"), attr(all, "fill"));
    assert_eq!(
        Some("rotate(-30 300 400)"),
        attr(tag_of(&svg, "MARK"), "transform")
    );
    // The text of a child is in the child, the others after every child.
    let one = tag_of(&svg, "over one");
    assert_eq!((10.0, 14.0), (number(one, "x"), number(one, "y")));
    assert!(svg.find("over one").unwrap() < svg.rfind("<svg").unwrap());
    assert!(svg.rfind("</svg>\n<text").is_some());
    assert!(svg.trim_end().ends_with("MARK\n</text>\n</svg>"));

    // In Rust.
    let mut multi = MultiChart::new();
    multi.add(ChildChart::Bar(
        BarChart::new(
            vec![("s", vec![1.0, 2.0]).into()],
            vec!["a".to_string(), "b".to_string()],
        ),
        None,
    ));
    multi.texts = vec![TextConfig {
        text: "mine".to_string(),
        left: Some(4.0),
        top: Some(2.0),
        ..Default::default()
    }];
    let svg = multi.svg().unwrap();
    let tag = tag_of(&svg, "mine");
    assert_eq!((4.0, 8.0), (number(tag, "x"), number(tag, "y")));
    assert_eq!(Some("Roboto"), attr(tag, "font-family"));
}

#[test]
fn texts_in_rust_and_in_a_theme() {
    // The same in Rust as from JSON.
    let series = || -> Vec<Series> { vec![("s", vec![1.0, 3.0, 2.0]).into()] };
    let labels = || -> Vec<String> { vec!["a".into(), "b".into(), "c".into()] };
    let mut chart = BarChart::new(series(), labels());
    chart.texts = vec![
        TextConfig {
            text: "© ACME".to_string(),
            right: Some(10.0),
            bottom: Some(6.0),
            ..Default::default()
        },
        TextConfig {
            text: "DRAFT".to_string(),
            font: FontConfig {
                size: 56.0,
                color: "#00000020".into(),
                weight: Some("bold".to_string()),
            },
            rotate: -25.0,
            ..Default::default()
        },
    ];
    let json = r##"[{"text": "© ACME", "right": 10, "bottom": 6},
        {"text": "DRAFT", "font_size": 56, "font_color": "#00000020", "font_weight": "bold", "rotate": -25}]"##;
    let from_json = BarChart::from_json(&format!(r#"{{{DATA}, "texts": {json}}}"#)).unwrap();
    assert_eq!(from_json.texts, chart.texts);
    assert_eq!(from_json.svg().unwrap(), chart.svg().unwrap());

    // Numbers that are none leave a text out rather than break the SVG.
    for text in [
        TextConfig {
            text: "nowhere".to_string(),
            left: Some(f32::NAN),
            ..Default::default()
        },
        TextConfig {
            text: "nowhere".to_string(),
            bottom: Some(f32::INFINITY),
            ..Default::default()
        },
    ] {
        let mut chart = BarChart::new(series(), labels());
        chart.texts = vec![text];
        let svg = chart.svg().unwrap();
        assert!(!svg.contains("nowhere") && !svg.contains("NaN") && !svg.contains("inf"));
    }
    let mut chart = BarChart::new(series(), labels());
    chart.texts = vec![TextConfig {
        text: "upright".to_string(),
        rotate: f32::NAN,
        font: FontConfig {
            size: -3.0,
            ..Default::default()
        },
        ..Default::default()
    }];
    let svg = chart.svg().unwrap();
    let tag = tag_of(&svg, "upright");
    assert_eq!(
        (None, 12.0),
        (attr(tag, "transform"), number(tag, "font-size"))
    );

    // The texts of a theme are on every chart of it.
    let mut theme = (*get_theme("light")).clone();
    theme.title.align = Align::Left;
    theme.texts = vec![TextConfig {
        text: "© Theme".to_string(),
        right: Some(10.0),
        bottom: Some(6.0),
        ..Default::default()
    }];
    add_theme("texts-test", theme);
    let themed = [
        BarChart::new_with_theme(series(), labels(), "texts-test")
            .svg()
            .unwrap(),
        PieChart::new_with_theme(series(), "texts-test")
            .svg()
            .unwrap(),
        TableChart::new_with_theme(
            vec![vec!["Name".to_string()], vec!["A".to_string()]],
            "texts-test",
        )
        .svg()
        .unwrap(),
        LineChart::from_json(&format!(r#"{{{DATA}, "theme": "texts-test"}}"#))
            .unwrap()
            .svg()
            .unwrap(),
    ];
    for svg in themed.iter() {
        assert_eq!(Some("end"), attr(tag_of(svg, "© Theme"), "text-anchor"));
    }
    // Those of the options stand in for them; none at all, too.
    let own = LineChart::from_json(&format!(
        r#"{{{DATA}, "theme": "texts-test", "texts": [{{"text": "own"}}]}}"#
    ))
    .unwrap()
    .svg()
    .unwrap();
    assert!(own.contains(">\nown\n") && !own.contains("© Theme"));
    let none = LineChart::from_json(&format!(
        r#"{{{DATA}, "theme": "texts-test", "texts": []}}"#
    ))
    .unwrap()
    .svg()
    .unwrap();
    assert!(!none.contains("© Theme"));
}
