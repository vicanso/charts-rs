//! Gantt charts: tasks as bars along a time axis.
mod common;

use charts_rs::{AxisType, Error, GanttChart, GanttTask, MultiChart, compact_svg};

fn attr<'a>(tag: &'a str, name: &str) -> &'a str {
    let key = format!(" {name}=\"");
    let start = tag.find(&key).map(|i| i + key.len()).unwrap();
    let end = start + tag[start..].find('"').unwrap();
    &tag[start..end]
}

fn number(tag: &str, name: &str) -> f32 {
    attr(tag, name).parse().unwrap()
}

/// The bars of the tasks: the rects that carry a name.
fn bars(svg: &str) -> Vec<&str> {
    svg.split("<rect")
        .skip(1)
        .filter(|t| t.contains("data-name="))
        .collect()
}

/// The milestones: the polygons that carry a name.
fn milestones(svg: &str) -> Vec<&str> {
    svg.split("<polygon")
        .skip(1)
        .filter(|t| t.contains("data-name="))
        .collect()
}

/// The text nodes of the svg.
fn texts(svg: &str) -> Vec<&str> {
    svg.lines()
        .map(str::trim)
        .filter(|l| !l.is_empty() && !l.starts_with('<'))
        .collect()
}

fn err(result: Result<impl Sized, Error>) -> String {
    match result {
        Ok(_) => panic!("expected an error"),
        Err(e) => e.to_string(),
    }
}

fn task(name: &str, start: f64, end: f64) -> GanttTask {
    (name, start, end).into()
}

fn on_row(name: &str, row: &str, start: f64, end: f64) -> GanttTask {
    GanttTask {
        row: Some(row.to_string()),
        ..task(name, start, end)
    }
}

/// A chart on an axis of plain numbers from 0 to 10, without the names of
/// the rows: the plot is as wide as the chart less its margins.
fn chart(tasks: Vec<GanttTask>) -> GanttChart {
    let mut chart = GanttChart::new(tasks);
    chart.x_axis.kind = AxisType::Value;
    chart.x_axis.min = Some(0.0);
    chart.x_axis.max = Some(10.0);
    chart.y_axis_hidden = true;
    chart
}

/// Width of the plot of [`chart`], and where it starts.
const PLOT: (f32, f32) = (590.0, 5.0);

#[test]
fn gantt_snapshots() {
    let basic = GanttChart::from_json(include_str!("../asset/gantt_chart/basic.json")).unwrap();
    common::assert_snapshot!("gantt_chart/basic_json.svg", basic.svg().unwrap());
    let rooms = GanttChart::from_json(include_str!("../asset/gantt_chart/rooms.json")).unwrap();
    common::assert_snapshot!("gantt_chart/rooms_json.svg", rooms.svg().unwrap());
}

#[test]
fn bars_span_their_tasks() {
    let svg = chart(vec![
        task("a", 2.0, 6.0),
        task("b", 5.0, 4.0),
        task("c", 0.0, 10.0),
    ])
    .svg()
    .unwrap();
    let bars = bars(&svg);
    assert_eq!(3, bars.len());
    let (width, left) = PLOT;
    let expect = |bar: &str, from: f32, to: f32| {
        assert!(
            (number(bar, "x") - (left + width * from / 10.0)).abs() < 0.1,
            "{bar}"
        );
        assert!(
            (number(bar, "width") - width * (to - from) / 10.0).abs() < 0.1,
            "{bar}"
        );
    };
    expect(bars[0], 2.0, 6.0);
    // A task given the wrong way round is the same span.
    expect(bars[1], 4.0, 5.0);
    expect(bars[2], 0.0, 10.0);
    assert_eq!(
        ("a", "2", "6"),
        (
            attr(bars[0], "data-name"),
            attr(bars[0], "data-start"),
            attr(bars[0], "data-end")
        )
    );

    // A row for each task, from the top, each as high as the others.
    let tops: Vec<f32> = bars.iter().map(|b| number(b, "y")).collect();
    assert!(tops[0] < tops[1] && tops[1] < tops[2]);
    assert!((tops[1] - tops[0] - (tops[2] - tops[1])).abs() < 0.1);
    let step = tops[1] - tops[0];
    // A bar takes 60% of its row.
    assert!((number(bars[0], "height") - step * 0.6).abs() < 0.2);
    assert_eq!("3", attr(bars[0], "rx"));
}

