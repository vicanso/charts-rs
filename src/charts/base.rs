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
    /// Color palette cycled through by the series.
    pub series_colors: Vec<Color>,
    /// Marker drawn on data points (circle, dot or none).
    pub series_symbol: Option<Symbol>,
    /// Draws line series as smooth curves.
    pub series_smooth: bool,
    /// Fills the area under line series.
    pub series_fill: bool,

    /// SVG animation (duration/easing/stagger delay) for the chart types that
    /// support it (bar, line, pie, funnel, sunburst, treemap, sankey).
    pub animation: Option<AnimationConfig>,
    /// When `true`, data shapes get a hover tooltip (`series: value`): a
    /// CSS-revealed label that works in any browser, plus a native `<title>`
    /// for accessibility. Supported by bar, horizontal bar, line, pie and
    /// scatter charts. Default: false; output is unchanged when off.
    pub tooltip_show: bool,
}

/// Gets y axis config by index.
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
}

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
                data_list.append(&mut series.data_values());
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
                    for (i, &v) in series.data_values().iter().enumerate() {
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
        let y_axis_width = if let Some(value) = y_axis_config.axis_width {
            value
        } else {
            let y_axis_formatter = &y_axis_config.axis_formatter.clone().unwrap_or_default();
            let mut longest_item: &str = "";
            for item in &y_axis_values.data {
                if item.chars().count() > longest_item.chars().count() {
                    longest_item = item
                }
            }
            let value = format_string(longest_item, y_axis_formatter);
            if let Ok(b) =
                measure_text_width_family(&self.font_family, y_axis_config.axis_font_size, &value)
            {
                b.width() + 5.0
            } else {
                DEFAULT_Y_AXIS_WIDTH
            }
        };
        (y_axis_values, y_axis_width)
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
    ) -> CartesianLayout {
        let mut c = c;
        let axis_top = self.render_header(&mut c);
        let left = self.get_y_axis_values(y_axis_configs, 0);
        let right = if self.series_list.iter().any(|s| s.y_axis_index != 0) {
            Some(self.get_y_axis_values(y_axis_configs, 1))
        } else {
            None
        };
        self.layout_cartesian_with(c, y_axis_configs, axis_top, left, right)
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
    ) -> CartesianLayout {
        let mut c = c;
        let x_axis_height = if self.x_axis_hidden {
            0.0
        } else {
            self.x_axis_height
        };
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
        let axis_height = (c.height() - x_axis_height - axis_top).max(0.0);
        let axis_width = (c.width() - left_width - right_width).max(0.0);
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
            self.render_x_axis(
                c.child(Box {
                    top: c.height() - x_axis_height,
                    left: left_width,
                    right: right_width,
                    ..Default::default()
                }),
                self.x_axis_data.clone(),
                axis_width,
            );
        }
        let max_height = c.height() - x_axis_height;
        CartesianLayout {
            canvas: c,
            left: left_values,
            right: right_values,
            left_width,
            right_width,
            axis_height,
            axis_width,
            max_height,
        }
    }
    /// True when there is nothing to draw: no series, or only empty ones.
    pub(crate) fn has_no_data(&self) -> bool {
        self.series_list
            .iter()
            .all(|s| s.data.iter().all(Option::is_none))
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
        if let Some(category) = self.x_axis_data.get(index) {
            dataset.push(("category".to_string(), category.clone()));
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
            if series.mark_lines.is_empty() {
                continue;
            }
            let y_axis_values = if series.y_axis_index >= y_axis_values_list.len() {
                y_axis_values_list[0]
            } else {
                y_axis_values_list[series.y_axis_index]
            };
            let color = get_color(&self.series_colors, series.index.unwrap_or(index));
            let values: Vec<f32> = series
                .data_values()
                .into_iter()
                .filter(|v| *v != NIL_VALUE)
                .collect();
            let (mut sum, mut min, mut max) = (0.0_f32, f32::MAX, f32::MIN);
            for &v in values.iter() {
                sum += v;
                max = max.max(v);
                min = min.min(v);
            }
            for mark_line in series.mark_lines.iter() {
                let value = match mark_line.category {
                    MarkLineCategory::Value(v) => v,
                    // No valid points: average/min/max are undefined.
                    _ if values.is_empty() => continue,
                    MarkLineCategory::Average => sum / values.len() as f32,
                    MarkLineCategory::Max => max,
                    MarkLineCategory::Min => min,
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
        let category = self
            .x_axis_data
            .get(index)
            .map(String::as_str)
            .unwrap_or("");
        format_series_label(&self.series_label_formatter, value, &series.name, category)
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
            height: self.x_axis_height,
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
            ..Default::default()
        });
    }
    /// Renders series label widget for canvas.
    pub(crate) fn render_series_label(&self, c: Canvas, series_labels_list: Vec<Vec<SeriesLabel>>) {
        if series_labels_list.is_empty() {
            return;
        }
        let mut c1 = c;
        for series_labels in series_labels_list.iter() {
            for series_label in series_labels.iter() {
                let mut dx = None;
                if let Ok(value) = measure_text_width_family(
                    &self.font_family,
                    self.series_label_font_size,
                    &series_label.text,
                ) {
                    dx = Some(-value.width() / 2.0);
                }
                c1.text(Text {
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
        radius: Option<f32>,
        animation: Option<&AnimationConfig>,
        tooltip: bool,
    ) -> Vec<Vec<SeriesLabel>> {
        if series_list.is_empty() {
            return vec![];
        }
        let mut c1 = c;

        let unit_width = c1.width() / series_data_count as f32;
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
            for (i, p) in series.data_values().iter().enumerate() {
                let value = p.to_owned();
                if value == NIL_VALUE {
                    continue;
                }
                let actual_i = i + series.start_index;
                if actual_i >= series_data_count {
                    continue;
                }

                let mut left = unit_width * actual_i as f32 + bar_chart_margin;
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
                    c1.text(Text {
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
                    series_labels.push(SeriesLabel {
                        point: (left + half_bar_width, y_end).into(),
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

            // Retrieve the current accumulated data values for this stack group.
            let acc_data: Vec<f32> = if let Some(ref key) = stack_key {
                stack_acc
                    .iter()
                    .find(|(k, _)| k == key)
                    .map(|(_, v)| v.clone())
                    .unwrap_or_else(|| vec![0.0_f32; series_data_count])
            } else {
                vec![0.0_f32; series_data_count]
            };

            let mut points: Vec<Point> = vec![];
            // For stacked fills: the "floor" points of the previous stack level.
            let mut floor_points: Vec<Point> = vec![];
            let mut points_list: Vec<Vec<Point>> = vec![];
            let mut floor_points_list: Vec<Vec<Point>> = vec![];
            let mut series_labels = vec![];
            let mut point_datasets = vec![];

            let mut max_value = f32::MIN;
            let mut min_value = f32::MAX;
            let mut max_index = 0;
            let mut min_index = 0;

            // Build updated accumulator for this series.
            let mut new_acc = acc_data.clone();

            for (i, p) in series.data_values().iter().enumerate() {
                let value = p.to_owned();
                let actual_i = i + series.start_index;
                if value == NIL_VALUE {
                    if !points.is_empty() {
                        points_list.push(points);
                        floor_points_list.push(floor_points);
                        points = vec![];
                        floor_points = vec![];
                    }
                    continue;
                }
                if actual_i >= series_data_count {
                    continue;
                }

                let base_acc = acc_data[actual_i];
                let effective_value = base_acc + value;

                // Index into `series_labels` (which skips missing points), not
                // the raw data index, so a mark point lands on the right label.
                if effective_value > max_value {
                    max_value = effective_value;
                    max_index = series_labels.len();
                }
                if effective_value < min_value {
                    min_value = effective_value;
                    min_index = series_labels.len();
                }

                let mut x = unit_width * actual_i as f32;
                if x_boundary_gap {
                    x += unit_width / 2.0;
                }
                let y = y_axis_values.get_offset_height(effective_value, max_height);
                points.push((x, y).into());

                // Floor points for stacked area fill (previous cumulative level).
                if is_stacked {
                    let floor_y = y_axis_values.get_offset_height(base_acc, max_height);
                    floor_points.push((x, floor_y).into());
                }

                new_acc[actual_i] += value;

                series_labels.push(SeriesLabel {
                    point: (x, y).into(),
                    text: self.format_series_label(series, actual_i, value),
                });
                point_datasets.push(self.point_dataset(series, actual_i, value));
            }

            if !points.is_empty() {
                points_list.push(points);
                floor_points_list.push(floor_points);
            }

            // Update stack accumulator for subsequent series in the same group.
            if let Some(ref key) = stack_key {
                if let Some(entry) = stack_acc.iter_mut().find(|(k, _)| k == key) {
                    entry.1 = new_acc;
                } else {
                    stack_acc.push((key.clone(), new_acc));
                }
            }

            if series.label_show {
                series_labels_list.push(series_labels.clone());
            }

            let color = get_color(&self.series_colors, series.index.unwrap_or(index));
            let fill_color = color.with_alpha(100);
            let fill: Fill = fill_color.into();
            let series_fill = series.fill.unwrap_or(self.series_fill);
            let series_smooth = series.smooth.unwrap_or(self.series_smooth);
            let symbol = series.symbol.clone().or_else(|| self.series_symbol.clone());

            for (seg_idx, points) in points_list.iter().enumerate() {
                let floor = floor_points_list.get(seg_idx);

                let line_class = animation.map(|_| format!("line-anim-{}", index));
                let line_path_length = animation.map(|_| 1.0_f32);

                // Fill first, then stroke. The stacked-area polygon is
                // identical for smooth and straight lines, so it lives in
                // one place; only the non-stacked fill and the stroke
                // itself differ by `series_smooth`.
                if series_fill {
                    if is_stacked {
                        if let Some(fp) = floor {
                            // Area between the current and previous stack
                            // level: top points forward + floor reversed.
                            let mut poly = points.clone();
                            let mut rev_floor = fp.clone();
                            rev_floor.reverse();
                            poly.extend(rev_floor);
                            c1.polygon(Polygon {
                                fill: Some(fill_color),
                                points: poly,
                                ..Default::default()
                            });
                        }
                    } else if series_smooth {
                        c1.smooth_line_fill(SmoothLineFill {
                            fill,
                            points: points.clone(),
                            bottom: axis_height,
                        });
                    } else {
                        c1.straight_line_fill(StraightLineFill {
                            fill,
                            points: points.clone(),
                            bottom: axis_height,
                            ..Default::default()
                        });
                    }
                }
                if series_smooth {
                    c1.smooth_line(SmoothLine {
                        points: points.clone(),
                        color: Some(color),
                        stroke_width: self.series_stroke_width,
                        symbol: symbol.clone(),
                        stroke_dash_array: series.stroke_dash_array.clone(),
                        class: line_class.clone(),
                        path_length: line_path_length,
                    });
                } else {
                    c1.straight_line(StraightLine {
                        points: points.clone(),
                        color: Some(color),
                        stroke_width: self.series_stroke_width,
                        symbol: symbol.clone(),
                        stroke_dash_array: series.stroke_dash_array.clone(),
                        class: line_class.clone(),
                        path_length: line_path_length,
                        ..Default::default()
                    });
                }
            }

            // Transparent hit-circles at each data point (the line's own
            // symbols are batch-drawn). Each carries a native <title>
            // (accessibility) and the `ct-trigger` class, immediately
            // followed by a hidden `.ct-tip` label revealed on hover.
            if tooltip {
                for (label, dataset) in series_labels.iter().zip(point_datasets) {
                    let text = format!("{}: {}", series.name, label.text);
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
                    c1.text(Text {
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
