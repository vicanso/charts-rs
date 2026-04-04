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

use super::canvas;
use super::color::*;
use super::common::*;
use super::component::*;
use super::params::*;
use super::theme::{get_default_theme_name, get_theme, Theme, DEFAULT_Y_AXIS_WIDTH};
use super::util::*;
use super::Canvas;
use crate::charts::measure_text_width_family;
use charts_rs_derive::Chart;
use std::sync::Arc;

// ── Data types ────────────────────────────────────────────────────────────────

/// A single bar in the waterfall chart.
#[derive(Clone, Debug, Default)]
pub struct WaterfallData {
    /// The numeric value for this bar (positive = increase, negative = decrease).
    /// For `is_total = true` bars this is the cumulative value to display (usually
    /// computed automatically, but can be set explicitly if you want to override it).
    pub value: f32,
    /// When `true` this bar is rendered as a "total" bar that starts at `0` and
    /// spans to the current running sum.  Colors it with `total_color`.
    pub is_total: bool,
}

impl From<f32> for WaterfallData {
    fn from(value: f32) -> Self {
        WaterfallData {
            value,
            is_total: false,
        }
    }
}

impl From<(f32, bool)> for WaterfallData {
    fn from(v: (f32, bool)) -> Self {
        WaterfallData {
            value: v.0,
            is_total: v.1,
        }
    }
}

// ── WaterfallChart ────────────────────────────────────────────────────────────

#[derive(Clone, Debug, Default, Chart)]
pub struct WaterfallChart {
    pub width: f32,
    pub height: f32,
    pub x: f32,
    pub y: f32,
    pub margin: Box,
    // dummy – required by #[derive(Chart)]
    pub series_list: Vec<Series>,
    pub font_family: String,
    pub background_color: Color,
    pub is_light: bool,

    // title
    pub title_text: String,
    pub title_font_size: f32,
    pub title_font_color: Color,
    pub title_font_weight: Option<String>,
    pub title_margin: Option<Box>,
    pub title_align: Align,
    pub title_height: f32,

    // sub title
    pub sub_title_text: String,
    pub sub_title_font_size: f32,
    pub sub_title_font_color: Color,
    pub sub_title_font_weight: Option<String>,
    pub sub_title_margin: Option<Box>,
    pub sub_title_align: Align,
    pub sub_title_height: f32,

    // legend (required by derive – not shown by default)
    pub legend_font_size: f32,
    pub legend_font_color: Color,
    pub legend_font_weight: Option<String>,
    pub legend_align: Align,
    pub legend_margin: Option<Box>,
    pub legend_category: LegendCategory,
    pub legend_show: Option<bool>,

    // x axis
    pub x_axis_data: Vec<String>,
    pub x_axis_height: f32,
    pub x_axis_stroke_color: Color,
    pub x_axis_font_size: f32,
    pub x_axis_font_color: Color,
    pub x_axis_font_weight: Option<String>,
    pub x_axis_name_gap: f32,
    pub x_axis_name_rotate: f32,
    pub x_axis_margin: Option<Box>,
    pub x_axis_hidden: bool,
    pub x_boundary_gap: Option<bool>,

    // y axis
    pub y_axis_hidden: bool,
    y_axis_configs: Vec<YAxisConfig>,

    // grid
    grid_stroke_color: Color,
    grid_stroke_width: f32,

    // series (required by derive)
    pub series_stroke_width: f32,
    pub series_label_font_color: Color,
    pub series_label_font_size: f32,
    pub series_label_font_weight: Option<String>,
    pub series_label_formatter: String,
    pub series_colors: Vec<Color>,
    pub series_symbol: Option<Symbol>,
    pub series_smooth: bool,
    pub series_fill: bool,

    // ── Waterfall-specific fields ─────────────────────────────────────────────

    /// The data points.  Each value is an increment/decrement, except for
    /// entries where `is_total = true` which reset to 0 and show the running sum.
    pub data: Vec<WaterfallData>,

    /// Bar color for positive increments.  Defaults to the first `series_colors` entry.
    pub increase_color: Color,

    /// Bar color for negative increments.  Defaults to a warm red.
    pub decrease_color: Color,

    /// Bar color for "total" bars.  Defaults to the second `series_colors` entry.
    pub total_color: Color,

    /// Whether to show value labels above/below each bar (default: true).
    pub label_show: bool,

    /// Whether to draw a dashed connector line between adjacent bars (default: true).
    pub connector_line_show: bool,

