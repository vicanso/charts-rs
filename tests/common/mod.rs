//! Geometry checks shared by the bar chart tests.
#![allow(dead_code)]

fn attr(tag: &str, name: &str) -> f32 {
    let key = format!(" {name}=\"");
    let start = tag.find(&key).map(|i| i + key.len()).unwrap();
    let end = start + tag[start..].find('"').unwrap();
    tag[start..end].parse().unwrap()
}

/// `(x1, y1, x2, y2)` of every `<line>`.
fn lines(svg: &str) -> Vec<(f32, f32, f32, f32)> {
    svg.split("<line")
        .skip(1)
        .map(|t| (attr(t, "x1"), attr(t, "y1"), attr(t, "x2"), attr(t, "y2")))
        .collect()
}

/// `(x, y, width, height)` of every `<rect>` after the background.
fn bar_rects(svg: &str) -> Vec<(f32, f32, f32, f32)> {
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

// Coordinates are written with one decimal.
const EPSILON: f32 = 0.15;

/// Asserts that every bar of a vertical bar chart has a positive size and lies
/// inside its own category band. The plot's horizontal extent comes from the
/// grid lines; bars are written series by series, `categories` per series.
pub fn assert_bars_in_x_bands(svg: &str, categories: usize) {
    let grid: Vec<_> = lines(svg)
        .into_iter()
        .filter(|l| l.1 == l.3 && l.2 - l.0 > 50.0)
        .collect();
    let left = grid.iter().map(|l| l.0).fold(f32::MAX, f32::min);
    let right = grid.iter().map(|l| l.2).fold(f32::MIN, f32::max);
    let unit = (right - left) / categories as f32;
    for (k, (x, _, width, _)) in bar_rects(svg).into_iter().enumerate() {
        let i = k % categories;
        let band = (left + unit * i as f32, left + unit * (i + 1) as f32);
        assert!(width > 0.0, "bar {k}: width {width}");
        assert!(
            x >= band.0 - EPSILON && x + width <= band.1 + EPSILON,
            "bar {k} spans {x}..{} outside its band {}..{}",
            x + width,
            band.0,
            band.1
        );
    }
}

/// Asserts that every bar of a horizontal bar chart has a positive size and
/// lies inside its own category band. The plot's vertical extent comes from
/// the axis and grid lines; the first category is drawn at the bottom.
pub fn assert_bars_in_y_bands(svg: &str, categories: usize) {
    let grid: Vec<_> = lines(svg)
        .into_iter()
        .filter(|l| l.0 == l.2 && (l.3 - l.1).abs() > 50.0)
        .collect();
    let top = grid.iter().map(|l| l.1.min(l.3)).fold(f32::MAX, f32::min);
    let bottom = grid.iter().map(|l| l.1.max(l.3)).fold(f32::MIN, f32::max);
    let unit = (bottom - top) / categories as f32;
    for (k, (_, y, _, height)) in bar_rects(svg).into_iter().enumerate() {
        let row = categories - 1 - k % categories;
        let band = (top + unit * row as f32, top + unit * (row + 1) as f32);
        assert!(height > 0.0, "bar {k}: height {height}");
        assert!(
            y >= band.0 - EPSILON && y + height <= band.1 + EPSILON,
            "bar {k} spans {y}..{} outside its band {}..{}",
            y + height,
            band.0,
            band.1
        );
    }
}

/// Chart JSON with `categories` categories and `series` series of made-up values.
pub fn dense_json(width: u32, height: u32, categories: usize, series: usize) -> String {
    let series_list: Vec<String> = (0..series)
        .map(|s| {
            let data: Vec<String> = (0..categories)
                .map(|i| ((i * 7 + s * 3) % 20 + 5).to_string())
                .collect();
            format!(r#"{{"name":"s{s}","data":[{}]}}"#, data.join(","))
        })
        .collect();
    let x_axis_data: Vec<String> = (0..categories).map(|i| format!("\"{i}\"")).collect();
    format!(
        r#"{{"width":{width},"height":{height},"series_list":[{}],"x_axis_data":[{}]}}"#,
        series_list.join(","),
        x_axis_data.join(",")
    )
}
