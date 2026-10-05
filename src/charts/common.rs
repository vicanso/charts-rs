// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use super::component::LegendCategory;
use super::{Box, Color, NIL_VALUE};
use crate::Point;
use serde::{Deserialize, Serialize};

/// The value scale of a y axis.
#[derive(Serialize, Deserialize, Clone, PartialEq, Debug, Default)]
pub enum AxisScale {
    #[default]
    /// Linear scale.
    Linear,
    /// Logarithmic scale; the field is the base (commonly 10.0).
    Log(f32),
}

/// A placement relative to a chart element.
#[derive(Clone, PartialEq, Debug, Default, Serialize, Deserialize)]
pub enum Position {
    #[default]
    /// Left side.
    Left,
    /// Top side.
    Top,
    /// Right side.
    Right,
    /// Bottom side.
    Bottom,
    /// Inside the element.
    Inside,
}

/// Horizontal alignment.
#[derive(Serialize, Deserialize, Clone, PartialEq, Debug, Default)]
pub enum Align {
    /// Left aligned.
    Left,
    #[default]
    /// Centered.
    Center,
    /// Right aligned.
    Right,
}

/// The marker drawn on data points.
#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub enum Symbol {
    /// No marker.
    None,
    /// Circle: (radius, optional fill color override)
    Circle(f32, Option<Color>),
    /// Square: (half-side, optional fill color override)
    Rect(f32, Option<Color>),
    /// Equilateral triangle pointing up: (circumradius, optional fill color override)
    Triangle(f32, Option<Color>),
    /// Diamond (rotated square): (half-diagonal, optional fill color override)
    Diamond(f32, Option<Color>),
}

/// How a series is drawn when it differs from the chart's default,
/// e.g. a line series inside a bar chart.
#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub enum SeriesCategory {
    /// Drawn as a line series.
    Line,
    /// Drawn as a bar series.
    Bar,
}

/// Where a stepped line changes from the value of one point to the next.
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
pub enum LineStep {
    /// At the point itself: the line turns to the next value right away,
    /// and then runs level to the next point.
    Start,
    /// Half way between the two points.
    Middle,
    /// At the next point: the line runs level until there.
    End,
}

/// The statistic (or fixed value) a mark line is drawn at.
#[derive(Serialize, Deserialize, Clone, PartialEq, Debug, Default)]
#[non_exhaustive]
pub enum MarkLineCategory {
    #[default]
    /// The series average.
    Average,
    /// The series minimum.
    Min,
    /// The series maximum.
    Max,
    /// A fixed value, e.g. a target or threshold.
    Value(f32),
}

/// A shaded band across the plot between two series statistics or fixed
/// values (e.g. from `Value(10.0)` to `Value(20.0)`, or `Min` to `Max`).
#[derive(Serialize, Deserialize, Clone, PartialEq, Debug, Default)]
pub struct MarkArea {
    /// One edge of the band.
    pub from: MarkLineCategory,
    /// The other edge of the band.
    pub to: MarkLineCategory,
    /// Color of the band; `None` is the color of the series.
    #[serde(default)]
    pub color: Option<Color>,
    /// How opaque the band is, from 0 to 1; `None` is 0.16.
    #[serde(default)]
    pub opacity: Option<f32>,
}

impl MarkArea {
    /// The fill of the band, on a series of the given color.
    pub(crate) fn fill(&self, series: Color) -> Color {
        self.color
            .unwrap_or(series)
            .with_alpha(self.opacity.map_or(40, opacity_alpha))
    }
}

/// The alpha of an opacity from 0 to 1.
pub(crate) fn opacity_alpha(opacity: f32) -> u8 {
    (opacity.clamp(0.0, 1.0) * 255.0).round() as u8
}

/// What the x axis of a line or bar chart measures.
///
/// A chart has a category axis until x values are given (`x_axis.values`,
/// or `x_values` on a series); then the axis is continuous and this says
/// how its values read.
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Debug, Default)]
pub enum AxisType {
    /// Evenly spaced categories (`x_axis.data`); with x values, plain
    /// numbers.
    #[default]
    Category,
    /// Plain numbers.
    Value,
    /// Timestamps in seconds since the unix epoch.
    Time,
}

