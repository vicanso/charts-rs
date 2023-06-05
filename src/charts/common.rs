use super::Color;

#[derive(Clone, PartialEq, Debug, Default)]
pub enum Position {
    #[default]
    Left,
    Top,
    Right,
    Bottom,
}

#[derive(Clone, PartialEq, Debug, Default)]
pub enum Align {
    Left,
    #[default]
    Center,
    Right,
}

#[derive(Clone, PartialEq, Debug)]
pub enum Symbol {
    Circle(f32, Option<Color>),
}

#[derive(Clone, PartialEq, Debug)]
pub enum SeriesCategory {
    Line,
    Bar,
}

#[derive(Clone, PartialEq, Debug, Default)]
pub struct Series {
    pub name: String,
    pub data: Vec<f32>,
    // 指定index
    pub index: Option<usize>,
    // 其对应的y轴
    pub y_axis_index: usize,
    pub category: Option<SeriesCategory>,
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

#[derive(Clone, PartialEq, Debug, Default)]
pub struct YAxisConfig {
    pub axis_font_size: f32,
    pub axis_font_color: Color,
    pub axis_stroke_color: Color,
    pub axis_width: Option<f32>,
    pub axis_split_number: usize,
    pub axis_name_gap: f32,
    pub axis_formatter: Option<String>,
}
