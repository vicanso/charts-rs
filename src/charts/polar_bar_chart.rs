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

use super::base::{ChartBase, LabelBoxes, axis_value_params, get_y_axis_config};
use super::canvas;
use super::color::*;
use super::common::*;
use super::component::*;
use super::params::*;
use super::theme::{get_default_theme_name, get_theme};
use super::util::*;
use crate::charts::measure_text_width_family;
use serde::{Deserialize, Serialize};

/// Gap between the plot and the labels around it.
const OUTER_LABEL_GAP: f32 = 6.0;
/// Gap between an axis ray or a bar end and the label beside it.
const SIDE_LABEL_GAP: f32 = 5.0;
/// Angle the value axis spans when the categories lie on the radius: three
/// quarters of the circle, the last one left to the category labels.
const DEFAULT_SPAN: f32 = 270.0;
/// Alpha of the backdrop that keeps the labels along the start ray legible
/// over the bars and the grid behind them.
const BACKDROP_ALPHA: u8 = 200;

/// The axis of a polar chart that carries its categories.
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum PolarAxis {
    /// The categories go around the circle and the bars grow outwards from
    /// the center.
    #[default]
    Angle,
    /// The categories go from the center outwards, one ring each, and the
    /// bars run around the circle.
    Radius,
}

/// One bar: a value of a series, on top of what its stack holds so far.
struct Bar {
    series: usize,
    /// Index into the data of the series (for its per-bar colors).
    index: usize,
    category: usize,
    /// The slot within the category: one per stack, one per other series.
    group: usize,
    from: f32,
    to: f32,
    value: f32,
}

/// A label next to a point: where it goes and how it is anchored.
pub(crate) struct Beside {
    pub x: f32,
    pub y: f32,
    pub anchor: &'static str,
}

impl Beside {
    /// The label beside `point`, `gap` pixels away in the direction of the
    /// unit vector `(dx, dy)`. Sideways it starts or ends at that spot;
    /// straight up or down it is centered on it, half its height further.
    pub(crate) fn new(point: Point, (dx, dy): (f32, f32), gap: f32, height: f32) -> Beside {
        let anchor = if dx > 0.3 {
            "start"
        } else if dx < -0.3 {
            "end"
        } else {
            "middle"
        };
        let lift = if anchor == "middle" {
            dy.signum() * height / 2.0
        } else {
            0.0
        };
        Beside {
            x: point.x + dx * gap,
            y: point.y + dy * gap + lift,
            anchor,
        }
    }
    /// Left edge of a label `width` wide.
    pub(crate) fn left(&self, width: f32) -> f32 {
        match self.anchor {
            "start" => self.x,
            "end" => self.x - width,
            _ => self.x - width / 2.0,
        }
    }
}

/// The direction of growing angles (clockwise) at `angle`, as a unit vector.
fn tangent(angle: f32) -> (f32, f32) {
    let a = angle.to_radians();
    (a.cos(), a.sin())
}

/// The direction away from the center at `angle`, as a unit vector.
pub(crate) fn outward(angle: f32) -> (f32, f32) {
    let a = angle.to_radians();
    (a.sin(), -a.cos())
}

/// A bar chart on polar axes.
///
/// With the categories on the angle axis (the default) every category gets
/// a slice of the circle and its bars grow outwards — a radial column chart.
/// With the categories on the radius axis every category gets a ring and
/// its bars run around the circle — a radial bar chart.
///
/// The categories are `x_axis_data`; series are drawn side by side within a
/// category, or on top of each other when they share a `stack` name.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct PolarBarChart {
    /// The shared chart options (size, series, title/legend, axes); exposed
    /// directly on the chart through `Deref`, e.g. `chart.title_text`.
    pub base: ChartBase,
    /// Configuration of the value axis: its range, split number and label
    /// format.
    pub y_axis_configs: Vec<YAxisConfig>,

    // polar-specific
    /// The axis the categories lie on.
    pub category_axis: PolarAxis,
    /// Largest outer radius; `None` (the default) fills the plot area.
    pub radius: Option<f32>,
    /// Radius of the hole in the center. Default: 0 with the categories on
    /// the angle axis, a quarter of the radius with them on the radius axis.
    pub inner_radius: Option<f32>,
    /// Angle both axes start at, in degrees clockwise from 12 o'clock.
    pub start_angle: f32,
    /// Angle the value axis ends at, with the categories on the radius axis.
    /// Default: 270 degrees after `start_angle`; at most a full turn.
    pub end_angle: Option<f32>,
    /// Rounds the ends of the bars that run around the circle.
    pub round_cap: bool,
    /// Share of a category's slot left free between it and its neighbours,
    /// from 0 to 0.9. Default: 0.2.
    pub category_gap: Option<f32>,
}

