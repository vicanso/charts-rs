//! The labels of a horizontal axis stay on the chart: the label of a tick at
//! an end of the axis, half of which is beyond it, is moved in rather than
//! cut where there is no margin for that half.

use charts_rs::{
    Align, Axis, Canvas, ChildChart, HorizontalBarChart, LineChart, MultiChart, Position,
    measure_text_width_family,
};

fn attr(tag: &str, name: &str) -> f32 {
    let key = format!(" {name}=\"");
    let start = tag.find(&key).map(|i| i + key.len()).unwrap();
    let end = start + tag[start..].find('"').unwrap();
    tag[start..end].parse().unwrap()
}

/// Where the text with this content starts and where it ends.
fn label(svg: &str, text: &str) -> (f32, f32) {
    let ending = format!(">\n{text}\n</text>");
    let end = svg
        .find(&ending)
        .unwrap_or_else(|| panic!("{text:?} is not in {svg}"));
    let tag = &svg[svg[..end].rfind("<text").unwrap()..end];
    let width = measure_text_width_family("Roboto", attr(tag, "font-size"), text)
        .unwrap()
        .width();
    (attr(tag, "x"), attr(tag, "x") + width)
}

fn has(svg: &str, text: &str) -> bool {
    svg.contains(&format!(">\n{text}\n</text>"))
}

const POPULATION: &str = r#""x_axis_data": ["Brazil", "Indonesia", "USA", "India", "China"],
    "series_list": [{"name": "2011", "data": [203000000, 239000000, 310000000, 1210000000, 1310000000]}]"#;