#[test]
fn tasks_share_a_row() {
    let mut chart = chart(vec![
        on_row("Standup", "Room 1", 1.0, 2.0),
        on_row("Workshop", "Room 2", 1.0, 6.0),
        on_row("Review", "Room 1", 4.0, 9.0),
    ]);
    chart.y_axis_hidden = false;
    let svg = chart.svg().unwrap();
    let bars = bars(&svg);
    // Two rows, in the order they are first met.
    assert_eq!(number(bars[0], "y"), number(bars[2], "y"));
    assert!(number(bars[1], "y") > number(bars[0], "y"));
    assert_eq!("Room 1", attr(bars[2], "data-row"));
    let texts = texts(&svg);
    let place = |name: &str| texts.iter().position(|t| *t == name).unwrap();
    assert!(place("Room 1") < place("Room 2"));
    // The tasks are named on their bars, as their row does not tell.
    for name in ["Workshop", "Review"] {
        assert!(texts.contains(&name), "{name} in {texts:?}");
    }
    let label = svg
        .split("<text")
        .find(|t| t.contains("\nWorkshop\n"))
        .unwrap();
    assert_eq!("middle", attr(label, "text-anchor"));
    // A name that does not fit its bar stands beside it.
    let label = svg
        .split("<text")
        .find(|t| t.contains("\nStandup\n"))
        .unwrap();
    assert_eq!("start", attr(label, "text-anchor"));
    assert!(number(label, "x") > number(bars[0], "x") + number(bars[0], "width"));

    // Without labels only the rows are named.
    chart.label_show = false;
    let svg = chart.svg().unwrap();
    assert!(!self::texts(&svg).contains(&"Workshop"));
    assert!(self::texts(&svg).contains(&"Room 2"));

    // A task on a row of its own is named by the row, not twice.
    let svg = self::chart(vec![task("Alone", 1.0, 2.0)]).svg().unwrap();
    assert!(!self::texts(&svg).contains(&"Alone"));
}

#[test]
fn a_label_gives_way_to_the_next_bar() {
    let svg = chart(vec![
        on_row("A long name", "r", 1.0, 1.2),
        on_row("x", "r", 1.3, 9.9),
    ])
    .svg()
    .unwrap();
    // No room on its bar, the next bar to its right, the edge to its left.
    assert!(!texts(&svg).contains(&"A long name"));
    // With the next bar further away the name stands between them.
    let svg = chart(vec![
        on_row("A long name", "r", 1.0, 1.2),
        on_row("x", "r", 6.0, 9.9),
    ])
    .svg()
    .unwrap();
    assert!(texts(&svg).contains(&"A long name"));
}

