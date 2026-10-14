# Moving from 1.x to 2.0

This is the 2.0 section of the changelog, kept here so `make changelog` can
regenerate [CHANGELOG.md](../CHANGELOG.md) from the commit history.

## 2.0.0

The options of a chart are grouped into structs of their own, and text is
measured without `fontdue`: the breaking changes below are the migration
guide from 1.x. The keys of a JSON document are unchanged, and every SVG
snapshot of 1.3.0 is byte-identical. The compatibility policy of 1.x
([moving from 0.7 to 1.0](./migrating-1.0.md)) carries over to 2.x, the option structs included.

### Breaking changes

#### Options are grouped by what they belong to

The shared options of a chart are no longer 68 flat fields of `ChartBase`.
Those of the title, the sub-title, the legend, the x axis, the grid, the
series and the tooltips are structs of their own (`TitleConfig`,
`LegendConfig`, `XAxisConfig`, `GridConfig`, `SeriesConfig`,
`TooltipConfig`), and every text has its font in a `FontConfig`. Charts
still expose them through `Deref`: `chart.title.font.size = 18.0`.

**The JSON keys are unchanged**, and so is every SVG: a key is the path of
the field joined with `_` (`title_font_size` is `title.font.size`).

| 1.x | 2.0 |
|-----|-----|
| `title_text`, `title_margin`, `title_align`, `title_height` | `title.text`, `title.margin`, `title.align`, `title.height` |
| `title_font_size`, `title_font_color`, `title_font_weight` | `title.font.size`, `title.font.color`, `title.font.weight` |
| `sub_title_*` | `sub_title.*`, as the title |
| `legend_font_size`, `legend_font_color`, `legend_font_weight` | `legend.font.size`, `legend.font.color`, `legend.font.weight` |
| `legend_align`, `legend_margin`, `legend_category`, `legend_show`, `legend_position` | `legend.align`, `legend.margin`, `legend.category`, `legend.show`, `legend.position` |
| `x_axis_font_size`, `x_axis_font_color`, `x_axis_font_weight` | `x_axis.font.size`, `x_axis.font.color`, `x_axis.font.weight` |
| `x_axis_data`, `x_axis_height`, `x_axis_stroke_color`, `x_axis_name_gap`, `x_axis_name_rotate`, `x_axis_margin`, `x_axis_label_overflow`, `x_axis_values`, `x_axis_min`, `x_axis_max`, `x_axis_formatter`, `x_axis_time_offset`, `x_axis_title`, `x_axis_hidden` | `x_axis.data`, `x_axis.height`, … : the name after `x_axis_` |
| `x_boundary_gap` | `x_axis.boundary_gap` |
| `x_axis_type` | `x_axis.kind` |
| `grid_stroke_color`, `grid_stroke_width` | `grid.stroke_color`, `grid.stroke_width` |
| `series_stroke_width`, `series_colors`, `series_symbol`, `series_smooth`, `series_fill` | `series.stroke_width`, `series.colors`, `series.symbol`, `series.smooth`, `series.fill` |
| `series_label_font_size`, `series_label_font_color`, `series_label_font_weight` | `series.label.font.size`, `series.label.font.color`, `series.label.font.weight` |
| `series_label_formatter`, `series_label_hide_overlap` | `series.label.formatter`, `series.label.hide_overlap` |
| `tooltip_show` | `tooltip.show` |
| `y_axis_configs[i].axis_font_size`, `axis_font_color`, `axis_font_weight` | `y_axis_configs[i].font.size`, `font.color`, `font.weight` |
| `y_axis_configs[i].axis_min` and the other `axis_*` | `y_axis_configs[i].min`, … : without the `axis_` |
| `TableChart`: `header_font_*`, `header_row_padding`, `header_row_height`, `header_background_color` | `header.font.*`, `header.row_padding`, `header.row_height`, `header.background_color` |
| `TableChart`: `body_font_*`, `body_row_padding`, `body_row_height`, `body_background_colors` | `body.font.*`, `body.row_padding`, `body.row_height`, `body.background_colors` |
| `HeatmapChart::series` | `HeatmapChart::heatmap_series` |

Unchanged: `width`, `height`, `x`, `y`, `margin`, `background_color`,
`is_light`, `font_family`, `compact`, `empty_text`, `stack_percent`,
`y_axis_hidden`, `animation`, `series_list`, and the fields a chart has of
its own (`radius`, `series_label_position`, …).

- `Theme` is grouped the same way, out of the same structs: `title`,
  `sub_title`, `legend`, `x_axis`, `y_axis`, `grid` and `series`
  (`title_font_size` is `title.font.size`, `y_axis_split_number` is
  `y_axis.split_number`). A theme can so hold any shared option — the
  position of the legend, the alignment of the title, the format of the
  labels — and serialized with serde it is nested as it is grouped.
