use super::color::*;
use super::common::*;
use super::component::*;
use super::font::DEFAULT_FONT_FAMILY;
use super::util::*;
use super::Canvas;

#[derive(Clone, Debug, Default)]
pub struct LineChart {
    pub width: f64,
    pub height: f64,
    pub margin: Box,
    pub series_list: Vec<Series>,
    pub font_family: String,
    pub x_axis_data: Vec<String>,
    pub x_axis_height: f64,
    pub x_axis_stroke_color: Color,
    pub x_axis_font_size: f64,
    pub x_axis_font_color: Color,
    pub x_axis_name_gap: f64,
    pub x_axis_name_rotate: f64,
}

impl LineChart {
    pub fn new(series_list: Vec<Series>, x_axis_data: Vec<String>) -> LineChart {
        let mut l = LineChart {
            width: DEFAULT_WIDTH,
            height: DEFAULT_HEIGHT,
            margin: (5.0).into(),
            font_family: DEFAULT_FONT_FAMILY.to_string(),
            series_list,
            x_axis_data,
            x_axis_height: DEFAULT_X_AXIS_HEIGHT,
            ..Default::default()
        };
        l.fill_theme("".to_string());
        l
    }
    pub fn fill_theme(&mut self, theme: String) {
        let t = get_theme(theme);
        self.x_axis_font_size = t.x_axis_font_size;
        self.x_axis_font_color = t.x_axis_font_color;
        self.x_axis_stroke_color = t.x_axis_stroke_color;
        self.x_axis_name_gap = t.x_axis_name_gap;
    }
    pub fn svg(&self) {
        let mut c = Canvas::new(self.width, self.height);
        c.margin = self.margin.clone();
        c.child(Box {
            top: self.height - self.x_axis_height,
            ..Default::default()
        })
        .axis(Axis {
            height: self.x_axis_height,
            width: c.width(),
            split_number: self.x_axis_data.len(),
            font_family: self.font_family.clone(),
            data: self.x_axis_data.clone(),
            font_color: Some(self.x_axis_font_color),
            stroke_color: Some(self.x_axis_stroke_color),
            font_size: self.x_axis_font_size,
            name_gap: self.x_axis_name_gap,
            name_rotate: self.x_axis_name_rotate,
            ..Default::default()
        });

        println!("{}", c.svg().unwrap())
    }
}
