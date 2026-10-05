//! Chord diagrams: nodes around a circle, ribbons across it.
mod common;

use charts_rs::{ChordChart, ChordLink, ChordNode, Error, MultiChart, compact_svg};

fn attr<'a>(tag: &'a str, name: &str) -> &'a str {
    let key = format!(" {name}=\"");
    let start = tag.find(&key).map(|i| i + key.len()).unwrap();
    let end = start + tag[start..].find('"').unwrap();
    &tag[start..end]
}

fn point(text: &str) -> (f32, f32) {
    let (x, y) = text.split_once(',').unwrap();
    (x.parse().unwrap(), y.parse().unwrap())
}

/// A path of arcs, read back from its data: the point it starts at, the
/// point every arc and every curve ends at, and the radius of every arc.
#[derive(Debug)]
struct Outline {
    points: Vec<(f32, f32)>,
    radii: Vec<f32>,
    curves: usize,
}

fn outline(tag: &str) -> Outline {
    let tokens: Vec<&str> = attr(tag, "d").split(' ').collect();
    let mut outline = Outline {
        points: vec![point(&tokens[0][1..])],
        radii: vec![],
        curves: 0,
    };
    let mut i = 1;
    while i < tokens.len() {
        if let Some(radius) = tokens[i].strip_prefix('A') {
            // A{r} {r} 0 {large} {sweep} {x},{y}
            outline.radii.push(radius.parse().unwrap());
            outline.points.push(point(tokens[i + 5]));
            i += 6;
        } else if tokens[i].starts_with('Q') {
            // Q{cx},{cy} {x},{y}
            outline.curves += 1;
            outline.points.push(point(tokens[i + 1]));
            i += 2;
        } else {
            i += 1;
        }
    }
    outline
}

/// The arcs of the nodes, in drawing order.
fn nodes(svg: &str) -> Vec<&str> {
    svg.split('<')
        .filter(|t| t.starts_with("path") && t.contains("data-name="))
        .collect()
}

/// The ribbons of the links, in drawing order.
fn ribbons(svg: &str) -> Vec<&str> {
    svg.split('<')
        .filter(|t| t.starts_with("path") && t.contains("data-source="))
        .collect()
}

/// The angle of `point` seen from the center of the chart, clockwise from
/// 12 o'clock.
fn angle(point: (f32, f32)) -> f32 {
    let degrees = (point.0 - CENTER.0).atan2(CENTER.1 - point.1).to_degrees();
    (degrees + 360.0) % 360.0
}

/// The text nodes of the svg.
fn texts(svg: &str) -> Vec<&str> {
    svg.lines()
        .map(str::trim)
        .filter(|l| !l.is_empty() && !l.starts_with('<'))
        .collect()
}

fn close(a: f32, b: f32, tolerance: f32) -> bool {
    (a - b).abs() <= tolerance
}

fn err(result: Result<impl Sized, Error>) -> String {
    match result {
        Ok(_) => panic!("expected an error"),
        Err(e) => e.to_string(),
    }
}

/// The center of the charts of [`chart`].
const CENTER: (f32, f32) = (200.0, 200.0);

fn triangle() -> Vec<ChordLink> {
    vec![
        ("A", "B", 10.0).into(),
        ("A", "C", 30.0).into(),
        ("B", "C", 20.0).into(),
    ]
}

/// A square chart without a header, and without gaps between its nodes.
fn chart(links: Vec<ChordLink>) -> ChordChart {
    let mut chart = ChordChart::new(vec![], links);
    chart.width = 400.0;
    chart.height = 400.0;
    chart.node_gap = 0.0;
    chart
}

#[test]
fn chord_snapshots() {
    let basic = ChordChart::from_json(include_str!("../asset/chord_chart/basic.json")).unwrap();
    common::assert_snapshot!("chord_chart/basic_json.svg", basic.svg().unwrap());
    let gradient =
        ChordChart::from_json(include_str!("../asset/chord_chart/gradient.json")).unwrap();
    common::assert_snapshot!("chord_chart/gradient_json.svg", gradient.svg().unwrap());
}

