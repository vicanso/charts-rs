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

use super::Canvas;
use super::canvas;
use super::color::*;
use super::common::*;
use super::component::*;
use super::params::*;
use super::theme::{DEFAULT_Y_AXIS_WIDTH, Theme, get_default_theme_name, get_theme};
use super::util::*;
use crate::charts::measure_text_width_family;
use charts_rs_derive::Chart;
use std::sync::Arc;

/// Generate arc approximation points (360 segments per full revolution).
fn arc_points(cx: f32, cy: f32, r: f32, start: f32, end_deg: f32, n: usize) -> Vec<Point> {
    (0..=n)
        .map(|i| {
            let angle = start + i as f32 * (end_deg - start) / n as f32;
            get_pie_point(cx, cy, r, angle)
        })
        .collect()
}

#[derive(Clone, Debug, Default, Chart)]
pub struct GaugeChart {
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

    // x/y axis — required by derive but not rendered
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
    y_axis_configs: Vec<YAxisConfig>,
    grid_stroke_color: Color,
    grid_stroke_width: f32,

    // series — required by derive
    pub series_stroke_width: f32,
    pub series_label_font_color: Color,
    pub series_label_font_size: f32,
    pub series_label_font_weight: Option<String>,
    pub series_label_formatter: String,
    pub series_colors: Vec<Color>,
    pub series_symbol: Option<Symbol>,
    pub series_smooth: bool,
    pub series_fill: bool,

    // ── Gauge-specific ──────────────────────────────────────────────────────
    /// Minimum value on the scale (default: 0).
    pub min: f32,
    /// Maximum value on the scale (default: 100).
    pub max: f32,

    /// Start angle in degrees, measured clockwise from 12 o'clock (default: 225).
    /// E.g. 225 = bottom-left (7:30 position).
    pub start_angle: f32,

    /// Total sweep in degrees clockwise (default: 270).
    pub sweep_angle: f32,

    /// Outer radius in pixels.  0 = auto-computed from available space (default: 0).
    pub radius: f32,

    /// Thickness of the gauge arc in pixels (default: 15).
    pub arc_width: f32,

    /// Color of the unfilled portion of the arc.  Defaults to a light gray.
    pub background_arc_color: Color,

    /// Whether to draw the needle pointer (default: true stored as None → true).
    pub show_pointer: Option<bool>,

    /// Color of the needle.  0 = use first series color.
    pub pointer_color: Color,

    /// Whether to draw min/max labels at the arc ends (default: true stored as None → true).
    pub show_axis_label: Option<bool>,

    /// Number of major tick divisions (default: 5).
    pub split_number: usize,

    /// Value label formatter.  `{c}` is replaced with the current value (default: "{c}").
    pub value_formatter: String,
}

impl GaugeChart {
    fn fill_default(&mut self) {
        if self.max <= self.min {
            self.max = self.min + 100.0;
        }
        if self.start_angle == 0.0 {
            self.start_angle = 225.0;
        }
        if self.sweep_angle <= 0.0 {
            self.sweep_angle = 270.0;
        }
        if self.arc_width <= 0.0 {
            self.arc_width = 15.0;
        }
        if self.background_arc_color.is_zero() {
            self.background_arc_color = (230, 230, 230).into();
        }
        if self.split_number == 0 {
            self.split_number = 5;
        }
    }

    pub fn new(series_list: Vec<Series>) -> GaugeChart {
        GaugeChart::new_with_theme(series_list, &get_default_theme_name())
    }

    pub fn new_with_theme(series_list: Vec<Series>, theme: &str) -> GaugeChart {
        let mut c = GaugeChart {
            series_list,
            ..Default::default()
        };
        c.fill_theme(get_theme(theme));
        c.fill_default();
        c
    }

    pub fn from_json(json: &str) -> canvas::Result<GaugeChart> {
        let mut c = GaugeChart {
            ..Default::default()
        };
        let value = c.fill_option(json)?;
        if let Some(v) = get_f32_from_value(&value, "min") {
            c.min = v;
        }
        if let Some(v) = get_f32_from_value(&value, "max") {
            c.max = v;
        }
        if let Some(v) = get_f32_from_value(&value, "start_angle") {
            c.start_angle = v;
        }
        if let Some(v) = get_f32_from_value(&value, "sweep_angle") {
            c.sweep_angle = v;
        }
        if let Some(v) = get_f32_from_value(&value, "radius") {
            c.radius = v;
        }
        if let Some(v) = get_f32_from_value(&value, "arc_width") {
            c.arc_width = v;
        }
        if let Some(v) = get_color_from_value(&value, "background_arc_color") {
            c.background_arc_color = v;
        }
        if let Some(v) = get_bool_from_value(&value, "show_pointer") {
            c.show_pointer = Some(v);
        }
        if let Some(v) = get_color_from_value(&value, "pointer_color") {
            c.pointer_color = v;
        }
        if let Some(v) = get_bool_from_value(&value, "show_axis_label") {
            c.show_axis_label = Some(v);
        }
        if let Some(v) = get_usize_from_value(&value, "split_number") {
            c.split_number = v;
        }
        if let Some(v) = get_string_from_value(&value, "value_formatter") {
            c.value_formatter = v;
        }
        c.fill_default();
        Ok(c)
    }

