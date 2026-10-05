# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Commands

```bash
# Build
cargo build

# Run tests (image-encoder feature required for most tests)
cargo test --features "image-encoder"

# Run a single test
cargo test --features "image-encoder" <test_name>

# Lint
cargo clippy --features=image-encoder --all-targets --all -- --deny=warnings

# Format
cargo fmt
```

## Architecture

**charts-rs** is a Rust library that generates SVG charts (optionally rendered to PNG/JPEG/WebP/AVIF). It supports 27 chart types and 10 built-in themes, with an API inspired by Apache ECharts.

### Chart Types
`BarChart`, `HorizontalBarChart`, `LineChart`, `PieChart`, `RadarChart`, `ScatterChart`, `CandlestickChart`, `TableChart`, `HeatmapChart`, `FunnelChart`, `WaterfallChart`, `MultiChart`, `CalendarChart`, `GaugeChart`, `TreemapChart`, `BoxPlotChart`, `SunburstChart`, `SankeyChart`, `TreeChart`, `GraphChart`, `ParallelChart`, `ThemeRiverChart`, `HistogramChart`, `PolarBarChart`, `ChordChart`, `GanttChart`, `MapChart`

### Two Creation Paths

**Builder API:**
```rust
let mut chart = BarChart::new(series_list, x_axis_data);
chart.title.text = "My Chart".to_string();
chart.svg()?;
```

**JSON API:**
```rust
let chart = BarChart::from_json(r#"{ "width": 630, ... }"#)?;
chart.svg()?;
```

### Rendering Pipeline
```
Chart struct (embeds ChartBase via Deref) → fill_theme() → svg() method
    → Canvas (coordinate system + SVG context)
    → Component primitives (Text, Line, Rect, Circle, etc.)
    → SVG string
    → [optional] svg_to_png() via resvg (feature: image-encoder)
```

### Key Modules

| Module | Purpose |
|--------|---------|
| `src/charts/component.rs` | SVG primitive components: Text, Line, Rect, Circle, Polygon, Polyline, Grid, Legend, Axis, Pie, Sector, Ribbon, Shape |
| `src/charts/canvas.rs` | Canvas abstraction — coordinate transformations, rendering context, SVG tag building |
| `src/charts/theme.rs` | Theme system: a `Theme` is the canvas defaults plus the same option groups a chart has; global registry via `LazyLock<ArcSwap<HashMap>>`; `get_theme()`, `add_theme()` |
| `src/charts/common.rs` | Shared types: `Series`, `MarkPoint`, `MarkLine`, `Position`, `Align`, `Symbol`, and the option groups `FontConfig`, `TitleConfig`, `LegendConfig`, `XAxisConfig`, `YAxisConfig`, `GridConfig`, `SeriesConfig`, `TooltipConfig` |
| `src/charts/params.rs` | JSON parsing utilities (`get_*_from_value()` functions) used in `from_json()` implementations |
| `src/charts/color.rs` | `Color` type with hex parsing (`"#345"`, `"#ffcc00"`) and opacity |
| `src/charts/font.rs` | Font registry and text measurement via `ttf-parser` (glyphs are read lazily from the font bytes); custom TTF/OTF loading; default: embedded `Roboto.ttf` |
| `src/charts/encoder.rs` | Raster image encoding via `resvg` + `image` (gated on `image-encoder` feature) |
| `src/charts/base.rs` | `ChartBase` — the shared chart options, grouped by element (`title`, `sub_title`, `legend`, `x_axis`, `grid`, `series`, `tooltip`), plus `fill_theme()`/`fill_option()`/`render_header()`/`render_bar()`/`render_line()` etc.; every chart embeds it and exposes it via `Deref`/`DerefMut` (`chart.title.text` works directly) |

### Important Conventions

- **`NIL_VALUE`**: `f32::MIN` — represents null/missing data points in series
- **Format labels**: `{c}` (value), `{a}` (series name), `{b}` (category), `{d}` (percentage), `{t}` (thousands)
- **Coordinate system**: top-left origin (0,0), x right, y down
- **Colors**: hex strings `"#345"` or `"#ffcc00"`; parsed in `color.rs`
- **Box margins**: `left, top, right, bottom` (CSS-like padding/margin fields)
- **`image-encoder` feature**: optional; enables PNG/JPEG/WebP/AVIF export
- **Options in Rust and JSON**: a JSON key is the Rust field path joined with `_` (`title_font_size` ↔ `title.font.size`); the exceptions are `x_boundary_gap` (`x_axis.boundary_gap`), `x_axis_type` (`x_axis.kind`), the `axis_` prefix of y axis keys, which the Rust fields leave out, and the `series` object of a heatmap (`heatmap_series`). JSON keys never change with the Rust structs

### Tests

Integration tests are in `tests/` (one file per chart type). Each test creates a chart from JSON, calls `.svg()`, and compares against a snapshot string. Uses `pretty_assertions` for readable diffs.
