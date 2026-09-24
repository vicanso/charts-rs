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

//! Validation of the JSON chart options before they are parsed.
//!
//! Every `from_json` first runs its input through [`validate`] with the
//! field table of its chart type (plus the shared [`BASE_FIELDS`]). A
//! misspelt key, a value of the wrong type, an unknown enum value or an
//! out-of-range size is reported as [`Error::Params`] instead of being
//! silently ignored, which used to leave the caller with a default-looking
//! chart and no hint why.

use super::color::Color;
use super::error::{Error, Result};

/// Largest accepted split / tick count. Each tick costs a grid line and a
/// label, so a huge value would only produce a huge SVG.
const MAX_COUNT: u64 = 1000;
/// Largest accepted data index (`start_index`, palette indexes, …).
const MAX_INDEX: u64 = 1_000_000;

/// What a field's value must look like.
#[derive(Clone, Copy)]
pub(crate) enum Kind {
    /// Any JSON number.
    Number,
    /// A finite number greater than 0 (widths, heights, radii).
    Size,
    /// A non-negative integer.
    Uint,
    /// A non-negative integer up to [`MAX_COUNT`].
    Count,
    /// A non-negative integer up to [`MAX_INDEX`].
    Index,
    Bool,
    String,
    /// A string accepted by [`Color::parse`].
    Color,
    /// One of the listed values, matched case-insensitively.
    Enum(&'static [&'static str]),
    /// A margin: a number applied to all sides, or `{left, top, right, bottom}`.
    Margin,
    /// An axis scale: `"linear"`, `"log"`, `"log2"`, `"log10"` or
    /// `{"type": "log", "base": n}`.
    Scale,
    /// Any JSON array.
    Array,
    /// An array of numbers; `null` marks a missing point.
    NumberArray,
    /// An array of strings.
    StringArray,
    /// An array of colors; `null` keeps the default.
    ColorArray,
    /// An object with the given fields.
    Object(&'static [Field]),
    /// An array of objects with the given fields.
    ArrayOf(&'static [Field]),
    /// A mark value: a number or `average` / `min` / `max`.
    MarkValue,
    /// An array of objects with the same fields as the enclosing object
    /// (tree children).
    SelfArray,
}

/// One accepted key and the shape of its value.
#[derive(Clone, Copy)]
pub(crate) struct Field {
    pub name: &'static str,
    pub kind: Kind,
}

const fn f(name: &'static str, kind: Kind) -> Field {
    Field { name, kind }
}

const ALIGN: &[&str] = &["left", "center", "right"];
const POSITION: &[&str] = &["left", "right", "top", "bottom", "inside"];

pub(crate) static MARGIN_FIELDS: &[Field] = &[
    f("left", Kind::Number),
    f("top", Kind::Number),
    f("right", Kind::Number),
    f("bottom", Kind::Number),
];

static SYMBOL_FIELDS: &[Field] = &[
    f(
        "type",
        Kind::Enum(&["circle", "rect", "square", "triangle", "diamond"]),
    ),
    f("size", Kind::Number),
    f("radius", Kind::Number),
    f("color", Kind::Color),
];

static MARK_LINE_FIELDS: &[Field] = &[
    f("category", Kind::Enum(&["average", "min", "max", "value"])),
    f("value", Kind::Number),
];
static MARK_POINT_FIELDS: &[Field] = &[f("category", Kind::Enum(&["min", "max"]))];
static MARK_AREA_FIELDS: &[Field] = &[f("from", Kind::MarkValue), f("to", Kind::MarkValue)];

pub(crate) static SERIES_FIELDS: &[Field] = &[
    f("name", Kind::String),
    f("data", Kind::NumberArray),
    f("index", Kind::Index),
    f("y_axis_index", Kind::Index),
    f("label_show", Kind::Bool),
    f("category", Kind::Enum(&["line", "bar"])),
    f("start_index", Kind::Index),
    f("mark_lines", Kind::ArrayOf(MARK_LINE_FIELDS)),
    f("mark_points", Kind::ArrayOf(MARK_POINT_FIELDS)),
    f("mark_areas", Kind::ArrayOf(MARK_AREA_FIELDS)),
    f("colors", Kind::ColorArray),
    f("stroke_dash_array", Kind::String),
    f("stack", Kind::String),
    f("smooth", Kind::Bool),
    f("fill", Kind::Bool),
    f("symbol", Kind::Object(SYMBOL_FIELDS)),
];

pub(crate) static Y_AXIS_FIELDS: &[Field] = &[
    f("axis_font_size", Kind::Number),
    f("axis_font_color", Kind::Color),
    f("axis_font_weight", Kind::String),
    f("axis_stroke_color", Kind::Color),
    f("axis_width", Kind::Number),
    f("axis_split_number", Kind::Count),
    f("axis_name_gap", Kind::Number),
    f("axis_formatter", Kind::String),
    f("axis_margin", Kind::Margin),
    f("axis_min", Kind::Number),
    f("axis_max", Kind::Number),
    f("axis_scale", Kind::Scale),
];

static ANIMATION_FIELDS: &[Field] = &[
    f("duration", Kind::Uint),
    f("easing", Kind::String),
    f("delay", Kind::Uint),
];

/// The fields every chart shares (`ChartBase`).
pub(crate) static BASE_FIELDS: &[Field] = &[
    f("theme", Kind::String),
    f("width", Kind::Size),
    f("height", Kind::Size),
    f("x", Kind::Number),
    f("y", Kind::Number),
    f("margin", Kind::Margin),
    f("font_family", Kind::String),
    f("title_text", Kind::String),
    f("title_font_size", Kind::Number),
    f("title_font_color", Kind::Color),
    f("title_font_weight", Kind::String),
    f("title_margin", Kind::Margin),
    f("title_align", Kind::Enum(ALIGN)),
    f("title_height", Kind::Number),
    f("sub_title_text", Kind::String),
    f("sub_title_font_size", Kind::Number),
    f("sub_title_font_color", Kind::Color),
    f("sub_title_font_weight", Kind::String),
    f("sub_title_margin", Kind::Margin),
    f("sub_title_align", Kind::Enum(ALIGN)),
    f("sub_title_height", Kind::Number),
    f("legend_font_size", Kind::Number),
    f("legend_font_color", Kind::Color),
    f("legend_font_weight", Kind::String),
    f("legend_align", Kind::Enum(ALIGN)),
    f("legend_margin", Kind::Margin),
    f(
        "legend_category",
        Kind::Enum(&["normal", "rect", "round_rect", "circle"]),
    ),
    f("legend_show", Kind::Bool),
    f("empty_text", Kind::String),
    f(
        "legend_position",
        Kind::Enum(&["top", "bottom", "left", "right"]),
    ),
    f("x_axis_data", Kind::StringArray),
    f("x_axis_height", Kind::Number),
    f("x_axis_stroke_color", Kind::Color),
    f("x_axis_font_size", Kind::Number),
    f("x_axis_font_color", Kind::Color),
    f("x_axis_font_weight", Kind::String),
    f("x_axis_name_gap", Kind::Number),
    f("x_axis_name_rotate", Kind::Number),
    f("x_axis_margin", Kind::Margin),
    f("x_boundary_gap", Kind::Bool),
    f(
        "x_axis_label_overflow",
        Kind::Enum(&["thin", "rotate", "ellipsis"]),
    ),
    f("x_axis_hidden", Kind::Bool),
    f("y_axis_hidden", Kind::Bool),
    f("y_axis_configs", Kind::ArrayOf(Y_AXIS_FIELDS)),
    f("grid_stroke_color", Kind::Color),
    f("grid_stroke_width", Kind::Number),
    f("series_stroke_width", Kind::Number),
    f("series_label_font_color", Kind::Color),
    f("series_label_font_size", Kind::Number),
    f("series_label_font_weight", Kind::String),
    f("series_label_formatter", Kind::String),
    f("series_colors", Kind::ColorArray),
    f("series_symbol", Kind::Object(SYMBOL_FIELDS)),
    f("series_smooth", Kind::Bool),
    f("series_fill", Kind::Bool),
    f("animation", Kind::Object(ANIMATION_FIELDS)),
    f("tooltip_show", Kind::Bool),
    f("series_list", Kind::ArrayOf(SERIES_FIELDS)),
];

// Per-chart fields, in addition to `BASE_FIELDS`.

pub(crate) static BAR_FIELDS: &[Field] = &[f("radius", Kind::Number)];

static BOX_SERIES_FIELDS: &[Field] = &[
    f("name", Kind::String),
    f("index", Kind::Index),
    f("data", Kind::Array),
];
pub(crate) static BOX_PLOT_FIELDS: &[Field] = &[f("box_series", Kind::ArrayOf(BOX_SERIES_FIELDS))];

pub(crate) static CALENDAR_FIELDS: &[Field] = &[
    f("start_date", Kind::String),
    f("end_date", Kind::String),
    f("min", Kind::Number),
    f("max", Kind::Number),
    f("min_color", Kind::Color),
    f("max_color", Kind::Color),
    f("empty_color", Kind::Color),
    f("cell_size", Kind::Number),
    f("cell_gap", Kind::Number),
    f("month_label_height", Kind::Number),
    f("week_label_width", Kind::Number),
    f("show_dow_labels", Kind::Array),
    f("data", Kind::Array),
];

pub(crate) static CANDLESTICK_FIELDS: &[Field] = &[
    f("candlestick_up_color", Kind::Color),
    f("candlestick_up_border_color", Kind::Color),
    f("candlestick_down_color", Kind::Color),
    f("candlestick_down_border_color", Kind::Color),
];

pub(crate) static FUNNEL_FIELDS: &[Field] = &[
    f("funnel_gap", Kind::Number),
    f("min_width", Kind::Number),
    f("sort_ascending", Kind::Bool),
    f(
        "series_label_position",
        Kind::Enum(&["inside", "left", "right"]),
    ),
    f("funnel_align", Kind::Enum(ALIGN)),
];

pub(crate) static GAUGE_FIELDS: &[Field] = &[
    f("min", Kind::Number),
    f("max", Kind::Number),
    f("start_angle", Kind::Number),
    f("sweep_angle", Kind::Number),
    f("radius", Kind::Number),
    f("arc_width", Kind::Number),
    f("background_arc_color", Kind::Color),
    f("show_pointer", Kind::Bool),
    f("pointer_color", Kind::Color),
    f("show_axis_label", Kind::Bool),
    f("split_number", Kind::Count),
    f("value_formatter", Kind::String),
];

static GRAPH_NODE_FIELDS: &[Field] = &[
    f("name", Kind::String),
    f("value", Kind::Number),
    f("color", Kind::Color),
    f("category", Kind::Index),
];
static LINK_FIELDS: &[Field] = &[
    f("source", Kind::String),
    f("target", Kind::String),
    f("value", Kind::Number),
];
pub(crate) static GRAPH_FIELDS: &[Field] = &[
    f("nodes", Kind::ArrayOf(GRAPH_NODE_FIELDS)),
    f("links", Kind::ArrayOf(LINK_FIELDS)),
    f("symbol_size", Kind::Number),
    f("layout", Kind::Enum(&["force", "circular"])),
    f("categories", Kind::StringArray),
];

static HEATMAP_SERIES_FIELDS: &[Field] = &[
    f("min", Kind::Number),
    f("max", Kind::Number),
    f("min_color", Kind::Color),
    f("max_color", Kind::Color),
    f("min_font_color", Kind::Color),
    f("max_font_color", Kind::Color),
    f("data", Kind::Array),
];
pub(crate) static HEATMAP_FIELDS: &[Field] = &[
    f("y_axis_data", Kind::StringArray),
    f("series", Kind::Object(HEATMAP_SERIES_FIELDS)),
];

pub(crate) static HORIZONTAL_BAR_FIELDS: &[Field] =
    &[f("series_label_position", Kind::Enum(POSITION))];

pub(crate) static PIE_FIELDS: &[Field] = &[
    f("radius", Kind::Number),
    f("inner_radius", Kind::Number),
    f("rose_type", Kind::Bool),
    f("border_radius", Kind::Number),
    f("start_angle", Kind::Number),
    f("series_label_position", Kind::Enum(&["inside", "outside"])),
    f("min_show_label_angle", Kind::Number),
];

static INDICATOR_FIELDS: &[Field] = &[f("name", Kind::String), f("max", Kind::Number)];
pub(crate) static RADAR_FIELDS: &[Field] = &[
    f("indicators", Kind::ArrayOf(INDICATOR_FIELDS)),
    f("split_number", Kind::Count),
];

static SANKEY_NODE_FIELDS: &[Field] = &[f("name", Kind::String), f("color", Kind::Color)];
pub(crate) static SANKEY_FIELDS: &[Field] = &[
    f("nodes", Kind::ArrayOf(SANKEY_NODE_FIELDS)),
    f("links", Kind::ArrayOf(LINK_FIELDS)),
    f("node_width", Kind::Number),
    f("node_gap", Kind::Number),
    f("link_opacity", Kind::Number),
    f("node_align", Kind::Enum(&["left", "right", "justify"])),
    f("link_gradient", Kind::Bool),
];

pub(crate) static SCATTER_FIELDS: &[Field] = &[
    f("series_symbol_sizes", Kind::NumberArray),
    f("series_symbols", Kind::Array),
    f("x_axis_config", Kind::Object(Y_AXIS_FIELDS)),
];

static TREE_NODE_FIELDS: &[Field] = &[
    f("name", Kind::String),
    f("value", Kind::Number),
    f("color", Kind::Color),
    f("children", Kind::SelfArray),
];
pub(crate) static SUNBURST_FIELDS: &[Field] = &[
    f("series_data", Kind::ArrayOf(TREE_NODE_FIELDS)),
    f("radius", Kind::Number),
    f("inner_radius", Kind::Number),
    f("start_angle", Kind::Number),
    f("level_thickness", Kind::NumberArray),
];

static CELL_STYLE_FIELDS: &[Field] = &[
    f("indexes", Kind::Array),
    f("font_color", Kind::Color),
    f("font_weight", Kind::String),
    f("background_color", Kind::Color),
];
/// The table chart does not embed `ChartBase`, so this is its full field set.
pub(crate) static TABLE_FIELDS: &[Field] = &[
    f("theme", Kind::String),
    f("width", Kind::Size),
    f("height", Kind::Size),
    f("x", Kind::Number),
    f("y", Kind::Number),
    f("font_family", Kind::String),
    f("background_color", Kind::Color),
    f("title_text", Kind::String),
    f("title_font_size", Kind::Number),
    f("title_font_color", Kind::Color),
    f("title_font_weight", Kind::String),
    f("title_margin", Kind::Margin),
    f("title_align", Kind::Enum(ALIGN)),
    f("title_height", Kind::Number),
    f("sub_title_text", Kind::String),
    f("sub_title_font_size", Kind::Number),
    f("sub_title_font_color", Kind::Color),
    f("sub_title_font_weight", Kind::String),
    f("sub_title_margin", Kind::Margin),
    f("sub_title_align", Kind::Enum(ALIGN)),
    f("sub_title_height", Kind::Number),
    f("data", Kind::Array),
    f("spans", Kind::NumberArray),
    f("text_aligns", Kind::StringArray),
    f("border_color", Kind::Color),
    f("header_row_padding", Kind::Margin),
    f("header_row_height", Kind::Number),
    f("header_font_size", Kind::Number),
    f("header_font_color", Kind::Color),
    f("header_font_weight", Kind::String),
    f("header_background_color", Kind::Color),
    f("body_row_padding", Kind::Margin),
    f("body_row_height", Kind::Number),
    f("body_font_size", Kind::Number),
    f("body_font_color", Kind::Color),
    f("body_font_weight", Kind::String),
    f("body_background_colors", Kind::ColorArray),
    f("outlined", Kind::Bool),
    f("cell_styles", Kind::ArrayOf(CELL_STYLE_FIELDS)),
    f("font_color", Kind::Color),
    f("font_weight", Kind::String),
    f("indexes", Kind::Array),
];

pub(crate) static THEME_RIVER_FIELDS: &[Field] = &[f("stream_opacity", Kind::Number)];

pub(crate) static TREE_FIELDS: &[Field] = &[
    f("series_data", Kind::ArrayOf(TREE_NODE_FIELDS)),
    f("orient", Kind::Enum(&["LR", "TB"])),
    f("symbol_size", Kind::Number),
];

pub(crate) static TREEMAP_FIELDS: &[Field] = &[
    f("item_gap", Kind::Number),
    f("series_data", Kind::ArrayOf(TREE_NODE_FIELDS)),
];

pub(crate) static WATERFALL_FIELDS: &[Field] = &[
    f("label_show", Kind::Bool),
    f("connector_line_show", Kind::Bool),
    f("bar_width_ratio", Kind::Number),
    f("increase_color", Kind::Color),
    f("decrease_color", Kind::Color),
    f("total_color", Kind::Color),
    f("data", Kind::Array),
];

pub(crate) static MULTI_FIELDS: &[Field] = &[
    f("theme", Kind::String),
    f("margin", Kind::Margin),
    f("gap", Kind::Number),
    f("background_color", Kind::Color),
    f("child_charts", Kind::Array),
];

/// Fields a multi chart child carries on top of its own chart's fields.
pub(crate) static CHILD_CHART_FIELDS: &[Field] = &[
    f("type", Kind::String),
    f("x", Kind::Number),
    f("y", Kind::Number),
];

/// Keys that tools around the library put next to the chart options, such
/// as the web editor's chart `type` and image `quality`. They are accepted
/// (and ignored) everywhere so such documents can be passed through as is.
static ENVELOPE_FIELDS: &[Field] = &[f("type", Kind::String), f("quality", Kind::Number)];

/// Checks a chart's JSON options against the given field tables: unknown
/// keys, wrongly typed values, unknown enum values and out-of-range sizes
/// are errors. `null` is accepted for any known key and means "not set".
pub(crate) fn validate(value: &serde_json::Value, tables: &[&[Field]]) -> Result<()> {
    let mut tables = tables.to_vec();
    tables.push(ENVELOPE_FIELDS);
    validate_object(value, &tables, "")
}

fn params(message: String) -> Error {
    Error::Params { message }
}

fn lookup<'a>(tables: &[&'a [Field]], key: &str) -> Option<&'a Field> {
    tables.iter().flat_map(|t| t.iter()).find(|f| f.name == key)
}

