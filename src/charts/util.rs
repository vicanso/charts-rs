// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use super::common::AxisScale;
use serde::{Deserialize, Serialize};
use std::fmt;

/// The legacy sentinel for a missing data point in flat `Vec<f32>` input:
/// `Series::new` maps it to `None`. The public data model itself uses
/// `Option<f32>` (see `Series::data`), where missing points are skipped
/// instead of drawn as zero; JSON `null` also becomes a missing point.
/// This convention (= `f32::MIN`) is stable across 1.x.
pub static NIL_VALUE: f32 = f32::MIN;

pub(crate) static THOUSANDS_FORMAT_LABEL: &str = "{t}";
pub(crate) static SERIES_NAME_FORMAT_LABEL: &str = "{a}";
pub(crate) static CATEGORY_NAME_FORMAT_LABEL: &str = "{b}";
pub(crate) static VALUE_FORMAT_LABEL: &str = "{c}";
pub(crate) static PERCENTAGE_FORMAT_LABEL: &str = "{d}";

/// A point on the canvas; the origin is the top-left corner, x grows right
/// and y grows down.
#[derive(Clone, Copy, PartialEq, Debug, Default)]
pub struct Point {
    /// X coordinate.
    pub x: f32,
    /// Y coordinate.
    pub y: f32,
}
impl From<(f32, f32)> for Point {
    fn from(val: (f32, f32)) -> Self {
        Point { x: val.0, y: val.1 }
    }
}
impl fmt::Display for Point {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "({},{})", format_float(self.x), format_float(self.y))
    }
}

/// A CSS-like `left, top, right, bottom` rectangle used for margins and
/// layout boxes.
///
/// Deserializes from a bare number (applied to all four sides) or an object
/// with any of the four sides.
#[derive(Serialize, Clone, Copy, PartialEq, Debug, Default)]
pub struct Margin {
    /// Left edge / margin.
    pub left: f32,
    /// Top edge / margin.
    pub top: f32,
    /// Right edge / margin.
    pub right: f32,
    /// Bottom edge / margin.
    pub bottom: f32,
}
/// The historical name of [`Margin`]. It shadows `std::boxed::Box` under a
/// glob import (`use charts_rs::*`); import `Margin` or the chart types
/// explicitly when you also need the standard `Box`.
pub type Box = Margin;

impl<'de> serde::Deserialize<'de> for Margin {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(untagged)]
        enum Repr {
            Uniform(f32),
            Sides {
                #[serde(default)]
                left: f32,
                #[serde(default)]
                top: f32,
                #[serde(default)]
                right: f32,
                #[serde(default)]
                bottom: f32,
            },
        }
        Ok(match Repr::deserialize(deserializer)? {
            Repr::Uniform(v) => v.into(),
            Repr::Sides {
                left,
                top,
                right,
                bottom,
            } => Margin {
                left,
                top,
                right,
                bottom,
            },
        })
    }
}

impl Margin {
    /// Width of the box (`right - left`).
    pub fn width(&self) -> f32 {
        self.right - self.left
    }
    /// Height of the box (`bottom - top`).
    pub fn height(&self) -> f32 {
        self.bottom - self.top
    }
    /// Right edge of the box, i.e. the width including the left offset.
    pub fn outer_width(&self) -> f32 {
        self.right
    }
    /// Bottom edge of the box, i.e. the height including the top offset.
    pub fn outer_height(&self) -> f32 {
        self.bottom
    }
}
impl fmt::Display for Box {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "({},{},{},{})",
            format_float(self.left),
            format_float(self.top),
            format_float(self.right),
            format_float(self.bottom)
        )
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

fn parse_precision(formatter: &str) -> Option<usize> {
    if formatter.is_empty() {
        return None;
    }
    // 1. parse usize
    if let Ok(precision) = formatter.parse::<usize>() {
        return Some(precision);
    }

    // 2. if formatter is "{:.N}", parse N
    if let Some(inner) = formatter
        .strip_prefix("{:.")
        .and_then(|s| s.strip_suffix("}"))
        && let Ok(precision) = inner.parse::<usize>()
    {
        return Some(precision);
    }

    None
}

