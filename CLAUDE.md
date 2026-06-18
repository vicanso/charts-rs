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

**charts-rs** is a Rust library that generates SVG charts (optionally rendered to PNG/JPEG/WebP/AVIF). It supports 18 chart types and 10 built-in themes, with an API inspired by Apache ECharts.

### Chart Types
`BarChart`, `HorizontalBarChart`, `LineChart`, `PieChart`, `RadarChart`, `ScatterChart`, `CandlestickChart`, `TableChart`, `HeatmapChart`, `FunnelChart`, `WaterfallChart`, `MultiChart`, `CalendarChart`, `GaugeChart`, `TreemapChart`, `BoxPlotChart`, `SunburstChart`, `SankeyChart`

### Two Creation Paths

**Builder API:**
```rust
let mut chart = BarChart::new(series_list, x_axis_data);
chart.title_text = "My Chart".to_string();
chart.svg()?;
```

**JSON API:**
```rust
let chart = BarChart::from_json(r#"{ "width": 630, ... }"#)?;
chart.svg()?;
```

### Rendering Pipeline
```
Chart struct → fill_theme() [via #[derive(Chart)] macro] → svg() method
    → Canvas (coordinate system + SVG context)
    → Component primitives (Text, Line, Rect, Circle, etc.)
    → SVG string
    → [optional] svg_to_png() via resvg (feature: image-encoder)
```

### Key Modules

| Module | Purpose |
|--------|---------|
| `src/charts/component.rs` | SVG primitive components: Text, Line, Rect, Circle, Polygon, Polyline, Grid, Legend, Axis, Pie |
| `src/charts/canvas.rs` | Canvas abstraction — coordinate transformations, rendering context, SVG tag building |
| `src/charts/theme.rs` | Theme system; global registry via `Lazy<ArcSwap<AHashMap>>`; `get_theme()`, `add_theme()` |
| `src/charts/common.rs` | Shared types: `Series`, `YAxisConfig`, `MarkPoint`, `MarkLine`, `Position`, `Align`, `Symbol` |
| `src/charts/params.rs` | JSON parsing utilities (`get_*_from_value()` functions) used in `from_json()` implementations |
| `src/charts/color.rs` | `Color` type with hex parsing (`"#345"`, `"#ffcc00"`) and opacity |
| `src/charts/font.rs` | Font management via `fontdue`; custom TTF/OTF loading; default: embedded `Roboto.ttf` |
| `src/charts/encoder.rs` | Raster image encoding via `resvg` + `image` (gated on `image-encoder` feature) |
| `charts-rs-derive/` | Proc-macro crate providing `#[derive(Chart)]` — auto-generates `fill_theme()` for chart structs |

### Important Conventions

- **`NIL_VALUE`**: `f32::MIN` — represents null/missing data points in series
- **Format labels**: `{c}` (value), `{a}` (series name), `{b}` (category), `{d}` (percentage), `{t}` (thousands)
- **Coordinate system**: top-left origin (0,0), x right, y down
- **Colors**: hex strings `"#345"` or `"#ffcc00"`; parsed in `color.rs`
- **Box margins**: `left, top, right, bottom` (CSS-like padding/margin fields)
- **`image-encoder` feature**: optional; enables PNG/JPEG/WebP/AVIF export

### Tests

Integration tests are in `tests/` (one file per chart type). Each test creates a chart from JSON, calls `.svg()`, and compares against a snapshot string. Uses `pretty_assertions` for readable diffs.
