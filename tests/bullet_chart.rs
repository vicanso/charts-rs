//! Measures against their targets, a row each: a bar, a target line and the
//! bands behind them.
mod common;

use charts_rs::{
    Bullet, BulletChart, ChildChart, Color, MultiChart, NIL_VALUE, measure_text_width_family,
};

fn attr<'a>(tag: &'a str, name: &str) -> Option<&'a str> {
    let key = format!(" {name}=\"");
    let start = tag.find(&key)? + key.len();
    Some(&tag[start..start + tag[start..].find('"')?])
}

fn number(tag: &str, name: &str) -> f32 {
    attr(tag, name).unwrap().parse().unwrap()
}

/// A rect as it is drawn: where it is, and what it is filled with.
#[derive(Debug, Clone, PartialEq)]
struct Shape {
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    fill: String,
    opacity: Option<f32>,
    tag: String,
}

/// The rects after the background: bands, then the bar of each row.
fn rects(svg: &str) -> Vec<Shape> {
    svg.split("<rect")
        .skip(2)
        .map(|tag| {
            let tag = &tag[..tag.find('>').unwrap()];
            Shape {
                x: number(tag, "x"),
                y: number(tag, "y"),
                width: number(tag, "width"),
                height: number(tag, "height"),
                fill: attr(tag, "fill").unwrap().to_string(),
                opacity: attr(tag, "fill-opacity").map(|o| o.parse().unwrap()),
                tag: tag.to_string(),
            }
        })
        .collect()
}

/// The bars: the rects that say whose they are.
fn bars(svg: &str) -> Vec<Shape> {
    rects(svg)
        .into_iter()
        .filter(|rect| rect.tag.contains("data-name="))
        .collect()
}

/// The bands: the rects that do not.
fn bands(svg: &str) -> Vec<Shape> {
    rects(svg)
        .into_iter()
        .filter(|rect| !rect.tag.contains("data-name="))
        .collect()
}

/// The target lines: `(x, top, bottom, color)`.
fn targets(svg: &str) -> Vec<(f32, f32, f32, String)> {
    svg.split("<line")
        .skip(1)
        .map(|tag| &tag[..tag.find('>').unwrap()])
        .filter(|tag| attr(tag, "x1") == attr(tag, "x2") && attr(tag, "y1") != attr(tag, "y2"))
        .filter(|tag| number(tag, "y2") - number(tag, "y1") > 5.0)
        .map(|tag| {
            (
                number(tag, "x1"),
                number(tag, "y1"),
                number(tag, "y2"),
                attr(tag, "stroke").unwrap().to_string(),
            )
        })
        .collect()
}

/// The texts, each with its tag.
fn texts(svg: &str) -> Vec<(String, String)> {
    svg.split("<text")
        .skip(1)
        .map(|tag| {
            let head = tag[..tag.find('>').unwrap()].to_string();
            let body = tag[tag.find('>').unwrap() + 1..tag.find("</text>").unwrap()].trim();
            (body.to_string(), head)
        })
        .collect()
}

fn chart(json: &str) -> String {
    BulletChart::from_json(json).unwrap().svg().unwrap()
}

#[test]
fn bullet_chart_basic_json() {
    let chart = BulletChart::from_json(include_str!("../asset/bullet_chart/basic.json")).unwrap();
    assert_eq!(5, chart.bullets.len());
    common::assert_snapshot!("bullet_chart/basic_json.svg", chart.svg().unwrap());
}