- `ChartBase`, `BarChart`, `CandlestickChart` and `ScatterChart` no longer
  derive `Serialize` / `Deserialize`. A chart is read with `from_json`: the
  derive read another format, and only three of the 27 charts had it.
- `HeatmapChart::series` is `heatmap_series`: `series` is the group of the
  options of the series, on every chart.
- `MarkLine`, `MarkArea`, `SeriesBand` and the components `Grid`, `Axis`,
  `Rect` and `Circle` have new fields: a struct literal of one needs
  `..Default::default()`.

#### Fonts

- `get_font` is removed, and `fontdue` with it: text is measured with
  `ttf-parser`, straight from the bytes of a font. Fonts are registered with
  `add_fonts` and text is measured with `measure_text_width_family` as
  before, and the measurements are the same to the pixel — every snapshot is
  byte-identical. They were checked against `fontdue` for every character of
  the embedded font and of over 300 other fonts (TrueType, CFF, collections,
  variable).
- A character is looked up in the Unicode tables of a font, or in the
  symbol table of a symbol font. `fontdue` also read the other legacy
  tables (Mac Roman, Big5, GB 2312) and, in fonts that have them, measured
  some characters as the glyph of another.
- `Error::ParseFont` carries the message of `ttf-parser`.
- A font is registered under the names of its family (see "Added"), which
  `get_font_families` lists: more names than before. Of several faces of a
  family given in one `add_fonts` call the regular one is measured with,
  not the last one.

### Fixed

- `add_theme` from several threads at once keeps every theme: it read the
  themes, added its own and wrote them back, so that one of two calls at
  the same time could write over the theme of the other. `add_fonts` is
  guarded the same way for the fonts of the rasterizer.
- A scatter or box plot chart with a logarithmic y axis and no values no
  longer panics (`index out of bounds`): the axis has no label to take its
  width from.
- `TreeChart`: `orient` is read whatever its case, as it is checked:
  `"tb"` was accepted and then drawn as `LR`.
- `PieChart`: the label of a slice whose angle runs past a full turn (with a
  `start_angle`) or below zero is put on the side of the pie it is on.
- A series with a `start_index` near `usize::MAX` and more than one value no
  longer overflows in bar and line charts.

### Added

- `add_fonts` registers every face of a font collection (`.ttc`), and a
  font under each name of its family: the typographic family and the
  family, in every language they are given in (`"PingFang SC"` and
  `"苹方-简"`) — the names the rasterizer and a browser find the font by.
  Only the first face of a collection was registered, under its full name
  without the words of a weight (still a name of the first face).
- Fonts whose names are in Mac Roman only (`Helvetica.ttc`, `Menlo.ttc`,
  `STHeiti`) are registered; `add_fonts` used to return `Ok` and skip them.
- Symbol fonts (Wingdings), which have no table of Unicode characters, are
  measured by their symbol table, as the rasterizer shapes their text.
- Style options for what used to be fixed values. Left out, each of them is
  that value, and no output changes:
  - `series_fill_opacity`: how opaque the fill of an area is (0.39 under a
    line, 0.2 in a radar chart).
  - `grid_stroke_dash_array`: dashed grid lines — the grid of a chart with
    an x and a y axis, of a gantt, radar and polar bar chart, the lines of a
    punch card and the axes of a parallel chart.
  - `x_axis_stroke_width`, and `axis_stroke_width` in `y_axis_configs`: the
    width of the axis lines and ticks (and of the zero line of a waterfall
    chart).
  - `tooltip_font_size`, `tooltip_font_color`, `tooltip_font_weight`: the
    font of the hover tooltips, which is otherwise that of the data labels.
  - `color`, `stroke_width` and `stroke_dash_array` of a mark line; `color`
    and `opacity` of a mark area (a color with an alpha of its own keeps
    it).
  - `stroke_width` of an `error_bar`.
  - `PieChart`: `ring_gap`, the gap between the rings of nested pies.
  - `TableChart`: `border_width`.
  - `WaterfallChart`: `connector_line_dash_array`.
- A theme can hold any shared option, not only the colors and sizes it had
  fields for (see the breaking changes); a `TableChart` takes the title and
  the sub-title of its theme whole as well.
- `SankeyChart`: `orient` `vertical` runs the flows from top to bottom, the
  columns of nodes becoming rows.
- `HeatmapChart`: `series.symbol` (`heatmap_series.symbol` in Rust)
  `circle` draws a punch card — a circle in
  every cell that has a value, its area following the value, the circles of
  a row on a line — instead of filled cells.