fn validate_object(value: &serde_json::Value, tables: &[&[Field]], path: &str) -> Result<()> {
    let Some(obj) = value.as_object() else {
        return Err(params(format!(
            "{} must be a JSON object",
            if path.is_empty() {
                "the chart options"
            } else {
                path
            }
        )));
    };
    for (key, v) in obj {
        let Some(field) = lookup(tables, key) else {
            let name = join(path, key);
            let hint = suggest(tables, key)
                .map(|s| format!(", did you mean `{}`?", join(path, s)))
                .unwrap_or_default();
            return Err(params(format!("unknown field `{name}`{hint}")));
        };
        if v.is_null() {
            continue;
        }
        let name = join(path, key);
        if let Kind::SelfArray = field.kind {
            validate_array_of(v, tables, &name)?;
        } else {
            validate_value(v, field.kind, &name)?;
        }
    }
    Ok(())
}

fn validate_array_of(v: &serde_json::Value, tables: &[&[Field]], name: &str) -> Result<()> {
    let Some(items) = v.as_array() else {
        return Err(params(format!(
            "field `{name}` must be an array of objects"
        )));
    };
    for (i, item) in items.iter().enumerate() {
        validate_object(item, tables, &format!("{name}[{i}]"))?;
    }
    Ok(())
}

