use super::{Box, Color};
use crate::Point;
use serde::{Deserialize, Serialize};

#[derive(Clone, PartialEq, Debug, Default)]
pub enum Position {
    #[default]
    Left,
    Top,
    Right,
    Bottom,
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
pub struct MarkLine {
    pub category: MarkLineCategory,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug, Default)]
pub struct Series {
    pub name: String,
    pub data: Vec<f32>,
    // 数据开始位置，用于数据不足且非从0开始场景
    pub start_index: usize,
    // 指定index
    pub index: Option<usize>,
    // 其对应的y轴
    pub y_axis_index: usize,
    // 是否展示label
    pub label_show: bool,
    // mark lines
    pub mark_lines: Vec<MarkLine>,
    pub category: Option<SeriesCategory>,
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
    pub axis_margin: Option<Box>,
    pub axis_formatter: Option<String>,
    pub axis_min: Option<f32>,
    pub axis_max: Option<f32>,
}
