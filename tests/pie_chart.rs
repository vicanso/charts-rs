mod common;

use charts_rs::PieChart;

#[test]
fn pie_chart() {
    let pie_chart = PieChart::from_json(
        r###"{
        "title_text": "Nightingale Chart",
        "sub_title_text": "Fake Data",
        "legend_show": false,
        "radius": 130,
        "inner_radius": 30,
        "series_list": [
            {
                "name": "rose 1",
                "data": [40]
            },
            {
                "name": "rose 2",
                "data": [38]
            },
            {
                "name": "rose 3",
                "data": [32]
            },
            {
                "name": "rose 4",
                "data": [30]
            },
            {
                "name": "rose 5",
                "data": [28]
            },
            {
                "name": "rose 6",
                "data": [26]
            },
            {
                "name": "rose 7",
                "data": [22]
            },
            {
                "name": "rose 8",
                "data": [18]
            }
        ]
    }"###,
    )
    .unwrap();

    common::assert_snapshot!("pie_chart/basic_json.svg", pie_chart.svg().unwrap());
}

#[test]
fn not_rose_radius_pie_chart() {
    let pie_chart = PieChart::from_json(
        r###"{
        "title_text": "Nightingale Chart",
        "sub_title_text": "Fake Data",
        "legend_show": false,
        "radius": 130,
        "inner_radius": 0,
        "border_radius": 0,
        "rose_type": false,
        "series_list": [
            {
                "name": "rose 1",
                "data": [40]
            },
            {
                "name": "rose 2",
                "data": [38]
            },
            {
                "name": "rose 3",
                "data": [32]
            },
            {
                "name": "rose 4",
                "data": [30]
            },
            {
                "name": "rose 5",
                "data": [28]
            },
            {
                "name": "rose 6",
                "data": [26]
            },
            {
                "name": "rose 7",
                "data": [22]
            },
            {
                "name": "rose 8",
                "data": [18]
            }
        ]
    }"###,
    )
    .unwrap();

    common::assert_snapshot!(
        "pie_chart/not_rose_radius_json.svg",
        pie_chart.svg().unwrap()
    );
}

/// Every coordinate pair of the slices of a pie (the paths that carry a
/// series).
fn slice_points(svg: &str) -> Vec<(f32, f32)> {
    svg.split("<path")
        .skip(1)
        .filter(|t| t.contains("data-series="))
        .flat_map(|t| {
            let start = t.find(" d=\"").unwrap() + 4;
            let d = &t[start..start + t[start..].find('"').unwrap()];
            d.split(' ')
                .filter_map(|token| {
                    let (x, y) = token.trim_start_matches(['M', 'L']).split_once(',')?;
                    Some((x.parse().ok()?, y.parse().ok()?))
                })
                .collect::<Vec<(f32, f32)>>()
        })
        .collect()
}

/// `(left, top, right, bottom)` of the slices.
fn slice_box(svg: &str) -> (f32, f32, f32, f32) {
    slice_points(svg).iter().fold(
        (f32::MAX, f32::MAX, f32::MIN, f32::MIN),
        |(l, t, r, b), (x, y)| (l.min(*x), t.min(*y), r.max(*x), b.max(*y)),
    )
}

fn plain_pie() -> PieChart {
    let mut chart = PieChart::new(vec![
        ("a", vec![25.0]).into(),
        ("b", vec![25.0]).into(),
        ("c", vec![25.0]).into(),
        ("d", vec![25.0]).into(),
    ]);
    // Equal slices end on the axes, where the pie is widest and highest.
    chart.rose_type = Some(false);
    chart.inner_radius = 50.0;
    chart.radius = 100.0;
    chart.border_radius = Some(0.0);
    chart
}

#[test]
fn pie_half_doughnut() {
    let chart = PieChart::from_json(include_str!("../asset/pie_chart/half.json")).unwrap();
    assert_eq!((-90.0, Some(90.0)), (chart.start_angle, chart.end_angle));
    common::assert_snapshot!("pie_chart/half_json.svg", chart.svg().unwrap());
}