- `MapChart`: a map of regions colored by their values. The regions are
  GeoJSON given by the caller (`geo_json` in the options, or
  `MapRegion::from_geo_json`): `Polygon` and `MultiPolygon` features, with
  their holes and islands; the library has no map data of its own. Mercator
  (the default) or equirectangular projection, fitted to the plot. The
  colors are a scale like the heatmap's (`min_color` / `max_color`,
  `colors`, `steps`, `thresholds`), shown beside the map; `label_show`
  names the regions. Supports tooltips with `data-*` attributes. Available
  in `MultiChart` as `"type": "map"`.
- `Shape` component: a filled shape of several closed outlines, an outline
  inside another one being a hole in it.
- `TreeChart`: `orient` also takes `RL` and `BT` (the root on the right, or
  at the bottom); `layout` `radial` puts the root in the middle and every
  level on a circle around it, with the names along the spokes;
  `edge_shape` `polyline` draws the links with square corners.
- `CandlestickChart`: `candlestick_style` `ohlc` draws OHLC bars — a line
  from the lowest to the highest price with a tick at the open and one at
  the close — instead of candles.
- `GaugeChart`: a pointer for every series (their values listed below the
  center), instead of the first one only; `thresholds` and `colors` split
  the scale into segments of their own colors, the pointer taking the color
  of the segment it points at; `multi_ring` draws every series as a ring of
  progress of its own, one inside the other.
- Color scales of several colors and of classes in `HeatmapChart` (under
  `series`) and `CalendarChart`: `colors` lists the colors the scale goes
  through, `steps` sorts the values into that many classes of the same
  width, each in one color, and `thresholds` gives the classes ends of
  one's own. The labels of a heatmap with such a scale take the light font
  on dark cells.
- `ScatterChart`: `regression` (`linear`, `exponential`, `logarithmic` or
  `polynomial` with `regression_order`) draws the curve that fits the points
  of each series best, by least squares, in the color of the series;
  `regression_label_show` writes its formula at its end.
- Error bars: a series' `error_bar` (`{"lower": [...], "upper": [...]}`,
  like `band`) draws a line with a cap at both ends over each bar, line
  point or scatter point. The y axis makes room for the bounds, and
  tooltips and `data-lower` / `data-upper` tell them.
- `GanttChart`: tasks as bars along a time axis, for project plans,
  schedules and timelines. A task has a row of its own or shares one
  (`row`); `category` colors the tasks and makes the legend, `progress`
  shows how much is done, a task without an `end` is a milestone and `now`
  marks a moment across the chart. Names are written on the bars, or beside
  them where there is room. The axis is the time axis of the line chart
  (`x_axis_min` / `x_axis_max`, `x_axis_formatter`, `x_axis_time_offset`),
  or one of plain numbers. Supports tooltips with `data-*` attributes and
  animation. Available in `MultiChart` as `"type": "gantt"`.
- Step lines: a series' `step` (`start`, `middle` or `end`) draws its line
  level between two points, changing value at a corner. The area under the
  line, stacked areas and bands step along; the markers stay on the points.
  `step` wins over `smooth`.
- `stack_percent`: the series of a stack are shown as their shares of it,
  every stack adding up to 100% on an axis in percent (bar, horizontal bar,
  line and polar bar charts). Labels and tooltips read in percent too.
- `BarChart`: `series_label_position` (`top` or `inside`) puts the value
  labels in the middle of the bars; it is the default with `stack_percent`.
  Inside labels of a `HorizontalBarChart` are centered on their own segment
  of a stack, not on the whole bar.
- `PieChart`: `end_angle`. The slices share the part of the circle from
  `start_angle` to it — a half doughnut with `-90` and `90` — and the chart
  is sized and centered on what is drawn.
- `PieChart`: nested pies. A series' `ring` puts its slice on a ring of its
  own: the series of a ring share a full turn among themselves, and the
  rings split the room from `inner_radius` to `radius`, the lowest number
  innermost. The slices of the inner rings are named on them (their name by
  default, `series_label_formatter` otherwise); the outermost ring keeps its
  labels and their lines. Slices carry `data-ring`.
- `axis_inverse` in `y_axis_configs`: the smallest value at the top of the
  axis and the largest at the bottom, as a ranking (bump chart) wants it.
  Lines, areas, bars, mark lines and bands follow; also the value axis of a
  horizontal bar chart and both axes of a scatter chart.

### Performance

- A font costs the memory of its file. `fontdue` read the outline of every
  glyph when a font was added and kept them all: 3.5 MB for the embedded
  Roboto and 255 MB for a 22 MB CJK font, next to its bytes. A glyph is now
  read when it is first measured, and only its box is kept, so adding that
  font no longer takes 200 ms of parsing either. Wrapping the text of a
  table is about 30% faster.
