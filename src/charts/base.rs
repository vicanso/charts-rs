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

//! The shared chart state and rendering helpers. Every chart struct embeds a
//! [`ChartBase`] (exposed through `Deref`/`DerefMut`, so `chart.title_text`
//! keeps working) instead of repeating these fields, and the theme filling,
//! JSON option parsing and common render passes live here as ordinary
//! methods — compiled once, lint-covered and debuggable, replacing the old
//! `#[derive(Chart)]` proc-macro expansion.

use std::collections::BTreeMap;
use std::sync::Arc;

use serde::{Deserialize, Serialize};

use super::canvas;
use super::canvas::Canvas;
use super::color::*;
use super::common::*;
use super::component::*;
use super::font::text_ellipsis;
use super::measure_text_width_family;
use super::params::*;
use super::schema;
use super::theme::{DEFAULT_Y_AXIS_WIDTH, Theme, get_theme, list_theme_name};
use super::util::*;
use super::x_axis::ContinuousX;

/// The options shared by every chart type: canvas size and position, the data
/// series, font/background, the title, sub-title and legend blocks, the x/y
/// axis and grid configuration, and the series styling defaults. Charts expose
/// these fields directly through `Deref`, e.g. `chart.title_text = ...`.
#[derive(Serialize, Deserialize, Clone, PartialEq, Debug, Default)]
pub struct ChartBase {
    /// Canvas width.
    pub width: f32,
    /// Canvas height.
    pub height: f32,
    /// Canvas x offset (used when composed inside a `MultiChart`).
    pub x: f32,
    /// Canvas y offset (used when composed inside a `MultiChart`).
    pub y: f32,
    /// Margin around the whole chart.
    pub margin: Box,
    /// The data series of the chart.
    pub series_list: Vec<Series>,
    /// Font family used for every text; must be registered with `add_fonts`
    /// (or be the embedded default) to be measured correctly. An unknown
    /// family is still written to the SVG for the viewer to resolve, but the
    /// layout measures text with the default font.
    pub font_family: String,
    /// Background color of the chart.
    pub background_color: Color,
    /// Whether the current theme is a light theme.
    pub is_light: bool,

    /// Title text.
    pub title_text: String,
    /// Title font size.
    pub title_font_size: f32,
    /// Title font color.
    pub title_font_color: Color,
    /// Title font weight, e.g. `"bold"`.
    pub title_font_weight: Option<String>,
    /// Margin around the title block.
    pub title_margin: Option<Box>,
    /// Horizontal alignment of the title.
    pub title_align: Align,
    /// Height reserved for the title row.
    pub title_height: f32,

    /// Sub-title text.
    pub sub_title_text: String,
    /// Sub-title font size.
    pub sub_title_font_size: f32,
    /// Sub-title font color.
    pub sub_title_font_color: Color,
    /// Sub-title font weight, e.g. `"bold"`.
    pub sub_title_font_weight: Option<String>,
    /// Margin around the sub-title block.
    pub sub_title_margin: Option<Box>,
    /// Horizontal alignment of the sub-title.
    pub sub_title_align: Align,
    /// Height reserved for the sub-title row.
    pub sub_title_height: f32,

    /// Legend font size.
    pub legend_font_size: f32,
    /// Legend font color.
    pub legend_font_color: Color,
    /// Legend font weight, e.g. `"bold"`.
    pub legend_font_weight: Option<String>,
    /// Horizontal alignment of the legend.
    pub legend_align: Align,
    /// Margin around the legend block.
    pub legend_margin: Option<Box>,
    /// Legend marker shape (normal, rect or round rect).
    pub legend_category: LegendCategory,
    /// Shows or hides the legend; `None` follows the chart's default.
    pub legend_show: Option<bool>,
    /// Where the legend goes: top (the default, beside the title), bottom,
    /// or stacked vertically on the left / right of the plot.
    #[serde(default)]
    pub legend_position: Option<Position>,
    /// Text shown in the middle of the plot when there is no data to draw
    /// (every series is empty); nothing is shown when unset.
    #[serde(default)]
    pub empty_text: Option<String>,
    /// Emit compact SVG: no whitespace, relative path data, merged grid
    /// lines, shared attributes hoisted, defaults dropped. Renders the same
    /// and is typically 20–30% smaller; see [`compact_svg`](crate::compact_svg).
    #[serde(default)]
    pub compact: bool,

    /// Labels of the x axis.
    pub x_axis_data: Vec<String>,
    /// Height reserved for the x axis block.
    pub x_axis_height: f32,
    /// Stroke color of the x axis line and ticks.
    pub x_axis_stroke_color: Color,
    /// X axis label font size.
    pub x_axis_font_size: f32,
    /// X axis label font color.
    pub x_axis_font_color: Color,
    /// X axis label font weight, e.g. `"bold"`.
    pub x_axis_font_weight: Option<String>,
    /// Gap between the axis line and its labels.
    pub x_axis_name_gap: f32,
    /// Rotation of the x axis labels, in radians (e.g. `0.785` for 45°).
    pub x_axis_name_rotate: f32,
    /// Margin around the x axis block.
    pub x_axis_margin: Option<Box>,
    /// Whether a gap is left on both ends of the x axis (bar-style) or the
    /// first/last points sit on the edges (line-style).
    pub x_boundary_gap: Option<bool>,
    /// What to do with x axis labels that do not fit side by side: thin
    /// them out (default), rotate them, or cut them with an ellipsis.
    #[serde(default)]
    pub x_axis_label_overflow: AxisLabelOverflow,
    /// How continuous x values read: plain numbers, or timestamps (`Time`).
    /// The axis is continuous whenever x values are given; see
    /// `x_axis_values`.
    #[serde(default)]
    pub x_axis_type: AxisType,
    /// The x value of each data point, shared by every series (a series may
    /// carry its own in `Series::x_values`). Setting them turns the x axis
    /// of line and bar charts from evenly spaced categories into a
    /// continuous scale, so unevenly sampled data keeps its real spacing.
    /// Timestamps are seconds since the unix epoch; JSON also takes date
    /// strings such as `"2024-01-05"` or `"2024-01-05 08:30"`.
    #[serde(default)]
    pub x_axis_values: Vec<f64>,
    /// Fixed start of a continuous x axis; `None` starts at the first value.
    #[serde(default)]
    pub x_axis_min: Option<f64>,
    /// Fixed end of a continuous x axis; `None` ends at the last value.
    #[serde(default)]
    pub x_axis_max: Option<f64>,
    /// Format of the x axis labels: `{c}` is the label; on a time axis a
    /// `strftime`-like pattern (`%Y %y %m %d %H %M %S %b`) replaces the
    /// automatic one.
    #[serde(default)]
    pub x_axis_formatter: Option<String>,
    /// Minutes east of UTC that a time axis is displayed in (480 for
    /// UTC+8). Timestamps are shown in UTC by default.
    #[serde(default)]
    pub x_axis_time_offset: i32,
    /// Title of the x axis, written below its labels.
    #[serde(default)]
    pub x_axis_title: String,
    /// Hides the x axis entirely (charts without an x axis ignore this).
    pub x_axis_hidden: bool,
    /// Hides the y axis entirely (charts without a y axis ignore this).
    pub y_axis_hidden: bool,

    /// Stroke color of the grid lines.
    pub grid_stroke_color: Color,
    /// Stroke width of the grid lines.
    pub grid_stroke_width: f32,

    /// Stroke width of series lines.
    pub series_stroke_width: f32,
    /// Series label font color.
    pub series_label_font_color: Color,
    /// Series label font size.
    pub series_label_font_size: f32,
    /// Series label font weight, e.g. `"bold"`.
    pub series_label_font_weight: Option<String>,
    /// Series label format, supporting `{c}` value, `{a}` series name,
    /// `{b}` category, `{d}` percentage and `{t}` thousands.
    pub series_label_formatter: String,
    /// Drops a data label that would overlap one already drawn, instead of
    /// printing them on top of each other.
    #[serde(default)]
    pub series_label_hide_overlap: bool,
    /// Shows the series of a stack as their shares of it: every stack adds
    /// up to 100% (bar, horizontal bar, line and polar bar charts).
    #[serde(default)]
    pub stack_percent: bool,
    /// Color palette cycled through by the series.
    pub series_colors: Vec<Color>,
    /// Marker drawn on data points (circle, dot or none).
    pub series_symbol: Option<Symbol>,
    /// Draws line series as smooth curves.
    pub series_smooth: bool,
    /// Fills the area under line series.
    pub series_fill: bool,

    /// SVG animation (duration/easing/stagger delay) for the chart types that
    /// support it (bar, horizontal bar, line, pie, funnel, sunburst, treemap,
    /// sankey, histogram, polar bar, chord).
    pub animation: Option<AnimationConfig>,
    /// When `true`, data shapes get a hover tooltip (`series: value`): a
    /// CSS-revealed label that works in any browser, plus a native `<title>`
    /// for accessibility. Not available in calendar, gauge, parallel, radar
    /// and theme river charts. Default: false; output is unchanged when off.
    pub tooltip_show: bool,
}

/// Gets y axis config by index.
/// Resolves a mark line / area edge to a value of the series: fixed values
/// as given, statistics from `values` (`None` when there are no points).
pub(crate) fn mark_statistics(values: &[f32]) -> impl Fn(&MarkLineCategory) -> Option<f32> + '_ {
    let (mut sum, mut min, mut max) = (0.0_f32, f32::MAX, f32::MIN);
    for &v in values.iter() {
        sum += v;
        max = max.max(v);
        min = min.min(v);
    }
    move |category| match category {
        MarkLineCategory::Value(v) => Some(*v),
        _ if values.is_empty() => None,
        MarkLineCategory::Average => Some(sum / values.len() as f32),
        MarkLineCategory::Max => Some(max),
        MarkLineCategory::Min => Some(min),
    }
}

/// Rejects a theme name that is not registered: a typo would otherwise
/// silently render with the light theme.
pub(crate) fn check_theme_name(theme: &str) -> canvas::Result<()> {
    if theme.is_empty() || list_theme_name().iter().any(|t| t == theme) {
        return Ok(());
    }
    Err(canvas::Error::Params {
        message: format!(
            "unknown theme `{theme}`; registered themes: {}",
            list_theme_name().join(", ")
        ),
    })
}

/// Builds the axis parameters from an axis config, so every chart applies
/// `axis_min` / `axis_max`, the split count, the scale and thousands
/// formatting the same way.
pub(crate) fn axis_value_params(
    config: &YAxisConfig,
    data_list: Vec<f32>,
    reverse: bool,
) -> AxisValueParams {
    let thousands_format = config
        .axis_formatter
        .as_deref()
        .is_some_and(|f| f.contains(THOUSANDS_FORMAT_LABEL));
    AxisValueParams {
        data_list,
        split_number: config.axis_split_number,
        reverse: Some(reverse),
        min: config.axis_min,
        max: config.axis_max,
        thousands_format,
        scale: config.axis_scale.clone(),
        inverse: config.axis_inverse,
    }
}

pub(crate) fn get_y_axis_config(y_axis_configs: &[YAxisConfig], index: usize) -> YAxisConfig {
    let size = y_axis_configs.len();
    if size == 0 {
        YAxisConfig::default()
    } else if index < size {
        y_axis_configs[index].clone()
    } else {
        y_axis_configs[0].clone()
    }
}