/// What to do with x axis labels that do not fit side by side.
#[derive(Serialize, Deserialize, Clone, PartialEq, Debug, Default)]
pub enum AxisLabelOverflow {
    /// Skip every n-th label so the remaining ones fit (the default).
    #[default]
    Thin,
    /// Rotate the labels by 45° (when no rotation is set), then thin out
    /// what still overlaps.
    Rotate,
    /// Cut every label to its slot with an ellipsis.
    Ellipsis,
}

/// The statistic a mark point highlights.
#[derive(Serialize, Deserialize, Clone, PartialEq, Debug, Default)]
pub enum MarkPointCategory {
    #[default]
    /// The series minimum.
    Min,
    /// The series maximum.
    Max,
}

/// A horizontal reference line at a series statistic (average, min or max).
#[derive(Serialize, Deserialize, Clone, PartialEq, Debug, Default)]
pub struct MarkLine {
    /// The statistic the line is drawn at.
    pub category: MarkLineCategory,
    /// Color of the line, its dot and its arrow; `None` is the color of
    /// the series.
    #[serde(default)]
    pub color: Option<Color>,
    /// Stroke width of the line; `None` is 1.
    #[serde(default)]
    pub stroke_width: Option<f32>,
    /// Dashes of the line, as `stroke-dasharray` takes them; `None` is
    /// `"4,2"`, and an empty string a solid line.
    #[serde(default)]
    pub stroke_dash_array: Option<String>,
}

impl MarkLine {
    /// The dashes of the line; `None` for a solid one.
    pub(crate) fn dash(&self) -> Option<String> {
        match &self.stroke_dash_array {
            None => Some("4,2".to_string()),
            Some(dash) if dash.is_empty() => None,
            Some(dash) => Some(dash.clone()),
        }
    }
}

/// A marker highlighting a series statistic (min or max) on its data point.
#[derive(Serialize, Deserialize, Clone, PartialEq, Debug, Default)]
pub struct MarkPoint {
    /// The statistic the marker highlights.
    pub category: MarkPointCategory,
}

/// A band around a line series: a lower and an upper bound for each data
/// point, with the area between them filled — a confidence interval, a
/// forecast range, a daily minimum and maximum.
///
/// Bound `i` belongs to data point `i` of the series. A series may carry a
/// band without any `data`, which draws the band alone.
#[derive(Serialize, Deserialize, Clone, PartialEq, Debug, Default)]
pub struct SeriesBand {
    /// Lower bound of each point; `None` leaves a gap in the band.
    pub lower: Vec<Option<f32>>,
    /// Upper bound of each point; `None` leaves a gap in the band.
    pub upper: Vec<Option<f32>>,
    /// Stroke width of the error bars the bounds are drawn as
    /// (`Series::error_bar`); `None` is 1.5. A band has no stroke.
    #[serde(default)]
    pub stroke_width: Option<f32>,
}

impl SeriesBand {
    /// The stroke width of the error bars the bounds are drawn as.
    pub(crate) fn error_bar_width(band: Option<&SeriesBand>) -> f32 {
        band.and_then(|b| b.stroke_width).unwrap_or(1.5)
    }
    /// Creates a band from its bounds. The legacy `NIL_VALUE` sentinel marks
    /// a missing bound, as in [`Series::new`].
    pub fn new(lower: Vec<f32>, upper: Vec<f32>) -> Self {
        let nullable = |values: Vec<f32>| -> Vec<Option<f32>> {
            values
                .into_iter()
                .map(|v| if v == NIL_VALUE { None } else { Some(v) })
                .collect()
        };
        SeriesBand {
            lower: nullable(lower),
            upper: nullable(upper),
            ..Default::default()
        }
    }
    /// Number of points the band spans.
    pub(crate) fn len(&self) -> usize {
        self.lower.len().max(self.upper.len())
    }
    /// The bounds of point `i`, lower first, when both are present.
    pub(crate) fn bounds(&self, i: usize) -> Option<(f32, f32)> {
        let bound =
            |values: &[Option<f32>]| values.get(i).copied().flatten().filter(|v| v.is_finite());
        let (a, b) = (bound(&self.lower)?, bound(&self.upper)?);
        Some((a.min(b), a.max(b)))
    }
    /// Every bound that is present, for sizing the axis.
    pub(crate) fn values(&self) -> impl Iterator<Item = f32> + '_ {
        self.lower
            .iter()
            .chain(self.upper.iter())
            .filter_map(|v| v.filter(|v| v.is_finite()))
    }
}

