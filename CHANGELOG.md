# Changelog

## 2.0.0

The options of a chart are grouped into structs of their own, and text is
measured without `fontdue`: the breaking changes below are the migration
guide from 1.x. The keys of a JSON document are unchanged, and every SVG
snapshot of 1.3.0 is byte-identical. The compatibility policy of 1.x (see
1.0.0) carries over to 2.x, the option structs included.

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

## 1.3.0

### Fixed

- `ScatterChart`: an object in `series_symbols` (`{"type": "triangle"}`) is
  drawn as that shape, like the bare string; it was always a circle.
- A data label on a point at the edge of the plot is moved inside the canvas
  instead of being cut off.
- A scatter point with a missing coordinate is skipped instead of being drawn
  at a bogus position.
- Builds cleanly with Rust 1.99 (`pie_chart.rs` no longer resolves `f32::MAX`
  to the deprecated module constant).

### Changed

- `TableChart::from_json` rejects `font_color`, `font_weight` and `indexes`
  at the top level like any other unknown key: they are keys of the
  `cell_styles` items, and were accepted and ignored next to them.
- Compact output no longer merges adjacent paths that carry `data-*`
  attributes, so every shape that stands for a piece of data stays an element
  of its own (no output of the existing charts changes).

### Added

- JSON options reference: [`docs/json.md`](./docs/json.md) (and
  [`docs/json-zh.md`](./docs/json-zh.md)) lists every key `from_json`
  accepts, for every chart, with its type, its default and what it does, and
  an example per chart. Tests keep it in step with the code: a key without a
  row (or a row without a key) fails, every example is rendered, and every
  accepted key has to change the parsed chart.
- `TableChart`: `body_font_weight`, and `background_color` (the color behind
  the title) in JSON. Both keys were accepted before, but not read.
- `ChordChart`: a chord diagram of the flows between nodes. The nodes
  (`nodes`, or derived from the names in `links`) are arcs of a circle as
  long as the flows through them; every link is a ribbon across the circle,
  as wide at both ends as its `value`, in the color of its source or as a
  source→target gradient (`link_gradient`, `link_opacity`). The ends of the
  links are ordered along their node so that the ribbons do not cross next
  to it. `node_width`, `node_gap`, `start_angle` and `radius` shape the
  diagram; labels take `series_label_formatter`. Supports tooltips with
  `data-*` attributes and animation. Available in `MultiChart` as
  `"type": "chord"`.
- `Ribbon` component: the band between two arcs of a circle, which the
  links of a chord diagram are made of.
- `PolarBarChart`: a bar chart on polar axes. The categories (`x_axis_data`)
  go around the circle and the bars grow outwards, or — with
  `category_axis: "radius"` — every category is a ring and the bars run
  around the circle (`end_angle`, `round_cap`). Series sit side by side or
  stack by `stack` name; `start_angle`, `inner_radius`, `radius` and
  `category_gap` shape the plot, and the value axis takes its range, split
  number and label format from `y_axis_configs`. Supports data labels,
  per-bar `colors`, tooltips with `data-*` attributes and animation.
  Available in `MultiChart` as `"type": "polar_bar"`.
- `Sector` component: an annular sector drawn with exact arcs (also a full
  ring, a wedge, or with round caps), which the polar bars are made of.
- Bands around lines: a series' `band` (`{"lower": [...], "upper": [...]}`,
  `SeriesBand` in Rust) fills the area between a lower and an upper bound of
  each point in the color of the series — confidence intervals, forecast
  ranges, min–max envelopes. The band is smooth when its line is, works on
  category, value and time x axes and for the line series of a bar chart,
  widens the y axis to fit, breaks at a `null` bound, and is drawn alone for
  a series without `data`. Tooltips tell the bounds, and the hover targets
  carry `data-lower` / `data-upper`.
- `HistogramChart`: the distribution of a sample over equal-width bins. The
  series data is the raw sample; the bins are chosen from it (Sturges' rule,
  edges rounded to 1 / 2 / 5 × 10ⁿ) or set with `bin_width` / `bin_count`,
  over the data range or `x_axis_min` / `x_axis_max`. `percent` shows shares
  instead of counts; several series overlap (translucent) or stack by
  `stack` name. The x axis ticks sit on the bin edges. Available in
  `MultiChart` as `"type": "histogram"`.