/// The frame of a cartesian chart after the header, axes and grid have been
/// drawn: what the series renderers need to place their marks.
pub(crate) struct CartesianLayout {
    /// The canvas below the header; series canvases are children of it.
    pub canvas: Canvas,
    /// Value range of the left y axis.
    pub left: AxisValues,
    /// Value range of the right y axis (default when no series uses it).
    pub right: AxisValues,
    /// Width taken by the left y axis (0 when hidden or absent).
    pub left_width: f32,
    /// Width taken by the right y axis (0 when hidden or absent).
    pub right_width: f32,
    /// Height of the plot area (the y axis).
    pub axis_height: f32,
    /// Width of the plot area between the y axes.
    pub axis_width: f32,
    /// Height available to the series: the canvas minus the x axis.
    pub max_height: f32,
    /// The scale of a continuous x axis; `None` on a category axis.
    pub x: Option<ContinuousX>,
    /// Number of slots along the x axis (categories, or x values).
    pub x_count: usize,
}

/// How a chart's series sit on the x axis.
#[derive(Clone, Copy, PartialEq, Debug)]
pub(crate) enum XAxisMode {
    /// Always evenly spaced categories.
    Category,
    /// Points: a continuous axis (when x values are given) spans exactly
    /// the values.
    Points,
    /// Bars: a continuous axis leaves half a band at both ends.
    Bands,
    /// A continuous number axis over exactly `min..max` with ticks every
    /// `tick` from `min` (a histogram's bin edges), whatever x values the
    /// chart carries.
    Range {
        /// Value at the left edge of the plot.
        min: f64,
        /// Value at the right edge of the plot.
        max: f64,
        /// Distance between two possible ticks.
        tick: f64,
    },
}

/// Space taken along the edges of a chart by its axis titles.
pub(crate) struct AxisTitles {
    /// The canvas before the space was taken; the titles are drawn on it.
    outer: Canvas,
    bottom: f32,
    left: f32,
    right: f32,
}

/// The boxes of the data labels drawn so far, to drop a label that would
/// overlap one of them (`series_label_hide_overlap`). When disabled every
/// label is accepted and nothing is recorded.
pub(crate) struct LabelBoxes {
    enabled: bool,
    boxes: Vec<(f32, f32, f32, f32)>,
}

impl LabelBoxes {
    pub fn new(enabled: bool) -> Self {
        LabelBoxes {
            enabled,
            boxes: vec![],
        }
    }
    /// Records the box and returns true, unless it overlaps a recorded one.
    pub fn try_place(&mut self, left: f32, top: f32, width: f32, height: f32) -> bool {
        if !self.enabled {
            return true;
        }
        let (right, bottom) = (left + width, top + height);
        // Labels that merely touch read as one number; keep a small gap.
        const GAP: f32 = 2.0;
        let overlaps = self
            .boxes
            .iter()
            .any(|&(l, t, r, b)| left - GAP < r && l < right + GAP && top < b && t < bottom);
        if !overlaps {
            self.boxes.push((left, top, right, bottom));
        }
        !overlaps
    }
}

/// Alpha of the band around a line (`Series::band`): light enough for the
/// line, the grid and other bands to show through.
const BAND_ALPHA: u8 = 60;

/// Hover target of a band point that has no line point of its own.
struct BandTip {
    series: usize,
    slot: usize,
    x: f32,
    top: f32,
    bottom: f32,
    lower: f32,
    upper: f32,
}

/// Gap between an axis title and the axis labels.
const AXIS_TITLE_GAP: f32 = 6.0;

/// Longest stack whose shares are worked out for `stack_percent`: a series
/// placed further along than this (by a huge `start_index`) is off any
/// chart anyway.
const MAX_STACK_LENGTH: usize = 1_000_000;

impl CartesianLayout {
    /// Canvas of the plot area between the y axes.
    pub fn plot(&self) -> Canvas {
        self.canvas.child(Box {
            left: self.left_width,
            right: self.right_width,
            ..Default::default()
        })
    }
    /// The value ranges indexed by `Series::y_axis_index`.
    pub fn y_axis_values(&self) -> [&AxisValues; 2] {
        [&self.left, &self.right]
    }
}

