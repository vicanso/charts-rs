use serde::{Deserialize, Serialize};
use std::fmt;
use substring::Substring;

pub static NIL_VALUE: f32 = f32::MIN;

pub(crate) static THOUSANDS_FORMAT_LABEL: &str = "{t}";
pub(crate) static SERIES_NAME_FORMAT_LABEL: &str = "{a}";
pub(crate) static CATEGORY_NAME_FORMAT_LABEL: &str = "{b}";
pub(crate) static VALUE_FORMAT_LABEL: &str = "{c}";
pub(crate) static PERCENTAGE_FORMAT_LABEL: &str = "{d}";

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

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
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

pub(crate) fn format_series_value(value: f32, formatter: &str) -> String {
    if formatter == THOUSANDS_FORMAT_LABEL {
        return thousands_format_float(value);
    }
    format_float(value)
}

pub(crate) fn thousands_format_float(value: f32) -> String {
    if value < 1000.0 {
        return format_float(value);
    }
    let str = format!("{:.0}", value);
    let unit = 3;
    let mut index = str.len() % unit;
    let mut arr = vec![];
    if index != 0 {
        arr.push(str.substring(0, index))
    }

    loop {
        if index >= str.len() {
            break;
        }
        arr.push(str.substring(index, index + unit));
        index += unit;
    }
    arr.join(",")
}

pub(crate) fn format_float(value: f32) -> String {
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
    pub thousands_format: bool,
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

const K_VALUE: f32 = 1000.00_f32;
const M_VALUE: f32 = K_VALUE * K_VALUE;
const G_VALUE: f32 = M_VALUE * K_VALUE;
const T_VALUE: f32 = G_VALUE * K_VALUE;

pub(crate) fn get_axis_values(params: AxisValueParams) -> AxisValues {
    let mut min = f32::MAX;
    let mut max = f32::MIN;

    let mut split_number = params.split_number;
    if split_number == 0 {
        split_number = 6;
    }
    for item in params.data_list.iter() {
        let value = item.to_owned();
        if value == NIL_VALUE {
            continue;
        }
        if value > max {
            max = value;
        }
        if value < min {
            min = value;
        }
    }
    let mut is_custom_min = false;

    if let Some(value) = params.min {
        if value < min {
            min = value;
            is_custom_min = true;
        }
    }
    // it should use 0, if min gt 0 and not custom value
    if !is_custom_min && min > 0.0 {
        min = 0.0;
    }
    let mut is_custom_max = false;
    if let Some(value) = params.max {
        if value > max {
            max = value;
            is_custom_max = true
        }
    }
    let mut unit = (max - min) / split_number as f32;
    if !is_custom_max {
        let ceil_value = (unit * 10.0).ceil();
        if ceil_value < 12.0 {
            unit = ceil_value / 10.0;
        } else {
            let mut new_unit = unit as i32;
            let adjust_unit = |current: i32, small_unit: i32| -> i32 {
                if current % small_unit == 0 {
                    return current + small_unit;
                }
                ((current / small_unit) + 1) * small_unit
            };
            if new_unit < 10 {
                new_unit = adjust_unit(new_unit, 2);
            } else if new_unit < 100 {
                new_unit = adjust_unit(new_unit, 5);
            } else if new_unit < 500 {
                new_unit = adjust_unit(new_unit, 10);
            } else if new_unit < 1000 {
                new_unit = adjust_unit(new_unit, 20);
            } else if new_unit < 5000 {
                new_unit = adjust_unit(new_unit, 50);
            } else if new_unit < 10000 {
                new_unit = adjust_unit(new_unit, 100);
            } else {
                let small_unit = ((max - min) / 20.0) as i32;
                new_unit = adjust_unit(new_unit, small_unit / 100 * 100);
            }
            unit = new_unit as f32;
        }
    }
    let split_unit = unit;

    let mut data = vec![];
    for i in 0..=split_number {
        let mut value = min + (i as f32) * split_unit;
        if params.thousands_format {
            data.push(thousands_format_float(value));
            continue;
        }
        let mut unit = "";
        value = if value >= T_VALUE {
            unit = "T";
            value / T_VALUE
        } else if value >= G_VALUE {
            unit = "G";
            value / G_VALUE
        } else if value >= M_VALUE {
            unit = "M";
            value / M_VALUE
        } else if value >= K_VALUE {
            unit = "k";
            value / K_VALUE
        } else {
            value
        };
        data.push(format_float(value) + unit);
    }
    if params.reverse.unwrap_or_default() {
        data.reverse();
    }

    AxisValues {
        data,
        min,
        max: min + split_unit * split_number as f32,
    }
}
pub fn convert_to_points(values: &[(f32, f32)]) -> Vec<Point> {
    values.iter().map(|item| item.to_owned().into()).collect()
}

#[derive(Clone, Debug, Default)]
pub(crate) struct LabelOption {
    pub series_name: String,
    pub category_name: String,
    pub value: f32,
    pub percentage: f32,
    pub formatter: String,
}
impl LabelOption {
    pub fn format(&self) -> String {
        // {a} for series name, {b} for category name, {c} for data value, {d} for percentage
        let value = format_float(self.value);
        let percentage = format_float(self.percentage * 100.0) + "%";
        if self.formatter.is_empty() {
            return value;
        }
        self.formatter
            .replace(SERIES_NAME_FORMAT_LABEL, &self.series_name)
            .replace(CATEGORY_NAME_FORMAT_LABEL, &self.category_name)
            .replace(VALUE_FORMAT_LABEL, &value)
            .replace(PERCENTAGE_FORMAT_LABEL, &percentage)
            .replace(THOUSANDS_FORMAT_LABEL, &thousands_format_float(self.value))
    }
}

pub fn format_string(value: &str, formatter: &str) -> String {
    if formatter.is_empty() {
        value.to_string()
    } else {
        formatter
            .replace(VALUE_FORMAT_LABEL, value)
            .replace(THOUSANDS_FORMAT_LABEL, value)
    }
}

pub(crate) fn get_pie_point(cx: f32, cy: f32, r: f32, angle: f32) -> Point {
    let value = angle / 180.0 * std::f32::consts::PI;
    let x = cx + r * value.sin();
    let y = cy - r * value.cos();
    Point { x, y }
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
    use crate::thousands_format_float;

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
        assert_eq!("100.1", format_float(100.14));
        assert_eq!("100", format_float(100.04));
        assert_eq!("1000.1", format_float(1000.14));
    }
    #[test]
    fn thousands_format() {
        assert_eq!("1", thousands_format_float(1.0));
        assert_eq!("1.1", thousands_format_float(1.12));
        assert_eq!("100.1", thousands_format_float(100.14));
        assert_eq!("100", thousands_format_float(100.04));
        assert_eq!("1,000", thousands_format_float(1000.14));
        assert_eq!("100,000", thousands_format_float(100000.14));
        assert_eq!("1,000,000", thousands_format_float(1_000_000.1));
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
        let points: Vec<Point> = convert_to_points(&[
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
