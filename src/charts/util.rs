use strict_num::NonZeroPositiveF64;
use usvg::{
    AlignmentBaseline, BaselineShift, DominantBaseline, Font, FontStretch, FontStyle, LengthAdjust,
    Paint, PaintOrder, PathData, Rect, Size, Stroke, StrokeWidth, Text, TextAnchor, TextChunk,
    TextDecoration, TextFlow, TextRendering, TextSpan, Transform, Visibility, WritingMode,
};

use super::color::Color;

#[derive(Clone, Debug, Default)]
pub enum Position {
    Left,
    Right,
    Top,
    #[default]
    Bottom,
}

#[derive(Clone, Debug, Default)]
pub enum LegendIcon {
    #[default]
    LineDot,
    Rect,
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

pub struct TextOption {
    pub font: Font,
    pub font_size: NonZeroPositiveF64,
    pub color: Color,
    pub x: f64,
    pub y: f64,
}

pub fn new_font_option(family: String, font_size: f64, color: Color) -> Result<TextOption> {
    let value = NonZeroPositiveF64::new(font_size).ok_or(Error {
        message: "font size should be > 0".to_string(),
    })?;
    Ok(TextOption {
        font: Font {
            families: family.split(',').map(|x| x.trim().to_string()).collect(),
            style: FontStyle::Normal,
            stretch: FontStretch::default(),
            weight: 0,
        },
        color,
        font_size: value,
        x: 0.0,
        y: 0.0,
    })
}

pub fn new_text(text: String, opt: TextOption) -> Text {
    let span = TextSpan {
        start: 0,
        end: text.len(),
        fill: Some(opt.color.into()),
        stroke: Some(new_stroke(1.0, opt.color)),
        paint_order: PaintOrder::default(),
        font: opt.font,
        font_size: opt.font_size,
        small_caps: false,
        apply_kerning: false,
        decoration: TextDecoration {
            underline: None,
            overline: None,
            line_through: None,
        },
        dominant_baseline: DominantBaseline::default(),
        alignment_baseline: AlignmentBaseline::default(),
        baseline_shift: vec![BaselineShift::default()],
        visibility: Visibility::default(),
        letter_spacing: 0.0,
        word_spacing: 0.0,
        text_length: None,
        length_adjust: LengthAdjust::default(),
    };
    let chunk = TextChunk {
        x: None,
        y: None,
        anchor: TextAnchor::default(),
        spans: vec![span],
        text_flow: TextFlow::Linear,
        text,
    };
    Text {
        id: String::new(),
        transform: Transform::new_translate(opt.x, opt.y),
        rendering_mode: TextRendering::default(),
        positions: vec![],
        rotate: vec![],
        writing_mode: WritingMode::LeftToRight,
        chunks: vec![chunk],
    }
}