#[test]
fn pie_end_angle() {
    let full = plain_pie();
    let (left, top, right, bottom) = slice_box(&full.svg().unwrap());
    assert!((right - left - 200.0).abs() < 1.0 && (bottom - top - 200.0).abs() < 1.0);

    // The upper half: as wide as the whole pie, half as high, and in the
    // middle of the plot, not in the upper half of it.
    let mut half = plain_pie();
    half.start_angle = -90.0;
    half.end_angle = Some(90.0);
    let svg = half.svg().unwrap();
    let (left, top, right, bottom) = slice_box(&svg);
    assert!((right - left - 200.0).abs() < 1.0, "{left} {right}");
    assert!((bottom - top - 100.0).abs() < 1.0, "{top} {bottom}");
    assert!(((left + right) / 2.0 - 300.0).abs() < 1.0);
    let middle = (top + bottom) / 2.0;
    assert!((middle - 200.0).abs() < 10.0, "{middle}");
    // The slices still tell their share of the whole.
    assert!(svg.contains("a: 25%") && svg.contains("d: 25%"));
    // A label left of the pie ends at its line, one on the right starts there.
    let text_x = |name: &str| -> f32 {
        let tag = svg
            .split("<text")
            .find(|t| t.contains(&format!("\n{name}: ")))
            .unwrap();
        let start = tag.find(" x=\"").unwrap() + 4;
        tag[start..start + tag[start..].find('"').unwrap()]
            .parse()
            .unwrap()
    };
    assert!(
        text_x("a") < left && text_x("d") > right,
        "{} {}",
        text_x("a"),
        text_x("d")
    );

    // A quarter, in a rose: every slice the same angle, a larger radius
    // allowed as there is room for it.
    let mut quarter = plain_pie();
    quarter.rose_type = Some(true);
    quarter.inner_radius = 0.0;
    quarter.radius = 250.0;
    quarter.end_angle = Some(90.0);
    let (left, top, right, bottom) = slice_box(&quarter.svg().unwrap());
    assert!((right - left - 250.0).abs() < 1.0, "{left} {right}");
    assert!((bottom - top - 250.0).abs() < 1.0, "{top} {bottom}");

    // An end that is not after the start, or a turn or more away from it,
    // is the full circle.
    for end in [
        None,
        Some(0.0),
        Some(-30.0),
        Some(360.0),
        Some(1000.0),
        Some(f32::NAN),
    ] {
        let mut chart = plain_pie();
        chart.end_angle = end;
        assert_eq!(full.svg().unwrap(), chart.svg().unwrap(), "{end:?}");
    }
    // A sliver, and a tiny canvas: nothing breaks.
    let mut sliver = plain_pie();
    sliver.end_angle = Some(0.5);
    sliver.width = 40.0;
    sliver.height = 30.0;
    let svg = sliver.svg().unwrap();
    assert!(!svg.contains("NaN") && !svg.contains("inf"));

    let message = match PieChart::from_json(r#"{"end_angle": "90"}"#) {
        Ok(_) => panic!("accepted"),
        Err(e) => e.to_string(),
    };
    assert!(message.contains("end_angle"), "{message}");
}

/// The slices of a pie, as their tags.
fn slices(svg: &str) -> Vec<&str> {
    svg.split("<path")
        .skip(1)
        .filter(|t| t.contains("data-series="))
        .collect()
}

fn slice_attr<'a>(tag: &'a str, name: &str) -> &'a str {
    let key = format!(" {name}=\"");
    let start = tag.find(&key).unwrap() + key.len();
    &tag[start..start + tag[start..].find('"').unwrap()]
}

/// The radii of the arcs of a slice: its inner edge, and its outer one. The
/// corners are arcs as well, of no radius when they are not rounded.
fn slice_radii(tag: &str) -> Vec<f32> {
    let mut radii: Vec<f32> = slice_attr(tag, "d")
        .split(' ')
        .filter_map(|token| token.strip_prefix('A')?.parse().ok())
        .filter(|radius| *radius > 0.0)
        .collect();
    radii.sort_by(f32::total_cmp);
    radii.dedup();
    radii
}

#[test]
fn pie_nested_snapshot() {
    let chart = PieChart::from_json(include_str!("../asset/pie_chart/nested.json")).unwrap();
    assert_eq!(1, chart.series_list[3].ring);
    common::assert_snapshot!("pie_chart/nested_json.svg", chart.svg().unwrap());
}

