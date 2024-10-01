// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use super::{Align, Box, Color, LegendCategory, Series, SeriesCategory, Theme, YAxisConfig};
use crate::{
    MarkLine, MarkLineCategory, MarkPoint, MarkPointCategory, Position, Symbol, NIL_VALUE,
};
use std::sync::Arc;

/// Gets bool value from serde json.
pub(crate) fn get_bool_from_value(value: &serde_json::Value, key: &str) -> Option<bool> {
    if let Some(value) = value.get(key) {
        if let Some(b) = value.as_bool() {
            return Some(b);
        }
    }
    None
}

/// Gets usize value from serde json.
pub(crate) fn get_usize_from_value(value: &serde_json::Value, key: &str) -> Option<usize> {
    if let Some(value) = value.get(key) {
        if let Some(u) = value.as_u64() {
            return Some(u as usize);
        }
    }
    None
}

/// Gets usize slice value from serde json.
pub(crate) fn get_usize_slice_from_value(
    value: &serde_json::Value,
    key: &str,
) -> Option<Vec<usize>> {
    if let Some(arr) = value.get(key) {
        if let Some(values) = arr.as_array() {
            return Some(
                values
                    .iter()
                    .map(|item| {
                        if let Some(v) = item.as_u64() {
                            v as usize
                        } else {
                            0
                        }
                    })
                    .collect(),
            );
        }
    }
    None
}

/// Gets float32 value from serde json.
pub(crate) fn get_f32_from_value(value: &serde_json::Value, key: &str) -> Option<f32> {
    if let Some(value) = value.get(key) {
        if value.is_null() {
            return None;
        }
        if let Some(v) = value.as_f64() {
            return Some(v as f32);
        }
    }
    None
}

/// Gets float32 slice value from serde json, the value can be null.
pub(crate) fn get_f32_slice_from_value_support_nil(
    value: &serde_json::Value,
    key: &str,
) -> Option<Vec<f32>> {
    if let Some(arr) = value.get(key) {
        if let Some(values) = arr.as_array() {
            return Some(
                values
                    .iter()
                    .map(|item| {
                        if item.is_null() {
                            return NIL_VALUE;
                        }
                        if let Some(v) = item.as_f64() {
                            v as f32
                        } else {
                            0.0
                        }
                    })
                    .collect(),
            );
        }
    }
    None
}

/// Gets float32 slice value from serde json, the value will set as 0.0 if not float32.
pub(crate) fn get_f32_slice_from_value(value: &serde_json::Value, key: &str) -> Option<Vec<f32>> {
    if let Some(arr) = value.get(key) {
        if let Some(values) = arr.as_array() {
            return Some(
                values
                    .iter()
                    .map(|item| {
                        if let Some(v) = item.as_f64() {
                            v as f32
                        } else {
                            0.0
                        }
                    })
                    .collect(),
            );
        }
    }
    None
}

fn convert_to_align(value: &serde_json::Value) -> Option<Align> {
    if value.is_null() {
        return None;
    }
    if let Some(value) = value.as_str() {
        let value = match value.to_lowercase().as_str() {
            "left" => Align::Left,
            "right" => Align::Right,
            _ => Align::Center,
        };
        return Some(value);
    }
    None
}

/// Gets align slice value from serde json.
pub(crate) fn get_align_slice_from_value(
    value: &serde_json::Value,
    key: &str,
) -> Option<Vec<Align>> {
    if let Some(arr) = value.get(key) {
        let mut align_list = vec![];
        if let Some(values) = arr.as_array() {
            for item in values.iter() {
                if let Some(align) = convert_to_align(item) {
                    align_list.push(align);
                }
            }
        }
        return Some(align_list);
    }
    None
}

/// Gets align value from serde json.
pub(crate) fn get_align_from_value(value: &serde_json::Value, key: &str) -> Option<Align> {
    if let Some(value) = value.get(key) {
        return convert_to_align(value);
    }
    None
}

/// Gets legend category value from serde json.
pub(crate) fn get_legend_category_from_value(
    value: &serde_json::Value,
    key: &str,
) -> Option<LegendCategory> {
    if let Some(value) = value.get(key) {
        if value.is_null() {
            return None;
        }
        if let Some(value) = value.as_str() {
            let value = match value.to_lowercase().as_str() {
                "rect" => LegendCategory::Rect,
                "round_rect" => LegendCategory::RoundRect,
                "circle" => LegendCategory::Circle,
                _ => LegendCategory::Normal,
            };
            return Some(value);
        }
    }
    None
}

