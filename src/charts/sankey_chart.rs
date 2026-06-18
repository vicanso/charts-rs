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

// ── Public data model ──────────────────────────────────────────────────────────

/// A node in the flow diagram, identified by `name`. Links reference nodes by
/// this name.
#[derive(Clone, Debug, Default)]
pub struct SankeyNode {
    pub name: String,
    /// Optional explicit color; when `None` the color is taken from the theme
    /// palette by the node's position.
    pub color: Option<Color>,
}

impl From<&str> for SankeyNode {
    fn from(name: &str) -> Self {
        SankeyNode {
            name: name.to_string(),
            color: None,
        }
    }
}

/// A directed flow of `value` units from the `source` node to the `target`
/// node (both referenced by name).
#[derive(Clone, Debug, Default)]
pub struct SankeyLink {
    pub source: String,
    pub target: String,
    pub value: f32,
}

impl From<(&str, &str, f32)> for SankeyLink {
    fn from(v: (&str, &str, f32)) -> Self {
        SankeyLink {
            source: v.0.to_string(),
            target: v.1.to_string(),
            value: v.2,
        }
    }
}

// ── Internal layout structures ───────────────────────────────────────────────

struct LayoutNode {
    name: String,
    color: Color,
    /// Throughput of the node: `max(sum of incoming, sum of outgoing)`.
    value: f32,
    layer: usize,
    x: f32,
    y: f32,
    dy: f32,
    in_links: Vec<usize>,
    out_links: Vec<usize>,
}

struct LayoutLink {
    source: usize,
    target: usize,
    value: f32,
    width: f32,
    /// Vertical offset of the band within the source node.
    sy: f32,
    /// Vertical offset of the band within the target node.
    ty: f32,
}

/// Samples a cubic Bézier with horizontal tangents from `(x0, y0)` to
/// `(x1, y1)`; the control points share the midpoint x, producing the classic
/// S-shaped Sankey link edge.
fn sample_link_edge(x0: f32, y0: f32, x1: f32, y1: f32, segments: usize, out: &mut Vec<Point>) {
    let xm = (x0 + x1) / 2.0;
    for i in 0..=segments {
        let t = i as f32 / segments as f32;
        let mt = 1.0 - t;
        let b0 = mt * mt * mt;
        let b1 = 3.0 * mt * mt * t;
        let b2 = 3.0 * mt * t * t;
        let b3 = t * t * t;
        let x = b0 * x0 + b1 * xm + b2 * xm + b3 * x1;
        let y = b0 * y0 + b1 * y0 + b2 * y1 + b3 * y1;
        out.push((x, y).into());
    }
}

// ── SankeyChart ────────────────────────────────────────────────────────────────

#[derive(Clone, Debug, Default, Chart)]
pub struct SankeyChart {
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

    // sankey-specific
    /// Diagram nodes. May be left empty, in which case nodes are derived from
    /// the names referenced by `links`, in first-seen order.
    pub nodes: Vec<SankeyNode>,
    /// Directed flows between nodes.
    pub links: Vec<SankeyLink>,
    /// Width of each node rectangle in pixels. Default: 16.0.
    pub node_width: f32,
    /// Vertical gap between nodes in the same column, in pixels. Default: 8.0.
    pub node_gap: f32,
    /// Opacity of the flow ribbons in `0.0..=1.0`. Default: 0.45.
    pub link_opacity: f32,
    /// Optional expand animation: nodes and links grow horizontally from the
    /// left, staggered column by column (`delay` ms per layer), so the flow
    /// reveals left to right; labels fade in alongside.
    pub animation: Option<AnimationConfig>,
    /// Horizontal node alignment: `"left"` (default — column = longest path
    /// from a source), `"right"` (column = longest path to a sink), or
    /// `"justify"` (sink nodes pushed to the last column).
    pub node_align: Option<String>,
    /// When `true`, each link is filled with a source→target color gradient
    /// instead of a translucent source color. Default: false.
    pub link_gradient: bool,
}

/// Number of iterations of the relaxation pass that reduces link crossings.
/// Fixed so the layout (and therefore the SVG) is deterministic.
const RELAX_ITERATIONS: usize = 32;
/// Number of straight segments used to approximate each Bézier link edge.
const LINK_SEGMENTS: usize = 24;

impl SankeyChart {
    fn fill_default(&mut self) {
        if self.node_width <= 0.0 {
            self.node_width = 16.0;
        }
        if self.node_gap <= 0.0 {
            self.node_gap = 8.0;
        }
        if self.link_opacity <= 0.0 {
            self.link_opacity = 0.45;
        }
        self.link_opacity = self.link_opacity.min(1.0);
    }