#[test]
fn a_row_is_a_bar_a_target_and_bands() {
    let svg = chart(
        r#"{"bullets": [{"name": "Done", "value": 92, "target": 95, "ranges": [80, 90, 100], "max": 100}]}"#,
    );
    // Three bands from the start of the scale, each where the one before
    // it ends, as high as each other.
    let bands = bands(&svg);
    assert_eq!(3, bands.len());
    let (start, top, high) = (bands[0].x, bands[0].y, bands[0].height);
    let length = bands[2].x + bands[2].width - start;
    for (band, end) in bands.iter().zip([0.8, 0.9, 1.0]) {
        assert!(
            (band.x + band.width - (start + length * end)).abs() < 0.2,
            "{band:?}"
        );
        assert_eq!((top, high), (band.y, band.height));
    }
    for pair in bands.windows(2) {
        assert!((pair[0].x + pair[0].width - pair[1].x).abs() < 0.2);
    }
    // In shades of the text color, the first one the darkest.
    let shades: Vec<f32> = bands.iter().map(|band| band.opacity.unwrap()).collect();
    assert!(shades[0] > shades[1] && shades[1] > shades[2], "{shades:?}");
    assert!(bands.iter().all(|band| band.fill == "#6E7079"));

    // The bar from 0 to the value, a third as high as the bands and in the
    // middle of them, in the first color of the palette.
    let bar = &bars(&svg)[0];
    assert!((bar.x - start).abs() < 0.2 && (bar.width - length * 0.92).abs() < 0.2);
    assert!((bar.height - high / 3.0).abs() < 0.2);
    assert!((bar.y + bar.height / 2.0 - (top + high / 2.0)).abs() < 0.2);
    assert_eq!("#5470C6", bar.fill);
    assert_eq!(Some("92"), attr(&bar.tag, "data-value"));
    assert_eq!(Some("95"), attr(&bar.tag, "data-target"));

    // The target across the bar and most of the bands, in the color of the
    // title.
    let (x, from, to, color) = targets(&svg)[0].clone();
    assert!((x - (start + length * 0.95)).abs() < 0.2, "{x}");
    assert!(from > top && to < top + high && to - from > high * 0.6);
    assert_eq!("#464646", color);

    // The name to the left of the row, the scale below it, and no value
    // without `label_show`.
    let texts = texts(&svg);
    let name = texts.iter().find(|(text, _)| text == "Done").unwrap();
    assert_eq!(Some("end"), attr(&name.1, "text-anchor"));
    assert!(number(&name.1, "x") < start);
    assert!((number(&name.1, "y") - (top + high / 2.0)).abs() < 0.2);
    let ticks: Vec<&str> = texts
        .iter()
        .filter(|(_, tag)| number(tag, "y") > top + high)
        .map(|(text, _)| text.as_str())
        .collect();
    assert_eq!(vec!["0", "20", "40", "60", "80", "100"], ticks);
    assert!(!texts.iter().any(|(text, _)| text == "92"));

    // With it the value is to the right of the row, as it is formatted.
    let svg = chart(
        r#"{"label_show": true, "series_label_formatter": "{c}%",
            "bullets": [{"name": "Done", "value": 92, "target": 95, "max": 100}]}"#,
    );
    let written = self::texts(&svg);
    let label = written.iter().find(|(text, _)| text == "92%").unwrap();
    let one = self::bands(&svg);
    assert!(number(&label.1, "x") > one[0].x + one[0].width);
    // Without ranges there is one band, all along.
    assert_eq!(1, one.len());
}

/// The ticks below each row.
fn ticks(svg: &str) -> Vec<Vec<String>> {
    let bands = bands(svg);
    let mut rows: Vec<(f32, f32)> = vec![];
    for band in bands.iter() {
        if !rows.iter().any(|(top, _)| *top == band.y) {
            rows.push((band.y, band.y + band.height));
        }
    }
    rows.iter()
        .enumerate()
        .map(|(index, (_, bottom))| {
            let next = rows.get(index + 1).map(|(top, _)| *top).unwrap_or(f32::MAX);
            texts(svg)
                .into_iter()
                .filter(|(_, tag)| {
                    let y = number(tag, "y");
                    y > *bottom && y < next
                })
                .map(|(text, _)| text)
                .collect()
        })
        .collect()
}