impl std::ops::Deref for PolarBarChart {
    type Target = ChartBase;
    fn deref(&self) -> &ChartBase {
        &self.base
    }
}
impl std::ops::DerefMut for PolarBarChart {
    fn deref_mut(&mut self) -> &mut ChartBase {
        &mut self.base
    }
}

impl PolarBarChart {
    /// Creates a polar bar chart with the default theme.
    pub fn new(series_list: Vec<Series>, x_axis_data: Vec<String>) -> PolarBarChart {
        PolarBarChart::new_with_theme(series_list, x_axis_data, &get_default_theme_name())
    }

    /// Creates a polar bar chart with a custom theme.
    pub fn new_with_theme(
        series_list: Vec<Series>,
        x_axis_data: Vec<String>,
        theme: &str,
    ) -> PolarBarChart {
        let mut c = PolarBarChart {
            ..Default::default()
        };
        c.series_list = series_list;
        c.x_axis_data = x_axis_data;
        c.base.fill_theme(get_theme(theme), &mut c.y_axis_configs);
        c
    }

    /// Creates a polar bar chart from a JSON string.
    pub fn from_json(json: &str) -> canvas::Result<PolarBarChart> {
        let mut c = PolarBarChart {
            ..Default::default()
        };
        let value =
            c.base
                .fill_option(json, &mut c.y_axis_configs, super::schema::POLAR_BAR_FIELDS)?;
        if let Some(axis) = get_string_from_value(&value, "category_axis") {
            c.category_axis = if axis.eq_ignore_ascii_case("radius") {
                PolarAxis::Radius
            } else {
                PolarAxis::Angle
            };
        }
        if let Some(v) = get_f32_from_value(&value, "radius") {
            c.radius = Some(v);
        }
        if let Some(v) = get_f32_from_value(&value, "inner_radius") {
            c.inner_radius = Some(v);
        }
        if let Some(v) = get_f32_from_value(&value, "start_angle") {
            c.start_angle = v;
        }
        if let Some(v) = get_f32_from_value(&value, "end_angle") {
            c.end_angle = Some(v);
        }
        if let Some(v) = get_bool_from_value(&value, "round_cap") {
            c.round_cap = v;
        }
        if let Some(v) = get_f32_from_value(&value, "category_gap") {
            c.category_gap = Some(v);
        }
        Ok(c)
    }

    /// The bars of the chart and the number of slots a category is split
    /// into. Stacked values pile up from 0, positive and negative apart.
    fn bars(&self, category_count: usize) -> (Vec<Bar>, usize) {
        let mut stacks: Vec<(&str, usize)> = vec![];
        let mut group_count = 0;
        let groups: Vec<usize> = self
            .series_list
            .iter()
            .map(|series| {
                let known = series
                    .stack
                    .as_deref()
                    .and_then(|key| stacks.iter().find(|(k, _)| *k == key));
                if let Some((_, group)) = known {
                    return *group;
                }
                if let Some(key) = series.stack.as_deref() {
                    stacks.push((key, group_count));
                }
                group_count += 1;
                group_count - 1
            })
            .collect();

        // The (positive, negative) total of every group in every category.
        let mut totals = vec![(0.0_f32, 0.0_f32); group_count * category_count];
        let mut bars = vec![];
        for (series_index, series) in self.series_list.iter().enumerate() {
            let group = groups[series_index];
            for (index, value) in series.iter_values().enumerate() {
                let category = index.saturating_add(series.start_index);
                if value == NIL_VALUE || category >= category_count {
                    continue;
                }
                let total = &mut totals[group * category_count + category];
                let from = if value >= 0.0 { total.0 } else { total.1 };
                let to = from + value;
                if value >= 0.0 {
                    total.0 = to;
                } else {
                    total.1 = to;
                }
                bars.push(Bar {
                    series: series_index,
                    index,
                    category,
                    group,
                    from,
                    to,
                    value,
                });
            }
        }
        (bars, group_count.max(1))
    }

