use std::fmt;
use substring::Substring;

#[derive(Clone, Copy, PartialEq, Debug, Default)]
pub struct Point {
    pub x: f32,
    pub y: f32,
}
impl From<(f32, f32)> for Point {
    fn from(val: (f32, f32)) -> Self {
        Point { x: val.0, y: val.1 }
    }
}
impl fmt::Display for Point {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let m = format!("({},{})", format_float(self.x), format_float(self.y));
        write!(f, "{m}")
    }
}

#[derive(Clone, Debug, Default)]
pub struct Box {
    pub left: f32,
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
}
impl Box {
    pub fn width(&self) -> f32 {
        self.right - self.left
    }
    pub fn height(&self) -> f32 {
        self.bottom - self.top
    }
    pub fn outer_width(&self) -> f32 {
        self.right
    }
    pub fn outer_height(&self) -> f32 {
        self.bottom
    }
}
impl fmt::Display for Box {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let m = format!(
            "({},{},{},{})",
            format_float(self.left),
            format_float(self.top),
            format_float(self.right),
            format_float(self.bottom)
        );
        write!(f, "{m}")
    }
}

impl From<f32> for Box {
    fn from(val: f32) -> Self {
        Box {
            left: val,
            top: val,
            right: val,
            bottom: val,
        }
    }
}
impl From<(f32, f32)> for Box {
    fn from(val: (f32, f32)) -> Self {
        Box {
            left: val.0,
            top: val.1,
            right: val.0,
            bottom: val.1,
        }
    }
}
impl From<(f32, f32, f32)> for Box {
    fn from(val: (f32, f32, f32)) -> Self {
        Box {
            left: val.0,
            top: val.1,
            right: val.2,
            bottom: val.1,
        }
    }
}
impl From<(f32, f32, f32, f32)> for Box {
    fn from(val: (f32, f32, f32, f32)) -> Self {
        Box {
            left: val.0,
            top: val.1,
            right: val.2,
            bottom: val.3,
        }
    }
}

pub fn format_float(value: f32) -> String {
    let str = format!("{:.1}", value);
    if str.ends_with(".0") {
        return str.substring(0, str.len() - 2).to_string();
    }
    str
}

#[derive(Clone, Debug, Default)]
pub(crate) struct AxisValueParams {
    pub data_list: Vec<f32>,
    pub min: Option<f32>,
    pub max: Option<f32>,
    pub split_number: usize,
    pub reverse: Option<bool>,
}
#[derive(Clone, Debug, Default)]
pub struct AxisValues {
    pub data: Vec<String>,
    pub min: f32,
    pub max: f32,
}

impl AxisValues {
    fn get_offset(&self) -> f32 {
        self.max - self.min
    }
    pub(crate) fn get_offset_height(&self, value: f32, max_height: f32) -> f32 {
        let percent = (value - self.min) / self.get_offset();
        max_height - percent * max_height
    }
}

pub(crate) fn get_axis_values(params: AxisValueParams) -> AxisValues {
    let mut min = 0.0;
    let mut max = f32::MIN;

    let mut split_number = params.split_number;
    if split_number == 0 {
        split_number = 6;
    }
    for item in params.data_list.iter() {
        let value = item.to_owned();
        if value > max {
            max = value;
        }
        if value < min {
            min = value;
        }
    }
    if let Some(value) = params.min {
        if value < min {
            min = value;
        }
    }
    if let Some(value) = params.max {
        if value > max {
            max = value;
        }
    }
    let mut unit = ((max - min) / split_number as f32) as i32;
    let mut base = 1;
    while unit >= 10 {
        unit /= 10;
        base *= 10;
    }

    unit = if unit < 1 { base } else { base * (unit + 1) };
    let split_unit = unit as usize;

    let mut data = vec![];
    for i in 0..=split_number {
        let value = min + (i * split_unit) as f32;
        data.push(format_float(value));
    }
    if params.reverse.unwrap_or_default() {
        data.reverse();
    }

    AxisValues {
        data,
        min,
        max: min + (split_unit * split_number) as f32,
    }
}
pub fn convert_to_points(values: &[(f32, f32)]) -> Vec<Point> {
    values.iter().map(|item| item.to_owned().into()).collect()
}

pub fn format_string(value: &str, formatter: &str) -> String {
    if formatter.is_empty() {
        value.to_string()
    } else {
        formatter.replace("{c}", &value)
    }
}

pub(crate) fn get_box_of_points(points: &[Point]) -> Box {
    let mut b = Box {
        left: f32::MAX,
        top: f32::MAX,
        ..Default::default()
    };
    for p in points.iter() {
        if p.x < b.left {
            b.left = p.x;
        }
        if p.x > b.right {
            b.right = p.x;
        }
        if p.y < b.top {
            b.top = p.y;
        }
        if p.y > b.bottom {
            b.bottom = p.y;
        }
    }
    b
}

#[cfg(test)]
mod tests {
    use super::{
        convert_to_points, format_float, get_axis_values, get_box_of_points, AxisValueParams, Box,
        Point,
    };
    use pretty_assertions::assert_eq;

    #[test]
    fn point() {
        let p: Point = (1.2, 1.3).into();

        assert_eq!(1.2, p.x);
        assert_eq!(1.3, p.y);
    }

    #[test]
    fn box_width_height() {
        let b: Box = (10.0).into();

        assert_eq!(10.0, b.left);
        assert_eq!(10.0, b.top);
        assert_eq!(10.0, b.right);
        assert_eq!(10.0, b.bottom);
        assert_eq!(0.0, b.width());
        assert_eq!(10.0, b.outer_width());
        assert_eq!(0.0, b.height());
        assert_eq!(10.0, b.outer_height());

        let b: Box = (5.0, 10.0, 30.0, 50.0).into();
        assert_eq!(5.0, b.left);
        assert_eq!(10.0, b.top);
        assert_eq!(30.0, b.right);
        assert_eq!(50.0, b.bottom);
        assert_eq!(25.0, b.width());
        assert_eq!(30.0, b.outer_width());
        assert_eq!(40.0, b.height());
        assert_eq!(50.0, b.outer_height());
    }

    #[test]
    fn format() {
        assert_eq!("1", format_float(1.0));
        assert_eq!("1.1", format_float(1.12));
    }

    #[test]
    fn axis_values() {
        let values = get_axis_values(AxisValueParams {
            data_list: vec![1.0, 10.0, 13.5, 18.9],
            ..Default::default()
        });

        assert_eq!(vec!["0", "4", "8", "12", "16", "20", "24"], values.data);
        assert_eq!(0.0, values.min);
        assert_eq!(24.0, values.max);
        assert_eq!(24.0, values.get_offset());
        assert_eq!(50.0, values.get_offset_height(12.0, 100.0));
    }

    #[test]
    fn get_box() {
        let points: Vec<Point> = convert_to_points(&vec![
            (2.0, 10.0),
            (50.0, 10.0),
            (50.0, 30.0),
            (150.0, 30.0),
            (150.0, 80.0),
            (210.0, 60.0),
            (250.0, 90.0),
        ]);
        let b = get_box_of_points(&points);
        assert_eq!(2.0, b.left);
        assert_eq!(10.0, b.top);
        assert_eq!(250.0, b.right);
        assert_eq!(90.0, b.bottom);
    }
}
