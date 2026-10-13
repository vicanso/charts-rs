mod common;

use charts_rs::SunburstChart;

const DATA: &str = r##"
    "inner_radius": 20,
    "series_data": [
        {
            "name": "Grandpa",
            "children": [
                {"name": "Uncle Leo", "children": [
                    {"name": "Cousin Jack", "value": 18},
                    {"name": "Cousin Mary", "value": 12}
                ]},
                {"name": "Father", "children": [
                    {"name": "Me", "value": 40},
                    {"name": "Brother Peter", "value": 20}
                ]}
            ]
        },
        {
            "name": "Nancy",
            "children": [
                {"name": "Uncle Nike", "children": [
                    {"name": "Cousin Betty", "value": 10},
                    {"name": "Cousin Jenny", "value": 30}
                ]}
            ]
        }
    ]
"##;

#[test]
fn sunburst_chart_basic_json() {
    let chart =
        SunburstChart::from_json(&format!(r##"{{"title_text": "Sunburst",{DATA}}}"##)).unwrap();
    common::assert_snapshot!("sunburst_chart/basic_json.svg", chart.svg().unwrap());
}

#[test]
fn sunburst_chart_grafana_json() {
    let chart = SunburstChart::from_json(&format!(
        r##"{{"theme": "grafana", "title_text": "Sunburst",{DATA}}}"##
    ))
    .unwrap();
    common::assert_snapshot!("sunburst_chart/grafana_json.svg", chart.svg().unwrap());
}

/// The names written on the arcs, each with the degrees it is turned by.
fn labels(svg: &str) -> Vec<(String, f32)> {
    svg.split("<text")
        .skip(1)
        .filter(|tag| !tag.contains("class=\"ct-tip\""))
        .filter_map(|tag| {
            let start = tag.find("transform=\"rotate(")? + 18;
            let turn = tag[start..start + tag[start..].find(' ')?].parse().ok()?;
            Some((tag.split('\n').nth(1)?.to_string(), turn))
        })
        .collect()
}

/// A ring of 120 pixels around a hole of 60, the roots as its slices: a
/// degree of it for each unit of their values.
fn ring(slices: &[(&str, u32)], options: &str) -> Vec<(String, f32)> {
    assert_eq!(360, slices.iter().map(|(_, value)| value).sum::<u32>());
    let data: Vec<String> = slices
        .iter()
        .map(|(name, value)| format!(r#"{{"name": "{name}", "value": {value}}}"#))
        .collect();
    let json = format!(
        r#"{{"width": 400, "height": 400, "margin": 0, "legend_show": false,
            "radius": 180, "inner_radius": 60{options}, "series_data": [{}]}}"#,
        data.join(",")
    );
    labels(&SunburstChart::from_json(&json).unwrap().svg().unwrap())
}

#[test]
fn sunburst_label_too_long_for_its_arc_is_along_the_radius() {
    let turn = |labels: &[(String, f32)], name: &str| -> Option<f32> {
        labels
            .iter()
            .find(|(text, _)| text == name)
            .map(|(_, turn)| *turn)
    };
    // Slices from 12 o'clock on: 200 degrees, 20, 4 and 136.
    let slices = [
        ("Wide slice", 200),
        ("A narrow slice", 20),
        ("tiny", 4),
        ("Left side", 136),
    ];
    let found = ring(&slices, "");
    // A name that its arc is long enough for is along the arc, as it was:
    // turned as the middle of the slice is, and by half a turn less on the
    // lower half of the chart, where it would be upside down.
    assert_eq!(Some(100.0 - 180.0), turn(&found, "Wide slice"));
    assert_eq!(Some(292.0), turn(&found, "Left side"));
    // The arc of 20 degrees is 42 pixels at the middle of the ring and the
    // name more than twice that: it was left out. The ring is thick enough
    // for it, and it is written along the radius now — on the left half
    // of the chart towards the middle, a quarter turn on from its slice.
    assert_eq!(Some(210.0 + 90.0), turn(&found, "A narrow slice"));
    // A slice that is not as wide as a line of text has no name still.
    assert_eq!(None, turn(&found, "tiny"));
    assert_eq!(3, found.len());

    // On the right half it reads from the middle outwards: a quarter turn
    // back from its slice, whose middle is at 10 degrees here and at 170.
    let found = ring(&[("A narrow slice", 20), ("Wide slice", 340)], "");
    assert_eq!(Some(10.0 - 90.0), turn(&found, "A narrow slice"));
    let found = ring(
        &[("First", 160), ("A narrow slice", 20), ("Wide slice", 180)],
        "",
    );
    assert_eq!(Some(170.0 - 90.0), turn(&found, "A narrow slice"));
    // And straight down or up, from the middle and to it.
    let found = ring(
        &[("First", 170), ("A narrow slice", 20), ("Wide slice", 170)],
        "",
    );
    assert_eq!(Some(270.0), turn(&found, "A narrow slice"));

    // In a ring that is thinner than the name is long there is no room for
    // it that way either.
    let found = ring(&slices, r#", "level_thickness": [1], "inner_radius": 140"#);
    assert_eq!(None, turn(&found, "A narrow slice"));
    assert_eq!(Some(-80.0), turn(&found, "Wide slice"));
    // A short name has room in it: "tiny" in a slice of 30 degrees, which
    // is too short an arc for it along a ring of 40 pixels.
    let found = ring(
        &[("Wide slice", 200), ("tiny", 30), ("Left side", 130)],
        r#", "inner_radius": 30, "radius": 70"#,
    );
    assert_eq!(Some(215.0 + 90.0), turn(&found, "tiny"));
    // At the end of the name that is nearer to the middle the slice has to
    // be as wide as the text is high: the same slice of a ring nearer to
    // the middle is not.
    let found = ring(
        &[
            ("Wide slice", 200),
            ("A narrow slice", 20),
            ("Left side", 140),
        ],
        r#", "inner_radius": 5, "radius": 125"#,
    );
    assert_eq!(None, turn(&found, "A narrow slice"));
}
