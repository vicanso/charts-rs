mod common;

use charts_rs::GaugeChart;

#[test]
fn gauge_chart_basic_json() {
    let chart = GaugeChart::from_json(
        r##"{
            "title_text": "Gauge",
            "min": 0,
            "max": 200,
            "series_list": [{"name": "Speed", "data": [120]}]
        }"##,
    )
    .unwrap();
    common::assert_snapshot!("gauge_chart/basic_json.svg", chart.svg().unwrap());
}

#[test]
fn gauge_chart_grafana_json() {
    let chart = GaugeChart::from_json(
        r##"{
            "theme": "grafana",
            "title_text": "Gauge",
            "min": 0,
            "max": 200,
            "series_list": [{"name": "Speed", "data": [120]}]
        }"##,
    )
    .unwrap();
    common::assert_snapshot!("gauge_chart/grafana_json.svg", chart.svg().unwrap());
}

/// The arcs of a gauge as `(color, width, number of points)`.
fn arcs(svg: &str) -> Vec<(String, String, usize)> {
    svg.split("<polyline")
        .skip(1)
        .map(|t| {
            let attr = |name: &str| {
                let key = format!(" {name}=\"");
                let start = t.find(&key).unwrap() + key.len();
                t[start..start + t[start..].find('"').unwrap()].to_string()
            };
            (
                attr("stroke"),
                attr("stroke-width"),
                attr("points").split(' ').count(),
            )
        })
        .collect()
}

/// The fills of the pointers.
fn pointers(svg: &str) -> Vec<String> {
    svg.split("<polygon")
        .skip(1)
        .map(|t| {
            let start = t.find(" fill=\"").unwrap() + 7;
            t[start..start + 7].to_string()
        })
        .collect()
}

fn texts(svg: &str) -> Vec<&str> {
    svg.lines()
        .map(str::trim)
        .filter(|l| !l.is_empty() && !l.starts_with('<'))
        .collect()
}

#[test]
fn gauge_segments() {
    let chart = GaugeChart::from_json(include_str!("../asset/gauge_chart/segments.json")).unwrap();
    assert_eq!(vec![80.0, 140.0], chart.thresholds);
    let svg = chart.svg().unwrap();
    common::assert_snapshot!("gauge_chart/segments_json.svg", svg);

    // The scale in three segments, as long as their share of it; no arc of
    // progress over them.
    let arcs_of = arcs(&svg);
    let colors: Vec<&str> = arcs_of.iter().map(|a| a.0.as_str()).collect();
    assert_eq!(vec!["#91CC75", "#FAC858", "#EE6666"], colors);
    let lengths: Vec<usize> = arcs_of.iter().map(|a| a.2 - 1).collect();
    assert_eq!(vec![144, 108, 108], lengths);
    // The pointer is in the color of the segment it points at: 156 is in
    // the last one.
    assert_eq!(vec!["#EE6666"], pointers(&svg));
    assert!(texts(&svg).contains(&"156 km/h"));

    // Without colors of their own the segments take those of the series;
    // a threshold off the scale, or given twice, makes no segment.
    let json = |value: f32, rest: &str| {
        format!(
            r##"{{"thresholds": [50, 50, -10, 100, 700], "series_list": [{{"name": "a", "data": [{value}]}}]{rest}}}"##
        )
    };
    let svg = GaugeChart::from_json(&json(20.0, ""))
        .unwrap()
        .svg()
        .unwrap();
    let colors: Vec<String> = arcs(&svg).into_iter().map(|a| a.0).collect();
    assert_eq!(vec!["#5470C6", "#91CC75"], colors);
    assert_eq!(vec!["#5470C6"], pointers(&svg));
    let svg = GaugeChart::from_json(&json(50.0, ""))
        .unwrap()
        .svg()
        .unwrap();
    assert_eq!(vec!["#91CC75"], pointers(&svg));
    // A pointer color of one's own is kept.
    let svg = GaugeChart::from_json(&json(50.0, r##", "pointer_color": "#123456""##))
        .unwrap()
        .svg()
        .unwrap();
    assert_eq!(vec!["#123456"], pointers(&svg));
}

#[test]
fn gauge_with_several_pointers() {
    let json = r##"{"value_formatter": "{c}%", "series_list": [
        {"name": "CPU", "data": [72]},
        {"name": "Memory", "data": [48.5]},
        {"name": "Idle", "data": []},
        {"name": "Disk", "data": [230]}
    ]}"##;
    let svg = GaugeChart::from_json(json).unwrap().svg().unwrap();
    // A pointer for every series that has a value, in its color; the arc
    // is the scale alone.
    assert_eq!(vec!["#5470C6", "#91CC75", "#EE6666"], pointers(&svg));
    assert_eq!(1, arcs(&svg).len());
    // The values are listed, as they were given.
    let texts = texts(&svg);
    for line in ["CPU: 72%", "Memory: 48.5%", "Disk: 230%"] {
        assert!(texts.contains(&line), "{line} in {texts:?}");
    }
    assert!(!texts.iter().any(|t| t.starts_with("Idle:")));

    // One series: the arc of its progress, and its value written large.
    let single = r##"{"series_list": [{"name": "CPU", "data": [72]}]}"##;
    let svg = GaugeChart::from_json(single).unwrap().svg().unwrap();
    assert_eq!(2, arcs(&svg).len());
    assert_eq!(1, pointers(&svg).len());
    assert!(self::texts(&svg).contains(&"72"));
}

