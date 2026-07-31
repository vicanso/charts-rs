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

/// The statistic a mark line is drawn at.
#[derive(Serialize, Deserialize, Clone, PartialEq, Debug, Default)]
pub enum MarkLineCategory {
    #[default]
    /// The series average.
    Average,
    /// The series minimum.
    Min,
    /// The series maximum.
    Max,
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
    /// Per-data-point color overrides (bar charts).
    pub colors: Option<Vec<Option<Color>>>,
    /// Overrides how the series is drawn, e.g. a line inside a bar chart.
    pub category: Option<SeriesCategory>,
    /// SVG stroke dash array for line series.
    pub stroke_dash_array: Option<String>,
    /// Stack group name; series with the same name and `y_axis_index` are stacked.
    pub stack: Option<String>,
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
    /// missing points. Lets the renderers keep their existing sentinel-based
    /// arithmetic while the public data model uses `Option<f32>`.
    pub(crate) fn data_values(&self) -> Vec<f32> {
        self.data.iter().map(|v| v.unwrap_or(NIL_VALUE)).collect()
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
#[derive(Serialize, Deserialize, Clone, Debug, Default)]
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
}

/// A fill that can be either a solid color or a linear gradient.
#[derive(Clone, Copy, PartialEq, Debug)]
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
