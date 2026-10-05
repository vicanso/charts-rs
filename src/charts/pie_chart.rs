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
use crate::charts::measure_text_width_family;

/// Gap between two rings of nested pies, unless `ring_gap` says so.
const DEFAULT_RING_GAP: f32 = 6.0;

/// A pie / nightingale rose chart; each series contributes one value.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct PieChart {
    /// The shared chart options (size, series, title/legend, axes); exposed
    /// directly on the chart through `Deref`, e.g. `chart.title.text`.
    pub base: ChartBase,
    /// Outer radius of the pie.
    pub radius: f32,
    /// Inner radius; a value above zero renders a donut.
    pub inner_radius: f32,
    /// Renders as a nightingale rose: the radius scales with the value.
    pub rose_type: Option<bool>,
    /// Corner radius of the slices.
    pub border_radius: Option<f32>,
    /// Start angle of the first slice, in degrees.
    pub start_angle: f32,
    /// Angle the last slice ends at, in degrees: the slices share the part
    /// of the circle between the two angles (`-90` to `90` is the upper
    /// half). `None` (the default) is a full turn after `start_angle`.
    pub end_angle: Option<f32>,

    // x axis

    // y axis
    /// Y axis configurations; one per axis, up to two.
    pub y_axis_configs: Vec<YAxisConfig>,

    // grid

    // series
    /// Position of the slice labels.
    pub series_label_position: Option<String>,
    /// Slices spanning less than this many degrees get no label or leader
    /// line (default 0: only zero-value slices are skipped).
    pub min_show_label_angle: f32,
    /// Gap between two rings of nested pies (series with a `ring`), in
    /// pixels. Default: 6.
    pub ring_gap: f32,
}

impl std::ops::Deref for PieChart {
    type Target = ChartBase;
    fn deref(&self) -> &ChartBase {
        &self.base
    }
}
impl std::ops::DerefMut for PieChart {
    fn deref_mut(&mut self) -> &mut ChartBase {
        &mut self.base
    }
}