    /// Fraction of each x-unit occupied by a bar (0..1, default: 0.6).
    pub bar_width_ratio: f32,
}

impl WaterfallChart {
    fn fill_default(&mut self) {
        // legend hidden by default (no series names in the usual sense)
        if self.legend_show.is_none() {
            self.legend_show = Some(false);
        }
        if self.bar_width_ratio <= 0.0 {
            self.bar_width_ratio = 0.6;
        }
        if self.increase_color.is_zero() {
            self.increase_color = get_color(&self.series_colors, 0);
        }
        if self.total_color.is_zero() {
            self.total_color = get_color(&self.series_colors, 1);
        }
        if self.decrease_color.is_zero() {
            // warm red not in the default palette – hard-coded
            self.decrease_color = (238, 102, 102).into(); // #EE6666
        }
    }

    /// Creates a waterfall chart with default theme.
    pub fn new(data: Vec<WaterfallData>, x_axis_data: Vec<String>) -> WaterfallChart {
        WaterfallChart::new_with_theme(data, x_axis_data, &get_default_theme_name())
    }

    /// Creates a waterfall chart with a custom theme.
    pub fn new_with_theme(
        data: Vec<WaterfallData>,
        x_axis_data: Vec<String>,
        theme: &str,
    ) -> WaterfallChart {
        let mut c = WaterfallChart {
            data,
            x_axis_data,
            label_show: true,
            connector_line_show: true,
            ..Default::default()
        };
        c.fill_theme(get_theme(theme));
        c.fill_default();
        c
    }

    /// Creates a waterfall chart from a JSON string.
    pub fn from_json(json: &str) -> canvas::Result<WaterfallChart> {
        let mut c = WaterfallChart {
            label_show: true,
            connector_line_show: true,
            ..Default::default()
        };
        let value = c.fill_option(json)?;

        if let Some(b) = get_bool_from_value(&value, "x_axis_hidden") {
            c.x_axis_hidden = b;
        }
        if let Some(b) = get_bool_from_value(&value, "y_axis_hidden") {
            c.y_axis_hidden = b;
        }
        if let Some(b) = get_bool_from_value(&value, "label_show") {
            c.label_show = b;
        }
        if let Some(b) = get_bool_from_value(&value, "connector_line_show") {
            c.connector_line_show = b;
        }
        if let Some(v) = get_f32_from_value(&value, "bar_width_ratio") {
            c.bar_width_ratio = v;
        }
        if let Some(col) = get_color_from_value(&value, "increase_color") {
            c.increase_color = col;
        }
        if let Some(col) = get_color_from_value(&value, "decrease_color") {
            c.decrease_color = col;
        }
        if let Some(col) = get_color_from_value(&value, "total_color") {
            c.total_color = col;
        }

        // parse data: array of either [value, is_total] or bare numbers
        if let Some(arr) = value.get("data").and_then(|v| v.as_array()) {
            let mut items = vec![];
            for item in arr {
                if let Some(pair) = item.as_array() {
                    let val = pair.first().and_then(|v| v.as_f64()).unwrap_or(0.0) as f32;
                    let is_total = pair.get(1).and_then(|v| v.as_bool()).unwrap_or(false);
                    items.push(WaterfallData { value: val, is_total });
                } else if let Some(v) = item.as_f64() {
                    items.push(WaterfallData { value: v as f32, is_total: false });
                }
            }
            c.data = items;
        }
        if let Some(x) = get_string_slice_from_value(&value, "x_axis_data") {
            c.x_axis_data = x;
        }

        c.fill_default();
        Ok(c)
    }

    /// Computes the cumulative running sum at each bar position.
    ///
    /// For a normal bar, the cumulative *before* is the sum of all previous
    /// non-total bars.  For a total bar, the running sum is the value to display
    /// (we auto-compute it unless the user supplied a non-zero explicit value).
    fn compute_cumulative(&self) -> Vec<(f32, f32)> {
        // Returns (bar_bottom, bar_top) in data coordinates for every bar.
        // (bar_bottom, bar_top) with bar_top >= bar_bottom means visually the
        // filled region runs from bar_bottom upward to bar_top.
        let mut cum: f32 = 0.0;
        let mut result = Vec::with_capacity(self.data.len());

        for item in &self.data {
            if item.is_total {
                // Total bar: visual range is [0, cumulative_sum].
                // If the user set an explicit value, use it; otherwise auto-compute.
                let display = if item.value != 0.0 { item.value } else { cum };
                result.push((0.0_f32, display));
                // Totals reset-accumulate to match the display value for subsequent deltas
                cum = display;
            } else {
                let bottom = cum;
                let top = cum + item.value;
                result.push((bottom, top));
                cum = top;
            }
        }
        result
    }

