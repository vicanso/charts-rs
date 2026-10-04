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
