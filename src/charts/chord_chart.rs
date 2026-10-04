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

use super::base::{ChartBase, LabelBoxes};
use super::canvas;
use super::color::*;
use super::common::*;
use super::component::*;
use super::params::*;
use super::polar_bar_chart::{Beside, outward};
use super::theme::{get_default_theme_name, get_theme};
use super::util::*;
use crate::charts::measure_text_width_family;

/// Thickness of the node ring unless set.
const DEFAULT_NODE_WIDTH: f32 = 12.0;
/// Gap between neighbouring nodes unless set, in degrees.
const DEFAULT_NODE_GAP: f32 = 3.0;
/// Opacity of the ribbons unless set: they overlap, and have to show
/// through each other.
const DEFAULT_LINK_OPACITY: f32 = 0.5;
/// Gap between the node ring and the ends of the ribbons.
const RIBBON_GAP: f32 = 2.0;
/// Gap between the node ring and the labels around it.
const LABEL_GAP: f32 = 6.0;

// ── Public data model ──────────────────────────────────────────────────────────

/// A node of the diagram, identified by `name`. Links reference nodes by
/// this name.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ChordNode {
    /// Name of the node, shown as its label.
    pub name: String,
    /// Optional explicit color; when `None` the color is taken from the theme
    /// palette by the node's position.
    pub color: Option<Color>,
}

impl From<&str> for ChordNode {
    fn from(name: &str) -> Self {
        ChordNode {
            name: name.to_string(),
            color: None,
        }
    }
}

/// A flow of `value` units between the `source` node and the `target` node
/// (both referenced by name).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ChordLink {
    /// Name of the source node.
    pub source: String,
    /// Name of the target node.
    pub target: String,
    /// Volume of the flow: the width of the ribbon at both of its ends.
    pub value: f32,
}

impl From<(&str, &str, f32)> for ChordLink {
    fn from(v: (&str, &str, f32)) -> Self {
        ChordLink {
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
    /// Sum of the links that start or end at the node.
    value: f32,
    /// The arc of the node, in degrees clockwise from 12 o'clock.
    start: f32,
    end: f32,
}

struct LayoutLink {
    source: usize,
    target: usize,
    value: f32,
    /// The part of the source node's arc the ribbon starts from.
    source_arc: (f32, f32),
    /// The part of the target node's arc the ribbon ends at.
    target_arc: (f32, f32),
}

// ── ChordChart ─────────────────────────────────────────────────────────────────

/// A chord diagram: the nodes are arcs of a circle, as long as the flows
/// through them, and every flow between two nodes is a ribbon across the
/// circle — who exchanges how much with whom, at a glance.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ChordChart {
    /// The shared chart options (size, series, title/legend, axes); exposed
    /// directly on the chart through `Deref`, e.g. `chart.title_text`.
    pub base: ChartBase,
    y_axis_configs: Vec<YAxisConfig>,

    // chord-specific
    /// Diagram nodes, clockwise. May be left empty, in which case nodes are
    /// derived from the names referenced by `links`, in first-seen order.
    pub nodes: Vec<ChordNode>,
    /// Flows between nodes.
    pub links: Vec<ChordLink>,
    /// Largest outer radius; `None` (the default) fills the plot area.
    pub radius: Option<f32>,
    /// Thickness of the node ring in pixels. Default: 12.0.
    pub node_width: f32,
    /// Gap between neighbouring nodes in degrees. Default: 3.0.
    pub node_gap: f32,
    /// Angle the first node starts at, in degrees clockwise from 12 o'clock.
    pub start_angle: f32,
    /// Opacity of the ribbons in `0.0..=1.0`. Default: 0.5.
    pub link_opacity: f32,
    /// When `true`, each ribbon is filled with a source→target color gradient
    /// instead of a translucent source color. Default: false.
    pub link_gradient: bool,
}

impl std::ops::Deref for ChordChart {
    type Target = ChartBase;
    fn deref(&self) -> &ChartBase {
        &self.base
    }
}
impl std::ops::DerefMut for ChordChart {
    fn deref_mut(&mut self) -> &mut ChartBase {
        &mut self.base
    }
}

impl ChordChart {
    fn fill_default(&mut self) {
        self.node_width = DEFAULT_NODE_WIDTH;
        self.node_gap = DEFAULT_NODE_GAP;
        self.link_opacity = DEFAULT_LINK_OPACITY;
    }