fn join(path: &str, key: &str) -> String {
    if path.is_empty() {
        key.to_string()
    } else {
        format!("{path}.{key}")
    }
}

fn validate_value(v: &serde_json::Value, kind: Kind, name: &str) -> Result<()> {
    let expect = |what: &str| Err(params(format!("field `{name}` must be {what}")));
    match kind {
        Kind::Number if v.is_number() => Ok(()),
        Kind::Number => expect("a number"),
        Kind::Size => match v.as_f64() {
            Some(n) if n.is_finite() && n > 0.0 => Ok(()),
            _ => expect("a positive number"),
        },
        Kind::Uint if v.as_u64().is_some() => Ok(()),
        Kind::Uint => expect("a non-negative integer"),
        Kind::Count => match v.as_u64() {
            Some(n) if n <= MAX_COUNT => Ok(()),
            _ => expect(&format!("an integer between 0 and {MAX_COUNT}")),
        },
        Kind::Index => match v.as_u64() {
            Some(n) if n <= MAX_INDEX => Ok(()),
            _ => expect(&format!("an integer between 0 and {MAX_INDEX}")),
        },
        Kind::Bool if v.is_boolean() => Ok(()),
        Kind::Bool => expect("true or false"),
        Kind::String if v.is_string() => Ok(()),
        Kind::String => expect("a string"),
        Kind::Color => match v.as_str().and_then(Color::parse) {
            Some(_) => Ok(()),
            None => expect("a color such as \"#345\", \"#ffcc00\" or \"rgba(255,204,0,0.5)\""),
        },
        Kind::Enum(values) => match v.as_str() {
            Some(s) if values.iter().any(|x| x.eq_ignore_ascii_case(s)) => Ok(()),
            _ => expect(&format!("one of {}", values.join(", "))),
        },
        Kind::Margin => {
            if v.is_number() {
                Ok(())
            } else if v.is_object() {
                validate_object(v, &[MARGIN_FIELDS], name)
            } else {
                expect("a number or {left, top, right, bottom}")
            }
        }
        Kind::Scale => {
            const SCALES: &[&str] = &["linear", "log", "log10", "log2"];
            match v {
                serde_json::Value::String(s)
                    if SCALES.iter().any(|x| x.eq_ignore_ascii_case(s)) =>
                {
                    Ok(())
                }
                serde_json::Value::Object(_) => validate_object(
                    v,
                    &[&[
                        f("type", Kind::Enum(&["log", "linear"])),
                        f("base", Kind::Size),
                    ]],
                    name,
                ),
                _ => expect("\"linear\", \"log\", \"log2\", \"log10\" or {type, base}"),
            }
        }
        Kind::MarkValue => match v {
            serde_json::Value::Number(_) => Ok(()),
            serde_json::Value::String(s)
                if ["average", "min", "max"]
                    .iter()
                    .any(|x| x.eq_ignore_ascii_case(s)) =>
            {
                Ok(())
            }
            _ => expect("a number or one of average, min, max"),
        },
        Kind::Array if v.is_array() => Ok(()),
        Kind::Array => expect("an array"),
        Kind::NumberArray => match v.as_array() {
            Some(items) if items.iter().all(|i| i.is_number() || i.is_null()) => Ok(()),
            _ => expect("an array of numbers (null for a missing value)"),
        },
        Kind::StringArray => match v.as_array() {
            Some(items) if items.iter().all(|i| i.is_string()) => Ok(()),
            _ => expect("an array of strings"),
        },
        Kind::ColorArray => match v.as_array() {
            Some(items)
                if items
                    .iter()
                    .all(|i| i.is_null() || i.as_str().and_then(Color::parse).is_some()) =>
            {
                Ok(())
            }
            _ => expect("an array of colors (null keeps the default)"),
        },
        Kind::Object(fields) => validate_object(v, &[fields], name),
        Kind::ArrayOf(fields) => validate_array_of(v, &[fields], name),
        // Resolved by `validate_object`, which knows the enclosing tables.
        Kind::SelfArray => Ok(()),
    }
}