#[test]
fn milestones_and_progress() {
    let mut done = task("done", 2.0, 6.0);
    done.progress = Some(0.25);
    let mut not_started = task("waiting", 2.0, 6.0);
    not_started.progress = Some(0.0);
    let mut over = task("over", 2.0, 6.0);
    over.progress = Some(7.0);
    let svg = chart(vec![
        done,
        not_started,
        over,
        task("launch", 8.0, 8.0),
        task("open end", 9.0, f64::NAN),
    ])
    .svg()
    .unwrap();

    // A task that ends when it starts is a diamond at that moment.
    let milestones = milestones(&svg);
    assert_eq!(2, milestones.len());
    assert_eq!(4, attr(milestones[0], "points").split(' ').count());
    assert_eq!("8", attr(milestones[0], "data-start"));
    assert!(!milestones[0].contains("data-end="));
    let (width, left) = PLOT;
    let tip = attr(milestones[0], "points").split(' ').next().unwrap();
    let x: f32 = tip.split_once(',').unwrap().0.parse().unwrap();
    assert!((x - (left + width * 0.8)).abs() < 0.1);

    // What is done in the full color, under the lighter bar of the whole.
    let bars = bars(&svg);
    assert_eq!(3, bars.len());
    assert_eq!("0.25", attr(bars[0], "data-progress"));
    assert_eq!("0.43", attr(bars[0], "fill-opacity"));
    let full = number(bars[0], "width");
    let rects: Vec<&str> = svg.split("<rect").skip(2).collect();
    let part = rects
        .iter()
        .find(|r| !r.contains("data-name=") && number(r, "y") == number(bars[0], "y"))
        .unwrap();
    assert!((number(part, "width") - full / 4.0).abs() < 0.1);
    assert!(!part.contains("fill-opacity"));
    // Nothing done: no part in the full color. More than all is all.
    assert_eq!("0", attr(bars[1], "data-progress"));
    assert_eq!("1", attr(bars[2], "data-progress"));
    assert_eq!(
        2,
        rects.iter().filter(|r| !r.contains("data-name=")).count()
    );
}

#[test]
fn categories_color_the_tasks() {
    let categorized = |name: &str, category: Option<&str>| GanttTask {
        category: category.map(str::to_string),
        ..task(name, 1.0, 5.0)
    };
    let mut colored = categorized("d", Some("Build"));
    colored.color = Some("#123456".into());
    let svg = chart(vec![
        categorized("a", Some("Plan")),
        categorized("b", Some("Build")),
        categorized("c", Some("Plan")),
        colored,
        categorized("e", None),
    ])
    .svg()
    .unwrap();
    let fills: Vec<&str> = bars(&svg).iter().map(|b| attr(b, "fill")).collect();
    // A color for each category, one of its own, and the next for the rest.
    assert_eq!(
        vec!["#5470C6", "#91CC75", "#5470C6", "#123456", "#FAC858"],
        fills
    );
    assert_eq!("Build", attr(bars(&svg)[1], "data-category"));
    // The categories are the legend.
    let texts = texts(&svg);
    assert!(
        texts.contains(&"Plan") && texts.contains(&"Build"),
        "{texts:?}"
    );

    // Without categories every task has the first color, and no legend.
    let svg = chart(vec![task("a", 1.0, 5.0), task("b", 2.0, 6.0)])
        .svg()
        .unwrap();
    assert!(bars(&svg).iter().all(|b| attr(b, "fill") == "#5470C6"));
    let mut hidden = chart(vec![categorized("a", Some("Plan"))]);
    hidden.legend.show = Some(false);
    assert!(!self::texts(&hidden.svg().unwrap()).contains(&"Plan"));
}

#[test]
fn time_axis_and_the_marked_moment() {
    let json = r#"{
        "tooltip_show": true,
        "now": "2024-03-06 12:00",
        "x_axis_time_offset": 480,
        "tasks": [
            {"name": "Research", "start": "2024-03-04", "end": "2024-03-08", "progress": 0.5},
            {"name": "Launch", "start": "2024-03-10T09:30:00Z"}
        ]
    }"#;
    let chart = GanttChart::from_json(json).unwrap();
    assert_eq!(1_709_510_400.0, chart.tasks[0].start);
    assert_eq!(chart.tasks[1].start, chart.tasks[1].end);
    assert_eq!(Some(1_709_726_400.0), chart.now);
    let svg = chart.svg().unwrap();
    // Told in the time of the axis: eight hours east of UTC.
    assert!(svg.contains("<title>Research: 2024-03-04 08:00 – 2024-03-08 08:00 (50%)</title>"));
    assert!(svg.contains("<title>Launch: 2024-03-10 17:30</title>"));
    assert_eq!("2024-03-04 08:00", attr(bars(&svg)[0], "data-start"));
    // The ticks of the axis are dates.
    assert!(
        texts(&svg).iter().any(|t| t.starts_with("03-")),
        "{:?}",
        texts(&svg)
    );
    // The marked moment is a dashed line down the plot.
    assert_eq!(1, svg.matches("stroke-dasharray=\"4 3\"").count());

    // Outside the range of the axis there is no line; inside, the range
    // makes room for it.
    let mut chart = self::chart(vec![task("a", 2.0, 6.0)]);
    chart.now = Some(50.0);
    assert!(!chart.svg().unwrap().contains("stroke-dasharray"));
    chart.x_axis.max = None;
    let svg = chart.svg().unwrap();
    assert!(svg.contains("stroke-dasharray"));
    assert!(number(bars(&svg)[0], "width") < 60.0);
}

