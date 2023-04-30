use super::Color;
use once_cell::sync::Lazy;

pub static DEFAULT_WIDTH: f64 = 600.0;
pub static DEFAULT_HEIGHT: f64 = 400.0;
pub static DEFAULT_X_AXIS_HEIGHT: f64 = 30.0;
pub static DEFAULT_FONT_SIZE: f64 = 14.0;
pub static DEFAULT_AXIS_NAME_GAP: f64 = 5.0;

#[derive(Clone, PartialEq, Debug, Default)]
pub struct Series {
    pub name: String,
    pub data: Vec<f64>,
}

#[derive(Clone, PartialEq, Debug, Default)]

pub struct Theme {
    pub x_axis_font_size: f64,
    pub x_axis_stroke_color: Color,
    pub x_axis_font_color: Color,
    pub x_axis_name_gap: f64,
}

static LIGHT_THEME: Lazy<Theme> = Lazy::new(|| {
    let black = (110, 112, 121).into();
    Theme {
        x_axis_font_size: DEFAULT_FONT_SIZE,
        x_axis_stroke_color: black,
        x_axis_font_color: black,
        x_axis_name_gap: DEFAULT_AXIS_NAME_GAP,
    }
});

pub fn get_theme(theme: String) -> Theme {
    match &theme as &str {
        "black" => LIGHT_THEME.clone(),
        _ => LIGHT_THEME.clone(),
    }
}