    /// Renders the waterfall chart to an SVG string.
    pub fn svg(&self) -> canvas::Result<String> {
        if self.data.is_empty() {
            return Err(canvas::Error::Params {
                message: "data is empty".to_string(),
            });
        }

        let mut c = Canvas::new_width_xy(self.width, self.height, self.x, self.y);
        self.render_background(c.child(Box::default()));

        let mut x_axis_height = self.x_axis_height;
        if self.x_axis_hidden {
            x_axis_height = 0.0;
        }
        c.margin = self.margin.clone();

        let title_height = self.render_title(c.child(Box::default()));
        let legend_height = self.render_legend(c.child(Box::default()));
        let axis_top = title_height.max(legend_height);

        // ── Compute axis values ───────────────────────────────────────────────
        let cum = self.compute_cumulative();
        // Collect all boundary values for the y-axis range
        let all_vals: Vec<f32> = cum
            .iter()
            .flat_map(|(b, t)| [*b, *t])
            .collect();

        let y_axis_config = &self.y_axis_configs[0];
        let y_axis_values = get_axis_values(AxisValueParams {
            data_list: all_vals,
            split_number: y_axis_config.axis_split_number,
            reverse: Some(true),
            min: y_axis_config.axis_min,
            max: y_axis_config.axis_max,
            thousands_format: y_axis_config
                .axis_formatter
                .as_deref()
                .unwrap_or("")
                .contains(THOUSANDS_FORMAT_LABEL),
            scale: y_axis_config.axis_scale.clone(),
        });

        let mut y_axis_width = if self.y_axis_hidden {
            0.0
        } else if let Some(w) = y_axis_config.axis_width {
            w
        } else {
            let formatter = y_axis_config.axis_formatter.clone().unwrap_or_default();
            let longest = y_axis_values
                .data
                .iter()
                .max_by_key(|s| s.len())
                .map(|s| s.as_str())
                .unwrap_or("");
            let label = format_string(longest, &formatter);
            measure_text_width_family(&self.font_family, y_axis_config.axis_font_size, &label)
                .map(|b| b.width() + 5.0)
                .unwrap_or(DEFAULT_Y_AXIS_WIDTH)
        };
        if self.y_axis_hidden {
            y_axis_width = 0.0;
        }

        let axis_height = c.height() - x_axis_height - axis_top;
        let axis_width = c.width() - y_axis_width;

        if axis_top > 0.0 {
            c = c.child(Box {
                top: axis_top,
                ..Default::default()
            });
        }

        // ── Render grid / axes ────────────────────────────────────────────────
        self.render_grid(
            c.child(Box {
                left: y_axis_width,
                ..Default::default()
            }),
            axis_width,
            axis_height,
        );

        if y_axis_width > 0.0 {
            self.render_y_axis(
                c.child(Box::default()),
                y_axis_values.data.clone(),
                axis_height,
                y_axis_width,
                0,
            );
        }

        if !self.x_axis_hidden {
            self.render_x_axis(
                c.child(Box {
                    top: c.height() - x_axis_height,
                    left: y_axis_width,
                    ..Default::default()
                }),
                self.x_axis_data.clone(),
                axis_width,
            );
        }

        // ── Render bars ───────────────────────────────────────────────────────
        let n = self.data.len();
        let max_height = c.height() - x_axis_height;
        let unit_w = axis_width / n as f32;
        let bar_w = unit_w * self.bar_width_ratio;
        let bar_margin = (unit_w - bar_w) / 2.0;

        // Label format – {c} = value, {a} = series name (empty here)
        let formatter = if self.series_label_formatter.is_empty() {
            "{c}".to_string()
        } else {
            self.series_label_formatter.clone()
        };

        let mut draw_c = c.child(Box {
            left: y_axis_width,
            ..Default::default()
        });

        let zero_y = y_axis_values.get_offset_height(0.0, max_height);

        for (i, item) in self.data.iter().enumerate() {
            let (bar_bot_val, bar_top_val) = cum[i];

            // Visual top is the larger value (smaller y-pixel)
            let high_val = bar_bot_val.max(bar_top_val);
            let low_val = bar_bot_val.min(bar_top_val);

            let y_high = y_axis_values.get_offset_height(high_val, max_height);
            let y_low = y_axis_values.get_offset_height(low_val, max_height);
            let bar_h = (y_low - y_high).max(1.0);

            let x_left = i as f32 * unit_w + bar_margin;

            let color = if item.is_total {
                self.total_color
            } else if item.value >= 0.0 {
                self.increase_color
            } else {
                self.decrease_color
            };

            draw_c.rect(Rect {
                color: Some(color),
                fill: Some(color),
                left: x_left,
                top: y_high,
                width: bar_w,
                height: bar_h,
                rx: Some(2.0),
                ry: Some(2.0),
                ..Default::default()
            });

            // ── Value label ───────────────────────────────────────────────────
            if self.label_show {
                let label_opt = LabelOption {
                    value: item.value.abs(),
                    formatter: formatter.clone(),
                    ..Default::default()
                };
                let label_text = label_opt.format();
                let label_y = if item.value >= 0.0 || item.is_total {
                    y_high - 4.0 // above bar
                } else {
                    y_low + self.series_label_font_size + 2.0 // below bar
                };
                let mut label_x = x_left + bar_w / 2.0;
                if let Ok(b) = measure_text_width_family(
                    &self.font_family,
                    self.series_label_font_size,
                    &label_text,
                ) {
                    label_x -= b.width() / 2.0;
                }
                draw_c.text(Text {
                    text: label_text,
                    font_family: Some(self.font_family.clone()),
                    font_color: Some(self.series_label_font_color),
                    font_size: Some(self.series_label_font_size),
                    font_weight: self.series_label_font_weight.clone(),
                    x: Some(label_x),
                    y: Some(label_y),
                    ..Default::default()
                });
            }

            // ── Connector line to next bar ────────────────────────────────────
            if self.connector_line_show && i + 1 < n {
                // Connect at the top of the running cumulative after this bar
                let connector_y = y_axis_values.get_offset_height(bar_top_val, max_height);
                let x_right = x_left + bar_w;
                let next_x_left = (i + 1) as f32 * unit_w + bar_margin;

                draw_c.line(Line {
                    color: Some(self.grid_stroke_color),
                    stroke_width: 1.0,
                    stroke_dash_array: Some("4,4".to_string()),
                    left: x_right,
                    top: connector_y,
                    right: next_x_left,
                    bottom: connector_y,
                });
            }
        }

        // Zero baseline (solid line at y=0 when there are negative values)
        let has_negative = self.data.iter().any(|d| d.value < 0.0);
        if has_negative {
            draw_c.line(Line {
                color: Some(self.x_axis_stroke_color),
                stroke_width: 1.0,
                left: 0.0,
                top: zero_y,
                right: axis_width,
                bottom: zero_y,
                ..Default::default()
            });
        }

        c.svg()
    }
}