#[test]
fn fixed_range_cuts_the_tasks() {
    let svg = chart(vec![
        task("before", -9.0, -1.0),
        task("into", -5.0, 5.0),
        task("after", 20.0, 30.0),
        task("out of", 8.0, 30.0),
    ])
    .svg()
    .unwrap();
    let bars = bars(&svg);
    // What is not on the axis is not drawn; what reaches off it is cut.
    let names: Vec<&str> = bars.iter().map(|b| attr(b, "data-name")).collect();
    assert_eq!(vec!["into", "out of"], names);
    let (width, left) = PLOT;
    assert_eq!(left, number(bars[0], "x"));
    assert!((number(bars[0], "width") - width / 2.0).abs() < 0.1);
    assert!((number(bars[1], "x") + number(bars[1], "width") - (left + width)).abs() < 0.1);
    // The rows of the tasks that are left out stay.
    assert!((number(bars[1], "y") - number(bars[0], "y")).abs() > 100.0);

    // Without a fixed range there is a little room at both ends.
    let mut chart = chart(vec![task("a", 2.0, 6.0)]);
    chart.x_axis.min = None;
    chart.x_axis.max = None;
    let svg = chart.svg().unwrap();
    let bar = self::bars(&svg)[0];
    assert!(number(bar, "x") > left + 5.0);
    assert!(number(bar, "x") + number(bar, "width") < left + width - 5.0);
}

#[test]
fn nothing_to_draw_and_odd_input() {
    let mut chart = GanttChart::new(vec![task("nowhere", f64::NAN, 5.0)]);
    chart.empty_text = Some("No tasks".to_string());
    let svg = chart.svg().unwrap();
    assert_eq!(vec!["No tasks"], texts(&svg));
    assert!(GanttChart::new(vec![]).svg().unwrap().starts_with("<svg"));

    for (bar_height, radius) in [(f32::NAN, f32::NAN), (-5.0, -5.0), (1e9, 1e9), (0.0, 0.0)] {
        let mut chart = GanttChart::new(vec![
            task("moment", 1e12, 1e12),
            task("", 1e12, 1e12),
            on_row("x", "", f64::MAX, f64::MIN),
            GanttTask {
                progress: Some(f32::NAN),
                ..task("p", 1e12, f64::INFINITY)
            },
        ]);
        chart.bar_height = Some(bar_height);
        chart.radius = radius;
        chart.now = Some(f64::NAN);
        chart.tooltip.show = true;
        let svg = chart.svg().unwrap();
        assert!(
            !svg.contains("NaN") && !svg.contains("inf"),
            "{bar_height} {radius}"
        );
    }
    // A canvas too small for a plot.
    let mut chart = GanttChart::new(vec![task("a task with a long name", 1.0, 2.0)]);
    chart.width = 40.0;
    chart.height = 20.0;
    assert!(chart.svg().unwrap().starts_with("<svg"));
}