/// One data series: a name plus its values, with per-series display options.
#[derive(Serialize, Deserialize, Clone, PartialEq, Debug, Default)]
pub struct Series {
    /// Name of the series, shown in the legend.
    pub name: String,
    /// Data list of the series; `None` marks a missing / null data point.
    pub data: Vec<Option<f32>>,
    /// X-axis index the first data point is placed at.
    pub start_index: usize,
    /// Explicit palette index; `None` follows the series position.
    pub index: Option<usize>,
    /// Which y axis (0 or 1) the series is bound to.
    pub y_axis_index: usize,
    /// Whether to display value labels on the data points.
    pub label_show: bool,
    /// Mark lines (average/min/max) drawn across the chart.
    pub mark_lines: Vec<MarkLine>,
    /// Mark points (min/max) drawn on the data points.
    pub mark_points: Vec<MarkPoint>,
    /// Shaded bands between two statistics / values.
    #[serde(default)]
    pub mark_areas: Vec<MarkArea>,
    /// Per-data-point color overrides (bar charts).
    pub colors: Option<Vec<Option<Color>>>,
    /// Overrides how the series is drawn, e.g. a line inside a bar chart.
    pub category: Option<SeriesCategory>,
    /// SVG stroke dash array for line series.
    pub stroke_dash_array: Option<String>,
    /// Stack group name; series with the same name and `y_axis_index` are stacked.
    pub stack: Option<String>,
    /// Overrides the chart-wide `series.smooth` for this series.
    #[serde(default)]
    pub smooth: Option<bool>,
    /// Overrides the chart-wide `series.fill` for this series.
    #[serde(default)]
    pub fill: Option<bool>,
    /// Overrides the chart-wide `series.symbol` for this series
    /// (`Some(Symbol::None)` draws no marker).
    #[serde(default)]
    pub symbol: Option<Symbol>,
    /// The x value of each data point, on a continuous x axis, for a series
    /// that is not sampled at the chart's `x_axis.values`. Timestamps are
    /// unix seconds. `start_index` does not apply to a series with its own
    /// x values.
    #[serde(default)]
    pub x_values: Option<Vec<f64>>,
    /// A filled band around the line (line series only): a lower and an
    /// upper bound per data point.
    #[serde(default)]
    pub band: Option<SeriesBand>,
    /// Draws the line as steps instead of joining the points directly
    /// (line series only); wins over `smooth`.
    #[serde(default)]
    pub step: Option<LineStep>,
    /// Error bars: a lower and an upper bound per data point, drawn as a
    /// line with a cap at both ends over the bar or the point (bar, line
    /// and scatter charts).
    #[serde(default)]
    pub error_bar: Option<SeriesBand>,
    /// The ring of a pie chart the series is a slice of. The series of a
    /// ring share a circle of their own, the lowest ring innermost: nested
    /// pies. Default: 0, every series on the one ring.
    #[serde(default)]
    pub ring: usize,
}

/// Animation configuration for SVG chart animations.
/// When set, bars grow from the bottom, lines draw progressively, and
/// pie / sunburst slices expand from the center while labels fade in.
/// PNG/JPEG export via resvg renders the fully-drawn static state.
#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub struct AnimationConfig {
    /// Total animation duration in milliseconds (default: 1000).
    pub duration: u32,
    /// CSS easing function: "ease", "linear", "ease-in", "ease-out", "ease-in-out" (default: "ease").
    pub easing: String,
    /// Stagger delay in milliseconds between each column (bars), series
    /// (lines), slice (pie), or ring level (sunburst) (default: 80).
    pub delay: u32,
}

impl Default for AnimationConfig {
    fn default() -> Self {
        AnimationConfig {
            duration: 1000,
            easing: "ease".to_string(),
            delay: 80,
        }
    }
}

