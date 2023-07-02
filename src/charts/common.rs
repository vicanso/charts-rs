use super::Color;
use crate::{
    get_bool_from_value, get_f32_slice_from_value, get_string_from_value, get_usize_from_value,
    Point,
};
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

fn get_series_category_from_value(value: &serde_json::Value, key: &str) -> Option<SeriesCategory> {
    if let Some(value) = value.get(key) {
        if let Some(value) = value.as_str() {
            return match value.to_lowercase().as_str() {
                "line" => Some(SeriesCategory::Line),
                "bar" => Some(SeriesCategory::Bar),
                _ => None,
            };
        }
    }
    None
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug, Default)]
pub struct Series {
    pub name: String,
    pub data: Vec<f32>,
    // 指定index
    pub index: Option<usize>,
    // 其对应的y轴
    pub y_axis_index: usize,
    // 是否展示label
    pub label_show: bool,
    pub category: Option<SeriesCategory>,
}

fn get_series_from_value(value: &serde_json::Value) -> Option<Series> {
    let name = get_string_from_value(value, "name").unwrap_or_default();
    if name.is_empty() {
        return None;
    }
    let data = get_f32_slice_from_value(value, "data").unwrap_or_default();
    if data.is_empty() {
        return None;
    }
    Some(Series {
        name,
        data,
        index: get_usize_from_value(value, "index"),
        y_axis_index: get_usize_from_value(value, "y_axis_index").unwrap_or_default(),
        label_show: get_bool_from_value(value, "label_show").unwrap_or_default(),
        category: get_series_category_from_value(value, "category"),
    })
}

pub(crate) fn get_series_list_from_value(value: &serde_json::Value) -> Option<Vec<Series>> {
    if let Some(data) = value.get("series_list") {
        if let Some(arr) = data.as_array() {
            let mut series_list = vec![];
            for item in arr.iter() {
                if let Some(series) = get_series_from_value(item) {
                    series_list.push(series);
                }
            }
            return Some(series_list);
        }
    }
    None
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

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug, Default)]
pub struct YAxisConfig {
    pub axis_font_size: f32,
    pub axis_font_color: Color,
    pub axis_stroke_color: Color,
    pub axis_width: Option<f32>,
    pub axis_split_number: usize,
    pub axis_name_gap: f32,
    pub axis_formatter: Option<String>,
}