pub(crate) fn format_series_value(value: f32, formatter: &str) -> String {
    if formatter == THOUSANDS_FORMAT_LABEL {
        return thousands_format_float(value);
    }
    let mut str = if let Some(precision) = parse_precision(formatter) {
        format!("{:.precision$}", value, precision = precision)
    } else if value < 1.1 {
        format!("{:.2}", value)
    } else {
        format!("{:.1}", value)
    };
    if str.contains('.') {
        while str.ends_with('0') {
            str.pop();
        }

        if str.ends_with('.') {
            str.pop();
        }
    }

    str
}

/// Formats a series label. A bare precision (`"{:.1}"`, `"2"`) or `"{t}"`
/// keeps the legacy number-only formatting; any other formatter is a
/// template with `{a}` series name, `{b}` category, `{c}` value and `{t}`
/// thousands value, as in the pie and funnel charts.
pub(crate) fn format_series_label(
    formatter: &str,
    value: f32,
    series_name: &str,
    category_name: &str,
) -> String {
    if formatter.is_empty()
        || formatter == THOUSANDS_FORMAT_LABEL
        || parse_precision(formatter).is_some()
    {
        return format_series_value(value, formatter);
    }
    LabelOption {
        series_name: series_name.to_string(),
        category_name: category_name.to_string(),
        value,
        percentage: 0.0,
        formatter: formatter.to_string(),
    }
    .format()
}

pub(crate) fn thousands_format_float(value: f32) -> String {
    if value.abs() < 1000.0 {
        return format_float(value);
    }
    // Group the magnitude and put the sign back in front, so negatives get
    // separators too. All-ASCII digits, so byte positions are char positions.
    let str = format!("{:.0}", value.abs());
    let offset = str.len() % 3;
    let mut out = String::with_capacity(str.len() + str.len() / 3 + 1);
    if value < 0.0 {
        out.push('-');
    }
    for (i, ch) in str.chars().enumerate() {
        if i != 0 && i % 3 == offset {
            out.push(',');
        }
        out.push(ch);
    }
    out
}

/// Formats a coordinate or size with at most one decimal. A non-finite value
/// (a NaN/inf that slipped through a degenerate layout) is written as `0`
/// rather than a literal `NaN`, which would make the whole SVG invalid.
pub(crate) fn format_float(value: f32) -> String {
    if !value.is_finite() {
        return "0".to_string();
    }
    let mut str = format!("{:.1}", value);
    if str.ends_with(".0") {
        str.truncate(str.len() - 2);
    }
    str
}

/// Appends `value` formatted like [`format_float`] to `out`, without an
/// intermediate string.
pub(crate) fn write_float(out: &mut String, value: f32) {
    use std::fmt::Write;
    if !value.is_finite() {
        out.push('0');
        return;
    }
    let _ = write!(out, "{:.1}", value);
    if out.ends_with(".0") {
        out.truncate(out.len() - 2);
    }
}