impl ChartBase {
    /// Fills the default options of current theme.
    ///
    /// `y_axis_configs` stays a per-chart field (see the module doc), so it is
    /// threaded in by reference; callers pass disjoint fields of the same
    /// chart, e.g. `c.base.fill_theme(theme, &mut c.y_axis_configs)`.
    pub(crate) fn fill_theme(&mut self, t: Arc<Theme>, y_axis_configs: &mut Vec<YAxisConfig>) {
        self.font_family = t.font_family.clone();
        self.margin = t.margin;
        self.width = t.width;
        self.height = t.height;
        self.background_color = t.background_color;
        self.is_light = t.is_light;

        self.title_font_color = t.title_font_color;
        self.title_font_size = t.title_font_size;
        self.title_font_weight = t.title_font_weight.clone();
        self.title_margin = t.title_margin;
        self.title_align = t.title_align.clone();
        self.title_height = t.title_height;

        self.sub_title_font_color = t.sub_title_font_color;
        self.sub_title_font_size = t.sub_title_font_size;
        self.sub_title_margin = t.sub_title_margin;
        self.sub_title_align = t.sub_title_align.clone();
        self.sub_title_height = t.sub_title_height;

        self.legend_font_color = t.legend_font_color;
        self.legend_font_size = t.legend_font_size;
        self.legend_align = t.legend_align.clone();
        self.legend_margin = t.legend_margin;

        self.x_axis_font_size = t.x_axis_font_size;
        self.x_axis_font_color = t.x_axis_font_color;
        self.x_axis_stroke_color = t.x_axis_stroke_color;
        self.x_axis_name_gap = t.x_axis_name_gap;
        self.x_axis_height = t.x_axis_height;

        *y_axis_configs = vec![YAxisConfig {
            axis_font_size: t.y_axis_font_size,
            axis_font_color: t.y_axis_font_color,
            axis_stroke_color: t.y_axis_stroke_color,
            axis_split_number: t.y_axis_split_number,
            axis_name_gap: t.y_axis_name_gap,
            ..Default::default()
        }];

        self.grid_stroke_color = t.grid_stroke_color;
        self.grid_stroke_width = t.grid_stroke_width;

        self.series_colors = t.series_colors.clone();
        self.series_label_font_color = t.series_label_font_color;
        self.series_label_font_size = t.series_label_font_size;
        self.series_stroke_width = t.series_stroke_width;

        self.series_symbol = Some(Symbol::Circle(
            self.series_stroke_width,
            Some(self.background_color),
        ));
    }
    /// Fills the options from json config; `y_axis_configs` is threaded in the
    /// same way as for [`ChartBase::fill_theme`].
    pub(crate) fn fill_option(
        &mut self,
        data: &str,
        y_axis_configs: &mut Vec<YAxisConfig>,
        chart_fields: &[schema::Field],
    ) -> canvas::Result<serde_json::Value> {
        let data: serde_json::Value = serde_json::from_str(data)?;
        // Reject typos, wrong types and out-of-range values up front; the
        // getters below can then stay lenient.
        schema::validate(&data, &[schema::BASE_FIELDS, chart_fields])?;
        let series_list = get_series_list_from_value(&data).unwrap_or_default();
        let theme = get_string_from_value(&data, "theme").unwrap_or_default();
        check_theme_name(&theme)?;
        let theme = get_theme(&theme);
        self.fill_theme(theme.clone(), y_axis_configs);
        self.series_list = series_list;

        if let Some(width) = get_f32_from_value(&data, "width") {
            self.width = width;
        }
        if let Some(height) = get_f32_from_value(&data, "height") {
            self.height = height;
        }
        if let Some(x) = get_f32_from_value(&data, "x") {
            self.x = x;
        }
        if let Some(y) = get_f32_from_value(&data, "y") {
            self.y = y;
        }
        if let Some(margin) = get_margin_from_value(&data, "margin") {
            self.margin = margin;
        }
        if let Some(font_family) = get_string_from_value(&data, "font_family") {
            self.font_family = font_family;
        }
        if let Some(title_text) = get_string_from_value(&data, "title_text") {
            self.title_text = title_text;
        }
        if let Some(title_font_size) = get_f32_from_value(&data, "title_font_size") {
            self.title_font_size = title_font_size;
        }
        if let Some(title_font_color) = get_color_from_value(&data, "title_font_color") {
            self.title_font_color = title_font_color;
        }
        if let Some(title_font_weight) = get_string_from_value(&data, "title_font_weight") {
            self.title_font_weight = Some(title_font_weight);
        }
        if let Some(title_margin) = get_margin_from_value(&data, "title_margin") {
            self.title_margin = Some(title_margin);
        }
        if let Some(title_align) = get_align_from_value(&data, "title_align") {
            self.title_align = title_align;
        }
        if let Some(title_height) = get_f32_from_value(&data, "title_height") {
            self.title_height = title_height;
        }

        if let Some(sub_title_text) = get_string_from_value(&data, "sub_title_text") {
            self.sub_title_text = sub_title_text;
        }
        if let Some(sub_title_font_size) = get_f32_from_value(&data, "sub_title_font_size") {
            self.sub_title_font_size = sub_title_font_size;
        }
        if let Some(sub_title_font_color) = get_color_from_value(&data, "sub_title_font_color") {
            self.sub_title_font_color = sub_title_font_color;
        }
        if let Some(sub_title_font_weight) = get_string_from_value(&data, "sub_title_font_weight") {
            self.sub_title_font_weight = Some(sub_title_font_weight);
        }
        if let Some(sub_title_margin) = get_margin_from_value(&data, "sub_title_margin") {
            self.sub_title_margin = Some(sub_title_margin);
        }
        if let Some(sub_title_align) = get_align_from_value(&data, "sub_title_align") {
            self.sub_title_align = sub_title_align;
        }
        if let Some(sub_title_height) = get_f32_from_value(&data, "sub_title_height") {
            self.sub_title_height = sub_title_height;
        }

        if let Some(legend_font_size) = get_f32_from_value(&data, "legend_font_size") {
            self.legend_font_size = legend_font_size;
        }
        if let Some(legend_font_color) = get_color_from_value(&data, "legend_font_color") {
            self.legend_font_color = legend_font_color;
        }
        if let Some(legend_font_weight) = get_string_from_value(&data, "legend_font_weight") {
            self.legend_font_weight = Some(legend_font_weight);
        }
        if let Some(legend_align) = get_align_from_value(&data, "legend_align") {
            self.legend_align = legend_align;
        }
        if let Some(legend_margin) = get_margin_from_value(&data, "legend_margin") {
            self.legend_margin = Some(legend_margin);
        }
        if let Some(legend_category) = get_legend_category_from_value(&data, "legend_category") {
            self.legend_category = legend_category;
        }
        if let Some(legend_show) = get_bool_from_value(&data, "legend_show") {
            self.legend_show = Some(legend_show);
        }
        if let Some(legend_position) = get_position_from_value(&data, "legend_position") {
            self.legend_position = Some(legend_position);
        }
        if let Some(empty_text) = get_string_from_value(&data, "empty_text") {
            self.empty_text = Some(empty_text);
        }
        if let Some(compact) = get_bool_from_value(&data, "compact") {
            self.compact = compact;
        }

        if let Some(x_axis_data) = get_string_slice_from_value(&data, "x_axis_data") {
            self.x_axis_data = x_axis_data;
        }
        if let Some(x_axis_height) = get_f32_from_value(&data, "x_axis_height") {
            self.x_axis_height = x_axis_height;
        }
        if let Some(x_axis_stroke_color) = get_color_from_value(&data, "x_axis_stroke_color") {
            self.x_axis_stroke_color = x_axis_stroke_color;
        }
        if let Some(x_axis_font_size) = get_f32_from_value(&data, "x_axis_font_size") {
            self.x_axis_font_size = x_axis_font_size;
        }
        if let Some(x_axis_font_color) = get_color_from_value(&data, "x_axis_font_color") {
            self.x_axis_font_color = x_axis_font_color;
        }
        if let Some(x_axis_font_weight) = get_string_from_value(&data, "x_axis_font_weight") {
            self.x_axis_font_weight = Some(x_axis_font_weight);
        }
        if let Some(x_axis_name_gap) = get_f32_from_value(&data, "x_axis_name_gap") {
            self.x_axis_name_gap = x_axis_name_gap;
        }
        if let Some(x_axis_name_rotate) = get_f32_from_value(&data, "x_axis_name_rotate") {
            self.x_axis_name_rotate = x_axis_name_rotate;
        }
        if let Some(x_axis_margin) = get_margin_from_value(&data, "x_axis_margin") {
            self.x_axis_margin = Some(x_axis_margin);
        }
        if let Some(x_boundary_gap) = get_bool_from_value(&data, "x_boundary_gap") {
            self.x_boundary_gap = Some(x_boundary_gap);
        }
        if let Some(overflow) = get_string_from_value(&data, "x_axis_label_overflow") {
            self.x_axis_label_overflow = match overflow.to_lowercase().as_str() {
                "rotate" => AxisLabelOverflow::Rotate,
                "ellipsis" => AxisLabelOverflow::Ellipsis,
                _ => AxisLabelOverflow::Thin,
            };
        }
        if let Some(kind) = get_string_from_value(&data, "x_axis_type") {
            self.x_axis_type = match kind.to_lowercase().as_str() {
                "value" => AxisType::Value,
                "time" => AxisType::Time,
                _ => AxisType::Category,
            };
        }
        // Date strings make the axis a time axis unless it says otherwise.
        let mut has_time_strings = false;
        if let Some((values, has_time)) = get_x_values_from_value(&data, "x_axis_values") {
            self.x_axis_values = values;
            has_time_strings |= has_time;
        }
        if let Some(list) = data.get("series_list").and_then(|v| v.as_array()) {
            has_time_strings |= list.iter().any(|item| {
                get_x_values_from_value(item, "x_values").is_some_and(|(_, has_time)| has_time)
            });
        }
        if let Some((value, has_time)) = data.get("x_axis_min").and_then(get_x_value) {
            self.x_axis_min = Some(value);
            has_time_strings |= has_time;
        }
        if let Some((value, has_time)) = data.get("x_axis_max").and_then(get_x_value) {
            self.x_axis_max = Some(value);
            has_time_strings |= has_time;
        }
        if has_time_strings && self.x_axis_type == AxisType::Category {
            self.x_axis_type = AxisType::Time;
        }
        if let Some(formatter) = get_string_from_value(&data, "x_axis_formatter") {
            self.x_axis_formatter = Some(formatter);
        }
        if let Some(offset) = get_f32_from_value(&data, "x_axis_time_offset") {
            self.x_axis_time_offset = offset as i32;
        }
        if let Some(title) = get_string_from_value(&data, "x_axis_title") {
            self.x_axis_title = title;
        }
        if let Some(x_axis_hidden) = get_bool_from_value(&data, "x_axis_hidden") {
            self.x_axis_hidden = x_axis_hidden;
        }
        if let Some(y_axis_hidden) = get_bool_from_value(&data, "y_axis_hidden") {
            self.y_axis_hidden = y_axis_hidden;
        }

        if let Some(value) = get_y_axis_configs_from_value(theme.clone(), &data, "y_axis_configs") {
            *y_axis_configs = value;
        }

        if let Some(grid_stroke_color) = get_color_from_value(&data, "grid_stroke_color") {
            self.grid_stroke_color = grid_stroke_color;
        }
        if let Some(grid_stroke_width) = get_f32_from_value(&data, "grid_stroke_width") {
            self.grid_stroke_width = grid_stroke_width;
        }

        if let Some(series_stroke_width) = get_f32_from_value(&data, "series_stroke_width") {
            self.series_stroke_width = series_stroke_width;
        }
        if let Some(series_label_font_color) =
            get_color_from_value(&data, "series_label_font_color")
        {
            self.series_label_font_color = series_label_font_color;
        }
        if let Some(series_label_font_size) = get_f32_from_value(&data, "series_label_font_size") {
            self.series_label_font_size = series_label_font_size;
        }
        if let Some(series_label_font_weight) =
            get_string_from_value(&data, "series_label_font_weight")
        {
            self.series_label_font_weight = Some(series_label_font_weight);
        }
        if let Some(series_label_formatter) = get_string_from_value(&data, "series_label_formatter")
        {
            self.series_label_formatter = series_label_formatter;
        }
        if let Some(hide) = get_bool_from_value(&data, "series_label_hide_overlap") {
            self.series_label_hide_overlap = hide;
        }
        if let Some(stack_percent) = get_bool_from_value(&data, "stack_percent") {
            self.stack_percent = stack_percent;
        }

        if let Some(series_colors) = get_color_slice_from_value(&data, "series_colors") {
            self.series_colors = series_colors;
        }
        if let Some(series_symbol) = get_series_symbol_from_value(&data, "series_symbol") {
            self.series_symbol = Some(series_symbol);
        }
        if let Some(series_smooth) = get_bool_from_value(&data, "series_smooth") {
            self.series_smooth = series_smooth;
        }
        if let Some(series_fill) = get_bool_from_value(&data, "series_fill") {
            self.series_fill = series_fill;
        }

        if let Some(anim) = data.get("animation")
            && !anim.is_null()
        {
            let mut config = AnimationConfig::default();
            if let Some(d) = get_usize_from_value(anim, "duration") {
                config.duration = d as u32;
            }
            if let Some(e) = get_string_from_value(anim, "easing") {
                config.easing = e;
            }
            if let Some(d) = get_usize_from_value(anim, "delay") {
                config.delay = d as u32;
            }
            self.animation = Some(config);
        }
        if let Some(v) = get_bool_from_value(&data, "tooltip_show") {
            self.tooltip_show = v;
        }

        Ok(data)
    }
    /// Gets y axis values by index.
    pub(crate) fn get_y_axis_values(
        &self,
        y_axis_configs: &[YAxisConfig],
        y_axis_index: usize,
    ) -> (AxisValues, f32) {
        let y_axis_config = get_y_axis_config(y_axis_configs, y_axis_index);
        let mut data_list = vec![];
        // Non-stacked series: include individual values directly.
        for series in self.series_list.iter() {
            if series.y_axis_index == y_axis_index && series.stack.is_none() {
                data_list.extend(series.iter_values());
            }
        }
        // The bounds of a band have to fit on the axis as well as the line.
        for series in self.series_list.iter() {
            if series.y_axis_index == y_axis_index
                && let Some(band) = &series.band
            {
                data_list.extend(band.values());
            }
        }
        // Stacked series: the effective max at each x-position is the sum of all
        // series in the same stack group, so collect per-x sums per stack key.
        let mut stack_keys: Vec<String> = vec![];
        for series in self.series_list.iter() {
            if series.y_axis_index == y_axis_index
                && let Some(ref s) = series.stack
                && !stack_keys.contains(s)
            {
                stack_keys.push(s.clone());
            }
        }
        for stack_key in &stack_keys {
            // Keyed by x position rather than a dense Vec: a huge `start_index`
            // from JSON must not allocate a vector that long. Positive and
            // negative values stack separately (each side grows away from 0),
            // so the axis has to cover both partial sums.
            let mut sums: BTreeMap<usize, (f32, f32)> = BTreeMap::new();
            for series in self.series_list.iter() {
                if series.y_axis_index == y_axis_index
                    && series.stack.as_deref() == Some(stack_key.as_str())
                {
                    for (i, v) in series.iter_values().enumerate() {
                        if v == NIL_VALUE {
                            continue;
                        }
                        let actual_i = i.saturating_add(series.start_index);
                        let entry = sums.entry(actual_i).or_insert((0.0, 0.0));
                        if v >= 0.0 {
                            entry.0 += v;
                        } else {
                            entry.1 += v;
                        }
                    }
                }
            }
            for (pos, neg) in sums.into_values() {
                data_list.push(pos);
                if neg < 0.0 {
                    data_list.push(neg);
                }
            }
        }
        if data_list.is_empty() {
            return (AxisValues::default(), 0.0);
        }
        let y_axis_values = get_axis_values(axis_value_params(&y_axis_config, data_list, true));
        let y_axis_width = self.y_axis_width_for(&y_axis_config, &y_axis_values);
        (y_axis_values, y_axis_width)
    }
    /// The width a y axis needs for its labels: the configured `axis_width`,
    /// or the longest (formatted) label plus a small gap.
    pub(crate) fn y_axis_width_for(&self, config: &YAxisConfig, values: &AxisValues) -> f32 {
        if let Some(value) = config.axis_width {
            return value;
        }
        let y_axis_formatter = &config.axis_formatter.clone().unwrap_or_default();
        let mut longest_item: &str = "";
        for item in &values.data {
            if item.chars().count() > longest_item.chars().count() {
                longest_item = item
            }
        }
        let value = format_string(longest_item, y_axis_formatter);
        if let Ok(b) = measure_text_width_family(&self.font_family, config.axis_font_size, &value) {
            b.width() + 5.0
        } else {
            DEFAULT_Y_AXIS_WIDTH
        }
    }
    /// Renders background for canvas.
    pub(crate) fn render_background(&self, c: Canvas) {
        if self.background_color.is_transparent() {
            return;
        }
        let mut c1 = c;
        c1.rect(Rect {
            fill: Some(self.background_color.into()),
            left: 0.0,
            top: 0.0,
            width: self.width,
            height: self.height,
            ..Default::default()
        });
    }
    /// Renders background, title and legend, returning the top offset
    /// for the plotting area (the greater of the title and legend heights).
    pub(crate) fn render_header(&self, c: &mut Canvas) -> f32 {
        self.render_background(c.child(Box::default()));
        c.margin = self.margin;

        let title_height = self.render_title(c.child(Box::default()));

        match self.legend_position {
            Some(Position::Bottom) => {
                // Reserve the legend rows at the bottom; the plot sits above.
                let legend_height = self.legend_height(c.width());
                if legend_height > 0.0 {
                    self.render_legend(c.child(Box {
                        top: c.height() - legend_height,
                        ..Default::default()
                    }));
                    c.margin.bottom += legend_height;
                }
                title_height
            }
            Some(Position::Left) | Some(Position::Right) => {
                let right = self.legend_position == Some(Position::Right);
                let width = self.render_legend_vertical(
                    c.child(Box {
                        top: title_height,
                        ..Default::default()
                    }),
                    right,
                );
                if right {
                    c.margin.right += width;
                } else {
                    c.margin.left += width;
                }
                title_height
            }
            _ => {
                let legend_height = self.render_legend(c.child(Box::default()));
                // get the max height of title and legend
                legend_height.max(title_height)
            }
        }
    }
    /// Height the horizontal legend takes in a canvas `width` wide, without
    /// drawing it.
    fn legend_height(&self, width: f32) -> f32 {
        let entries = self.legend_entries();
        let legends: Vec<&str> = entries.iter().map(|(name, _)| *name).collect();
        if legends.is_empty() {
            return 0.0;
        }
        let legend_margin = self.legend_margin.unwrap_or_default();
        let rows = wrap_legends_to_rows(
            &self.font_family,
            self.legend_font_size,
            &legends,
            width - legend_margin.left - legend_margin.right,
        );
        (self.legend_font_size + LEGEND_MARGIN) * rows.len() as f32
            + legend_margin.top
            + legend_margin.bottom
    }
    /// Draws the legend entries stacked vertically along the left (or right)
    /// edge and returns the width they take, margins included.
    fn render_legend_vertical(&self, c: Canvas, right: bool) -> f32 {
        let entries = self.legend_entries();
        let legends: Vec<&str> = entries.iter().map(|(name, _)| *name).collect();
        if legends.is_empty() {
            return 0.0;
        }
        let widths = measure_legend_widths(&self.font_family, self.legend_font_size, &legends);
        let max_width = widths.iter().copied().fold(0.0_f32, f32::max);
        let legend_margin = self.legend_margin.unwrap_or_default();
        let margin_width = legend_margin.left + legend_margin.right;
        let mut legend_canvas = c.child(legend_margin);
        let left = if right {
            legend_canvas.width() - max_width
        } else {
            0.0
        };
        let unit_height = self.legend_font_size + LEGEND_MARGIN;
        let mut top = 0.0;
        for (name, color) in entries.iter() {
            if name.is_empty() {
                continue;
            }
            let color = *color;
            let fill = if self.is_light {
                Some(self.background_color)
            } else {
                Some(color)
            };
            legend_canvas.legend(Legend {
                text: name.to_string(),
                font_size: self.legend_font_size,
                font_family: self.font_family.clone(),
                font_color: Some(self.legend_font_color),
                font_weight: self.legend_font_weight.clone(),
                stroke_color: Some(color),
                fill,
                left,
                top,
                category: self.legend_category.clone(),
            });
            top += unit_height;
        }
        max_width + margin_width + LEGEND_MARGIN
    }
    /// Render title widget for canvas.
    pub(crate) fn render_title(&self, c: Canvas) -> f32 {
        let mut title_height = 0.0;

        if !self.title_text.is_empty() {
            let title_margin = self.title_margin.unwrap_or_default();
            // A title wider than the canvas is cut with an ellipsis rather
            // than pushed to a negative x and clipped.
            let title_text = text_ellipsis(
                &self.font_family,
                self.title_font_size,
                &self.title_text,
                c.width() - title_margin.left - title_margin.right,
            );
            let mut x = 0.0;
            if let Ok(title_box) =
                measure_text_width_family(&self.font_family, self.title_font_size, &title_text)
            {
                x = match self.title_align {
                    Align::Center => (c.width() - title_box.width()) / 2.0,
                    Align::Right => c.width() - title_box.width(),
                    _ => 0.0,
                }
            }
            let title_margin_bottom = title_margin.bottom;
            let b = c.child(title_margin).text(Text {
                text: title_text,
                font_family: Some(self.font_family.clone()),
                font_size: Some(self.title_font_size),
                font_weight: self.title_font_weight.clone(),
                font_color: Some(self.title_font_color),
                line_height: Some(self.title_height),
                x: Some(x),
                ..Default::default()
            });
            title_height = b.outer_height() + title_margin_bottom;
        }
        if !self.sub_title_text.is_empty() {
            let mut sub_title_margin = self.sub_title_margin.unwrap_or_default();
            let sub_title_text = text_ellipsis(
                &self.font_family,
                self.sub_title_font_size,
                &self.sub_title_text,
                c.width() - sub_title_margin.left - sub_title_margin.right,
            );
            let mut x = 0.0;
            if let Ok(sub_title_box) = measure_text_width_family(
                &self.font_family,
                self.sub_title_font_size,
                &sub_title_text,
            ) {
                x = match self.sub_title_align {
                    Align::Center => (c.width() - sub_title_box.width()) / 2.0,
                    Align::Right => c.width() - sub_title_box.width(),
                    _ => 0.0,
                }
            }
            let sub_title_margin_bottom = sub_title_margin.bottom;
            sub_title_margin.top += self.title_height;
            let b = c.child(sub_title_margin).text(Text {
                text: sub_title_text,
                font_family: Some(self.font_family.clone()),
                font_size: Some(self.sub_title_font_size),
                font_color: Some(self.sub_title_font_color),
                line_height: Some(self.sub_title_height),
                font_weight: self.sub_title_font_weight.clone(),
                x: Some(x),
                ..Default::default()
            });
            title_height = b.outer_height() + sub_title_margin_bottom;
        }
        title_height
    }
    /// Renders legend widget for canvas.
    pub(crate) fn render_legend(&self, c: Canvas) -> f32 {
        let entries = self.legend_entries();
        self.render_legend_entries(c, &entries)
    }
    /// The legend entries of the series: name and color, in series order.
    fn legend_entries(&self) -> Vec<(&str, Color)> {
        if !self.legend_show.unwrap_or(true) {
            return vec![];
        }
        self.series_list
            .iter()
            .enumerate()
            .map(|(index, series)| {
                let color = get_color(&self.series_colors, series.index.unwrap_or(index));
                (series.name.as_str(), color)
            })
            .collect()
    }
    /// Draws the given legend entries in rows across the top of `c` (the
    /// series legend, or e.g. a graph's categories) and returns the height
    /// they take, margins included.
    pub(crate) fn render_legend_entries(&self, c: Canvas, entries: &[(&str, Color)]) -> f32 {
        if entries.is_empty() {
            return 0.0;
        }
        let legends: Vec<&str> = entries.iter().map(|(name, _)| *name).collect();
        let legend_margin = self.legend_margin.unwrap_or_default();
        let legend_margin_value = legend_margin.top + legend_margin.bottom;
        let mut legend_canvas = c.child(legend_margin);
        let legend_canvas_width = legend_canvas.width();
        let rows = wrap_legends_to_rows(
            &self.font_family,
            self.legend_font_size,
            &legends,
            legend_canvas_width,
        );
        let mut current_legend_index = 0;
        let legend_unit_height = self.legend_font_size + LEGEND_MARGIN;
        let mut legend_top = 0.0;
        let row_count = rows.len();
        for (row_index, (legend_width, legend_texts)) in rows.iter().enumerate() {
            let mut legend_left = match self.legend_align {
                Align::Right => legend_canvas_width - legend_width,
                Align::Left => 0.0,
                Align::Center => (legend_canvas_width - legend_width) / 2.0,
            };
            if legend_left < 0.0 {
                legend_left = 0.0;
            }
            for _text in legend_texts.iter() {
                let index = current_legend_index;
                current_legend_index += 1;
                let Some((name, color)) = entries.get(index) else {
                    continue;
                };
                if name.is_empty() {
                    continue;
                }
                let color = *color;
                let fill = if self.is_light {
                    Some(self.background_color)
                } else {
                    Some(color)
                };
                let b = legend_canvas.legend(Legend {
                    text: name.to_string(),
                    font_size: self.legend_font_size,
                    font_family: self.font_family.clone(),
                    font_color: Some(self.legend_font_color),
                    font_weight: self.legend_font_weight.clone(),
                    stroke_color: Some(color),
                    fill,
                    left: legend_left,
                    top: legend_top,
                    category: self.legend_category.clone(),
                });
                legend_left += b.width() + LEGEND_MARGIN;
            }
            // if not the last row, add the legend unit height
            if row_index < row_count - 1 {
                legend_top += legend_unit_height;
            }
        }
        legend_unit_height + legend_top + legend_margin_value
    }
    /// Draws the header, grid, y axes and x axis of a cartesian chart whose
    /// value ranges come from `series_list`, and returns the plot frame.
    pub(crate) fn layout_cartesian(
        &self,
        c: Canvas,
        y_axis_configs: &[YAxisConfig],
        x_mode: XAxisMode,
    ) -> CartesianLayout {
        let mut c = c;
        let axis_top = self.render_header(&mut c);
        let left = self.get_y_axis_values(y_axis_configs, 0);
        let right = if self.series_list.iter().any(|s| s.y_axis_index != 0) {
            Some(self.get_y_axis_values(y_axis_configs, 1))
        } else {
            None
        };
        self.layout_cartesian_with(c, y_axis_configs, axis_top, left, right, x_mode)
    }
    /// Takes the space the axis titles need off the edges of `c`: the x
    /// title below the x axis, the y titles beside the y axes. Call it once
    /// the header is drawn and before sizing the plot, then draw the titles
    /// with [`Self::render_axis_titles`] when the plot is known.
    pub(crate) fn reserve_axis_titles(
        &self,
        c: &mut Canvas,
        y_axis_configs: &[YAxisConfig],
        x_title: &str,
        right_axis: bool,
    ) -> AxisTitles {
        let outer = c.clone();
        let strip = |title: Option<&str>, font_size: f32| -> f32 {
            match title {
                Some(text) if !text.is_empty() => font_size + AXIS_TITLE_GAP,
                _ => 0.0,
            }
        };
        let bottom = if self.x_axis_hidden {
            0.0
        } else {
            strip(Some(x_title), self.x_axis_font_size)
        };
        let side = |index: usize| -> f32 {
            match y_axis_configs.get(index) {
                Some(config) if !self.y_axis_hidden => {
                    strip(config.axis_title.as_deref(), config.axis_font_size)
                }
                _ => 0.0,
            }
        };
        let left = side(0);
        let right = if right_axis { side(1) } else { 0.0 };
        c.margin.bottom += bottom;
        c.margin.left += left;
        c.margin.right += right;
        AxisTitles {
            outer,
            bottom,
            left,
            right,
        }
    }
    /// Draws the axis titles reserved by [`Self::reserve_axis_titles`],
    /// centred on the plot: `plot_left` / `plot_top` are its offset in the
    /// canvas the space was taken from, `plot_width` / `plot_height` its
    /// size.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn render_axis_titles(
        &self,
        titles: &AxisTitles,
        y_axis_configs: &[YAxisConfig],
        x_title: &str,
        plot_left: f32,
        plot_top: f32,
        plot_width: f32,
        plot_height: f32,
    ) {
        let mut outer = titles.outer.clone();
        if titles.bottom > 0.0 {
            outer.text_unmeasured(Text {
                text: x_title.to_string(),
                font_family: Some(self.font_family.clone()),
                font_size: Some(self.x_axis_font_size),
                font_color: Some(self.x_axis_font_color),
                font_weight: self.x_axis_font_weight.clone(),
                x: Some(titles.left + plot_left + plot_width / 2.0),
                // The baseline sits a descender above the bottom edge.
                y: Some(outer.height() - self.x_axis_font_size * 0.25),
                text_anchor: Some("middle".to_string()),
                ..Default::default()
            });
        }
        // The y titles read bottom-to-top on the left and top-to-bottom on
        // the right, centred on the plot's height. A rotated text is placed
        // by its transform alone, in absolute coordinates.
        let center_y = outer.margin.top + plot_top + plot_height / 2.0;
        for (index, strip, angle) in [(0, titles.left, -90), (1, titles.right, 90)] {
            let Some(config) = y_axis_configs.get(index) else {
                continue;
            };
            let Some(title) = config.axis_title.as_deref() else {
                continue;
            };
            if strip <= 0.0 {
                continue;
            }
            let inset = config.axis_font_size * 0.85;
            let x = if index == 0 {
                outer.margin.left + inset
            } else {
                outer.margin.left + outer.width() - inset
            };
            outer.append(Component::Text(Text {
                text: title.to_string(),
                font_family: Some(self.font_family.clone()),
                font_size: Some(config.axis_font_size),
                font_color: Some(config.axis_font_color),
                font_weight: config.axis_font_weight.clone(),
                transform: Some(format!(
                    "translate({},{}) rotate({angle})",
                    format_float(x),
                    format_float(center_y)
                )),
                text_anchor: Some("middle".to_string()),
                ..Default::default()
            }));
        }
    }
    /// [`Self::layout_cartesian`] with precomputed value ranges (and their
    /// axis widths), for charts whose data does not live in `series_list`.
    /// `axis_top` is the height the header took.
    pub(crate) fn layout_cartesian_with(
        &self,
        c: Canvas,
        y_axis_configs: &[YAxisConfig],
        axis_top: f32,
        left: (AxisValues, f32),
        right: Option<(AxisValues, f32)>,
        x_mode: XAxisMode,
    ) -> CartesianLayout {
        let mut c = c;
        let titles =
            self.reserve_axis_titles(&mut c, y_axis_configs, &self.x_axis_title, right.is_some());
        let (left_values, mut left_width) = left;
        let (right_values, mut right_width) = right.unwrap_or_default();
        // The value ranges are still needed to place the series when the
        // axes are hidden; only their space is dropped.
        if self.y_axis_hidden {
            left_width = 0.0;
            right_width = 0.0;
        }
        // A canvas too small for the header and the x axis has no plot
        // area left; clamp instead of emitting negative sizes.
        let axis_width = (c.width() - left_width - right_width).max(0.0);
        let x = match x_mode {
            XAxisMode::Category => None,
            XAxisMode::Points => self.continuous_x(axis_width, false),
            XAxisMode::Bands => self.continuous_x(axis_width, true),
            XAxisMode::Range { min, max, tick } => Some(ContinuousX {
                min,
                max,
                width: axis_width,
                band_width: 0.0,
                time: false,
                tick_step: Some(tick),
            }),
        };
        // The ticks of a continuous axis are chosen to fit, so its labels
        // never need the extra height rotated categories do.
        let x_axis_height = match x {
            Some(_) if self.x_axis_hidden => 0.0,
            Some(_) => self.x_axis_height,
            None => self.x_axis_height_for(c.width() - left_width - right_width),
        };
        let axis_height = (c.height() - x_axis_height - axis_top).max(0.0);
        self.render_axis_titles(
            &titles,
            y_axis_configs,
            &self.x_axis_title,
            left_width,
            axis_top,
            axis_width,
            axis_height,
        );
        // minus the height of top text area
        if axis_top > 0.0 {
            c = c.child(Box {
                top: axis_top,
                ..Default::default()
            });
        }
        self.render_empty_text(c.child(Box {
            left: left_width,
            right: right_width,
            bottom: x_axis_height,
            ..Default::default()
        }));
        self.render_grid(
            c.child(Box {
                left: left_width,
                ..Default::default()
            }),
            y_axis_configs,
            axis_width,
            axis_height,
        );
        if left_width > 0.0 {
            self.render_y_axis(
                c.child(Box::default()),
                y_axis_configs,
                left_values.data.clone(),
                axis_height,
                left_width,
                0,
            );
        }
        if right_width > 0.0 {
            self.render_y_axis(
                c.child(Box {
                    left: c.width() - right_width,
                    ..Default::default()
                }),
                y_axis_configs,
                right_values.data.clone(),
                axis_height,
                right_width,
                1,
            );
        }
        if !self.x_axis_hidden {
            let axis_canvas = c.child(Box {
                top: c.height() - x_axis_height,
                left: left_width,
                right: right_width,
                ..Default::default()
            });
            match &x {
                Some(scale) => {
                    self.render_continuous_x_axis(axis_canvas, scale, axis_width, x_axis_height)
                }
                None => self.render_x_axis_with_height(
                    axis_canvas,
                    self.x_axis_data.clone(),
                    axis_width,
                    x_axis_height,
                ),
            }
        }
        let max_height = c.height() - x_axis_height;
        let x_count = if x.is_some() {
            self.x_count()
        } else {
            self.x_axis_data.len()
        };
        CartesianLayout {
            canvas: c,
            left: left_values,
            right: right_values,
            left_width,
            right_width,
            axis_height,
            axis_width,
            max_height,
            x,
            x_count,
        }
    }
    /// The canvas a chart draws on: its size and position, with the
    /// output options (`compact`) applied.
    pub(crate) fn new_canvas(&self) -> Canvas {
        let mut c = Canvas::new_width_xy(self.width, self.height, self.x, self.y);
        c.compact = self.compact;
        c
    }
    /// Turns the values of the stacked series into their shares of the
    /// stack, in percent, and the value axes into percent axes. This is
    /// what `stack_percent` draws: the chart with these values instead.
    pub(crate) fn apply_stack_percent(&mut self, y_axis_configs: &mut [YAxisConfig]) {
        self.stack_percent = false;
        // The stacks, as the charts tell them apart: by name and axis.
        let key = |series: &Series| {
            series
                .stack
                .as_ref()
                .map(|stack| (stack.clone(), series.y_axis_index))
        };
        let mut stacks: Vec<(String, usize)> = vec![];
        for series in self.series_list.iter() {
            if let Some(key) = key(series)
                && !stacks.contains(&key)
            {
                stacks.push(key);
            }
        }
        let mut has_negative = false;
        for stack in stacks.iter() {
            // What the values of the stack add up to at every category;
            // a negative value takes its share too, on the other side of 0.
            let mut totals: Vec<f32> = vec![];
            for series in self
                .series_list
                .iter()
                .filter(|s| key(s).as_ref() == Some(stack))
            {
                for (i, value) in series.iter_values().enumerate() {
                    let index = i.saturating_add(series.start_index);
                    if value == NIL_VALUE || index >= MAX_STACK_LENGTH {
                        continue;
                    }
                    if totals.len() <= index {
                        totals.resize(index + 1, 0.0);
                    }
                    totals[index] += value.abs();
                    has_negative |= value < 0.0;
                }
            }
            for series in self
                .series_list
                .iter_mut()
                .filter(|s| key(s).as_ref() == Some(stack))
            {
                let start_index = series.start_index;
                for (i, value) in series.data.iter_mut().enumerate() {
                    let total = totals.get(i.saturating_add(start_index)).copied();
                    let known = value.filter(|v| v.is_finite() && *v != NIL_VALUE);
                    if let (Some(v), Some(total)) = (known, total) {
                        *value = Some(if total > 0.0 { v / total * 100.0 } else { 0.0 });
                    }
                }
            }
        }
        if stacks.is_empty() {
            return;
        }
        // The axes of the stacks read in percent, up to 100, with ticks on
        // whole percents — unless they are set otherwise.
        for (index, config) in y_axis_configs.iter_mut().enumerate() {
            if !stacks.iter().any(|(_, axis)| *axis == index) {
                continue;
            }
            if config.axis_formatter.is_none() {
                config.axis_formatter = Some("{c}%".to_string());
            }
            if config.axis_max.is_none() && config.axis_min.is_none() && !has_negative {
                config.axis_max = Some(100.0);
                let split = config.axis_split_number;
                if split == 0 || 100 % split != 0 {
                    config.axis_split_number = 5;
                }
            }
        }
        if self.series_label_formatter.is_empty() {
            self.series_label_formatter = "{c}%".to_string();
        }
    }
    /// True when there is nothing to draw: no series, or only empty ones.
    pub(crate) fn has_no_data(&self) -> bool {
        self.series_list.iter().all(|s| {
            s.data.iter().all(Option::is_none)
                && s.band
                    .as_ref()
                    .is_none_or(|band| band.values().next().is_none())
        })
    }
    /// Draws `empty_text` centered in `c` when the chart has no data.
    pub(crate) fn render_empty_text(&self, c: Canvas) {
        let Some(text) = self.empty_text.as_deref() else {
            return;
        };
        if text.is_empty() || !self.has_no_data() {
            return;
        }
        let mut c = c;
        c.text(Text {
            text: text.to_string(),
            font_family: Some(self.font_family.clone()),
            font_size: Some(self.series_label_font_size),
            font_color: Some(self.series_label_font_color),
            x: Some(c.width() / 2.0),
            y: Some(c.height() / 2.0),
            text_anchor: Some("middle".to_string()),
            dominant_baseline: Some("central".to_string()),
            ..Default::default()
        });
    }
    /// The `data-*` attributes of one data point: its series, x category and
    /// value, for callers that make the SVG interactive.
    pub(crate) fn point_dataset(
        &self,
        series: &Series,
        index: usize,
        value: f32,
    ) -> Vec<(String, String)> {
        let mut dataset = vec![("series".to_string(), series.name.clone())];
        if let Some(category) = self.x_label(series, index) {
            dataset.push(("category".to_string(), category.into_owned()));
        }
        dataset.push(("value".to_string(), format_float(value)));
        dataset
    }
    /// Draws the mark lines (average / min / max / fixed value) of each
    /// series across the plot area, labelled above their right end.
    pub(crate) fn render_mark_line(
        &self,
        c: Canvas,
        series_list: &[&Series],
        y_axis_values_list: &[&AxisValues],
        max_height: f32,
    ) {
        let mut c = c;
        for (index, series) in series_list.iter().enumerate() {
            if series.mark_lines.is_empty() && series.mark_areas.is_empty() {
                continue;
            }
            let y_axis_values = if series.y_axis_index >= y_axis_values_list.len() {
                y_axis_values_list[0]
            } else {
                y_axis_values_list[series.y_axis_index]
            };
            let color = get_color(&self.series_colors, series.index.unwrap_or(index));
            let values: Vec<f32> = series.iter_values().filter(|v| *v != NIL_VALUE).collect();
            let stat = mark_statistics(&values);
            // Bands first, so the lines stay visible on top of them.
            for mark_area in series.mark_areas.iter() {
                let (Some(from), Some(to)) = (stat(&mark_area.from), stat(&mark_area.to)) else {
                    continue;
                };
                let y_from = y_axis_values.get_offset_height(from, max_height);
                let y_to = y_axis_values.get_offset_height(to, max_height);
                c.rect(Rect {
                    fill: Some(color.with_alpha(40).into()),
                    left: 0.0,
                    top: y_from.min(y_to),
                    width: c.width(),
                    height: (y_from - y_to).abs(),
                    ..Default::default()
                });
            }
            for mark_line in series.mark_lines.iter() {
                let Some(value) = stat(&mark_line.category) else {
                    continue;
                };
                let y = y_axis_values.get_offset_height(value, max_height);
                let arrow_width = 10.0;
                c.circle(Circle {
                    stroke_color: Some(color),
                    fill: Some(color),
                    cx: 3.0,
                    cy: y,
                    r: 3.5,
                    ..Default::default()
                });
                c.line(Line {
                    color: Some(color),
                    left: 8.0,
                    top: y,
                    right: c.width() - arrow_width,
                    bottom: y,
                    stroke_dash_array: Some("4,2".to_string()),
                    ..Default::default()
                });
                c.arrow(Arrow {
                    x: c.width() - arrow_width,
                    y,
                    stroke_color: color,
                    ..Arrow::default()
                });
                // Inside the plot area, so it is never clipped by the canvas.
                c.text(Text {
                    text: format_float(value),
                    font_family: Some(self.font_family.clone()),
                    font_size: Some(self.series_label_font_size),
                    font_color: Some(self.series_label_font_color),
                    x: Some(c.width() - arrow_width - 4.0),
                    y: Some(y),
                    dy: Some(-6.0),
                    text_anchor: Some("end".to_string()),
                    ..Default::default()
                });
            }
        }
    }
    /// Formats the label of one data point with `series_label_formatter`;
    /// `{b}` is the x axis category at `index`.
    pub(crate) fn format_series_label(&self, series: &Series, index: usize, value: f32) -> String {
        let category = self.x_label(series, index);
        format_series_label(
            &self.series_label_formatter,
            value,
            &series.name,
            category.as_deref().unwrap_or(""),
        )
    }
    /// Renders grid for canvas, the axis width is the right padding of grid canvas,
    /// and the axis height is the bottom padding of grid canvas.
    pub(crate) fn render_grid(
        &self,
        c: Canvas,
        y_axis_configs: &[YAxisConfig],
        axis_width: f32,
        axis_height: f32,
    ) {
        let mut c1 = c;
        let y_axis_config = get_y_axis_config(y_axis_configs, 0);
        let axis_split_number = y_axis_config.axis_split_number;
        c1.grid(Grid {
            right: axis_width,
            bottom: axis_height,
            color: Some(self.grid_stroke_color),
            stroke_width: self.grid_stroke_width,
            horizontals: axis_split_number,
            hidden_horizontals: vec![axis_split_number],
            ..Default::default()
        });
    }
    /// Renders y axis for canvas, if the axis index greater than zero means the right y axis.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn render_y_axis(
        &self,
        c: Canvas,
        y_axis_configs: &[YAxisConfig],
        data: Vec<String>,
        axis_height: f32,
        axis_width: f32,
        axis_index: usize,
    ) {
        let c1 = c;
        let y_axis_config = &get_y_axis_config(y_axis_configs, axis_index);
        let mut position = Position::Left;
        if axis_index > 0 {
            position = Position::Right;
        }
        let mut name_align = Align::Left;
        if let Some(value) = &y_axis_config.axis_name_align {
            name_align = value.clone();
        }
        let margin = y_axis_config.axis_margin.unwrap_or_default();
        c1.child(margin).axis(Axis {
            position,
            height: axis_height,
            width: axis_width,
            split_number: y_axis_config.axis_split_number,
            font_family: self.font_family.clone(),
            stroke_color: Some(y_axis_config.axis_stroke_color),
            name_align,
            name_gap: y_axis_config.axis_name_gap,
            font_color: Some(y_axis_config.axis_font_color),
            font_size: y_axis_config.axis_font_size,
            font_weight: y_axis_config.axis_font_weight.clone(),
            data,
            formatter: y_axis_config.axis_formatter.clone(),
            ..Default::default()
        });
    }
    /// Renders x axis widget for canvas, the x_boundary_gap parameter set to false,
    /// the align will be left.
    pub(crate) fn render_x_axis(&self, c: Canvas, data: Vec<String>, axis_width: f32) {
        self.render_x_axis_with_height(c, data, axis_width, self.x_axis_height)
    }
    /// The height the x axis needs for its labels: `x_axis_height`, or more
    /// when the labels are rotated to fit (`x_axis_label_overflow`).
    pub(crate) fn x_axis_height_for(&self, axis_width: f32) -> f32 {
        if self.x_axis_hidden {
            return 0.0;
        }
        if self.x_axis_label_overflow != AxisLabelOverflow::Rotate
            || self.x_axis_name_rotate != 0.0
            || self.x_axis_data.is_empty()
        {
            return self.x_axis_height;
        }
        let mut total = 0.0;
        let mut widest = 0.0_f32;
        for text in self.x_axis_data.iter() {
            if let Ok(b) = measure_text_width_family(&self.font_family, self.x_axis_font_size, text)
            {
                total += b.width();
                widest = widest.max(b.width());
            }
        }
        if total <= axis_width {
            return self.x_axis_height;
        }
        // Rotated 45°: the label's projection plus the tick and gap.
        let rotated = (widest + self.x_axis_font_size) * std::f32::consts::FRAC_1_SQRT_2;
        self.x_axis_height.max(rotated + self.x_axis_name_gap + 8.0)
    }
    /// [`Self::render_x_axis`] with an explicit axis height.
    pub(crate) fn render_x_axis_with_height(
        &self,
        c: Canvas,
        data: Vec<String>,
        axis_width: f32,
        x_axis_height: f32,
    ) {
        let c1 = c;

        let mut split_number = data.len();
        let name_align = if self.x_boundary_gap.unwrap_or(true) {
            Align::Center
        } else {
            split_number = split_number.saturating_sub(1);
            Align::Left
        };
        let margin = self.x_axis_margin.unwrap_or_default();
        c1.child(margin).axis(Axis {
            height: x_axis_height,
            width: axis_width,
            split_number,
            font_family: self.font_family.clone(),
            data,
            font_color: Some(self.x_axis_font_color),
            font_weight: self.x_axis_font_weight.clone(),
            stroke_color: Some(self.x_axis_stroke_color),
            font_size: self.x_axis_font_size,
            name_gap: self.x_axis_name_gap,
            name_rotate: self.x_axis_name_rotate,
            name_align,
            label_overflow: self.x_axis_label_overflow.clone(),
            ..Default::default()
        });
    }
    /// Renders series label widget for canvas.
    pub(crate) fn render_series_label(&self, c: Canvas, series_labels_list: Vec<Vec<SeriesLabel>>) {
        if series_labels_list.is_empty() {
            return;
        }
        let mut c1 = c;
        // Boxes of the labels drawn so far, when overlapping ones are hidden.
        let mut placed = LabelBoxes::new(self.series_label_hide_overlap);
        for series_labels in series_labels_list.iter() {
            for series_label in series_labels.iter() {
                let mut dx = None;
                let mut width = 0.0;
                if let Ok(value) = measure_text_width_family(
                    &self.font_family,
                    self.series_label_font_size,
                    &series_label.text,
                ) {
                    width = value.width();
                    dx = Some(-value.width() / 2.0);
                }
                // The label sits centred above its point (see `dy`), unless
                // that would push it off the canvas: a point on the edge of
                // the plot keeps its whole label.
                let centred = series_label.point.x - width / 2.0;
                let left = centred
                    .min(c1.width() + c1.margin.right - width)
                    .max(-c1.margin.left);
                if left != centred {
                    dx = Some(left - series_label.point.x);
                }
                let bottom = series_label.point.y - 8.0;
                if !placed.try_place(
                    left,
                    bottom - self.series_label_font_size,
                    width,
                    self.series_label_font_size,
                ) {
                    continue;
                }
                c1.text_unmeasured(Text {
                    text: series_label.text.clone(),
                    dy: Some(-8.0),
                    dx,
                    font_family: Some(self.font_family.clone()),
                    font_color: Some(self.series_label_font_color),
                    font_size: Some(self.series_label_font_size),
                    font_weight: self.series_label_font_weight.clone(),
                    x: Some(series_label.point.x),
                    y: Some(series_label.point.y),
                    ..Default::default()
                });
            }
        }
    }
    /// Renders the bar widget for canvas.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn render_bar(
        &self,
        c: Canvas,
        series_list: &[&Series],
        y_axis_values_list: &[&AxisValues],
        max_height: f32,
        series_data_count: usize,
        x_scale: Option<&ContinuousX>,
        radius: Option<f32>,
        animation: Option<&AnimationConfig>,
        tooltip: bool,
        label_inside: bool,
    ) -> Vec<Vec<SeriesLabel>> {
        if series_list.is_empty() {
            return vec![];
        }
        let mut c1 = c;

        // A band per category, or on a continuous axis as wide as the
        // smallest gap between two x values.
        let unit_width = match x_scale {
            Some(scale) => scale.band_width,
            None => c1.width() / series_data_count as f32,
        };
        let bar_chart_margin = 5.0_f32;
        let bar_chart_gap = 3.0_f32;
        let bar_chart_min_width = 1.0_f32;
        let bar_chart_margin_width = bar_chart_margin * 2.0;

        // Assign each series a visual slot index.
        // Non-stacked series each occupy their own slot.
        // All series sharing the same stack key (name + y_axis_index) share one slot.
        let mut stack_slot_keys: Vec<String> = vec![];
        let mut slot_count = 0_usize;
        let mut series_slot_indices: Vec<usize> = Vec::with_capacity(series_list.len());
        for series in series_list.iter() {
            if let Some(ref s) = series.stack {
                let key = format!("{}_{}", s, series.y_axis_index);
                if let Some(pos) = stack_slot_keys.iter().position(|k| k == &key) {
                    series_slot_indices.push(pos);
                } else {
                    series_slot_indices.push(slot_count);
                    stack_slot_keys.push(key);
                    slot_count += 1;
                }
            } else {
                series_slot_indices.push(slot_count);
                slot_count += 1;
            }
        }
        if slot_count == 0 {
            return vec![];
        }

        let bar_chart_gap_width = bar_chart_gap * (slot_count - 1) as f32;
        // A narrow band (many categories in a small canvas) has less room than
        // the fixed margins and gaps need, which would give a negative bar
        // width. Shrink the margins and gaps together, just enough to keep each
        // bar `bar_chart_min_width` wide, so the group stays inside its band
        // and centred on its tick. With room to spare `scale` is 1 and the
        // layout is unchanged.
        let bar_chart_spacing = bar_chart_margin_width + bar_chart_gap_width;
        let scale = ((unit_width - bar_chart_min_width * slot_count as f32) / bar_chart_spacing)
            .clamp(0.0, 1.0);
        let bar_chart_margin = bar_chart_margin * scale;
        let bar_chart_gap = bar_chart_gap * scale;
        let bar_width = (unit_width - bar_chart_spacing * scale) / slot_count as f32;
        let half_bar_width = bar_width / 2.0;

        // Per-stack accumulator: maps slot key → per-x cumulative data values,
        // kept as (positive sum, negative sum) so each sign stacks away from
        // the baseline on its own side.
        let mut stack_acc: Vec<(String, Vec<(f32, f32)>)> = stack_slot_keys
            .iter()
            .map(|k| (k.clone(), vec![(0.0_f32, 0.0_f32); series_data_count]))
            .collect();

        let mut series_labels_list = vec![];
        let get_bar_color = |colors: &Option<Vec<Option<Color>>>, index: usize| -> Option<Color> {
            if let Some(colors) = &colors {
                if colors.len() <= index {
                    return None;
                }
                if let Some(color) = colors[index] {
                    return Some(color);
                }
            }
            None
        };

        for (series_idx, series) in series_list.iter().enumerate() {
            let slot_index = series_slot_indices[series_idx];
            let y_axis_values = if series.y_axis_index >= y_axis_values_list.len() {
                y_axis_values_list[0]
            } else {
                y_axis_values_list[series.y_axis_index]
            };
            let color = get_color(&self.series_colors, series.index.unwrap_or(series_idx));
            // Bars grow from the 0 line, not from the axis bottom, so negative
            // values hang below it. With a minimum above 0 (custom `axis_min`)
            // or a log scale there is no 0 on the axis; fall back to the axis
            // bottom as before.
            let zero_y = y_axis_values
                .get_offset_height(0.0, max_height)
                .clamp(0.0, max_height);
            let zero_y = if zero_y.is_finite() {
                zero_y
            } else {
                max_height
            };

            // Find this series' stack accumulator (None for non-stacked).
            let stack_key = series
                .stack
                .as_ref()
                .map(|s| format!("{}_{}", s, series.y_axis_index));
            let acc_idx = stack_key
                .as_ref()
                .and_then(|k| stack_acc.iter().position(|(ak, _)| ak == k));

            let mut series_labels = vec![];
            for (i, value) in series.iter_values().enumerate() {
                if value == NIL_VALUE {
                    continue;
                }
                let actual_i = match x_scale {
                    Some(_) => self.x_slot(series, i),
                    None => i.saturating_add(series.start_index),
                };
                if actual_i >= series_data_count {
                    continue;
                }
                // The band starts at its category, or is centred on the x
                // value (a point without one cannot be placed).
                let band_left = match x_scale {
                    Some(scale) => match self.x_value(series, actual_i) {
                        Some(x) => scale.px(x) - unit_width / 2.0,
                        None => continue,
                    },
                    None => unit_width * actual_i as f32,
                };

                let mut left = band_left + bar_chart_margin;
                left += (bar_width + bar_chart_gap) * slot_index as f32;

                // `y_end` is the value end of the bar (where a label goes);
                // the rect spans from there to the stack base / zero line.
                let (y_top, bar_height, y_end) = if let Some(aidx) = acc_idx {
                    let (pos, neg) = stack_acc[aidx].1[actual_i];
                    let base = if value >= 0.0 { pos } else { neg };
                    let y_end = y_axis_values.get_offset_height(base + value, max_height);
                    let y_base = if base == 0.0 {
                        zero_y
                    } else {
                        y_axis_values.get_offset_height(base, max_height)
                    };
                    (y_end.min(y_base), (y_base - y_end).abs(), y_end)
                } else {
                    let y = y_axis_values.get_offset_height(value, max_height);
                    (y.min(zero_y), (zero_y - y).abs(), y)
                };

                let mut fill_color = get_bar_color(&series.colors, i);
                if fill_color.is_none() {
                    fill_color = Some(color);
                }
                let fill: Option<Fill> = fill_color.map(|c| c.into());

                let bar_style =
                    animation.map(|a| format!("animation-delay:{}ms", actual_i as u32 * a.delay));
                // A bar may carry both the grow-animation class and the
                // CSS hover-tooltip trigger class.
                let mut bar_classes: Vec<&str> = vec![];
                if animation.is_some() {
                    bar_classes.push("bar-anim");
                }
                if tooltip {
                    bar_classes.push("ct-trigger");
                }
                let bar_class = if bar_classes.is_empty() {
                    None
                } else {
                    Some(bar_classes.join(" "))
                };
                // Formatted once; only paid for when a label or tooltip
                // actually shows it.
                let label = if series.label_show || tooltip {
                    self.format_series_label(series, actual_i, value)
                } else {
                    String::new()
                };
                let tooltip_text = if tooltip {
                    format!("{}: {}", series.name, label)
                } else {
                    String::new()
                };

                c1.rect(Rect {
                    fill,
                    left,
                    top: y_top,
                    width: bar_width,
                    height: bar_height,
                    rx: radius,
                    ry: radius,
                    class: bar_class,
                    style: bar_style,
                    // Native <title> for accessibility / screen readers.
                    title: if tooltip {
                        Some(tooltip_text.clone())
                    } else {
                        None
                    },
                    dataset: self.point_dataset(series, actual_i, value),
                    color: None,
                });

                // CSS hover tooltip: a hidden label drawn immediately
                // after the bar, revealed via the adjacent-sibling rule
                // `.ct-trigger:hover + .ct-tip`. Works in any browser.
                if tooltip {
                    c1.text_unmeasured(Text {
                        text: tooltip_text,
                        class: Some("ct-tip".to_string()),
                        font_family: Some(self.font_family.clone()),
                        font_color: Some(self.series_label_font_color),
                        font_size: Some(self.series_label_font_size),
                        x: Some(left + half_bar_width),
                        y: Some(y_top),
                        dy: Some(-6.0),
                        text_anchor: Some("middle".to_string()),
                        ..Default::default()
                    });
                }

                // Update stack accumulator after rendering this bar segment.
                if let Some(aidx) = acc_idx {
                    let acc = &mut stack_acc[aidx].1[actual_i];
                    if value >= 0.0 {
                        acc.0 += value;
                    } else {
                        acc.1 += value;
                    }
                }

                if series.label_show {
                    // A label stands above the end of its bar, or in the
                    // middle of it (it is written 8 above its point). On an
                    // inverse axis the bar hangs down from its base, and the
                    // label goes below its end instead of into it.
                    let font_size = self.series_label_font_size;
                    let label_y = if label_inside {
                        y_top + bar_height / 2.0 + font_size * 0.35 + 8.0
                    } else if y_axis_values.inverse && value >= 0.0 {
                        y_end + font_size + 12.0
                    } else {
                        y_end
                    };
                    series_labels.push(SeriesLabel {
                        point: (left + half_bar_width, label_y).into(),
                        text: label,
                    });
                }
            }
            if series.label_show {
                series_labels_list.push(series_labels);
            }
        }
        series_labels_list
    }
    /// Renders the line widget for canvas.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn render_line(
        &self,
        c: Canvas,
        series_list: &[&Series],
        y_axis_values_list: &[&AxisValues],
        max_height: f32,
        axis_height: f32,
        series_data_count: usize,
        x_scale: Option<&ContinuousX>,
        animation: Option<&AnimationConfig>,
        tooltip: bool,
    ) -> Vec<Vec<SeriesLabel>> {
        if series_list.is_empty() {
            return vec![];
        }
        let mut c1 = c;
        let x_boundary_gap = self.x_boundary_gap.unwrap_or(true);
        let split_unit_offset = if !x_boundary_gap { 1.0_f32 } else { 0.0_f32 };
        // A single point without boundary gap would give 0 units (inf width).
        let split_unit_count = (series_data_count as f32 - split_unit_offset).max(1.0);
        let unit_width = c1.width() / split_unit_count;
        let mut series_labels_list = vec![];

        // Bands first, so every line is drawn over every band. A point with
        // a missing bound (or no place on the x axis) ends a stretch of band.
        let mut band_tips: Vec<BandTip> = vec![];
        for (index, series) in series_list.iter().enumerate() {
            let Some(band) = &series.band else {
                continue;
            };
            let y_axis_values = if series.y_axis_index >= y_axis_values_list.len() {
                y_axis_values_list[0]
            } else {
                y_axis_values_list[series.y_axis_index]
            };
            let color = get_color(&self.series_colors, series.index.unwrap_or(index))
                .with_alpha(BAND_ALPHA);
            let smooth = series.step.is_none() && series.smooth.unwrap_or(self.series_smooth);
            let (mut top, mut bottom): (Vec<Point>, Vec<Point>) = (vec![], vec![]);
            for i in 0..=band.len() {
                let slot = match x_scale {
                    Some(_) => self.x_slot(series, i),
                    None => i.saturating_add(series.start_index),
                };
                let x = if slot >= series_data_count {
                    None
                } else {
                    match x_scale {
                        Some(scale) => self.x_value(series, slot).map(|v| scale.px(v)),
                        None if x_boundary_gap => Some(unit_width * slot as f32 + unit_width / 2.0),
                        None => Some(unit_width * slot as f32),
                    }
                };
                if let (Some((lower, upper)), Some(x)) = (band.bounds(i), x) {
                    let top_y = y_axis_values.get_offset_height(upper, max_height);
                    let bottom_y = y_axis_values.get_offset_height(lower, max_height);
                    top.push((x, top_y).into());
                    bottom.push((x, bottom_y).into());
                    // Where the line has a point its tooltip tells the bounds;
                    // elsewhere the band needs a hover target of its own.
                    let has_point = matches!(
                        series.data.get(i),
                        Some(Some(v)) if v.is_finite() && *v != NIL_VALUE
                    );
                    if tooltip && !has_point {
                        band_tips.push(BandTip {
                            series: index,
                            slot,
                            x,
                            top: top_y,
                            bottom: bottom_y,
                            lower,
                            upper,
                        });
                    }
                    continue;
                }
                // End of a stretch (the index past the last point always is).
                let (top, bottom) = (std::mem::take(&mut top), std::mem::take(&mut bottom));
                if top.len() < 2 {
                    continue;
                }
                let dataset = vec![("series".to_string(), series.name.clone())];
                if smooth {
                    c1.smooth_band(SmoothBand {
                        top,
                        bottom,
                        fill: Some(color),
                        class: Some("ct-band".to_string()),
                        dataset,
                        ..Default::default()
                    });
                } else {
                    // The band of a stepped line steps along with it.
                    let (mut points, bottom) = match series.step {
                        Some(step) => (step_points(&top, step), step_points(&bottom, step)),
                        None => (top, bottom),
                    };
                    points.extend(bottom.into_iter().rev());
                    c1.polygon(Polygon {
                        fill: Some(color),
                        points,
                        class: Some("ct-band".to_string()),
                        dataset,
                        ..Default::default()
                    });
                }
            }
        }

        // Stack accumulators for line series: stack_key -> Vec<f32> of cumulative
        // data values per x-position (data space, not pixel space).
        let mut stack_acc: Vec<(String, Vec<f32>)> = vec![];

        for (index, series) in series_list.iter().enumerate() {
            let y_axis_values = if series.y_axis_index >= y_axis_values_list.len() {
                y_axis_values_list[0]
            } else {
                y_axis_values_list[series.y_axis_index]
            };

            let stack_key = series
                .stack
                .as_ref()
                .map(|s| format!("{}_{}", s, series.y_axis_index));
            let is_stacked = stack_key.is_some();

            // Previous stack totals, updated in place. A non-stacked series
            // reads a base of 0 and allocates nothing.
            let acc_idx = if let Some(ref key) = stack_key {
                if let Some(pos) = stack_acc.iter().position(|(k, _)| k == key) {
                    Some(pos)
                } else {
                    stack_acc.push((key.clone(), vec![0.0_f32; series_data_count]));
                    Some(stack_acc.len() - 1)
                }
            } else {
                None
            };

            let mut points: Vec<Point> = Vec::with_capacity(series.data.len());
            // For stacked fills: the "floor" points of the previous stack level.
            let mut floor_points: Vec<Point> = Vec::new();
            let mut points_list: Vec<Vec<Point>> = vec![];
            let mut floor_points_list: Vec<Vec<Point>> = vec![];
            // Labels and hit targets are only built when something displays them.
            let track_points = series.label_show || tooltip || !series.mark_points.is_empty();
            let format_every_label = series.label_show || tooltip;
            let mut series_labels = Vec::new();
            let mut point_datasets = Vec::new();

            let mut max_value = f32::MIN;
            let mut min_value = f32::MAX;
            let mut max_index = 0;
            let mut min_index = 0;
            let mut max_actual_i = 0;
            let mut min_actual_i = 0;
            let mut max_raw = 0.0_f32;
            let mut min_raw = 0.0_f32;

            for (i, value) in series.iter_values().enumerate() {
                let actual_i = match x_scale {
                    Some(_) => self.x_slot(series, i),
                    None => i.saturating_add(series.start_index),
                };
                // On a continuous axis the point sits at its x value; one
                // without an x value breaks the line like a missing point.
                let continuous_x = x_scale.map(|scale| {
                    self.x_value(series, actual_i)
                        .map(|x_value| scale.px(x_value))
                });
                if value == NIL_VALUE || continuous_x == Some(None) {
                    if !points.is_empty() {
                        points_list.push(std::mem::take(&mut points));
                        floor_points_list.push(std::mem::take(&mut floor_points));
                    }
                    continue;
                }
                if actual_i >= series_data_count {
                    continue;
                }

                let base_acc = match acc_idx {
                    Some(idx) => stack_acc[idx].1[actual_i],
                    None => 0.0,
                };
                let effective_value = base_acc + value;

                // Index into `series_labels` (which skips missing points), not
                // the raw data index, so a mark point lands on the right label.
                if track_points {
                    if effective_value > max_value {
                        max_value = effective_value;
                        max_index = series_labels.len();
                        max_actual_i = actual_i;
                        max_raw = value;
                    }
                    if effective_value < min_value {
                        min_value = effective_value;
                        min_index = series_labels.len();
                        min_actual_i = actual_i;
                        min_raw = value;
                    }
                }

                let x = match continuous_x.flatten() {
                    Some(x) => x,
                    None => {
                        let mut x = unit_width * actual_i as f32;
                        if x_boundary_gap {
                            x += unit_width / 2.0;
                        }
                        x
                    }
                };
                let y = y_axis_values.get_offset_height(effective_value, max_height);
                points.push((x, y).into());

                // Floor points for stacked area fill (previous cumulative level).
                if is_stacked {
                    let floor_y = y_axis_values.get_offset_height(base_acc, max_height);
                    floor_points.push((x, floor_y).into());
                }

                if let Some(idx) = acc_idx {
                    stack_acc[idx].1[actual_i] = effective_value;
                }

                if track_points {
                    let text = if format_every_label {
                        self.format_series_label(series, actual_i, value)
                    } else {
                        String::new()
                    };
                    series_labels.push(SeriesLabel {
                        point: (x, y).into(),
                        text,
                    });
                    if tooltip {
                        let mut dataset = self.point_dataset(series, actual_i, value);
                        // A point inside a band also tells its bounds.
                        let range = series.band.as_ref().and_then(|band| band.bounds(i)).map(
                            |(lower, upper)| {
                                let (lower, upper) = (format_float(lower), format_float(upper));
                                dataset.push(("lower".to_string(), lower.clone()));
                                dataset.push(("upper".to_string(), upper.clone()));
                                format!(" ({lower} – {upper})")
                            },
                        );
                        point_datasets.push((dataset, range));
                    }
                }
            }

            if !points.is_empty() {
                points_list.push(points);
                floor_points_list.push(floor_points);
            }

            // Mark points only need the min and max labels, not one per point.
            if track_points && !format_every_label {
                if let Some(label) = series_labels.get_mut(max_index) {
                    label.text = self.format_series_label(series, max_actual_i, max_raw);
                }
                if min_index != max_index
                    && let Some(label) = series_labels.get_mut(min_index)
                {
                    label.text = self.format_series_label(series, min_actual_i, min_raw);
                }
            }

            if series.label_show {
                if tooltip || !series.mark_points.is_empty() {
                    series_labels_list.push(series_labels.clone());
                } else {
                    series_labels_list.push(std::mem::take(&mut series_labels));
                }
            }

            let color = get_color(&self.series_colors, series.index.unwrap_or(index));
            let fill_color = color.with_alpha(100);
            let fill: Fill = fill_color.into();
            let series_fill = series.fill.unwrap_or(self.series_fill);
            // The area of a line reaches down to the start of its axis,
            // which is at the top when the axis is inverse.
            let fill_bottom = if y_axis_values.inverse {
                0.0
            } else {
                axis_height
            };
            // Steps have corners: a stepped line is never smooth.
            let step = series.step;
            let series_smooth = step.is_none() && series.smooth.unwrap_or(self.series_smooth);
            let symbol = series.symbol.clone().or_else(|| self.series_symbol.clone());

            for (points, floor) in points_list.into_iter().zip(floor_points_list) {
                let line_class = animation.map(|_| format!("line-anim-{}", index));
                let line_path_length = animation.map(|_| 1.0_f32);

                // Fill first, then stroke. The stacked-area polygon is
                // identical for smooth and straight lines, so it lives in
                // one place; only the non-stacked fill and the stroke
                // itself differ by `series_smooth`. The stroke takes ownership
                // of `points`; the fill clones only when it is also drawn.
                // The outline of the area: the line itself, or its steps.
                let outline = |points: &[Point]| match step {
                    Some(step) => step_points(points, step),
                    None => points.to_vec(),
                };
                if series_fill {
                    if is_stacked {
                        // Area between the current and previous stack
                        // level: top points forward + floor reversed.
                        let mut poly = outline(&points);
                        let mut rev_floor = outline(&floor);
                        rev_floor.reverse();
                        poly.extend(rev_floor);
                        c1.polygon(Polygon {
                            fill: Some(fill_color),
                            points: poly,
                            ..Default::default()
                        });
                    } else if series_smooth {
                        c1.smooth_line_fill(SmoothLineFill {
                            fill,
                            points: points.clone(),
                            bottom: fill_bottom,
                        });
                    } else {
                        c1.straight_line_fill(StraightLineFill {
                            fill,
                            points: outline(&points),
                            bottom: fill_bottom,
                            ..Default::default()
                        });
                    }
                }
                if series_smooth {
                    c1.smooth_line(SmoothLine {
                        points,
                        color: Some(color),
                        stroke_width: self.series_stroke_width,
                        symbol: symbol.clone(),
                        stroke_dash_array: series.stroke_dash_array.clone(),
                        class: line_class,
                        path_length: line_path_length,
                    });
                } else {
                    c1.straight_line(StraightLine {
                        points,
                        color: Some(color),
                        stroke_width: self.series_stroke_width,
                        symbol: symbol.clone(),
                        stroke_dash_array: series.stroke_dash_array.clone(),
                        class: line_class,
                        path_length: line_path_length,
                        step,
                        ..Default::default()
                    });
                }
            }

            // Transparent hit-circles at each data point (the line's own
            // symbols are batch-drawn). Each carries a native <title>
            // (accessibility) and the `ct-trigger` class, immediately
            // followed by a hidden `.ct-tip` label revealed on hover.
            if tooltip {
                for (label, (dataset, range)) in series_labels.iter().zip(point_datasets) {
                    let text = format!(
                        "{}: {}{}",
                        series.name,
                        label.text,
                        range.as_deref().unwrap_or("")
                    );
                    c1.circle(Circle {
                        cx: label.point.x,
                        cy: label.point.y,
                        r: self.series_stroke_width.max(2.0) + 2.0,
                        fill: Some(Color::transparent()),
                        title: Some(text.clone()),
                        class: Some("ct-trigger".to_string()),
                        dataset,
                        ..Default::default()
                    });
                    c1.text_unmeasured(Text {
                        text,
                        class: Some("ct-tip".to_string()),
                        font_family: Some(self.font_family.clone()),
                        font_color: Some(self.series_label_font_color),
                        font_size: Some(self.series_label_font_size),
                        x: Some(label.point.x),
                        y: Some(label.point.y),
                        dy: Some(-8.0),
                        text_anchor: Some("middle".to_string()),
                        ..Default::default()
                    });
                }
                // The band points without a line point: a transparent strip
                // across the band, as wide as the hit-circles above.
                let half_width = self.series_stroke_width.max(2.0) + 2.0;
                for tip in band_tips.iter().filter(|tip| tip.series == index) {
                    let (lower, upper) = (format_float(tip.lower), format_float(tip.upper));
                    let text = format!("{}: {lower} – {upper}", series.name);
                    let mut dataset = vec![("series".to_string(), series.name.clone())];
                    if let Some(category) = self.x_label(series, tip.slot) {
                        dataset.push(("category".to_string(), category.into_owned()));
                    }
                    dataset.push(("lower".to_string(), lower));
                    dataset.push(("upper".to_string(), upper));
                    c1.rect(Rect {
                        fill: Some(Color::transparent().into()),
                        left: tip.x - half_width,
                        top: tip.top,
                        width: half_width * 2.0,
                        height: (tip.bottom - tip.top).max(1.0),
                        title: Some(text.clone()),
                        class: Some("ct-trigger".to_string()),
                        // Nothing is painted, so ask for the hover anyway.
                        style: Some("pointer-events:all".to_string()),
                        dataset,
                        ..Default::default()
                    });
                    c1.text_unmeasured(Text {
                        text,
                        class: Some("ct-tip".to_string()),
                        font_family: Some(self.font_family.clone()),
                        font_color: Some(self.series_label_font_color),
                        font_size: Some(self.series_label_font_size),
                        x: Some(tip.x),
                        y: Some(tip.top),
                        dy: Some(-8.0),
                        text_anchor: Some("middle".to_string()),
                        ..Default::default()
                    });
                }
            }

            for mark_point in series.mark_points.iter() {
                let mp_index = match mark_point.category {
                    MarkPointCategory::Max => max_index,
                    MarkPointCategory::Min => min_index,
                };
                if let Some(label) = series_labels.get(mp_index) {
                    let r = 15.0;
                    let y = label.point.y - r * 2.0;
                    c1.bubble(Bubble {
                        x: label.point.x,
                        y,
                        r,
                        fill: color,
                    });
                    let mut dx = None;
                    if let Ok(value) = measure_text_width_family(
                        &self.font_family,
                        self.series_label_font_size,
                        &label.text,
                    ) {
                        dx = Some(-value.width() / 2.0 + 1.0);
                    }
                    let font_color = if color.is_light() {
                        "#464646".into()
                    } else {
                        "#D8D9DA".into()
                    };
                    c1.text(Text {
                        text: label.text.clone(),
                        line_height: Some(r),
                        dx,
                        font_color: Some(font_color),
                        font_family: Some(self.font_family.clone()),
                        font_size: Some(self.series_label_font_size),
                        x: Some(label.point.x),
                        y: Some(y - r * 0.5 + 2.0),
                        ..Default::default()
                    });
                }
            }
        }
        series_labels_list
    }
}
