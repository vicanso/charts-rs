//! Stacks shown as shares: every stack adds up to 100%.
mod common;

use charts_rs::{BarChart, Error, HorizontalBarChart, LineChart, PolarBarChart, Position, Series};

fn attr<'a>(tag: &'a str, name: &str) -> &'a str {
    let key = format!(" {name}=\"");
    let start = tag.find(&key).map(|i| i + key.len()).unwrap();
    let end = start + tag[start..].find('"').unwrap();
    &tag[start..end]
}

fn number(tag: &str, name: &str) -> f32 {
    attr(tag, name).parse().unwrap()
}

/// The bars of the svg: the rects that carry a series.
fn bars(svg: &str) -> Vec<&str> {
    svg.split("<rect")
        .skip(1)
        .filter(|t| t.contains("data-series="))
        .collect()
}

/// The value every bar (or polar bar) stands for.
fn values(svg: &str) -> Vec<&str> {
    svg.split('<')
        .filter(|t| t.contains("data-series="))
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

fn categories(count: usize) -> Vec<String> {
    (0..count).map(|i| format!("c{i}")).collect()
}

fn stacked(name: &str, data: Vec<f32>, stack: &str) -> Series {
    let mut series: Series = (name, data).into();
    series.stack = Some(stack.to_string());
    series
}

fn chart(series_list: Vec<Series>) -> BarChart {
    let count = series_list.iter().map(|s| s.data.len()).max().unwrap_or(0);
    let mut chart = BarChart::new(series_list, categories(count));
    chart.legend.show = Some(false);
    chart.stack_percent = true;
    chart
}

#[test]
fn stack_percent_snapshot() {
    let chart = BarChart::from_json(include_str!("../asset/bar_chart/stack_percent.json")).unwrap();
    assert!(chart.stack_percent);
    common::assert_snapshot!("bar_chart/stack_percent_json.svg", chart.svg().unwrap());
}

#[test]
fn shares_add_up_to_the_whole() {
    let chart = chart(vec![
        stacked("A", vec![10.0, 30.0], "s"),
        stacked("B", vec![30.0, 10.0], "s"),
    ]);
    let svg = chart.svg().unwrap();
    assert_eq!(vec!["25", "75", "75", "25"], values(&svg));

    // B stands on A; together they fill the plot from 0% to 100%.
    let bars = bars(&svg);
    let (a, b) = (bars[0], bars[2]);
    let height = number(a, "height") + number(b, "height");
    assert!((number(b, "height") / number(a, "height") - 3.0).abs() < 0.05);
    assert!((number(a, "y") - (number(b, "y") + number(b, "height"))).abs() < 0.1);
    let top = number(b, "y");
    for pair in [(bars[0], bars[2]), (bars[1], bars[3])] {
        assert!((number(pair.1, "y") - top).abs() < 0.1);
        let whole = number(pair.0, "height") + number(pair.1, "height");
        assert!((whole - height).abs() < 0.2);
    }

    // The axis reads in percent, on whole numbers, up to 100.
    let texts = texts(&svg);
    for label in ["0%", "20%", "40%", "60%", "80%", "100%"] {
        assert!(texts.contains(&label), "{label} in {texts:?}");
    }
    assert!(!texts.iter().any(|t| t.contains("120")));

    // The chart is drawn from its shares, not turned into them.
    assert_eq!(Some(10.0), chart.series_list[0].data[0]);
    assert!(chart.y_axis_configs[0].max.is_none());
    assert_eq!(svg, chart.svg().unwrap());
}

#[test]
fn each_stack_is_its_own_whole() {
    let mut right = stacked("D", vec![5.0, 5.0], "t");
    right.y_axis_index = 1;
    let mut chart = chart(vec![
        stacked("A", vec![10.0, 30.0], "s"),
        stacked("B", vec![30.0, 10.0], "s"),
        stacked("C", vec![2.0, 6.0], "t"),
        // The same name on the other axis is another stack.
        right,
        // Not in a stack: its values stay what they are.
        ("E", vec![7.0, 9.0]).into(),
    ]);
    chart.y_axis_configs.push(chart.y_axis_configs[0].clone());
    let svg = chart.svg().unwrap();
    assert_eq!(
        vec!["25", "75", "75", "25", "100", "100", "100", "100", "7", "9"],
        values(&svg)
    );
}

#[test]
fn gaps_zeros_and_negative_values() {
    let mut chart = chart(vec![
        Series::new_nullable(
            "A".to_string(),
            vec![Some(10.0), None, Some(0.0), Some(-10.0)],
        ),
        Series::new_nullable(
            "B".to_string(),
            vec![Some(30.0), Some(8.0), Some(0.0), Some(30.0), Some(4.0)],
        ),
    ]);
    for series in chart.series_list.iter_mut() {
        series.stack = Some("s".to_string());
    }
    let svg = chart.svg().unwrap();
    assert!(!svg.contains("NaN") && !svg.contains("inf"));
    // A missing value takes no share; a stack of nothing has none to give;
    // a negative value takes its share below 0.
    assert_eq!(
        vec!["25", "0", "-25", "75", "100", "0", "75", "100"],
        values(&svg)
    );
    // With values below 0 the axis is left to the data.
    assert!(
        texts(&svg)
            .iter()
            .any(|t| t.starts_with('-') && t.ends_with('%'))
    );

    // A series placed far off the chart does not blow anything up.
    chart.series_list[0].start_index = usize::MAX;
    assert!(chart.svg().unwrap().starts_with("<svg"));
}

#[test]
fn axis_and_labels_can_be_set() {
    let series = || {
        let mut list = vec![
            stacked("A", vec![10.0, 30.0], "s"),
            stacked("B", vec![30.0, 10.0], "s"),
        ];
        for series in list.iter_mut() {
            series.label_show = true;
        }
        list
    };
    // The labels read in percent, in the middle of their bars.
    let svg = chart(series()).svg().unwrap();
    let bars_of = bars(&svg);
    let label = svg.split("<text").find(|t| t.contains("\n25%\n")).unwrap();
    let (top, height) = (number(bars_of[0], "y"), number(bars_of[0], "height"));
    let baseline = number(label, "y") + number(label, "dy");
    assert!(
        baseline > top + height * 0.4 && baseline < top + height * 0.8,
        "{baseline}"
    );
    assert_eq!(2, texts(&svg).iter().filter(|t| **t == "75%").count());

    // A format, a position and an axis of one's own are kept.
    let mut chart = chart(series());
    chart.series.label.formatter = "{a} {c}".to_string();
    chart.series_label_position = Some(Position::Top);
    chart.y_axis_configs[0].formatter = Some("{c} pct".to_string());
    chart.y_axis_configs[0].split_number = 4;
    let svg = chart.svg().unwrap();
    let texts = texts(&svg);
    assert!(
        texts.contains(&"A 25") && texts.contains(&"B 75"),
        "{texts:?}"
    );
    assert!(
        texts.contains(&"25 pct") && texts.contains(&"100 pct"),
        "{texts:?}"
    );
    let label = svg.split("<text").find(|t| t.contains("\nA 25\n")).unwrap();
    assert!(number(label, "y") <= number(bars(&svg)[0], "y"));
    // A fixed end of the axis is not moved to 100.
    chart.y_axis_configs[0].max = Some(200.0);
    assert!(self::texts(&chart.svg().unwrap()).contains(&"200 pct"));
}

#[test]
fn labels_inside_bars_without_shares() {
    let mut series: Series = ("A", vec![10.0, 30.0]).into();
    series.label_show = true;
    let mut chart = BarChart::new(vec![series], categories(2));
    chart.legend.show = Some(false);
    let above = chart.svg().unwrap();
    chart.series_label_position = Some(Position::Inside);
    let inside = chart.svg().unwrap();
    let place = |svg: &str| -> (f32, f32, f32) {
        let label = svg
            .split("<text")
            .find(|t| t.contains("\n30\n") && t.contains(" dy="))
            .unwrap();
        let bar = bars(svg)[1];
        (
            number(label, "y") + number(label, "dy"),
            number(bar, "y"),
            number(bar, "height"),
        )
    };
    let (baseline, top, _) = place(&above);
    assert!(baseline < top);
    let (baseline, top, height) = place(&inside);
    assert!(
        (baseline - (top + height / 2.0)).abs() < 8.0,
        "{baseline} {top} {height}"
    );
}

#[test]
fn other_charts_that_stack() {
    let series = || {
        vec![
            stacked("A", vec![10.0, 30.0], "s"),
            stacked("B", vec![30.0, 10.0], "s"),
        ]
    };
    // Horizontal bars: side by side they fill the row.
    let mut chart = HorizontalBarChart::new(series(), categories(2));
    chart.legend.show = Some(false);
    let plain = chart.svg().unwrap();
    chart.stack_percent = true;
    let svg = chart.svg().unwrap();
    assert_eq!(vec!["25", "75", "75", "25"], values(&svg));
    assert_ne!(plain, svg);
    let bars_of = bars(&svg);
    let row = number(bars_of[0], "width") + number(bars_of[2], "width");
    let other = number(bars_of[1], "width") + number(bars_of[3], "width");
    assert!((row - other).abs() < 0.2 && row > 300.0, "{row} {other}");
    assert!(texts(&svg).contains(&"100%") || texts(&svg).contains(&"80%"));

    // Lines: the top of the stack runs along 100%.
    let mut chart = LineChart::new(series(), categories(2));
    chart.legend.show = Some(false);
    chart.stack_percent = true;
    chart.tooltip.show = true;
    let svg = chart.svg().unwrap();
    assert!(svg.contains("<title>A: 25%</title>") && svg.contains("<title>B: 75%</title>"));
    let top: Vec<f32> = svg
        .split("<circle")
        .skip(1)
        .filter(|t| t.contains("data-series=\"B\""))
        .map(|t| number(t, "cy"))
        .collect();
    assert_eq!(2, top.len());
    assert_eq!(top[0], top[1]);

    // Polar bars.
    let mut chart = PolarBarChart::new(series(), categories(2));
    chart.stack_percent = true;
    assert_eq!(vec!["25", "75", "75", "25"], values(&chart.svg().unwrap()));
}

#[test]
fn stack_percent_from_json() {
    let json = |extra: &str| {
        format!(
            r#"{{"legend_show": false, "x_axis_data": ["c0", "c1"], "series_list": [
                {{"name": "A", "stack": "s", "data": [10, 30]}},
                {{"name": "B", "stack": "s", "data": [30, 10]}}
            ]{extra}}}"#
        )
    };
    let plain = BarChart::from_json(&json("")).unwrap();
    assert!(!plain.stack_percent);
    let off = BarChart::from_json(&json(r#", "stack_percent": false"#)).unwrap();
    assert_eq!(plain.svg().unwrap(), off.svg().unwrap());

    let on = BarChart::from_json(&json(r#", "stack_percent": true"#)).unwrap();
    assert!(on.stack_percent);
    let built = chart(vec![
        stacked("A", vec![10.0, 30.0], "s"),
        stacked("B", vec![30.0, 10.0], "s"),
    ]);
    assert_eq!(built.svg().unwrap(), on.svg().unwrap());

    let position = BarChart::from_json(&json(r#", "series_label_position": "Inside""#)).unwrap();
    assert_eq!(Some(Position::Inside), position.series_label_position);

    let message = err(BarChart::from_json(&json(r#", "stack_percent": "yes""#)));
    assert!(message.contains("stack_percent"), "{message}");
    let message = err(BarChart::from_json(&json(
        r#", "series_label_position": "left""#,
    )));
    assert!(message.contains("top, inside"), "{message}");
}