#[test]
fn a_scale_has_round_ticks() {
    let scales = ticks(&chart(include_str!("../asset/bullet_chart/basic.json")));
    assert_eq!(
        vec![
            vec!["0", "20", "40", "60", "80", "100"],
            vec!["0", "1", "2", "3", "4", "5"],
            vec!["0", "5", "10", "15", "20", "25", "30"],
            vec!["0", "500", "1k", "1.5k", "2k", "2.5k"],
            vec!["0", "20", "40", "60", "80", "100"],
        ],
        scales
    );
    // Where it starts and where it ends, when they are given, and below 0
    // where a value is.
    let scales = ticks(&chart(
        r#"{"bullets": [
            {"name": "a", "value": 72, "min": 50, "max": 100},
            {"name": "b", "value": -12, "target": 30, "ranges": [0, 20, 40]},
            {"name": "c", "value": 0.42, "target": 0.5, "max": 1}
        ]}"#,
    ));
    assert_eq!(vec!["50", "60", "70", "80", "90", "100"], scales[0]);
    assert_eq!(vec!["-20", "-10", "0", "10", "20", "30", "40"], scales[1]);
    assert_eq!(vec!["0", "0.2", "0.4", "0.6", "0.8", "1"], scales[2]);
}

#[test]
fn rows_share_the_chart() {
    let json = |options: &str| {
        format!(
            r#"{{"height": 200{options}, "bullets": [
                {{"name": "a", "value": 3, "ranges": [2, 4]}},
                {{"name": "b", "value": 6, "ranges": [5, 8]}},
                {{"name": "c", "value": 1, "ranges": [1, 2]}}]}}"#
        )
    };
    let svg = chart(&json(""));
    let bars = bars(&svg);
    assert_eq!(3, bars.len());
    // As far apart as each other, and as high.
    let apart = bars[1].y - bars[0].y;
    assert!((bars[2].y - bars[1].y - apart).abs() < 0.2 && apart > 50.0);
    assert!(
        bars.iter()
            .all(|bar| (bar.height - bars[0].height).abs() < 0.01)
    );
    // Without their scales the bands have more room.
    let hidden = chart(&json(r#", "x_axis_hidden": true"#));
    assert!(ticks(&hidden).iter().all(|row| row.is_empty()));
    assert!(bands(&hidden)[0].height > bands(&svg)[0].height);
    // The bands are not higher than 40 pixels, however much room a row has.
    let tall = chart(r#"{"height": 900, "bullets": [{"name": "a", "value": 3}]}"#);
    assert_eq!(40.0, bands(&tall)[0].height);
}

#[test]
fn colors_of_the_bands_the_bar_and_the_target() {
    let svg = chart(
        r##"{"range_colors": ["#ee6666", "#fac858"], "target_color": "#123456",
            "bullets": [
                {"name": "a", "value": 3, "target": 4, "ranges": [2, 4, 6]},
                {"name": "b", "value": 5, "target": 4, "color": "#00aa00"}]}"##,
    );
    let fills: Vec<String> = bands(&svg).iter().map(|band| band.fill.clone()).collect();
    // From the first color on, and round again for a band past the last.
    assert_eq!(vec!["#EE6666", "#FAC858", "#EE6666", "#EE6666"], fills);
    let bars = bars(&svg);
    assert_eq!(
        ("#5470C6", "#00AA00"),
        (bars[0].fill.as_str(), bars[1].fill.as_str())
    );
    assert!(targets(&svg).iter().all(|(.., color)| color == "#123456"));
    // On a dark theme the target is in the light color of its title.
    let dark = chart(r#"{"theme": "dark", "bullets": [{"name": "a", "value": 3, "target": 4}]}"#);
    assert_eq!("#EEEEEE", targets(&dark)[0].3);
}

#[test]
fn what_is_not_on_the_scale() {
    let svg = chart(
        r#"{"bullets": [
            {"name": "none", "target": 4, "ranges": [2, 5]},
            {"name": "beyond", "value": 140, "target": 120, "max": 100},
            {"name": "null", "value": null}]}"#,
    );
    // A row without a value has its bands and its target, and no bar.
    let bars = bars(&svg);
    assert_eq!(
        vec!["beyond"],
        bars.iter()
            .map(|bar| attr(&bar.tag, "data-name").unwrap())
            .collect::<Vec<_>>()
    );
    assert_eq!(1, targets(&svg).len());
    // A value past the end of the scale reaches to its end; a target past
    // it is not drawn.
    let band = bands(&svg)
        .into_iter()
        .find(|band| (band.y - bars[0].y).abs() < band.height)
        .unwrap();
    assert!((bars[0].x + bars[0].width - (band.x + band.width)).abs() < 0.2);
    // A value that is none is no value.
    let mut chart = BulletChart::new(vec![("a", NIL_VALUE).into(), ("b", f32::NAN).into()]);
    chart.bullets[0].target = Some(f32::INFINITY);
    let svg = chart.svg().unwrap();
    assert!(self::bars(&svg).is_empty() && targets(&svg).is_empty());
    assert!(!svg.contains("NaN") && !svg.contains("inf"));
}