/// The known key closest to `key`, when it is close enough to be a typo.
fn suggest<'a>(tables: &[&'a [Field]], key: &str) -> Option<&'a str> {
    let limit = (key.len() / 3).clamp(1, 3);
    tables
        .iter()
        .flat_map(|t| t.iter())
        .map(|f| (edit_distance(f.name, key), f.name))
        .filter(|(d, _)| *d <= limit)
        .min_by_key(|(d, _)| *d)
        .map(|(_, name)| name)
}

fn edit_distance(a: &str, b: &str) -> usize {
    let a: Vec<char> = a.chars().collect();
    let b: Vec<char> = b.chars().collect();
    let mut prev: Vec<usize> = (0..=b.len()).collect();
    let mut cur = vec![0; b.len() + 1];
    for (i, ca) in a.iter().enumerate() {
        cur[0] = i + 1;
        for (j, cb) in b.iter().enumerate() {
            let cost = usize::from(ca != cb);
            cur[j + 1] = (prev[j + 1] + 1).min(cur[j] + 1).min(prev[j] + cost);
        }
        std::mem::swap(&mut prev, &mut cur);
    }
    prev[b.len()]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn check(json: &str, tables: &[&[Field]]) -> std::result::Result<(), String> {
        let value: serde_json::Value = serde_json::from_str(json).unwrap();
        validate(&value, tables).map_err(|e| e.to_string())
    }

    #[test]
    fn unknown_key_with_suggestion() {
        let err = check(r#"{"tittle_text": "x"}"#, &[BASE_FIELDS]).unwrap_err();
        assert!(err.contains("unknown field `tittle_text`"), "{err}");
        assert!(err.contains("did you mean `title_text`"), "{err}");

        let err = check(
            r#"{"series_list": [{"name": "a", "lable_show": true}]}"#,
            &[BASE_FIELDS],
        )
        .unwrap_err();
        assert!(err.contains("`series_list[0].lable_show`"), "{err}");
        assert!(err.contains("`series_list[0].label_show`"), "{err}");

        let err = check(r#"{"zzz": 1}"#, &[BASE_FIELDS]).unwrap_err();
        assert!(!err.contains("did you mean"), "{err}");
    }

    #[test]
    fn value_kinds() {
        assert!(
            check(r#"{"width": "600"}"#, &[BASE_FIELDS])
                .unwrap_err()
                .contains("positive number")
        );
        assert!(check(r#"{"width": 0}"#, &[BASE_FIELDS]).is_err());
        assert!(check(r#"{"width": -1}"#, &[BASE_FIELDS]).is_err());
        assert!(check(r#"{"width": null}"#, &[BASE_FIELDS]).is_ok());
        assert!(
            check(r#"{"title_align": "lefft"}"#, &[BASE_FIELDS])
                .unwrap_err()
                .contains("left, center, right")
        );
        assert!(check(r#"{"title_align": "LEFT"}"#, &[BASE_FIELDS]).is_ok());
        assert!(
            check(r#"{"title_font_color": "red"}"#, &[BASE_FIELDS])
                .unwrap_err()
                .contains("color")
        );
        assert!(check(r##"{"series_colors": ["#f00", 5]}"##, &[BASE_FIELDS]).is_err());
        assert!(check(r##"{"series_colors": ["#f00", null]}"##, &[BASE_FIELDS]).is_ok());
        assert!(
            check(
                r#"{"y_axis_configs": [{"axis_split_number": 2000000}]}"#,
                &[BASE_FIELDS]
            )
            .is_err()
        );
        assert!(check(r#"{"y_axis_configs": [{"axis_scale": "log2"}, {"axis_scale": {"type": "log", "base": 3}}]}"#, &[BASE_FIELDS]).is_ok());
        assert!(
            check(
                r#"{"y_axis_configs": [{"axis_scale": "sqrt"}]}"#,
                &[BASE_FIELDS]
            )
            .is_err()
        );
        assert!(
            check(
                r#"{"margin": 10, "title_margin": {"top": 5}}"#,
                &[BASE_FIELDS]
            )
            .is_ok()
        );
        assert!(check(r#"{"margin": "10"}"#, &[BASE_FIELDS]).is_err());
        assert!(
            check(
                r#"{"series_list": [{"name": "a", "data": [1, null, "x"]}]}"#,
                &[BASE_FIELDS]
            )
            .is_err()
        );
        assert!(
            check(
                r#"{"series_list": [{"name": "a", "start_index": 4000000000000000000}]}"#,
                &[BASE_FIELDS]
            )
            .is_err()
        );
        assert!(
            check(r#"[1]"#, &[BASE_FIELDS])
                .unwrap_err()
                .contains("JSON object")
        );
        // Chart specific fields extend the base ones.
        assert!(check(r#"{"radius": 3}"#, &[BASE_FIELDS]).is_err());
        assert!(check(r#"{"radius": 3}"#, &[BASE_FIELDS, BAR_FIELDS]).is_ok());
        // Recursive trees.
        assert!(check(
            r#"{"series_data": [{"name": "r", "children": [{"name": "c", "children": [{"name": "g", "valeu": 1}]}]}]}"#,
            &[BASE_FIELDS, TREE_FIELDS]
        )
        .unwrap_err()
        .contains("series_data[0].children[0].children[0].valeu"));
    }

    #[test]
    fn distance() {
        assert_eq!(0, edit_distance("abc", "abc"));
        assert_eq!(1, edit_distance("title_text", "tittle_text"));
        assert_eq!(3, edit_distance("kitten", "sitting"));
    }
}