#[test]
fn nodes_are_as_long_as_their_flows() {
    let svg = chart(triangle()).svg().unwrap();
    let nodes = nodes(&svg);
    assert_eq!(3, nodes.len());
    // Every link counts at both of its nodes.
    let named: Vec<(&str, &str)> = nodes
        .iter()
        .map(|n| (attr(n, "data-name"), attr(n, "data-value")))
        .collect();
    assert_eq!(vec![("A", "40"), ("B", "30"), ("C", "50")], named);
    // In the colors of the theme, clockwise from 12 o'clock.
    let fills: Vec<&str> = nodes.iter().map(|n| attr(n, "fill")).collect();
    assert_eq!(vec!["#5470C6", "#91CC75", "#FAC858"], fills);
    let outlines: Vec<Outline> = nodes.iter().map(|n| outline(n)).collect();
    for (outline, (from, to)) in outlines
        .iter()
        .zip([(0.0, 120.0), (120.0, 210.0), (210.0, 0.0)])
    {
        assert!(close(angle(outline.points[0]), from, 0.2), "{outline:?}");
        assert!(close(angle(outline.points[1]), to, 0.2), "{outline:?}");
        // A ring 12 pixels thick.
        assert!(close(outline.radii[0] - outline.radii[1], 12.0, 0.1));
    }
    let r = outlines[0].radii[0];
    assert!(r > 150.0 && r < 200.0, "{r}");

    // The names stand around the ring.
    assert_eq!(vec!["A", "B", "C"], texts(&svg));

    // With a gap after every node there is less of the circle to share.
    let mut spaced = chart(triangle());
    spaced.node_gap = 12.0;
    let svg = spaced.svg().unwrap();
    let first = outline(self::nodes(&svg)[0]);
    assert!(close(angle(first.points[0]), 6.0, 0.2));
    assert!(close(angle(first.points[1]), 6.0 + 108.0, 0.2));
}

#[test]
fn ribbons_join_the_ends_of_their_link() {
    let svg = chart(triangle()).svg().unwrap();
    let ribbons = ribbons(&svg);
    assert_eq!(3, ribbons.len());
    assert_eq!(
        ("A", "C", "30"),
        (
            attr(ribbons[1], "data-source"),
            attr(ribbons[1], "data-target"),
            attr(ribbons[1], "data-value")
        )
    );
    // In the color of the source node, and half transparent.
    assert_eq!("#5470C6", attr(ribbons[1], "fill"));
    assert_eq!("0.5", attr(ribbons[1], "fill-opacity"));
    assert_eq!("#91CC75", attr(ribbons[2], "fill"));

    // (source start, source end, target start, target end) of every ribbon:
    // as wide as the value at both ends, and at the end of each node that
    // is next to the other one.
    let expected = [
        (90.0, 120.0, 120.0, 150.0),
        (0.0, 90.0, 270.0, 0.0),
        (150.0, 210.0, 210.0, 270.0),
    ];
    let node = outline(nodes(&svg)[0]);
    for (ribbon, ends) in ribbons.iter().zip(expected) {
        let outline = outline(ribbon);
        // Two arcs, joined by two curves through the center; back at the start.
        assert_eq!((2, 2), (outline.radii.len(), outline.curves));
        assert_eq!(outline.points[0], outline.points[4]);
        let angles: Vec<f32> = outline.points[..4].iter().map(|p| angle(*p)).collect();
        for (found, wanted) in angles.iter().zip([ends.0, ends.1, ends.2, ends.3]) {
            assert!(close(*found, wanted, 0.3), "{angles:?} for {ends:?}");
        }
        // Just inside the ring of the nodes.
        assert!(close(outline.radii[0], node.radii[1] - 2.0, 0.1));
        assert!(attr(ribbon, "d").contains(" Q200,200 "));
    }

    // The ribbons lie under the ring.
    assert!(svg.rfind("data-source=").unwrap() < svg.find("data-name=").unwrap());
}

#[test]
fn nodes_can_be_given() {
    let mut chart = chart(triangle());
    chart.nodes = vec![
        ChordNode {
            name: "C".to_string(),
            color: Some("#ff0000".into()),
        },
        "A".into(),
        // Listed twice, counted once; without links, not drawn.
        "A".into(),
        "D".into(),
    ];
    let svg = chart.svg().unwrap();
    let nodes = nodes(&svg);
    // The given nodes first and in their order, then the ones of the links.
    let names: Vec<&str> = nodes.iter().map(|n| attr(n, "data-name")).collect();
    assert_eq!(vec!["C", "A", "B"], names);
    // A color of its own, or the color of its place in the list.
    let fills: Vec<&str> = nodes.iter().map(|n| attr(n, "fill")).collect();
    assert_eq!(vec!["#FF0000", "#91CC75", "#EE6666"], fills);
    assert_eq!(vec!["C", "A", "B"], texts(&svg));
    assert!(close(angle(outline(nodes[0]).points[1]), 150.0, 0.2));
}