#[test]
fn pie_rings() {
    let json = |rings: [usize; 5], extra: &str| {
        format!(
            r##"{{"rose_type": false, "radius": 100, "inner_radius": 20, "border_radius": 0{extra},
                "series_list": [
                    {{"name": "in a", "data": [30], "ring": {}}},
                    {{"name": "in b", "data": [10], "ring": {}}},
                    {{"name": "out a", "data": [5], "ring": {}}},
                    {{"name": "out b", "data": [5], "ring": {}}},
                    {{"name": "out c", "data": [10], "ring": {}}}
                ]}}"##,
            rings[0], rings[1], rings[2], rings[3], rings[4]
        )
    };
    let nested = PieChart::from_json(&json([0, 0, 1, 1, 1], ""))
        .unwrap()
        .svg()
        .unwrap();
    let slices_of = slices(&nested);
    assert_eq!(5, slices_of.len());

    // Each ring shares the turn among its own slices.
    let shares: Vec<&str> = slices_of
        .iter()
        .map(|s| slice_attr(s, "data-percentage"))
        .collect();
    assert_eq!(vec!["75", "25", "25", "25", "50"], shares);
    assert_eq!(
        ("0", "1"),
        (
            slice_attr(slices_of[0], "data-ring"),
            slice_attr(slices_of[2], "data-ring")
        )
    );
    // The rings split the room between the hole and the edge, with a gap:
    // 80 pixels for two rings of 37 and 6 between them.
    assert_eq!(vec![20.0, 57.0], slice_radii(slices_of[0]));
    assert_eq!(vec![63.0, 100.0], slice_radii(slices_of[2]));
    assert_eq!(slice_radii(slices_of[0]), slice_radii(slices_of[1]));

    // The slices of the inner ring are named on them, by their name; the
    // outer ring keeps its labels and their lines.
    let texts: Vec<&str> = nested
        .lines()
        .map(str::trim)
        .filter(|l| !l.is_empty() && !l.starts_with('<'))
        .collect();
    assert!(texts.contains(&"in a"), "{texts:?}");
    assert!(texts.contains(&"out c: 50%"), "{texts:?}");
    assert!(!texts.iter().any(|t| t.starts_with("in a:")), "{texts:?}");
    // A format of one's own is used on every ring.
    let formatted = PieChart::from_json(&json(
        [0, 0, 1, 1, 1],
        r#", "series_label_formatter": "{c}""#,
    ))
    .unwrap()
    .svg()
    .unwrap();
    assert!(formatted.lines().any(|l| l.trim() == "30"));
    assert!(texts.contains(&"in b"), "{texts:?}");
    // A name wider than its slice is left out rather than written across it.
    let thin = PieChart::from_json(
        r##"{"rose_type": false, "radius": 100, "series_list": [
            {"name": "wide", "data": [97], "ring": 0},
            {"name": "thin", "data": [3], "ring": 0},
            {"name": "out", "data": [1], "ring": 1}
        ]}"##,
    )
    .unwrap()
    .svg()
    .unwrap();
    let lines: Vec<&str> = thin.lines().map(str::trim).collect();
    assert!(lines.contains(&"wide") && !lines.contains(&"thin"));
    assert_eq!(3, slices(&thin).len());

    // The numbers of the rings only tell their order: 3 and 9 are 0 and 1.
    let renumbered = PieChart::from_json(&json([3, 3, 9, 9, 9], ""))
        .unwrap()
        .svg()
        .unwrap();
    assert_eq!(
        nested
            .replace("data-ring=\"0\"", "data-ring=\"3\"")
            .replace("data-ring=\"1\"", "data-ring=\"9\""),
        renumbered
    );
    // Three rings.
    let three = PieChart::from_json(&json([0, 1, 2, 2, 2], ""))
        .unwrap()
        .svg()
        .unwrap();
    let slices_of = slices(&three);
    assert_eq!("100", slice_attr(slices_of[0], "data-percentage"));
    assert_eq!("100", slice_attr(slices_of[1], "data-percentage"));
    let edges: Vec<Vec<f32>> = slices_of.iter().map(|s| slice_radii(s)).collect();
    assert!(
        edges[0][1] < edges[1][0] && edges[1][1] < edges[2][0],
        "{edges:?}"
    );
    assert!((edges[2][1] - 100.0).abs() < 0.1);

    // One ring is the pie as it always was, whatever its number.
    let plain = PieChart::from_json(&json([0, 0, 0, 0, 0], ""))
        .unwrap()
        .svg()
        .unwrap();
    let moved = PieChart::from_json(&json([4, 4, 4, 4, 4], ""))
        .unwrap()
        .svg()
        .unwrap();
    assert_eq!(plain, moved);
    assert!(!plain.contains("data-ring"));

    // A rose in each ring, half a turn, a ring of nothing: all are drawn.
    for extra in [
        r#", "rose_type": true"#,
        r#", "start_angle": -90, "end_angle": 90"#,
        r#", "inner_radius": 500"#,
    ] {
        let svg = PieChart::from_json(&json([0, 0, 1, 1, 1], extra))
            .unwrap()
            .svg()
            .unwrap();
        assert!(!svg.contains("NaN") && !svg.contains("inf"), "{extra}");
        assert_eq!(5, slices(&svg).len(), "{extra}");
    }

    let message =
        match PieChart::from_json(r#"{"series_list": [{"name": "a", "data": [1], "ring": -1}]}"#) {
            Ok(_) => panic!("accepted"),
            Err(e) => e.to_string(),
        };
    assert!(message.contains("series_list[0].ring"), "{message}");
}
