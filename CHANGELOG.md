# Changelog

## 1.0.0 (unreleased)

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
