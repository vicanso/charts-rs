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
  [label formatters](#label-formatters), [overlapping labels](#overlapping-data-labels),
  [copyright lines and watermarks](#copyright-lines-and-watermarks)
- [Axes](#axes): [value and time x axes](#value-and-time-x-axes), [titles](#axis-titles)
- [Charts](#charts): [band around a line](#band-around-a-line),
  [bubble](#bubble-chart), [marginal histograms](#marginal-histograms),
  [density contours](#density-contours),
  [histogram](#histogram), [polar bar](#polar-bar-chart),
  [chord](#chord-chart), [gantt](#gantt-chart), [map](#map-chart),
  [lines](#lines-chart), [bullet](#bullet-chart),
  [sparklines in a table](#sparklines-in-a-table),
  [charts one above the other](#charts-one-above-the-other)
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

### Copyright lines and watermarks

Every chart takes `texts`: lines of text written over it, wherever they are
asked for.

<p align="center">
  <img src="../asset/image/texts.png" width="60%" alt="A chart with a source, a copyright line and a watermark">
</p>

```json
{
  "texts": [
    { "text": "Source: web analytics", "left": 10, "bottom": 6, "reserve": true },
    { "text": "© 2026 Example Inc.", "right": 10, "bottom": 6, "reserve": true },
    { "text": "DRAFT", "font_size": 72, "font_color": "#00000014", "font_weight": "bold", "rotate": -25 }
  ]
}
```

- A text is placed by its distances from the edges of the chart, in pixels:
  with a `left` it starts there, with a `right` it ends there, and with
  neither it is centered; `top` and `bottom` do the same from above and from
  below. Without any of them it is in the very middle, where a watermark
  goes.
- `font_size` (12 by default), `font_color` (that of the sub-title) and
  `font_weight` are its font, and `rotate` turns it by so many degrees. A
  watermark wants a color with an alpha, such as `"#00000014"`.
- The texts are written last, over everything else, and a chart leaves no
  room for them: a watermark is meant to be over the plot. A line that is to
  be below the axis, or above the title, says `"reserve": true`, and the
  chart is drawn in what is left: a text with a `top` or a `bottom` takes a
  band across there, one with a `left` or a `right` only a band down that
  side. A table, a calendar and a multi chart are larger by that room
  instead.
- In a `MultiChart` the `texts` at the top are over the whole picture, and
  those of a child over that child.
- A theme can have `texts` too: register one with your copyright line and
  every chart of that theme carries it. The `texts` of a chart stand in for
  those of its theme (`"texts": []` for none).

```rust
use charts_rs::{BarChart, TextConfig};

let mut chart = BarChart::new(
    vec![("Email", vec![120.0, 132.0, 101.0]).into()],
    ["Mon", "Tue", "Wed"].map(String::from).to_vec(),
);
chart.texts.push(TextConfig {
    text: "© 2026 Example Inc.".to_string(),
    right: Some(10.0),
    bottom: Some(6.0),
    // below the axis, not over its labels
    reserve: true,
    ..Default::default()
});
let svg = chart.svg().unwrap();
```

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

### Marginal histograms

`"marginal": {}` on a scatter chart draws the distribution of its own points:
above the plot, how the x values fall, and to the right, how the y values
fall. The bins run from the axis minimum to its maximum, so each bar is
exactly over the points it counts. The histograms use this chart's axes
and its points.

```rust
use charts_rs::{MarginalHistogram, ScatterChart};

let mut chart = ScatterChart::new(vec![
    ("Female", vec![161.2, 51.6, 167.5, 59.0, 159.5, 49.2]).into(),
    ("Male", vec![174.0, 65.6, 175.3, 71.8, 193.5, 80.7]).into(),
]);
chart.marginal = Some(MarginalHistogram {
    bin_count: 8,
    ..Default::default()
});
let _svg = chart.svg().unwrap();
```

```json
{
  "marginal": { "bin_count": 8, "size": 70 },
  "series_list": [
    {"name": "Female", "data": [161.2, 51.6, 167.5, 59, 159.5, 49.2]},
    {"name": "Male", "data": [174, 65.6, 175.3, 71.8, 193.5, 80.7]}
  ]
}
```

- `size` is the height of the top histogram and the width of the right one,
  in pixels (default 64). The plot keeps at least 40 pixels.
- `bin_count` of `0` chooses the number of bins; it is never more than 40.
- Several series share the bins and are drawn over each other, in the color
  of the series. Each side is scaled to its own tallest bar.
- A bubble is one point: its size is not a value on either axis. A missing
  coordinate is left out, and so is a point that falls outside the axis.

### Density contours

`"contour": {}` on a scatter chart draws unfilled density lines of its own
points. Each series is estimated with a Gaussian kernel, and the lines are
traced under the points in the color of the series. The contours use this
chart's axes and its points.

```rust
use charts_rs::{DensityContour, ScatterChart};

let mut chart = ScatterChart::new(vec![(
    "A",
    vec![
        1.0, 2.1, 1.4, 1.8, 0.8, 2.4, 1.6, 2.0, 1.1, 1.7, 0.9, 2.2, 1.3, 2.5, 1.2, 1.9,
    ],
)
.into()]);
chart.contour = Some(DensityContour::default());
let _svg = chart.svg().unwrap();
```

```json
{
  "contour": { "levels": 4, "grid_size": 48 },
  "series_list": [
    {"name": "A", "data": [1, 2.1, 1.4, 1.8, 0.8, 2.4, 1.6, 2, 1.1, 1.7, 0.9, 2.2, 1.3, 2.5, 1.2, 1.9]}
  ]
}
```

- `levels` of `4` (the default) draws the lines at 20%, 40%, 60% and 80% of
  the highest density. `0` is those four, and there are never more than 8.
- `grid_size` is the number of cells along each axis of the density grid
  (default 64, from 16 to 128).
- The bandwidth follows the spread of the points on each axis (Scott's
  rule). A series with fewer than 8 points on the axes draws no line.
- A bubble is one point: its size is not a coordinate. A missing coordinate
  is left out, and so is a point that falls outside the axis. On a
  logarithmic axis the density is estimated in the log of the values, so
  the lines stay over the points.

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

**Where to get the regions.** A real map needs a GeoJSON file of its
regions, which the library does not come with. Some places that have them:

| Source | What it has | Terms |
|---|---|---|
| [Natural Earth](https://www.naturalearthdata.com/) | The countries of the world and their states and provinces, at three levels of detail (1:10m, 1:50m, 1:110m); as GeoJSON in the `geojson` folder of [its repository](https://github.com/nvkelso/natural-earth-vector). | Public domain. |
| [geoBoundaries](https://www.geoboundaries.org/) | The administrative divisions of every country, level by level; as GeoJSON, simplified as well. | CC BY 4.0: say where the boundaries are from. |
| [GADM](https://gadm.org/) | The administrative areas of every country, several levels deep; in GeoJSON among other formats. | Free for academic and other non-commercial use; redistribution and commercial use need permission. |
| [DataV.GeoAtlas](https://datav.aliyun.com/portal/school/atlas/area_selector) | The provinces, cities and districts of China, as GeoJSON. The provinces and the cities of each are in this repository as downloaded, in [`asset/geo/china`](../asset/geo/china/README.md), with how they were got. | See the terms of the site. |

Before a file is used, it is worth a look:

- **Format.** What is read is GeoJSON with longitudes and latitudes in
  degrees, as its standard has them. A Shapefile or a TopoJSON file has to be
  converted, and coordinates in the metres of a projection turned back into
  degrees; [mapshaper](https://mapshaper.org/) does both.
- **Size.** Every point of every border is written into the SVG, so a file
  of several megabytes makes a picture of several megabytes. Simplify it to
  what a chart of a few hundred pixels can show:

  ```bash
  mapshaper provinces.shp -simplify 10% -o precision=0.001 provinces.geojson
  ```

- **Names.** `data` finds a region by its name, written exactly as in the
  file. Which property of a feature holds the name differs from source to
  source — `name`, `NAME`, `NAME_1`, `shapeName` — so look at the
  `properties` of a feature and give that key as `name_property`.
- **Terms.** The sources have terms of their own, as the table says. One
  that asks to be named can be, in a line of
  [`texts`](#copyright-lines-and-watermarks).
- **Borders.** Sources draw disputed borders differently, and some countries
  have rules for the maps that are published in them: in China a map for the
  public has to pass an official review. Check what applies before a map is
  published.

### Lines chart

A `LinesChart` draws flows from one point to another. The width of a line
is its value, between `min_width` and `max_width`. `curveness` bends each
line (`0` is straight); an arrow at the end shows the direction.

Without `geo_json` the points are on a plane. With it they are longitude
and latitude, projected like a map, and the regions are drawn underneath.

```json
{
  "curveness": 0.3,
  "lines": [
    {"name": "North", "value": 80, "from": [0, 0], "to": [40, 30]}
  ]
}
```

In Rust: `LinesChart::new(vec![((0.0, 0.0), (40.0, 30.0), 80.0).into()])`.
A fuller picture, sixteen flows over the sample country, is
`asset/lines_chart/flows.json`.

### Bullet chart

A `BulletChart` puts measures against their targets, a row each: a bar for
the value, a line across it for the target, and bands behind them for what
is poor, fair and good. Five of them take the room of one gauge, which makes
them the chart for a report of KPIs.

<p align="center">
  <img src="../asset/image/bullet.png" width="60%" alt="A bullet chart of five measures">
</p>

```json
{
  "label_show": true,
  "bullets": [
    {"name": "Done in time (%)", "value": 92, "target": 95, "ranges": [80, 90, 100], "max": 100},
    {"name": "Wait (min)", "value": 12, "target": 10, "ranges": [10, 20, 30]}
  ]
}
```

- Every row has a scale of its own, from 0 (or from below it, where a value
  is) to the first round number past its largest value, unless `min` and
  `max` say otherwise. Its ticks are round numbers.
- The bands are shades of the text color, the first one the darkest;
  `range_colors` gives them colors of their own. A measure for which less is
  better, a waiting time, wants them the other way round.
- `label_show` writes the value to the right of each row, in the format of
  `series_label_formatter`; the scales take the format of the first of
  `y_axis_configs` (`axis_formatter`), and `x_axis_hidden` leaves them out.

In Rust: `BulletChart::new(vec![("CPU", 72.0, 80.0).into()])` — a name, a
value and a target; a `Bullet` has the bands, the scale and a color as well.

### Sparklines in a table

A cell of a `TableChart` that is an array of numbers is a sparkline: a
small line of the values, drawn in place of text, from the lowest at the
bottom of the cell to the highest at the top, with a dot on the last one.

<p align="center">
  <img src="../asset/image/table-sparkline.png" width="60%" alt="A table of servers with the trend of each">
</p>

```json
{
  "data": [
    ["Server", "CPU", "Last 24 hours"],
    ["api-1", "72%", [42, 45, 51, 48, 60, 66, 63, 71, 69, 74, 70, 72]],
    ["api-2", "38%", [30, 34, 31, 29, 35, 41, null, null, 37, 38]]
  ]
}
```

- Every sparkline has a scale of its own: it shows the shape of its values,
  not how they compare with those of the other rows.
- A `null` leaves a gap. `"sparkline_type": "bar"` draws a bar for each
  value instead, from 0; `sparkline_color` colors them.
- A number in a cell is written as it is, and `null` is an empty cell.

In Rust: `table.sparklines.push(TableSparkline { indexes: vec![1, 2],
values: vec![42.0, 45.0, 51.0] })`, `indexes` being the row and the column.

### Charts one above the other

A `MultiChart` stacks its children; each draws its own axes, and the plot
of each starts after the labels of its y axis — wider for the volumes of a
stock than for its prices. With `align_plots` the plots line up: they start
where the one that starts furthest in starts and end where the one that
ends furthest in ends, so that a day is in the same place in each.

<p align="center">
  <img src="../asset/image/multi-aligned.png" width="60%" alt="Prices, volumes and MACD of a month, lined up">
</p>

```json
{
  "align_plots": true,
  "child_charts": [
    {"type": "candlestick", "height": 300, "x_axis_hidden": true, "x_axis_data": ["10-01", "10-02"],
     "series_list": [{"name": "Day", "data": [100, 101.5, 99.2, 102.1, 101.5, 99.8, 99.1, 102.4]}]},
    {"type": "bar", "height": 120, "legend_show": false, "x_axis_data": ["10-01", "10-02"],
     "series_list": [{"name": "Volume", "data": [1830000, 2240000]}]}
  ]
}
```

- It works on the bar, line, candlestick, histogram, scatter, box plot and
  waterfall children that stack; one placed by `x` and `y` keeps its plot
  where its axes put it.
- It is done with `grid_left` and `grid_right`, which every one of those
  charts takes: how far in its plot starts and ends at the least. They line
  up charts that are drawn apart, too — two SVGs on one page.
- A child that wants its plot further in (its own `grid_left`) has it
  there, and the others line up with it.

The example of the gallery is `asset/multi_chart/aligned.json`: a month of
prices with their moving averages, the volumes below them and the MACD
below those. The bars of the volumes and of the MACD are red or green with
the day (`colors` of a series, a color for each bar), and the first color
of the palette, that of the candles and the bars in the legend, is a grey.

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