- Continuous x axes on line and bar charts: `x_axis_values` (shared) or a
  series' `x_values` place every point at its x value instead of in evenly
  spaced category slots, so unevenly sampled data keeps its real spacing.
  `x_axis_type` is `value` (numbers) or `time` (unix seconds; JSON also
  takes date strings such as `"2024-01-05 08:30"`, which select the time
  axis by themselves). Ticks land on round values or round times (seconds
  up to years, months and years on real calendar boundaries);
  `x_axis_min` / `x_axis_max` fix the range, `x_axis_formatter` formats the
  labels (`{c}`, or a `%Y-%m-%d %H:%M`-style pattern on a time axis) and
  `x_axis_time_offset` shows timestamps in a zone other than UTC. Bars are as
  wide as the smallest gap between two x values.
- Axis titles: `x_axis_title` below the x axis and `axis_title` in a y axis
  config, written along its axis (rotated), on line, bar, horizontal bar,
  candlestick, scatter, waterfall, box plot and heatmap charts.
- Bubble charts: `ScatterChart::bubble` reads the series data as
  `[x, y, size]` triples and scales each circle's area by its size, between
  `bubble_min_size` and `bubble_max_size`.
- `series_label_hide_overlap` drops a data label that would overlap one
  already drawn (line, bar and horizontal bar charts).

## 1.2.0

### Added

- Compact output: `compact: true` on every chart (and `MultiChart`) or
  `charts_rs::compact_svg()` rewrites the SVG like an optimizer would —
  no whitespace, relative path data, merged grid lines, hoisted shared
  attributes, dropped defaults — for the same picture in ~20–30% fewer
  bytes.

### Performance

- Path coordinates are written from a stack buffer straight into the SVG.
  A line series formats labels and `data-*` attributes only when they are
  shown, and a non-stacked line no longer allocates a zero-filled
  accumulator. Glyph-measurement hits no longer allocate a cache key, a
  horizontal axis remembers the width of its joined labels, and graph node
  names are indexed in a `HashMap`. The rendered SVG is unchanged.

## 1.1.0

### Fixed

- Bar and horizontal bar charts draw negative values from the 0 line instead
  of the axis bottom; stacked negatives no longer produce negative `height`
  attributes, and an axis that crosses 0 now puts 0 on a tick (an all-negative
  range extends up to 0).
- `animation.easing` is sanitized in the treemap, sunburst and sankey charts
  too, closing a `<style>` breakout with untrusted JSON.
- `BarChart` keeps the right y axis scale when `y_axis_hidden` is set, so
  series bound to `y_axis_index: 1` are no longer drawn with zero height.
- `MultiChart::from_json` no longer fails when neither the top level nor a
  child sets `theme`; an unknown child `type` is rejected instead of being
  rendered as a bar chart.
- Waterfall total bars label the running sum instead of `0`.
- Horizontal bar charts place a shorter series on the rows of its own
  categories instead of shifting it.
- Mark points skip missing (`null`) points when locating the min/max label.
- Panics on extreme input: tick rounding of values beyond `i32` range, an
  empty x axis with `x_boundary_gap: false`, a table with an empty
  `body_background_colors`, and a stacked series with a huge `start_index`.
- NaN/inf never reach the SVG: non-finite series values are treated as
  missing, pie totals and candlesticks skip missing points, and a single
  point without boundary gap no longer divides by zero.
- Opacity is written with two decimals, so alphas below 13/255 are no longer
  rounded down to fully transparent (snapshots updated accordingly).
- Table sub titles honour `sub_title_align`; gauge labels show the raw value
  while only the needle is clamped to the dial.
- Negative numbers get thousands separators with the `{t}` formatter.
- `x_axis_name_rotate` is documented as radians, which is what it always was.

### Changed