    /// Renders the chart to an SVG string.
    pub fn svg(&self) -> canvas::Result<String> {
        // Shares of the stacks: the same chart, drawn from their percentages.
        if self.stack_percent {
            let mut chart = self.clone();
            chart.base.apply_stack_percent(&mut chart.y_axis_configs);
            return chart.svg();
        }
        let mut c = self.new_canvas();
        let axis_top = self.render_header(&mut c);
        if axis_top > 0.0 {
            c = c.child(Box {
                top: axis_top,
                ..Default::default()
            });
        }

        // The categories are the x axis data, as in a bar chart; without
        // any, the longest series tells how many there are.
        let category_count = if self.x_axis_data.is_empty() {
            let longest = self.series_list.iter().map(|s| s.data.len()).max();
            longest.unwrap_or(0)
        } else {
            self.x_axis_data.len()
        };
        if category_count == 0 || self.has_no_data() {
            self.render_empty_text(c.child(Box::default()));
            return c.svg();
        }
        let (bars, group_count) = self.bars(category_count);

        // The value axis covers every bar end; it is always linear.
        let config = get_y_axis_config(&self.y_axis_configs, 0);
        let mut params = axis_value_params(
            &config,
            bars.iter().flat_map(|bar| [bar.from, bar.to]).collect(),
            false,
        );
        params.scale = AxisScale::Linear;
        let values = get_axis_values(params);
        let range = (values.max - values.min).max(f32::MIN_POSITIVE);
        let fraction = |value: f32| ((value - values.min) / range).clamp(0.0, 1.0);
        let formatter = config.axis_formatter.clone().unwrap_or_default();
        let ticks: Vec<String> = values
            .data
            .iter()
            .map(|label| format_string(label, &formatter))
            .collect();
        let split = ticks.len().saturating_sub(1).max(1);

        let on_angle = self.category_axis == PolarAxis::Angle;
        let start = self.start_angle;
        let span = if on_angle {
            360.0
        } else {
            match self.end_angle {
                Some(end) if end > start => (end - start).min(360.0),
                _ => DEFAULT_SPAN,
            }
        };
        let category = |index: usize| -> &str {
            self.x_axis_data
                .get(index)
                .map(String::as_str)
                .unwrap_or("")
        };
        let measure = |text: &str, font_size: f32| -> (f32, f32) {
            measure_text_width_family(&self.font_family, font_size, text)
                .map(|b| (b.width(), b.height()))
                .unwrap_or((0.0, font_size))
        };

        // The labels around the plot: the categories, or the value ticks.
        let (outer_hidden, outer_font_size, outer_font_color, outer_font_weight) = if on_angle {
            (
                self.x_axis_hidden,
                self.x_axis_font_size,
                self.x_axis_font_color,
                self.x_axis_font_weight.clone(),
            )
        } else {
            (
                self.y_axis_hidden,
                config.axis_font_size,
                config.axis_font_color,
                config.axis_font_weight.clone(),
            )
        };
        let outer_labels: Vec<(f32, &str)> = if outer_hidden {
            vec![]
        } else if on_angle {
            let slot = span / category_count as f32;
            (0..category_count)
                .map(|i| (start + slot * (i as f32 + 0.5), category(i)))
                .collect()
        } else {
            // On a full turn the last tick lands on the first.
            let count = if span >= 360.0 { split } else { split + 1 };
            (0..count)
                .map(|i| (start + span * i as f32 / split as f32, ticks[i].as_str()))
                .collect()
        };
        let (mut outer_width, mut outer_height) = (0.0_f32, 0.0_f32);
        for (_, label) in outer_labels.iter() {
            let (width, height) = measure(label, outer_font_size);
            outer_width = outer_width.max(width);
            outer_height = outer_height.max(height);
        }
        // Data labels of bars that grow outwards sit past their ends: keep
        // a ring free for them between the plot and the labels around it.
        let label_room = if on_angle && self.series_list.iter().any(|s| s.label_show) {
            measure("0", self.series_label_font_size).1 + SIDE_LABEL_GAP
        } else {
            0.0
        };
        let room = |size: f32| {
            if size > 0.0 {
                size + OUTER_LABEL_GAP + label_room + 2.0
            } else {
                label_room + 2.0
            }
        };

        let cx = c.width() / 2.0;
        let cy = c.height() / 2.0;
        let fit = (cx - room(outer_width))
            .min(cy - room(outer_height))
            .max(1.0);
        let r = match self.radius {
            Some(radius) if radius.is_finite() && radius > 0.0 => fit.min(radius),
            _ => fit,
        };
        let default_inner = if on_angle { 0.0 } else { r / 4.0 };
        let ir = self
            .inner_radius
            .filter(|v| v.is_finite())
            .unwrap_or(default_inner)
            .clamp(0.0, (r - 1.0).max(0.0));
        let depth = r - ir;
        let gap = self
            .category_gap
            .filter(|v| v.is_finite())
            .unwrap_or(0.2)
            .clamp(0.0, 0.9);

        // The grid: rings and spokes under the bars.
        let ring = |c: &mut canvas::Canvas, radius: f32| {
            if radius > 0.0 {
                c.circle(Circle {
                    stroke_color: Some(self.grid_stroke_color),
                    fill: None,
                    stroke_width: self.grid_stroke_width,
                    cx,
                    cy,
                    r: radius,
                    ..Default::default()
                });
            }
        };
        let spoke = |c: &mut canvas::Canvas, angle: f32| {
            let (from, to) = (
                get_pie_point(cx, cy, ir, angle),
                get_pie_point(cx, cy, r, angle),
            );
            c.line(Line {
                color: Some(self.grid_stroke_color),
                stroke_width: self.grid_stroke_width,
                left: from.x,
                top: from.y,
                right: to.x,
                bottom: to.y,
                ..Default::default()
            });
        };
        if on_angle {
            for i in 0..=split {
                ring(&mut c, ir + depth * i as f32 / split as f32);
            }
            let slot = span / category_count as f32;
            for i in 0..category_count {
                spoke(&mut c, start + slot * i as f32);
            }
        } else {
            ring(&mut c, ir);
            ring(&mut c, r);
            let count = if span >= 360.0 { split } else { split + 1 };
            for i in 0..count {
                spoke(&mut c, start + span * i as f32 / split as f32);
            }
        }

        // The bars.
        let band = if on_angle {
            span / category_count as f32
        } else {
            depth / category_count as f32
        };
        let bar_size = band * (1.0 - gap) / group_count as f32;
        // How far a round cap reaches past the end of its bar.
        let cap = if self.round_cap && !on_angle {
            bar_size / 2.0
        } else {
            0.0
        };
        let mut labels: Vec<(Point, (f32, f32), String)> = vec![];
        for bar in bars.iter() {
            let series = &self.series_list[bar.series];
            let offset =
                band * bar.category as f32 + band * gap / 2.0 + bar_size * bar.group as f32;
            let (from, to) = (fraction(bar.from), fraction(bar.to));
            // (outer radius, inner radius, start angle, sweep) of the bar.
            let (outer, inner, angle, delta) = if on_angle {
                let (a, b) = (ir + depth * from, ir + depth * to);
                (a.max(b), a.min(b), start + offset, bar_size)
            } else {
                let inner = ir + offset;
                (
                    inner + bar_size,
                    inner,
                    start + span * from,
                    span * (to - from),
                )
            };
            // A bar without any length (a zero, or one clipped by the axis).
            if outer - inner <= 0.0 || delta == 0.0 {
                continue;
            }
            let color = series
                .colors
                .as_ref()
                .and_then(|colors| colors.get(bar.index).copied().flatten())
                .unwrap_or_else(|| {
                    get_color(&self.series_colors, series.index.unwrap_or(bar.series))
                });
            let label = if series.label_show || self.tooltip_show {
                self.format_series_label(series, bar.category, bar.value)
            } else {
                String::new()
            };
            let tooltip_text = self
                .tooltip_show
                .then(|| format!("{}: {}", series.name, label));
            let mut classes: Vec<&str> = vec![];
            if self.animation.is_some() {
                classes.push("polar-anim");
            }
            if tooltip_text.is_some() {
                classes.push("ct-trigger");
            }
            c.sector(Sector {
                fill: color.into(),
                cx,
                cy,
                r: outer,
                ir: inner,
                start_angle: angle,
                delta,
                round_cap: self.round_cap && !on_angle,
                class: (!classes.is_empty()).then(|| classes.join(" ")),
                style: self
                    .animation
                    .as_ref()
                    .map(|a| format!("animation-delay:{}ms", bar.category as u32 * a.delay)),
                title: tooltip_text.clone(),
                dataset: self.point_dataset(series, bar.category, bar.value),
            });
            let middle = get_pie_point(cx, cy, (outer + inner) / 2.0, angle + delta / 2.0);
            if let Some(text) = tooltip_text {
                c.text_unmeasured(Text {
                    text,
                    class: Some("ct-tip".to_string()),
                    font_family: Some(self.font_family.clone()),
                    font_color: Some(self.series_label_font_color),
                    font_size: Some(self.series_label_font_size),
                    x: Some(middle.x),
                    y: Some(middle.y),
                    text_anchor: Some("middle".to_string()),
                    dominant_baseline: Some("central".to_string()),
                    ..Default::default()
                });
            }
            if series.label_show {
                // Past the end of the bar: further out, or further round.
                let (point, direction) = if on_angle {
                    let mid = angle + delta / 2.0;
                    (get_pie_point(cx, cy, outer, mid), outward(mid))
                } else {
                    let end = angle + delta;
                    let (dx, dy) = tangent(end);
                    let forward = if delta < 0.0 { (-dx, -dy) } else { (dx, dy) };
                    (get_pie_point(cx, cy, (outer + inner) / 2.0, end), forward)
                };
                labels.push((point, direction, label));
            }
        }

        // The labels of the axis along the start ray, beside it and over the
        // bars: the value ticks, or the categories.
        let (ray_hidden, ray_font_size, ray_font_color, ray_font_weight) = if on_angle {
            (
                self.y_axis_hidden,
                config.axis_font_size,
                config.axis_font_color,
                config.axis_font_weight.clone(),
            )
        } else {
            (
                self.x_axis_hidden,
                self.x_axis_font_size,
                self.x_axis_font_color,
                self.x_axis_font_weight.clone(),
            )
        };
        if !ray_hidden {
            let ray_labels: Vec<(f32, &str)> = if on_angle {
                (0..=split)
                    .map(|i| (ir + depth * i as f32 / split as f32, ticks[i].as_str()))
                    .collect()
            } else {
                (0..category_count)
                    .map(|i| (ir + band * (i as f32 + 0.5), category(i)))
                    .collect()
            };
            // Back from where the bars start, clear of their round caps.
            let (dx, dy) = tangent(start);
            let backdrop = (!self.background_color.is_transparent())
                .then(|| self.background_color.with_alpha(BACKDROP_ALPHA));
            let mut boxes = LabelBoxes::new(true);
            // From the outside in: where the labels are too close to each
            // other, the largest value (the outermost category) is kept.
            for (radius, text) in ray_labels.into_iter().rev() {
                if text.is_empty() {
                    continue;
                }
                let (width, height) = measure(text, ray_font_size);
                // A tick label hangs just inside its ring, so the outermost
                // one stays clear of the labels around the plot; one that
                // would reach across the center, where every bar starts, is
                // left out.
                let radius = if on_angle {
                    let inside = radius - height / 2.0 - 1.0;
                    if inside < height / 2.0 {
                        continue;
                    }
                    inside
                } else {
                    radius
                };
                let at = Beside::new(
                    get_pie_point(cx, cy, radius, start),
                    (-dx, -dy),
                    SIDE_LABEL_GAP + cap,
                    height,
                );
                let (left, top) = (at.left(width), at.y - height / 2.0);
                if !boxes.try_place(left, top, width, height) {
                    continue;
                }
                if let Some(color) = backdrop {
                    c.rect(Rect {
                        fill: Some(color.into()),
                        left: left - 2.0,
                        top,
                        width: width + 4.0,
                        height,
                        rx: Some(2.0),
                        ry: Some(2.0),
                        ..Default::default()
                    });
                }
                c.text_unmeasured(Text {
                    text: text.to_string(),
                    font_family: Some(self.font_family.clone()),
                    font_size: Some(ray_font_size),
                    font_color: Some(ray_font_color),
                    font_weight: ray_font_weight.clone(),
                    x: Some(at.x),
                    y: Some(at.y),
                    text_anchor: Some(at.anchor.to_string()),
                    dominant_baseline: Some("central".to_string()),
                    ..Default::default()
                });
            }
        }

        // The labels around the plot; one that would run into its neighbour
        // is left out.
        let mut boxes = LabelBoxes::new(true);
        for (angle, text) in outer_labels {
            if text.is_empty() {
                continue;
            }
            let (width, height) = measure(text, outer_font_size);
            let at = Beside::new(
                get_pie_point(cx, cy, r, angle),
                outward(angle),
                OUTER_LABEL_GAP + label_room,
                height,
            );
            if !boxes.try_place(at.left(width), at.y - height / 2.0, width, height) {
                continue;
            }
            c.text_unmeasured(Text {
                text: text.to_string(),
                font_family: Some(self.font_family.clone()),
                font_size: Some(outer_font_size),
                font_color: Some(outer_font_color),
                font_weight: outer_font_weight.clone(),
                x: Some(at.x),
                y: Some(at.y),
                text_anchor: Some(at.anchor.to_string()),
                dominant_baseline: Some("central".to_string()),
                ..Default::default()
            });
        }

        // The data labels, over everything else.
        let mut boxes = LabelBoxes::new(self.series_label_hide_overlap);
        for (point, direction, text) in labels {
            let (width, height) = measure(&text, self.series_label_font_size);
            let at = Beside::new(point, direction, SIDE_LABEL_GAP + cap, height);
            if !boxes.try_place(at.left(width), at.y - height / 2.0, width, height) {
                continue;
            }
            c.text_unmeasured(Text {
                text,
                font_family: Some(self.font_family.clone()),
                font_size: Some(self.series_label_font_size),
                font_color: Some(self.series_label_font_color),
                font_weight: self.series_label_font_weight.clone(),
                x: Some(at.x),
                y: Some(at.y),
                text_anchor: Some(at.anchor.to_string()),
                dominant_baseline: Some("central".to_string()),
                ..Default::default()
            });
        }

        let mut css = String::new();
        if let Some(ref anim) = self.animation {
            css.push_str(&format!(
                "@keyframes polar-grow{{from{{transform:scale(0)}}to{{transform:scale(1)}}}} \
                 .polar-anim{{transform-origin:{}px {}px;animation:polar-grow {}ms {} both}} ",
                format_float(cx + c.margin.left),
                format_float(cy + c.margin.top),
                anim.duration,
                anim.safe_easing()
            ));
        }
        if self.tooltip_show {
            css.push_str(TOOLTIP_STYLE);
        }
        if css.is_empty() {
            c.svg()
        } else {
            c.svg_with_style(&css)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Beside, PolarAxis, PolarBarChart, outward, tangent};
    use crate::Series;
    use pretty_assertions::assert_eq;

    fn categories() -> Vec<String> {
        ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"]
            .iter()
            .map(|s| s.to_string())
            .collect()
    }

    #[test]
    fn directions() {
        let close = |a: (f32, f32), b: (f32, f32)| {
            assert!(
                (a.0 - b.0).abs() < 1e-5 && (a.1 - b.1).abs() < 1e-5,
                "{a:?}"
            );
        };
        // 0 degrees is 12 o'clock, and the angles grow clockwise.
        close(outward(0.0), (0.0, -1.0));
        close(outward(90.0), (1.0, 0.0));
        close(tangent(0.0), (1.0, 0.0));
        close(tangent(90.0), (0.0, 1.0));
        close(tangent(180.0), (-1.0, 0.0));
    }

    #[test]
    fn labels_beside_a_point() {
        let point = (100.0, 50.0).into();
        let right = Beside::new(point, (1.0, 0.0), 5.0, 14.0);
        assert_eq!((105.0, 50.0, "start"), (right.x, right.y, right.anchor));
        assert_eq!(105.0, right.left(30.0));
        let left = Beside::new(point, (-1.0, 0.0), 5.0, 14.0);
        assert_eq!((95.0, 50.0, "end"), (left.x, left.y, left.anchor));
        assert_eq!(65.0, left.left(30.0));
        // Above: centered, and lifted by half its height to clear the point.
        let above = Beside::new(point, (0.0, -1.0), 5.0, 14.0);
        assert_eq!((100.0, 38.0, "middle"), (above.x, above.y, above.anchor));
        assert_eq!(85.0, above.left(30.0));
    }

    #[test]
    fn stacks_share_a_slot() {
        let mut a: Series = ("A", vec![1.0, 2.0]).into();
        let mut b: Series = ("B", vec![3.0, -4.0]).into();
        let c: Series = ("C", vec![5.0, 6.0]).into();
        let mut d: Series = ("D", vec![7.0, -8.0]).into();
        for series in [&mut a, &mut b, &mut d] {
            series.stack = Some("total".to_string());
        }
        let chart = PolarBarChart::new(vec![a, b, c, d], categories());
        let (bars, group_count) = chart.bars(7);
        // One slot for the stack and one for the series outside of it.
        assert_eq!(2, group_count);
        let ends: Vec<(usize, usize, f32, f32)> = bars
            .iter()
            .map(|bar| (bar.series, bar.group, bar.from, bar.to))
            .collect();
        assert_eq!(
            vec![
                (0, 0, 0.0, 1.0),
                (0, 0, 0.0, 2.0),
                (1, 0, 1.0, 4.0),
                // Negative values pile up downwards from 0.
                (1, 0, 0.0, -4.0),
                (2, 1, 0.0, 5.0),
                (2, 1, 0.0, 6.0),
                (3, 0, 4.0, 11.0),
                (3, 0, -4.0, -12.0),
            ],
            ends
        );
    }

    #[test]
    fn polar_bar_chart_basic() {
        let mut chart = PolarBarChart::new(
            vec![
                (
                    "Email",
                    vec![120.0, 132.0, 101.0, 134.0, 90.0, 230.0, 210.0],
                )
                    .into(),
                (
                    "Search",
                    vec![220.0, 182.0, 191.0, 234.0, 290.0, 330.0, 310.0],
                )
                    .into(),
            ],
            categories(),
        );
        chart.title_text = "Visits".to_string();
        assert_snapshot!("polar_bar_chart/basic.svg", chart.svg().unwrap());
    }

    #[test]
    fn polar_bar_chart_radial() {
        let mut chart = PolarBarChart::new(
            vec![("Progress", vec![45.0, 62.0, 78.0, 91.0]).into()],
            vec![
                "Q1".to_string(),
                "Q2".to_string(),
                "Q3".to_string(),
                "Q4".to_string(),
            ],
        );
        chart.category_axis = PolarAxis::Radius;
        chart.round_cap = true;
        chart.series_list[0].label_show = true;
        chart.y_axis_configs[0].axis_max = Some(100.0);
        assert_snapshot!("polar_bar_chart/radial.svg", chart.svg().unwrap());
    }
}
