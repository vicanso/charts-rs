# Guide

[中文](./guide-zh.md)

How to build charts with `charts-rs`: the two ways in, the options that take
a word of explanation, fonts and output. Every JSON key is listed in the
[JSON reference](./json.md), and the charts are shown in the
[gallery](./gallery.md).

- [Building a chart](#building-a-chart): [in Rust](#in-rust),
  [from JSON](#from-json), [the same options on both paths](#options-in-rust-and-in-json)
- [Output](#output): [images](#images), [compact SVG](#compact-svg)
- [Data and labels](#data-and-labels): [null points](#null-data-points),
  [label formatters](#label-formatters), [overlapping labels](#overlapping-data-labels)
- [Axes](#axes): [value and time x axes](#value-and-time-x-axes), [titles](#axis-titles)
- [Charts](#charts): [band around a line](#band-around-a-line),
  [bubble](#bubble-chart), [histogram](#histogram), [polar bar](#polar-bar-chart),
  [chord](#chord-chart), [gantt](#gantt-chart), [map](#map-chart)
- [Fonts](#fonts)
- [Development](#development)

## Building a chart

### In Rust

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
bar_chart.title.text = "Mixed Line and Bar".to_string();
bar_chart.legend.margin = Some(Box {
    top: bar_chart.title.height,
    bottom: 5.0,
    ..Default::default()
});
bar_chart.series_list[2].category = Some(SeriesCategory::Line);
bar_chart.series_list[2].y_axis_index = 1;
bar_chart.series_list[2].label_show = true;

bar_chart
    .y_axis_configs
    .push(bar_chart.y_axis_configs[0].clone());
bar_chart.y_axis_configs[0].formatter = Some("{c} ml".to_string());
bar_chart.y_axis_configs[1].formatter = Some("{c} °C".to_string());

println!("{}", &bar_chart.svg().unwrap());
svg_to_png(&bar_chart.svg().unwrap()).unwrap();
```

### From JSON

Every key such a JSON document may have, for every chart — with its type,
its default and what it does — is listed in the
[JSON options reference](./json.md).

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

### Options in Rust and in JSON

The options are the same on both paths, and this README names them by their
JSON key. In Rust the shared ones are grouped by what they belong to —
`title`, `sub_title`, `legend`, `x_axis`, `grid`, `series`, `tooltip` — and a
JSON key is the path of the field joined with `_`:

| JSON | Rust |
|------|------|
| `"title_font_size": 18` | `chart.title.font.size = 18.0` |
| `"legend_position": "bottom"` | `chart.legend.position = Some(Position::Bottom)` |
| `"x_axis_data": [...]` | `chart.x_axis.data = vec![...]` |
| `"series_label_font_color": "#333"` | `chart.series.label.font.color = "#333".into()` |
| `"tooltip_show": true` | `chart.tooltip.show = true` |

The exceptions: `x_boundary_gap` is `x_axis.boundary_gap`, `x_axis_type` is
`x_axis.kind`, the `axis_` of the keys of a y axis is left out (`"axis_min"`
is `y_axis_configs[0].min`), and the `series` object of a heatmap is
`heatmap_series` (`series` being the options of the series, on every chart).
Every JSON key is described in the [JSON reference](./json.md).

### Runnable examples

Runnable examples live in [`examples/`](../examples); each writes an `svg` file:

```bash
cargo run --example bar       # basic bar chart
cargo run --example line      # smooth line with area fill and mark line / points
cargo run --example pie       # nightingale (rose) chart
cargo run --example sunburst  # sunburst with label formatter, ring thickness, animation
cargo run --example sankey     # sankey flow diagram (nodes auto-derived from links)
cargo run --example tree       # node-link tree with curved links (LR layout)
cargo run --example histogram  # distribution of a sample over equal-width bins
cargo run --example polar_bar  # polar bars: stacked around the circle, and rings with round caps
cargo run --example chord      # chord diagram of the flows between nodes (gradient ribbons)
cargo run --example gantt      # gantt chart: a project plan with progress, milestones and today
cargo run --example map        # map of regions (GeoJSON) colored by value, in classes
```

## Output

### Images

SVG output works with the default build. Raster export is opt-in, one
feature per format: `png`, `jpeg`, `webp` and `avif` enable `svg_to_png`,
`svg_to_jpeg`, … (and their `_with_size` variants). Pick only what you need —
`avif` in particular pulls in the heavyweight `rav1e` encoder:

```toml
[dependencies]
charts-rs = { version = "2", features = ["png"] }
```

`image-encoder` is the umbrella feature that turns on all four formats.

An image is as large as its chart, or as large as asked for:

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

### Compact SVG

`chart.compact = true` (JSON `"compact": true`) writes the SVG the way an
optimizer would: no whitespace, relative path data with `h`/`v` shorthands,
grid lines and ticks merged into single paths, shared attributes hoisted onto
groups, defaults and long hex colors dropped. The picture is the same and the
file is typically 20–30% smaller. The same rewrite is available as
`charts_rs::compact_svg(&svg)` for output you already have.

## Data and labels

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

### Overlapping data labels

`"series_label_hide_overlap": true` drops a data label that would be printed
over another one.

## Axes

### Value and time x axes

A line or bar chart spaces its points evenly by default (a category axis).
Give the points x values and the axis becomes continuous: every point sits at
its value, so irregular samples keep their real spacing.

```json
{
  "x_axis_title": "Time (UTC)",
  "y_axis_configs": [{ "axis_title": "Temperature (°C)" }],
  "x_axis_values": ["2024-03-01 00:00", "2024-03-01 00:20", "2024-03-01 04:00", "2024-03-01 12:00"],
  "series_list": [
    { "name": "Indoor", "data": [21.5, 21.8, 20.1, 24.6] },
    { "name": "Outdoor", "x_values": ["2024-03-01 02:00", "2024-03-01 11:00"], "data": [8.2, 14.9] }
  ]
}
```

- `x_axis_values` are shared by every series; a series sampled at other
  moments carries its own `x_values`.
- Date strings (`2024-03-01`, `2024-03-01 08:30`, `2024-03-01T08:30:00+08:00`)
  make it a time axis. Numbers are plain values, or unix seconds with
  `"x_axis_type": "time"`.
- Ticks fall on round values and round times. `x_axis_min` / `x_axis_max` fix
  the range, `x_axis_formatter` formats the labels (`"{c} km"`, or a pattern
  such as `"%m-%d %H:%M"` on a time axis) and `x_axis_time_offset` (minutes
  east of UTC) shows timestamps in local time.

In Rust, set `chart.x_axis.values` (`Vec<f64>`) and, for timestamps,
`chart.x_axis.kind = AxisType::Time`.

### Axis titles

`x_axis_title` is written below the x axis; `axis_title` in a y axis config
is written along that axis.

## Charts

What takes a word of explanation; the options of every chart are in the
[JSON reference](./json.md).

### Band around a line

A line series can carry a `band`: a lower and an upper bound for each point,
with the area between them filled in the color of the series — a confidence
interval, a forecast range, a daily minimum and maximum.

```json
{
  "x_axis_data": ["Mon", "Tue", "Wed", "Thu"],
  "series_list": [{
    "name": "Forecast",
    "data": [120, 132, 128, 141],
    "band": { "lower": [116, 126, 119, 129], "upper": [124, 138, 137, 153] }
  }]
}
```

```rust
use charts_rs::{Series, SeriesBand};
let mut series: Series = ("Forecast", vec![120.0, 132.0, 128.0, 141.0]).into();
series.band = Some(SeriesBand::new(
    vec![116.0, 126.0, 119.0, 129.0],
    vec![124.0, 138.0, 137.0, 153.0],
));
```

- The band follows its line: smooth when the series is smooth, on a category,
  value or time x axis, also for the line series of a bar chart. The y axis
  makes room for the bounds.
- A `null` bound leaves a gap in the band.
- A series with a band and no `data` draws the band alone (a min–max range).
- With `tooltip_show` the tooltip of a point tells its bounds, and the points
  carry `data-lower` / `data-upper`.
- The bounds are absolute values: a band is not stacked with its series.

### Bubble chart

A scatter chart with `"bubble": true` reads its series data as `[x, y, size]`
triples and scales each circle by its size (`bubble_min_size` /
`bubble_max_size` set the radius range):

```json
{
  "bubble": true,
  "series_list": [{ "name": "Asia", "data": [12, 77, 1400, 40, 84, 125] }]
}
```

### Histogram

A `HistogramChart` takes the raw sample as the series data and counts it
into equal-width bins:

```json
{
  "x_axis_title": "Height (cm)",
  "series_list": [{ "name": "Adults", "data": [162.4, 171.0, 168.3, 175.9, 158.2, 169.7] }]
}
```

- The bins are chosen from the sample (with round edges) by default;
  `bin_width` or `bin_count` set them, and `x_axis_min` / `x_axis_max` fix
  the range.
- `"percent": true` shows the share of each sample instead of a count.
- Several series share the bins and are drawn over each other; give them
  the same `stack` name to pile them up instead.

### Polar bar chart

A `PolarBarChart` draws the bars of a bar chart on polar axes. The categories
are `x_axis_data`; series sit side by side within a category, or pile up when
they share a `stack` name.

```json
{
  "inner_radius": 30,
  "x_axis_data": ["Jan", "Feb", "Mar", "Apr"],
  "series_list": [
    { "name": "North", "stack": "total", "data": [42, 38, 51, 64] },
    { "name": "South", "stack": "total", "data": [30, 34, 40, 58] }
  ]
}
```

- By default the categories go around the circle and the bars grow outwards.
  `"category_axis": "radius"` gives every category a ring instead, and the
  bars run around the circle: by default over three quarters of it for the
  largest value on the axis (`end_angle` changes that), with `"round_cap":
  true` for rounded ends.
- `start_angle` (degrees clockwise from 12 o'clock) turns the chart,
  `inner_radius` leaves a hole in the center, `radius` limits the size and
  `category_gap` (0 to 0.9, default 0.2) sets the space between categories.
- The value axis is the first of `y_axis_configs` (`axis_min`, `axis_max`,
  `axis_split_number`, `axis_formatter`); `x_axis_hidden` / `y_axis_hidden`
  hide the category and the value labels.
- `label_show`, `colors` (a color per bar), `tooltip_show` and `animation`
  work as in a bar chart.

In Rust: `PolarBarChart::new(series_list, x_axis_data)`, then
`chart.category_axis = PolarAxis::Radius`.

### Chord chart

A `ChordChart` shows who exchanges how much with whom: the nodes are arcs of
a circle, as long as the flows through them, and every link is a ribbon
across the circle, as wide at both ends as its value.

```json
{
  "link_gradient": true,
  "links": [
    { "source": "Asia", "target": "Europe", "value": 60 },
    { "source": "Asia", "target": "Americas", "value": 45 },
    { "source": "Europe", "target": "Americas", "value": 50 }
  ]
}
```

- The nodes are derived from the names in `links`, in first-seen order;
  list them in `nodes` (`{"name": ..., "color": ...}`) to fix their order or
  their colors. A link from a node to itself is allowed.
- A ribbon takes the color of its source node, or fades to the color of its
  target with `"link_gradient": true`; `link_opacity` (default 0.5) lets the
  ribbons show through each other.
- `node_width` (default 12) is the thickness of the ring, `node_gap`
  (default 3) the gap between nodes in degrees, `start_angle` turns the
  chart and `radius` limits its size.
- The labels are the node names; `series_label_formatter` can add the flow
  (`{c}`) or the share (`{d}`), e.g. `"{b} ({d})"`. `tooltip_show` and
  `animation` work as in the other charts.

In Rust: `ChordChart::new(vec![], vec![("Asia", "Europe", 60.0).into()])`.

### Gantt chart

A `GanttChart` draws tasks as bars along a time axis:

```json
{
  "now": "2024-03-19",
  "tasks": [
    { "name": "Research", "category": "Plan", "start": "2024-03-04", "end": "2024-03-08", "progress": 1 },
    { "name": "Design", "category": "Build", "start": "2024-03-07", "end": "2024-03-22", "progress": 0.6 },
    { "name": "Sign-off", "category": "Build", "start": "2024-03-22" }
  ]
}
```

- Every task has a row of its own, named after it; tasks with the same `row`
  share one (the bookings of a room, the flights at a gate) and are named on
  their bars.
- `category` colors the tasks and makes the legend, `progress` (0 to 1) shows
  how much is done, a task without an `end` is a milestone, and `now` marks
  a moment across the chart.
- The x axis is the time axis of the line chart: `x_axis_min` /
  `x_axis_max`, `x_axis_formatter` and `x_axis_time_offset` apply, and
  `"x_axis_type": "value"` makes it an axis of plain numbers.

In Rust: `GanttChart::new(vec![("Research", start, end).into()])`, with the
times as unix seconds.

### Map chart

A `MapChart` colors the regions of a map by their values. The regions are
GeoJSON that comes with the options — the library has no map data of its own:

```json
{
  "label_show": true,
  "thresholds": [100, 300],
  "colors": ["#deebf7", "#9ecae1", "#3182bd"],
  "data": [["West", 80], ["East", 420]],
  "geo_json": { "type": "FeatureCollection", "features": [] }
}
```

- `geo_json` is a `FeatureCollection` of `Polygon` and `MultiPolygon`
  geometries (holes and islands included); `data` gives the value of a
  region by its name, the property `name` of its feature (or another one,
  with `name_property`).
- The scale is the one of the heatmap: from `min_color` to `max_color`,
  through several `colors`, or in classes (`steps`, `thresholds`); it is
  shown beside the map. A region without a value is drawn in `empty_color`.
- `projection` is `mercator` (the default) or `equirectangular`.

In Rust: `MapChart::new(MapRegion::from_geo_json(text, "name")?, data)`.

## Fonts

```rust,no_run
use charts_rs::add_fonts;
let buf = std::fs::read("path/to/font.ttf").unwrap();
add_fonts(&[&buf]).unwrap();
```

A font is known by the names of its family, in every language it gives them
in: `"font_family": "PingFang SC"` and `"苹方-简"` are the same font. Every
face of a collection (`.ttc`) is registered, and text is measured with the
regular face of a family. `get_font_families()` lists the names.

A `font_family` that is not registered is still written to the SVG (the
viewer resolves it), but text is measured with the default font.

A font is kept as its bytes and its glyphs are read as text is measured with
them, so a large CJK font costs no more memory than its file.

## Development

The SVG output is covered by snapshot tests under `asset/`. After an
intentional rendering change, regenerate them and review the diff:

```bash
UPDATE_SNAPSHOTS=1 cargo test --features image-encoder
git diff --stat asset/
```