#[cfg(test)]
mod tests {
    use super::{WaterfallChart, WaterfallData};
    use pretty_assertions::assert_eq;

    fn make_data() -> (Vec<WaterfallData>, Vec<String>) {
        let data = vec![
            (900.0, false).into(),
            (345.0, false).into(),
            (393.0, false).into(),
            (-108.0, false).into(),
            (-154.0, false).into(),
            (135.0, false).into(),
            (-333.0, false).into(),
            (548.0, false).into(),
            (0.0, true).into(), // auto-total
        ];
        let labels = vec![
            "Initial".to_string(),
            "Product Revenue".to_string(),
            "Service Revenue".to_string(),
            "Purchases".to_string(),
            "Marketing".to_string(),
            "Other Income".to_string(),
            "Payroll".to_string(),
            "Other Expenses".to_string(),
            "Profit".to_string(),
        ];
        (data, labels)
    }

    #[test]
    fn waterfall_chart_basic() {
        let (data, labels) = make_data();
        let chart = WaterfallChart::new(data, labels);
        assert_eq!(
            include_str!("../../asset/waterfall_chart/basic.svg"),
            chart.svg().unwrap()
        );
    }

    #[test]
    fn waterfall_chart_basic_json() {
        let chart = WaterfallChart::from_json(
            r#"{
                "title_text": "Waterfall Chart",
                "x_axis_data": ["Initial","Revenue","Services","Purchases","Marketing","Profit"],
                "data": [
                    [900, false],
                    [345, false],
                    [393, false],
                    [-108, false],
                    [-154, false],
                    [0, true]
                ]
            }"#,
        )
        .unwrap();
        assert_eq!(
            include_str!("../../asset/waterfall_chart/basic_json.svg"),
            chart.svg().unwrap()
        );
    }
}
