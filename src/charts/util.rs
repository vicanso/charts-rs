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
    fn from(val: (f64, f64)) -> Self {
        Point { x: val.0, y: val.1 }
    }
}

#[derive(Clone, Debug, Default)]
pub struct Circle {
    pub cx: f64,
    pub cy: f64,
    pub r: f64,
}
impl From<(f64, f64, f64)> for Circle {
    fn from(val: (f64, f64, f64)) -> Self {
        Circle {
            cx: val.0,
            cy: val.1,
            r: val.2,
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct Box {
    pub left: f64,
    pub top: f64,
    pub right: f64,
    pub bottom: f64,
}
impl From<f64> for Box {
    fn from(val: f64) -> Self {
        Box {
            left: val,
            top: val,
            right: val,
            bottom: val,
        }
    }
}
impl From<(f64, f64)> for Box {
    fn from(val: (f64, f64)) -> Self {
        Box {
            left: val.0,
            top: val.1,
            ..Default::default()
        }
    }
}
impl From<(f64, f64, f64)> for Box {
    fn from(val: (f64, f64, f64)) -> Self {
        Box {
            left: val.0,
            top: val.1,
            right: val.2,
            ..Default::default()
        }
    }
}
impl From<(f64, f64, f64, f64)> for Box {
    fn from(val: (f64, f64, f64, f64)) -> Self {
        Box {
            left: val.0,
            top: val.1,
            right: val.2,
            bottom: val.3,
        }
    }
}
impl Box {
    pub fn new_neg_infinity() -> Self {
        Box {
            left: f64::INFINITY,
            top: f64::INFINITY,
            right: f64::NEG_INFINITY,
            bottom: f64::NEG_INFINITY,
        }
    }
    pub fn add(&self, data: Box) -> Self {
        let mut b = self.clone();
        b.left += data.left;
        b.top += data.top;
        b.right += data.right;
        b.bottom += data.bottom;
        b
    }
    pub fn merge(&mut self, data: Box) {
        if data.left < self.left {
            self.left = data.left
        }
        if data.top < self.top {
            self.top = data.top
        }
        if data.right > self.right {
            self.right = data.right
        }
        if data.bottom > self.bottom {
            self.bottom = data.bottom
        }
    }
    pub fn width(&self) -> f64 {
        self.right - self.left
    }
    pub fn height(&self) -> f64 {
        self.bottom - self.top
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
    Err(Error {
        message: "value of rect is invalid".to_string(),
    })
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