    /// Creates a chord chart with the default theme.
    pub fn new(nodes: Vec<ChordNode>, links: Vec<ChordLink>) -> ChordChart {
        ChordChart::new_with_theme(nodes, links, &get_default_theme_name())
    }

    /// Creates a chord chart with a custom theme.
    pub fn new_with_theme(nodes: Vec<ChordNode>, links: Vec<ChordLink>, theme: &str) -> ChordChart {
        let mut c = ChordChart {
            nodes,
            links,
            ..Default::default()
        };
        c.fill_default();
        c.base.fill_theme(get_theme(theme), &mut c.y_axis_configs);
        c
    }

    /// Creates a chord chart from a JSON string.
    pub fn from_json(json: &str) -> canvas::Result<ChordChart> {
        let mut c = ChordChart {
            ..Default::default()
        };
        c.fill_default();
        let value = c
            .base
            .fill_option(json, &mut c.y_axis_configs, super::schema::CHORD_FIELDS)?;
        if let Some(arr) = value.get("nodes").and_then(|v| v.as_array()) {
            c.nodes = arr
                .iter()
                .filter_map(|item| {
                    let name = get_string_from_value(item, "name").unwrap_or_default();
                    if name.is_empty() {
                        return None;
                    }
                    Some(ChordNode {
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
                    Some(ChordLink {
                        source,
                        target,
                        value: get_f32_from_value(item, "value").unwrap_or_default(),
                    })
                })
                .collect();
        }
        if let Some(v) = get_f32_from_value(&value, "radius") {
            c.radius = Some(v);
        }
        if let Some(v) = get_f32_from_value(&value, "node_width") {
            c.node_width = v;
        }
        if let Some(v) = get_f32_from_value(&value, "node_gap") {
            c.node_gap = v;
        }
        if let Some(v) = get_f32_from_value(&value, "start_angle") {
            c.start_angle = v;
        }
        if let Some(v) = get_f32_from_value(&value, "link_opacity") {
            c.link_opacity = v;
        }
        if let Some(v) = get_bool_from_value(&value, "link_gradient") {
            c.link_gradient = v;
        }
        Ok(c)
    }

    /// The arcs of the nodes and of the ends of every link, or `None` when
    /// there is nothing to draw.
    fn layout(&self) -> Option<(Vec<LayoutNode>, Vec<LayoutLink>)> {
        // Collect node names, preserving explicit order then link order.
        let mut names: Vec<&str> = vec![];
        let mut explicit_color: Vec<Option<Color>> = vec![];
        for node in &self.nodes {
            if !names.contains(&node.name.as_str()) {
                names.push(&node.name);
                explicit_color.push(node.color);
            }
        }
        let mut links: Vec<LayoutLink> = vec![];
        for link in &self.links {
            if !link.value.is_finite() || link.value <= 0.0 {
                continue;
            }
            let [source, target] = [&link.source, &link.target].map(|name| {
                names
                    .iter()
                    .position(|n| *n == name.as_str())
                    .unwrap_or_else(|| {
                        names.push(name);
                        explicit_color.push(None);
                        names.len() - 1
                    })
            });
            links.push(LayoutLink {
                source,
                target,
                value: link.value,
                source_arc: (0.0, 0.0),
                target_arc: (0.0, 0.0),
            });
        }
        if links.is_empty() {
            return None;
        }

        let count = names.len();
        let mut nodes: Vec<LayoutNode> = names
            .iter()
            .enumerate()
            .map(|(index, name)| LayoutNode {
                name: name.to_string(),
                color: explicit_color[index]
                    .unwrap_or_else(|| get_color(&self.series_colors, index)),
                value: 0.0,
                start: 0.0,
                end: 0.0,
            })
            .collect();
        // A link takes its width from both of its nodes; one from a node to
        // itself has a single end.
        for link in links.iter() {
            nodes[link.source].value += link.value;
            if link.target != link.source {
                nodes[link.target].value += link.value;
            }
        }

        // The circle is shared by the nodes that have links, a gap after
        // each; a node on its own gets the whole of it.
        let active = nodes.iter().filter(|n| n.value > 0.0).count();
        let total: f32 = nodes.iter().map(|n| n.value).sum();
        if active == 0 || !total.is_finite() || total <= 0.0 {
            return None;
        }
        let gap = if active > 1 && self.node_gap.is_finite() {
            self.node_gap.clamp(0.0, 180.0 / active as f32)
        } else {
            0.0
        };
        let scale = (360.0 - gap * active as f32) / total;
        let start = if self.start_angle.is_finite() {
            self.start_angle
        } else {
            0.0
        };
        let mut cursor = start + gap / 2.0;
        for node in nodes.iter_mut().filter(|n| n.value > 0.0) {
            node.start = cursor;
            node.end = cursor + node.value * scale;
            cursor = node.end + gap;
        }

        // The ends of the links along each node. A link to the next node
        // clockwise leaves from the clockwise end of the arc and one to the
        // previous node from its start, so the ribbons fan out without
        // crossing next to their node; links between the same two nodes are
        // laid in opposite orders at either end for the same reason.
        for (index, node) in nodes.iter().enumerate() {
            let mut ends: Vec<(usize, usize)> = links
                .iter()
                .enumerate()
                .filter(|(_, link)| link.source == index || link.target == index)
                .map(|(i, link)| {
                    let other = if link.source == index {
                        link.target
                    } else {
                        link.source
                    };
                    // How far round the circle the other node is.
                    (i, (other + count - index) % count)
                })
                .collect();
            ends.sort_by(|a, b| {
                b.1.cmp(&a.1).then_with(|| {
                    let other = (index + a.1) % count;
                    if index < other {
                        a.0.cmp(&b.0)
                    } else {
                        b.0.cmp(&a.0)
                    }
                })
            });
            let mut cursor = node.start;
            for (i, _) in ends {
                let link = &mut links[i];
                let arc = (cursor, cursor + link.value * scale);
                cursor = arc.1;
                if link.source == index {
                    link.source_arc = arc;
                }
                if link.target == index {
                    link.target_arc = arc;
                }
            }
        }
        Some((nodes, links))
    }

    /// Renders the chart to an SVG string.
    pub fn svg(&self) -> canvas::Result<String> {
        let mut c = self.new_canvas();
        let axis_top = self.render_header(&mut c);
        let mut content = c.child(Box {
            top: axis_top,
            ..Default::default()
        });
        if content.width() <= 0.0 || content.height() <= 0.0 {
            return c.svg();
        }
        let Some((nodes, links)) = self.layout() else {
            self.render_empty_text(content);
            return c.svg();
        };
        let total: f32 = nodes.iter().map(|n| n.value).sum();

        // The labels: the name of the node, or what the formatter makes of it.
        let font_size = self.series_label_font_size;
        let labels: Vec<(String, f32, f32)> = nodes
            .iter()
            .map(|node| {
                if node.value <= 0.0 {
                    return (String::new(), 0.0, 0.0);
                }
                let text = if self.series_label_formatter.is_empty() {
                    node.name.clone()
                } else {
                    LabelOption {
                        series_name: node.name.clone(),
                        category_name: node.name.clone(),
                        value: node.value,
                        percentage: node.value / total,
                        formatter: self.series_label_formatter.clone(),
                    }
                    .format()
                };
                let (width, height) = if text.is_empty() {
                    (0.0, 0.0)
                } else {
                    measure_text_width_family(&self.font_family, font_size, &text)
                        .map(|b| (b.width(), b.height()))
                        .unwrap_or((0.0, font_size))
                };
                (text, width, height)
            })
            .collect();
        let widest = labels.iter().map(|l| l.1).fold(0.0, f32::max);
        let tallest = labels.iter().map(|l| l.2).fold(0.0, f32::max);
        let room = |size: f32| {
            if size > 0.0 {
                size + LABEL_GAP + 2.0
            } else {
                2.0
            }
        };

        let cx = content.width() / 2.0;
        let cy = content.height() / 2.0;
        let fit = (cx - room(widest)).min(cy - room(tallest)).max(1.0);
        let r = match self.radius {
            Some(radius) if radius.is_finite() && radius > 0.0 => fit.min(radius),
            _ => fit,
        };
        let node_width = if self.node_width.is_finite() {
            self.node_width.clamp(0.0, r / 2.0)
        } else {
            DEFAULT_NODE_WIDTH.min(r / 2.0)
        };
        let ribbon_r = (r - node_width - RIBBON_GAP).max(0.0);
        let opacity = if self.link_opacity.is_finite() {
            self.link_opacity.clamp(0.0, 1.0)
        } else {
            DEFAULT_LINK_OPACITY
        };
        let alpha = (opacity * 255.0).round() as u8;

        let classes = |tooltip: bool| -> Option<String> {
            let mut classes: Vec<&str> = vec![];
            if self.animation.is_some() {
                classes.push("chord-anim");
            }
            if tooltip {
                classes.push("ct-trigger");
            }
            (!classes.is_empty()).then(|| classes.join(" "))
        };
        let delay = |index: usize| {
            self.animation
                .as_ref()
                .map(|a| format!("animation-delay:{}ms", index as u32 * a.delay))
        };
        let tip = |content: &mut canvas::Canvas, text: String, point: Point| {
            content.text_unmeasured(Text {
                text,
                class: Some("ct-tip".to_string()),
                font_family: Some(self.font_family.clone()),
                font_color: Some(self.series_label_font_color),
                font_size: Some(self.series_label_font_size),
                x: Some(point.x),
                y: Some(point.y),
                text_anchor: Some("middle".to_string()),
                dominant_baseline: Some("central".to_string()),
                ..Default::default()
            });
        };

        // ── Ribbons (drawn first so the node ring sits on top) ────────────────
        for link in &links {
            let source = &nodes[link.source];
            let target = &nodes[link.target];
            let middle = |arc: (f32, f32)| (arc.0 + arc.1) / 2.0;
            let from = get_pie_point(cx, cy, ribbon_r, middle(link.source_arc));
            let to = get_pie_point(cx, cy, ribbon_r, middle(link.target_arc));
            let fill = if self.link_gradient && source.color != target.color {
                // From the one end of the ribbon to the other.
                let angle = (to.x - from.x).atan2(to.y - from.y).to_degrees();
                Fill::LinearGradient {
                    start_color: source.color.with_alpha(alpha),
                    end_color: target.color.with_alpha(alpha),
                    angle: (angle.round() + 360.0) % 360.0,
                }
            } else {
                source.color.with_alpha(alpha).into()
            };
            let tooltip_text = self.tooltip_show.then(|| {
                format!(
                    "{} → {}: {}",
                    source.name,
                    target.name,
                    format_float(link.value)
                )
            });
            content.ribbon(Ribbon {
                fill,
                cx,
                cy,
                r: ribbon_r,
                source: link.source_arc,
                target: link.target_arc,
                class: classes(tooltip_text.is_some()),
                style: delay(link.source),
                title: tooltip_text.clone(),
                dataset: vec![
                    ("source".to_string(), source.name.clone()),
                    ("target".to_string(), target.name.clone()),
                    ("value".to_string(), format_float(link.value)),
                ],
            });
            if let Some(text) = tooltip_text {
                // Half way along the ribbon, which bends towards the center.
                let point = if link.source == link.target {
                    get_pie_point(cx, cy, ribbon_r / 2.0, middle(link.source_arc))
                } else {
                    Point {
                        x: (from.x + to.x) / 4.0 + cx / 2.0,
                        y: (from.y + to.y) / 4.0 + cy / 2.0,
                    }
                };
                tip(&mut content, text, point);
            }
        }

        // ── Node ring ─────────────────────────────────────────────────────────
        for (index, node) in nodes.iter().enumerate() {
            if node.value <= 0.0 || node_width <= 0.0 {
                continue;
            }
            let tooltip_text = self
                .tooltip_show
                .then(|| format!("{}: {}", node.name, format_float(node.value)));
            content.sector(Sector {
                fill: node.color.into(),
                cx,
                cy,
                r,
                ir: r - node_width,
                start_angle: node.start,
                delta: node.end - node.start,
                round_cap: false,
                class: classes(tooltip_text.is_some()),
                style: delay(index),
                title: tooltip_text.clone(),
                dataset: vec![
                    ("name".to_string(), node.name.clone()),
                    ("value".to_string(), format_float(node.value)),
                ],
            });
            if let Some(text) = tooltip_text {
                let middle = (node.start + node.end) / 2.0;
                tip(
                    &mut content,
                    text,
                    get_pie_point(cx, cy, r - node_width / 2.0, middle),
                );
            }
        }

        // ── Node labels ───────────────────────────────────────────────────────
        // Around the ring; one that would run into its neighbour is left out.
        let mut boxes = LabelBoxes::new(true);
        for (node, (text, width, height)) in nodes.iter().zip(labels) {
            if text.is_empty() {
                continue;
            }
            let middle = (node.start + node.end) / 2.0;
            let at = Beside::new(
                get_pie_point(cx, cy, r, middle),
                outward(middle),
                LABEL_GAP,
                height,
            );
            if !boxes.try_place(at.left(width), at.y - height / 2.0, width, height) {
                continue;
            }
            content.text_unmeasured(Text {
                text,
                font_family: Some(self.font_family.clone()),
                font_color: Some(self.series_label_font_color),
                font_size: Some(font_size),
                font_weight: self.series_label_font_weight.clone(),
                x: Some(at.x),
                y: Some(at.y),
                text_anchor: Some(at.anchor.to_string()),
                dominant_baseline: Some("central".to_string()),
                class: self.animation.as_ref().map(|_| "chord-fade".to_string()),
                ..Default::default()
            });
        }

        let mut css = String::new();
        if let Some(ref anim) = self.animation {
            css.push_str(&format!(
                "@keyframes chord-grow{{from{{transform:scale(0)}}to{{transform:scale(1)}}}} \
                 @keyframes chord-fade{{from{{opacity:0}}to{{opacity:1}}}} \
                 .chord-anim{{transform-origin:{}px {}px;animation:chord-grow {}ms {} both}} \
                 .chord-fade{{animation:chord-fade {}ms {} both}} ",
                format_float(cx + content.margin.left),
                format_float(cy + content.margin.top),
                anim.duration,
                anim.safe_easing(),
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
    use super::{ChordChart, ChordLink};
    use pretty_assertions::assert_eq;

    fn links() -> Vec<ChordLink> {
        vec![
            ("A", "B", 10.0).into(),
            ("A", "C", 30.0).into(),
            ("B", "C", 20.0).into(),
        ]
    }

    #[test]
    fn nodes_share_the_circle() {
        let mut chart = ChordChart::new(vec![], links());
        chart.node_gap = 0.0;
        let (nodes, links) = chart.layout().unwrap();
        // Every link counts at both of its nodes: 40 + 30 + 50 = 120.
        let arcs: Vec<(&str, f32, f32, f32)> = nodes
            .iter()
            .map(|n| (n.name.as_str(), n.value, n.start, n.end))
            .collect();
        assert_eq!(
            vec![
                ("A", 40.0, 0.0, 120.0),
                ("B", 30.0, 120.0, 210.0),
                ("C", 50.0, 210.0, 360.0),
            ],
            arcs
        );
        // The ends of the links tile their node. At A the link to C (the
        // node before it, going round) comes first, the one to B (the next
        // node) last; at C the link to B comes first and the one to A last.
        let ends: Vec<((f32, f32), (f32, f32))> =
            links.iter().map(|l| (l.source_arc, l.target_arc)).collect();
        assert_eq!(
            vec![
                ((90.0, 120.0), (120.0, 150.0)),
                ((0.0, 90.0), (270.0, 360.0)),
                ((150.0, 210.0), (210.0, 270.0)),
            ],
            ends
        );
    }

    #[test]
    fn gaps_and_start_angle() {
        let mut chart = ChordChart::new(vec![], links());
        chart.node_gap = 12.0;
        chart.start_angle = 90.0;
        let (nodes, _) = chart.layout().unwrap();
        // 360 - 3 * 12 = 324 degrees for 120 units; half a gap before A.
        assert_eq!((96.0, 204.0), (nodes[0].start, nodes[0].end));
        assert_eq!((216.0, 297.0), (nodes[1].start, nodes[1].end));
        assert_eq!((309.0, 444.0), (nodes[2].start, nodes[2].end));
        // A gap cannot take more than half the circle from the nodes.
        chart.node_gap = 1000.0;
        let (nodes, _) = chart.layout().unwrap();
        assert_eq!(60.0, nodes[0].end - nodes[0].start);
    }

    #[test]
    fn links_between_the_same_nodes_do_not_cross() {
        let chart = ChordChart::new(
            vec![],
            vec![("A", "B", 10.0).into(), ("B", "A", 20.0).into()],
        );
        let (_, links) = chart.layout().unwrap();
        let (first, second) = (&links[0], &links[1]);
        // At A the first link comes first, at B it comes last.
        assert!(first.source_arc.0 < second.target_arc.0);
        assert!(first.target_arc.0 > second.source_arc.0);
    }

    #[test]
    fn chord_chart_basic() {
        let mut chart = ChordChart::new(
            vec![],
            vec![
                ("Asia", "Europe", 60.0).into(),
                ("Asia", "Americas", 45.0).into(),
                ("Europe", "Americas", 50.0).into(),
                ("Europe", "Africa", 25.0).into(),
                ("Americas", "Africa", 15.0).into(),
                ("Asia", "Oceania", 20.0).into(),
                ("Oceania", "Americas", 10.0).into(),
            ],
        );
        chart.title_text = "Trade between regions".to_string();
        assert_snapshot!("chord_chart/basic.svg", chart.svg().unwrap());
    }
}