    pub fn svg(&self) -> canvas::Result<String> {
        let mut c = Canvas::new_width_xy(self.width, self.height, self.x, self.y);
        self.render_background(c.child(Box::default()));
        c.margin = self.margin.clone();

        let title_height = self.render_title(c.child(Box::default()));
        let legend_height = self.render_legend(c.child(Box::default()));
        let axis_top = title_height.max(legend_height);

        let mut body = if axis_top > 0.0 {
            c.child(Box {
                top: axis_top,
                ..Default::default()
            })
        } else {
            c.child(Box::default())
        };

        let avail_w = body.width();
        let avail_h = body.height();

        // ── Gauge geometry ────────────────────────────────────────────────────
        let r = if self.radius > 0.0 {
            self.radius
        } else {
            (avail_w.min(avail_h) * 0.5 - self.arc_width).max(10.0)
        };

        let cx = avail_w / 2.0;
        let cy = avail_h / 2.0;

        let start = self.start_angle;
        let sweep = self.sweep_angle;
        let end = start + sweep;

        // ── Value extraction ──────────────────────────────────────────────────
        let raw_value = self
            .series_list
            .first()
            .and_then(|s| s.data_values().first().copied())
            .unwrap_or(self.min)
            .clamp(self.min, self.max);

        let ratio = if self.max > self.min {
            (raw_value - self.min) / (self.max - self.min)
        } else {
            0.0
        };
        let value_angle = start + ratio * sweep;

        // arc center radius (middle of the stroke)
        let arc_r = r - self.arc_width / 2.0;

        // ── Background arc (full sweep, many polyline segments) ───────────────
        let n_full = 360_usize;
        let bg_pts = arc_points(cx, cy, arc_r, start, end, n_full);
        body.polyline(Polyline {
            color: Some(self.background_arc_color),
            stroke_width: self.arc_width,
            points: bg_pts,
        });

        // ── Progress arc ──────────────────────────────────────────────────────
        let progress_color = if !self.pointer_color.is_zero() {
            self.pointer_color
        } else {
            get_color(&self.series_colors, 0)
        };

        if ratio > 0.0 {
            let n_prog = (n_full as f32 * ratio).round() as usize;
            let n_prog = n_prog.max(1);
            let prog_pts = arc_points(cx, cy, arc_r, start, value_angle, n_prog);
            body.polyline(Polyline {
                color: Some(progress_color),
                stroke_width: self.arc_width,
                points: prog_pts,
            });
        }

        // ── Tick marks ────────────────────────────────────────────────────────
        let tick_color = self.background_arc_color;
        for i in 0..=self.split_number {
            let tick_angle = start + i as f32 * sweep / self.split_number as f32;
            let outer = get_pie_point(cx, cy, r + 2.0, tick_angle);
            let inner = get_pie_point(cx, cy, r - self.arc_width - 2.0, tick_angle);
            body.line(Line {
                color: Some(tick_color),
                stroke_width: 2.0,
                left: inner.x,
                top: inner.y,
                right: outer.x,
                bottom: outer.y,
                ..Default::default()
            });
        }

        // ── Axis labels (min / max) ───────────────────────────────────────────
        let show_label = self.show_axis_label.unwrap_or(true);
        if show_label {
            let label_font_size = self.series_label_font_size;
            let label_color = self.series_label_font_color;
            let label_r = r + self.arc_width + 6.0;

            let min_pt = get_pie_point(cx, cy, label_r, start);
            let max_pt = get_pie_point(cx, cy, label_r, end);

            let fmt = |v: f32| -> String {
                if v == v.round() {
                    format!("{}", v as i64)
                } else {
                    format!("{:.1}", v)
                }
            };

            body.text(Text {
                text: fmt(self.min),
                font_family: Some(self.font_family.clone()),
                font_color: Some(label_color),
                font_size: Some(label_font_size),
                dominant_baseline: Some("middle".to_string()),
                text_anchor: Some("middle".to_string()),
                x: Some(min_pt.x),
                y: Some(min_pt.y),
                ..Default::default()
            });
            body.text(Text {
                text: fmt(self.max),
                font_family: Some(self.font_family.clone()),
                font_color: Some(label_color),
                font_size: Some(label_font_size),
                dominant_baseline: Some("middle".to_string()),
                text_anchor: Some("middle".to_string()),
                x: Some(max_pt.x),
                y: Some(max_pt.y),
                ..Default::default()
            });

            // Tick value labels
            if self.split_number > 0 {
                for i in 1..self.split_number {
                    let tick_angle = start + i as f32 * sweep / self.split_number as f32;
                    let tick_v =
                        self.min + i as f32 * (self.max - self.min) / self.split_number as f32;
                    let pt = get_pie_point(cx, cy, label_r, tick_angle);
                    body.text(Text {
                        text: fmt(tick_v),
                        font_family: Some(self.font_family.clone()),
                        font_color: Some(label_color),
                        font_size: Some(label_font_size),
                        dominant_baseline: Some("middle".to_string()),
                        text_anchor: Some("middle".to_string()),
                        x: Some(pt.x),
                        y: Some(pt.y),
                        ..Default::default()
                    });
                }
            }
        }

        // ── Pointer / needle ──────────────────────────────────────────────────
        let show_ptr = self.show_pointer.unwrap_or(true);
        if show_ptr {
            let needle_len = r - self.arc_width - 8.0;
            let base_offset = 6.0;

            let tip = get_pie_point(cx, cy, needle_len, value_angle);
            let base_l = get_pie_point(cx, cy, base_offset, value_angle + 90.0);
            let base_r = get_pie_point(cx, cy, base_offset, value_angle - 90.0);

            body.polygon(Polygon {
                color: Some(progress_color),
                fill: Some(progress_color),
                points: vec![tip, base_l, base_r],
            });

            // Center hub circle
            body.circle(Circle {
                cx,
                cy,
                r: base_offset + 2.0,
                stroke_color: Some(progress_color),
                fill: Some(progress_color),
                stroke_width: 1.0,
            });
        }

        // ── Value label ───────────────────────────────────────────────────────
        let value_font_size = (r * 0.25).clamp(14.0, 36.0);
        let formatter = if self.value_formatter.is_empty() {
            "{c}".to_string()
        } else {
            self.value_formatter.clone()
        };
        let value_text = if raw_value == raw_value.round() {
            formatter.replace("{c}", &format!("{}", raw_value as i64))
        } else {
            formatter.replace("{c}", &format!("{:.1}", raw_value))
        };

        // Place the detail label at ~40% radius below center
        let detail_y = cy + r * 0.55;

        body.text(Text {
            text: value_text,
            font_family: Some(self.font_family.clone()),
            font_color: Some(self.title_font_color),
            font_size: Some(value_font_size),
            font_weight: Some("bold".to_string()),
            dominant_baseline: Some("middle".to_string()),
            text_anchor: Some("middle".to_string()),
            x: Some(cx),
            y: Some(detail_y),
            ..Default::default()
        });

        // Series name label below the value
        if let Some(series) = self.series_list.first()
            && !series.name.is_empty()
        {
            body.text(Text {
                text: series.name.clone(),
                font_family: Some(self.font_family.clone()),
                font_color: Some(self.series_label_font_color),
                font_size: Some(self.series_label_font_size),
                dominant_baseline: Some("middle".to_string()),
                text_anchor: Some("middle".to_string()),
                x: Some(cx),
                y: Some(detail_y + value_font_size + 4.0),
                ..Default::default()
            });
        }

        c.svg()
    }
}

#[cfg(test)]
mod tests {
    use super::GaugeChart;
    use pretty_assertions::assert_eq;

    #[test]
    fn gauge_chart_basic() {
        let chart = GaugeChart::new(vec![("Speed", vec![75.0]).into()]);
        assert_eq!(
            include_str!("../../asset/gauge_chart/basic.svg"),
            chart.svg().unwrap()
        );
    }

    #[test]
    fn gauge_chart_basic_json() {
        let chart = GaugeChart::from_json(
            r##"{
                "title_text": "Gauge",
                "min": 0,
                "max": 200,
                "series_list": [{"name": "Speed", "data": [120]}]
            }"##,
        )
        .unwrap();
        assert_eq!(
            include_str!("../../asset/gauge_chart/basic_json.svg"),
            chart.svg().unwrap()
        );
    }
}
