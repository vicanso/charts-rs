//! Polar bar charts: bars that grow outwards, and bars that run around.
mod common;

use charts_rs::{Error, MultiChart, PolarAxis, PolarBarChart, Series, compact_svg};

fn attr<'a>(tag: &'a str, name: &str) -> &'a str {
    let key = format!(" {name}=\"");
    let start = tag.find(&key).map(|i| i + key.len()).unwrap();
    let end = start + tag[start..].find('"').unwrap();
    &tag[start..end]
}

fn number(tag: &str, name: &str) -> f32 {
    attr(tag, name).parse().unwrap()
}

/// The outline of a bar, read back from its path data.
#[derive(Debug)]
struct Shape {
    /// Where the outer edge starts and ends.
    start: (f32, f32),
    end: (f32, f32),
    /// Radius of the outer and of the inner edge (0 for a wedge).
    outer: f32,
    inner: f32,
    /// Whether the outer edge takes the long way round.
    large: bool,
    /// Number of arcs: 2 for a plain bar, 4 with round caps or a full ring.
    arcs: usize,
    /// The radius of every arc, in drawing order.
    radii: Vec<f32>,
    /// Number of closed outlines: 2 for a full ring with a hole.
    outlines: usize,
}

fn point(text: &str) -> (f32, f32) {
    let (x, y) = text.split_once(',').unwrap();
    (x.parse().unwrap(), y.parse().unwrap())
}

fn shape(tag: &str) -> Shape {
    let d = attr(tag, "d");
    let tokens: Vec<&str> = d.split(' ').collect();
    let mut shape = Shape {
        start: point(&tokens[0][1..]),
        end: (0.0, 0.0),
        outer: 0.0,
        inner: 0.0,
        large: false,
        arcs: 0,
        radii: vec![],
        outlines: d.matches('Z').count(),
    };
    let mut i = 1;
    while i < tokens.len() {
        if let Some(radius) = tokens[i].strip_prefix('A') {
            let radius: f32 = radius.parse().unwrap();
            // A{r} {r} 0 {large} {sweep} {x},{y}
            let (large, sweep) = (tokens[i + 3] == "1", tokens[i + 4] == "1");
            if shape.arcs == 0 {
                shape.outer = radius;
                shape.large = large;
                shape.end = point(tokens[i + 5]);
            } else if !sweep {
                shape.inner = radius;
            }
            shape.arcs += 1;
            shape.radii.push(radius);
            i += 6;
        } else {
            i += 1;
        }
    }
    shape
}

/// The bars of the chart, in drawing order.
fn bars(svg: &str) -> Vec<&str> {
    svg.split('<')
        .filter(|t| t.starts_with("path") && t.contains("data-series="))
        .collect()
}

/// The rings of the grid as `(cx, cy, r)`; the legend has to be hidden.
fn rings(svg: &str) -> Vec<(f32, f32, f32)> {
    svg.split('<')
        .filter(|t| t.starts_with("circle"))
        .map(|t| (number(t, "cx"), number(t, "cy"), number(t, "r")))
        .collect()
}