#[test]
fn the_last_label_of_a_value_axis_is_on_the_chart() {
    let chart = |options: &str| -> String {
        HorizontalBarChart::from_json(&format!("{{{POPULATION}{options}}}"))
            .unwrap()
            .svg()
            .unwrap()
    };
    // The axis ends 5 pixels from the edge of a chart of 600, and half of
    // its last label is more than that: it was written up to 612, and cut
    // to "1.6". It ends at the edge now.
    let svg = chart("");
    let (start, end) = label(&svg, "1.6G");
    assert!(end - start > 24.0, "{start} {end}");
    assert!((end - 600.0).abs() < 0.1, "{end}");
    // The others are where they were, in the middle below their ticks: as
    // far apart as each other, and the last of them as far from the end of
    // the axis, which is 5 pixels from the edge.
    let middle = |text: &str| {
        let (start, end) = label(&svg, text);
        (start + end) / 2.0
    };
    let middles: Vec<f32> = ["0", "262M", "524M", "786M", "1G", "1.3G"]
        .into_iter()
        .map(middle)
        .collect();
    let apart = middles[1] - middles[0];
    for pair in middles.windows(2) {
        assert!((pair[1] - pair[0] - apart).abs() < 1.0, "{middles:?}");
    }
    assert!((middles[5] + apart - 595.0).abs() < 1.0, "{middles:?}");
    // Without a margin as well, and in a chart of another width.
    let (_, end) = label(&chart(r#", "margin": 0"#), "1.6G");
    assert!((end - 600.0).abs() < 0.1, "{end}");
    let (_, end) = label(&chart(r#", "width": 431"#), "1.6G");
    assert!((end - 431.0).abs() < 0.1, "{end}");

    // With a margin for it, it is in the middle below its tick as well.
    let svg = chart(r#", "margin": {"right": 40}"#);
    let (start, end) = label(&svg, "1.6G");
    assert!(((start + end) / 2.0 - 560.0).abs() < 1.0, "{start} {end}");
    // As it is with one that is just enough: nothing is moved then.
    let svg = chart(r#", "margin": {"right": 14}"#);
    let (start, end) = label(&svg, "1.6G");
    assert!(end <= 600.0 && ((start + end) / 2.0 - 586.0).abs() < 1.0);
}

#[test]
fn the_labels_at_both_ends_of_a_category_axis_are_on_the_chart() {
    let chart = |options: &str| -> String {
        LineChart::from_json(&format!(
            r#"{{"x_boundary_gap": false,
                "x_axis_data": ["2026-01-01", "2026-02-01", "2026-03-01", "2026-04-01"],
                "series_list": [{{"name": "s", "data": [1, 3, 2, 4]}}]{options}}}"#
        ))
        .unwrap()
        .svg()
        .unwrap()
    };
    // Without a gap at its ends the axis has a label below each of them:
    // the first one began before the chart and the last one ended after it.
    let svg = chart("");
    let (first, _) = label(&svg, "2026-01-01");
    let (_, last) = label(&svg, "2026-04-01");
    assert_eq!((0.0, 600.0), (first, (last * 10.0).round() / 10.0));
    // Those between them are below their ticks: a third of the axis from
    // each other and from its end, which is 5 pixels from the edge.
    let middle = |text: &str| {
        let (start, end) = label(&svg, text);
        (start + end) / 2.0
    };
    let apart = middle("2026-03-01") - middle("2026-02-01");
    assert!(apart > 150.0 && (middle("2026-03-01") + apart - 595.0).abs() < 1.0);
    // With margins that are wide enough nothing is moved.
    let svg = chart(r#", "margin": {"left": 40, "right": 40}"#);
    let (start, end) = label(&svg, "2026-04-01");
    assert!(((start + end) / 2.0 - 560.0).abs() < 1.0, "{start} {end}");
    let (start, end) = label(&svg, "2026-01-01");
    assert!(start > 0.0 && end < 600.0);
}

/// An axis along the bottom with a label below each of its ticks.
fn axis(left: f32, width: f32, data: &[&str]) -> Axis {
    Axis {
        position: Position::Bottom,
        data: data.iter().map(|label| label.to_string()).collect(),
        name_align: Align::Left,
        left,
        top: 10.0,
        width,
        height: 30.0,
        ..Default::default()
    }
}

fn on_canvas(axis: Axis) -> String {
    let mut canvas = Canvas::new(300.0, 60.0);
    canvas.axis(axis);
    canvas.svg().unwrap()
}

#[test]
fn an_axis_keeps_its_labels_between_its_edges() {
    let ends = ["a long first label", "b", "c", "a long last label"];
    // On its own an axis has no edges: its labels are where their ticks are.
    let alone = axis(5.0, 290.0, &ends).svg().unwrap();
    let (first, _) = label(&alone, ends[0]);
    let (_, last) = label(&alone, ends[3]);
    assert!(first < -40.0 && last > 340.0, "{first} {last}");
    // On a canvas they are kept on it,
    let svg = on_canvas(axis(5.0, 290.0, &ends));
    let (first, _) = label(&svg, ends[0]);
    let (_, last) = label(&svg, ends[3]);
    assert!(first == 0.0 && (last - 300.0).abs() < 0.1, "{first} {last}");
    // or between the edges the axis is given.
    let mut given = axis(5.0, 290.0, &ends);
    given.label_bounds = Some((12.0, 280.0));
    let svg = on_canvas(given);
    let (first, _) = label(&svg, ends[0]);
    let (_, last) = label(&svg, ends[3]);
    assert!(
        first == 12.0 && (last - 280.0).abs() < 0.1,
        "{first} {last}"
    );

    // Labels that are between the edges as it is are not touched,
    let short = ["0", "1", "2", "3"];
    let mut wide = axis(20.0, 260.0, &short);
    let alone = wide.svg().unwrap();
    wide.label_bounds = Some((0.0, 300.0));
    assert_eq!(alone, wide.svg().unwrap());
    // nor are those that are turned, and those of an axis at a side.
    let mut turned = axis(5.0, 290.0, &ends);
    turned.name_rotate = 0.5;
    let alone = turned.svg().unwrap();
    turned.label_bounds = Some((0.0, 300.0));
    assert_eq!(alone, turned.svg().unwrap());
    let mut side = axis(5.0, 290.0, &ends);
    side.position = Position::Left;
    let alone = side.svg().unwrap();
    side.label_bounds = Some((0.0, 300.0));
    assert_eq!(alone, side.svg().unwrap());
}

#[test]
fn a_label_moved_in_is_not_over_the_one_next_to_it() {
    // The ticks are 100 pixels apart, from 0 to 300.
    // The last label would be over the one before it: that one is left out.
    let last = ["0", "100", "200", "300 thousand units"];
    let svg = on_canvas(axis(0.0, 300.0, &last));
    let (start, end) = label(&svg, last[3]);
    assert!(start < 212.0 && (end - 300.0).abs() < 0.1, "{start} {end}");
    assert!(has(&svg, "0") && has(&svg, "100") && !has(&svg, "200"));
    // On its own, where nothing is moved, all four are written.
    let alone = axis(0.0, 300.0, &last).svg().unwrap();
    assert!(last.iter().all(|text| has(&alone, text)));

    // The first label keeps its place, and the one after it is left out.
    let first = ["a first label that is long", "1", "2", "3"];
    let svg = on_canvas(axis(0.0, 300.0, &first));
    let (start, end) = label(&svg, first[0]);
    assert!(start == 0.0 && end > 100.0, "{start} {end}");
    assert!(!has(&svg, "1") && has(&svg, "2") && has(&svg, "3"));
    // The last one is moved in by the little it is beyond the edge.
    let (_, end) = label(&svg, "3");
    assert!((end - 300.0).abs() < 0.1, "{end}");

    // With room between them every label is written, the moved ones too.
    let roomy = ["first", "1", "2", "last"];
    let svg = on_canvas(axis(0.0, 300.0, &roomy));
    assert!(roomy.iter().all(|text| has(&svg, text)));
    assert_eq!(0.0, label(&svg, "first").0);
}

#[test]
fn the_charts_of_a_multi_chart_keep_their_labels_on_themselves() {
    // Two charts of 300 side by side: the last label of the left one ends
    // at its own edge, not at that of the picture.
    let child = || {
        let mut chart =
            HorizontalBarChart::from_json(&format!("{{{POPULATION}, \"width\": 300}}")).unwrap();
        chart.margin = 0.0.into();
        chart
    };
    let mut multi = MultiChart::new();
    multi.margin = 0.0.into();
    multi.add(ChildChart::HorizontalBar(child(), Some((0.0, 0.0))));
    multi.add(ChildChart::HorizontalBar(child(), Some((300.0, 0.0))));
    let svg = multi.svg().unwrap();
    assert!(svg.starts_with("<svg width=\"600\""), "{}", &svg[..60]);
    let alone = child().svg().unwrap();
    let text = ["1.6G", "1.5G", "1.4G", "2G"]
        .into_iter()
        .find(|text| has(&alone, text))
        .or_else(|| {
            // Whatever the last label of the axis is at this width.
            alone
                .rsplit("</text>")
                .filter_map(|part| part.rsplit('\n').nth(1))
                .find(|text| text.ends_with('G'))
        })
        .unwrap();
    let (_, end) = label(&alone, text);
    assert!((end - 300.0).abs() < 0.1, "{text}: {end}");
    // Both children are written as the chart on its own is.
    assert_eq!(2, svg.matches(&format!(">\n{text}\n</text>")).count());
    let inner = &alone[alone.find("<rect").unwrap()..alone.rfind("</svg>").unwrap()];
    assert_eq!(2, svg.matches(inner).count());
}
