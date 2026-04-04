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

#[derive(Clone, Debug, Default, Chart)]
pub struct FunnelChart {
    pub width: f32,
    pub height: f32,
    pub x: f32,
    pub y: f32,
    pub margin: Box,
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

    // legend
    pub legend_font_size: f32,
    pub legend_font_color: Color,
    pub legend_font_weight: Option<String>,
    pub legend_align: Align,
    pub legend_margin: Option<Box>,
    pub legend_category: LegendCategory,
    pub legend_show: Option<bool>,

    // x axis (required by derive – not rendered)
    pub x_axis_data: Vec<String>,
    pub x_axis_height: f32,
    pub x_axis_stroke_color: Color,
    pub x_axis_font_size: f32,
    pub x_axis_font_color: Color,
    pub x_axis_font_weight: Option<String>,
    pub x_axis_name_gap: f32,
    pub x_axis_name_rotate: f32,
    pub x_axis_margin: Option<Box>,
    pub x_boundary_gap: Option<bool>,

    // y axis (required by derive)
    y_axis_configs: Vec<YAxisConfig>,

    // grid (required by derive)
    grid_stroke_color: Color,
    grid_stroke_width: f32,

    // series (required by derive)
    pub series_stroke_width: f32,
    pub series_label_font_color: Color,
    pub series_label_font_size: f32,
    pub series_label_font_weight: Option<String>,
    pub series_label_formatter: String,
    /// Label position: `"inside"`, `"left"`, or `"right"` (default).
    pub series_label_position: Option<String>,
    pub series_colors: Vec<Color>,
    pub series_symbol: Option<Symbol>,
    pub series_smooth: bool,
    pub series_fill: bool,

    // ── Funnel-specific fields ────────────────────────────────────────────────
    /// Vertical gap between trapezoids in pixels (default: 2).
    pub funnel_gap: f32,

    /// Horizontal alignment of all trapezoids (default: `Center`).
    pub funnel_align: Align,

    /// If `true`, smallest value at top; if `false` (default), largest at top.
    pub sort_ascending: bool,

    /// Minimum trapezoid width for the narrowest end, in pixels (default: 20).
    pub min_width: f32,
}

impl FunnelChart {
    fn fill_default(&mut self) {
        if self.funnel_gap <= 0.0 {
            self.funnel_gap = 2.0;
        }
        if self.min_width <= 0.0 {
            self.min_width = 20.0;
        }
        // default label position
        if self.series_label_position.is_none() {
            self.series_label_position = Some("right".to_string());
        }
    }

    /// Creates a funnel chart with default theme.
    pub fn new(series_list: Vec<Series>) -> FunnelChart {
        FunnelChart::new_with_theme(series_list, &get_default_theme_name())
    }

    /// Creates a funnel chart with a custom theme.
    pub fn new_with_theme(series_list: Vec<Series>, theme: &str) -> FunnelChart {
        let mut c = FunnelChart {
            series_list,
            ..Default::default()
        };
        c.fill_theme(get_theme(theme));
        c.fill_default();
        c
    }

    /// Creates a funnel chart from a JSON string.
    pub fn from_json(json: &str) -> canvas::Result<FunnelChart> {
        let mut c = FunnelChart {
            ..Default::default()
        };
        let value = c.fill_option(json)?;
        if let Some(v) = get_f32_from_value(&value, "funnel_gap") {
            c.funnel_gap = v;
        }
        if let Some(v) = get_f32_from_value(&value, "min_width") {
            c.min_width = v;
        }
        if let Some(b) = get_bool_from_value(&value, "sort_ascending") {
            c.sort_ascending = b;
        }
        if let Some(s) = get_string_from_value(&value, "series_label_position") {
            c.series_label_position = Some(s);
        }
        if let Some(a) = get_align_from_value(&value, "funnel_align") {
            c.funnel_align = a;
        }
        c.fill_default();
        Ok(c)
    }

