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

use super::base::ChartBase;
use super::canvas;
use super::color::*;
use super::common::*;
use super::component::*;
use super::params::*;
use super::theme::{get_default_theme_name, get_theme};
use super::util::*;

/// Generate arc approximation points (360 segments per full revolution).
fn arc_points(cx: f32, cy: f32, r: f32, start: f32, end_deg: f32, n: usize) -> Vec<Point> {
    (0..=n)
        .map(|i| {
            let angle = start + i as f32 * (end_deg - start) / n as f32;
            get_pie_point(cx, cy, r, angle)
        })
        .collect()
}

/// A gauge (dial) chart displaying values on a circular scale.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct GaugeChart {
    /// The shared chart options (size, series, title/legend, axes); exposed
    /// directly on the chart through `Deref`, e.g. `chart.title.text`.
    pub base: ChartBase,
    y_axis_configs: Vec<YAxisConfig>,

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

    /// The values where one segment of the scale ends and the next begins.
    /// With thresholds the arc shows the segments, each in a color of its
    /// own, instead of the progress, and a pointer takes the color of the
    /// segment it points at.
    pub thresholds: Vec<f32>,

    /// The colors of the segments, from the lowest to the highest. Default:
    /// the colors of the series.
    pub colors: Vec<Color>,

    /// Draws every series as a ring of progress of its own, one inside the
    /// other, instead of as pointers on one dial.
    pub multi_ring: bool,
}

/// Gap between the rings of a gauge of several rings.
const RING_GAP: f32 = 4.0;

impl std::ops::Deref for GaugeChart {
    type Target = ChartBase;
    fn deref(&self) -> &ChartBase {
        &self.base
    }
}
impl std::ops::DerefMut for GaugeChart {
    fn deref_mut(&mut self) -> &mut ChartBase {
        &mut self.base
    }
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

    /// Creates a gauge chart with the default theme.
    pub fn new(series_list: Vec<Series>) -> GaugeChart {
        GaugeChart::new_with_theme(series_list, &get_default_theme_name())
    }

    /// Creates a gauge chart with the given theme.
    pub fn new_with_theme(series_list: Vec<Series>, theme: &str) -> GaugeChart {
        let mut c = GaugeChart {
            ..Default::default()
        };
        c.series_list = series_list;
        c.base.fill_theme(get_theme(theme), &mut c.y_axis_configs);
        c.fill_default();
        c
    }

    /// Creates a gauge chart from JSON options.
    pub fn from_json(json: &str) -> canvas::Result<GaugeChart> {
        let mut c = GaugeChart {
            ..Default::default()
        };
        let value = c
            .base
            .fill_option(json, &mut c.y_axis_configs, super::schema::GAUGE_FIELDS)?;
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
        if let Some(v) = get_f32_slice_from_value(&value, "thresholds") {
            c.thresholds = v;
        }
        if let Some(v) = get_color_slice_from_value(&value, "colors") {
            c.colors = v;
        }
        if let Some(v) = get_bool_from_value(&value, "multi_ring") {
            c.multi_ring = v;
        }
        c.fill_default();
        Ok(c)
    }