impl AnimationConfig {
    /// Returns the easing value sanitized for safe interpolation into a CSS
    /// `<style>` block. Any value containing characters outside those valid for
    /// a CSS timing-function (ASCII letters/digits and ` .,()%+-`) falls back to
    /// `"ease"`. This prevents `<style>`/tag breakout (CSS/SVG injection) when
    /// `easing` originates from untrusted JSON.
    pub(crate) fn safe_easing(&self) -> &str {
        const DEFAULT: &str = "ease";
        let ok = !self.easing.is_empty()
            && self.easing.chars().all(|c| {
                c.is_ascii_alphanumeric()
                    || matches!(c, ' ' | '.' | ',' | '(' | ')' | '%' | '+' | '-')
            });
        if ok { &self.easing } else { DEFAULT }
    }
}

/// A rendered series label: its text and anchor point.
#[derive(Clone, PartialEq, Debug, Default)]
pub struct SeriesLabel {
    /// Anchor point of the label.
    pub point: Point,
    /// Label text.
    pub text: String,
}

impl Series {
    /// Creates a series from a flat value list. For backward compatibility the
    /// legacy `NIL_VALUE` sentinel is mapped to a missing point (`None`).
    pub fn new(name: String, data: Vec<f32>) -> Self {
        Series {
            name,
            data: data
                .into_iter()
                .map(|v| if v == NIL_VALUE { None } else { Some(v) })
                .collect(),
            index: None,
            ..Default::default()
        }
    }
    /// Creates a series from nullable values, where `None` marks a missing
    /// data point (rendered as a gap).
    pub fn new_nullable(name: String, data: Vec<Option<f32>>) -> Self {
        Series {
            name,
            data,
            index: None,
            ..Default::default()
        }
    }
    /// Effective values with the legacy `NIL_VALUE` sentinel substituted for
    /// missing points, without allocating a `Vec`. NaN/inf can only arrive
    /// through the builder API; they are missing so they never reach the axis
    /// math or the SVG.
    /// Number of slots the series spans: its data points, or the points of
    /// its band when that is longer (a band without a line).
    pub(crate) fn slot_len(&self) -> usize {
        self.data
            .len()
            .max(self.band.as_ref().map(SeriesBand::len).unwrap_or(0))
    }
    pub(crate) fn iter_values(&self) -> impl Iterator<Item = f32> + '_ {
        self.data.iter().map(|v| match v {
            Some(value) if value.is_finite() => *value,
            _ => NIL_VALUE,
        })
    }
    /// Effective values with the legacy `NIL_VALUE` sentinel substituted for
    /// missing points. Lets the renderers keep their existing sentinel-based
    /// arithmetic while the public data model uses `Option<f32>`.
    pub(crate) fn data_values(&self) -> Vec<f32> {
        self.iter_values().collect()
    }
}
impl From<(&str, Vec<f32>)> for Series {
    fn from(value: (&str, Vec<f32>)) -> Self {
        Series::new(value.0.to_string(), value.1)
    }
}
impl From<(&str, Vec<Option<f32>>)> for Series {
    fn from(value: (&str, Vec<Option<f32>>)) -> Self {
        Series::new_nullable(value.0.to_string(), value.1)
    }
}

/// How a text is written: the title, the sub-title, the legend, the labels
/// of the axes and those of the series each have one.
#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
#[serde(default)]
pub struct FontConfig {
    /// Font size.
    pub size: f32,
    /// Font color.
    pub color: Color,
    /// Font weight, e.g. `"bold"`.
    pub weight: Option<String>,
}

/// The title of a chart, and its sub-title.
#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
#[serde(default)]
pub struct TitleConfig {
    /// The text.
    pub text: String,
    /// Font of the text.
    pub font: FontConfig,
    /// Margin around the block.
    pub margin: Option<Box>,
    /// Horizontal alignment.
    pub align: Align,
    /// Height reserved for the row.
    pub height: f32,
}

/// The legend of a chart.
#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
#[serde(default)]
pub struct LegendConfig {
    /// Font of the entries.
    pub font: FontConfig,
    /// Horizontal alignment of the legend.
    pub align: Align,
    /// Margin around the legend block.
    pub margin: Option<Box>,
    /// Marker shape (normal, rect or round rect).
    pub category: LegendCategory,
    /// Shows or hides the legend; `None` follows the chart's default.
    pub show: Option<bool>,
    /// Where the legend goes: top (the default, beside the title), bottom,
    /// or stacked vertically on the left / right of the plot.
    pub position: Option<Position>,
}

