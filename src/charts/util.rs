use usvg::{Fill, Opacity, Paint, PathData, Rect, Size, Stroke, StrokeWidth};

use super::color::Color;

#[derive(Clone, Debug, Default)]
pub enum Position {
    Left,
    Right,
    Top,
    #[default]
    Bottom,
}

#[derive(Debug)]
pub struct Error {
    pub message: String,
}
impl From<png::EncodingError> for Error {
    fn from(err: png::EncodingError) -> Self {
        Error {
            message: err.to_string(),
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}
impl From<(f64, f64)> for Point {
    fn from(value: (f64, f64)) -> Self {
        Point {
            x: value.0,
            y: value.1,
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct Circle {
    pub cx: f64,
    pub cy: f64,
    pub r: f64,
}
impl From<(f64, f64, f64)> for Circle {
    fn from(value: (f64, f64, f64)) -> Self {
        Circle {
            cx: value.0,
            cy: value.1,
            r: value.2,
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct Margin {
    pub top: f64,
    pub right: f64,
    pub bottom: f64,
    pub left: f64,
}
impl From<f64> for Margin {
    fn from(value: f64) -> Self {
        Margin {
            top: value,
            right: value,
            bottom: value,
            left: value,
        }
    }
}
impl From<(f64, f64)> for Margin {
    fn from(value: (f64, f64)) -> Self {
        Margin {
            top: value.0,
            right: value.1,
            bottom: value.0,
            left: value.1,
        }
    }
}
impl From<(f64, f64, f64)> for Margin {
    fn from(value: (f64, f64, f64)) -> Self {
        Margin {
            top: value.0,
            right: value.1,
            bottom: value.2,
            left: value.1,
        }
    }
}
impl From<(f64, f64, f64, f64)> for Margin {
    fn from(value: (f64, f64, f64, f64)) -> Self {
        Margin {
            top: value.0,
            right: value.1,
            bottom: value.2,
            left: value.3,
        }
    }
}
impl Margin {
    pub fn add(&self, margin: Margin) -> Self {
        let mut m = self.clone();
        m.top += margin.top;
        m.right += margin.right;
        m.bottom += margin.bottom;
        m.left += margin.left;
        m
    }
}

pub fn new_size(width: f64, height: f64) -> Result<Size> {
    if let Some(value) = Size::new(width, height) {
        return Ok(value);
    }
    Err(Error {
        message: "width or height is invalid".to_string(),
    })
}

pub fn new_rect(x: f64, y: f64, width: f64, height: f64) -> Result<Rect> {
    if let Some(value) = Rect::new(x, y, width, height) {
        return Ok(value);
    }
    return Err(Error {
        message: "value of rect is invalid".to_string(),
    });
}

pub fn new_stroke(width: f64, color: Color) -> Stroke {
    let mut stroke = Stroke::default();
    if width > 0.0 && width.is_finite() {
        stroke.width = StrokeWidth::new(width).unwrap();
    }
    let (c, opacity) = color.divide();
    stroke.paint = Paint::Color(c);
    stroke.opacity = opacity;
    stroke
}

pub fn new_circle_path(cx: f64, cy: f64, r: f64) -> PathData {
    let rx = r;
    let ry = r;
    let mut p = PathData::new();
    p.push_move_to(cx + rx, cy);
    p.push_arc_to(rx, ry, 0.0, false, true, cx, cy + ry);
    p.push_arc_to(rx, ry, 0.0, false, true, cx - rx, cy);
    p.push_arc_to(rx, ry, 0.0, false, true, cx, cy - ry);
    p.push_arc_to(rx, ry, 0.0, false, true, cx + rx, cy);
    p.push_close_path();
    p
}

pub type Result<T> = std::result::Result<T, Error>;