    /// Renders the funnel chart to an SVG string.
    pub fn svg(&self) -> canvas::Result<String> {
        if self.series_list.is_empty() {
            return Err(canvas::Error::Params {
                message: "series_list is empty".to_string(),
            });
        }

        let mut c = Canvas::new_width_xy(self.width, self.height, self.x, self.y);
        self.render_background(c.child(Box::default()));
        c.margin = self.margin.clone();

        let title_height = self.render_title(c.child(Box::default()));
        let legend_height = self.render_legend(c.child(Box::default()));
        let axis_top = title_height.max(legend_height);

        if axis_top > 0.0 {
            c = c.child(Box {
                top: axis_top,
                ..Default::default()
            });
        }

        let funnel_width = c.width();
        let funnel_height = c.height();

        // ── Collect & sort series ─────────────────────────────────────────────
        // Tuple: (color_index, value, name).
        let mut stages: Vec<(usize, f32, String)> = self
            .series_list
            .iter()
            .enumerate()
            .map(|(i, s)| {
                let val: f32 = s.data.iter().copied().sum();
                (s.index.unwrap_or(i), val, s.name.clone())
            })
            .collect();

        if self.sort_ascending {
            stages.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
        } else {
            stages.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        }

        let max_val = stages
            .iter()
            .map(|(_, v, _)| *v)
            .fold(f32::NEG_INFINITY, f32::max);
        if max_val <= 0.0 {
            return c.svg();
        }
        let total: f32 = stages.iter().map(|(_, v, _)| *v).sum();

        let n = stages.len();
        let gap = self.funnel_gap;
        let stage_h = (funnel_height - (n as f32 - 1.0) * gap) / n as f32;

        let label_pos = self.series_label_position.as_deref().unwrap_or("right");
        let label_font_size = self.series_label_font_size;
        let label_color = self.series_label_font_color;
        let mut formatter = self.series_label_formatter.clone();
        if formatter.is_empty() {
            formatter = "{a}: {c}".to_string();
        }

        for (stage_idx, (color_idx, val, name)) in stages.iter().enumerate() {
            let top_w = (val / max_val) * funnel_width;
            // bottom width = next stage's width (or min_width for last stage)
            let bot_w = if stage_idx + 1 < n {
                let next_val = stages[stage_idx + 1].1;
                ((next_val / max_val) * funnel_width).max(self.min_width)
            } else {
                self.min_width
            };

            let y_top = stage_idx as f32 * (stage_h + gap);
            let y_bot = y_top + stage_h;

            // horizontal offset per alignment
            let (x_left_top, x_left_bot) = match self.funnel_align {
                Align::Left => (0.0, 0.0),
                Align::Right => (funnel_width - top_w, funnel_width - bot_w),
                _ => ((funnel_width - top_w) / 2.0, (funnel_width - bot_w) / 2.0),
            };
            let x_right_top = x_left_top + top_w;
            let x_right_bot = x_left_bot + bot_w;

            let color = get_color(&self.series_colors, *color_idx);

            c.polygon(Polygon {
                color: Some(color),
                fill: Some(color),
                points: vec![
                    (x_left_top, y_top).into(),
                    (x_right_top, y_top).into(),
                    (x_right_bot, y_bot).into(),
                    (x_left_bot, y_bot).into(),
                ],
            });

            let label_option = LabelOption {
                series_name: name.clone(),
                value: *val,
                percentage: if total > 0.0 { val / total } else { 0.0 },
                formatter: formatter.clone(),
                ..Default::default()
            };
            let label_text = label_option.format();

            let mid_y = (y_top + y_bot) / 2.0;

            match label_pos {
                "inside" => {
                    // Center text inside the trapezoid
                    let mid_x = (x_left_top + x_right_top) / 2.0;
                    let mut text_x = mid_x;
                    if let Ok(b) =
                        measure_text_width_family(&self.font_family, label_font_size, &label_text)
                    {
                        text_x -= b.width() / 2.0;
                    }
                    c.text(Text {
                        text: label_text,
                        font_family: Some(self.font_family.clone()),
                        font_color: Some(label_color),
                        font_size: Some(label_font_size),
                        font_weight: self.series_label_font_weight.clone(),
                        dominant_baseline: Some("central".to_string()),
                        x: Some(text_x),
                        y: Some(mid_y),
                        ..Default::default()
                    });
                }
                "left" => {
                    let x_edge = x_left_top.min(x_left_bot) - 5.0;
                    let mut text_x = x_edge;
                    if let Ok(b) =
                        measure_text_width_family(&self.font_family, label_font_size, &label_text)
                    {
                        text_x -= b.width();
                    }
                    c.text(Text {
                        text: label_text,
                        font_family: Some(self.font_family.clone()),
                        font_color: Some(label_color),
                        font_size: Some(label_font_size),
                        font_weight: self.series_label_font_weight.clone(),
                        dominant_baseline: Some("central".to_string()),
                        x: Some(text_x.max(0.0)),
                        y: Some(mid_y),
                        ..Default::default()
                    });
                }
                _ => {
                    // "right" (default): to the right of the widest edge
                    let x_edge = x_right_top.max(x_right_bot) + 5.0;
                    c.text(Text {
                        text: label_text,
                        font_family: Some(self.font_family.clone()),
                        font_color: Some(label_color),
                        font_size: Some(label_font_size),
                        font_weight: self.series_label_font_weight.clone(),
                        dominant_baseline: Some("central".to_string()),
                        x: Some(x_edge),
                        y: Some(mid_y),
                        ..Default::default()
                    });
                }
            }
        }

        c.svg()
    }
}

#[cfg(test)]
mod tests {
    use super::FunnelChart;
    use crate::Series;
    use pretty_assertions::assert_eq;

    fn make_series() -> Vec<Series> {
        vec![
            ("Impression", vec![60000.0]).into(),
            ("Click", vec![40000.0]).into(),
            ("Inquiry", vec![20000.0]).into(),
            ("Order", vec![8000.0]).into(),
            ("Re-order", vec![2000.0]).into(),
        ]
    }

    #[test]
    fn funnel_chart_basic() {
        let chart = FunnelChart::new(make_series());
        assert_eq!(
            include_str!("../../asset/funnel_chart/basic.svg"),
            chart.svg().unwrap()
        );
    }

    #[test]
    fn funnel_chart_inside_label() {
        let mut chart = FunnelChart::new(make_series());
        chart.title_text = "Conversion Funnel".to_string();
        chart.series_label_position = Some("inside".to_string());
        assert_eq!(
            include_str!("../../asset/funnel_chart/inside_label.svg"),
            chart.svg().unwrap()
        );
    }

    #[test]
    fn funnel_chart_basic_json() {
        let chart = FunnelChart::from_json(
            r##"{
                "title_text": "Funnel Chart",
                "series_label_position": "inside",
                "funnel_gap": 4,
                "series_list": [
                    {"name": "Impression", "data": [60000]},
                    {"name": "Click",      "data": [40000]},
                    {"name": "Inquiry",    "data": [20000]},
                    {"name": "Order",      "data": [8000]},
                    {"name": "Re-order",   "data": [2000]}
                ]
            }"##,
        )
        .unwrap();
        assert_eq!(
            include_str!("../../asset/funnel_chart/basic_json.svg"),
            chart.svg().unwrap()
        );
    }
}