/// The angle of `point` seen from `center`, clockwise from 12 o'clock.
fn angle(center: (f32, f32), point: (f32, f32)) -> f32 {
    let degrees = (point.0 - center.0).atan2(center.1 - point.1).to_degrees();
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

fn names(list: &[&str]) -> Vec<String> {
    list.iter().map(|s| s.to_string()).collect()
}

/// A square chart without a header, so the plot is centered on the canvas;
/// the value axis ends at 40.
fn chart(series_list: Vec<Series>, categories: &[&str]) -> PolarBarChart {
    let mut chart = PolarBarChart::new(series_list, names(categories));
    chart.width = 400.0;
    chart.height = 400.0;
    chart.legend_show = Some(false);
    chart.y_axis_configs[0].axis_max = Some(40.0);
    chart.y_axis_configs[0].axis_split_number = 4;
    chart
}

#[test]
fn polar_bar_snapshots() {
    let basic =
        PolarBarChart::from_json(include_str!("../asset/polar_bar_chart/basic.json")).unwrap();
    common::assert_snapshot!("polar_bar_chart/basic_json.svg", basic.svg().unwrap());
    let stack =
        PolarBarChart::from_json(include_str!("../asset/polar_bar_chart/stack.json")).unwrap();
    common::assert_snapshot!("polar_bar_chart/stack_json.svg", stack.svg().unwrap());
    let radial =
        PolarBarChart::from_json(include_str!("../asset/polar_bar_chart/radial.json")).unwrap();
    common::assert_snapshot!("polar_bar_chart/radial_json.svg", radial.svg().unwrap());
    let group =
        PolarBarChart::from_json(include_str!("../asset/polar_bar_chart/radial_group.json"))
            .unwrap();
    common::assert_snapshot!(
        "polar_bar_chart/radial_group_json.svg",
        group.svg().unwrap()
    );
}

#[test]
fn bars_grow_outwards_with_their_value() {
    let chart = chart(
        vec![("A", vec![10.0, 20.0, 40.0, 0.0]).into()],
        &["a", "b", "c", "d"],
    );
    let svg = chart.svg().unwrap();

    // One ring per tick of the value axis, the largest one the plot itself.
    let rings = rings(&svg);
    assert_eq!(4, rings.len());
    let (cx, cy, r) = rings[3];
    assert_eq!((200.0, 200.0), (cx, cy));
    assert!(r > 150.0 && r < 200.0, "{r}");
    assert!(close(rings[0].2, r / 4.0, 0.1));

    // A zero has no bar.
    let bars = bars(&svg);
    assert_eq!(3, bars.len());
    let shapes: Vec<Shape> = bars.iter().map(|b| shape(b)).collect();
    for (shape, share) in shapes.iter().zip([0.25, 0.5, 1.0]) {
        assert!(close(shape.outer, r * share, 0.1), "{shape:?}");
        // A wedge from the center.
        assert_eq!(0.0, shape.inner);
        assert_eq!((1, 1), (shape.arcs, shape.outlines));
    }

    // Every category has a quarter of the circle; a bar fills 80% of it.
    for (i, shape) in shapes.iter().enumerate() {
        let from = angle((cx, cy), shape.start);
        let to = angle((cx, cy), shape.end);
        assert!(close(from, 90.0 * i as f32 + 9.0, 0.3), "{i}: {from}");
        assert!(close(to - from, 72.0, 0.3), "{i}: {from}..{to}");
    }
    assert_eq!("a", attr(bars[0], "data-category"));
    assert_eq!("40", attr(bars[2], "data-value"));

    // The categories are named around the plot, the ticks along the ray.
    let texts = texts(&svg);
    for label in ["a", "b", "c", "d", "10", "20", "30", "40"] {
        assert!(texts.contains(&label), "{label} in {texts:?}");
    }
}

#[test]
fn series_sit_side_by_side_or_stacked() {
    let series = || -> Vec<Series> {
        vec![
            ("A", vec![10.0, 20.0]).into(),
            ("B", vec![20.0, 10.0]).into(),
        ]
    };
    let side_by_side = chart(series(), &["a", "b"]);
    let svg = side_by_side.svg().unwrap();
    let (cx, cy, r) = *rings(&svg).last().unwrap();
    let shapes: Vec<Shape> = bars(&svg).iter().map(|b| shape(b)).collect();
    assert_eq!(4, shapes.len());
    // Drawn series by series: A in both categories, then B.
    let starts: Vec<f32> = shapes.iter().map(|s| angle((cx, cy), s.start)).collect();
    for (start, expected) in starts.iter().zip([18.0, 198.0, 90.0, 270.0]) {
        assert!(close(*start, expected, 0.3), "{starts:?}");
    }
    assert!(close(shapes[0].outer, r / 4.0, 0.1));
    assert!(close(shapes[2].outer, r / 2.0, 0.1));

    let mut list = series();
    for series in list.iter_mut() {
        series.stack = Some("total".to_string());
    }
    let stacked = chart(list, &["a", "b"]);
    let svg = stacked.svg().unwrap();
    let shapes: Vec<Shape> = bars(&svg).iter().map(|b| shape(b)).collect();
    // B starts where A ends, in the same slice of the circle.
    assert!(close(shapes[2].inner, shapes[0].outer, 0.1));
    assert!(close(shapes[2].outer, r * 0.75, 0.1));
    assert_eq!(2, shapes[2].arcs);
    let (a, b) = (
        angle((cx, cy), shapes[0].start),
        angle((cx, cy), shapes[2].start),
    );
    assert!(close(a, b, 0.3), "{a} {b}");
}

#[test]
fn hole_in_the_center() {
    let mut chart = chart(vec![("A", vec![20.0, 40.0]).into()], &["a", "b"]);
    chart.inner_radius = Some(50.0);
    let svg = chart.svg().unwrap();
    let rings = rings(&svg);
    // The hole is the first ring: the value axis starts there.
    assert_eq!(5, rings.len());
    assert_eq!(50.0, rings[0].2);
    let r = rings[4].2;
    let shapes: Vec<Shape> = bars(&svg).iter().map(|b| shape(b)).collect();
    assert_eq!(50.0, shapes[0].inner);
    assert!(close(shapes[0].outer, 50.0 + (r - 50.0) / 2.0, 0.1));
    assert!(close(shapes[1].outer, r, 0.1));

    // A hole larger than the plot leaves a sliver of it.
    chart.inner_radius = Some(5000.0);
    let svg = chart.svg().unwrap();
    assert!(!svg.contains("NaN") && !svg.contains("inf"));
    assert_eq!(2, bars(&svg).len());
}

#[test]
fn bars_run_around_the_circle() {
    let mut chart = chart(
        vec![("A", vec![10.0, 20.0, 40.0]).into()],
        &["inner", "middle", "outer"],
    );
    chart.category_axis = PolarAxis::Radius;
    let svg = chart.svg().unwrap();

    // The hole and the edge of the plot.
    let rings = rings(&svg);
    assert_eq!(2, rings.len());
    let ((cx, cy, hole), (_, _, r)) = (rings[0], rings[1]);
    assert!(close(hole, r / 4.0, 0.1));

    let shapes: Vec<Shape> = bars(&svg).iter().map(|b| shape(b)).collect();
    assert_eq!(3, shapes.len());
    let band = (r - hole) / 3.0;
    for (i, (shape, sweep)) in shapes.iter().zip([67.5, 135.0, 270.0]).enumerate() {
        // Three quarters of the circle for the largest value on the axis.
        assert!(close(angle((cx, cy), shape.start), 0.0, 0.3), "{shape:?}");
        assert!(close(angle((cx, cy), shape.end), sweep, 0.3), "{shape:?}");
        assert_eq!(sweep > 180.0, shape.large);
        // One ring per category, from the hole outwards.
        let inner = hole + band * (i as f32 + 0.1);
        assert!(close(shape.inner, inner, 0.1), "{shape:?}");
        assert!(close(shape.outer, inner + band * 0.8, 0.15), "{shape:?}");
        assert_eq!(2, shape.arcs);
    }

    // The ticks go around the plot, the categories sit where the bars start.
    let texts = texts(&svg);
    for label in ["inner", "middle", "outer", "0", "10", "20", "30", "40"] {
        assert!(texts.contains(&label), "{label} in {texts:?}");
    }
    // The category labels end left of the start ray.
    let label = svg
        .split("<text")
        .find(|t| t.contains("\nmiddle\n"))
        .unwrap();
    assert_eq!("end", attr(label, "text-anchor"));
    assert!(number(label, "x") < cx);
}

#[test]
fn angles_can_be_set() {
    let mut chart = chart(vec![("A", vec![20.0, 40.0]).into()], &["a", "b"]);
    chart.category_axis = PolarAxis::Radius;
    chart.start_angle = 90.0;
    chart.end_angle = Some(270.0);
    let svg = chart.svg().unwrap();
    let (cx, cy, _) = rings(&svg)[0];
    let shapes: Vec<Shape> = bars(&svg).iter().map(|b| shape(b)).collect();
    assert!(close(angle((cx, cy), shapes[0].start), 90.0, 0.3));
    assert!(close(angle((cx, cy), shapes[0].end), 180.0, 0.3));
    assert!(close(angle((cx, cy), shapes[1].end), 270.0, 0.3));

    // A full turn: the largest value closes its ring, as two outlines (the
    // ring and its hole) of two half circles each.
    chart.start_angle = 0.0;
    chart.end_angle = Some(360.0);
    let svg = chart.svg().unwrap();
    let shapes: Vec<Shape> = bars(&svg).iter().map(|b| shape(b)).collect();
    assert!(close(angle((cx, cy), shapes[0].end), 180.0, 0.3));
    assert_eq!((4, 2), (shapes[1].arcs, shapes[1].outlines));
    // The last tick would land on the first: it is not named twice.
    let texts = texts(&svg);
    assert!(texts.contains(&"0") && !texts.contains(&"40"), "{texts:?}");
    // More than a full turn is a full turn.
    chart.end_angle = Some(1000.0);
    assert_eq!(svg, chart.svg().unwrap());

    // An end before the start is ignored.
    chart.end_angle = Some(-90.0);
    let svg = chart.svg().unwrap();
    let shapes: Vec<Shape> = bars(&svg).iter().map(|b| shape(b)).collect();
    assert!(close(angle((cx, cy), shapes[1].end), 270.0, 0.3));

    // With the categories around the circle, the start angle turns them.
    let mut chart = chart_turned();
    let svg = chart.svg().unwrap();
    let (cx, cy, _) = rings(&svg)[0];
    let from = angle((cx, cy), shape(bars(&svg)[0]).start);
    assert!(close(from, 45.0 + 18.0, 0.3), "{from}");
    // The end angle belongs to the other layout.
    chart.end_angle = Some(180.0);
    assert_eq!(svg, chart.svg().unwrap());
}

fn chart_turned() -> PolarBarChart {
    let mut chart = chart(vec![("A", vec![20.0, 40.0]).into()], &["a", "b"]);
    chart.start_angle = 45.0;
    chart
}

#[test]
fn round_caps() {
    let mut chart = chart(vec![("A", vec![20.0, 30.0]).into()], &["a", "b"]);
    chart.category_axis = PolarAxis::Radius;
    let plain = chart.svg().unwrap();
    chart.round_cap = true;
    let round = chart.svg().unwrap();
    let (plain_shape, round_shape) = (shape(bars(&plain)[0]), shape(bars(&round)[0]));
    assert_eq!(2, plain_shape.arcs);
    // A half circle at each end, as wide as the bar.
    assert_eq!(4, round_shape.arcs);
    let cap = (round_shape.outer - round_shape.inner) / 2.0;
    assert!(close(round_shape.radii[1], cap, 0.1), "{round_shape:?}");
    assert!(close(round_shape.radii[3], cap, 0.1), "{round_shape:?}");
    assert_eq!(plain_shape.end, round_shape.end);

    // Bars that grow outwards have no ends to round.
    chart.category_axis = PolarAxis::Angle;
    let round = chart.svg().unwrap();
    chart.round_cap = false;
    assert_eq!(round, chart.svg().unwrap());
}

#[test]
fn negative_values_grow_the_other_way() {
    let mut chart = chart(vec![("A", vec![20.0, -20.0]).into()], &["a", "b"]);
    chart.y_axis_configs[0].axis_min = Some(-40.0);
    chart.y_axis_configs[0].axis_max = Some(40.0);
    let svg = chart.svg().unwrap();
    let r = rings(&svg).last().unwrap().2;
    let shapes: Vec<Shape> = bars(&svg).iter().map(|b| shape(b)).collect();
    // 0 is half way out: one bar goes on from there, the other comes back.
    assert!(close(shapes[0].inner, r / 2.0, 0.1) && close(shapes[0].outer, r * 0.75, 0.1));
    assert!(close(shapes[1].inner, r / 4.0, 0.1) && close(shapes[1].outer, r / 2.0, 0.1));

    chart.category_axis = PolarAxis::Radius;
    let svg = chart.svg().unwrap();
    let (cx, cy, _) = rings(&svg)[0];
    let shapes: Vec<Shape> = bars(&svg).iter().map(|b| shape(b)).collect();
    // 0 is at 135 of 270 degrees.
    let (from, to) = (
        angle((cx, cy), shapes[0].start),
        angle((cx, cy), shapes[0].end),
    );
    assert!(
        close(from, 135.0, 0.3) && close(to, 202.5, 0.3),
        "{from} {to}"
    );
    let (from, to) = (
        angle((cx, cy), shapes[1].start),
        angle((cx, cy), shapes[1].end),
    );
    assert!(
        close(from, 67.5, 0.3) && close(to, 135.0, 0.3),
        "{from} {to}"
    );
}

#[test]
fn tooltips_labels_and_colors() {
    let mut series: Series = ("Visits", vec![10.0, 20.0, 40.0]).into();
    series.label_show = true;
    series.colors = Some(vec![None, Some("#ff0000".into()), None]);
    let mut chart = chart(vec![series], &["a", "b", "c"]);
    chart.tooltip_show = true;
    chart.series_label_formatter = "{c} k".to_string();
    let svg = chart.svg().unwrap();

    let bars = bars(&svg);
    assert_eq!(3, bars.len());
    assert!(bars.iter().all(|b| attr(b, "class") == "ct-trigger"));
    assert_eq!("Visits", attr(bars[1], "data-series"));
    assert_eq!("b", attr(bars[1], "data-category"));
    assert_eq!("20", attr(bars[1], "data-value"));
    // A bar may have a color of its own.
    assert_eq!("#5470C6", attr(bars[0], "fill"));
    assert_eq!("#FF0000", attr(bars[1], "fill"));
    assert!(svg.contains("<title>Visits: 20 k</title>"));
    assert!(svg.contains(".ct-trigger:hover+.ct-tip"));
    // The hover label follows its bar; the data label is drawn once more.
    assert_eq!(3, svg.matches(r#"class="ct-tip""#).count());
    assert_eq!(1, texts(&svg).iter().filter(|t| **t == "20 k").count());
    assert_eq!(
        1,
        texts(&svg).iter().filter(|t| **t == "Visits: 20 k").count()
    );

    chart.tooltip_show = false;
    chart.series_list[0].label_show = false;
    let svg = chart.svg().unwrap();
    assert!(!svg.contains("ct-t") && !svg.contains("<title>") && !svg.contains("<style>"));
    assert!(!texts(&svg).contains(&"20 k"));
}

#[test]
fn data_labels_get_room_outside_the_bars() {
    let mut chart = chart(
        vec![("A", vec![40.0, 40.0, 40.0, 40.0]).into()],
        &["a", "b", "c", "d"],
    );
    let plain = chart.svg().unwrap();
    chart.series_list[0].label_show = true;
    let labelled = chart.svg().unwrap();
    // The plot shrinks to keep a ring free between it and the categories.
    let (plain_r, labelled_r) = (rings(&plain)[3].2, rings(&labelled)[3].2);
    assert!(labelled_r < plain_r - 10.0, "{plain_r} {labelled_r}");
    assert_eq!(
        4,
        texts(&labelled).iter().filter(|t| **t == "40").count() - 1
    );
}

#[test]
fn axis_labels_can_be_hidden() {
    let mut chart = chart(vec![("A", vec![10.0, 20.0]).into()], &["first", "second"]);
    let both = chart.svg().unwrap();
    assert!(texts(&both).contains(&"first") && texts(&both).contains(&"20"));
    // The ticks along the ray stand on a backdrop.
    assert!(both.matches("<rect").count() > 1);

    chart.x_axis_hidden = true;
    let svg = chart.svg().unwrap();
    assert!(!texts(&svg).contains(&"first") && texts(&svg).contains(&"20"));
    // Without labels around it, the plot is larger.
    assert!(rings(&svg)[3].2 > rings(&both)[3].2);

    chart.y_axis_hidden = true;
    let svg = chart.svg().unwrap();
    assert!(texts(&svg).is_empty(), "{:?}", texts(&svg));
    // Only the background is left of the rects.
    assert_eq!(1, svg.matches("<rect").count());

    // The same switches, the other way round.
    chart.category_axis = PolarAxis::Radius;
    chart.y_axis_hidden = false;
    let svg = chart.svg().unwrap();
    assert!(!texts(&svg).contains(&"first") && texts(&svg).contains(&"20"));
}

#[test]
fn formatter_and_axis_range() {
    let mut chart = chart(vec![("A", vec![10.0, 80.0]).into()], &["a", "b"]);
    chart.y_axis_configs[0].axis_formatter = Some("{c} mm".to_string());
    let svg = chart.svg().unwrap();
    assert!(texts(&svg).contains(&"40 mm"));
    // A value beyond the axis is cut off at its end.
    let r = rings(&svg).last().unwrap().2;
    assert!(close(shape(bars(&svg)[1]).outer, r, 0.1));

    // Without a fixed end the axis is rounded up from the data.
    chart.y_axis_configs[0].axis_max = None;
    chart.y_axis_configs[0].axis_split_number = 0;
    let svg = chart.svg().unwrap();
    let longest = shape(bars(&svg)[1]).outer;
    let r = rings(&svg).last().unwrap().2;
    assert!(longest < r && longest > r * 0.6, "{longest} of {r}");
}

#[test]
fn nothing_to_draw() {
    let mut chart = PolarBarChart::new(vec![], vec![]);
    chart.empty_text = Some("No data".to_string());
    let svg = chart.svg().unwrap();
    assert!(texts(&svg).contains(&"No data"));
    assert!(bars(&svg).is_empty() && !svg.contains("<circle"));

    // Without categories, the longest series tells how many there are.
    let chart = PolarBarChart::new(vec![("A", vec![1.0, 2.0, 3.0]).into()], vec![]);
    assert_eq!(3, bars(&chart.svg().unwrap()).len());
    // With them, a value beyond the last one has no place.
    let chart = PolarBarChart::new(vec![("A", vec![1.0, 2.0, 3.0]).into()], names(&["a", "b"]));
    assert_eq!(2, bars(&chart.svg().unwrap()).len());

    // Categories without values are as empty.
    let mut chart = PolarBarChart::new(vec![("A", Vec::<f32>::new()).into()], names(&["a", "b"]));
    chart.empty_text = Some("No data".to_string());
    assert!(texts(&chart.svg().unwrap()).contains(&"No data"));
}

#[test]
fn odd_input_stays_finite() {
    let mut odd: Series = ("A", vec![f32::NAN, 0.0, f32::INFINITY, 5.0, -3.0]).into();
    odd.start_index = usize::MAX;
    let mut late: Series = ("B", vec![4.0, 2.0]).into();
    late.start_index = 3;
    for axis in [PolarAxis::Angle, PolarAxis::Radius] {
        for (gap, inner, radius) in [
            (f32::NAN, f32::NAN, f32::NAN),
            (-3.0, -50.0, -1.0),
            (7.0, 1e9, 1e9),
            (0.0, 0.0, 0.5),
        ] {
            let mut chart = PolarBarChart::new(
                vec![odd.clone(), late.clone(), ("C", vec![1.0]).into()],
                names(&["a", "b", "c", "d", "e"]),
            );
            chart.category_axis = axis;
            chart.category_gap = Some(gap);
            chart.inner_radius = Some(inner);
            chart.radius = Some(radius);
            chart.round_cap = true;
            chart.tooltip_show = true;
            chart.series_list[1].label_show = true;
            let svg = chart.svg().unwrap();
            assert!(
                !svg.contains("NaN") && !svg.contains("inf"),
                "{axis:?} {gap}"
            );
            // B reaches the fourth and fifth category; A is off the chart.
            assert_eq!(3, bars(&svg).len(), "{axis:?} {gap}");
        }
    }

    // A tiny canvas, and a single value.
    let mut chart = PolarBarChart::new(vec![("A", vec![1.0]).into()], names(&["a"]));
    chart.width = 30.0;
    chart.height = 20.0;
    let svg = chart.svg().unwrap();
    assert!(!svg.contains("NaN") && !svg.contains("inf"));
}

#[test]
fn polar_bar_from_json() {
    let json = r##"{
        "width": 400,
        "height": 400,
        "legend_show": false,
        "category_axis": "Radius",
        "radius": 120,
        "inner_radius": 30,
        "start_angle": 90,
        "end_angle": 300,
        "round_cap": true,
        "category_gap": 0.4,
        "y_axis_configs": [{"axis_max": 40, "axis_split_number": 4}],
        "x_axis_data": ["a", "b"],
        "series_list": [{"name": "A", "data": [20, 40]}]
    }"##;
    let from_json = PolarBarChart::from_json(json).unwrap();
    assert_eq!(PolarAxis::Radius, from_json.category_axis);
    assert_eq!(Some(120.0), from_json.radius);
    assert_eq!(Some(30.0), from_json.inner_radius);
    assert_eq!(
        (90.0, Some(300.0)),
        (from_json.start_angle, from_json.end_angle)
    );
    assert_eq!(
        (true, Some(0.4)),
        (from_json.round_cap, from_json.category_gap)
    );

    // The same chart, built in code.
    let mut built = chart(vec![("A", vec![20.0, 40.0]).into()], &["a", "b"]);
    built.category_axis = PolarAxis::Radius;
    built.radius = Some(120.0);
    built.inner_radius = Some(30.0);
    built.start_angle = 90.0;
    built.end_angle = Some(300.0);
    built.round_cap = true;
    built.category_gap = Some(0.4);
    let svg = from_json.svg().unwrap();
    assert_eq!(built.svg().unwrap(), svg);
    // The radius is a limit for the plot.
    let rings = rings(&svg);
    assert_eq!((30.0, 120.0), (rings[0].2, rings[1].2));

    let message = err(PolarBarChart::from_json(r#"{"category_axis": "diagonal"}"#));
    assert!(
        message.contains("category_axis") && message.contains("radius"),
        "{message}"
    );
    let message = err(PolarBarChart::from_json(r#"{"radius": 0}"#));
    assert!(message.contains("radius"), "{message}");
    let message = err(PolarBarChart::from_json(r#"{"round_caps": true}"#));
    assert!(message.contains("round_cap"), "{message}");
}

#[test]
fn polar_bar_in_a_multi_chart() {
    let json = r##"{
        "child_charts": [
            {"type": "polar_bar", "category_axis": "radius", "x_axis_data": ["a", "b"],
             "series_list": [{"name": "A", "data": [1, 2]}]},
            {"type": "bar", "x_axis_data": ["a", "b"], "series_list": [{"name": "A", "data": [1, 2]}]}
        ]
    }"##;
    let svg = MultiChart::from_json(json).unwrap().svg().unwrap();
    assert_eq!(2, bars(&svg).len());
    let message = err(MultiChart::from_json(
        r#"{"child_charts": [{"type": "polar_bar", "bin_count": 3}]}"#,
    ));
    assert!(message.contains("bin_count"), "{message}");
}

#[test]
fn animation_and_compact_output() {
    let json = include_str!("../asset/polar_bar_chart/radial.json");
    let mut chart = PolarBarChart::from_json(json).unwrap();
    let plain = chart.svg().unwrap();
    assert!(!plain.contains("polar-anim"));
    chart.animation = Some(Default::default());
    let animated = chart.svg().unwrap();
    assert!(animated.contains("@keyframes polar-grow"));
    assert_eq!(
        5,
        animated.matches(r#"class="polar-anim ct-trigger""#).count()
    );
    assert!(animated.contains("animation-delay:"));

    // The compact form keeps every bar and what it says.
    let compact = compact_svg(&plain);
    assert!(compact.len() < plain.len());
    for needle in ["data-series=", "<title>", "class=\"ct-trigger\""] {
        assert_eq!(5, compact.matches(needle).count(), "{needle}");
    }
}