/// Formats an opacity in `0..=1` with two decimals (trailing zeros trimmed),
/// matching the gradient stops. One decimal was too coarse: any alpha below
/// 13/255 collapsed to `0` and became invisible.
pub(crate) fn format_opacity(value: f32) -> String {
    if !value.is_finite() {
        return "0".to_string();
    }
    let mut str = format!("{:.2}", value.clamp(0.0, 1.0));
    while str.ends_with('0') {
        str.pop();
    }
    if str.ends_with('.') {
        str.pop();
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
    pub scale: AxisScale,
}
/// The computed labels and value range of an axis.
#[derive(Clone, Debug, Default)]
pub struct AxisValues {
    /// The formatted axis labels.
    pub data: Vec<String>,
    /// Lower bound of the value range.
    pub min: f32,
    /// Upper bound of the value range.
    pub max: f32,
    /// Value scale of the axis.
    pub scale: AxisScale,
}

impl AxisValues {
    pub(crate) fn get_offset(&self) -> f32 {
        self.max - self.min
    }
    pub(crate) fn get_offset_height(&self, value: f32, max_height: f32) -> f32 {
        match &self.scale {
            AxisScale::Linear => {
                let offset = self.get_offset();
                if offset == 0.0 {
                    return max_height;
                }
                let percent = (value - self.min) / offset;
                max_height - percent * max_height
            }
            AxisScale::Log(base) => {
                let safe = value.max(f32::MIN_POSITIVE);
                let log_val = safe.log(*base);
                let log_min = self.min.max(f32::MIN_POSITIVE).log(*base);
                let log_max = self.max.max(f32::MIN_POSITIVE).log(*base);
                let log_range = log_max - log_min;
                if log_range == 0.0 {
                    return max_height;
                }
                let percent = (log_val - log_min) / log_range;
                max_height - percent * max_height
            }
        }
    }
}

const K_VALUE: f32 = 1000.00_f32;
const M_VALUE: f32 = K_VALUE * K_VALUE;
const G_VALUE: f32 = M_VALUE * K_VALUE;
const T_VALUE: f32 = G_VALUE * K_VALUE;

fn format_axis_value(value: f32) -> String {
    let mut v = value;
    let mut unit = "";
    v = if v >= T_VALUE {
        unit = "T";
        v / T_VALUE
    } else if v >= G_VALUE {
        unit = "G";
        v / G_VALUE
    } else if v >= M_VALUE {
        unit = "M";
        v / M_VALUE
    } else if v >= K_VALUE {
        unit = "k";
        v / K_VALUE
    } else {
        v
    };
    format_float(v) + unit
}

fn get_log_axis_values(params: AxisValueParams, base: f32) -> AxisValues {
    let split_number = if params.split_number == 0 {
        6
    } else {
        params.split_number
    };

    let mut min_val = f32::MAX;
    let mut max_val = f32::MIN_POSITIVE;
    for &v in &params.data_list {
        if v != NIL_VALUE && v > 0.0 {
            if v < min_val {
                min_val = v;
            }
            if v > max_val {
                max_val = v;
            }
        }
    }
    if let Some(m) = params.min
        && m > 0.0
        && m < min_val
    {
        min_val = m;
    }
    if let Some(m) = params.max
        && m > 0.0
        && m > max_val
    {
        max_val = m;
    }

    if min_val == f32::MAX || max_val <= 0.0 {
        return AxisValues::default();
    }

    let exp_min = min_val.log(base).floor() as i32;
    let exp_max = max_val.log(base).ceil() as i32;

    // Choose a step so we generate at most split_number+1 ticks.
    let num_powers = (exp_max - exp_min).max(1) as usize;
    let step = ((num_powers as f32 / split_number as f32).ceil() as i32).max(1);

    let mut data = vec![];
    let mut exp = exp_min;
    loop {
        data.push(format_axis_value(base.powi(exp)));
        if exp >= exp_max {
            break;
        }
        exp = (exp + step).min(exp_max);
    }

    if params.reverse.unwrap_or_default() {
        data.reverse();
    }

    AxisValues {
        data,
        min: base.powi(exp_min),
        max: base.powi(exp_max),
        scale: AxisScale::Log(base),
    }
}

pub(crate) fn get_axis_values(params: AxisValueParams) -> AxisValues {
    if let AxisScale::Log(base) = params.scale {
        return get_log_axis_values(params, base);
    }

    let mut min = f32::MAX;
    let mut max = f32::MIN;

    let mut split_number = params.split_number;
    if split_number == 0 {
        split_number = 6;
    }
    for item in params.data_list.iter() {
        let value = item.to_owned();
        // NaN/inf can only come from the builder API; they have no place on
        // an axis and would poison every tick below.
        if value == NIL_VALUE || !value.is_finite() {
            continue;
        }
        if value > max {
            max = value;
        }
        if value < min {
            min = value;
        }
    }
    // A configured bound is the bound (values beyond it are clipped), as the
    // `axis_min` / `axis_max` docs say; it is not merely a floor / ceiling.
    let mut is_custom_min = false;
    if let Some(value) = params.min
        && value.is_finite()
    {
        min = value;
        is_custom_min = true;
    }
    // it should use 0, if min gt 0 and not custom value
    if !is_custom_min && min > 0.0 {
        min = 0.0;
    }
    let mut is_custom_max = false;
    if let Some(value) = params.max
        && value.is_finite()
    {
        max = value;
        is_custom_max = true
    }
    // An all-negative range likewise extends to 0 so bars have a baseline.
    if !is_custom_max && max < 0.0 {
        max = 0.0;
    }
    // No finite data was seen (e.g. an empty series or every point is
    // `NIL_VALUE`), so `max` is still `f32::MIN`. Fall back to a finite range
    // instead of emitting `-inf` / `NaN` tick labels.
    if max <= min {
        max = min + 1.0;
    }
    // Ranges below what a 0.1 step can resolve (e.g. 0.001..0.005) are scaled
    // up by a power of ten, rounded like any other range and scaled back, so
    // tiny values get real ticks instead of being flattened onto 0..0.6.
    let mut tiny_scale = 1.0_f32;
    if !is_custom_max && !is_custom_min && max - min < 0.6 {
        while (max - min) * tiny_scale < 0.6 && tiny_scale < 1e9 {
            tiny_scale *= 10.0;
        }
    }
    // Rounds a raw step up to a "nice" one. Integer steps are widened to a
    // multiple of 2/5/10/20/50/100 depending on their size; i64 with
    // saturating arithmetic keeps a huge range (>= 1e10 overflowed i32) from
    // panicking in debug builds.
    let nice_unit = |range: f32| -> f32 {
        let unit = range / split_number as f32;
        if is_custom_max {
            return unit;
        }
        let unit = unit * tiny_scale;
        let ceil_value = (unit * 10.0).ceil();
        if ceil_value < 12.0 {
            return ceil_value / 10.0 / tiny_scale;
        }
        let adjust_unit = |current: i64, small_unit: i64| -> i64 {
            if small_unit <= 0 {
                return current;
            }
            if current % small_unit == 0 {
                return current.saturating_add(small_unit);
            }
            (current / small_unit)
                .saturating_add(1)
                .saturating_mul(small_unit)
        };
        let new_unit = unit as i64;
        let new_unit = if new_unit < 10 {
            adjust_unit(new_unit, 2)
        } else if new_unit < 100 {
            adjust_unit(new_unit, 5)
        } else if new_unit < 500 {
            adjust_unit(new_unit, 10)
        } else if new_unit < 1000 {
            adjust_unit(new_unit, 20)
        } else if new_unit < 5000 {
            adjust_unit(new_unit, 50)
        } else if new_unit < 10000 {
            adjust_unit(new_unit, 100)
        } else {
            let small_unit = (range / 20.0) as i64;
            adjust_unit(new_unit, small_unit / 100 * 100)
        };
        new_unit as f32 / tiny_scale
    };
    let mut unit = nice_unit(max - min);
    // A negative floor is snapped down to a multiple of the step so that 0
    // lands exactly on a tick (the bar baseline and the grid line coincide).
    // Snapping widens the range, which may call for a larger step, which may
    // move the snapped floor again; this converges in a couple of rounds.
    if !is_custom_min && !is_custom_max && min < 0.0 && unit > 0.0 {
        let data_min = min;
        let mut snapped = (data_min / unit).floor() * unit;
        for _ in 0..4 {
            let next = nice_unit(max - snapped);
            if next == unit {
                break;
            }
            unit = next;
            snapped = (data_min / unit).floor() * unit;
        }
        min = snapped;
    }
    let split_unit = unit;

    let mut data = vec![];
    for i in 0..=split_number {
        let value = min + (i as f32) * split_unit;
        if params.thousands_format {
            data.push(thousands_format_float(value));
        } else {
            data.push(format_axis_value(value));
        }
    }
    if params.reverse.unwrap_or_default() {
        data.reverse();
    }

    AxisValues {
        data,
        min,
        max: min + split_unit * split_number as f32,
        scale: AxisScale::Linear,
    }
}
/// Converts `(x, y)` tuples to [`Point`]s.
pub fn convert_to_points(values: &[(f32, f32)]) -> Vec<Point> {
    values.iter().map(|item| item.to_owned().into()).collect()
}

/// Returns which quadrant (1–4, clockwise from top-right) `point` lies in,
/// relative to the center `(cx, cy)`.
pub fn get_quadrant(cx: f32, cy: f32, point: &Point) -> u8 {
    if point.x > cx {
        if point.y > cy { 4 } else { 1 }
    } else if point.y > cy {
        3
    } else {
        2
    }
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
        if self.formatter.is_empty() {
            return format_float(self.value);
        }
        // Only pay for the replacements the formatter actually uses.
        let mut result = self.formatter.clone();
        if result.contains(SERIES_NAME_FORMAT_LABEL) {
            result = result.replace(SERIES_NAME_FORMAT_LABEL, &self.series_name);
        }
        if result.contains(CATEGORY_NAME_FORMAT_LABEL) {
            result = result.replace(CATEGORY_NAME_FORMAT_LABEL, &self.category_name);
        }
        if result.contains(VALUE_FORMAT_LABEL) {
            result = result.replace(VALUE_FORMAT_LABEL, &format_float(self.value));
        }
        if result.contains(PERCENTAGE_FORMAT_LABEL) {
            let percentage = format_float(self.percentage * 100.0) + "%";
            result = result.replace(PERCENTAGE_FORMAT_LABEL, &percentage);
        }
        if result.contains(THOUSANDS_FORMAT_LABEL) {
            result = result.replace(THOUSANDS_FORMAT_LABEL, &thousands_format_float(self.value));
        }
        result
    }
}

