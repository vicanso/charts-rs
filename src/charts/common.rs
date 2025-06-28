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

use super::{Box, Color};
use crate::Point;
use serde::{Deserialize, Serialize};

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
    Circle(f32, Option<Color>),
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
    // data list of series
    pub data: Vec<f32>,
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
}

#[derive(Clone, PartialEq, Debug, Default)]
pub struct SeriesLabel {
    pub point: Point,
    pub text: String,
}

impl Series {
    pub fn new(name: String, data: Vec<f32>) -> Self {
        Series {
            name,
            data,
            index: None,
            ..Default::default()
        }
    }
}
impl From<(&str, Vec<f32>)> for Series {
    fn from(value: (&str, Vec<f32>)) -> Self {
        Series::new(value.0.to_string(), value.1)
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
}