    /// The ends of the segments of the scale that lie on it, in order.
    fn segment_limits(&self) -> Vec<f32> {
        let mut limits: Vec<f32> = self
            .thresholds
            .iter()
            .copied()
            .filter(|v| v.is_finite() && *v > self.min && *v < self.max)
            .collect();
        limits.sort_by(f32::total_cmp);
        limits.dedup();
        limits
    }
    /// The color of the `index`-th segment of the scale.
    fn segment_color(&self, index: usize) -> Color {
        if self.colors.is_empty() {
            get_color(&self.series.colors, index)
        } else {
            self.colors[index % self.colors.len()]
        }
    }
    /// The value of a series as it is written: by the formatter, without
    /// decimals when it has none.
    fn value_text(&self, value: f32) -> String {
        let formatter = if self.value_formatter.is_empty() {
            "{c}"
        } else {
            &self.value_formatter
        };
        if value == value.round() {
            formatter.replace("{c}", &format!("{}", value as i64))
        } else {
            formatter.replace("{c}", &format!("{:.1}", value))
        }
    }
    /// The series that have a value, with their position and that value.
    fn values(&self) -> Vec<(usize, &Series, f32)> {
        self.series_list
            .iter()
            .enumerate()
            .filter_map(|(index, series)| {
                let value = series.iter_values().next().filter(|v| *v != NIL_VALUE)?;
                Some((index, series, value))
            })
            .collect()
    }
    /// Draws every series as a ring of its own: a track over the whole
    /// scale and the progress of the series on it, the outermost ring the
    /// first series; their values are listed in the middle.
    fn render_rings(&self, body: &mut canvas::Canvas, (cx, cy): (f32, f32), r: f32) {
        let values = self.values();
        let (start, sweep) = (self.start_angle, self.sweep_angle);
        // Thin the rings where they would not all fit around the list.
        let count = values.len().max(1) as f32;
        let arc_width = self
            .arc_width
            .min((r * 0.6 - RING_GAP * (count - 1.0)) / count)
            .max(1.0);
        for (ring, (index, _, value)) in values.iter().enumerate() {
            let arc_r = r - arc_width / 2.0 - ring as f32 * (arc_width + RING_GAP);
            body.polyline(Polyline {
                color: Some(self.background_arc_color),
                stroke_width: arc_width,
                points: arc_points(cx, cy, arc_r, start, start + sweep, 360),
            });
            let ratio = if self.max > self.min {
                ((value.clamp(self.min, self.max) - self.min) / (self.max - self.min))
                    .clamp(0.0, 1.0)
            } else {
                0.0
            };
            if ratio > 0.0 {
                let pieces = ((360.0 * ratio).round() as usize).max(1);
                body.polyline(Polyline {
                    color: Some(get_color(&self.series.colors, *index)),
                    stroke_width: arc_width,
                    points: arc_points(cx, cy, arc_r, start, start + ratio * sweep, pieces),
                });
            }
        }
        // The values, one line each, centered on the middle of the rings.
        let font_size = self.series.label.font.size;
        let line_height = font_size + 6.0;
        let top = cy - line_height * (values.len() as f32 - 1.0) / 2.0;
        for (line, (index, series, value)) in values.iter().enumerate() {
            let text = if series.name.is_empty() {
                self.value_text(*value)
            } else {
                format!("{} {}", series.name, self.value_text(*value))
            };
            body.text(Text {
                text,
                font_family: Some(self.font_family.clone()),
                font_color: Some(get_color(&self.series.colors, *index)),
                font_size: Some(font_size),
                font_weight: Some("bold".to_string()),
                dominant_baseline: Some("middle".to_string()),
                text_anchor: Some("middle".to_string()),
                x: Some(cx),
                y: Some(top + line_height * line as f32),
                ..Default::default()
            });
        }
    }

