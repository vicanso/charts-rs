use substring::Substring;

#[derive(Clone, Copy, PartialEq, Debug, Default)]
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
            right: val.0,
            bottom: val.1,
        }
    }
}
impl From<(f64, f64, f64)> for Box {
    fn from(val: (f64, f64, f64)) -> Self {
        Box {
            left: val.0,
            top: val.1,
            right: val.2,
            bottom: val.1,
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

pub fn format_float(value: f64) -> String {
    let str = format!("{:.1}", value);
    if str.ends_with(".0") {
        return str.substring(0, str.len() - 2).to_string();
    }
    str
}

#[derive(Clone, PartialEq, Debug, Default)]
pub enum Position {
    #[default]
    Left,
    Top,
    Right,
    Bottom,
}
