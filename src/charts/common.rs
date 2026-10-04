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
}

/// What the x axis of a line or bar chart measures.
///
/// A chart has a category axis until x values are given (`x_axis_values`,
/// or `x_values` on a series); then the axis is continuous and this says
/// how its values read.
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Debug, Default)]
pub enum AxisType {
    /// Evenly spaced categories (`x_axis_data`); with x values, plain
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
}

impl SeriesBand {
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
    /// Overrides the chart-wide `series_smooth` for this series.
    #[serde(default)]
    pub smooth: Option<bool>,
    /// Overrides the chart-wide `series_fill` for this series.
    #[serde(default)]
    pub fill: Option<bool>,
    /// Overrides the chart-wide `series_symbol` for this series
    /// (`Some(Symbol::None)` draws no marker).
    #[serde(default)]
    pub symbol: Option<Symbol>,
    /// The x value of each data point, on a continuous x axis, for a series
    /// that is not sampled at the chart's `x_axis_values`. Timestamps are
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

/// Configuration of one y axis; charts hold one entry per axis in
/// `y_axis_configs` (up to two).
#[derive(Serialize, Deserialize, Clone, Debug, Default, PartialEq)]
pub struct YAxisConfig {
    /// Y axis label font size.
    pub axis_font_size: f32,
    /// Y axis label font color.
    pub axis_font_color: Color,
    /// Y axis label font weight, e.g. `"bold"`.
    pub axis_font_weight: Option<String>,
    /// Stroke color of the axis line.
    pub axis_stroke_color: Color,
    /// Width reserved for the axis block; `None` sizes it from the labels.
    pub axis_width: Option<f32>,
    /// Number of intervals the value range splits into.
    pub axis_split_number: usize,
    /// Gap between the axis line and its labels.
    pub axis_name_gap: f32,
    /// Alignment of the axis labels.
    pub axis_name_align: Option<Align>,
    /// Margin around the axis block.
    pub axis_margin: Option<Box>,
    /// Label format, supporting `{c}` value and `{t}` thousands.
    pub axis_formatter: Option<String>,
    /// Fixed lower bound of the value range; `None` derives it from the data.
    pub axis_min: Option<f32>,
    /// Fixed upper bound of the value range; `None` derives it from the data.
    pub axis_max: Option<f32>,
    /// Value scale of the axis (linear or logarithmic).
    pub axis_scale: AxisScale,
    /// Title of the axis (e.g. `"Temperature (°C)"`), written along it.
    #[serde(default)]
    pub axis_title: Option<String>,
    /// Turns the axis upside down: the smallest value at the top, the
    /// largest at the bottom (a ranking, where 1 is the best place).
    #[serde(default)]
    pub axis_inverse: bool,
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
        };
        assert_eq!(None, band.bounds(0));
        assert_eq!(None, band.bounds(1));
        assert_eq!(vec![1.0, 2.0], band.values().collect::<Vec<_>>());
        assert_eq!(0, SeriesBand::default().len());
    }
}