impl PieChart {
    fn fill_default(&mut self) {
        self.radius = 150.0;
        self.inner_radius = 40.0;
        self.ring_gap = DEFAULT_RING_GAP;
        self.rose_type = Some(true);
    }
    /// A pie names its slices beside them: without a say of the theme or of
    /// the options, it has no legend.
    fn fill_legend_default(&mut self) {
        if self.legend.show.is_none() {
            self.legend.show = Some(false);
        }
    }
    /// Creates a pie chart from json.
    pub fn from_json(data: &str) -> canvas::Result<PieChart> {
        let mut p = PieChart {
            ..Default::default()
        };
        p.fill_default();
        let value = p
            .base
            .fill_option(data, &mut p.y_axis_configs, super::schema::PIE_FIELDS)?;
        p.fill_legend_default();
        if let Some(radius) = get_f32_from_value(&value, "radius") {
            p.radius = radius;
        }
        if let Some(inner_radius) = get_f32_from_value(&value, "inner_radius") {
            p.inner_radius = inner_radius;
        }
        if let Some(rose_type) = get_bool_from_value(&value, "rose_type") {
            p.rose_type = Some(rose_type);
        }
        if let Some(border_radius) = get_f32_from_value(&value, "border_radius") {
            p.border_radius = Some(border_radius);
        }
        if let Some(start_angle) = get_f32_from_value(&value, "start_angle") {
            p.start_angle = start_angle;
        }
        if let Some(end_angle) = get_f32_from_value(&value, "end_angle") {
            p.end_angle = Some(end_angle);
        }
        if let Some(position) = get_string_from_value(&value, "series_label_position") {
            p.series_label_position = Some(position.to_lowercase());
        }
        if let Some(angle) = get_f32_from_value(&value, "min_show_label_angle") {
            p.min_show_label_angle = angle;
        }
        if let Some(ring_gap) = get_f32_from_value(&value, "ring_gap") {
            p.ring_gap = ring_gap;
        }
        Ok(p)
    }
    /// Creates a pie chart with custom theme.
    pub fn new_with_theme(series_list: Vec<Series>, theme: &str) -> PieChart {
        let mut p = PieChart {
            ..Default::default()
        };
        p.series_list = series_list;
        p.fill_default();
        p.base.fill_theme(get_theme(theme), &mut p.y_axis_configs);
        p.fill_legend_default();
        p
    }
    /// Creates a pie chart with default theme.
    pub fn new(series_list: Vec<Series>) -> PieChart {
        PieChart::new_with_theme(series_list, &get_default_theme_name())
    }
    /// The angle the slices share: a full turn, or what an `end_angle`
    /// after the start angle leaves of it.
    fn span(&self) -> f32 {
        match self.end_angle {
            Some(end) if end.is_finite() && end > self.start_angle => {
                (end - self.start_angle).min(360.0)
            }
            _ => 360.0,
        }
    }
    /// The box around a part of the unit circle from `start` over `span`
    /// degrees, with a hole of radius `hole`: `(left, top, right, bottom)`.
    fn extent(start: f32, span: f32, hole: f32) -> (f32, f32, f32, f32) {
        // The ends of the part, and where it is widest in between.
        let mut angles = vec![start, start + span];
        let mut cardinal = (start / 90.0).ceil() * 90.0;
        while cardinal < start + span {
            angles.push(cardinal);
            cardinal += 90.0;
        }
        let (mut left, mut top, mut right, mut bottom) = (f32::MAX, f32::MAX, f32::MIN, f32::MIN);
        for angle in angles {
            for radius in [hole, 1.0] {
                let point = get_pie_point(0.0, 0.0, radius, angle);
                left = left.min(point.x);
                right = right.max(point.x);
                top = top.min(point.y);
                bottom = bottom.max(point.y);
            }
        }
        (left, top, right, bottom)
    }
    /// Converts pie chart to svg.
    pub fn svg(&self) -> canvas::Result<String> {
        let mut c = self.new_canvas();

        let axis_top = self.render_header(&mut c);
        if axis_top > 0.0 {
            c = c.child(Box {
                top: axis_top,
                ..Default::default()
            });
        }

        // Missing points contribute nothing; summing the `NIL_VALUE` sentinel
        // would push the total to -inf and every angle to NaN.
        let values: Vec<f32> = self
            .series_list
            .iter()
            .map(|item| item.data.iter().flatten().sum())
            .collect();
        // The rings of nested pies: the series of a ring share a circle of
        // their own, the lowest ring innermost. Usually there is just one.
        let mut ring_ids: Vec<usize> = self.series_list.iter().map(|s| s.ring).collect();
        ring_ids.sort_unstable();
        ring_ids.dedup();
        let ring_count = ring_ids.len().max(1);
        let ring_of = |series: &Series| {
            ring_ids
                .iter()
                .position(|id| *id == series.ring)
                .unwrap_or(0)
        };
        // The total, the largest value and the number of slices of each ring.
        let mut sums = vec![0.0_f32; ring_count];
        let mut maxes = vec![0.0_f32; ring_count];
        let mut counts = vec![0_usize; ring_count];
        for (series, item) in self.series_list.iter().zip(values.iter()) {
            let ring = ring_of(series);
            sums[ring] += *item;
            if *item > maxes[ring] {
                maxes[ring] = *item;
            }
            counts[ring] += 1;
        }
        // Guard against non-positive totals (e.g. every series value is 0):
        // all numerators are 0 in that case, so flooring the divisors to a
        // positive value keeps `delta` / `cr` / percentages finite instead of
        // producing NaN coordinates.
        for value in sums.iter_mut().chain(maxes.iter_mut()) {
            if *value <= 0.0 {
                *value = 1.0;
            }
        }
        let span = self.span();
        // Where the next slice of each ring starts.
        let mut starts = vec![self.start_angle; ring_count];
        let mut radius_double = c.height();

        if c.width() < radius_double {
            radius_double = c.width();
        }
        radius_double *= 0.8;
        let mut r = radius_double / 2.0;
        if r > self.radius {
            r = self.radius;
        }

        let mut cx = (c.width() - radius_double) / 2.0 + r;
        let mut cy = (c.height() - radius_double) / 2.0 + r;
        // A part of a circle is not as wide and high as the whole: it may
        // take a larger radius, and is centered on what is drawn of it.
        if span < 360.0 {
            // How large the hole is depends on the radius, which depends on
            // the room the part takes: twice round settles it.
            r = self.radius.max(1.0);
            for _ in 0..2 {
                let hole = (self.inner_radius / r).clamp(0.0, 1.0);
                let (left, top, right, bottom) = Self::extent(self.start_angle, span, hole);
                let fit = (c.width() * 0.8 / (right - left).max(0.1))
                    .min(c.height() * 0.8 / (bottom - top).max(0.1));
                r = fit.min(self.radius).max(1.0);
                cx = c.width() / 2.0 - (left + right) / 2.0 * r;
                cy = c.height() / 2.0 - (top + bottom) / 2.0 * r;
            }
        }
        let label_offset = 20.0;
        let mut series_label_formatter = self.series.label.formatter.clone();
        if series_label_formatter.is_empty() {
            series_label_formatter = "{a}: {d}".to_string();
        }
        let rose_type = self.rose_type.unwrap_or_default();

        // The inner and the outer radius of each ring: the room between the
        // hole and the edge, shared evenly, with a gap between two rings.
        let bands: Vec<(f32, f32)> = if ring_count < 2 {
            vec![(self.inner_radius, r)]
        } else {
            let room = (r - self.inner_radius).max(1.0);
            let gap = self.ring_gap.max(0.0).min(room / (ring_count as f32 * 3.0));
            let width = (room - gap * (ring_count - 1) as f32) / ring_count as f32;
            (0..ring_count)
                .map(|ring| {
                    let inner = self.inner_radius + (width + gap) * ring as f32;
                    (inner, inner + width)
                })
                .collect()
        };

        let mut prev_quadrant = u8::MAX;
        let mut prev_end_y = f32::MAX;
        for (index, series) in self.series_list.iter().enumerate() {
            let ring = ring_of(series);
            let (sum, max) = (sums[ring], maxes[ring]);
            let (band_inner, band_outer) = bands[ring];
            let start_angle = starts[ring];
            let value = values[index];
            let mut delta = span / counts[ring] as f32;
            let mut cr = value / max * (band_outer - band_inner) + band_inner;
            let color = get_color(&self.series.colors, series.index.unwrap_or(index));
            // normal pie
            if !rose_type {
                cr = band_outer;
                delta = value / sum * span;
            }
            let half_delta = delta / 2.0;
            if cr - band_inner < 1.0 {
                cr = band_inner + 1.0;
            }
            let (anim_class, anim_style, fade_class) = if let Some(ref a) = self.animation {
                (
                    Some("pie-anim".to_string()),
                    Some(format!("animation-delay:{}ms", index as u32 * a.delay)),
                    Some("pie-fade".to_string()),
                )
            } else {
                (None, None, None)
            };
            let mut pie = Pie {
                fill: color.into(),
                cx,
                cy,
                r: cr,
                ir: band_inner,
                start_angle,
                delta,
                class: anim_class,
                style: anim_style,
                dataset: vec![
                    ("series".to_string(), series.name.clone()),
                    ("value".to_string(), format_float(value)),
                    ("percentage".to_string(), format_float(value / sum * 100.0)),
                ],
                ..Default::default()
            };
            // Which of the nested pies the slice is in.
            if ring_count > 1 {
                pie.dataset
                    .push(("ring".to_string(), series.ring.to_string()));
            }
            if let Some(border_radius) = self.border_radius {
                pie.border_radius = border_radius;
            }
            let tooltip_text = if self.tooltip.show {
                Some(
                    LabelOption {
                        series_name: series.name.clone(),
                        value,
                        percentage: value / sum,
                        formatter: "{a}: {c} ({d})".to_string(),
                        ..Default::default()
                    }
                    .format(),
                )
            } else {
                None
            };
            if let Some(ref t) = tooltip_text {
                pie.title = Some(t.clone());
                let trigger = match pie.class.take() {
                    Some(c) => format!("{c} ct-trigger"),
                    None => "ct-trigger".to_string(),
                };
                pie.class = Some(trigger);
            }

            c.pie(pie);

            // Only the outermost ring has room around it for its labels:
            // the slices of the rings inside it are named on them.
            let nested = ring + 1 < ring_count;
            let is_inside = nested || self.series_label_position == Some("inside".to_string());

            let angle = start_angle + half_delta;
            // Hidden hover label, drawn immediately after the slice so the
            // adjacent-sibling CSS rule reveals it on hover.
            if let Some(text) = tooltip_text {
                let p = get_pie_point(cx, cy, (cr + band_inner) / 2.0, angle);
                c.text(Text {
                    text,
                    class: Some("ct-tip".to_string()),
                    font_weight: self.tooltip.font.weight.clone(),
                    font_family: Some(self.font_family.clone()),
                    font_color: Some(self.tooltip_font_color(self.series.label.font.color)),
                    font_size: Some(self.tooltip_font_size(self.series.label.font.size)),
                    x: Some(p.x),
                    y: Some(p.y),
                    text_anchor: Some("middle".to_string()),
                    dominant_baseline: Some("central".to_string()),
                    ..Default::default()
                });
            }
            starts[ring] += delta;
            // A zero-value (or too thin) slice has nothing to point at.
            if value <= 0.0 || delta <= 0.0 || delta < self.min_show_label_angle {
                continue;
            }
            let label_option = LabelOption {
                series_name: series.name.clone(),
                value,
                percentage: value / sum,
                // On a slice of an inner ring there is room for its name,
                // unless a format asks for more.
                formatter: if nested && self.series.label.formatter.is_empty() {
                    "{a}".to_string()
                } else {
                    series_label_formatter.clone()
                },
                ..Default::default()
            };
            let label_text = label_option.format();
            let mut label_color = self.series.label.font.color;

            let label_margin = if is_inside {
                // A little past the middle of the slice of a nested pie,
                // where it is wider; two thirds out on the one pie there is
                // otherwise.
                let label_radius = if ring_count > 1 {
                    band_inner + (cr - band_inner) * 0.6
                } else {
                    2. * cr / 3.
                };
                let label_point = get_pie_point(cx, cy, label_radius, angle);
                let mut label_margin = Box {
                    left: label_point.x,
                    top: label_point.y,
                    ..Default::default()
                };
                let mut width = 0.0;
                if let Ok(b) = measure_text_width_family(
                    &self.font_family,
                    self.series.label.font.size,
                    &label_text,
                ) {
                    width = b.width();
                    label_margin.left -= b.width() / 2.;
                }
                if ring_count > 1 {
                    // A name wider or higher than its slice is left out;
                    // the others stand on it, in a color that shows there.
                    let across = 2.0 * label_radius * (delta.min(180.0) / 2.0).to_radians().sin();
                    if width > across || cr - band_inner < self.series.label.font.size + 2.0 {
                        continue;
                    }
                    label_margin.top += self.series.label.font.size * 0.35;
                    label_color = if color.is_light() {
                        Color::black().with_alpha(200)
                    } else {
                        Color::white()
                    };
                }

                label_margin
            } else {
                let mut points = vec![];
                points.push(get_pie_point(cx, cy, cr, angle));
                let mut end = get_pie_point(cx, cy, r + label_offset, angle);

                let quadrant = get_quadrant(cx, cy, &end);
                // quadrant change
                if quadrant != prev_quadrant {
                    prev_end_y = f32::MAX;
                    prev_quadrant = quadrant;
                }
                // label overlap
                if (end.y - prev_end_y).abs() < self.series.label.font.size {
                    if quadrant == 1 || quadrant == 4 {
                        end.y = prev_end_y + self.series.label.font.size;
                    } else {
                        end.y = prev_end_y - self.series.label.font.size;
                    }
                }
                prev_end_y = end.y;

                points.push(end);

                // Angles go on past a full turn, and below zero.
                let is_left = angle.rem_euclid(360.0) > 180.0;
                if is_left {
                    end.x -= label_offset;
                } else {
                    end.x += label_offset;
                }
                let mut label_margin = Box {
                    left: end.x,
                    top: end.y + 5.0,
                    ..Default::default()
                };

                if is_left {
                    if let Ok(b) = measure_text_width_family(
                        &self.font_family,
                        self.series.label.font.size,
                        &label_text,
                    ) {
                        label_margin.left -= b.width();
                    }
                } else {
                    label_margin.left += 3.0;
                }

                points.push(end);

                c.smooth_line(SmoothLine {
                    color: Some(color),
                    points,
                    symbol: None,
                    class: fade_class.clone(),
                    ..Default::default()
                });
                label_margin
            };

            c.child(label_margin).text(Text {
                text: label_text,
                font_family: Some(self.font_family.clone()),
                font_size: Some(self.series.label.font.size),
                font_color: Some(label_color),
                class: fade_class,
                ..Default::default()
            });
        }

        let mut css = String::new();
        if let Some(ref anim) = self.animation {
            css.push_str(&format!(
                "@keyframes pie-grow{{from{{transform:scale(0)}}to{{transform:scale(1)}}}} \
                 @keyframes pie-fade{{from{{opacity:0}}to{{opacity:1}}}} \
                 .pie-anim{{transform-origin:{}px {}px;animation:pie-grow {}ms {} both}} \
                 .pie-fade{{animation:pie-fade {}ms {} both}}",
                format_float(cx + c.margin.left),
                format_float(cy + c.margin.top),
                anim.duration,
                anim.safe_easing(),
                anim.duration,
                anim.safe_easing()
            ));
        }
        if self.tooltip.show {
            if !css.is_empty() {
                css.push(' ');
            }
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
    use super::PieChart;

    #[test]
    fn pie_basic() {
        let mut pie_chart = PieChart::new(vec![
            ("rose 1", vec![40.0]).into(),
            ("rose 2", vec![38.0]).into(),
            ("rose 3", vec![32.0]).into(),
            ("rose 4", vec![30.0]).into(),
            ("rose 5", vec![28.0]).into(),
            ("rose 6", vec![26.0]).into(),
            ("rose 7", vec![22.0]).into(),
            ("rose 8", vec![18.0]).into(),
        ]);
        pie_chart.title.text = "Nightingale Chart".to_string();
        pie_chart.sub_title.text = "Fake Data".to_string();
        assert_snapshot!("pie_chart/basic.svg", pie_chart.svg().unwrap());
    }

    #[test]
    fn small_pie_basic() {
        let mut pie_chart = PieChart::new(vec![
            ("rose 1", vec![400.0]).into(),
            ("rose 2", vec![38.0]).into(),
            ("rose 3", vec![32.0]).into(),
            ("rose 4", vec![30.0]).into(),
            ("rose 5", vec![28.0]).into(),
            ("rose 6", vec![26.0]).into(),
            ("rose 7", vec![22.0]).into(),
            ("rose 8", vec![18.0]).into(),
        ]);
        pie_chart.width = 400.0;
        pie_chart.height = 300.0;
        pie_chart.title.text = "Nightingale Chart".to_string();
        pie_chart.sub_title.text = "Fake Data".to_string();
        assert_snapshot!("pie_chart/small_basic.svg", pie_chart.svg().unwrap());
    }

    #[test]
    fn not_rose_pie() {
        let mut pie_chart = PieChart::new(vec![
            ("rose 1", vec![400.0]).into(),
            ("rose 2", vec![38.0]).into(),
            ("rose 3", vec![32.0]).into(),
            ("rose 4", vec![30.0]).into(),
            ("rose 5", vec![28.0]).into(),
            ("rose 6", vec![26.0]).into(),
            ("rose 7", vec![22.0]).into(),
            ("rose 8", vec![18.0]).into(),
        ]);
        pie_chart.rose_type = Some(false);
        pie_chart.title.text = "Pie Chart".to_string();
        pie_chart.sub_title.text = "Fake Data".to_string();
        assert_snapshot!("pie_chart/not_rose.svg", pie_chart.svg().unwrap());
    }

    #[test]
    fn not_rose_radius_pie() {
        let mut pie_chart = PieChart::new(vec![
            ("rose 1", vec![40.0]).into(),
            ("rose 2", vec![38.0]).into(),
            ("rose 3", vec![32.0]).into(),
            ("rose 4", vec![30.0]).into(),
        ]);
        pie_chart.start_angle = 10.0;
        pie_chart.series_label_position = Some("inside".to_string());
        pie_chart.rose_type = Some(false);
        pie_chart.inner_radius = 0.0;
        pie_chart.border_radius = Some(0.0);
        pie_chart.title.text = "Pie Chart".to_string();
        pie_chart.sub_title.text = "Fake Data".to_string();
        assert_snapshot!("pie_chart/not_rose_radius.svg", pie_chart.svg().unwrap());
    }

    #[test]
    fn pie_animation_json() {
        let chart = PieChart::from_json(
            r###"{
                "series_list": [
                    {"name": "a", "data": [40]},
                    {"name": "b", "data": [60]}
                ],
                "rose_type": false,
                "animation": {"duration": 800, "easing": "ease-out", "delay": 50}
            }"###,
        )
        .unwrap();
        let svg = chart.svg().unwrap();
        assert!(svg.contains("pie-grow"), "missing @keyframes pie-grow");
        assert!(svg.contains(".pie-anim"), "missing .pie-anim class rule");
        assert!(svg.contains("transform-origin"), "missing transform-origin");
        assert!(svg.contains("800ms ease-out"), "missing duration/easing");
        assert!(
            svg.contains(r#"class="pie-anim""#),
            "missing class attr on slice"
        );
        assert!(
            svg.contains("animation-delay:0ms"),
            "missing delay for slice 0"
        );
        assert!(
            svg.contains("animation-delay:50ms"),
            "missing delay for slice 1"
        );
        assert!(
            svg.contains(r#"class="pie-fade""#),
            "missing fade class on labels"
        );
    }

    #[test]
    fn pie_rose_small_piece() {
        let mut pie_chart = PieChart::new(vec![
            ("rose 1", vec![40000.0]).into(),
            ("rose 2", vec![38.0]).into(),
            ("rose 3", vec![32.0]).into(),
            ("rose 4", vec![30.0]).into(),
            ("rose 5", vec![28.0]).into(),
            ("rose 6", vec![26.0]).into(),
            ("rose 7", vec![22.0]).into(),
            ("rose 8", vec![18.0]).into(),
        ]);
        pie_chart.title.text = "Nightingale Chart".to_string();
        pie_chart.sub_title.text = "Fake Data".to_string();
        assert_snapshot!("pie_chart/rose_small_piece.svg", pie_chart.svg().unwrap());
    }

    #[test]
    fn pie_chart_tooltip() {
        let chart = PieChart::from_json(
            r#"{"tooltip_show": true, "series_list": [{"name": "a", "data": [10]}, {"name": "b", "data": [30]}]}"#,
        )
        .unwrap();
        let svg = chart.svg().unwrap();
        // "a" is 10 of 40 -> 25%.
        assert!(
            svg.contains("<title>a: 10 (25%)</title>"),
            "missing pie title"
        );
        assert!(svg.contains(r#"class="ct-tip""#), "missing hover label");
        assert!(
            svg.contains(".ct-trigger:hover+.ct-tip"),
            "missing hover css"
        );
        let off = PieChart::from_json(
            r#"{"series_list": [{"name": "a", "data": [10]}, {"name": "b", "data": [30]}]}"#,
        )
        .unwrap();
        let off_svg = off.svg().unwrap();
        assert!(!off_svg.contains("<title>"));
        assert!(!off_svg.contains("ct-tip"));
    }

    // All-zero totals previously divided by zero, producing `LNaN,NaN` paths.
    #[test]
    fn all_zero_values_no_nan() {
        let chart = PieChart::from_json(
            r#"{"series_list":[{"name":"a","data":[0.0]},{"name":"b","data":[0.0]}]}"#,
        )
        .unwrap();
        let svg = chart.svg().unwrap();
        assert!(!svg.contains("NaN"), "all-zero pie must not emit NaN");
    }
}
