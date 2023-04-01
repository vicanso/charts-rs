use usvg::{Fill, Opacity, Paint, PathData, Rect, Size, Stroke, StrokeWidth};

use super::color::Color;

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

pub fn new_fill(color: Color) -> Fill {
    let mut fill = Fill::default();
    let (c, opacity) = color.divide();
    fill.paint = Paint::Color(c);
    fill.opacity = opacity;
    fill
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