/// The x axis of a chart.
#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
#[serde(default)]
pub struct XAxisConfig {
    /// Labels of the axis.
    pub data: Vec<String>,
    /// Height reserved for the axis block.
    pub height: f32,
    /// Stroke color of the axis line and ticks.
    pub stroke_color: Color,
    /// Stroke width of the axis line and ticks; `None` is 1.
    pub stroke_width: Option<f32>,
    /// Font of the labels.
    pub font: FontConfig,
    /// Gap between the axis line and its labels.
    pub name_gap: f32,
    /// Rotation of the labels, in radians (e.g. `0.785` for 45°).
    pub name_rotate: f32,
    /// Margin around the axis block.
    pub margin: Option<Box>,
    /// Whether a gap is left on both ends of the axis (bar-style) or the
    /// first/last points sit on the edges (line-style).
    pub boundary_gap: Option<bool>,
    /// What to do with labels that do not fit side by side: thin them out
    /// (default), rotate them, or cut them with an ellipsis.
    pub label_overflow: AxisLabelOverflow,
    /// How continuous x values read: plain numbers, or timestamps (`Time`).
    /// The axis is continuous whenever x values are given; see `values`.
    pub kind: AxisType,
    /// The x value of each data point, shared by every series (a series may
    /// carry its own in `Series::x_values`). Setting them turns the x axis
    /// of line and bar charts from evenly spaced categories into a
    /// continuous scale, so unevenly sampled data keeps its real spacing.
    /// Timestamps are seconds since the unix epoch; JSON also takes date
    /// strings such as `"2024-01-05"` or `"2024-01-05 08:30"`.
    pub values: Vec<f64>,
    /// Fixed start of a continuous axis; `None` starts at the first value.
    pub min: Option<f64>,
    /// Fixed end of a continuous axis; `None` ends at the last value.
    pub max: Option<f64>,
    /// Format of the labels: `{c}` is the label; on a time axis a
    /// `strftime`-like pattern (`%Y %y %m %d %H %M %S %b`) replaces the
    /// automatic one.
    pub formatter: Option<String>,
    /// Minutes east of UTC that a time axis is displayed in (480 for
    /// UTC+8). Timestamps are shown in UTC by default.
    pub time_offset: i32,
    /// Title of the axis, written below its labels.
    pub title: String,
    /// Hides the axis entirely (charts without an x axis ignore this).
    pub hidden: bool,
}

/// Configuration of one y axis; charts hold one entry per axis in
/// `y_axis_configs` (up to two).
#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
#[serde(default)]
pub struct YAxisConfig {
    /// Font of the labels.
    pub font: FontConfig,
    /// Stroke color of the axis line.
    pub stroke_color: Color,
    /// Stroke width of the axis line and ticks; `None` is 1.
    pub stroke_width: Option<f32>,
    /// Width reserved for the axis block; `None` sizes it from the labels.
    pub width: Option<f32>,
    /// Number of intervals the value range splits into.
    pub split_number: usize,
    /// Gap between the axis line and its labels.
    pub name_gap: f32,
    /// Alignment of the axis labels.
    pub name_align: Option<Align>,
    /// Margin around the axis block.
    pub margin: Option<Box>,
    /// Label format, supporting `{c}` value and `{t}` thousands.
    pub formatter: Option<String>,
    /// Fixed lower bound of the value range; `None` derives it from the data.
    pub min: Option<f32>,
    /// Fixed upper bound of the value range; `None` derives it from the data.
    pub max: Option<f32>,
    /// Value scale of the axis (linear or logarithmic).
    pub scale: AxisScale,
    /// Title of the axis (e.g. `"Temperature (°C)"`), written along it.
    pub title: Option<String>,
    /// Turns the axis upside down: the smallest value at the top, the
    /// largest at the bottom (a ranking, where 1 is the best place).
    pub inverse: bool,
}

/// The grid lines of a chart.
#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
#[serde(default)]
pub struct GridConfig {
    /// Stroke color of the grid lines.
    pub stroke_color: Color,
    /// Stroke width of the grid lines.
    pub stroke_width: f32,
    /// Dashes of the grid lines, as `stroke-dasharray` takes them (e.g.
    /// `"4,2"`); `None` is solid lines.
    pub stroke_dash_array: Option<String>,
}

