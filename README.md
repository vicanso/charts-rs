# charts-rs

[中文](./README-zh.md)

`charts-rs` is a charting library for rust. It's simple and fast.

[![Crates.io][crates-badge]][crates-url]
[![Apache licensed][apache-badge]][apache-url]
[![Build status](https://github.com/vicanso/charts-rs/actions/workflows/test.yml/badge.svg?branch=main)](https://github.com/vicanso/charts-rs/actions/workflows/test.yml)

[crates-badge]: https://img.shields.io/crates/v/charts-rs.svg
[crates-url]: https://crates.io/crates/charts-rs
[apache-badge]: https://img.shields.io/badge/license-apache2-blue.svg
[apache-url]: https://github.com/vicanso/charts-rs/blob/main/LICENSE

## Overview

`charts-rs` provides a straightforward approach to generating charts with support for multiple output formats including `svg`, `png`, `jpeg`, `webp`, and `avif`. The library offers ten distinct themes: `light`, `dark`, `grafana`, `ant`, `vintage`, `walden`, `westeros`, `chalk`, `shine`, and `shadcn`, with `light` as the default theme.

The library supports twenty-two chart types: `Bar`, `HorizontalBar`, `Line`, `Pie`, `Radar`, `Scatter`, `Candlestick`, `Table`, `Heatmap`, `Funnel`, `Waterfall`, `MultiChart`, `Calendar`, `Gauge`, `Treemap`, `BoxPlot`, `Sunburst`, `Sankey`, `Tree`, `Graph`, `Parallel`, and `ThemeRiver`. Drawing inspiration from `Apache ECharts`, `charts-rs` enables developers to create charts with similar functionality and appearance.

## Themes

[themes](./theme.md)

## Features

- Ten built-in themes; custom themes supported via `add_theme()`
- Custom font loading from ttf or otf files
- Advanced line chart features: smooth curves, area filling, mark points and mark lines
- Multiple legend styles across all charts: `round rect`, `circle`, and `rect`
- Dual y-axis support via `y_axis_configs` and `series.y_axis_index`
- Logarithmic scale support (`"log"`, `"log2"`, or `{"type":"log","base":N}`)
- Gradient fill for bars, areas, and pie slices (`Fill::LinearGradient`)
- Per-series mixed chart types (bar + line on the same chart)
- Series stacking, dash patterns, and per-bar custom colors
- SVG animation support (duration, easing, stagger) for bar, line, pie, sunburst, funnel, treemap, and sankey charts
- Null / missing data points via `Option<f32>` (`null` in JSON; legacy `NIL_VALUE` still accepted)
- JSON-based chart configuration for all chart types
- Multiple output formats: svg, png, jpeg, webp, avif
- Scaled image export via `svg_to_png_with_size` and equivalent functions
- Web-based JSON editor for interactive chart configuration and testing
- Validated JSON input: typos, wrong types and out-of-range values are errors
  with a hint, not silently ignored
- Legend placement (`legend_position`: top, bottom, left, right), per-series
  `smooth` / `fill` / `symbol` overrides, fixed-value mark lines
- Hover tooltips (`tooltip_show`) and `data-*` attributes on the data shapes
  of every chart, for interactive consumers

## Installation

Add `charts-rs` to your `Cargo.toml`:

```toml
[dependencies]
charts-rs = "1"
```

SVG output works with the default build. Raster export is opt-in, one
feature per format: `png`, `jpeg`, `webp` and `avif` enable `svg_to_png`,
`svg_to_jpeg`, … (and their `_with_size` variants). Pick only what you need —
`avif` in particular pulls in the heavyweight `rav1e` encoder:

```toml
[dependencies]
charts-rs = { version = "1", features = ["png"] }
```

`image-encoder` is the umbrella feature that turns on all four formats.

### Minimum supported Rust version

charts-rs 1.x builds with Rust 1.88 or newer (edition 2024). The MSRV may
rise in a minor release; the change is noted in the changelog.

## Demo

You can try to use the web demo page, it's simple and useful.

Charts Web Demo Page: [https://charts.npmtrend.com/](https://charts.npmtrend.com/)

Charts Web Source: [https://github.com/vicanso/charts-rs-web](https://github.com/vicanso/charts-rs-web)

<p align="center">
    <img src="https://raw.githubusercontent.com/vicanso/charts-rs/main/asset/image/charts-demo.png" alt="charts-rs">
</p>

## Mix line bar

<p align="center">
    <img src="https://raw.githubusercontent.com/vicanso/charts-rs/main/asset/image/mix-line-bar.png" alt="charts-rs">
</p>

## Horizontal bar

<p align="center">
    <img src="https://raw.githubusercontent.com/vicanso/charts-rs/main/asset/image/horizontal-bar.png" alt="charts-rs">
</p>

## Line

<p align="center">
    <img src="https://raw.githubusercontent.com/vicanso/charts-rs/main/asset/image/line.png" alt="charts-rs">
</p>

## Pie

<p align="center">
    <img src="https://raw.githubusercontent.com/vicanso/charts-rs/main/asset/image/pie.png" alt="charts-rs">
</p>

## Radar

<p align="center">
    <img src="https://raw.githubusercontent.com/vicanso/charts-rs/main/asset/image/radar.png" alt="charts-rs">
</p>

## Scatter

<p align="center">
    <img src="https://raw.githubusercontent.com/vicanso/charts-rs/main/asset/image/scatter.png" alt="charts-rs">
</p>

## Candlestick

<p align="center">
    <img src="https://raw.githubusercontent.com/vicanso/charts-rs/main/asset/image/candlestick.png" alt="charts-rs">
</p>

## Table

<p align="center">
    <img src="https://raw.githubusercontent.com/vicanso/charts-rs/main/asset/image/table.avif" alt="charts-rs">
</p>

## Heatmap

<p align="center">
    <img src="https://raw.githubusercontent.com/vicanso/charts-rs/main/asset/image/heatmap.png" alt="charts-rs">
</p>

## Funnel

<p align="center">
    <img src="https://raw.githubusercontent.com/vicanso/charts-rs/main/asset/image/funnel.png" alt="charts-rs">
</p>

## Waterfall

<p align="center">
    <img src="https://raw.githubusercontent.com/vicanso/charts-rs/main/asset/image/waterfall.png" alt="charts-rs">
</p>

## Calendar

<p align="center">
    <img src="https://raw.githubusercontent.com/vicanso/charts-rs/main/asset/image/calendar.png" alt="charts-rs">
</p>

## Gauge

<p align="center">
    <img src="https://raw.githubusercontent.com/vicanso/charts-rs/main/asset/image/gauge.png" alt="charts-rs">
</p>

## Treemap

<p align="center">
    <img src="https://raw.githubusercontent.com/vicanso/charts-rs/main/asset/image/treemap.png" alt="charts-rs">
</p>

## Box Plot

<p align="center">
    <img src="https://raw.githubusercontent.com/vicanso/charts-rs/main/asset/image/box-plot.png" alt="charts-rs">
</p>

## Sunburst

<p align="center">
    <img src="https://raw.githubusercontent.com/vicanso/charts-rs/main/asset/image/sunburst.png" alt="charts-rs">
</p>

## Multi Chart

<p align="center">
    <img src="https://raw.githubusercontent.com/vicanso/charts-rs/main/asset/image/multi-chart.webp" alt="charts-rs">
</p>

## Sankey

<p align="center">
    <img src="https://raw.githubusercontent.com/vicanso/charts-rs/main/asset/image/sankey.png" alt="charts-rs">
</p>

## Tree

<p align="center">
    <img src="https://raw.githubusercontent.com/vicanso/charts-rs/main/asset/image/tree.png" alt="charts-rs">
</p>

## Graph

<p align="center">
    <img src="https://raw.githubusercontent.com/vicanso/charts-rs/main/asset/image/graph.png" alt="charts-rs">
</p>

## Parallel

<p align="center">
    <img src="https://raw.githubusercontent.com/vicanso/charts-rs/main/asset/image/parallel.png" alt="charts-rs">
</p>

## Theme River

<p align="center">
    <img src="https://raw.githubusercontent.com/vicanso/charts-rs/main/asset/image/theme-river.png" alt="charts-rs">
</p>

## Example

Runnable examples live in [`examples/`](./examples); each writes an `svg` file:

```bash
cargo run --example bar       # basic bar chart
cargo run --example line      # smooth line with area fill and mark line / points
cargo run --example pie       # nightingale (rose) chart
cargo run --example sunburst  # sunburst with label formatter, ring thickness, animation
cargo run --example sankey     # sankey flow diagram (nodes auto-derived from links)
cargo run --example tree       # node-link tree with curved links (LR layout)
```

### New from option

```rust,no_run
use charts_rs::{
    BarChart, Box, SeriesCategory, THEME_GRAFANA, svg_to_png
};
let mut bar_chart = BarChart::new_with_theme(
    vec![
        ("Evaporation", vec![2.0, 4.9, 7.0, 23.2, 25.6, 76.7, 135.6]).into(),
        (
            "Precipitation",
            vec![2.6, 5.9, 9.0, 26.4, 28.7, 70.7, 175.6],
        )
            .into(),
        ("Temperature", vec![2.0, 2.2, 3.3, 4.5, 6.3, 10.2, 20.3]).into(),
    ],
    vec![
        "Mon".to_string(),
        "Tue".to_string(),
        "Wed".to_string(),
        "Thu".to_string(),
        "Fri".to_string(),
        "Sat".to_string(),
        "Sun".to_string(),
    ],
    THEME_GRAFANA,
);
bar_chart.title_text = "Mixed Line and Bar".to_string();
bar_chart.legend_margin = Some(Box {
    top: bar_chart.title_height,
    bottom: 5.0,
    ..Default::default()
});
bar_chart.series_list[2].category = Some(SeriesCategory::Line);
bar_chart.series_list[2].y_axis_index = 1;
bar_chart.series_list[2].label_show = true;

bar_chart
    .y_axis_configs
    .push(bar_chart.y_axis_configs[0].clone());
bar_chart.y_axis_configs[0].axis_formatter = Some("{c} ml".to_string());
bar_chart.y_axis_configs[1].axis_formatter = Some("{c} °C".to_string());

println!("{}", &bar_chart.svg().unwrap());
svg_to_png(&bar_chart.svg().unwrap()).unwrap();
```

### From json

```rust,no_run
use charts_rs::{BarChart, svg_to_png};
let bar_chart = BarChart::from_json(
    r###"{
        "width": 630,
        "height": 410,
        "margin": {
            "left": 10,
            "top": 5,
            "right": 10
        },
        "title_text": "Bar Chart",
        "title_font_color": "#345",
        "title_align": "right",
        "sub_title_text": "demo",
        "legend_align": "left",
        "series_list": [
            {
                "name": "Email",
                "label_show": true,
                "data": [120.0, 132.0, 101.0, 134.0, 90.0, 230.0, 210.0]
            },
            {
                "name": "Union Ads",
                "data": [220.0, 182.0, 191.0, 234.0, 290.0, 330.0, 310.0]
            }
        ],
        "x_axis_data": [
            "Mon",
            "Tue",
            "Wed",
            "Thu",
            "Fri",
            "Sat",
            "Sun"
        ]
    }"###,
).unwrap();
println!("{}", bar_chart.svg().unwrap());
svg_to_png(&bar_chart.svg().unwrap()).unwrap();
```

### Scaled image export

```rust,no_run
use charts_rs::{BarChart, svg_to_png_with_size};
let chart = BarChart::from_json(
    r###"{"series_list": [{"name": "Email", "data": [120, 132, 101]}], "x_axis_data": ["Mon", "Tue", "Wed"]}"###,
).unwrap();
let svg = chart.svg().unwrap();

// Scale to exactly 800×400
let png = svg_to_png_with_size(&svg, Some(800), Some(400)).unwrap();

// Scale to width 800, preserve aspect ratio
let png = svg_to_png_with_size(&svg, Some(800), None).unwrap();
```

### Null data points

Use `null` in JSON arrays to represent missing data; the chart skips rendering for those positions.

```json
{
  "series_list": [{
    "name": "Sales",
    "data": [120.0, null, 101.0, null, 90.0]
  }]
}
```

In Rust, build the series with `Option<f32>` values (`None` is a missing point):

```rust
use charts_rs::Series;
let series = Series::new_nullable(
    "Sales".to_string(),
    vec![Some(120.0), None, Some(101.0), None, Some(90.0)],
);
// or via the tuple conversion
let series: Series = ("Sales", vec![Some(120.0), None, Some(101.0)]).into();
```

The legacy `NIL_VALUE` sentinel (`Series::new` / `Vec<f32>`) still works and is treated as `None`.

### Label formatters

Formatters are supported in `series_label_formatter`, `axis_formatter`, and `value_formatter` (GaugeChart).

| Placeholder | Meaning |
|-------------|---------|
| `{c}` | Data value |
| `{a}` | Series name |
| `{b}` | Category (x-axis label) |
| `{d}` | Percentage (pie / funnel) |
| `{t}` | Thousands notation (1.2K, 5.6M) |

## Load more fonts

```rust,no_run
use charts_rs::add_fonts;
let buf = std::fs::read("path/to/font.ttf").unwrap();
add_fonts(&[&buf]).unwrap();
```

A `font_family` that is not registered is still written to the SVG (the
viewer resolves it), but text is measured with the default font.

## Compact output

`chart.compact = true` (JSON `"compact": true`) writes the SVG the way an
optimizer would: no whitespace, relative path data with `h`/`v` shorthands,
grid lines and ticks merged into single paths, shared attributes hoisted onto
groups, defaults and long hex colors dropped. The picture is the same and the
file is typically 20–30% smaller. The same rewrite is available as
`charts_rs::compact_svg(&svg)` for output you already have.

## Snapshot tests

The SVG output is covered by snapshot tests under `asset/`. After an
intentional rendering change, regenerate them and review the diff:

```bash
UPDATE_SNAPSHOTS=1 cargo test --features image-encoder
git diff --stat asset/
```

## License

This project is licensed under the [Apache-2.0 license].

[Apache-2.0 license]: https://github.com/vicanso/charts-rs/blob/main/LICENSE