#[test]
fn a_long_name_is_cut_to_a_third_of_the_chart() {
    let name = "A name that is much longer than a third of a chart of 360 pixels";
    let svg = chart(&format!(
        r#"{{"width": 360, "bullets": [{{"name": "{name}", "value": 3}}]}}"#
    ));
    let (text, tag) = texts(&svg)
        .into_iter()
        .find(|(text, _)| text.starts_with("A name"))
        .unwrap();
    assert!(text.ends_with('…') && text.len() < name.len());
    let width = measure_text_width_family("Roboto", 14.0, &text)
        .unwrap()
        .width();
    assert!(width <= 350.0 / 3.0 + 0.1, "{width}");
    assert!(number(&tag, "x") - width >= -0.1);
}

#[test]
fn bullets_in_rust_with_tooltips_and_in_a_multi_chart() {
    let json = r##"{"tooltip_show": true, "bullets": [
        {"name": "CPU", "value": 72, "target": 80},
        {"name": "Memory", "value": 64, "target": 70, "ranges": [50, 80, 100], "min": 0, "max": 100, "color": "#ff0000"}]}"##;
    let from_json = BulletChart::from_json(json).unwrap();
    let mut chart = BulletChart::new(vec![
        ("CPU", 72.0, 80.0).into(),
        Bullet {
            name: "Memory".to_string(),
            value: 64.0,
            target: Some(70.0),
            ranges: vec![50.0, 80.0, 100.0],
            min: Some(0.0),
            max: Some(100.0),
            color: Some(Color::from("#ff0000")),
        },
    ]);
    chart.tooltip.show = true;
    assert_eq!(from_json.bullets, chart.bullets);
    let svg = chart.svg().unwrap();
    assert_eq!(from_json.svg().unwrap(), svg);
    // A bar tells its name and its value on hover.
    assert!(svg.contains("<title>CPU: 72</title>") && svg.contains("ct-tip"));

    // Of a multi chart.
    let multi = MultiChart::from_json(
        r#"{"child_charts": [{"type": "bullet", "height": 200, "bullets": [{"name": "a", "value": 3}]}]}"#,
    )
    .unwrap();
    assert_eq!(1, bars(&multi.svg().unwrap()).len());
    let mut multi = MultiChart::new();
    multi.add(ChildChart::Bullet(
        BulletChart::new(vec![("a", 3.0).into()]),
        None,
    ));
    assert_eq!(1, bars(&multi.svg().unwrap()).len());

    // Without bullets there is nothing but the background, and the empty
    // text where there is one.
    let empty = BulletChart::from_json(r#"{"empty_text": "No data"}"#)
        .unwrap()
        .svg()
        .unwrap();
    assert!(rects(&empty).is_empty() && empty.contains(">\nNo data\n</text>"));
}
