//! Stepped lines: level between two points, changing value at a corner.
mod common;

use charts_rs::{BarChart, Error, LineChart, LineStep, Series, SeriesBand, SeriesCategory};

fn attr<'a>(tag: &'a str, name: &str) -> &'a str {
    let key = format!(" {name}=\"");
    let start = tag.find(&key).map(|i| i + key.len()).unwrap();
    let end = start + tag[start..].find('"').unwrap();
    &tag[start..end]
}

type Points = Vec<(f32, f32)>;

/// The points of every path of the svg that is made of straight segments,
/// lines and area outlines alike.
fn paths(svg: &str) -> Vec<Points> {
    svg.split("<path d=\"M ")
        .skip(1)
        .map(|t| {
            t[..t.find('"').unwrap()]
                .trim_end_matches(" Z")
                .split(" L ")
                .map(|p| {
                    let (x, y) = p.split_once(' ').unwrap();
                    (x.parse().unwrap(), y.parse().unwrap())
                })
                .collect()
        })
        .collect()
}

/// The corners of every polygon of the svg.
fn polygons(svg: &str) -> Vec<Points> {
    svg.split("<polygon")
        .skip(1)
        .map(|t| {
            attr(t, "points")
                .split(' ')
                .map(|p| {
                    let (x, y) = p.split_once(',').unwrap();
                    (x.parse().unwrap(), y.parse().unwrap())
                })
                .collect()
        })
        .collect()
}