    /// Renders the chart to an SVG string.
    pub fn svg(&self) -> canvas::Result<String> {
        let mut c = self.new_canvas();
        let axis_top = self.render_header(&mut c);

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

        if self.multi_ring {
            self.render_rings(&mut body, (cx, cy), r);
            return c.svg();
        }

        let start = self.start_angle;
        let sweep = self.sweep_angle;
        let end = start + sweep;

        // ── Value extraction ──────────────────────────────────────────────────
        // The label shows the value as given; only the needle is clamped to
        // the dial so an out-of-range reading is still legible as such.
        let raw_value = self
            .series_list
            .first()
            .and_then(|s| s.iter_values().next())
            .filter(|v| *v != NIL_VALUE)
            .unwrap_or(self.min);
        let needle_value = raw_value.clamp(self.min, self.max);

        let ratio = if self.max > self.min {
            (needle_value - self.min) / (self.max - self.min)
        } else {
            0.0
        };
        let value_angle = start + ratio * sweep;

        // arc center radius (middle of the stroke)
        let arc_r = r - self.arc_width / 2.0;

        // ── Background arc (full sweep, many polyline segments) ───────────────
        let n_full = 360_usize;
        // Where a value lies along the scale, from 0 to 1.
        let share = |value: f32| -> f32 {
            if self.max > self.min {
                ((value - self.min) / (self.max - self.min)).clamp(0.0, 1.0)
            } else {
                0.0
            }
        };
        let limits = self.segment_limits();
        // The color of the segment `value` lies in.
        let color_at = |value: f32| -> Color {
            self.segment_color(limits.iter().filter(|limit| value >= **limit).count())
        };
        // The other series of a dial with several pointers.
        let others: Vec<(usize, &Series, f32)> = self.values().into_iter().skip(1).collect();
        if limits.is_empty() {
            let bg_pts = arc_points(cx, cy, arc_r, start, end, n_full);
            body.polyline(Polyline {
                color: Some(self.background_arc_color),
                stroke_width: self.arc_width,
                points: bg_pts,
            });
        } else {
            // The scale in its segments, each from one limit to the next.
            let mut from = self.min;
            for (index, to) in limits.iter().copied().chain([self.max]).enumerate() {
                let (a, b) = (start + share(from) * sweep, start + share(to) * sweep);
                let pieces = ((n_full as f32 * (share(to) - share(from))).round() as usize).max(1);
                body.polyline(Polyline {
                    color: Some(self.segment_color(index)),
                    stroke_width: self.arc_width,
                    points: arc_points(cx, cy, arc_r, a, b, pieces),
                });
                from = to;
            }
        }

        // ── Progress arc ──────────────────────────────────────────────────────
        let progress_color = if !self.pointer_color.is_zero() {
            self.pointer_color
        } else if !limits.is_empty() {
            // The pointer tells the segment it is in.
            color_at(needle_value)
        } else {
            get_color(&self.series.colors, 0)
        };

        // The progress of the one value of the dial: segments, and several
        // pointers, leave the arc to the scale.
        if ratio > 0.0 && limits.is_empty() && others.is_empty() {
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
            let label_font_size = self.series.label.font.size;
            let label_color = self.series.label.font.color;
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
                ..Default::default()
            });
            // A pointer for each of the other series, in its color (or
            // that of its segment).
            for (index, _, value) in others.iter() {
                let value = value.clamp(self.min, self.max);
                let angle = start + share(value) * sweep;
                let color = if limits.is_empty() {
                    get_color(&self.series.colors, *index)
                } else {
                    color_at(value)
                };
                body.polygon(Polygon {
                    color: Some(color),
                    fill: Some(color),
                    points: vec![
                        get_pie_point(cx, cy, needle_len, angle),
                        get_pie_point(cx, cy, base_offset, angle + 90.0),
                        get_pie_point(cx, cy, base_offset, angle - 90.0),
                    ],
                    ..Default::default()
                });
            }

            // Center hub circle
            body.circle(Circle {
                cx,
                cy,
                r: base_offset + 2.0,
                stroke_color: Some(progress_color),
                fill: Some(progress_color),
                stroke_width: 1.0,
                ..Default::default()
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

        // Several pointers: their values in a list, each in the color of
        // its pointer, instead of the one large value.
        if !others.is_empty() {
            let font_size = self.series.label.font.size;
            let line_height = font_size + 4.0;
            let first = self
                .series_list
                .first()
                .map(|series| (0, series, raw_value))
                .into_iter();
            for (line, (index, series, value)) in first.chain(others.iter().copied()).enumerate() {
                let text = if series.name.is_empty() {
                    self.value_text(value)
                } else {
                    format!("{}: {}", series.name, self.value_text(value))
                };
                let color = if index == 0 {
                    progress_color
                } else if limits.is_empty() {
                    get_color(&self.series.colors, index)
                } else {
                    color_at(value.clamp(self.min, self.max))
                };
                body.text(Text {
                    text,
                    font_family: Some(self.font_family.clone()),
                    font_color: Some(color),
                    font_size: Some(font_size),
                    font_weight: Some("bold".to_string()),
                    dominant_baseline: Some("middle".to_string()),
                    text_anchor: Some("middle".to_string()),
                    x: Some(cx),
                    y: Some(detail_y - line_height + line_height * line as f32),
                    ..Default::default()
                });
            }
            return c.svg();
        }

        body.text(Text {
            text: value_text,
            font_family: Some(self.font_family.clone()),
            font_color: Some(self.title.font.color),
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
                font_color: Some(self.series.label.font.color),
                font_size: Some(self.series.label.font.size),
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

    #[test]
    fn gauge_chart_basic() {
        let chart = GaugeChart::new(vec![("Speed", vec![75.0]).into()]);
        assert_snapshot!("gauge_chart/basic.svg", chart.svg().unwrap());
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
        assert_snapshot!("gauge_chart/basic_json.svg", chart.svg().unwrap());
    }
}
