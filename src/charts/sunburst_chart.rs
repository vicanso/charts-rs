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

// ── Hierarchical data ─────────────────────────────────────────────────────────

/// A node in the sunburst hierarchy. A node is a leaf when `children` is empty,
/// in which case `value` is used directly; otherwise its value is the sum of
/// its children's values.
#[derive(Clone, Debug, Default)]
pub struct SunburstData {
    pub name: String,
    pub value: f32,
    pub children: Vec<SunburstData>,
    /// Optional explicit color; when `None` the color is derived from the
    /// top-level palette and lightened with depth.
    pub color: Option<Color>,
}

impl SunburstData {
    /// Total value of the node: its own value when a leaf, otherwise the sum
    /// of all descendant leaf values.
    fn total(&self) -> f32 {
        if self.children.is_empty() {
            self.value.max(0.0)
        } else {
            self.children.iter().map(|c| c.total()).sum()
        }
    }
}

fn max_depth(nodes: &[SunburstData]) -> usize {
    nodes
        .iter()
        .map(|n| {
            if n.children.is_empty() {
                1
            } else {
                1 + max_depth(&n.children)
            }
        })
        .max()
        .unwrap_or(0)
}

/// Mixes a color toward white by `factor` (0.0 = unchanged, 1.0 = white),
/// used to fade deeper rings while keeping the parent hue.
fn lighten(c: Color, factor: f32) -> Color {
    let f = factor.clamp(0.0, 0.85);
    let mix = |v: u8| (v as f32 + (255.0 - v as f32) * f) as u8;
    Color {
        r: mix(c.r),
        g: mix(c.g),
        b: mix(c.b),
        a: c.a,
    }
}

pub(crate) fn parse_node(value: &serde_json::Value) -> Option<SunburstData> {
    let name = get_string_from_value(value, "name").unwrap_or_default();
    let val = get_f32_from_value(value, "value").unwrap_or_default();
    let color = get_color_from_value(value, "color");
    let mut children = vec![];
    if let Some(arr) = value.get("children").and_then(|v| v.as_array()) {
        for item in arr.iter() {
            if let Some(node) = parse_node(item) {
                children.push(node);
            }
        }
    }
    if name.is_empty() && children.is_empty() && val == 0.0 {
        return None;
    }
    Some(SunburstData {
        name,
        value: val,
        children,
        color,
    })
}

/// Geometry and totals shared by every ring while drawing the hierarchy.
struct RingLayout<'a> {
    cx: f32,
    cy: f32,
    grand_total: f32,
    /// Pixel thickness of each ring level, indexed by depth.
    thicknesses: &'a [f32],
}

// ── SunburstChart ──────────────────────────────────────────────────────────────

#[derive(Clone, Debug, Default, Chart)]
pub struct SunburstChart {
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

    // x/y axis (required by #[derive(Chart)], unused in rendering)
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
    pub y_axis_hidden: bool,
    y_axis_configs: Vec<YAxisConfig>,
    grid_stroke_color: Color,
    grid_stroke_width: f32,

    // series (required by #[derive(Chart)])
    pub series_stroke_width: f32,
    pub series_label_font_color: Color,
    pub series_label_font_size: f32,
    pub series_label_font_weight: Option<String>,
    pub series_label_formatter: String,
    pub series_colors: Vec<Color>,
    pub series_symbol: Option<Symbol>,
    pub series_smooth: bool,
    pub series_fill: bool,

    // sunburst-specific
    /// Hierarchy roots. Multiple roots form a forest sharing the full circle.
    pub series_data: Vec<SunburstData>,
    /// Outer radius in pixels. `0.0` (default) auto-fits the content area.
    pub radius: f32,
    /// Inner radius (center hole) in pixels. Default: 0.0.
    pub inner_radius: f32,
    /// Starting angle in degrees, clockwise from the top. Default: 0.0.
    pub start_angle: f32,
    /// Relative thickness weight of each ring level; unspecified levels
    /// default to `1.0`. E.g. `[2.0]` makes the innermost ring twice as thick
    /// as the others. Empty (default) splits all rings equally.
    pub level_thickness: Vec<f32>,
    /// Optional expand animation: rings scale out from the center, staggered
    /// by depth (`delay` ms per level); labels fade in alongside.
    pub animation: Option<AnimationConfig>,
}

impl SunburstChart {
    pub fn new_with_theme(series_data: Vec<SunburstData>, theme: &str) -> SunburstChart {
        let mut c = SunburstChart {
            series_data,
            ..Default::default()
        };
        c.fill_theme(get_theme(theme));
        c
    }