/// Gets margin box value from serde json.
pub(crate) fn get_margin_from_value(value: &serde_json::Value, key: &str) -> Option<Box> {
    if let Some(data) = value.get(key) {
        if data.is_null() {
            return None;
        }
        return Some(get_box_from_value(data));
    }
    None
}

fn get_box_from_value(value: &serde_json::Value) -> Box {
    Box {
        left: get_f32_from_value(value, "left").unwrap_or_default(),
        top: get_f32_from_value(value, "top").unwrap_or_default(),
        right: get_f32_from_value(value, "right").unwrap_or_default(),
        bottom: get_f32_from_value(value, "bottom").unwrap_or_default(),
    }
}

/// Gets string slice value from serde json.
pub(crate) fn get_string_slice_from_value(
    value: &serde_json::Value,
    key: &str,
) -> Option<Vec<String>> {
    if let Some(arr) = value.get(key) {
        if let Some(values) = arr.as_array() {
            return Some(
                values
                    .iter()
                    .map(|item| item.as_str().unwrap_or_default().to_string())
                    .collect(),
            );
        }
    }
    None
}

/// Gets y axis config value from serde json.
pub(crate) fn get_y_axis_config_from_value(t: Arc<Theme>, item: &serde_json::Value) -> YAxisConfig {
    let mut y_config = YAxisConfig {
        axis_font_size: t.y_axis_font_size,
        axis_font_color: t.y_axis_font_color,
        axis_stroke_color: t.y_axis_stroke_color,
        axis_split_number: t.y_axis_split_number,
        axis_name_gap: t.y_axis_name_gap,
        ..Default::default()
    };
    if let Some(axis_font_size) = get_f32_from_value(item, "axis_font_size") {
        y_config.axis_font_size = axis_font_size;
    }
    if let Some(axis_font_color) = get_color_from_value(item, "axis_font_color") {
        y_config.axis_font_color = axis_font_color;
    }
    if let Some(axis_font_weight) = get_string_from_value(item, "axis_font_weight") {
        y_config.axis_font_weight = Some(axis_font_weight);
    }
    if let Some(axis_stroke_color) = get_color_from_value(item, "axis_stroke_color") {
        y_config.axis_stroke_color = axis_stroke_color;
    }
    if let Some(axis_width) = get_f32_from_value(item, "axis_width") {
        y_config.axis_width = Some(axis_width);
    }
    if let Some(axis_split_number) = get_usize_from_value(item, "axis_split_number") {
        y_config.axis_split_number = axis_split_number;
    }
    if let Some(axis_name_gap) = get_f32_from_value(item, "axis_name_gap") {
        y_config.axis_name_gap = axis_name_gap;
    }
    if let Some(axis_formatter) = get_string_from_value(item, "axis_formatter") {
        y_config.axis_formatter = Some(axis_formatter);
    }
    if let Some(axis_margin) = get_margin_from_value(item, "axis_margin") {
        y_config.axis_margin = Some(axis_margin);
    }
    if let Some(axis_min) = get_f32_from_value(item, "axis_min") {
        y_config.axis_min = Some(axis_min);
    }
    if let Some(axis_max) = get_f32_from_value(item, "axis_max") {
        y_config.axis_max = Some(axis_max);
    }
    y_config
}

/// Gets y axis config value from serde json.
pub(crate) fn get_y_axis_configs_from_value(
    t: Arc<Theme>,
    value: &serde_json::Value,
    key: &str,
) -> Option<Vec<YAxisConfig>> {
    if let Some(arr) = value.get(key) {
        if let Some(values) = arr.as_array() {
            return Some(
                values
                    .iter()
                    .map(|item| get_y_axis_config_from_value(t.clone(), item))
                    .collect(),
            );
        }
    }
    None
}

/// Gets color value from serde json.
pub(crate) fn get_color_slice_from_value(
    value: &serde_json::Value,
    key: &str,
) -> Option<Vec<Color>> {
    if let Some(arr) = value.get(key) {
        if let Some(values) = arr.as_array() {
            return Some(
                values
                    .iter()
                    .map(|item| item.as_str().unwrap_or_default().into())
                    .collect(),
            );
        }
    }
    None
}

/// Gets string value from serde json.
pub(crate) fn get_string_from_value(value: &serde_json::Value, key: &str) -> Option<String> {
    if let Some(s) = value.get(key) {
        if s.is_null() {
            return None;
        }
        if let Some(v) = s.as_str() {
            return Some(v.to_string());
        }
    }
    None
}