#[test]
fn gantt_from_json() {
    let json = r##"{
        "x_axis_type": "value",
        "x_axis_min": 0,
        "x_axis_max": 10,
        "bar_height": 12,
        "radius": 0,
        "label_show": false,
        "now": 4,
        "tasks": [
            {"name": "a", "row": "r", "category": "c", "start": 2, "end": 6, "progress": 0.5, "color": "#123456"},
            {"name": "b", "start": 7},
            {"name": "no start", "end": 7},
            {"name": "null start", "start": null, "end": 7}
        ]
    }"##;
    let from_json = GanttChart::from_json(json).unwrap();
    // A task needs a start; without an end it is a milestone.
    assert_eq!(2, from_json.tasks.len());
    assert_eq!(
        GanttTask {
            name: "a".to_string(),
            start: 2.0,
            end: 6.0,
            row: Some("r".to_string()),
            category: Some("c".to_string()),
            progress: Some(0.5),
            color: Some("#123456".into()),
        },
        from_json.tasks[0]
    );
    assert_eq!(
        (7.0, 7.0),
        (from_json.tasks[1].start, from_json.tasks[1].end)
    );
    assert_eq!(
        (Some(12.0), 0.0, false, Some(4.0)),
        (
            from_json.bar_height,
            from_json.radius,
            from_json.label_show,
            from_json.now
        )
    );

    // The same chart, built in code.
    let mut built = GanttChart::new(from_json.tasks.clone());
    built.x_axis.kind = AxisType::Value;
    built.x_axis.min = Some(0.0);
    built.x_axis.max = Some(10.0);
    built.bar_height = Some(12.0);
    built.radius = 0.0;
    built.label_show = false;
    built.now = Some(4.0);
    let svg = from_json.svg().unwrap();
    assert_eq!(built.svg().unwrap(), svg);
    assert_eq!("12", attr(bars(&svg)[0], "height"));
    assert!(!bars(&svg)[0].contains(" rx="));

    // The defaults of the builder are the defaults of the JSON.
    let plain =
        GanttChart::from_json(r#"{"tasks": [{"name": "a", "start": 1, "end": 2}]}"#).unwrap();
    assert_eq!(
        (None, 3.0, true, None),
        (plain.bar_height, plain.radius, plain.label_show, plain.now)
    );
    assert_eq!(
        GanttChart::new(vec![task("a", 1.0, 2.0)]).svg().unwrap(),
        plain.svg().unwrap()
    );

    let message = err(GanttChart::from_json(r#"{"task": []}"#));
    assert!(message.contains("tasks"), "{message}");
    let message = err(GanttChart::from_json(
        r#"{"tasks": [{"start": "yesterday"}]}"#,
    ));
    assert!(message.contains("tasks[0].start"), "{message}");
    let message = err(GanttChart::from_json(
        r#"{"tasks": [{"start": 1, "duration": 3}]}"#,
    ));
    assert!(message.contains("duration"), "{message}");
    let message = err(GanttChart::from_json(r#"{"now": true}"#));
    assert!(message.contains("now"), "{message}");
}

#[test]
fn gantt_in_a_multi_chart_animated_and_compact() {
    let json = r##"{
        "child_charts": [
            {"type": "gantt", "tasks": [{"name": "a", "start": "2024-03-04", "end": "2024-03-08"}]},
            {"type": "bar", "x_axis_data": ["a", "b"], "series_list": [{"name": "A", "data": [1, 2]}]}
        ]
    }"##;
    let svg = MultiChart::from_json(json).unwrap().svg().unwrap();
    assert_eq!(1, bars(&svg).len());

    let json = include_str!("../asset/gantt_chart/basic.json");
    let mut chart = GanttChart::from_json(json).unwrap();
    let plain = chart.svg().unwrap();
    assert!(!plain.contains("gantt-anim"));
    chart.animation = Some(Default::default());
    let animated = chart.svg().unwrap();
    assert!(animated.contains("@keyframes gantt-grow"));
    // Every bar and milestone grows from its start, row after row.
    assert_eq!(
        8,
        animated.matches(r#"class="gantt-anim ct-trigger""#).count()
    );
    assert!(animated.contains("animation-delay:"));

    // The compact form keeps every task and what it says.
    let compact = compact_svg(&plain);
    assert!(compact.len() < plain.len());
    for needle in ["data-name=", "<title>", "class=\"ct-trigger\""] {
        assert_eq!(8, compact.matches(needle).count(), "{needle}");
    }
}