#[test]
fn links_without_a_flow_are_left_out() {
    let mut links = triangle();
    links.push(("A", "D", 0.0).into());
    links.push(("B", "E", -5.0).into());
    links.push(("C", "F", f32::NAN).into());
    links.push(("C", "G", f32::INFINITY).into());
    let odd = chart(links).svg().unwrap();
    assert_eq!(chart(triangle()).svg().unwrap(), odd);

    // Nothing left to draw.
    let mut empty = chart(vec![("A", "B", 0.0).into()]);
    empty.nodes = vec!["A".into(), "B".into()];
    empty.empty_text = Some("No data".to_string());
    let svg = empty.svg().unwrap();
    assert_eq!(vec!["No data"], texts(&svg));
    assert!(nodes(&svg).is_empty() && ribbons(&svg).is_empty());
    assert!(
        ChordChart::new(vec![], vec![])
            .svg()
            .unwrap()
            .starts_with("<svg")
    );
}

#[test]
fn link_from_a_node_to_itself() {
    let svg = chart(vec![("A", "A", 10.0).into(), ("A", "B", 30.0).into()])
        .svg()
        .unwrap();
    // It has one end, so it counts once: 40 + 30.
    let nodes = nodes(&svg);
    assert_eq!("40", attr(nodes[0], "data-value"));
    assert!(close(
        angle(outline(nodes[0]).points[1]),
        360.0 * 40.0 / 70.0,
        0.3
    ));
    // The ribbon leaves its arc and comes back to it.
    let own = outline(ribbons(&svg)[0]);
    assert_eq!((1, 1), (own.radii.len(), own.curves));
    assert_eq!(own.points[0], own.points[2]);
    assert_eq!(
        ("A", "A"),
        (
            attr(ribbons(&svg)[0], "data-source"),
            attr(ribbons(&svg)[0], "data-target")
        )
    );

    // A node with nothing but itself takes the whole circle.
    let mut alone = chart(vec![("A", "A", 10.0).into()]);
    alone.node_gap = 30.0;
    let svg = alone.svg().unwrap();
    assert!(!svg.contains("NaN"));
    let ring = self::nodes(&svg);
    assert_eq!(1, ring.len());
    assert_eq!(2, attr(ring[0], "d").matches('Z').count());
}

#[test]
fn links_between_the_same_two_nodes() {
    let svg = chart(vec![("A", "B", 10.0).into(), ("B", "A", 30.0).into()])
        .svg()
        .unwrap();
    let ribbons = ribbons(&svg);
    assert_eq!(2, ribbons.len());
    // Each in the color of its own source.
    assert_eq!("#5470C6", attr(ribbons[0], "fill"));
    assert_eq!("#91CC75", attr(ribbons[1], "fill"));
    // A and B have half the circle each; the two links lie in opposite
    // orders on them, so the ribbons run side by side.
    let (first, second) = (outline(ribbons[0]), outline(ribbons[1]));
    let angles = |o: &Outline| -> Vec<f32> { o.points[..4].iter().map(|p| angle(*p)).collect() };
    for (found, wanted) in angles(&first).iter().zip([0.0, 45.0, 315.0, 0.0]) {
        assert!(close(*found, wanted, 0.3), "{:?}", angles(&first));
    }
    for (found, wanted) in angles(&second).iter().zip([180.0, 315.0, 45.0, 180.0]) {
        assert!(close(*found, wanted, 0.3), "{:?}", angles(&second));
    }
}

