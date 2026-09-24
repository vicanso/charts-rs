# Changelog

## Unreleased

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
