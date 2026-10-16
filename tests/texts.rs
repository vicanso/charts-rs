//! Texts written over a chart wherever they are asked for: a copyright line,
//! a watermark.
mod common;

use charts_rs::{
    Align, BarChart, BoxPlotChart, BulletChart, CalendarChart, CandlestickChart, ChildChart,
    ChordChart, FontConfig, FunnelChart, GanttChart, GaugeChart, GraphChart, HeatmapChart,
    HistogramChart, HorizontalBarChart, LineChart, LinesChart, MapChart, MultiChart, ParallelChart,
    PieChart, PolarBarChart, RadarChart, SankeyChart, ScatterChart, Series, SunburstChart,
    TableChart, TextConfig, ThemeRiverChart, TreeChart, TreemapChart, WaterfallChart, add_theme,
    get_theme, measure_text_width_family,
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

const SERIES: &str = r#""x_axis_data": ["a", "b"], "series_list": [{"name": "s", "data": [1, 2]}]"#;

/// Every chart there is but the one of charts, each with `$chart!`: its
/// name, its type and the options it is drawn from. With data and without:
/// a chart of nothing still has what is around it.
macro_rules! every_chart {
    ($chart:ident) => {
        $chart!("bar", BarChart, "", SERIES);
        $chart!("horizontal bar", HorizontalBarChart, "", SERIES);
        $chart!("line", LineChart, "", SERIES);
        $chart!("pie", PieChart, "", SERIES);
        $chart!(
            "radar",
            RadarChart,
            "",
            r#""indicators": [{"name": "a", "max": 5}, {"name": "b", "max": 5}, {"name": "c", "max": 5}], "series_list": [{"name": "s", "data": [1, 2, 3]}]"#
        );
        $chart!(
            "scatter",
            ScatterChart,
            "",
            r#""series_list": [{"name": "s", "data": [1, 2, 3, 4]}]"#
        );
        $chart!(
            "candlestick",
            CandlestickChart,
            "",
            r#""x_axis_data": ["a"], "series_list": [{"name": "k", "data": [1, 3, 0, 4]}]"#
        );
        $chart!(
            "heatmap",
            HeatmapChart,
            "",
            r#""x_axis_data": ["a", "b"], "y_axis_data": ["p"], "series": {"data": [[0, 1], [1, 5]]}"#
        );
        $chart!("funnel", FunnelChart, "", SERIES);
        $chart!(
            "waterfall",
            WaterfallChart,
            "",
            r#""x_axis_data": ["a", "b"], "data": [3, -1]"#
        );
        $chart!(
            "calendar",
            CalendarChart,
            "",
            r#""data": [["2024-02-05", 3]]"#
        );
        $chart!(
            "gauge",
            GaugeChart,
            "",
            r#""series_list": [{"name": "s", "data": [40]}]"#
        );
        $chart!(
            "treemap",
            TreemapChart,
            "",
            r#""series_data": [{"name": "n", "value": 1}]"#
        );
        $chart!(
            "box plot",
            BoxPlotChart,
            "",
            r#""x_axis_data": ["a"], "box_series": [{"name": "b", "data": [[1, 2, 3, 4, 5]]}]"#
        );
        $chart!(
            "sunburst",
            SunburstChart,
            "",
            r#""series_data": [{"name": "n", "value": 1}]"#
        );
        $chart!(
            "sankey",
            SankeyChart,
            "",
            r#""links": [{"source": "a", "target": "b", "value": 1}]"#
        );
        $chart!(
            "tree",
            TreeChart,
            "",
            r#""series_data": [{"name": "n", "children": [{"name": "m"}]}]"#
        );
        $chart!(
            "graph",
            GraphChart,
            "",
            r#""nodes": [{"name": "a"}, {"name": "b"}], "links": [{"source": "a", "target": "b"}]"#
        );
        $chart!("parallel", ParallelChart, "", SERIES);
        $chart!("theme river", ThemeRiverChart, "", SERIES);
        $chart!(
            "histogram",
            HistogramChart,
            "",
            r#""series_list": [{"name": "s", "data": [1, 2, 2, 3]}]"#
        );
        $chart!("polar bar", PolarBarChart, "", SERIES);
        $chart!(
            "chord",
            ChordChart,
            "",
            r#""links": [{"source": "a", "target": "b", "value": 1}]"#
        );
        $chart!(
            "gantt",
            GanttChart,
            "",
            r#""x_axis_type": "value", "tasks": [{"name": "t", "start": 1, "end": 5}]"#
        );
        $chart!(
            "bullet",
            BulletChart,
            "",
            r#""bullets": [{"name": "b", "value": 3, "target": 4, "ranges": [2, 5]}]"#
        );
        $chart!(
            "lines",
            LinesChart,
            "",
            r#""lines": [{"from": [0, 0], "to": [1, 1], "value": 2}]"#
        );
        $chart!(
            "map",
            MapChart,
            "",
            r#""data": [["a", 3]], "geo_json": {"type": "Feature", "properties": {"name": "a"}, "geometry": {"type": "Polygon", "coordinates": [[[0, 0], [1, 0], [1, 1]]]}}"#
        );
        $chart!("table", TableChart, r#""data": [["Name"], ["A"]]"#);
    };
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
    every_chart!(chart);
    // All 28 of them, most of them empty as well.
    let mut names: Vec<&str> = svgs.iter().map(|(name, _)| *name).collect();
    names.dedup();
    assert_eq!(28, names.len(), "{names:?}");
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

/// The chart with texts that reserve nothing, and this much more margin
/// instead: left, top, right and bottom.
macro_rules! margined {
    ($chart:expr, $room:expr) => {{
        let mut chart = $chart.clone();
        for text in chart.texts.iter_mut() {
            text.reserve = false;
        }
        let (left, top, right, bottom): (f32, f32, f32, f32) = $room;
        chart.margin.left += left;
        chart.margin.top += top;
        chart.margin.right += right;
        chart.margin.bottom += bottom;
        chart.svg()
    }};
}

/// How wide "note" is, 12 pixels high.
fn note_width() -> f32 {
    measure_text_width_family("Roboto", 12.0, "note")
        .unwrap()
        .width()
}

#[test]
fn a_text_that_reserves_takes_room_at_its_edge() {
    let width = note_width();
    assert!(width > 20.0 && width < 30.0, "{width}");
    // What a text takes is what a margin of that much more leaves: as far
    // in as the text reaches, on the side it is placed from.
    for (texts, room) in [
        // Across the bottom and across the top, 12 pixels high or 10.
        (
            r#"{"text": "note", "bottom": 6, "reserve": true}"#.to_string(),
            (0.0, 0.0, 0.0, 18.0),
        ),
        (
            r#"{"text": "note", "top": 4, "font_size": 10, "reserve": true}"#.to_string(),
            (0.0, 14.0, 0.0, 0.0),
        ),
        // Down a side for one placed from there only: as wide as it is.
        (
            r#"{"text": "note", "left": 8, "reserve": true}"#.to_string(),
            (8.0 + width, 0.0, 0.0, 0.0),
        ),
        (
            r#"{"text": "note", "right": 8, "reserve": true}"#.to_string(),
            (0.0, 0.0, 8.0 + width, 0.0),
        ),
        // In a corner it is the top or the bottom that it takes.
        (
            r#"{"text": "note", "right": 10, "bottom": 6, "reserve": true}"#.to_string(),
            (0.0, 0.0, 0.0, 18.0),
        ),
        (
            r#"{"text": "note", "left": 10, "top": 6, "reserve": true}"#.to_string(),
            (0.0, 18.0, 0.0, 0.0),
        ),
        // The top and the left win, as they do in placing it.
        (
            r#"{"text": "note", "top": 6, "bottom": 30, "reserve": true}"#.to_string(),
            (0.0, 18.0, 0.0, 0.0),
        ),
        (
            r#"{"text": "note", "left": 8, "right": 30, "reserve": true}"#.to_string(),
            (8.0 + width, 0.0, 0.0, 0.0),
        ),
        // Two at an edge are placed one by one: the room is that of the
        // one further in, not that of both.
        (
            r#"{"text": "note", "left": 10, "bottom": 6, "reserve": true},
               {"text": "more", "right": 10, "bottom": 6, "reserve": true},
               {"text": "above", "bottom": 22, "reserve": true}"#
                .to_string(),
            (0.0, 0.0, 0.0, 34.0),
        ),
        // One at each edge.
        (
            r#"{"text": "note", "left": 8, "reserve": true},
               {"text": "note", "top": 4, "font_size": 10, "reserve": true},
               {"text": "note", "right": 8, "reserve": true},
               {"text": "note", "bottom": 6, "reserve": true}"#
                .to_string(),
            (8.0 + width, 14.0, 8.0 + width, 18.0),
        ),
        // Only those that ask for it.
        (
            r#"{"text": "note", "bottom": 6, "reserve": true},
               {"text": "over", "bottom": 60},
               {"text": "over", "top": 80, "reserve": false},
               {"text": "MARK", "font_size": 60, "rotate": -30}"#
                .to_string(),
            (0.0, 0.0, 0.0, 18.0),
        ),
        // In the middle there is no edge to take room at.
        (
            r#"{"text": "MARK", "font_size": 60, "rotate": -30, "reserve": true}"#.to_string(),
            (0.0, 0.0, 0.0, 0.0),
        ),
        // Turned to read upwards or downwards, it is its height that is
        // across: half of it to each side of where it is placed.
        (
            r#"{"text": "note", "left": 14, "rotate": -90, "reserve": true}"#.to_string(),
            (20.0, 0.0, 0.0, 0.0),
        ),
        (
            r#"{"text": "note", "right": 14, "rotate": 90, "reserve": true}"#.to_string(),
            (0.0, 0.0, 20.0, 0.0),
        ),
        // Outside of the chart there is nothing to make room for.
        (
            r#"{"text": "note", "bottom": -40, "reserve": true}"#.to_string(),
            (0.0, 0.0, 0.0, 0.0),
        ),
    ] {
        let chart = BarChart::from_json(&format!(r#"{{{DATA}, "texts": [{texts}]}}"#)).unwrap();
        assert!(chart.texts.iter().any(|text| text.reserve), "{texts}");
        let svg = chart.svg().unwrap();
        assert_eq!(margined!(chart, room).unwrap(), svg, "{texts}");
        // Which is another chart than the one without the room, where
        // there is any.
        let over = margined!(chart, (0.0, 0.0, 0.0, 0.0)).unwrap();
        assert_eq!(room == (0.0, 0.0, 0.0, 0.0), over == svg, "{texts}");
    }

    // A text turned by less reaches as far as its turned box does: here
    // from 6 above the bottom, about its middle.
    let turned = BarChart::from_json(&format!(
        r#"{{{DATA}, "texts": [{{"text": "note", "bottom": 6, "rotate": 45, "reserve": true}}]}}"#
    ))
    .unwrap();
    let plain = BarChart::from_json(&format!("{{{DATA}}}")).unwrap();
    let reach = 12.0 + (width / 2.0 + 6.0) * 45_f32.to_radians().sin();
    let moved = number(tag_of(&plain.svg().unwrap(), "a"), "y")
        - number(tag_of(&turned.svg().unwrap(), "a"), "y");
    assert!((moved - reach).abs() < 0.2, "{moved} {reach}");
}

#[test]
fn a_title_is_below_the_room_at_the_top() {
    let chart = |options: &str| -> String {
        BarChart::from_json(&format!(
            r#"{{{DATA}, "title_text": "Title", "sub_title_text": "Sub"{options}}}"#
        ))
        .unwrap()
        .svg()
        .unwrap()
    };
    let y = |svg: &str, text: &str| number(tag_of(svg, text), "y");
    let plain = chart("");
    // All of the chart is 14 pixels further down, and ends where it did.
    let above =
        chart(r#", "texts": [{"text": "note", "top": 4, "font_size": 10, "reserve": true}]"#);
    assert_eq!(9.0, y(&above, "note"));
    for text in ["Title", "Sub", "s"] {
        assert_eq!(y(&plain, text) + 14.0, y(&above, text), "{text}");
    }
    assert_eq!(y(&plain, "a"), y(&above, "a"));
    // The plot starts as far below the title as it does without the text.
    let top = |svg: &str| -> f32 {
        svg.split("<text")
            .filter(|tag| tag.contains("text-anchor=\"end\""))
            .map(|tag| number(tag, "y"))
            .fold(f32::MAX, f32::min)
    };
    assert_eq!(top(&plain) + 14.0, top(&above));

    // And below, all of it ends 18 pixels higher and starts where it did.
    let below = chart(r#", "texts": [{"text": "note", "bottom": 6, "reserve": true}]"#);
    for text in ["Title", "Sub", "s"] {
        assert_eq!(y(&plain, text), y(&below, text), "{text}");
    }
    assert_eq!(top(&plain), top(&below));
    assert_eq!(y(&plain, "a") - 18.0, y(&below, "a"));
    // So does a legend at the bottom: above the text, not under it.
    let legend = |options: &str| {
        y(
            &chart(&format!(r#", "legend_position": "bottom"{options}"#)),
            "s",
        )
    };
    assert_eq!(
        legend("") - 18.0,
        legend(r#", "texts": [{"text": "note", "bottom": 6, "reserve": true}]"#)
    );
}

#[test]
fn every_chart_makes_room_for_its_texts() {
    // 12 pixels high and 6 from the edge: 18 across the top or the bottom,
    // 6 and its width down a side.
    let width = measure_text_width_family("Roboto", 12.0, "© ACME")
        .unwrap()
        .width();
    let places = [
        (r#""bottom": 6"#, (0.0, 0.0, 0.0, 18.0)),
        (r#""top": 6"#, (0.0, 18.0, 0.0, 0.0)),
        (r#""left": 6"#, (6.0 + width, 0.0, 0.0, 0.0)),
        (r#""right": 6"#, (0.0, 0.0, 6.0 + width, 0.0)),
    ];
    let (mut names, mut moved): (Vec<&str>, Vec<&str>) = (vec![], vec![]);
    let mut drawn = 0;
    macro_rules! chart {
        ($name:literal, $chart:ty, $($json:expr),+) => {
            $(for (place, room) in places {
                let texts = format!(r#""texts": [{{"text": "© ACME", {place}, "reserve": true}}]"#);
                let json = if $json.is_empty() {
                    format!("{{{texts}}}")
                } else {
                    format!("{{{}, {texts}}}", $json)
                };
                let chart = <$chart>::from_json(&json).unwrap();
                assert!(chart.texts[0].reserve, "{}", $name);
                // A table has no margin, and is looked at below.
                if $name == "table" {
                    continue;
                }
                match (chart.svg(), margined!(chart, room)) {
                    (Ok(reserved), Ok(margined)) => {
                        // A heatmap makes its y axis wider by its left
                        // margin as well, which the room is none of.
                        if $name != "heatmap" || room.0 == 0.0 {
                            assert_eq!(margined, reserved, "{}: {place}", $name);
                        }
                        // Which is another picture than with the text
                        // over the chart, where the chart has anything
                        // that is in the way.
                        if margined!(chart, (0.0, 0.0, 0.0, 0.0)).unwrap() != reserved {
                            moved.push($name);
                        }
                        names.push($name);
                        drawn += 1;
                    }
                    (reserved, margined) => assert!(
                        $json.is_empty() && reserved.is_err() && margined.is_err(),
                        "{}: {place}",
                        $name
                    ),
                }
            })+
        };
    }
    every_chart!(chart);
    names.dedup();
    assert_eq!(27, names.len(), "{names:?}");
    assert!(drawn > 160, "{drawn}");
    moved.dedup();
    assert_eq!(names, moved);
}

/// The width and the height of the picture.
fn size(svg: &str) -> (f32, f32) {
    let head = &svg[..svg.find('>').unwrap()];
    (number(head, "width"), number(head, "height"))
}

#[test]
fn a_table_makes_room_around_its_rows() {
    let table = |texts: &str| -> String {
        TableChart::from_json(&format!(
            r#"{{"title_text": "Prices", "data": [["Name", "Price"], ["A", "1"]], "texts": [{texts}]}}"#
        ))
        .unwrap()
        .svg()
        .unwrap()
    };
    let at = |svg: &str, text: &str| -> (f32, f32) {
        let tag = tag_of(svg, text);
        (number(tag, "x"), number(tag, "y"))
    };
    let width = note_width();
    // A table is as high as its rows, and a text is over the last of them.
    let over = table(r#"{"text": "note", "bottom": 6}"#);
    let (wide, high) = size(&over);
    assert!(at(&over, "note").1 < at(&over, "A").1 + 40.0);
    assert!(!over.contains(&format!(r#"height="{high}" fill"#)));

    // With room for it the table is higher by that, and the text is under
    // the last row: in the background of the table, which is behind all.
    let below = table(r#"{"text": "note", "bottom": 6, "reserve": true}"#);
    assert_eq!((wide, high + 18.0), size(&below));
    assert_eq!(high + 6.0, at(&below, "note").1);
    for text in ["Prices", "Name", "Price", "A", "1"] {
        assert_eq!(at(&over, text), at(&below, text), "{text}");
    }
    assert!(below.contains(&format!(
        "\">\n<rect x=\"0\" y=\"0\" width=\"600\" height=\"{}\" fill=\"#FFFFFF\"/>\n<rect",
        high + 18.0
    )));

    // Above, the title and the rows are that much further down.
    let above = table(r#"{"text": "note", "top": 6, "reserve": true}"#);
    assert_eq!((wide, high + 18.0), size(&above));
    assert_eq!(12.0, at(&above, "note").1);
    for text in ["Prices", "Name", "Price", "A", "1"] {
        let (x, y) = at(&over, text);
        assert_eq!((x, y + 18.0), at(&above, text), "{text}");
    }
    // Below a sub-title as well, and without a title.
    for head in [r#""sub_title_text": "per unit", "#, r#""title_text": "", "#] {
        let table = |texts: &str| -> String {
            TableChart::from_json(&format!(
                r#"{{"title_text": "Prices", {head}"data": [["Name"], ["A"]], "texts": [{texts}]}}"#
            ))
            .unwrap()
            .svg()
            .unwrap()
        };
        let over = table(r#"{"text": "note", "top": 6}"#);
        let above = table(r#"{"text": "note", "top": 6, "reserve": true}"#);
        assert_eq!(size(&over).1 + 18.0, size(&above).1, "{head}");
        for text in ["Name", "A"] {
            assert_eq!(at(&over, text).1 + 18.0, at(&above, text).1, "{head}");
        }
    }

    // At a side the columns share what is left of the width.
    let room = 8.0 + width;
    let left = table(r#"{"text": "note", "left": 8, "reserve": true}"#);
    assert_eq!((wide, high), size(&left));
    assert_eq!(at(&over, "Name").0 + room, at(&left, "Name").0);
    assert_eq!(at(&over, "Price").0 + room / 2.0, at(&left, "Price").0);
    assert_eq!(at(&over, "Prices").0 + room / 2.0, at(&left, "Prices").0);
    let right = table(r#"{"text": "note", "right": 8, "reserve": true}"#);
    assert_eq!((wide, high), size(&right));
    assert_eq!(at(&over, "Name").0, at(&right, "Name").0);
    assert_eq!(at(&over, "Price").0 - room / 2.0, at(&right, "Price").0);
    assert!(right.contains(&format!(r#"width="{}" height="40""#, wide - room)));

    // Nothing of this without a text that asks for it.
    assert_eq!(
        table(r#"{"text": "note", "bottom": 6}"#),
        table(r#"{"text": "note", "bottom": 6, "reserve": false}"#)
    );
}

#[test]
fn a_calendar_is_larger_by_the_room() {
    let calendar = |options: &str| -> String {
        CalendarChart::from_json(&format!(
            r#"{{"title_text": "Days", "start_date": "2024-02-01", "end_date": "2024-03-31",
                "data": [["2024-02-05", 3]]{options}}}"#
        ))
        .unwrap()
        .svg()
        .unwrap()
    };
    let at = |svg: &str, text: &str| -> (f32, f32) {
        let tag = tag_of(svg, text);
        (number(tag, "x"), number(tag, "y"))
    };
    let width = note_width();
    // A calendar is as large as its days are: with room for a text, it is
    // larger by that, and its days are as large as they were.
    let plain = calendar("");
    let (wide, high) = size(&plain);
    let cells = |svg: &str| svg.matches("<rect").count();
    for (place, (left, top, right, bottom)) in [
        (r#""bottom": 6"#, (0.0, 0.0, 0.0, 18.0)),
        (r#""top": 6"#, (0.0, 18.0, 0.0, 0.0)),
        (r#""left": 8"#, (8.0 + width, 0.0, 0.0, 0.0)),
        (r#""right": 8"#, (0.0, 0.0, 8.0 + width, 0.0)),
    ] {
        let svg = calendar(&format!(
            r#", "texts": [{{"text": "note", {place}, "reserve": true}}]"#
        ));
        assert_eq!(
            (wide + left + right, high + top + bottom),
            size(&svg),
            "{place}"
        );
        assert_eq!(cells(&plain), cells(&svg), "{place}");
        // The days start below and beside the room, and so does the title.
        let (x, y) = at(&plain, "Mon");
        assert_eq!((x + left, y + top), at(&svg, "Mon"), "{place}");
        assert_eq!(at(&plain, "Days").1 + top, at(&svg, "Days").1, "{place}");
        // Over the days, the calendar is as large as without the text.
        let over = calendar(&format!(r#", "texts": [{{"text": "note", {place}}}]"#));
        assert_eq!((wide, high), size(&over), "{place}");
    }
}

#[test]
fn a_multi_chart_makes_room_around_its_children() {
    let multi = |texts: &str, place: &str| -> String {
        MultiChart::from_json(&format!(
            r#"{{"margin": 10, "gap": 10, "texts": [{texts}],
                "child_charts": [{{{DATA}{place}}}, {{{DATA}, "height": 200}}]}}"#
        ))
        .unwrap()
        .svg()
        .unwrap()
    };
    // Where the children are: the x and the y of their pictures.
    let children = |svg: &str| -> Vec<(f32, f32)> {
        svg.split("<svg")
            .skip(2)
            .map(|tag| {
                let head = &tag[..tag.find('>').unwrap()];
                (number(head, "x"), number(head, "y"))
            })
            .collect()
    };
    let width = note_width();
    let over = multi(r#"{"text": "note", "bottom": 6}"#, "");
    let (wide, high) = size(&over);
    assert_eq!((620.0, 630.0), (wide, high));
    assert_eq!(vec![(10.0, 10.0), (10.0, 420.0)], children(&over));

    for (place, (left, top, right, bottom)) in [
        (r#""bottom": 6"#, (0.0, 0.0, 0.0, 18.0)),
        (r#""top": 6"#, (0.0, 18.0, 0.0, 0.0)),
        (r#""left": 8"#, (8.0 + width, 0.0, 0.0, 0.0)),
        (r#""right": 8"#, (0.0, 0.0, 8.0 + width, 0.0)),
    ] {
        let text = format!(r#"{{"text": "note", {place}, "reserve": true}}"#);
        // The picture is larger by the room, which is around its margin.
        let svg = multi(&text, "");
        assert_eq!(
            (wide + left + right, high + top + bottom),
            size(&svg),
            "{place}"
        );
        assert_eq!(
            vec![(10.0 + left, 10.0 + top), (10.0 + left, 420.0 + top)],
            children(&svg),
            "{place}"
        );
        // The text is placed on all of it, as it is without the room.
        let tag = tag_of(&svg, "note");
        if bottom > 0.0 {
            assert_eq!(high + bottom - 12.0, number(tag, "y"), "{place}");
        }
        if right > 0.0 {
            assert_eq!(wide + right - 8.0, number(tag, "x"), "{place}");
        }
        // A child placed by hand stays where it is put: the room at the
        // top and at the left is for those that are not.
        let placed = multi(&text, r#", "x": 3, "y": 2"#);
        assert_eq!((3.0, 2.0), children(&placed)[0], "{place}");
    }

    // In Rust.
    let mut multi = MultiChart::new();
    multi.add(ChildChart::Bar(
        BarChart::new(
            vec![("s", vec![1.0, 2.0]).into()],
            vec!["a".to_string(), "b".to_string()],
        ),
        None,
    ));
    let plain = size(&multi.svg().unwrap());
    multi.texts = vec![TextConfig {
        text: "mine".to_string(),
        bottom: Some(6.0),
        reserve: true,
        ..Default::default()
    }];
    let svg = multi.svg().unwrap();
    assert_eq!((plain.0, plain.1 + 18.0), size(&svg));
    assert_eq!(plain.1 + 6.0, number(tag_of(&svg, "mine"), "y"));
}

#[test]
fn room_for_texts_in_rust_and_in_a_theme() {
    let series = || -> Vec<Series> { vec![("s", vec![1.0, 3.0, 2.0]).into()] };
    let labels = || -> Vec<String> { vec!["a".into(), "b".into(), "c".into()] };
    let note = |reserve: bool| TextConfig {
        text: "note".to_string(),
        bottom: Some(6.0),
        reserve,
        ..Default::default()
    };
    // The same in Rust as from JSON, and nothing where it is not asked for.
    let from_json = BarChart::from_json(&format!(
        r#"{{{DATA}, "texts": [{{"text": "note", "bottom": 6, "reserve": true}}]}}"#
    ))
    .unwrap();
    assert_eq!(vec![note(true)], from_json.texts);
    let mut chart = BarChart::new(series(), labels());
    chart.texts = vec![note(true)];
    assert_eq!(from_json.svg().unwrap(), chart.svg().unwrap());
    let y = |svg: &str| number(tag_of(svg, "a"), "y");
    let reserved = y(&chart.svg().unwrap());
    chart.texts = vec![note(false)];
    assert_eq!(reserved + 18.0, y(&chart.svg().unwrap()));
    assert!(!TextConfig::default().reserve);

    // The texts of a theme make room on every chart of it.
    let mut theme = (*get_theme("light")).clone();
    theme.texts = vec![note(true)];
    add_theme("texts-reserve-test", theme);
    let themed = BarChart::new_with_theme(series(), labels(), "texts-reserve-test");
    assert_eq!(reserved, y(&themed.svg().unwrap()));
    let table = |theme: &str| {
        let data = vec![vec!["Name".to_string()], vec!["A".to_string()]];
        size(&TableChart::new_with_theme(data, theme).svg().unwrap())
    };
    assert_eq!(table("light").1 + 18.0, table("texts-reserve-test").1);

    // Numbers that are none make no room, and break nothing.
    for text in [
        TextConfig {
            bottom: Some(f32::NAN),
            ..note(true)
        },
        TextConfig {
            bottom: Some(f32::INFINITY),
            ..note(true)
        },
        TextConfig {
            font: FontConfig {
                size: f32::INFINITY,
                ..Default::default()
            },
            ..note(true)
        },
        TextConfig {
            text: String::new(),
            bottom: Some(90.0),
            ..note(true)
        },
    ] {
        let mut chart = BarChart::new(series(), labels());
        chart.texts = vec![text];
        let svg = chart.svg().unwrap();
        assert_eq!(reserved + 18.0, y(&svg));
        assert!(!svg.contains("NaN") && !svg.contains("inf"));
    }
    // A turn that is none is no turn.
    let mut chart = BarChart::new(series(), labels());
    chart.texts = vec![TextConfig {
        rotate: f32::NAN,
        ..note(true)
    }];
    assert_eq!(reserved, y(&chart.svg().unwrap()));
}

#[test]
fn room_for_more_than_there_is_breaks_no_chart() {
    // More room than the chart has, at each edge and at all of them: what
    // is left of the chart is nothing, and it is drawn as that.
    let texts = [
        r#"{"text": "far", "bottom": 5000, "reserve": true}"#.to_string(),
        r#"{"text": "far", "top": 1e30, "reserve": true}"#.to_string(),
        r#"{"text": "far", "left": 5000, "reserve": true}"#.to_string(),
        r#"{"text": "far", "right": 1e30, "reserve": true}"#.to_string(),
        r#"{"text": "large", "left": 0, "font_size": 4000, "reserve": true}"#.to_string(),
        r#"{"text": "turned", "bottom": 0, "rotate": 1e30, "reserve": true}"#.to_string(),
        ["top", "bottom", "left", "right"]
            .map(|edge| format!(r#"{{"text": "all", "{edge}": 390, "reserve": true}}"#))
            .join(", "),
    ];
    let mut drawn = 0;
    macro_rules! chart {
        ($name:literal, $chart:ty, $($json:expr),+) => {
            $(for texts in texts.iter() {
                let json = if $json.is_empty() {
                    format!(r#"{{"title_text": "T", "texts": [{texts}]}}"#)
                } else {
                    format!(r#"{{{}, "title_text": "T", "texts": [{texts}]}}"#, $json)
                };
                if let Ok(svg) = <$chart>::from_json(&json).unwrap().svg() {
                    assert!(!svg.contains("NaN") && !svg.contains("inf"), "{}: {texts}", $name);
                    drawn += 1;
                }
            })+
        };
    }
    every_chart!(chart);
    assert!(drawn > 200, "{drawn}");
    for texts in texts.iter() {
        let svg = MultiChart::from_json(&format!(
            r#"{{"texts": [{texts}], "child_charts": [{{{DATA}}}]}}"#
        ))
        .unwrap()
        .svg()
        .unwrap();
        assert!(!svg.contains("NaN") && !svg.contains("inf"), "{texts}");
    }
}