#[test]
fn gauge_rings() {
    let chart = GaugeChart::from_json(include_str!("../asset/gauge_chart/rings.json")).unwrap();
    assert!(chart.multi_ring);
    let svg = chart.svg().unwrap();
    common::assert_snapshot!("gauge_chart/rings_json.svg", svg);

    // A track and the progress on it for every series; no pointer.
    let arcs_of = arcs(&svg);
    assert_eq!(6, arcs_of.len());
    assert!(pointers(&svg).is_empty());
    let colors: Vec<&str> = arcs_of.iter().map(|a| a.0.as_str()).collect();
    assert_eq!(
        vec![
            "#E6E6E6", "#5470C6", "#E6E6E6", "#91CC75", "#E6E6E6", "#FAC858"
        ],
        colors
    );
    assert!(arcs_of.iter().all(|a| a.1 == "22"));
    // The progress is as long as the value: 82, 64 and 45% of the turn.
    let lengths: Vec<usize> = arcs_of.iter().map(|a| a.2 - 1).collect();
    assert_eq!(vec![360, 295, 360, 230, 360, 162], lengths);
    // The rings lie inside each other, and the values in the middle.
    let first_y = |n: usize| -> f32 {
        let tag = svg.split("<polyline").nth(n + 1).unwrap();
        let start = tag.find("points=\"").unwrap() + 8;
        let point = tag[start..].split(' ').next().unwrap();
        point.split_once(',').unwrap().1.parse().unwrap()
    };
    assert!((first_y(2) - first_y(0) - 26.0).abs() < 0.1);
    assert!((first_y(4) - first_y(2) - 26.0).abs() < 0.1);
    let texts = texts(&svg);
    for line in ["Move 82%", "Exercise 64%", "Stand 45%"] {
        assert!(texts.contains(&line), "{line} in {texts:?}");
    }

    // Many rings are thinned to leave the middle free.
    let series: Vec<String> = (0..12)
        .map(|i| format!(r#"{{"name": "s{i}", "data": [{}]}}"#, i * 8))
        .collect();
    let json = format!(
        r#"{{"multi_ring": true, "arc_width": 30, "series_list": [{}]}}"#,
        series.join(", ")
    );
    let svg = GaugeChart::from_json(&json).unwrap().svg().unwrap();
    assert!(!svg.contains("NaN"));
    let width: f32 = arcs(&svg)[0].1.parse().unwrap();
    assert!((1.0..15.0).contains(&width), "{width}");

    let message = match GaugeChart::from_json(r#"{"multi_ring": 1}"#) {
        Ok(_) => panic!("accepted"),
        Err(e) => e.to_string(),
    };
    assert!(message.contains("multi_ring"), "{message}");
}