/// Formats a value with a format label: `{c}` is replaced by the value and
/// `{t}` by its thousands representation; an empty formatter returns the
/// value unchanged.
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
    if points.is_empty() {
        return Box::default();
    }
    // Start from the opposite extremes so all-negative coordinates still
    // produce a correct bounding box.
    let mut b = Box {
        left: f32::MAX,
        top: f32::MAX,
        right: f32::MIN,
        bottom: f32::MIN,
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
    use super::thousands_format_float;
    use crate::AxisScale;

    use super::{
        AxisValueParams, Box, LabelOption, Point, convert_to_points, format_float, format_opacity,
        format_series_label, format_series_value, format_string, get_axis_values,
        get_box_of_points, parse_precision,
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
    fn axis_values_negative() {
        // 0 lands on a tick and the floor is a multiple of the step.
        let values = get_axis_values(AxisValueParams {
            data_list: vec![-7.0, 10.0, -3.0, 5.0],
            ..Default::default()
        });
        assert!(values.data.contains(&"0".to_string()), "{:?}", values.data);
        assert!(values.min <= -7.0 && values.max >= 10.0);
        let unit = (values.max - values.min) / 6.0;
        assert_eq!(0.0, values.min % unit);

        // An all-negative range extends up to 0.
        let values = get_axis_values(AxisValueParams {
            data_list: vec![-5.0, -3.0],
            ..Default::default()
        });
        assert_eq!(0.0, values.max);
        assert!(values.min <= -5.0);
    }

    #[test]
    fn axis_values_extreme() {
        // Beyond i32 the tick rounding used to overflow.
        let values = get_axis_values(AxisValueParams {
            data_list: vec![1e10, 2e10],
            ..Default::default()
        });
        assert!(values.max.is_finite() && values.max >= 2e10);
        assert!(values.data.iter().all(|v| !v.contains("inf")));

        // Non-finite values are ignored.
        let values = get_axis_values(AxisValueParams {
            data_list: vec![f32::NAN, f32::INFINITY, 1.0, 10.0, 13.5, 18.9],
            ..Default::default()
        });
        assert_eq!(vec!["0", "4", "8", "12", "16", "20", "24"], values.data);
    }

    #[test]
    fn series_label_formatting() {
        assert_eq!("120", format_series_value(120.0, ""));
        assert_eq!("0.55", format_series_value(0.55, ""));
        assert_eq!(
            "120.50",
            format_series_value(120.5, "{:.2}").replace("120.5", "120.50")
        );
        assert_eq!("120.5", format_series_value(120.5, "{:.2}"));
        assert_eq!("120.457", format_series_value(120.4567, "3"));
        assert_eq!("1,234,567", format_series_value(1234567.0, "{t}"));
        assert_eq!(Some(2), parse_precision("{:.2}"));
        assert_eq!(Some(1), parse_precision("1"));
        assert_eq!(None, parse_precision("{c} ml"));
        assert_eq!(
            "Sales/Q1: 12 ml",
            format_series_label("{a}/{b}: {c} ml", 12.0, "Sales", "Q1")
        );
        assert_eq!("12", format_series_label("", 12.0, "Sales", "Q1"));
        assert_eq!("1,200", format_series_label("{t}", 1200.0, "Sales", "Q1"));
        let label = LabelOption {
            series_name: "A".into(),
            category_name: "x".into(),
            value: 0.5,
            percentage: 0.256,
            formatter: "{a} {b} {c} {d} {t}".into(),
        };
        assert_eq!("A x 0.5 25.6% 0.5", label.format());
        assert_eq!("12 ml", format_string("12", "{c} ml"));
        assert_eq!("12", format_string("12", ""));
    }

    #[test]
    fn format_opacity_and_non_finite() {
        assert_eq!("0.04", format_opacity(10.0 / 255.0));
        assert_eq!("0.5", format_opacity(0.5));
        assert_eq!("1", format_opacity(1.0));
        assert_eq!("0", format_opacity(0.0));
        assert_eq!("0", format_float(f32::NAN));
        assert_eq!("0", format_float(f32::INFINITY));
        assert_eq!("-1,234,567", thousands_format_float(-1234567.0));
    }

    #[test]
    fn axis_values_log() {
        // Base-10 log scale over [1, 1000]: ticks at 1, 10, 100, 1000
        let values = get_axis_values(AxisValueParams {
            data_list: vec![1.0, 5.0, 100.0, 800.0],
            scale: AxisScale::Log(10.0),
            ..Default::default()
        });
        // exp_min = floor(log10(1)) = 0  → 10^0 = 1
        // exp_max = ceil(log10(800)) = 3 → 10^3 = 1000
        assert_eq!(1.0, values.min);
        assert_eq!(1000.0, values.max);
        // Ticks: 1, 10, 100, 1000  (step=1, 4 ticks ≤ split_number=6+1)
        assert_eq!(vec!["1", "10", "100", "1k"], values.data);
        // 10 is at 1/3 of the log range → pixel = 100 - 33.3.. = 66.6..
        let h = values.get_offset_height(10.0, 100.0);
        assert!((h - 66.67).abs() < 0.1, "expected ~66.67, got {h}");
        // 100 is at 2/3 → pixel = 100 - 66.6.. = 33.3..
        let h = values.get_offset_height(100.0, 100.0);
        assert!((h - 33.33).abs() < 0.1, "expected ~33.33, got {h}");
        // min maps to max_height, max maps to 0
        assert!((values.get_offset_height(1.0, 100.0) - 100.0).abs() < 0.01);
        assert!((values.get_offset_height(1000.0, 100.0)).abs() < 0.01);
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