- `from_json` validates its input: an unknown key (with a "did you mean"
  hint), a value of the wrong type, an unknown enum value (matched
  case-insensitively), a non-positive `width`/`height`, a split count above
  1000 or an index above 1 000 000 is an `Error::Params` instead of being
  silently ignored. `type` and `quality` (the web editor's envelope keys) are
  still accepted. A multi chart child is validated by its own chart type.
- `axis_min` / `axis_max` are exact bounds, as documented: data beyond them is
  clipped rather than extending the axis.
- The horizontal bar chart honours `x_axis_height` (the theme default, 30)
  instead of a hard-coded 25.
- Mark line labels are drawn inside the plot area, above the line's right
  end, so they are no longer clipped without a right y axis.
- Vertical axes skip labels when there are more than fit, like the x axis.
- An empty series in JSON is kept (with its legend entry) instead of being
  dropped.
- Bars, line points (with tooltips), pie slices, scatter symbols, funnel
  stages, treemap cells, candles, waterfall bars, box plots, graph nodes and
  theme river streams carry `data-*` attributes (`data-series`,
  `data-category`, `data-value`, …) like the heatmap and calendar cells.

### Added

- `series_label_formatter` templates (`{a}` series, `{b}` category, `{c}`
  value, `{t}` thousands) work on bar, line, horizontal bar, radar and heatmap
  charts; precision-only formatters keep their meaning.
- `legend_position`: `top` (default), `bottom`, `left` or `right`.
- Per-series `smooth`, `fill` and `symbol` overrides on `Series`.
- `MarkLineCategory::Value(f32)` for a fixed mark line (JSON
  `{"category": "value", "value": 42}`); mark lines are drawn on bar and
  candlestick charts too.
- `empty_text`: shown in the plot area when every series is empty.
- `x_axis_label_overflow`: `thin` (default), `rotate` (45°, with the axis
  growing to fit) or `ellipsis` for x axis labels that do not fit.
- `mark_areas` on a series: shaded bands between two statistics or values
  (`{"from": 10, "to": "max"}`), on line, bar, candlestick and horizontal
  bar charts; horizontal bar charts also draw `mark_lines`.
- Tooltips and `data-*` attributes on sunburst arcs, tree nodes and sankey
  nodes and links.
- Titles wider than the canvas are cut with an ellipsis.
- Horizontal bar charts: stacking, per-bar `colors`, `x_axis_hidden` /
  `y_axis_hidden`, animation, and the value axis honours `axis_min`,
  `axis_max`, `axis_formatter` and `axis_scale`. Scatter (both axes) and box
  plot charts honour `axis_scale` and `{t}` formatters too.
- Tooltips (`tooltip_show`) on funnel, treemap, candlestick, heatmap,
  waterfall, box plot and graph charts.
- `MultiChart` composes every chart type (`ChildChart` gained the missing
  variants and is `#[non_exhaustive]`; it and `MultiChart` are `Clone` +
  `Debug`).
- Graph charts: `categories` (shown as the legend) and link widths scaled by
  `value`. Parallel charts: `y_axis_configs` per dimension (`axis_min`,
  `axis_max`, `axis_formatter`). Theme river: `series_smooth` draws smooth
  bands (`SmoothBand` component). Treemap: nested `series_data` and
  `series_label_formatter`. Radar: `split_number` rings, and the web fits the
  shorter canvas side. Pie: `min_show_label_angle`; zero slices get no label.
- Tiny value ranges (below 0.1 per tick) get proper ticks.
- `Color::parse` for `#rgb`, `#rgba`, `#rrggbb`, `#rrggbbaa`, `rgb()`,
  `rgba()` and `transparent`. JSON colors go through it: an unparsable color
  keeps the theme default instead of turning transparent, and a bad entry in
  a color list keeps its position instead of shifting the later ones.
- Snapshot tests can be regenerated with `UPDATE_SNAPSHOTS=1 cargo test
  --features image-encoder`; a plain test run no longer rewrites the preview
  images under `asset/image/`.
- `Margin`, the new name of `Box` (which stays as a type alias; it shadows
  `std::boxed::Box` under `use charts_rs::*`). It is `Copy` + `PartialEq` and
  deserializes from a number or an object.
- `Color` serializes as a hex string and deserializes from any string
  `Color::parse` accepts (or the previous `{r, g, b, a}` object), so
  `serde_json::from_str::<ChartBase>` takes the same colors as `from_json`.
  `Fill` derives serde; every chart type, `ChartBase` and `YAxisConfig` derive
  `PartialEq`.
- An unknown `theme` in JSON is an error naming the registered themes.
- Integration tests and preview images for the sankey, tree, graph, parallel
  and theme river charts; the README's Rust snippets are compiled as doctests
  (with the `png` feature).
- Benchmarks for large line charts (1k / 10k points), table text wrapping,
  sankey layout, pie paths and JSON parsing.

### Performance

- Shapes stream their attributes straight into the output (no per-attribute
  strings or vectors), and the paint shared by a line's symbols, a grid's
  lines and an axis' ticks sits on their `<g>` instead of on every element;
  a circle without a stroke no longer carries a `stroke-width`. The SVG is
  smaller and the snapshots changed accordingly.
- Font bytes are shared between the fontdue registry and the raster `fontdb`
  (`Arc`), so adding fonts no longer deep-copies every registered font, and
  the embedded default font is referenced in place.
- The SVG document is written once: the `<svg>` header is streamed ahead of
  the components instead of the finished body being copied into it, and the
  smooth-curve path (the largest piece of a line chart) is written straight
  into the output without per-point strings.
- Axis labels are measured through the per-thread cache; bar labels and
  tooltips are only formatted when they are shown.

### Documentation

- README: version `1`, MSRV section, the lightweight per-format raster
  features, sections for the five node charts, absolute image URLs (so they
  show on crates.io) and compilable examples.
- `cargo doc` marks the raster export functions with the feature that enables
  them (docs.rs); the internal drawing helpers re-exported for compatibility
  are hidden from the docs.
- CI also checks the `png`-only build, `cargo doc` with warnings denied, and
  the README doctests.

## 1.0.0

First stable release. The sections below double as a migration guide from
0.7.x; for anything not listed here the API is unchanged. All SVG output is
byte-identical to 0.7.1 (verified against the full snapshot-test suite).

### Breaking changes

#### Chart structs embed `ChartBase`

The `charts-rs-derive` proc-macro crate is gone. Every chart now stores the
~50 shared fields (title, legend, axes, series styling, …) in a public
`base: ChartBase` field, exposed through `Deref`/`DerefMut`:

- Field access and the builder pattern are unchanged — `chart.title_text = …`
  still works, as do all `XChart::new(…)` constructors and `from_json`.
- Only struct-literal construction changes; the shared fields now live in
  `base`:

  ```rust
  // 0.7
  let chart = BarChart { series_list, ..Default::default() };
  // 1.0
  let chart = BarChart {
      base: ChartBase { series_list, ..Default::default() },
      ..Default::default()
  };
  ```

#### Error type

- The `CanvasError`, `CanvasResult`, `FontError` and `EncoderError` aliases
  are removed — use `charts_rs::Error` / `charts_rs::Result`, which every
  module already produced.
- The unused `Error::Io` variant is removed.
- `impl From<&str> for Error` is removed — construct
  `Error::Params { message }` explicitly.

#### Fonts

- `get_font` returns `Arc<Font>` instead of `&Font`. Call sites that pass the
  font on by reference keep working through deref coercion; only code that
  stored the `&Font` long-term needs to hold the `Arc` instead.
- `get_or_try_init_fonts` is removed — use `add_fonts(&[data])`, which can be
  called at any time (not just before the first render). Fonts registered
  later now also reach the raster (PNG/JPEG/WebP/AVIF) pipeline, which the
  old init-once design silently ignored.

### Added

- `Chart` trait, implemented by all 22 chart types (`svg()` +
  `from_json()`), so mixed charts can be handled as `Vec<Box<dyn Chart>>`.
- `ChartBase` is a public type and can be filled directly.
- `add_fonts` — register TTF/OTF fonts at any time; replaces
  `get_or_try_init_fonts`.
- Per-format raster features: `png`, `jpeg`, `webp`, `avif`. Each
  `svg_to_*` function is gated on its own format feature; `image-encoder`
  remains as the umbrella enabling all four, so existing users are
  unaffected. A png-only build compiles ~55 fewer crates than the full
  umbrella (AVIF's rav1e encoder is the heavyweight).
- The common options `x_axis_hidden`, `y_axis_hidden`, `animation` and
  `tooltip_show` are now uniformly available on every chart via `ChartBase`.

### Performance

- SVG rendering streams into a single output buffer instead of concatenating
  per-component strings.
- Identical gradient definitions within one document are emitted once and
  shared by all shapes referencing them.
- Text measurement reuses a per-thread layout and memoizes results;
  `text_wrap_fit` does one incremental layout pass instead of re-laying out
  every prefix.
- Dependencies trimmed (`substring`, `regex`, `snafu`, `html-escape`,
  `ahash`, `syn`, `quote` all dropped): a default SVG-only build compiles 24
  crates instead of 40.

### Compatibility policy for 1.x

- Missing data points are `None` in `Series::data` (`Vec<Option<f32>>`) and
  are skipped instead of drawn as zero. Flat `Vec<f32>` input and JSON keep
  accepting the legacy `NIL_VALUE` sentinel (= `f32::MIN`), which maps to a
  missing point; JSON `null` does too.
- Chart structs keep their public fields. New optional fields may be added in
  minor releases — construct charts via `new(…)`, `from_json`, or functional
  update syntax (`..Default::default()`) rather than exhaustive struct
  literals, which are not covered by the compatibility guarantee.
- `Error` stays `#[non_exhaustive]`; keep a wildcard arm when matching.
- MSRV is 1.88 and may rise in a minor release, noted here.

## 0.7.1 and earlier

See the [GitHub releases](https://github.com/vicanso/charts-rs/releases) and
git history.