    /// Creates a sankey chart with the default theme.
    pub fn new(nodes: Vec<SankeyNode>, links: Vec<SankeyLink>) -> SankeyChart {
        SankeyChart::new_with_theme(nodes, links, &get_default_theme_name())
    }

    /// Creates a sankey chart with a custom theme.
    pub fn new_with_theme(
        nodes: Vec<SankeyNode>,
        links: Vec<SankeyLink>,
        theme: &str,
    ) -> SankeyChart {
        let mut c = SankeyChart {
            nodes,
            links,
            ..Default::default()
        };
        c.fill_theme(get_theme(theme));
        c.fill_default();
        c
    }

    /// Creates a sankey chart from a JSON string.
    pub fn from_json(json: &str) -> canvas::Result<SankeyChart> {
        let mut c = SankeyChart {
            ..Default::default()
        };
        let value = c.fill_option(json)?;
        if let Some(arr) = value.get("nodes").and_then(|v| v.as_array()) {
            c.nodes = arr
                .iter()
                .filter_map(|item| {
                    let name = get_string_from_value(item, "name").unwrap_or_default();
                    if name.is_empty() {
                        return None;
                    }
                    Some(SankeyNode {
                        name,
                        color: get_color_from_value(item, "color"),
                    })
                })
                .collect();
        }
        if let Some(arr) = value.get("links").and_then(|v| v.as_array()) {
            c.links = arr
                .iter()
                .filter_map(|item| {
                    let source = get_string_from_value(item, "source").unwrap_or_default();
                    let target = get_string_from_value(item, "target").unwrap_or_default();
                    if source.is_empty() || target.is_empty() {
                        return None;
                    }
                    Some(SankeyLink {
                        source,
                        target,
                        value: get_f32_from_value(item, "value").unwrap_or_default(),
                    })
                })
                .collect();
        }
        if let Some(v) = get_f32_from_value(&value, "node_width") {
            c.node_width = v;
        }
        if let Some(v) = get_f32_from_value(&value, "node_gap") {
            c.node_gap = v;
        }
        if let Some(v) = get_f32_from_value(&value, "link_opacity") {
            c.link_opacity = v;
        }
        if let Some(s) = get_string_from_value(&value, "node_align") {
            c.node_align = Some(s);
        }
        if let Some(b) = get_bool_from_value(&value, "link_gradient") {
            c.link_gradient = b;
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
        c.fill_default();
        Ok(c)
    }

    /// Builds the laid-out nodes and links, or `None` when there is nothing to
    /// draw. `cw`/`ch` are the plotting area dimensions.
    fn layout(&self, cw: f32, ch: f32) -> Option<(Vec<LayoutNode>, Vec<LayoutLink>)> {
        // Collect node names, preserving explicit order then link order.
        let mut names: Vec<String> = vec![];
        let mut explicit_color: Vec<Option<Color>> = vec![];
        for n in &self.nodes {
            if !names.contains(&n.name) {
                names.push(n.name.clone());
                explicit_color.push(n.color);
            }
        }
        let index_of = |names: &[String], name: &str| names.iter().position(|n| n == name);

        let mut links: Vec<LayoutLink> = vec![];
        for link in &self.links {
            if link.value <= 0.0 {
                continue;
            }
            for name in [&link.source, &link.target] {
                if index_of(&names, name).is_none() {
                    names.push(name.clone());
                    explicit_color.push(None);
                }
            }
            let source = index_of(&names, &link.source)?;
            let target = index_of(&names, &link.target)?;
            if source == target {
                continue;
            }
            links.push(LayoutLink {
                source,
                target,
                value: link.value,
                width: 0.0,
                sy: 0.0,
                ty: 0.0,
            });
        }
        if links.is_empty() || names.is_empty() {
            return None;
        }

        let node_count = names.len();
        let mut nodes: Vec<LayoutNode> = names
            .iter()
            .enumerate()
            .map(|(i, name)| {
                let color = explicit_color[i].unwrap_or_else(|| get_color(&self.series_colors, i));
                LayoutNode {
                    name: name.clone(),
                    color,
                    value: 0.0,
                    layer: 0,
                    x: 0.0,
                    y: 0.0,
                    dy: 0.0,
                    in_links: vec![],
                    out_links: vec![],
                }
            })
            .collect();

        // Wire links to nodes and accumulate per-node in/out totals.
        let mut in_sum = vec![0.0_f32; node_count];
        let mut out_sum = vec![0.0_f32; node_count];
        for (li, link) in links.iter().enumerate() {
            nodes[link.source].out_links.push(li);
            nodes[link.target].in_links.push(li);
            out_sum[link.source] += link.value;
            in_sum[link.target] += link.value;
        }
        for (i, node) in nodes.iter_mut().enumerate() {
            node.value = in_sum[i].max(out_sum[i]);
        }

        // Longest-path layering: every link pushes its target at least one
        // column to the right of its source (Bellman-Ford style relaxation).
        for _ in 0..node_count {
            let mut changed = false;
            for link in &links {
                let want = nodes[link.source].layer + 1;
                if nodes[link.target].layer < want {
                    nodes[link.target].layer = want;
                    changed = true;
                }
            }
            if !changed {
                break;
            }
        }
        let layer_count = nodes.iter().map(|n| n.layer).max().unwrap_or(0) + 1;

        // Node alignment. "left" (default) keeps the longest-path-from-source
        // columns computed above; "justify" pushes sinks to the last column;
        // "right" re-columns every node by its longest path to a sink.
        match self.node_align.as_deref() {
            Some("justify") => {
                for node in nodes.iter_mut() {
                    if node.out_links.is_empty() {
                        node.layer = layer_count - 1;
                    }
                }
            }
            Some("right") => {
                let mut backward = vec![0usize; node_count];
                for _ in 0..node_count {
                    let mut changed = false;
                    for link in &links {
                        let want = backward[link.target] + 1;
                        if backward[link.source] < want {
                            backward[link.source] = want;
                            changed = true;
                        }
                    }
                    if !changed {
                        break;
                    }
                }
                for (i, node) in nodes.iter_mut().enumerate() {
                    node.layer = (layer_count - 1) - backward[i];
                }
            }
            _ => {}
        }

        // Column x positions.
        for node in nodes.iter_mut() {
            node.x = if layer_count <= 1 {
                0.0
            } else {
                node.layer as f32 / (layer_count - 1) as f32 * (cw - self.node_width)
            };
        }

        // Group node indices by layer (input order within a layer).
        let mut layers_nodes: Vec<Vec<usize>> = vec![vec![]; layer_count];
        for (i, node) in nodes.iter().enumerate() {
            layers_nodes[node.layer].push(i);
        }

        // Vertical scale: pick the factor so the busiest column just fits.
        let mut ky = f32::MAX;
        for layer in &layers_nodes {
            let sum: f32 = layer.iter().map(|&i| nodes[i].value).sum();
            if sum <= 0.0 {
                continue;
            }
            let avail = ch - (layer.len() as f32 - 1.0) * self.node_gap;
            if avail <= 0.0 {
                return None;
            }
            ky = ky.min(avail / sum);
        }
        if !ky.is_finite() || ky <= 0.0 {
            return None;
        }

        for node in nodes.iter_mut() {
            node.dy = node.value * ky;
        }
        for link in links.iter_mut() {
            link.width = link.value * ky;
        }

        // Initial stacking: index within column, then resolve overlaps.
        for layer in &layers_nodes {
            for (i, &ni) in layer.iter().enumerate() {
                nodes[ni].y = i as f32;
            }
        }
        resolve_collisions(&mut nodes, &layers_nodes, ch, self.node_gap);

        // Relaxation to reduce crossings (deterministic, fixed iterations).
        let mut alpha = 1.0_f32;
        for _ in 0..RELAX_ITERATIONS {
            alpha *= 0.99;
            relax(&mut nodes, &links, &layers_nodes, alpha, true);
            resolve_collisions(&mut nodes, &layers_nodes, ch, self.node_gap);
            relax(&mut nodes, &links, &layers_nodes, alpha, false);
            resolve_collisions(&mut nodes, &layers_nodes, ch, self.node_gap);
        }

        // Stack link bands within each node, ordered by the opposite endpoint's
        // vertical center so ribbons fan out without crossing inside a node.
        for ni in 0..node_count {
            let mut outs = nodes[ni].out_links.clone();
            outs.sort_by(|&a, &b| {
                center(&nodes[links[a].target])
                    .partial_cmp(&center(&nodes[links[b].target]))
                    .unwrap_or(std::cmp::Ordering::Equal)
            });
            let mut sy = 0.0;
            for li in outs {
                links[li].sy = sy;
                sy += links[li].width;
            }

            let mut ins = nodes[ni].in_links.clone();
            ins.sort_by(|&a, &b| {
                center(&nodes[links[a].source])
                    .partial_cmp(&center(&nodes[links[b].source]))
                    .unwrap_or(std::cmp::Ordering::Equal)
            });
            let mut ty = 0.0;
            for li in ins {
                links[li].ty = ty;
                ty += links[li].width;
            }
        }

        Some((nodes, links))
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
        if cw <= 0.0 || ch <= 0.0 {
            return c.svg();
        }

        let Some((nodes, links)) = self.layout(cw, ch) else {
            return c.svg();
        };

        let grand_total: f32 = {
            let inflow: f32 = nodes.iter().filter(|n| n.layer == 0).map(|n| n.value).sum();
            if inflow > 0.0 {
                inflow
            } else {
                nodes.iter().map(|n| n.value).sum()
            }
        };

        // ── Links (drawn first so node rectangles sit on top) ─────────────────
        let alpha = (self.link_opacity.clamp(0.0, 1.0) * 255.0).round() as u8;
        for link in &links {
            let source = &nodes[link.source];
            let target = &nodes[link.target];
            let x0 = source.x + self.node_width;
            let x1 = target.x;
            let top_s = source.y + link.sy;
            let top_t = target.y + link.ty;

            let mut points: Vec<Point> = vec![];
            // Top edge: source → target.
            sample_link_edge(x0, top_s, x1, top_t, LINK_SEGMENTS, &mut points);
            // Bottom edge: target → source (reversed) closes the ribbon.
            sample_link_edge(
                x1,
                top_t + link.width,
                x0,
                top_s + link.width,
                LINK_SEGMENTS,
                &mut points,
            );

            let (fill, gradient) = if self.link_gradient {
                (
                    None,
                    Some(Fill::LinearGradient {
                        start_color: source.color.with_alpha(alpha),
                        end_color: target.color.with_alpha(alpha),
                        // 90 degrees = left (source) to right (target).
                        angle: 90.0,
                    }),
                )
            } else {
                (Some(source.color.with_alpha(alpha)), None)
            };
            content.polygon(Polygon {
                color: None,
                fill,
                gradient,
                points,
                class: self.animation.as_ref().map(|_| "sankey-anim".to_string()),
                style: self
                    .animation
                    .as_ref()
                    .map(|a| format!("animation-delay:{}ms", source.layer as u32 * a.delay)),
            });
        }

        // ── Node rectangles ───────────────────────────────────────────────────
        for node in &nodes {
            if node.dy <= 0.0 {
                continue;
            }
            content.rect(Rect {
                fill: Some(node.color.into()),
                left: node.x,
                top: node.y,
                width: self.node_width,
                height: node.dy,
                class: self.animation.as_ref().map(|_| "sankey-anim".to_string()),
                style: self
                    .animation
                    .as_ref()
                    .map(|a| format!("animation-delay:{}ms", node.layer as u32 * a.delay)),
                ..Default::default()
            });
        }

        // ── Node labels ───────────────────────────────────────────────────────
        let font_size = self.series_label_font_size.max(10.0);
        let font_color = self.series_label_font_color;
        for node in &nodes {
            if node.dy <= 0.0 {
                continue;
            }
            let text = if self.series_label_formatter.is_empty() {
                node.name.clone()
            } else {
                LabelOption {
                    series_name: node.name.clone(),
                    category_name: node.name.clone(),
                    value: node.value,
                    percentage: if grand_total > 0.0 {
                        node.value / grand_total
                    } else {
                        0.0
                    },
                    formatter: self.series_label_formatter.clone(),
                }
                .format()
            };
            if text.is_empty() {
                continue;
            }
            // Label outside the node: to the right for left-half nodes, to the
            // left otherwise, keeping text within the content area.
            let mid_y = node.y + node.dy / 2.0;
            let (x, anchor) = if node.x + self.node_width / 2.0 < cw / 2.0 {
                (node.x + self.node_width + 5.0, "start")
            } else {
                (node.x - 5.0, "end")
            };
            content.text(Text {
                text,
                font_family: Some(self.font_family.clone()),
                font_color: Some(font_color),
                font_size: Some(font_size),
                font_weight: self.series_label_font_weight.clone(),
                x: Some(x),
                y: Some(mid_y),
                text_anchor: Some(anchor.to_string()),
                dominant_baseline: Some("central".to_string()),
                class: self.animation.as_ref().map(|_| "sankey-fade".to_string()),
                ..Default::default()
            });
        }

        if let Some(ref anim) = self.animation {
            let css = format!(
                "@keyframes sankey-grow{{from{{transform:scaleX(0)}}to{{transform:scaleX(1)}}}} \
                 @keyframes sankey-fade{{from{{opacity:0}}to{{opacity:1}}}} \
                 .sankey-anim{{transform-box:fill-box;transform-origin:left center;\
                 animation:sankey-grow {}ms {} both}} \
                 .sankey-fade{{animation:sankey-fade {}ms {} both}}",
                anim.duration, anim.easing, anim.duration, anim.easing
            );
            c.svg_with_style(&css)
        } else {
            c.svg()
        }
    }
}

/// Vertical center of a node.
fn center(node: &LayoutNode) -> f32 {
    node.y + node.dy / 2.0
}

/// Pulls each node toward the weighted center of its connected neighbours.
/// `use_targets` selects the right-to-left pass (align to outgoing targets);
/// otherwise the left-to-right pass aligns to incoming sources.
fn relax(
    nodes: &mut [LayoutNode],
    links: &[LayoutLink],
    layers_nodes: &[Vec<usize>],
    alpha: f32,
    use_targets: bool,
) {
    // Right-to-left walks columns in descending order; left-to-right ascending.
    let order: Vec<usize> = if use_targets {
        (0..layers_nodes.len()).rev().collect()
    } else {
        (0..layers_nodes.len()).collect()
    };
    for layer_idx in order {
        for &ni in &layers_nodes[layer_idx] {
            let (cur_center, weighted, has) = {
                let node = &nodes[ni];
                let link_ids = if use_targets {
                    &node.out_links
                } else {
                    &node.in_links
                };
                if link_ids.is_empty() {
                    (0.0, 0.0, false)
                } else {
                    let mut sum_v = 0.0;
                    let mut acc = 0.0;
                    for &li in link_ids {
                        let other = if use_targets {
                            &nodes[links[li].target]
                        } else {
                            &nodes[links[li].source]
                        };
                        acc += center(other) * links[li].value;
                        sum_v += links[li].value;
                    }
                    if sum_v <= 0.0 {
                        (0.0, 0.0, false)
                    } else {
                        (center(node), acc / sum_v, true)
                    }
                }
            };
            if has {
                nodes[ni].y += (weighted - cur_center) * alpha;
            }
        }
    }
}

/// Within each column, push overlapping nodes apart (respecting `node_gap`) and
/// keep the column inside `[0, height]`.
fn resolve_collisions(
    nodes: &mut [LayoutNode],
    layers_nodes: &[Vec<usize>],
    height: f32,
    node_gap: f32,
) {
    for layer in layers_nodes {
        if layer.is_empty() {
            continue;
        }
        let mut order = layer.clone();
        order.sort_by(|&a, &b| {
            nodes[a]
                .y
                .partial_cmp(&nodes[b].y)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        // Push down from the top.
        let mut y0 = 0.0_f32;
        for &ni in &order {
            let dy = y0 - nodes[ni].y;
            if dy > 0.0 {
                nodes[ni].y += dy;
            }
            y0 = nodes[ni].y + nodes[ni].dy + node_gap;
        }

        // If the column overflows the bottom, push back up.
        let last = *order.last().unwrap();
        let overflow = nodes[last].y + nodes[last].dy - height;
        if overflow > 0.0 {
            nodes[last].y -= overflow;
            let mut y_limit = nodes[last].y;
            for &ni in order.iter().rev().skip(1) {
                let dy = nodes[ni].y + nodes[ni].dy + node_gap - y_limit;
                if dy > 0.0 {
                    nodes[ni].y -= dy;
                }
                y_limit = nodes[ni].y;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{SankeyChart, SankeyLink, SankeyNode};
    use pretty_assertions::assert_eq;

    fn make_links() -> Vec<SankeyLink> {
        // Fuels fan out through two carriers and back into three sectors, so the
        // layout exercises branching, crossings and varied ribbon widths.
        vec![
            ("Coal", "Electricity", 25.0).into(),
            ("Coal", "Heat", 10.0).into(),
            ("Gas", "Electricity", 15.0).into(),
            ("Gas", "Heat", 20.0).into(),
            ("Solar", "Electricity", 10.0).into(),
            ("Electricity", "Residential", 18.0).into(),
            ("Electricity", "Industrial", 22.0).into(),
            ("Electricity", "Commercial", 10.0).into(),
            ("Heat", "Residential", 12.0).into(),
            ("Heat", "Industrial", 18.0).into(),
        ]
    }

    #[test]
    fn sankey_chart_basic() {
        // Nodes auto-derived from the links.
        let chart = SankeyChart::new(vec![], make_links());
        assert_eq!(
            include_str!("../../asset/sankey_chart/basic.svg"),
            chart.svg().unwrap()
        );
    }

    #[test]
    fn sankey_chart_basic_json() {
        let chart = SankeyChart::from_json(
            r##"{
                "title_text": "Energy Flow",
                "nodes": [
                    {"name": "Coal"},
                    {"name": "Gas"},
                    {"name": "Solar"},
                    {"name": "Electricity"},
                    {"name": "Heat"},
                    {"name": "Residential"},
                    {"name": "Industrial"},
                    {"name": "Commercial"}
                ],
                "links": [
                    {"source": "Coal", "target": "Electricity", "value": 25},
                    {"source": "Coal", "target": "Heat", "value": 10},
                    {"source": "Gas", "target": "Electricity", "value": 15},
                    {"source": "Gas", "target": "Heat", "value": 20},
                    {"source": "Solar", "target": "Electricity", "value": 10},
                    {"source": "Electricity", "target": "Residential", "value": 18},
                    {"source": "Electricity", "target": "Industrial", "value": 22},
                    {"source": "Electricity", "target": "Commercial", "value": 10},
                    {"source": "Heat", "target": "Residential", "value": 12},
                    {"source": "Heat", "target": "Industrial", "value": 18}
                ]
            }"##,
        )
        .unwrap();
        assert_eq!(
            include_str!("../../asset/sankey_chart/basic_json.svg"),
            chart.svg().unwrap()
        );
    }

    #[test]
    fn sankey_chart_label_formatter() {
        let mut chart = SankeyChart::new(
            vec![SankeyNode::from("a"), SankeyNode::from("b")],
            vec![("a", "b", 10.0).into()],
        );
        chart.series_label_formatter = "{b}: {c}".to_string();
        let svg = chart.svg().unwrap();
        // Node "a" is the sole layer-0 node, so its value (10) is the grand total.
        assert!(svg.contains("a: 10"), "missing formatted source label");
        assert!(svg.contains("b: 10"), "missing formatted target label");
    }

    #[test]
    fn sankey_chart_animation() {
        let mut chart = SankeyChart::new(vec![], make_links());
        chart.animation = Some(super::AnimationConfig {
            duration: 700,
            easing: "linear".to_string(),
            delay: 100,
        });
        let svg = chart.svg().unwrap();
        assert!(
            svg.contains("sankey-grow"),
            "missing @keyframes sankey-grow"
        );
        assert!(
            svg.contains(r#"class="sankey-anim""#),
            "missing class on node/link"
        );
        assert!(
            svg.contains(r#"class="sankey-fade""#),
            "missing fade class on labels"
        );
        assert!(svg.contains("700ms linear"), "missing duration/easing");
        // Layer-0 nodes (Coal/Gas/Solar) have delay 0; layer-1 (Electricity/Heat) 100ms.
        assert!(svg.contains("animation-delay:0ms"), "missing layer-0 delay");
        assert!(
            svg.contains("animation-delay:100ms"),
            "missing layer-1 delay"
        );
    }

    #[test]
    fn sankey_chart_link_gradient() {
        let mut chart = SankeyChart::new(vec![], make_links());
        chart.link_gradient = true;
        let svg = chart.svg().unwrap();
        assert!(svg.contains("<linearGradient"), "missing gradient def");
        assert!(
            svg.contains("url(#grad_"),
            "links should reference a gradient"
        );
    }

    #[test]
    fn sankey_chart_node_align() {
        // a -> b -> c is the long path; x is a sink reachable directly from a.
        let links: Vec<SankeyLink> = vec![
            ("a", "b", 4.0).into(),
            ("b", "c", 4.0).into(),
            ("a", "x", 2.0).into(),
        ];
        let left = SankeyChart::new(vec![], links.clone()).svg().unwrap();
        let mut justify = SankeyChart::new(vec![], links);
        justify.node_align = Some("justify".to_string());
        // "justify" moves the sink "x" from the middle column to the last one,
        // so the layout (and therefore the SVG) must change.
        assert_ne!(left, justify.svg().unwrap());
    }

    #[test]
    fn sankey_chart_empty_links() {
        // No links → nothing to lay out, but the header still renders.
        let chart = SankeyChart::new(vec![SankeyNode::from("a")], vec![]);
        assert!(chart.svg().unwrap().starts_with("<svg"));
    }
}