#[test]
fn tooltips_and_labels() {
    let mut chart = chart(triangle());
    chart.tooltip.show = true;
    chart.series.label.formatter = "{b}: {c} ({d})".to_string();
    let svg = chart.svg().unwrap();
    assert!(svg.contains(".ct-trigger:hover+.ct-tip"));
    assert_eq!(6, svg.matches(r#"class="ct-trigger""#).count());
    assert_eq!(6, svg.matches(r#"class="ct-tip""#).count());
    assert!(svg.contains("<title>A → C: 30</title>"));
    assert!(svg.contains("<title>C: 50</title>"));
    // The labels tell the flow through the node and its share.
    let texts = texts(&svg);
    assert!(texts.contains(&"A: 40 (33.3%)"), "{texts:?}");
    assert!(texts.contains(&"C: 50 (41.7%)"), "{texts:?}");
    // Longer labels leave less room for the ring.
    let plain = self::chart(triangle()).svg().unwrap();
    assert!(outline(nodes(&svg)[0]).radii[0] < outline(nodes(&plain)[0]).radii[0] - 30.0);
    assert!(!plain.contains("ct-t") && !plain.contains("<style>"));

    // A label that would run into its neighbour is left out.
    let links: Vec<ChordLink> = (0..60)
        .map(|i| ChordLink {
            source: "hub".to_string(),
            target: format!("node {i}"),
            value: 1.0,
        })
        .collect();
    let svg = self::chart(links).svg().unwrap();
    assert_eq!(61, nodes(&svg).len());
    let labels = self::texts(&svg).len();
    assert!(labels > 10 && labels < 50, "{labels}");
}

#[test]
fn ribbons_can_be_gradients() {
    let mut chart = chart(vec![
        ("A", "B", 10.0).into(),
        ("A", "C", 30.0).into(),
        ("A", "A", 5.0).into(),
    ]);
    chart.link_gradient = true;
    chart.link_opacity = 0.8;
    let svg = chart.svg().unwrap();
    let ribbons = ribbons(&svg);
    assert!(attr(ribbons[0], "fill").starts_with("url(#grad_5470C6CC_91CC75CC_"));
    assert!(attr(ribbons[1], "fill").starts_with("url(#grad_5470C6CC_FAC858CC_"));
    assert_eq!(2, svg.matches("<linearGradient ").count());
    assert_eq!(4, svg.matches(r#"stop-opacity="0.80""#).count());
    // From a node to itself there is nothing to fade to.
    assert_eq!("#5470C6", attr(ribbons[2], "fill"));
    assert_eq!("0.8", attr(ribbons[2], "fill-opacity"));

    // The opacity is kept between 0 and 1.
    chart.link_gradient = false;
    chart.link_opacity = 7.0;
    let svg = chart.svg().unwrap();
    assert!(!self::ribbons(&svg)[0].contains("fill-opacity"));
    chart.link_opacity = f32::NAN;
    let svg = chart.svg().unwrap();
    assert_eq!("0.5", attr(self::ribbons(&svg)[0], "fill-opacity"));
}

#[test]
fn size_and_angles_can_be_set() {
    let mut chart = chart(triangle());
    chart.radius = Some(100.0);
    chart.node_width = 30.0;
    chart.start_angle = 90.0;
    let svg = chart.svg().unwrap();
    let first = outline(nodes(&svg)[0]);
    assert_eq!(vec![100.0, 70.0], first.radii);
    assert!(close(angle(first.points[0]), 90.0, 0.2));
    assert!(close(angle(first.points[1]), 210.0, 0.2));
    assert_eq!(68.0, outline(ribbons(&svg)[0]).radii[0]);

    // The radius is a limit: a larger one than fits changes nothing.
    chart.radius = Some(5000.0);
    let limited = chart.svg().unwrap();
    chart.radius = None;
    assert_eq!(limited, chart.svg().unwrap());

    // Without a ring only the ribbons are left, out to the edge.
    chart.node_width = 0.0;
    let svg = chart.svg().unwrap();
    assert!(nodes(&svg).is_empty());
    assert_eq!(3, ribbons(&svg).len());
    assert_eq!(3, texts(&svg).len());
}

#[test]
fn odd_input_stays_finite() {
    for (gap, width, radius, start) in [
        (f32::NAN, f32::NAN, f32::NAN, f32::NAN),
        (-10.0, -10.0, -10.0, -720.0),
        (1e9, 1e9, 1e9, 1e9),
        (f32::INFINITY, f32::INFINITY, 0.5, f32::NEG_INFINITY),
    ] {
        let mut chart = chart(vec![
            ("A", "B", 1e-20).into(),
            ("B", "C", 1e20).into(),
            ("C", "C", 3.0).into(),
        ]);
        chart.node_gap = gap;
        chart.node_width = width;
        chart.radius = Some(radius);
        chart.start_angle = start;
        chart.tooltip.show = true;
        chart.link_gradient = true;
        let svg = chart.svg().unwrap();
        assert!(
            !svg.contains("NaN") && !svg.contains("inf"),
            "{gap} {width}"
        );
        assert_eq!(3, ribbons(&svg).len());
    }
    // A canvas too small for anything.
    let mut chart = chart(triangle());
    chart.width = 12.0;
    chart.height = 8.0;
    let svg = chart.svg().unwrap();
    assert!(!svg.contains("NaN") && !svg.contains("inf"));
}

#[test]
fn chord_from_json() {
    let json = r##"{
        "width": 400,
        "height": 400,
        "radius": 120,
        "node_width": 20,
        "node_gap": 6,
        "start_angle": 45,
        "link_opacity": 0.7,
        "link_gradient": true,
        "nodes": [{"name": "C", "color": "#123456"}, {"name": "A"}, {"name": ""}],
        "links": [
            {"source": "A", "target": "B", "value": 10},
            {"source": "A", "target": "C", "value": 30},
            {"source": "B", "target": "C", "value": 20},
            {"source": "", "target": "C", "value": 20},
            {"source": "B", "target": "C"}
        ]
    }"##;
    let from_json = ChordChart::from_json(json).unwrap();
    assert_eq!(2, from_json.nodes.len());
    // A link needs both of its names; without a value it has no flow.
    assert_eq!(4, from_json.links.len());
    assert_eq!(0.0, from_json.links[3].value);
    assert_eq!(
        (Some(120.0), 20.0, 6.0, 45.0, 0.7, true),
        (
            from_json.radius,
            from_json.node_width,
            from_json.node_gap,
            from_json.start_angle,
            from_json.link_opacity,
            from_json.link_gradient
        )
    );

    // The same chart, built in code.
    let mut built = ChordChart::new(
        vec![
            ChordNode {
                name: "C".to_string(),
                color: Some("#123456".into()),
            },
            "A".into(),
        ],
        triangle(),
    );
    built.width = 400.0;
    built.height = 400.0;
    built.radius = Some(120.0);
    built.node_width = 20.0;
    built.node_gap = 6.0;
    built.start_angle = 45.0;
    built.link_opacity = 0.7;
    built.link_gradient = true;
    assert_eq!(built.svg().unwrap(), from_json.svg().unwrap());

    // The defaults of the builder are the defaults of the JSON.
    let plain = ChordChart::from_json(r#"{"links": [{"source": "A", "target": "B", "value": 1}]}"#)
        .unwrap();
    let built = ChordChart::new(vec![], vec![("A", "B", 1.0).into()]);
    assert_eq!(
        (12.0, 3.0, 0.5),
        (plain.node_width, plain.node_gap, plain.link_opacity)
    );
    assert_eq!(built.svg().unwrap(), plain.svg().unwrap());

    let message = err(ChordChart::from_json(r#"{"link": []}"#));
    assert!(message.contains("links"), "{message}");
    let message = err(ChordChart::from_json(r#"{"links": [{"from": "A"}]}"#));
    assert!(message.contains("from"), "{message}");
    let message = err(ChordChart::from_json(r#"{"radius": -1}"#));
    assert!(message.contains("radius"), "{message}");
    let message = err(ChordChart::from_json(
        r#"{"nodes": [{"name": "A", "color": "red?"}]}"#,
    ));
    assert!(message.contains("color"), "{message}");
}

#[test]
fn chord_in_a_multi_chart() {
    let json = r##"{
        "child_charts": [
            {"type": "chord", "links": [{"source": "A", "target": "B", "value": 1}]},
            {"type": "bar", "x_axis_data": ["a", "b"], "series_list": [{"name": "A", "data": [1, 2]}]}
        ]
    }"##;
    let svg = MultiChart::from_json(json).unwrap().svg().unwrap();
    assert_eq!((2, 1), (nodes(&svg).len(), ribbons(&svg).len()));
    let message = err(MultiChart::from_json(
        r#"{"child_charts": [{"type": "chord", "node_align": "left"}]}"#,
    ));
    assert!(message.contains("node_align"), "{message}");
}

#[test]
fn animation_and_compact_output() {
    let json = include_str!("../asset/chord_chart/gradient.json");
    let mut chart = ChordChart::from_json(json).unwrap();
    let plain = chart.svg().unwrap();
    assert!(!plain.contains("chord-anim"));
    chart.animation = Some(Default::default());
    let animated = chart.svg().unwrap();
    assert!(
        animated.contains("@keyframes chord-grow") && animated.contains("@keyframes chord-fade")
    );
    // The nodes and the ribbons grow from the center, the labels fade in.
    assert_eq!(
        19,
        animated.matches(r#"class="chord-anim ct-trigger""#).count()
    );
    assert_eq!(7, animated.matches(r#"class="chord-fade""#).count());
    assert!(animated.contains("transform-origin:"));

    // The compact form keeps every shape and what it says.
    let compact = compact_svg(&plain);
    assert!(compact.len() < plain.len());
    for (needle, count) in [
        ("data-name=", 7),
        ("data-source=", 12),
        ("<title>", 19),
        ("class=\"ct-trigger\"", 19),
        (
            "<linearGradient ",
            plain.matches("<linearGradient ").count(),
        ),
    ] {
        assert_eq!(count, compact.matches(needle).count(), "{needle}");
    }
}
