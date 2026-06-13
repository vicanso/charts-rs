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

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug, Default)]
pub enum AxisScale {
    #[default]
    Linear,
    /// Logarithmic scale; the field is the base (commonly 10.0).
    Log(f32),
}

#[derive(Clone, PartialEq, Debug, Default, Serialize, Deserialize)]
pub enum Position {
    #[default]
    Left,
    Top,
    Right,
    Bottom,
    Inside,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug, Default)]
pub enum Align {
    Left,
    #[default]
    Center,
    Right,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub enum Symbol {
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

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug)]
pub enum SeriesCategory {
    Line,
    Bar,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug, Default)]
pub enum MarkLineCategory {
    #[default]
    Average,
    Min,
    Max,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug, Default)]
pub enum MarkPointCategory {
    #[default]
    Min,
    Max,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug, Default)]
pub struct MarkLine {
    pub category: MarkLineCategory,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug, Default)]
pub struct MarkPoint {
    pub category: MarkPointCategory,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug, Default)]
pub struct Series {
    // name of series
    pub name: String,
    // data list of series; `None` marks a missing / null data point
    pub data: Vec<Option<f32>>,
    // start index of series
    pub start_index: usize,
    // index of series
    pub index: Option<usize>,
    // y axis index of series
    pub y_axis_index: usize,
    // whether to display the label
    pub label_show: bool,
    // mark lines
    pub mark_lines: Vec<MarkLine>,
    // mark points
    pub mark_points: Vec<MarkPoint>,
    // colors of series bar
    pub colors: Option<Vec<Option<Color>>>,
    // category of series
    pub category: Option<SeriesCategory>,
    // stroke dash array for series
    pub stroke_dash_array: Option<String>,
    // stack group name; series with the same name and y_axis_index are stacked
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

#[derive(Clone, PartialEq, Debug, Default)]
pub struct SeriesLabel {
    pub point: Point,
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

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct YAxisConfig {
    pub axis_font_size: f32,
    pub axis_font_color: Color,
    pub axis_font_weight: Option<String>,
    pub axis_stroke_color: Color,
    pub axis_width: Option<f32>,
    pub axis_split_number: usize,
    pub axis_name_gap: f32,
    pub axis_name_align: Option<Align>,
    pub axis_margin: Option<Box>,
    pub axis_formatter: Option<String>,
    pub axis_min: Option<f32>,
    pub axis_max: Option<f32>,
    pub axis_scale: AxisScale,
}

/// A fill that can be either a solid color or a linear gradient.
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Fill {
    Solid(Color),
    LinearGradient {
        start_color: Color,
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