/// Gets position value from serde json.
pub(crate) fn get_position_from_value(value: &serde_json::Value, key: &str) -> Option<Position> {
    if let Some(value) = get_string_from_value(value, key) {
        let p = match value.as_str() {
            "inside" => Position::Inside,
            "top" => Position::Top,
            "right" => Position::Right,
            "bottom" => Position::Bottom,
            _ => Position::Left,
        };
        return Some(p);
    }
    None
}

/// Gets color value from serde json.
pub(crate) fn get_color_from_value(value: &serde_json::Value, key: &str) -> Option<Color> {
    if let Some(s) = get_string_from_value(value, key) {
        return Some(s.as_str().into());
    }
    None
}

fn get_series_category_from_value(value: &serde_json::Value, key: &str) -> Option<SeriesCategory> {
    if let Some(value) = value.get(key) {
        if value.is_null() {
            return None;
        }
        if let Some(value) = value.as_str() {
            return match value.to_lowercase().as_str() {
                "line" => Some(SeriesCategory::Line),
                "bar" => Some(SeriesCategory::Bar),
                _ => None,
            };
        }
    }
    None
}

/// Gets series symbol value from serde json.
pub(crate) fn get_series_symbol_from_value(value: &serde_json::Value, key: &str) -> Option<Symbol> {
    if let Some(value) = value.get(key) {
        if value.is_null() {
            return Some(Symbol::None);
        }
        // only supports circle
        let color = get_color_from_value(value, "color");
        let radius = get_f32_from_value(value, "radius").unwrap_or(3.0);
        return Some(Symbol::Circle(radius, color));
    }
    None
}

fn get_mark_lines(value: &serde_json::Value, key: &str) -> Vec<MarkLine> {
    let mut mark_lines = vec![];
    if let Some(data) = value.get(key) {
        if let Some(arr) = data.as_array() {
            for item in arr.iter() {
                if let Some(value) = item.get("category") {
                    let category = match value.as_str().unwrap_or_default() {
                        "max" => MarkLineCategory::Max,
                        "min" => MarkLineCategory::Min,
                        _ => MarkLineCategory::Average,
                    };
                    mark_lines.push(MarkLine { category })
                }
            }
        }
    }
    mark_lines
}

fn get_mark_points(value: &serde_json::Value, key: &str) -> Vec<MarkPoint> {
    let mut mark_points = vec![];
    if let Some(data) = value.get(key) {
        if let Some(arr) = data.as_array() {
            for item in arr.iter() {
                if let Some(value) = item.get("category") {
                    let category = match value.as_str().unwrap_or_default() {
                        "max" => MarkPointCategory::Max,
                        _ => MarkPointCategory::Min,
                    };
                    mark_points.push(MarkPoint { category })
                }
            }
        }
    }
    mark_points
}

fn get_series_colors_from_value(
    value: &serde_json::Value,
    key: &str,
) -> Option<Vec<Option<Color>>> {
    if let Some(data) = value.get(key) {
        if let Some(arr) = data.as_array() {
            let mut colors = vec![];
            for item in arr.iter() {
                if item.is_null() {
                    colors.push(None);
                } else if let Some(str) = item.as_str() {
                    colors.push(Some(str.into()))
                }
            }
            return Some(colors);
        }
    }
    None
}

fn get_series_from_value(value: &serde_json::Value) -> Option<Series> {
    let name = get_string_from_value(value, "name").unwrap_or_default();
    let data = get_f32_slice_from_value_support_nil(value, "data").unwrap_or_default();
    if data.is_empty() {
        return None;
    }
    Some(Series {
        name,
        data,
        index: get_usize_from_value(value, "index"),
        y_axis_index: get_usize_from_value(value, "y_axis_index").unwrap_or_default(),
        label_show: get_bool_from_value(value, "label_show").unwrap_or_default(),
        category: get_series_category_from_value(value, "category"),
        start_index: get_usize_from_value(value, "start_index").unwrap_or_default(),
        mark_lines: get_mark_lines(value, "mark_lines"),
        mark_points: get_mark_points(value, "mark_points"),
        colors: get_series_colors_from_value(value, "colors"),
        stroke_dash_array: get_string_from_value(value, "stroke_dash_array"),
    })
}

/// Gets series list value from serde json.
pub(crate) fn get_series_list_from_value(value: &serde_json::Value) -> Option<Vec<Series>> {
    if let Some(data) = value.get("series_list") {
        if let Some(arr) = data.as_array() {
            let mut series_list = vec![];
            for (index, item) in arr.iter().enumerate() {
                if let Some(mut series) = get_series_from_value(item) {
                    if series.index.is_none() {
                        series.index = Some(index)
                    }
                    series_list.push(series);
                }
            }
            return Some(series_list);
        }
    }
    None
}