/// The markers on the points of the lines (not the one of the legend).
fn markers(svg: &str) -> usize {
    svg.matches(r#" r="2"/>"#).count()
}

fn err(result: Result<impl Sized, Error>) -> String {
    match result {
        Ok(_) => panic!("expected an error"),
        Err(e) => e.to_string(),
    }
}

fn categories(count: usize) -> Vec<String> {
    (0..count).map(|i| format!("c{i}")).collect()
}

fn stepped(data: Vec<f32>, step: Option<LineStep>) -> Series {
    let mut series: Series = ("A", data).into();
    series.step = step;
    series
}

fn chart(series_list: Vec<Series>) -> LineChart {
    let count = series_list.iter().map(|s| s.data.len()).max().unwrap_or(0);
    LineChart::new(series_list, categories(count))
}

#[test]
fn step_snapshot() {
    let chart = LineChart::from_json(include_str!("../asset/line_chart/step.json")).unwrap();
    common::assert_snapshot!("line_chart/step_json.svg", chart.svg().unwrap());
}

#[test]
fn steps_turn_where_asked() {
    let line = |step| {
        let svg = chart(vec![stepped(vec![10.0, 30.0, 20.0], step)])
            .svg()
            .unwrap();
        let paths = paths(&svg);
        assert_eq!(1, paths.len());
        paths.into_iter().next().unwrap()
    };
    let plain = line(None);
    let [a, b, c] = plain[..] else {
        panic!("{plain:?}")
    };
    // The value changes at the point itself, then runs level to the next.
    assert_eq!(
        vec![a, (a.0, b.1), b, (b.0, c.1), c],
        line(Some(LineStep::Start))
    );
    // It runs level to the next point, and changes there.
    assert_eq!(
        vec![a, (b.0, a.1), b, (c.0, b.1), c],
        line(Some(LineStep::End))
    );
    // It changes half way.
    let (ab, bc) = ((a.0 + b.0) / 2.0, (b.0 + c.0) / 2.0);
    let middle = line(Some(LineStep::Middle));
    assert_eq!(7, middle.len());
    for (found, wanted) in middle
        .iter()
        .zip([a, (ab, a.1), (ab, b.1), b, (bc, b.1), (bc, c.1), c])
    {
        assert!(
            (found.0 - wanted.0).abs() < 0.1 && found.1 == wanted.1,
            "{middle:?}"
        );
    }
}

#[test]
fn markers_and_labels_stay_on_the_points() {
    let mut plain = chart(vec![stepped(vec![10.0, 30.0, 20.0], None)]);
    plain.series_list[0].label_show = true;
    plain.tooltip.show = true;
    let mut steps = plain.clone();
    steps.series_list[0].step = Some(LineStep::Middle);
    let (plain, steps) = (plain.svg().unwrap(), steps.svg().unwrap());
    // Three points, whatever the number of corners.
    assert_eq!(3, markers(&plain));
    assert_eq!(3, markers(&steps));
    assert_eq!(3, steps.matches(r#"class="ct-trigger""#).count());
    // Only the path of the line differs.
    let strip = |svg: &str| -> String {
        svg.lines()
            .filter(|l| !l.starts_with("<path d="))
            .collect::<Vec<_>>()
            .join("\n")
    };
    assert_eq!(strip(&plain), strip(&steps));
    assert_ne!(plain, steps);
}

#[test]
fn step_wins_over_smooth() {
    let mut chart = chart(vec![stepped(vec![10.0, 30.0, 20.0], Some(LineStep::End))]);
    let steps = chart.svg().unwrap();
    chart.series.smooth = true;
    assert_eq!(steps, chart.svg().unwrap());
    chart.series_list[0].smooth = Some(true);
    assert_eq!(steps, chart.svg().unwrap());
    // Without the step the curve is back.
    chart.series_list[0].step = None;
    assert!(chart.svg().unwrap().contains(" C"));
}

#[test]
fn area_follows_the_steps() {
    let mut chart = chart(vec![stepped(vec![10.0, 30.0, 20.0], Some(LineStep::End))]);
    let line = paths(&chart.svg().unwrap()).remove(0);
    assert_eq!(5, line.len());
    chart.series.fill = true;
    let svg = chart.svg().unwrap();
    let paths = paths(&svg);
    // The area first, then the line: the same corners, closed along the
    // bottom of the plot.
    assert_eq!(2, paths.len());
    assert_eq!(line, paths[1]);
    assert_eq!(line[..], paths[0][..5]);
    assert_eq!(paths[0][5].1, paths[0][6].1);
    assert!(paths[0][5].1 > line.iter().map(|p| p.1).fold(0.0, f32::max));

    // Stacked: the area between two stepped lines.
    let mut lower = stepped(vec![10.0, 30.0, 20.0], Some(LineStep::End));
    let mut upper = stepped(vec![5.0, 5.0, 15.0], Some(LineStep::End));
    for series in [&mut lower, &mut upper] {
        series.stack = Some("total".to_string());
        series.fill = Some(true);
    }
    upper.name = "B".to_string();
    let svg = self::chart(vec![lower, upper]).svg().unwrap();
    let (lines, areas) = (self::paths(&svg), polygons(&svg));
    assert_eq!((2, 2), (lines.len(), areas.len()));
    // The second area lies between the two lines, corner for corner.
    let mut outline = lines[1].clone();
    outline.extend(lines[0].iter().rev());
    assert_eq!(outline, areas[1]);
}

#[test]
fn missing_point_ends_a_run_of_steps() {
    let series = Series::new_nullable(
        "A".to_string(),
        vec![
            Some(10.0),
            Some(30.0),
            None,
            Some(20.0),
            Some(40.0),
            None,
            Some(5.0),
        ],
    );
    let mut chart = chart(vec![series]);
    chart.series_list[0].step = Some(LineStep::Start);
    let svg = chart.svg().unwrap();
    let paths = paths(&svg);
    // Two runs of two points (three corners each), and a point on its own.
    let lengths: Vec<usize> = paths.iter().map(Vec::len).collect();
    assert_eq!(vec![3, 3, 1], lengths);
    assert_eq!(5, markers(&svg));
    // No step reaches across the gap.
    assert!(paths[0][2].0 < paths[1][0].0);
}

#[test]
fn band_steps_with_its_line() {
    let mut series = stepped(vec![10.0, 30.0, 20.0], Some(LineStep::End));
    series.band = Some(SeriesBand::new(
        vec![5.0, 20.0, 10.0],
        vec![15.0, 40.0, 30.0],
    ));
    let svg = chart(vec![series]).svg().unwrap();
    let band = polygons(&svg).remove(0);
    let line = paths(&svg).remove(0);
    // The upper bounds from left to right, then the lower ones back: five
    // corners each, at the corners of the line.
    assert_eq!(10, band.len());
    for (i, point) in line.iter().enumerate() {
        assert_eq!(point.0, band[i].0);
        assert_eq!(point.0, band[9 - i].0);
        assert!(band[i].1 < point.1 && point.1 < band[9 - i].1, "{i}");
    }
}

#[test]
fn step_on_a_continuous_axis_and_in_a_bar_chart() {
    let mut chart = LineChart::new(
        vec![stepped(vec![10.0, 30.0, 20.0], Some(LineStep::End))],
        vec![],
    );
    chart.x_axis.values = vec![0.0, 1.0, 10.0];
    let line = paths(&chart.svg().unwrap()).remove(0);
    assert_eq!(5, line.len());
    // The steps are as long as the distance to the next x value.
    assert!(
        (line[3].0 - line[2].0) > 5.0 * (line[1].0 - line[0].0),
        "{line:?}"
    );

    let mut line_series = stepped(vec![10.0, 30.0, 20.0], Some(LineStep::Start));
    line_series.category = Some(SeriesCategory::Line);
    let bars = BarChart::new(
        vec![("Bars", vec![5.0, 6.0, 7.0]).into(), line_series],
        categories(3),
    );
    let line = paths(&bars.svg().unwrap()).remove(0);
    assert_eq!(5, line.len());
    assert_eq!(line[0].0, line[1].0);
}

#[test]
fn step_from_json() {
    let json = |step: &str| {
        format!(
            r#"{{"x_axis_data": ["a", "b", "c"], "series_list": [{{"name": "A", "data": [10, 30, 20]{step}}}]}}"#
        )
    };
    let plain = LineChart::from_json(&json("")).unwrap();
    assert_eq!(None, plain.series_list[0].step);
    for (name, step) in [
        ("start", LineStep::Start),
        ("Middle", LineStep::Middle),
        ("END", LineStep::End),
    ] {
        let chart = LineChart::from_json(&json(&format!(r#", "step": "{name}""#))).unwrap();
        assert_eq!(Some(step), chart.series_list[0].step);
        // The same chart, built in code.
        let built = LineChart::new(
            vec![stepped(vec![10.0, 30.0, 20.0], Some(step))],
            ["a", "b", "c"].iter().map(|c| c.to_string()).collect(),
        );
        assert_eq!(built.svg().unwrap(), chart.svg().unwrap());
    }
    // `null` is no step.
    let chart = LineChart::from_json(&json(r#", "step": null"#)).unwrap();
    assert_eq!(plain.svg().unwrap(), chart.svg().unwrap());

    let message = err(LineChart::from_json(&json(r#", "step": "left""#)));
    assert!(
        message.contains("step") && message.contains("start, middle, end"),
        "{message}"
    );
    let message = err(LineChart::from_json(&json(r#", "step": true"#)));
    assert!(message.contains("series_list[0].step"), "{message}");
}
