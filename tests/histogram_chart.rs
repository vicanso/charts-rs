mod common;

use charts_rs::{Error, HistogramChart, MultiChart, Series};

fn attr(tag: &str, name: &str) -> f32 {
    let key = format!(" {name}=\"");
    let start = tag.find(&key).map(|i| i + key.len()).unwrap();
    let end = start + tag[start..].find('"').unwrap();
    tag[start..end].parse().unwrap()
}

/// `(x, y, width, height)` of every `<rect>` after the background.
fn rects(svg: &str) -> Vec<(f32, f32, f32, f32)> {
    svg.split("<rect")
        .skip(2)
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

/// The `data-value` of every bar, in drawing order.
fn bar_values(svg: &str) -> Vec<f32> {
    svg.split("<rect")
        .skip(2)
        .map(|t| attr(t, "data-value"))
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

fn sample() -> Vec<f32> {
    vec![1.0, 2.0, 2.0, 3.0, 3.0, 3.0, 4.0, 4.0, 4.0, 4.0, 10.0]
}

#[test]
fn every_value_is_counted_once() {
    let mut chart = HistogramChart::new(vec![("A", sample()).into()]);
    chart.bin_width = Some(2.0);
    let svg = chart.svg().unwrap();
    // Bins 0–2, 2–4, 4–6, 6–8 (empty, so no bar) and 8–10, which keeps the
    // value on its right edge.
    assert_eq!(vec![1.0, 5.0, 4.0, 1.0], bar_values(&svg));
    assert!(
        svg.contains("data-from=\"2\" data-to=\"4\" data-value=\"5\""),
        "{svg}"
    );
    assert!(svg.contains("data-from=\"8\" data-to=\"10\""));
    let bars = rects(&svg);
    // Heights follow the counts, bars are equally wide and one bin apart.
    assert!((bars[1].3 / bars[0].3 - 5.0).abs() < 0.05, "{bars:?}");
    assert!(
        bars.iter()
            .all(|b| (b.2 - bars[0].2).abs() < 0.2 && b.2 > 50.0)
    );
    let bin = bars[1].0 - bars[0].0;
    assert!(
        ((bars[3].0 - bars[2].0) - 2.0 * bin).abs() < 0.3,
        "the empty bin leaves a gap"
    );
    // Every bin edge is a tick.
    let labels = texts(&svg);
    for edge in ["0", "2", "4", "6", "8", "10"] {
        assert!(labels.contains(&edge), "{edge} in {labels:?}");
    }

    // An explicit number of bins splits the range evenly: 1..10 in 3.
    chart.bin_width = None;
    chart.bin_count = 3;
    let svg = chart.svg().unwrap();
    assert_eq!(vec![6.0, 4.0, 1.0], bar_values(&svg));
    assert!(svg.contains("data-from=\"1\" data-to=\"4\""));

    // By default the bins are chosen from the sample, with round edges.
    chart.bin_count = 0;
    let svg = chart.svg().unwrap();
    assert_eq!(11.0, bar_values(&svg).iter().sum::<f32>());
    assert!(svg.contains("data-from=\"0\" data-to=\"2\""), "{svg}");

    // A fixed range leaves out what lies beyond it.
    chart.x_axis_min = Some(0.0);
    chart.x_axis_max = Some(6.0);
    chart.bin_width = Some(3.0);
    assert_eq!(vec![3.0, 7.0], bar_values(&chart.svg().unwrap()));
}

#[test]
fn percent_shows_the_share_of_each_sample() {
    let mut chart = HistogramChart::new(vec![
        ("A", vec![1.0, 1.0, 1.0, 9.0]).into(),
        ("B", vec![1.0, 9.0]).into(),
    ]);
    chart.bin_count = 2;
    chart.percent = true;
    chart.series_list[0].label_show = true;
    let svg = chart.svg().unwrap();
    assert_eq!(vec![75.0, 25.0, 50.0, 50.0], bar_values(&svg));
    let labels = texts(&svg);
    assert!(
        labels.contains(&"75%") && labels.contains(&"25%"),
        "{labels:?}"
    );
    // The axis reads as percentages too.
    assert!(
        labels
            .iter()
            .any(|l| l.ends_with('%') && *l != "75%" && *l != "25%")
    );
    // A format of its own wins.
    chart.y_axis_configs[0].axis_formatter = Some("{c} pct".to_string());
    assert!(
        texts(&chart.svg().unwrap())
            .iter()
            .any(|l| l.ends_with(" pct"))
    );
}

#[test]
fn several_samples_overlap_or_stack() {
    let make = || {
        let mut chart = HistogramChart::new(vec![
            ("A", vec![1.0, 1.0, 5.0]).into(),
            ("B", vec![1.0, 5.0, 5.0, 5.0]).into(),
        ]);
        chart.bin_count = 2;
        chart
    };
    // Overlaid: both start from the axis and show through each other.
    let svg = make().svg().unwrap();
    let bars = rects(&svg);
    assert_eq!(4, bars.len());
    let bottom = |b: &(f32, f32, f32, f32)| b.1 + b.3;
    assert!((bottom(&bars[0]) - bottom(&bars[2])).abs() < 0.2);
    assert_eq!(4, svg.matches("fill-opacity=\"0.59\"").count(), "{svg}");
    // A single sample is opaque.
    let single = HistogramChart::new(vec![("A", vec![1.0, 2.0]).into()]);
    assert!(!single.svg().unwrap().contains("fill-opacity"));

    // Stacked: B sits on top of A, and the axis covers the total.
    let mut chart = make();
    for series in chart.series_list.iter_mut() {
        series.stack = Some("all".to_string());
    }
    let svg = chart.svg().unwrap();
    let bars = rects(&svg);
    assert!((bottom(&bars[2]) - bars[0].1).abs() < 0.2, "{bars:?}");
    assert!((bottom(&bars[3]) - bars[1].1).abs() < 0.2, "{bars:?}");
    assert!(!svg.contains("fill-opacity"));
    assert!(bars.iter().all(|b| b.1 >= 0.0 && b.3 > 0.0));
}

#[test]
fn labels_tooltips_and_animation() {
    let mut chart = HistogramChart::new(vec![("Heights", sample()).into()]);
    chart.bin_width = Some(2.0);
    chart.tooltip_show = true;
    chart.series_list[0].label_show = true;
    chart.animation = Some(charts_rs::AnimationConfig::default());
    let svg = chart.svg().unwrap();
    assert!(svg.contains("<title>Heights: 2 – 4: 5</title>"), "{svg}");
    assert!(svg.contains("class=\"bar-anim ct-trigger\"") && svg.contains("bar-grow"));
    assert_eq!(4, svg.matches("class=\"ct-tip\"").count());
    // One count label per bar, on top of the tooltip texts.
    assert_eq!(1, texts(&svg).iter().filter(|l| **l == "5").count());
}

#[test]
fn degenerate_samples() {
    for data in [
        vec![],
        vec![7.0],
        vec![7.0, 7.0, 7.0],
        vec![f32::NAN, f32::INFINITY],
    ] {
        let svg = HistogramChart::new(vec![("A", data.clone()).into()])
            .svg()
            .unwrap();
        for bad in ["NaN", "inf", "width=\"-", "height=\"-"] {
            assert!(!svg.contains(bad), "{bad} for {data:?}");
        }
    }
    // Missing values are left out of the sample.
    let chart = HistogramChart::new(vec![Series::new_nullable(
        "A".to_string(),
        vec![Some(1.0), None, Some(2.0)],
    )]);
    assert_eq!(2.0, bar_values(&chart.svg().unwrap()).iter().sum::<f32>());
    // No series at all, with a placeholder.
    let mut chart = HistogramChart::new(vec![]);
    chart.empty_text = Some("No data".to_string());
    assert!(chart.svg().unwrap().contains("No data"));
}

#[test]
fn from_json() {
    let chart = HistogramChart::from_json(
        r#"{"bin_count": 4, "bin_width": 2.5, "percent": true, "bar_gap": 0,
            "series_list": [{"name": "A", "data": [1, 2, null, 4]}]}"#,
    )
    .unwrap();
    assert_eq!(4, chart.bin_count);
    assert_eq!(Some(2.5), chart.bin_width);
    assert!(chart.percent);
    assert_eq!(Some(0.0), chart.bar_gap);
    // The width wins over the count: bins 0–2.5 and 2.5–5; the null is
    // not part of the sample.
    let values = bar_values(&chart.svg().unwrap());
    assert_eq!(2, values.len());
    assert!(
        (values[0] - 66.7).abs() < 0.1 && (values[1] - 33.3).abs() < 0.1,
        "{values:?}"
    );

    assert!(err(HistogramChart::from_json(r#"{"bin_width": 0}"#)).contains("bin_width"));
    assert!(err(HistogramChart::from_json(r#"{"bin_count": 5000}"#)).contains("bin_count"));
    let e = err(HistogramChart::from_json(r#"{"bins": 5}"#));
    assert!(e.contains("unknown field `bins`"), "{e}");

    // As a child of a multi chart.
    let multi = MultiChart::from_json(
        r#"{"child_charts": [{"type": "histogram", "bin_count": 2, "series_list": [{"name": "A", "data": [1, 2, 3]}]}]}"#,
    )
    .unwrap();
    assert!(multi.svg().unwrap().contains("data-from=\"1\""));
}

#[test]
fn histogram_chart_basic_json() {
    let chart =
        HistogramChart::from_json(include_str!("../asset/histogram_chart/basic.json")).unwrap();
    common::assert_snapshot!("histogram_chart/basic_json.svg", chart.svg().unwrap());
}

#[test]
fn histogram_chart_overlay_json() {
    let chart =
        HistogramChart::from_json(include_str!("../asset/histogram_chart/overlay.json")).unwrap();
    common::assert_snapshot!("histogram_chart/overlay_json.svg", chart.svg().unwrap());
}
