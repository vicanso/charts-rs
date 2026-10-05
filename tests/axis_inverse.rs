//! Inverse value axes: the smallest value where the largest usually is.
mod common;

use charts_rs::{AxisScale, BarChart, Error, HorizontalBarChart, LineChart, ScatterChart, Series};

fn attr(tag: &str, name: &str) -> f32 {
    let key = format!(" {name}=\"");
    let start = tag.find(&key).map(|i| i + key.len()).unwrap();
    let end = start + tag[start..].find('"').unwrap();
    tag[start..end].parse().unwrap()
}

/// The points of every straight path of the svg.
fn paths(svg: &str) -> Vec<Vec<(f32, f32)>> {
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

/// `(x, y, width, height)` of every bar: the rects that carry a series.
fn bars(svg: &str) -> Vec<(f32, f32, f32, f32)> {
    svg.split("<rect")
        .skip(1)
        .filter(|t| t.contains("data-series="))
        .map(|t| {
            (
                attr(t, "x"),
                attr(t, "y"),
                attr(t, "width"),
                attr(t, "height"),
            )
        })
        .collect()
}

/// The numbers written in the svg, in order, with the y they stand at.
fn numbers(svg: &str) -> Vec<(f32, f32)> {
    svg.split("<text")
        .skip(1)
        .filter_map(|t| {
            let text = t.split('\n').nth(1)?.trim();
            Some((text.parse().ok()?, attr(t, "y")))
        })
        .collect()
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

/// A line chart with an axis from 0 to 40, without a legend.
fn line(series_list: Vec<Series>) -> LineChart {
    let mut chart = LineChart::new(series_list, categories(4));
    chart.legend.show = Some(false);
    chart.y_axis_configs[0].max = Some(40.0);
    chart.y_axis_configs[0].split_number = 4;
    chart
}

#[test]
fn bump_chart_snapshot() {
    let chart = LineChart::from_json(include_str!("../asset/line_chart/bump.json")).unwrap();
    assert!(chart.y_axis_configs[0].inverse);
    common::assert_snapshot!("line_chart/bump_json.svg", chart.svg().unwrap());
}

#[test]
fn inverse_axis_turns_the_plot_upside_down() {
    let mut chart = line(vec![("A", vec![0.0, 10.0, 30.0, 40.0]).into()]);
    let plain = chart.svg().unwrap();
    chart.y_axis_configs[0].inverse = true;
    let inverse = chart.svg().unwrap();

    let (up, down) = (&paths(&plain)[0], &paths(&inverse)[0]);
    // The same x, and the y mirrored between the ends of the axis.
    let (top, bottom) = (up[3].1, up[0].1);
    assert!(top < bottom);
    for (a, b) in up.iter().zip(down.iter()) {
        assert_eq!(a.0, b.0);
        assert!((a.1 + b.1 - (top + bottom)).abs() < 0.1, "{a:?} {b:?}");
    }
    // The smallest value is at the top now.
    assert_eq!((top, bottom), (down[0].1, down[3].1));

    // The labels of the axis follow: 0 at the top, 40 at the bottom.
    let order = |svg: &str| -> Vec<f32> {
        let mut labels = numbers(svg);
        labels.sort_by(|a, b| a.1.total_cmp(&b.1));
        labels.into_iter().map(|l| l.0).collect()
    };
    assert_eq!(vec![40.0, 30.0, 20.0, 10.0, 0.0], order(&plain));
    assert_eq!(vec![0.0, 10.0, 20.0, 30.0, 40.0], order(&inverse));
}

#[test]
fn area_reaches_to_the_start_of_the_axis() {
    let mut chart = line(vec![("A", vec![10.0, 20.0, 30.0, 20.0]).into()]);
    chart.series.fill = true;
    let plain = chart.svg().unwrap();
    chart.y_axis_configs[0].inverse = true;
    let inverse = chart.svg().unwrap();
    // The area is closed along the line of the value 0: the bottom of the
    // plot, or its top on an inverse axis.
    let (up, down) = (&paths(&plain)[0], &paths(&inverse)[0]);
    let zero = |points: &Vec<(f32, f32)>| points[4].1;
    assert_eq!(up[4].1, up[5].1);
    assert_eq!(down[4].1, down[5].1);
    assert!(zero(up) > up.iter().take(4).map(|p| p.1).fold(0.0, f32::max));
    assert!(zero(down) < down.iter().take(4).map(|p| p.1).fold(f32::MAX, f32::min));
}

#[test]
fn bars_hang_from_their_base() {
    let mut series: Series = ("A", vec![10.0, 40.0, -10.0]).into();
    series.label_show = true;
    let mut chart = BarChart::new(vec![series], categories(3));
    chart.legend.show = Some(false);
    chart.y_axis_configs[0].min = Some(-20.0);
    chart.y_axis_configs[0].max = Some(40.0);
    chart.y_axis_configs[0].split_number = 3;
    let plain = chart.svg().unwrap();
    chart.y_axis_configs[0].inverse = true;
    let inverse = chart.svg().unwrap();

    let (up, down) = (bars(&plain), bars(&inverse));
    assert_eq!((3, 3), (up.len(), down.len()));
    // As high as before, on the other side of the line of 0.
    for (a, b) in up.iter().zip(down.iter()) {
        assert!((a.3 - b.3).abs() < 0.1 && a.0 == b.0, "{a:?} {b:?}");
    }
    let zero_up = up[0].1 + up[0].3;
    let zero_down = down[0].1;
    // A positive bar starts at 0 and hangs down; a negative one stands up.
    assert!((down[1].1 - zero_down).abs() < 0.1);
    assert!((down[2].1 + down[2].3 - zero_down).abs() < 0.1);
    assert!((up[1].1 + up[1].3 - zero_up).abs() < 0.1);
    // 0 is a third of the way down instead of a third of the way up.
    assert!(zero_down < zero_up);

    // The label of a hanging bar is below its end, not inside it.
    let label = |svg: &str, value: f32| numbers(svg).iter().find(|n| n.0 == value).unwrap().1;
    assert!(label(&inverse, 10.0) > down[0].1 + down[0].3);
    assert!(label(&plain, 10.0) <= up[0].1);
}

#[test]
fn each_axis_is_inverse_on_its_own() {
    let mut right: Series = ("B", vec![0.0, 10.0, 30.0, 40.0]).into();
    right.y_axis_index = 1;
    let mut chart = line(vec![("A", vec![0.0, 10.0, 30.0, 40.0]).into(), right]);
    chart.y_axis_configs.push(chart.y_axis_configs[0].clone());
    chart.y_axis_configs[1].inverse = true;
    let svg = chart.svg().unwrap();
    let lines = paths(&svg);
    // The same values: up on the left axis, down on the right one.
    assert!(lines[0][0].1 > lines[0][3].1);
    assert!(lines[1][0].1 < lines[1][3].1);
    assert_eq!(lines[0][0].1, lines[1][3].1);
}

#[test]
fn inverse_log_axis() {
    let mut chart = LineChart::new(
        vec![("A", vec![1.0, 10.0, 100.0, 1000.0]).into()],
        categories(4),
    );
    chart.legend.show = Some(false);
    chart.y_axis_configs[0].scale = AxisScale::Log(10.0);
    let plain = chart.svg().unwrap();
    chart.y_axis_configs[0].inverse = true;
    let inverse = chart.svg().unwrap();
    let (up, down) = (&paths(&plain)[0], &paths(&inverse)[0]);
    // Evenly spaced decades, in the other direction.
    let step = up[0].1 - up[1].1;
    assert!(step > 10.0);
    for i in 0..3 {
        assert!((up[i].1 - up[i + 1].1 - step).abs() < 0.2);
        assert!((down[i + 1].1 - down[i].1 - step).abs() < 0.2);
    }
}

#[test]
fn inverse_axes_of_other_charts() {
    // A scatter chart: each of its two value axes on its own.
    let scatter = |x: bool, y: bool| {
        let mut chart = ScatterChart::new(vec![("A", vec![1.0, 2.0, 9.0, 8.0]).into()]);
        chart.x_axis_config.inverse = x;
        chart.y_axis_configs[0].inverse = y;
        let svg = chart.svg().unwrap();
        let points: Vec<(f32, f32)> = svg
            .split("<circle")
            .skip(1)
            .filter(|t| t.contains("data-series="))
            .map(|t| (attr(t, "cx"), attr(t, "cy")))
            .collect();
        assert_eq!(2, points.len());
        (points[1].0 > points[0].0, points[1].1 < points[0].1)
    };
    // The larger point is right of and above the smaller one, unless turned.
    assert_eq!((true, true), scatter(false, false));
    assert_eq!((true, false), scatter(false, true));
    assert_eq!((false, true), scatter(true, false));

    // Horizontal bars run to the left from the right end of their axis.
    let mut series: Series = ("A", vec![10.0, 33.0]).into();
    series.label_show = true;
    let mut chart = HorizontalBarChart::new(vec![series], categories(2));
    chart.legend.show = Some(false);
    let plain = bars(&chart.svg().unwrap());
    chart.y_axis_configs[0].inverse = true;
    let svg = chart.svg().unwrap();
    let inverse = bars(&svg);
    assert_eq!(plain[0].0, plain[1].0);
    assert!((inverse[0].0 + inverse[0].2 - (inverse[1].0 + inverse[1].2)).abs() < 0.1);
    assert!(inverse[1].0 < inverse[0].0);
    for (a, b) in plain.iter().zip(inverse.iter()) {
        assert!((a.2 - b.2).abs() < 0.1, "{a:?} {b:?}");
    }
    // Their labels are past their end: left of it.
    let label = svg.split("<text").find(|t| t.contains("\n33\n")).unwrap();
    assert!(attr(label, "x") + attr(label, "dx") < inverse[1].0);
}

#[test]
fn inverse_from_json() {
    let json = |config: &str| {
        format!(
            r#"{{"x_axis_data": ["a", "b"], "y_axis_configs": [{config}], "series_list": [{{"name": "A", "data": [1, 5]}}]}}"#
        )
    };
    let plain = LineChart::from_json(&json("{}")).unwrap();
    assert!(!plain.y_axis_configs[0].inverse);
    let off = LineChart::from_json(&json(r#"{"axis_inverse": false}"#)).unwrap();
    assert_eq!(plain.svg().unwrap(), off.svg().unwrap());
    let on = LineChart::from_json(&json(r#"{"axis_inverse": true}"#)).unwrap();
    assert!(on.y_axis_configs[0].inverse);
    let (up, down) = (
        &paths(&plain.svg().unwrap())[0],
        &paths(&on.svg().unwrap())[0],
    );
    assert!(up[0].1 > up[1].1 && down[0].1 < down[1].1);

    let message = err(LineChart::from_json(&json(r#"{"axis_inverse": 1}"#)));
    assert!(
        message.contains("y_axis_configs[0].axis_inverse"),
        "{message}"
    );
}
