use super::{Align, Box, Color, Series, SeriesCategory};

pub(crate) fn get_bool_from_value(value: &serde_json::Value, key: &str) -> Option<bool> {
    if let Some(value) = value.get(key) {
        if let Some(b) = value.as_bool() {
            return Some(b);
        }
    }
    None
}

pub(crate) fn get_usize_from_value(value: &serde_json::Value, key: &str) -> Option<usize> {
    if let Some(value) = value.get(key) {
        if let Some(u) = value.as_u64() {
            return Some(u as usize);
        }
    }
    None
}

pub(crate) fn get_f32_from_value(value: &serde_json::Value, key: &str) -> Option<f32> {
    if let Some(width) = value.get(key) {
        if let Some(v) = width.as_f64() {
            return Some(v as f32);
        }
    }
    None
}
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

pub(crate) fn get_align_from_value(value: &serde_json::Value, key: &str) -> Option<Align> {
    if let Some(value) = value.get(key) {
        if let Some(value) = value.as_str() {
            let value = match value.to_lowercase().as_str() {
                "left" => Align::Left,
                "right" => Align::Right,
                _ => Align::Center,
            };
            return Some(value);
        }
    }
    None
}

pub(crate) fn get_margin_from_value(value: &serde_json::Value, key: &str) -> Option<Box> {
    if let Some(data) = value.get(key) {
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
pub(crate) fn get_string_from_value(value: &serde_json::Value, key: &str) -> Option<String> {
    if let Some(s) = value.get(key) {
        if let Some(v) = s.as_str() {
            return Some(v.to_string());
        }
    }
    None
}

pub(crate) fn get_color_from_value(value: &serde_json::Value, key: &str) -> Option<Color> {
    if let Some(s) = get_string_from_value(value, key) {
        return Some(s.as_str().into());
    }
    None
}

fn get_series_category_from_value(value: &serde_json::Value, key: &str) -> Option<SeriesCategory> {
    if let Some(value) = value.get(key) {
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

fn get_series_from_value(value: &serde_json::Value) -> Option<Series> {
    let name = get_string_from_value(value, "name").unwrap_or_default();
    if name.is_empty() {
        return None;
    }
    let data = get_f32_slice_from_value(value, "data").unwrap_or_default();
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
    })
}

pub(crate) fn get_series_list_from_value(value: &serde_json::Value) -> Option<Vec<Series>> {
    if let Some(data) = value.get("series_list") {
        if let Some(arr) = data.as_array() {
            let mut series_list = vec![];
            for item in arr.iter() {
                if let Some(series) = get_series_from_value(item) {
                    series_list.push(series);
                }
            }
            return Some(series_list);
        }
    }
    None
}
