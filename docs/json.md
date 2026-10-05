# JSON options

[中文](./json-zh.md)

Every chart can be created from a JSON document with its `from_json`, e.g. `BarChart::from_json(json)`. This page lists every key such a document may have.

- A key that is left out, or set to `null`, keeps its default.
- The document is checked before it is used: an unknown key (with a hint when it looks like a typo), a value of the wrong type, an unknown enum value or a size out of range is an error, not silently ignored.
- Enum values are matched without regard to case.
- “theme” as a default means the value comes from the theme.
- The names of the keys are the names of the fields of the chart structs, so the [API documentation](https://docs.rs/charts-rs) tells the same story for the builder API.

## Contents

- [Value types](#value-types)
- [Common options](#common-options)
- Charts: [Bar](#bar), [Horizontal bar](#horizontal-bar), [Line](#line), [Pie](#pie), [Radar](#radar), [Scatter](#scatter), [Candlestick](#candlestick), [Table](#table), [Heatmap](#heatmap), [Funnel](#funnel), [Waterfall](#waterfall), [Calendar](#calendar), [Gauge](#gauge), [Treemap](#treemap), [Box plot](#box-plot), [Sunburst](#sunburst), [Sankey](#sankey), [Tree](#tree), [Graph](#graph), [Parallel](#parallel), [Theme river](#theme-river), [Histogram](#histogram), [Polar bar](#polar-bar), [Chord](#chord), [Gantt](#gantt), [Map](#map), [Multi chart](#multi-chart)
- [Keys for tools](#keys-for-tools)

## Value types

| Type | Written as |
|---|---|
| number | Any JSON number. |
| integer | A whole number, not negative. Counts of ticks and splits go up to 1000, indexes up to 1 000 000. |
| color | `"#rgb"`, `"#rgba"`, `"#rrggbb"`, `"#rrggbbaa"`, `"rgb(r, g, b)"`, `"rgba(r, g, b, a)"` or `"transparent"`. |
| margin | A number for all four sides, or `{"left": 5, "top": 5, "right": 5, "bottom": 5}` (sides left out are `0`). |
| scale | `"linear"`, `"log"` (base 10), `"log2"`, `"log10"`, or `{"type": "log", "base": 5}`. |
| x value | A number, or a date as a string: `"2024-03-01"`, `"2024-03-01 08:30"`, `"2024-03-01T08:30:00+08:00"`. Numbers on a time axis are unix seconds. |
| number[] | An array of numbers; `null` is a missing value. |
| color[] | An array of colors; `null` keeps the default of that position. |

## Common options

These keys are accepted by every chart except the table and the multi chart. A chart uses the ones that mean something for it: a pie chart has no axes, for instance, and ignores the axis keys.

<!-- keys: base -->

### Canvas

| Key | Type | Default | Description |
|---|---|---|---|
| `theme` | string | `"light"` | Name of the theme: `light`, `dark`, `ant`, `vintage`, `shine`, `walden`, `westeros`, `chalk`, `grafana`, `shadcn`, or one registered with `add_theme`. An unknown name is an error. |
| `width` | number > 0 | `600` | Width of the chart in pixels. |
| `height` | number > 0 | `400` | Height of the chart in pixels. |
| `x` | number | `0` | Horizontal offset of the chart inside the SVG. |
| `y` | number | `0` | Vertical offset of the chart inside the SVG. |
| `margin` | margin | `5` | Margin around the whole chart. |
| `font_family` | string | `"Roboto"` | Font family of every text. The font has to be loaded (see [Fonts](./guide.md#fonts) in the guide) for text to be measured correctly. |
| `compact` | boolean | `false` | Emits compact SVG: the same picture, typically 20–30% smaller. |
| `empty_text` | string |  | Text shown in the middle of the plot when there is no data to draw. |

### Title

| Key | Type | Default | Description |
|---|---|---|---|
| `title_text` | string |  | Title text; no title is drawn when empty. |
| `title_font_size` | number | `18` | Title font size. |
| `title_font_color` | color | theme | Title font color. |
| `title_font_weight` | string | `"bold"` | Title font weight, e.g. `"bold"`. |
| `title_margin` | margin |  | Margin around the title. |
| `title_align` | `"left"` / `"center"` / `"right"` | `"center"` | Horizontal alignment of the title. |
| `title_height` | number | `30` | Height reserved for the title row. |

### Sub-title

| Key | Type | Default | Description |
|---|---|---|---|
| `sub_title_text` | string |  | Sub-title text, drawn under the title. |
| `sub_title_font_size` | number | `14` | Sub-title font size. |
| `sub_title_font_color` | color | theme | Sub-title font color. |
| `sub_title_font_weight` | string |  | Sub-title font weight, e.g. `"bold"`. |
| `sub_title_margin` | margin |  | Margin around the sub-title. |
| `sub_title_align` | `"left"` / `"center"` / `"right"` | `"center"` | Horizontal alignment of the sub-title. |
| `sub_title_height` | number | `20` | Height reserved for the sub-title row. |

### Legend

| Key | Type | Default | Description |
|---|---|---|---|
| `legend_show` | boolean |  | Shows or hides the legend. It is shown by default, except in pie, theme river and waterfall charts. |
| `legend_position` | `"top"` / `"bottom"` / `"left"` / `"right"` | `"top"` | Where the legend goes: at the top beside the title (the default), at the bottom, or stacked vertically on the left or right of the plot. |
| `legend_align` | `"left"` / `"center"` / `"right"` | `"center"` | Horizontal alignment of the legend. |
| `legend_category` | `"normal"` / `"rect"` / `"round_rect"` / `"circle"` | `"normal"` | Shape of the legend markers: a line with a dot (`normal`), a rectangle, a rounded rectangle or a circle. |
| `legend_font_size` | number | `14` | Legend font size. |
| `legend_font_color` | color | theme | Legend font color. |
| `legend_font_weight` | string |  | Legend font weight, e.g. `"bold"`. |
| `legend_margin` | margin |  | Margin around the legend. |

### X axis

| Key | Type | Default | Description |
|---|---|---|---|
| `x_axis_data` | string[] |  | Labels of the x axis: the categories of the chart. |
| `x_axis_type` | `"category"` / `"value"` / `"time"` | `"category"` | How the values of a continuous x axis read: plain numbers (`value`) or timestamps (`time`). The axis is continuous whenever x values are given; date strings select `time` by themselves. |
| `x_axis_values` | x value[] |  | The x value of each data point, shared by every series. Setting them makes the x axis continuous: points sit at their value instead of in evenly spaced slots (line and bar charts). |
| `x_axis_min` | x value |  | Fixed start of a continuous x axis; by default the first value. In a histogram: the start of the first bin. |
| `x_axis_max` | x value |  | Fixed end of a continuous x axis; by default the last value. In a histogram: the end of the last bin. |
| `x_axis_formatter` | string |  | Format of the x axis labels: `{c}` is the label; on a time axis a pattern of `%Y %y %m %d %H %M %S %b`, e.g. `"%m-%d %H:%M"`. |
| `x_axis_time_offset` | number | `0` | Minutes east of UTC a time axis is displayed in (480 for UTC+8). |
| `x_axis_title` | string |  | Title of the x axis, written below its labels. |
| `x_axis_hidden` | boolean | `false` | Hides the x axis. In a polar bar chart: the category labels. |
| `x_axis_height` | number | `30` | Height reserved for the x axis. |
| `x_axis_stroke_color` | color | theme | Color of the x axis line and ticks. |
| `x_axis_stroke_width` | number | `1` | Width of the x axis line and ticks. |
| `x_axis_font_size` | number | `14` | Font size of the x axis labels. |
| `x_axis_font_color` | color | theme | Font color of the x axis labels. |
| `x_axis_font_weight` | string |  | Font weight of the x axis labels, e.g. `"bold"`. |
| `x_axis_name_gap` | number | `5` | Gap between the x axis line and its labels. |
| `x_axis_name_rotate` | number | `0` | Rotation of the x axis labels in radians, e.g. `0.785` for 45°. |
| `x_axis_margin` | margin |  | Margin around the x axis. |
| `x_boundary_gap` | boolean | `true` | Leaves a gap at both ends of the x axis, so points sit in the middle of their category (`true`), or puts the first and last point on the ends of the axis (`false`). |
| `x_axis_label_overflow` | `"thin"` / `"rotate"` / `"ellipsis"` | `"thin"` | What to do with x axis labels that do not fit side by side: leave some out (`thin`), rotate them, or cut them with an ellipsis. |

### Y axis

| Key | Type | Default | Description |
|---|---|---|---|
| `y_axis_hidden` | boolean | `false` | Hides the y axis. In a polar bar chart: the value labels. |
| `y_axis_configs` | object[] |  | The value axes: the first one is the left y axis, a second one the right y axis. |

### Grid

| Key | Type | Default | Description |
|---|---|---|---|
| `grid_stroke_color` | color | theme | Color of the grid lines. |
| `grid_stroke_width` | number | `1` | Width of the grid lines. |
| `grid_stroke_dash_array` | string |  | Dashes of the grid lines, as the `stroke-dasharray` of SVG takes them (e.g. `"4,2"`). Solid lines by default. Applies to the grid of the charts with an x and a y axis, to those of the gantt, radar and polar bar charts, to the lines of a punch card and to the axes of a parallel chart. |

### Series

| Key | Type | Default | Description |
|---|---|---|---|
| `series_list` | object[] |  | The data series of the chart. |
| `series_colors` | color[] | theme | The color palette: a series takes the color at its position, starting over when the palette runs out. |
| `series_stroke_width` | number | `2` | Stroke width of the series lines. |
| `series_symbol` | object |  | Marker drawn on the data points of every line; `null` draws none. Default: a circle as large as `series_stroke_width`, filled with the background color. |
| `series_smooth` | boolean | `false` | Draws line series as smooth curves. |
| `series_fill` | boolean | `false` | Fills the area under line series. |
| `series_fill_opacity` | number |  | How opaque the fill of an area is, from 0 to 1. Default: `0.39` under a line, `0.2` in a radar chart. |
| `series_label_formatter` | string |  | Format of the data labels: `{c}` value, `{a}` series name, `{b}` category, `{d}` percentage, `{t}` value in thousands notation. |
| `series_label_font_size` | number | `14` | Font size of the data labels. |
| `series_label_font_color` | color | theme | Font color of the data labels. |
| `series_label_font_weight` | string |  | Font weight of the data labels, e.g. `"bold"`. |
| `series_label_hide_overlap` | boolean | `false` | Leaves out a data label that would overlap one already drawn. |
| `stack_percent` | boolean | `false` | Shows the series of a stack as their shares of it: every stack adds up to 100%, on an axis in percent (bar, horizontal bar, line and polar bar charts). |

### Tooltip and animation

| Key | Type | Default | Description |
|---|---|---|---|
| `tooltip_show` | boolean | `false` | Gives every data shape a tooltip that shows on hover (plain CSS, no script), and a `<title>` for assistive tools. Not available in calendar, gauge, parallel, radar and theme river charts. |
| `tooltip_font_size` | number |  | Font size of the tooltips. Default: that of the data labels (`series_label_font_size`); a scatter chart leaves it to the viewer. |
| `tooltip_font_color` | color |  | Font color of the tooltips. Default: that of the data labels (`series_label_font_color`); a scatter chart leaves it to the viewer. |
| `tooltip_font_weight` | string |  | Font weight of the tooltips, e.g. `"bold"`. None by default. |
| `animation` | object |  | Animates the chart as it appears; `{}` uses the defaults. Supported by the bar, horizontal bar, line, pie, funnel, sunburst, treemap, sankey, histogram, polar bar and chord charts. |

### `series_list[]`

<!-- keys: base.series_list -->

| Key | Type | Default | Description |
|---|---|---|---|
| `name` | string |  | Name of the series, shown in the legend and in tooltips. |
| `data` | number[] |  | The values of the series, one per category; `null` is a missing point. Some charts read them differently, as told in their sections. |
| `index` | integer ≥ 0 |  | Position in the color palette; by default the position of the series. |
| `y_axis_index` | integer ≥ 0 | `0` | The y axis the series belongs to: `0` (left) or `1` (right). |
| `label_show` | boolean | `false` | Shows the value of every data point as a label. |
| `category` | `"line"` / `"bar"` |  | In a bar chart: draws this series as a line (`line`) instead of bars. |
| `start_index` | integer ≥ 0 | `0` | Index of the category the first value belongs to. |
| `mark_lines` | object[] |  | Lines across the plot at a statistic of the series or at a fixed value. |
| `mark_points` | object[] |  | Markers on the smallest or largest point of the series. |
| `mark_areas` | object[] |  | Shaded bands across the plot between two values. |
| `colors` | color[] |  | A color for each data point (bar, horizontal bar and polar bar charts); `null` keeps the color of the series. |
| `stroke_dash_array` | string |  | Dash pattern of a line series, e.g. `"4 2"`. |
| `stack` | string |  | Name of a stack: series with the same name (and y axis) are piled on top of each other. |
| `smooth` | boolean |  | Overrides `series_smooth` for this series. |
| `fill` | boolean |  | Overrides `series_fill` for this series. |
| `symbol` | object |  | Overrides `series_symbol` for this series; `null` draws no marker. |
| `x_values` | x value[] |  | The x value of each data point, for a series that is not sampled at the chart’s `x_axis_values`. |
| `band` | object |  | A filled band around the line: a lower and an upper bound for each data point. |
| `step` | `"start"` / `"middle"` / `"end"` |  | Draws the line as steps: level between two points, changing to the next value at the point itself (`start`), half way to the next one (`middle`) or at the next point (`end`). Wins over `smooth`. |
| `error_bar` | object |  | Error bars: a lower and an upper bound for each data point, drawn as a line with a cap at both ends over the bar or the point (bar, line and scatter charts). The y axis makes room for them, and tooltips tell the bounds. |
| `ring` | integer ≥ 0 | `0` | In a pie chart: the ring the series is a slice of. The series of a ring share a circle of their own, the lowest ring innermost — nested pies. |

#### `series_list[].mark_lines[]`

<!-- keys: base.series_list.mark_lines -->

| Key | Type | Default | Description |
|---|---|---|---|
| `category` | `"average"` / `"min"` / `"max"` / `"value"` |  | Where the line is drawn: at the average, the minimum or the maximum of the series, or at `value`. |
| `value` | number |  | The value of the line when `category` is `value`. |
| `color` | color |  | Color of the line, its dot and its arrow. Default: the color of the series. |
| `stroke_width` | number | `1` | Width of the line. |
| `stroke_dash_array` | string | `"4,2"` | Dashes of the line, as the `stroke-dasharray` of SVG takes them; `""` draws a solid line. |

#### `series_list[].mark_points[]`

<!-- keys: base.series_list.mark_points -->

| Key | Type | Default | Description |
|---|---|---|---|
| `category` | `"min"` / `"max"` |  | The point that is marked. |

#### `series_list[].mark_areas[]`

<!-- keys: base.series_list.mark_areas -->

| Key | Type | Default | Description |
|---|---|---|---|
| `from` | number / `"average"` / `"min"` / `"max"` |  | One edge of the band. |
| `to` | number / `"average"` / `"min"` / `"max"` |  | The other edge of the band. |
| `color` | color |  | Color of the band. Default: the color of the series. |
| `opacity` | number | `0.16` | How opaque the band is, from 0 to 1. A `color` with an alpha of its own (`"#ff000080"`) keeps it when no opacity is given. |

#### `series_list[].symbol` and `series_symbol`

<!-- keys: base.series_list.symbol -->
<!-- keys: base.series_symbol = base.series_list.symbol -->

| Key | Type | Default | Description |
|---|---|---|---|
| `type` | `"circle"` / `"rect"` / `"square"` / `"triangle"` / `"diamond"` |  | Shape of the marker (`square` is the same as `rect`). |
| `size` | number |  | Size of the marker: its radius, or half its side. |
| `radius` | number |  | The same as `size`, which wins when both are set. |
| `color` | color |  | Fill color of the marker; without one the marker is hollow. |

#### `series_list[].band`

<!-- keys: base.series_list.band -->

| Key | Type | Default | Description |
|---|---|---|---|
| `lower` | number[] |  | Lower bound of each point; `null` leaves a gap in the band. |
| `upper` | number[] |  | Upper bound of each point; `null` leaves a gap in the band. |

#### `series_list[].error_bar`

<!-- keys: base.series_list.error_bar -->

The same two lists as a band: a point has an error bar where both of its bounds are given.

| Key | Type | Default | Description |
|---|---|---|---|
| `lower` | number[] |  | Lower end of the error bar of each point; `null` leaves the point without one. |
| `upper` | number[] |  | Upper end of the error bar of each point; `null` leaves the point without one. |
| `stroke_width` | number | `1.5` | Width of the error bars. |

### `y_axis_configs[]`

<!-- keys: base.y_axis_configs -->

| Key | Type | Default | Description |
|---|---|---|---|
| `axis_font_size` | number | `14` | Font size of the axis labels. |
| `axis_font_color` | color | theme | Font color of the axis labels. |
| `axis_font_weight` | string |  | Font weight of the axis labels, e.g. `"bold"`. |
| `axis_stroke_color` | color | `transparent` | Color of the axis line; transparent by default. |
| `axis_stroke_width` | number | `1` | Width of the axis line and ticks. |
| `axis_width` | number |  | Width reserved for the axis; by default as wide as its labels need. |
| `axis_split_number` | integer 0–1000 | `6` | Number of intervals the value range is split into. |
| `axis_name_gap` | number | `8` | Gap between the axis line and its labels. |
| `axis_formatter` | string |  | Format of the axis labels: `{c}` value, `{t}` value in thousands notation, e.g. `"{c} ms"`. |
| `axis_margin` | margin |  | Margin around the axis. |
| `axis_min` | number |  | Fixed lower bound of the axis; by default derived from the data. |
| `axis_max` | number |  | Fixed upper bound of the axis; by default derived from the data. |
| `axis_scale` | scale | `"linear"` | Scale of the axis: linear or logarithmic. |
| `axis_title` | string |  | Title of the axis, written along it. |
| `axis_inverse` | boolean | `false` | Turns the axis upside down: the smallest value at the top, the largest at the bottom (a ranking, where 1 is the best place). |

### `animation`

<!-- keys: base.animation -->

| Key | Type | Default | Description |
|---|---|---|---|
| `duration` | integer ≥ 0 | `1000` | Duration of the animation in milliseconds. |
| `easing` | string | `"ease"` | CSS easing function: `ease`, `linear`, `ease-in`, `ease-out`, `ease-in-out`. |
| `delay` | integer ≥ 0 | `80` | Delay in milliseconds between one element and the next (bars, series, slices, levels). |

## Bar

`BarChart::from_json` — Vertical bars, one group per category of `x_axis_data`. A series with `"category": "line"` is drawn as a line; series with the same `stack` pile up; `y_axis_index` binds a series to the right y axis.

<!-- keys: bar -->

| Key | Type | Default | Description |
|---|---|---|---|
| `radius` | number |  | Corner radius of the bars. |
| `series_label_position` | `"top"` / `"inside"` | `"top"` | Where the value labels of the bars go: above their end, or in the middle of the bar, which suits stacked bars (and is the default with `stack_percent`). |

<!-- example: bar -->

```json
{
  "title_text": "Visits",
  "radius": 3,
  "x_axis_data": ["Mon", "Tue", "Wed"],
  "series_list": [
    {"name": "Email", "data": [120, 132, 101], "label_show": true},
    {"name": "Search", "data": [220, 182, 191], "category": "line"}
  ]
}
```

## Horizontal bar

`HorizontalBarChart::from_json` — Horizontal bars: the categories of `x_axis_data` go down the left side, the values along the bottom.

<!-- keys: horizontal_bar -->

| Key | Type | Default | Description |
|---|---|---|---|
| `series_label_position` | `"left"` / `"right"` / `"top"` / `"bottom"` / `"inside"` | `"right"` | Where the value labels go; by default to the right of the bars. |

<!-- example: horizontal_bar -->

```json
{
  "x_axis_data": ["Brazil", "India", "China"],
  "series_label_position": "inside",
  "series_list": [
    {"name": "2023", "data": [182, 234, 290], "label_show": true}
  ]
}
```

## Line

`LineChart::from_json` — Lines over the categories of `x_axis_data`, or over a continuous x axis when `x_axis_values` are given. It has no keys of its own: smooth curves, steps, area fill, markers, mark lines and bands are all [common options](#common-options).

<!-- example: line -->

```json
{
  "series_smooth": true,
  "x_axis_data": ["Mon", "Tue", "Wed", "Thu"],
  "series_list": [
    {
      "name": "Email",
      "data": [120, 132, null, 134],
      "mark_lines": [
        {"category": "average"}
      ]
    },
    {"name": "Search", "data": [220, 182, 191, 234], "fill": true}
  ]
}
```

## Pie

`PieChart::from_json` — A pie, a donut or a nightingale rose. Every series is one slice: its value is the sum of its `data`. Series with different `ring` numbers make nested pies: every ring shares the turn among its own slices, the rings split the room between `inner_radius` and `radius`, and the slices of the inner rings are named on them.

<!-- keys: pie -->

| Key | Type | Default | Description |
|---|---|---|---|
| `radius` | number | `150` | Outer radius of the pie in pixels. |
| `inner_radius` | number | `40` | Inner radius; above `0` the pie is a donut. |
| `rose_type` | boolean | `true` | Draws a nightingale rose: every slice has the same angle and its radius follows its value. `false` draws a plain pie. |
| `border_radius` | number |  | Corner radius of the slices. |
| `start_angle` | number | `0` | Angle the first slice starts at, in degrees clockwise from 12 o’clock. |
| `end_angle` | number |  | Angle the last slice ends at: the slices share the part of the circle between the two angles (`-90` to `90` is the upper half, a half doughnut). Default: a full turn after `start_angle`. |
| `series_label_position` | `"inside"` / `"outside"` | `"outside"` | Where the labels go: inside the slices, or outside with a leader line (the default). |
| `min_show_label_angle` | number | `0` | Slices spanning fewer degrees than this get no label. |
| `ring_gap` | number | `6` | Gap between two rings of nested pies (series with a `ring`), in pixels. |

<!-- example: pie -->

```json
{
  "rose_type": false,
  "radius": 120,
  "inner_radius": 60,
  "series_list": [
    {"name": "Email", "data": [40]},
    {"name": "Search", "data": [38]},
    {"name": "Direct", "data": [32]}
  ]
}
```

## Radar

`RadarChart::from_json` — Every series is a polygon over the `indicators`, with one value for each of them.

<!-- keys: radar -->

| Key | Type | Default | Description |
|---|---|---|---|
| `indicators` | object[] |  | The axes of the radar; every series has one value for each. |
| `split_number` | integer 0–1000 | `5` | Number of rings of the web; `0` means 5. |

### `indicators[]`

<!-- keys: radar.indicators -->

| Key | Type | Default | Description |
|---|---|---|---|
| `name` | string |  | Name of the indicator. |
| `max` | number |  | Value at the outer end of the axis; raised to the largest value of the data when that is larger. |

<!-- example: radar -->

```json
{
  "indicators": [
    {"name": "Sales", "max": 6500},
    {"name": "Marketing", "max": 16000},
    {"name": "Support", "max": 30000}
  ],
  "series_list": [
    {"name": "Budget", "data": [4200, 3000, 20000]},
    {"name": "Spending", "data": [5000, 14000, 28000]}
  ]
}
```

## Scatter

`ScatterChart::from_json` — Points on two value axes. The `data` of a series is a flat list of `x, y` pairs (`x, y, size` triples with `bubble`). `x_boundary_gap` is not used: the points always sit on the scale.

<!-- keys: scatter -->

| Key | Type | Default | Description |
|---|---|---|---|
| `series_symbol_sizes` | number[] |  | Radius of the markers of each series. |
| `bubble` | boolean | `false` | Bubble chart: the series data are `[x, y, size]` triples and the area of each marker follows its size. |
| `bubble_min_size` | number | `4` | Radius of the smallest bubble. |
| `bubble_max_size` | number | `30` | Radius of the largest bubble. |
| `series_symbols` | array |  | Marker of each series: its shape as a string (`"circle"`, `"triangle"`, `"rect"`, `"diamond"`), or an object with that `type` and a fill `color`. The size is `series_symbol_sizes`. By default the series cycle through the four shapes, in the color of the series. |
| `x_axis_config` | object |  | Configuration of the x axis, which is a value axis here. By default the same as the first of `y_axis_configs`. |
| `regression` | `"linear"` / `"exponential"` / `"logarithmic"` / `"polynomial"` |  | Draws the curve of this kind that fits the points of each series best (by least squares), in the color of the series. An exponential curve is fitted to the points above 0, a logarithmic one to the points right of 0. |
| `regression_order` | integer 0–1000 | `2` | Order of a `polynomial` regression, from 1 to 6. |
| `regression_label_show` | boolean | `false` | Writes the formula of each fitted curve at its end, e.g. `y = 1.5x + 2`. |

### `x_axis_config`

<!-- keys: scatter.x_axis_config = base.y_axis_configs -->

The same keys as [`y_axis_configs[]`](#y_axis_configs). What they say of the values of the axis is used — `axis_min`, `axis_max`, `axis_split_number`, `axis_formatter`; the font, the color and the line width of the x axis are those of the `x_axis_*` options.

<!-- example: scatter -->

```json
{
  "series_symbol_sizes": [6, 6],
  "series_list": [
    {"name": "Female", "data": [161.2, 51.6, 167.5, 59, 159.5, 49.2]},
    {"name": "Male", "data": [174, 65.6, 175.3, 71.8, 193.5, 80.7]}
  ]
}
```

## Candlestick

`CandlestickChart::from_json` — One candle per category. The `data` of the series is a flat list of four values per category: open, close, lowest, highest.

<!-- keys: candlestick -->

| Key | Type | Default | Description |
|---|---|---|---|
| `candlestick_up_color` | color | `#EC0000` | Fill color of the candles that close higher than they open. |
| `candlestick_up_border_color` | color | `#8A0000` | Border color of those candles. |
| `candlestick_down_color` | color | `#00DA3C` | Fill color of the candles that close lower than they open. |
| `candlestick_down_border_color` | color | `#008F28` | Border color of those candles. |
| `candlestick_style` | `"candle"` / `"ohlc"` | `"candle"` | Draws candles, or OHLC bars: a line from the lowest to the highest price with a tick to its left at the open and one to its right at the close. |

<!-- example: candlestick -->

```json
{
  "x_axis_data": ["2024-10-24", "2024-10-25", "2024-10-28"],
  "series_list": [
    {"name": "MA5", "data": [20, 34, 10, 38, 40, 35, 30, 50, 31, 38, 33, 44]}
  ]
}
```

## Table

`TableChart::from_json` — A table. It does not take the common options: the keys below are all it accepts.

<!-- keys: table -->

| Key | Type | Default | Description |
|---|---|---|---|
| `theme` | string | `"light"` | Name of the theme: `light`, `dark`, `ant`, `vintage`, `shine`, `walden`, `westeros`, `chalk`, `grafana`, `shadcn`, or one registered with `add_theme`. An unknown name is an error. |
| `width` | number > 0 | `600` | Width of the chart in pixels. |
| `height` | number > 0 |  | Ignored: a table is as high as its rows. |
| `x` | number | `0` | Horizontal offset of the chart inside the SVG. |
| `y` | number | `0` | Vertical offset of the chart inside the SVG. |
| `font_family` | string | `"Roboto"` | Font family of every text. The font has to be loaded (see [Fonts](./guide.md#fonts) in the guide) for text to be measured correctly. |
| `background_color` | color | theme | Background color behind the title; the rows have `header_background_color` and `body_background_colors`. |
| `title_text` | string |  | Title text; no title is drawn when empty. |
| `title_font_size` | number | `18` | Title font size. |
| `title_font_color` | color | theme | Title font color. |
| `title_font_weight` | string | `"bold"` | Title font weight, e.g. `"bold"`. |
| `title_margin` | margin |  | Margin around the title. |
| `title_align` | `"left"` / `"center"` / `"right"` | `"center"` | Horizontal alignment of the title. |
| `title_height` | number | `45` | Height reserved for the title row. |
| `sub_title_text` | string |  | Sub-title text, drawn under the title. |
| `sub_title_font_size` | number | `14` | Sub-title font size. |
| `sub_title_font_color` | color | theme | Sub-title font color. |
| `sub_title_font_weight` | string |  | Sub-title font weight, e.g. `"bold"`. |
| `sub_title_margin` | margin |  | Margin around the sub-title. |
| `sub_title_align` | `"left"` / `"center"` / `"right"` | `"center"` | Horizontal alignment of the sub-title. |
| `sub_title_height` | number | `20` | Height reserved for the sub-title row. |
| `data` | array |  | The rows of the table, each an array of strings; the first row is the header. |
| `spans` | number[] |  | Width of each column: values below 1 are shares of the table width, larger ones pixels; the other columns share the rest. |
| `text_aligns` | string[] |  | Alignment of the text of each column: `"left"`, `"center"` or `"right"`. |
| `border_color` | color | theme | Color of the lines between the rows. |
| `border_width` | number | `1` | Width of the lines between the rows, and of the outer border. |
| `header_row_padding` | margin | `{left: 10, top: 8, right: 10, bottom: 8}` | Padding of the header row. |
| `header_row_height` | number | `30` | Smallest height of the header row. |
| `header_font_size` | number | `14` | Font size of the header. |
| `header_font_color` | color | theme | Font color of the header. |
| `header_font_weight` | string |  | Font weight of the header, e.g. `"bold"`. |
| `header_background_color` | color | theme | Background color of the header. |
| `body_row_padding` | margin | `{left: 10, top: 5, right: 10, bottom: 5}` | Padding of the body rows. |
| `body_row_height` | number | `30` | Smallest height of a body row. |
| `body_font_size` | number | `14` | Font size of the body. |
| `body_font_color` | color | theme | Font color of the body. |
| `body_font_weight` | string |  | Font weight of the body, e.g. `"bold"`. |
| `body_background_colors` | color[] | theme | Background colors of the body rows, taken in turn. |
| `outlined` | boolean | `false` | Draws a border around the table. |
| `cell_styles` | object[] |  | Styles of single cells. |
| `compact` | boolean | `false` | Emits compact SVG: the same picture, typically 20–30% smaller. |

### `cell_styles[]`

<!-- keys: table.cell_styles -->

| Key | Type | Default | Description |
|---|---|---|---|
| `indexes` | array |  | The cell as `[row, column]`, counted from 0 with the header as row 0. |
| `font_color` | color |  | Font color of the cell. |
| `font_weight` | string |  | Font weight of the cell. |
| `background_color` | color |  | Background color of the cell. |

<!-- example: table -->

```json
{
  "title_text": "NASDAQ",
  "data": [
    ["Name", "Price", "Change"],
    ["Datadog Inc", "97.32", "-7.49%"],
    ["Hashicorp Inc", "28.66", "-9.25%"]
  ],
  "text_aligns": ["left", "center", "right"],
  "body_font_weight": "bold",
  "cell_styles": [
    {"indexes": [1, 2], "font_color": "#fff", "background_color": "#2d7c2b"}
  ]
}
```

## Heatmap

`HeatmapChart::from_json` — A grid of cells over the categories of `x_axis_data` and `y_axis_data`, colored by value. The data are in `series`, not in `series_list`.

<!-- keys: heatmap -->

| Key | Type | Default | Description |
|---|---|---|---|
| `y_axis_data` | string[] |  | Labels of the y axis: the categories from bottom to top. |
| `series` | object |  | The cells and how their values map to colors. |

### `series`

<!-- keys: heatmap.series -->

| Key | Type | Default | Description |
|---|---|---|---|
| `min` | number | `0` | Value shown in `min_color`. |
| `max` | number | `0` | Value shown in `max_color`; `0` takes the largest value of the data. |
| `min_color` | color | `#F0D99C` | Color of the smallest value. |
| `max_color` | color | `#BF444C` | Color of the largest value. |
| `min_font_color` | color | `#464646` | Font color of the labels on cells with low values; with `colors`, `steps` or `thresholds`, on light cells. |
| `max_font_color` | color | `#EEEEEE` | Font color of the labels on cells with high values; with `colors`, `steps` or `thresholds`, on dark cells. |
| `colors` | color[] |  | The colors of the scale, from the smallest value to the largest. Two or more take the place of `min_color` and `max_color`: the scale goes through all of them. |
| `steps` | integer 0–1000 | `0` | Number of classes of the same width the values are sorted into, each drawn in one color, instead of a continuous scale; `0` and `1` keep it continuous. With as many `colors` as steps, every class has one of them. |
| `thresholds` | number[] |  | The values where one class ends and the next begins; takes the place of `steps`. A value below the first one is in the first class. |
| `symbol` | `"rect"` / `"circle"` | `"rect"` | Fills the cells (`rect`), or draws a circle in each cell that has a value, its area following the value — a punch card. The circles of a row lie on a line; they carry no label. |
| `data` | array |  | The cells as `[index, value]`, where `index` = y index × number of x categories + x index. |

<!-- example: heatmap -->

```json
{
  "x_axis_data": ["12a", "6a", "12p"],
  "y_axis_data": ["Sat", "Sun"],
  "series": {
    "min": 0,
    "max": 10,
    "data": [[0, 9], [1, 3], [2, 6], [3, 8], [4, 2], [5, 5]]
  }
}
```

## Funnel

`FunnelChart::from_json` — Stages of a process, one above the other. Every series is one stage: its value is the sum of its `data`.

<!-- keys: funnel -->

| Key | Type | Default | Description |
|---|---|---|---|
| `funnel_gap` | number | `2` | Gap between the stages in pixels. |
| `min_width` | number | `20` | Width of the narrowest stage in pixels. |
| `sort_ascending` | boolean | `false` | Puts the smallest value at the top instead of the largest. |
| `series_label_position` | `"inside"` / `"left"` / `"right"` | `"right"` | Where the labels go: inside the stages, or to their left or right. |
| `funnel_align` | `"left"` / `"center"` / `"right"` | `"center"` | Horizontal alignment of the stages. |

<!-- example: funnel -->

```json
{
  "series_label_position": "inside",
  "series_list": [
    {"name": "Show", "data": [100]},
    {"name": "Click", "data": [80]},
    {"name": "Order", "data": [20]}
  ]
}
```

## Waterfall

`WaterfallChart::from_json` — Bars that start where the previous one ended. The values are in `data`, one per category of `x_axis_data`, not in `series_list`.

<!-- keys: waterfall -->

| Key | Type | Default | Description |
|---|---|---|---|
| `label_show` | boolean | `true` | Shows the value of every bar as a label. |
| `connector_line_show` | boolean | `true` | Draws a dashed line from each bar to the next. |
| `connector_line_dash_array` | string | `"4,4"` | Dashes of the connector lines, as the `stroke-dasharray` of SVG takes them; `""` draws solid lines. |
| `bar_width_ratio` | number | `0.6` | Share of a category’s width taken by its bar, from 0 to 1. |
| `increase_color` | color | theme | Color of the bars that add to the total. |
| `decrease_color` | color | `#EE6666` | Color of the bars that take from the total. |
| `total_color` | color | theme | Color of the bars that show the running total. |
| `data` | array |  | One entry per category: a number (the change), or `[value, is_total]`, where a total shows the sum so far instead of a change. |

<!-- example: waterfall -->

```json
{
  "x_axis_data": ["Revenue", "Cost", "Tax", "Profit"],
  "data": [[900, false], [-345, false], [-108, false], [0, true]]
}
```

## Calendar

`CalendarChart::from_json` — A year (or any range of days) as a grid of day squares, colored by value. The chart is as large as its squares: `width` and `height` are not used, set `cell_size` and `cell_gap` instead.

<!-- keys: calendar -->

| Key | Type | Default | Description |
|---|---|---|---|
| `start_date` | string |  | First day shown, as `"YYYY-MM-DD"`. Default: January 1st of the current year. |
| `end_date` | string |  | Last day shown, as `"YYYY-MM-DD"`. Default: December 31st of the current year. |
| `min` | number | `0` | Value shown in `min_color`; `0` takes the smallest value of the data. |
| `max` | number | `0` | Value shown in `max_color`; `0` takes the largest value of the data. |
| `min_color` | color | `#EBEDF0` | Color of the days with the smallest value. |
| `max_color` | color | `#216E39` | Color of the days with the largest value. |
| `colors` | color[] |  | The colors of the scale, from the smallest value to the largest. Two or more take the place of `min_color` and `max_color`: the scale goes through all of them. |
| `steps` | integer 0–1000 | `0` | Number of classes of the same width the values are sorted into, each drawn in one color, instead of a continuous scale; `0` and `1` keep it continuous. With as many `colors` as steps, every class has one of them. |
| `thresholds` | number[] |  | The values where one class ends and the next begins; takes the place of `steps`. A value below the first one is in the first class. |
| `empty_color` | color | theme | Color of the days without a value. |
| `cell_size` | number | `13` | Side of a day square in pixels. |
| `cell_gap` | number | `3` | Gap between the squares in pixels. |
| `month_label_height` | number | `20` | Height of the row of month names above the grid. |
| `week_label_width` | number | `30` | Width of the column of weekday names left of the grid. |
| `show_dow_labels` | array | `[1, 3, 5]` | The weekdays that are named, as numbers from 0 (Sunday) to 6 (Saturday). |
| `data` | array |  | The values: `["YYYY-MM-DD", value]` for each day. |

<!-- example: calendar -->

```json
{
  "start_date": "2024-01-01",
  "end_date": "2024-03-31",
  "data": [["2024-01-05", 3], ["2024-02-10", 7], ["2024-03-21", 5]]
}
```

## Gauge

`GaugeChart::from_json` — A dial. Every series is one value on the scale (the first of its `data`): a pointer each, or a ring each with `multi_ring`.

<!-- keys: gauge -->

| Key | Type | Default | Description |
|---|---|---|---|
| `min` | number | `0` | Value at the start of the scale. |
| `max` | number | `100` | Value at the end of the scale. |
| `start_angle` | number | `225` | Angle the scale starts at, in degrees clockwise from 12 o’clock. `0` selects the default: write `360` to start at 12 o’clock. |
| `sweep_angle` | number | `270` | Angle the scale spans, in degrees clockwise. |
| `radius` | number | `0` | Outer radius in pixels; `0` fits the plot. |
| `arc_width` | number | `15` | Thickness of the arc in pixels. |
| `background_arc_color` | color | `#E6E6E6` | Color of the part of the arc that is not filled. |
| `show_pointer` | boolean | `true` | Draws the needle. |
| `pointer_color` | color |  | Color of the needle; by default the color of the first series. |
| `show_axis_label` | boolean | `true` | Writes the minimum and maximum at the ends of the arc. |
| `split_number` | integer 0–1000 | `5` | Number of intervals between the major ticks. |
| `value_formatter` | string | `"{c}"` | Format of the value shown in the center: `{c}` is the value. |
| `thresholds` | number[] |  | The values where one segment of the scale ends and the next begins. With thresholds the arc shows the segments, each in a color of its own, instead of the progress, and a pointer takes the color of the segment it points at. |
| `colors` | color[] |  | The colors of the segments, from the lowest to the highest. Default: the colors of the series. |
| `multi_ring` | boolean | `false` | Draws every series as a ring of progress of its own, one inside the other, with the values listed in the middle, instead of as pointers on one dial. |

<!-- example: gauge -->

```json
{
  "min": 0,
  "max": 100,
  "value_formatter": "{c}%",
  "series_list": [
    {"name": "CPU", "data": [72]}
  ]
}
```

## Treemap

`TreemapChart::from_json` — Rectangles as large as their values. Every series is one rectangle (the first of its `data`); `series_data` nests them.

<!-- keys: treemap -->

| Key | Type | Default | Description |
|---|---|---|---|
| `item_gap` | number | `2` | Gap between adjacent cells in pixels. |
| `series_data` | object[] |  | Nested data; when set it takes the place of `series_list`, and every branch is divided into its children. |

### `series_data[]`

<!-- keys: treemap.series_data -->

| Key | Type | Default | Description |
|---|---|---|---|
| `name` | string |  | Name of the node, shown as its label. |
| `value` | number |  | Value of a leaf; a node with children is as large as their sum. |
| `color` | color |  | Color of the node; by default taken from the palette. |
| `children` | object[] |  | The child nodes, each with the same keys. |

<!-- example: treemap -->

```json
{
  "series_data": [
    {
      "name": "Docs",
      "children": [
        {"name": "Guides", "value": 6},
        {"name": "API", "value": 4}
      ]
    },
    {"name": "Media", "value": 5}
  ]
}
```

## Box plot

`BoxPlotChart::from_json` — One box per category of `x_axis_data` and series, from five numbers. The data are in `box_series`, not in `series_list`.

<!-- keys: box_plot -->

| Key | Type | Default | Description |
|---|---|---|---|
| `box_series` | object[] |  | The series of boxes; takes the place of `series_list`. |

### `box_series[]`

<!-- keys: box_plot.box_series -->

| Key | Type | Default | Description |
|---|---|---|---|
| `name` | string |  | Name of the series, shown in the legend. |
| `index` | integer ≥ 0 |  | Position in the color palette; by default the position of the series. |
| `data` | array |  | One box per category: `[min, q1, median, q3, max]`. |

<!-- example: box_plot -->

```json
{
  "x_axis_data": ["Mon", "Tue"],
  "box_series": [
    {
      "name": "Latency",
      "data": [[655, 850, 940, 980, 1175], [672, 800, 845, 885, 1012]]
    }
  ]
}
```

## Sunburst

`SunburstChart::from_json` — A hierarchy as rings: every level a ring, every node an arc as long as its value.

<!-- keys: sunburst -->

| Key | Type | Default | Description |
|---|---|---|---|
| `series_data` | object[] |  | The roots of the hierarchy; several roots share the circle. |
| `radius` | number | `0` | Outer radius in pixels; `0` fits the plot. |
| `inner_radius` | number | `0` | Radius of the hole in the center. |
| `start_angle` | number | `0` | Angle the first root starts at, in degrees clockwise from 12 o’clock. |
| `level_thickness` | number[] |  | Relative thickness of each ring, from the center outwards; rings without one count as `1`. |

### `series_data[]`

<!-- keys: sunburst.series_data -->

| Key | Type | Default | Description |
|---|---|---|---|
| `name` | string |  | Name of the node, shown as its label. |
| `value` | number |  | Value of a leaf; a node with children is as large as their sum. |
| `color` | color |  | Color of the node; by default taken from the palette. |
| `children` | object[] |  | The child nodes, each with the same keys. |

<!-- example: sunburst -->

```json
{
  "inner_radius": 30,
  "series_data": [
    {
      "name": "Grandpa",
      "children": [
        {"name": "Uncle Leo", "value": 15},
        {
          "name": "Father",
          "children": [
            {"name": "Me", "value": 5},
            {"name": "Brother", "value": 1}
          ]
        }
      ]
    }
  ]
}
```

## Sankey

`SankeyChart::from_json` — Directed flows from left to right: nodes in columns, links as ribbons as thick as their value.

<!-- keys: sankey -->

| Key | Type | Default | Description |
|---|---|---|---|
| `nodes` | object[] |  | The nodes. Optional: nodes are derived from the names in `links`, in first-seen order; list them to fix their order or colors. |
| `links` | object[] |  | The directed flows between the nodes. |
| `node_width` | number | `16` | Width of the node rectangles in pixels. |
| `node_gap` | number | `8` | Vertical gap between the nodes of a column in pixels. |
| `link_opacity` | number | `0.45` | Opacity of the flow ribbons, from 0 to 1. |
| `node_align` | `"left"` / `"right"` / `"justify"` | `"left"` | Which column a node goes in: as far left as its sources allow (`left`, the default), as far right as its targets allow (`right`), or `left` with the end nodes in the last column (`justify`). |
| `link_gradient` | boolean | `false` | Fills every link with a gradient from the color of its source to the color of its target, instead of the color of its source. |
| `orient` | `"horizontal"` / `"vertical"` | `"horizontal"` | Direction of the flows: from left to right, or from top to bottom (the columns of nodes become rows). |

### `nodes[]`

<!-- keys: sankey.nodes -->

| Key | Type | Default | Description |
|---|---|---|---|
| `name` | string |  | Name of the node, shown as its label; links refer to it. |
| `color` | color |  | Color of the node; by default the palette color at its position. |

### `links[]`

<!-- keys: sankey.links -->

| Key | Type | Default | Description |
|---|---|---|---|
| `source` | string |  | Name of the node the link starts at. |
| `target` | string |  | Name of the node the link ends at. |
| `value` | number |  | Volume of the flow; a link without a positive value is left out. |

<!-- example: sankey -->

```json
{
  "link_gradient": true,
  "links": [
    {"source": "Coal", "target": "Electricity", "value": 25},
    {"source": "Gas", "target": "Electricity", "value": 15},
    {"source": "Electricity", "target": "Homes", "value": 30},
    {"source": "Electricity", "target": "Industry", "value": 10}
  ]
}
```

## Tree

`TreeChart::from_json` — A hierarchy as nodes joined by lines.

<!-- keys: tree -->

| Key | Type | Default | Description |
|---|---|---|---|
| `series_data` | object[] |  | The roots of the hierarchy; several roots are laid out side by side. |
| `orient` | `"LR"` / `"RL"` / `"TB"` / `"BT"` | `"LR"` | Where the root is, and which way the tree grows: root on the left (`LR`), on the right (`RL`), at the top (`TB`) or at the bottom (`BT`). |
| `symbol_size` | number | `6` | Radius of the node circles in pixels. |
| `layout` | `"orthogonal"` / `"radial"` | `"orthogonal"` | `orthogonal` lays the levels out side by side, by `orient`; `radial` puts the root in the middle and every level on a circle around it. |
| `edge_shape` | `"curve"` / `"polyline"` | `"curve"` | Shape of the links: curves, or lines with square corners (straight lines in a radial tree). |

### `series_data[]`

<!-- keys: tree.series_data -->

| Key | Type | Default | Description |
|---|---|---|---|
| `name` | string |  | Name of the node, shown as its label. |
| `value` | number |  | Value of the node, shown in its tooltip; a node with children takes the sum of theirs. |
| `color` | color |  | Color of the node; by default taken from the palette. |
| `children` | object[] |  | The child nodes, each with the same keys. |

<!-- example: tree -->

```json
{
  "orient": "TB",
  "series_data": [
    {
      "name": "root",
      "children": [
        {
          "name": "a",
          "children": [
            {"name": "a1"},
            {"name": "a2"}
          ]
        },
        {"name": "b"}
      ]
    }
  ]
}
```

## Graph

`GraphChart::from_json` — A network: nodes joined by links of any shape.

<!-- keys: graph -->

| Key | Type | Default | Description |
|---|---|---|---|
| `nodes` | object[] |  | The nodes. Optional: nodes are derived from the names in `links`, in first-seen order. |
| `links` | object[] |  | The edges between the nodes. |
| `symbol_size` | number | `10` | Radius of a node circle in pixels; nodes with a `value` are scaled from it. |
| `layout` | `"force"` / `"circular"` | `"force"` | Placement of the nodes: by a force simulation (`force`, the default) or evenly on a circle (`circular`). |
| `categories` | string[] |  | Names of the node categories (a node refers to one by `category`); when set they are the legend. |

### `nodes[]`

<!-- keys: graph.nodes -->

| Key | Type | Default | Description |
|---|---|---|---|
| `name` | string |  | Name of the node, shown as its label; links refer to it. |
| `value` | number |  | Importance of the node: larger values draw larger circles. |
| `color` | color |  | Color of the node; by default the palette color of its category, or of its position. |
| `category` | integer ≥ 0 |  | Index into `categories`; nodes of a category share a color. |

### `links[]`

<!-- keys: graph.links -->

| Key | Type | Default | Description |
|---|---|---|---|
| `source` | string |  | Name of the node the link starts at. |
| `target` | string |  | Name of the node the link ends at. |
| `value` | number |  | Weight of the edge: larger values draw thicker lines. |

<!-- example: graph -->

```json
{
  "categories": ["Team", "Tool"],
  "nodes": [
    {"name": "Ann", "category": 0, "value": 3},
    {"name": "Bob", "category": 0},
    {"name": "Git", "category": 1}
  ],
  "links": [
    {"source": "Ann", "target": "Bob", "value": 2},
    {"source": "Ann", "target": "Git"},
    {"source": "Bob", "target": "Git"}
  ]
}
```

## Parallel

`ParallelChart::from_json` — Parallel coordinates: `x_axis_data` names the dimensions (one vertical axis each), and every series is one record with a value for each dimension. It has no keys of its own.

<!-- example: parallel -->

```json
{
  "x_axis_data": ["Price", "Weight", "Rating"],
  "series_list": [
    {"name": "A", "data": [120, 3.2, 4.5]},
    {"name": "B", "data": [90, 4.1, 3.8]}
  ]
}
```

## Theme river

`ThemeRiverChart::from_json` — A streamgraph: every series is a stream over the categories of `x_axis_data`, as thick as its value.

<!-- keys: theme_river -->

| Key | Type | Default | Description |
|---|---|---|---|
| `stream_opacity` | number | `0.85` | Opacity of the streams, from 0 to 1. |

<!-- example: theme_river -->

```json
{
  "x_axis_data": ["Jan", "Feb", "Mar", "Apr"],
  "series_list": [
    {"name": "News", "data": [10, 25, 18, 30]},
    {"name": "Sport", "data": [15, 12, 22, 16]}
  ]
}
```

## Histogram

`HistogramChart::from_json` — The distribution of a sample over equal-width bins. The `data` of a series is the raw sample; `x_axis_min` and `x_axis_max` fix the range of the bins.

<!-- keys: histogram -->

| Key | Type | Default | Description |
|---|---|---|---|
| `bin_count` | integer 0–1000 | `0` | Number of bins; `0` chooses it from the size of the sample, with round bin edges. |
| `bin_width` | number > 0 |  | Width of a bin; wins over `bin_count`. The bin edges are multiples of it. |
| `percent` | boolean | `false` | Shows the share of each series’ sample, in percent, instead of a count. |
| `bar_gap` | number | `1` | Gap between adjacent bars in pixels. |

<!-- example: histogram -->

```json
{
  "bin_width": 5,
  "x_axis_title": "Height (cm)",
  "series_list": [
    {
      "name": "Adults",
      "data": [162.4, 171, 168.3, 175.9, 158.2, 169.7, 173.1, 166.8]
    }
  ]
}
```

## Polar bar

`PolarBarChart::from_json` — Bars on polar axes, over the categories of `x_axis_data`. The value axis is the first of `y_axis_configs`.

<!-- keys: polar_bar -->

| Key | Type | Default | Description |
|---|---|---|---|
| `category_axis` | `"angle"` / `"radius"` | `"angle"` | The axis the categories lie on: around the circle (`angle`, bars grow outwards) or from the center outwards (`radius`, bars run around the circle). |
| `radius` | number > 0 |  | Largest outer radius; by default the chart fills the plot. |
| `inner_radius` | number |  | Radius of the hole in the center. Default: `0` with the categories on the angle axis, a quarter of the radius otherwise. |
| `start_angle` | number | `0` | Angle both axes start at, in degrees clockwise from 12 o’clock. |
| `end_angle` | number |  | Angle the value axis ends at, with the categories on the radius axis. Default: 270 degrees after `start_angle`; at most a full turn. |
| `round_cap` | boolean | `false` | Rounds the ends of the bars that run around the circle. |
| `category_gap` | number | `0.2` | Share of a category’s slot left free between it and its neighbours, from 0 to 0.9. |

<!-- example: polar_bar -->

```json
{
  "category_axis": "radius",
  "round_cap": true,
  "y_axis_configs": [
    {"axis_max": 100}
  ],
  "x_axis_data": ["Sleep", "Steps", "Water"],
  "series_list": [
    {"name": "Done", "data": [92, 74, 61], "label_show": true}
  ]
}
```

## Chord

`ChordChart::from_json` — Flows between nodes around a circle: every node an arc as long as the flows through it, every link a ribbon across the circle.

<!-- keys: chord -->

| Key | Type | Default | Description |
|---|---|---|---|
| `nodes` | object[] |  | The nodes, clockwise. Optional: nodes are derived from the names in `links`, in first-seen order; list them to fix their order or colors. |
| `links` | object[] |  | The flows between the nodes. |
| `radius` | number > 0 |  | Largest outer radius; by default the diagram fills the plot. |
| `node_width` | number | `12` | Thickness of the ring of nodes in pixels. |
| `node_gap` | number | `3` | Gap between neighbouring nodes in degrees. |
| `start_angle` | number | `0` | Angle the first node starts at, in degrees clockwise from 12 o’clock. |
| `link_opacity` | number | `0.5` | Opacity of the ribbons, from 0 to 1. |
| `link_gradient` | boolean | `false` | Fills every ribbon with a gradient from the color of its source to the color of its target, instead of the color of its source. |

### `nodes[]`

<!-- keys: chord.nodes -->

| Key | Type | Default | Description |
|---|---|---|---|
| `name` | string |  | Name of the node, shown as its label; links refer to it. |
| `color` | color |  | Color of the node; by default the palette color at its position. |

### `links[]`

<!-- keys: chord.links -->

| Key | Type | Default | Description |
|---|---|---|---|
| `source` | string |  | Name of the node the link starts at. |
| `target` | string |  | Name of the node the link ends at. |
| `value` | number |  | Volume of the flow; a link without a positive value is left out. |

<!-- example: chord -->

```json
{
  "link_gradient": true,
  "links": [
    {"source": "Asia", "target": "Europe", "value": 60},
    {"source": "Asia", "target": "Americas", "value": 45},
    {"source": "Europe", "target": "Americas", "value": 50}
  ]
}
```

## Gantt

`GanttChart::from_json` — Tasks as bars along a time axis: a row for each task, or several tasks on a row. The axis is a time axis (numbers are unix seconds) unless `x_axis_type` is `value`; `x_axis_min` and `x_axis_max` fix its range, `x_axis_formatter` and `x_axis_time_offset` its labels. It takes no `series_list`: the legend is the categories of the tasks.

<!-- keys: gantt -->

| Key | Type | Default | Description |
|---|---|---|---|
| `tasks` | object[] |  | The tasks; the rows are taken from them, top to bottom in the order they are first met. |
| `bar_height` | number |  | Height of the bars in pixels; by default 60% of the height of a row. |
| `radius` | number | `3` | Corner radius of the bars. |
| `label_show` | boolean | `true` | Writes the name of a task on its bar, or beside it when it does not fit there. A task on a row of its own is not named again: the row tells its name. |
| `now` | x value |  | A moment marked by a dashed line across the chart, such as today. |

### `tasks[]`

<!-- keys: gantt.tasks -->

| Key | Type | Default | Description |
|---|---|---|---|
| `name` | string |  | Name of the task. |
| `start` | x value |  | When the task starts; a task without a start is left out. |
| `end` | x value |  | When the task ends. Without an end, or with the end at the start, the task is a milestone, drawn as a diamond. |
| `row` | string |  | The row the task is drawn on; tasks with the same row share it. By default a row of its own, named after the task. |
| `category` | string |  | Category of the task: the tasks of a category share a color, and the categories are the legend. |
| `progress` | number |  | How much of the task is done, from 0 to 1: that part of the bar is in the full color, the rest lighter. |
| `color` | color |  | Color of the task; by default the color of its category. |

<!-- example: gantt -->

```json
{
  "now": "2024-03-19",
  "tasks": [
    {
      "name": "Research",
      "category": "Plan",
      "start": "2024-03-04",
      "end": "2024-03-08",
      "progress": 1
    },
    {
      "name": "Design",
      "category": "Build",
      "start": "2024-03-07",
      "end": "2024-03-22",
      "progress": 0.6
    },
    {"name": "Sign-off", "category": "Build", "start": "2024-03-22"},
    {"name": "Launch", "category": "Plan", "start": "2024-03-25", "end": "2024-03-29"}
  ]
}
```

## Map

`MapChart::from_json` — A map of regions colored by their values. The regions come with the options, as GeoJSON: the chart has no map data of its own. It takes no `series_list`.

<!-- keys: map -->

| Key | Type | Default | Description |
|---|---|---|---|
| `geo_json` | object |  | The regions: a GeoJSON `FeatureCollection` (or a single `Feature`) of `Polygon` and `MultiPolygon` geometries, with longitudes and latitudes in degrees. Other geometries are left out. |
| `name_property` | string | `"name"` | The property of a GeoJSON feature that is the name of its region; without it, the `id` of the feature. |
| `data` | array |  | The values: `["name of the region", value]` for each region that has one. |
| `projection` | `"mercator"` / `"equirectangular"` | `"mercator"` | How longitudes and latitudes are laid on the plane: Mercator keeps the shapes (and cuts the map off at 85° north and south), equirectangular takes them as they are. |
| `min` | number | `0` | Value at the start of the scale. With `min` and `max` both `0` the scale goes from the smallest value to the largest. |
| `max` | number | `0` | Value at the end of the scale. |
| `min_color` | color | theme | Color of the smallest value: by default a light tint of the first color of the theme. |
| `max_color` | color | theme | Color of the largest value: by default the first color of the theme. |
| `colors` | color[] |  | The colors of the scale, from the smallest value to the largest. Two or more take the place of `min_color` and `max_color`: the scale goes through all of them. |
| `steps` | integer 0–1000 | `0` | Number of classes of the same width the values are sorted into, each drawn in one color, instead of a continuous scale; `0` and `1` keep it continuous. |
| `thresholds` | number[] |  | The values where one class ends and the next begins; takes the place of `steps`. A value below the first one is in the first class. |
| `empty_color` | color | theme | Color of the regions without a value. |
| `border_color` | color | theme | Color of the borders of the regions: by default the background color. |
| `border_width` | number | `1` | Width of the borders of the regions; `0` draws none. |
| `label_show` | boolean | `false` | Writes the name of every region on it; a name that would run into another one is left out. |
| `visual_map_show` | boolean | `true` | Shows the scale of the colors beside the map: a bar from the smallest value to the largest, or a swatch for every class. |

<!-- example: map -->

```json
{
  "label_show": true,
  "thresholds": [100, 300],
  "colors": ["#deebf7", "#9ecae1", "#3182bd"],
  "data": [["West", 80], ["East", 420]],
  "geo_json": {
    "type": "FeatureCollection",
    "features": [
      {
        "type": "Feature",
        "properties": {"name": "West"},
        "geometry": {
          "type": "Polygon",
          "coordinates": [[[100, 20], [106, 20], [107, 26], [101, 27], [100, 20]]]
        }
      },
      {
        "type": "Feature",
        "properties": {"name": "East"},
        "geometry": {
          "type": "Polygon",
          "coordinates": [[[106, 20], [113, 21], [112, 27], [107, 26], [106, 20]]]
        }
      }
    ]
  }
}
```

## Multi chart

`MultiChart::from_json` — Several charts in one SVG, one below the other or at positions of their own. It does not take the common options.

<!-- keys: multi -->

| Key | Type | Default | Description |
|---|---|---|---|
| `theme` | string |  | Theme of every child that does not set its own. |
| `margin` | margin | `10` | Margin around the whole composition. |
| `gap` | number | `10` | Vertical gap between the children that are placed automatically. |
| `background_color` | color |  | Background color of the whole composition; without one only the children draw their backgrounds. |
| `child_charts` | array |  | The charts, drawn one below the other: each one the options of its chart type, plus the keys below. |
| `compact` | boolean | `false` | Emits compact SVG for the composition and every child. |

### `child_charts[]`

<!-- keys: multi.child_charts -->

| Key | Type | Default | Description |
|---|---|---|---|
| `type` | string |  | Type of the child: `bar` (the default), `line`, `horizontal_bar`, `pie`, `radar`, `table`, `scatter`, `candlestick`, `heatmap`, `funnel`, `waterfall`, `calendar`, `gauge`, `treemap`, `box_plot`, `sunburst`, `sankey`, `tree`, `graph`, `parallel`, `theme_river`, `histogram`, `polar_bar`, `chord`, `gantt` or `map`. |
| `x` | number |  | Horizontal position of the child; with `x` or `y` set, the child is placed there instead of below the previous one. |
| `y` | number |  | Vertical position of the child. |

<!-- example: multi -->

```json
{
  "gap": 10,
  "child_charts": [
    {
      "type": "bar",
      "x_axis_data": ["Mon", "Tue"],
      "series_list": [
        {"name": "Email", "data": [120, 132]}
      ]
    },
    {
      "type": "pie",
      "height": 300,
      "series_list": [
        {"name": "Email", "data": [40]},
        {"name": "Search", "data": [38]}
      ]
    }
  ]
}
```

## Keys for tools

Two more keys are accepted by every chart and do nothing, so that a tool around the library (the web editor, for one) can keep them in the same document:

<!-- keys: envelope -->

| Key | Type | Default | Description |
|---|---|---|---|
| `type` | string |  | Chart type, for tools that pass one document to several chart types. Ignored by `from_json`. |
| `quality` | number |  | Image quality, for tools that also encode the image. Ignored by `from_json`. |