/// How the series of a chart are drawn, where a series does not say so
/// itself. Their data is in `series_list`.
#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
#[serde(default)]
pub struct SeriesConfig {
    /// Stroke width of series lines.
    pub stroke_width: f32,
    /// Color palette cycled through by the series.
    pub colors: Vec<Color>,
    /// Marker drawn on data points (circle, dot or none).
    pub symbol: Option<Symbol>,
    /// Draws line series as smooth curves.
    pub smooth: bool,
    /// Fills the area under line series.
    pub fill: bool,
    /// How opaque the fill of an area is, from 0 to 1; `None` is 0.39
    /// under a line and 0.2 in a radar chart.
    pub fill_opacity: Option<f32>,
    /// The data labels.
    pub label: SeriesLabelConfig,
}

/// The data labels of the series.
#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
#[serde(default)]
pub struct SeriesLabelConfig {
    /// Font of the labels.
    pub font: FontConfig,
    /// Label format, supporting `{c}` value, `{a}` series name, `{b}`
    /// category, `{d}` percentage and `{t}` thousands.
    pub formatter: String,
    /// Drops a data label that would overlap one already drawn, instead of
    /// printing them on top of each other.
    pub hide_overlap: bool,
}

/// The hover tooltips of a chart.
#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
#[serde(default)]
pub struct TooltipConfig {
    /// When `true`, data shapes get a hover tooltip (`series: value`): a
    /// CSS-revealed label that works in any browser, plus a native `<title>`
    /// for accessibility. Not available in calendar, gauge, parallel, radar
    /// and theme river charts. Default: false; output is unchanged when off.
    pub show: bool,
    /// Font of the tooltips. What is left unset is that of the data labels:
    /// a size of 0, a color of nothing, no weight.
    pub font: FontConfig,
}

/// A fill that can be either a solid color or a linear gradient.
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Debug)]
pub enum Fill {
    /// A solid color fill.
    Solid(Color),
    /// A linear gradient fill.
    LinearGradient {
        /// Color at the gradient start.
        start_color: Color,
        /// Color at the gradient end.
        end_color: Color,
        /// Angle in degrees: 0 = top→bottom, 90 = left→right, 180 = bottom→top, 270 = right→left.
        angle: f32,
    },
}

impl Default for Fill {
    fn default() -> Self {
        Fill::Solid(Color::default())
    }
}

impl From<Color> for Fill {
    fn from(c: Color) -> Self {
        Fill::Solid(c)
    }
}

impl Fill {
    /// Returns true if the fill is fully transparent (only for solid fills).
    pub fn is_transparent(&self) -> bool {
        matches!(self, Fill::Solid(c) if c.is_transparent())
    }
}

#[cfg(test)]
mod tests {
    use super::{NIL_VALUE, SeriesBand};
    use pretty_assertions::assert_eq;

    #[test]
    fn series_band_bounds() {
        let band = SeriesBand::new(vec![1.0, NIL_VALUE, 9.0, 4.0], vec![3.0, 5.0, 6.0]);
        assert_eq!(vec![Some(1.0), None, Some(9.0), Some(4.0)], band.lower);
        // The band spans the longer bound, but a point needs both.
        assert_eq!(4, band.len());
        assert_eq!(Some((1.0, 3.0)), band.bounds(0));
        assert_eq!(None, band.bounds(1));
        // Swapped bounds are put in order.
        assert_eq!(Some((6.0, 9.0)), band.bounds(2));
        assert_eq!(None, band.bounds(3));
        assert_eq!(None, band.bounds(4));
        assert_eq!(
            vec![1.0, 9.0, 4.0, 3.0, 5.0, 6.0],
            band.values().collect::<Vec<_>>()
        );

        // A bound that is not a number is no bound.
        let band = SeriesBand {
            lower: vec![Some(f32::NAN), Some(1.0)],
            upper: vec![Some(2.0), Some(f32::INFINITY)],
            ..Default::default()
        };
        assert_eq!(None, band.bounds(0));
        assert_eq!(None, band.bounds(1));
        assert_eq!(vec![1.0, 2.0], band.values().collect::<Vec<_>>());
        assert_eq!(0, SeriesBand::default().len());
    }
}