    pub fn new(series_data: Vec<SunburstData>) -> SunburstChart {
        SunburstChart::new_with_theme(series_data, &get_default_theme_name())
    }

    pub fn from_json(json: &str) -> canvas::Result<SunburstChart> {
        let mut c = SunburstChart {
            ..Default::default()
        };
        let value = c.fill_option(json)?;
        if let Some(arr) = value.get("series_data").and_then(|v| v.as_array()) {
            c.series_data = arr.iter().filter_map(parse_node).collect();
        }
        if let Some(v) = get_f32_from_value(&value, "radius") {
            c.radius = v;
        }
        if let Some(v) = get_f32_from_value(&value, "inner_radius") {
            c.inner_radius = v;
        }
        if let Some(v) = get_f32_from_value(&value, "start_angle") {
            c.start_angle = v;
        }
        if let Some(v) = get_f32_slice_from_value(&value, "level_thickness") {
            c.level_thickness = v;
        }
        if let Some(anim) = value.get("animation")
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
            c.animation = Some(config);
        }
        Ok(c)
    }

    #[allow(clippy::too_many_arguments)]
    fn draw_nodes(
        &self,
        c: &mut Canvas,
        nodes: &[SunburstData],
        start_angle: f32,
        total: f32,
        // Angular extent (degrees) available to this set of siblings. The top
        // level spans the full 360; children only span their parent's slice.
        span: f32,
        depth: usize,
        inner_r: f32,
        base_color: Color,
        l: &RingLayout,
    ) {
        if total <= 0.0 {
            return;
        }
        let Some(thickness) = l.thicknesses.get(depth).copied() else {
            return;
        };
        let mut angle = start_angle;
        for (i, node) in nodes.iter().enumerate() {
            let node_total = node.total();
            if node_total <= 0.0 {
                continue;
            }
            let delta = node_total / total * span;
            let color = node.color.unwrap_or_else(|| {
                if depth == 0 {
                    get_color(&self.series_colors, i)
                } else {
                    lighten(base_color, depth as f32 * 0.16)
                }
            });
            let outer_r = inner_r + thickness;

            // Tiny slices would render as degenerate arcs; skip drawing but
            // still consume their angle so the layout stays consistent.
            if delta >= 1.0 {
                let (anim_class, anim_style) = if let Some(ref a) = self.animation {
                    (
                        Some("sunburst-anim".to_string()),
                        Some(format!("animation-delay:{}ms", depth as u32 * a.delay)),
                    )
                } else {
                    (None, None)
                };
                c.pie(Pie {
                    fill: color.into(),
                    stroke_color: Some(self.background_color),
                    cx: l.cx,
                    cy: l.cy,
                    r: outer_r,
                    ir: inner_r,
                    start_angle: angle,
                    delta,
                    border_radius: 0.0,
                    class: anim_class,
                    style: anim_style,
                    ..Default::default()
                });
                self.draw_label(
                    c, node, angle, delta, inner_r, thickness, color, node_total, l,
                );
            }

            if !node.children.is_empty() {
                let child_base = if depth == 0 { color } else { base_color };
                self.draw_nodes(
                    c,
                    &node.children,
                    angle,
                    node_total,
                    delta,
                    depth + 1,
                    outer_r,
                    child_base,
                    l,
                );
            }
            angle += delta;
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn draw_label(
        &self,
        c: &mut Canvas,
        node: &SunburstData,
        start_angle: f32,
        delta: f32,
        inner_r: f32,
        thickness: f32,
        fill: Color,
        value: f32,
        l: &RingLayout,
    ) {
        // {a}/{b}: node name, {c}: value, {d}: percentage of the grand total.
        let text = if self.series_label_formatter.is_empty() {
            node.name.clone()
        } else {
            LabelOption {
                series_name: node.name.clone(),
                category_name: node.name.clone(),
                value,
                percentage: if l.grand_total > 0.0 {
                    value / l.grand_total
                } else {
                    0.0
                },
                formatter: self.series_label_formatter.clone(),
            }
            .format()
        };
        if text.is_empty() || thickness < 12.0 {
            return;
        }
        let font_size = self.series_label_font_size.max(10.0);
        let mid_angle = start_angle + delta / 2.0;
        let mid_r = inner_r + thickness / 2.0;
        let point = get_pie_point(l.cx, l.cy, mid_r, mid_angle);

        let name_w = measure_text_width_family(&self.font_family, font_size, &text)
            .map(|b| b.width())
            .unwrap_or(text.len() as f32 * font_size * 0.6);
        // Labels are drawn tangentially (along the arc). Skip the label when
        // the slice's arc length cannot hold the text — this keeps the text
        // inside its own ring band instead of overflowing onto neighbours.
        let arc_len = delta.to_radians() * mid_r;
        if name_w + 6.0 > arc_len {
            return;
        }

        // Tangential rotation: chart angle 0 points up and increases clockwise,
        // which matches the SVG `rotate` convention, so the tangent at
        // `mid_angle` is `rotate(mid_angle)`. Flip text on the bottom half so it
        // stays upright rather than upside-down.
        let mut deg = mid_angle % 360.0;
        if deg < 0.0 {
            deg += 360.0;
        }
        if deg > 90.0 && deg < 270.0 {
            deg -= 180.0;
        }
        // The transform is emitted verbatim, so bake in the canvas offset that
        // `Canvas::text` adds to x / y.
        let abs_x = point.x + c.margin.left;
        let abs_y = point.y + c.margin.top;
        let transform = format!(
            "rotate({} {} {})",
            format_float(deg),
            format_float(abs_x),
            format_float(abs_y)
        );

        let text_color = if fill.is_light() {
            Color {
                r: 30,
                g: 30,
                b: 30,
                a: 255,
            }
        } else {
            Color {
                r: 255,
                g: 255,
                b: 255,
                a: 230,
            }
        };
        c.text(Text {
            text,
            font_family: Some(self.font_family.clone()),
            font_color: Some(text_color),
            font_size: Some(font_size),
            x: Some(point.x),
            y: Some(point.y),
            transform: Some(transform),
            text_anchor: Some("middle".to_string()),
            dominant_baseline: Some("central".to_string()),
            class: self.animation.as_ref().map(|_| "sunburst-fade".to_string()),
            ..Default::default()
        });
    }

    pub fn svg(&self) -> canvas::Result<String> {
        let mut c = Canvas::new_width_xy(self.width, self.height, self.x, self.y);

        let axis_top = self.render_header(&mut c);

        let mut content = c.child(Box {
            top: axis_top,
            ..Default::default()
        });

        let cw = content.width();
        let ch = content.height();
        if cw <= 0.0 || ch <= 0.0 || self.series_data.is_empty() {
            return c.svg();
        }

        let total: f32 = self.series_data.iter().map(|n| n.total()).sum();
        if total <= 0.0 {
            return c.svg();
        }

        let depth = max_depth(&self.series_data).max(1);
        let cx = cw / 2.0;
        let cy = ch / 2.0;
        let mut max_r = cw.min(ch) / 2.0 * 0.95;
        if self.radius > 0.0 {
            max_r = max_r.min(self.radius);
        }
        let inner = self.inner_radius.max(0.0);
        let available = max_r - inner;
        if available <= 0.0 {
            return c.svg();
        }
        // Ring thicknesses from relative weights; levels without an explicit
        // weight get 1.0, so an empty `level_thickness` splits rings equally.
        let mut weights: Vec<f32> = (0..depth)
            .map(|i| self.level_thickness.get(i).copied().unwrap_or(1.0).max(0.0))
            .collect();
        let weight_sum: f32 = weights.iter().sum();
        if weight_sum <= 0.0 {
            weights = vec![1.0; depth];
        }
        let weight_sum: f32 = weights.iter().sum();
        // Multiply before dividing so the equal-weight case is bit-identical
        // to the previous `available / depth` computation.
        let thicknesses: Vec<f32> = weights.iter().map(|w| available * w / weight_sum).collect();

        self.draw_nodes(
            &mut content,
            &self.series_data,
            self.start_angle,
            total,
            360.0,
            0,
            inner,
            Color::black(),
            &RingLayout {
                cx,
                cy,
                grand_total: total,
                thicknesses: &thicknesses,
            },
        );

        if let Some(ref anim) = self.animation {
            let css = format!(
                "@keyframes sunburst-grow{{from{{transform:scale(0)}}to{{transform:scale(1)}}}} \
                 @keyframes sunburst-fade{{from{{opacity:0}}to{{opacity:1}}}} \
                 .sunburst-anim{{transform-origin:{}px {}px;animation:sunburst-grow {}ms {} both}} \
                 .sunburst-fade{{animation:sunburst-fade {}ms {} both}}",
                format_float(cx + content.margin.left),
                format_float(cy + content.margin.top),
                anim.duration,
                anim.easing,
                anim.duration,
                anim.easing
            );
            c.svg_with_style(&css)
        } else {
            c.svg()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{SunburstChart, SunburstData};
    use pretty_assertions::assert_eq;

    fn leaf(name: &str, value: f32) -> SunburstData {
        SunburstData {
            name: name.to_string(),
            value,
            ..Default::default()
        }
    }

    fn make_sunburst() -> SunburstChart {
        SunburstChart::new(vec![
            SunburstData {
                name: "Grandpa".to_string(),
                children: vec![
                    SunburstData {
                        name: "Uncle Leo".to_string(),
                        children: vec![leaf("Cousin Jack", 18.0), leaf("Cousin Mary", 12.0)],
                        ..Default::default()
                    },
                    SunburstData {
                        name: "Father".to_string(),
                        children: vec![leaf("Me", 40.0), leaf("Brother Peter", 20.0)],
                        ..Default::default()
                    },
                ],
                ..Default::default()
            },
            SunburstData {
                name: "Nancy".to_string(),
                children: vec![SunburstData {
                    name: "Uncle Nike".to_string(),
                    children: vec![leaf("Cousin Betty", 10.0), leaf("Cousin Jenny", 30.0)],
                    ..Default::default()
                }],
                ..Default::default()
            },
        ])
    }

    #[test]
    fn sunburst_chart_basic() {
        assert_eq!(
            include_str!("../../asset/sunburst_chart/basic.svg"),
            make_sunburst().svg().unwrap()
        );
    }

    #[test]
    fn sunburst_chart_label_formatter() {
        let mut chart = make_sunburst();
        chart.series_label_formatter = "{b}: {c} ({d})".to_string();
        let svg = chart.svg().unwrap();
        // "Me" leaf: value 40 of grand total 130 -> 30.8%
        assert!(
            svg.contains("Me: 40 (30.8%)"),
            "missing formatted leaf label"
        );
        // "Father" branch: 40 + 20 = 60 -> 46.2%
        assert!(
            svg.contains("Father: 60 (46.2%)"),
            "missing formatted branch label"
        );
    }

    #[test]
    fn sunburst_chart_level_thickness() {
        let mut chart = make_sunburst();
        // Three levels with weights 2:1:1 over 160px (radius 180, inner 20):
        // ring outer radii become 100, 140 and 180.
        chart.radius = 180.0;
        chart.inner_radius = 20.0;
        chart.level_thickness = vec![2.0, 1.0, 1.0];
        let svg = chart.svg().unwrap();
        assert!(svg.contains("A100 100"), "inner ring should end at r=100");
        assert!(svg.contains("A140 140"), "middle ring should end at r=140");
        assert!(svg.contains("A180 180"), "outer ring should end at r=180");
    }

    #[test]
    fn sunburst_chart_animation() {
        let mut chart = make_sunburst();
        chart.animation = Some(super::AnimationConfig {
            duration: 600,
            easing: "linear".to_string(),
            delay: 100,
        });
        let svg = chart.svg().unwrap();
        assert!(
            svg.contains("sunburst-grow"),
            "missing @keyframes sunburst-grow"
        );
        assert!(
            svg.contains(r#"class="sunburst-anim""#),
            "missing class attr on arc"
        );
        assert!(svg.contains("600ms linear"), "missing duration/easing");
        assert!(
            svg.contains("animation-delay:0ms"),
            "missing delay for depth 0"
        );
        assert!(
            svg.contains("animation-delay:100ms"),
            "missing delay for depth 1"
        );
        assert!(
            svg.contains("animation-delay:200ms"),
            "missing delay for depth 2"
        );
        assert!(
            svg.contains(r#"class="sunburst-fade""#),
            "missing fade class on labels"
        );
    }

    #[test]
    fn sunburst_chart_basic_json() {
        let chart = SunburstChart::from_json(
            r##"{
                "title_text": "Sunburst",
                "inner_radius": 20,
                "series_data": [
                    {
                        "name": "Grandpa",
                        "children": [
                            {"name": "Uncle Leo", "children": [
                                {"name": "Cousin Jack", "value": 18},
                                {"name": "Cousin Mary", "value": 12}
                            ]},
                            {"name": "Father", "children": [
                                {"name": "Me", "value": 40},
                                {"name": "Brother Peter", "value": 20}
                            ]}
                        ]
                    },
                    {
                        "name": "Nancy",
                        "children": [
                            {"name": "Uncle Nike", "children": [
                                {"name": "Cousin Betty", "value": 10},
                                {"name": "Cousin Jenny", "value": 30}
                            ]}
                        ]
                    }
                ]
            }"##,
        )
        .unwrap();
        assert_eq!(
            include_str!("../../asset/sunburst_chart/basic_json.svg"),
            chart.svg().unwrap()
        );
    }
}
