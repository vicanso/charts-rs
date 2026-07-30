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
use super::base::ChartBase;
use super::canvas;
use super::color::*;
use super::common::*;
use super::component::*;
use super::params::*;
use super::theme::{get_default_theme_name, get_theme};
use super::util::*;

// ── Public data model ──────────────────────────────────────────────────────────

/// A node in the relationship graph, identified by `name`. Links reference nodes
/// by this name.
#[derive(Clone, Debug, Default)]
pub struct GraphNode {
    pub name: String,
    /// Relative importance; scales the node's circle radius. Default 0 → all
    /// nodes share the base `symbol_size`.
    pub value: f32,
    /// Optional explicit color; when `None` the color comes from the palette by
    /// `category` (or the node's position).
    pub color: Option<Color>,
    /// Optional category index; nodes in the same category share a palette color.
    pub category: Option<usize>,
}

impl From<&str> for GraphNode {
    fn from(name: &str) -> Self {
        GraphNode {
            name: name.to_string(),
            ..Default::default()
        }
    }
}

/// An undirected relationship of optional `value` weight between the `source`
/// and `target` nodes (both referenced by name).
#[derive(Clone, Debug, Default)]
pub struct GraphLink {
    pub source: String,
    pub target: String,
    pub value: f32,
}

impl From<(&str, &str)> for GraphLink {
    fn from(v: (&str, &str)) -> Self {
        GraphLink {
            source: v.0.to_string(),
            target: v.1.to_string(),
            value: 0.0,
        }
    }
}
impl From<(&str, &str, f32)> for GraphLink {
    fn from(v: (&str, &str, f32)) -> Self {
        GraphLink {
            source: v.0.to_string(),
            target: v.1.to_string(),
            value: v.2,
        }
    }
}

// ── GraphChart ───────────────────────────────────────────────────────────────

/// A relationship (network) graph: nodes connected by arbitrary edges, laid out
/// either by a deterministic force simulation (`"force"`, the default) or evenly
/// on a circle (`"circular"`). Unlike [`TreeChart`](super::TreeChart) (strictly
/// hierarchical) or [`SankeyChart`](super::SankeyChart) (directed flow), the
/// edges here may form any topology.
#[derive(Clone, Debug, Default)]
pub struct GraphChart {
    /// The shared chart options (size, series, title/legend, axes); exposed
    /// directly on the chart through `Deref`, e.g. `chart.title_text`.
    pub base: ChartBase,
    y_axis_configs: Vec<YAxisConfig>,

    // graph-specific
    /// Graph nodes. May be left empty, in which case nodes are derived from the
    /// names referenced by `links`, in first-seen order.
    pub nodes: Vec<GraphNode>,
    /// Edges between nodes.
    pub links: Vec<GraphLink>,
    /// Base node circle radius in pixels. Default: 10.0.
    pub symbol_size: f32,
    /// Layout: `"force"` (default, force-directed) or `"circular"`.
    pub layout: Option<String>,
}

impl std::ops::Deref for GraphChart {
    type Target = ChartBase;
    fn deref(&self) -> &ChartBase {
        &self.base
    }
}
impl std::ops::DerefMut for GraphChart {
    fn deref_mut(&mut self) -> &mut ChartBase {
        &mut self.base
    }
}

/// Fixed number of force-simulation iterations; fixed so the layout — and thus
/// the SVG — is fully deterministic.
const FORCE_ITERATIONS: usize = 300;

impl GraphChart {
    fn fill_default(&mut self) {
        if self.symbol_size <= 0.0 {
            self.symbol_size = 10.0;
        }
    }

    /// Creates a graph chart with the default theme.
    pub fn new(nodes: Vec<GraphNode>, links: Vec<GraphLink>) -> GraphChart {
        GraphChart::new_with_theme(nodes, links, &get_default_theme_name())
    }

    /// Creates a graph chart with a custom theme.
    pub fn new_with_theme(nodes: Vec<GraphNode>, links: Vec<GraphLink>, theme: &str) -> GraphChart {
        let mut c = GraphChart {
            nodes,
            links,
            ..Default::default()
        };
        c.base.fill_theme(get_theme(theme), &mut c.y_axis_configs);
        c.fill_default();
        c
    }

    /// Creates a graph chart from a JSON string.
    pub fn from_json(json: &str) -> canvas::Result<GraphChart> {
        let mut c = GraphChart {
            ..Default::default()
        };
        let value = c.base.fill_option(json, &mut c.y_axis_configs)?;
        if let Some(arr) = value.get("nodes").and_then(|v| v.as_array()) {
            c.nodes = arr
                .iter()
                .filter_map(|item| {
                    let name = get_string_from_value(item, "name").unwrap_or_default();
                    if name.is_empty() {
                        return None;
                    }
                    Some(GraphNode {
                        name,
                        value: get_f32_from_value(item, "value").unwrap_or_default(),
                        color: get_color_from_value(item, "color"),
                        category: get_usize_from_value(item, "category"),
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
                    Some(GraphLink {
                        source,
                        target,
                        value: get_f32_from_value(item, "value").unwrap_or_default(),
                    })
                })
                .collect();
        }
        if let Some(v) = get_f32_from_value(&value, "symbol_size") {
            c.symbol_size = v;
        }
        if let Some(s) = get_string_from_value(&value, "layout") {
            c.layout = Some(s);
        }
        c.fill_default();
        Ok(c)
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

        // Collect node names (explicit first, then link-referenced), with color
        // and value bookkeeping in parallel.
        let mut names: Vec<String> = vec![];
        let mut colors: Vec<Color> = vec![];
        let mut values: Vec<f32> = vec![];
        for (i, n) in self.nodes.iter().enumerate() {
            if names.contains(&n.name) {
                continue;
            }
            let color = n
                .color
                .unwrap_or_else(|| get_color(&self.series_colors, n.category.unwrap_or(i)));
            names.push(n.name.clone());
            colors.push(color);
            values.push(n.value.max(0.0));
        }
        let index_of = |names: &[String], name: &str| names.iter().position(|n| n == name);

        let mut edges: Vec<(usize, usize)> = vec![];
        for link in &self.links {
            for name in [&link.source, &link.target] {
                if index_of(&names, name).is_none() {
                    let color = get_color(&self.series_colors, names.len());
                    names.push(name.clone());
                    colors.push(color);
                    values.push(0.0);
                }
            }
            let (Some(a), Some(b)) = (
                index_of(&names, &link.source),
                index_of(&names, &link.target),
            ) else {
                continue;
            };
            if a != b {
                edges.push((a, b));
            }
        }

        let n = names.len();
        if n == 0 {
            return c.svg();
        }

        // Node radii: base size, scaled by sqrt(value) when values are provided.
        let max_val = values.iter().cloned().fold(0.0_f32, f32::max);
        let radii: Vec<f32> = values
            .iter()
            .map(|&v| {
                if max_val > 0.0 && v > 0.0 {
                    self.symbol_size * (0.6 + 0.8 * (v / max_val).sqrt())
                } else {
                    self.symbol_size
                }
            })
            .collect();
        let max_r = radii.iter().cloned().fold(self.symbol_size, f32::max);

        // Initial positions: evenly on a circle (deterministic seed).
        let mut xs = vec![0.0_f32; n];
        let mut ys = vec![0.0_f32; n];
        let cx = cw / 2.0;
        let cy = ch / 2.0;
        let init_r = (cw.min(ch) / 2.0 - max_r).max(1.0);
        for i in 0..n {
            let a = std::f32::consts::TAU * i as f32 / n as f32;
            xs[i] = cx + init_r * a.cos();
            ys[i] = cy + init_r * a.sin();
        }

        let circular = self.layout.as_deref() == Some("circular");
        if !circular && n > 1 {
            // Deterministic Fruchterman–Reingold force-directed layout.
            let area = cw * ch;
            let k = (area / n as f32).sqrt().max(1.0);
            let mut dx = vec![0.0_f32; n];
            let mut dy = vec![0.0_f32; n];
            for iter in 0..FORCE_ITERATIONS {
                dx.iter_mut().for_each(|v| *v = 0.0);
                dy.iter_mut().for_each(|v| *v = 0.0);
                // Repulsion between every pair.
                for i in 0..n {
                    for j in (i + 1)..n {
                        let ddx = xs[i] - xs[j];
                        let ddy = ys[i] - ys[j];
                        let dist = ddx.hypot(ddy).max(0.01);
                        let force = k * k / dist;
                        let ux = ddx / dist * force;
                        let uy = ddy / dist * force;
                        dx[i] += ux;
                        dy[i] += uy;
                        dx[j] -= ux;
                        dy[j] -= uy;
                    }
                }
                // Attraction along edges.
                for &(a, b) in &edges {
                    let ddx = xs[a] - xs[b];
                    let ddy = ys[a] - ys[b];
                    let dist = ddx.hypot(ddy).max(0.01);
                    let force = dist * dist / k;
                    let ux = ddx / dist * force;
                    let uy = ddy / dist * force;
                    dx[a] -= ux;
                    dy[a] -= uy;
                    dx[b] += ux;
                    dy[b] += uy;
                }
                // Cooling: limit each step to the current temperature.
                let temp = cw.max(ch) * 0.1 * (1.0 - iter as f32 / FORCE_ITERATIONS as f32);
                for i in 0..n {
                    let d = dx[i].hypot(dy[i]);
                    if d > 0.0 {
                        let step = d.min(temp);
                        xs[i] += dx[i] / d * step;
                        ys[i] += dy[i] / d * step;
                    }
                }
            }
        }

        // Normalize the laid-out positions into the content box, leaving room for
        // node radii and the labels drawn just beneath each node.
        let font_size = self.series_label_font_size.max(10.0);
        let (mut min_x, mut max_x, mut min_y, mut max_y) = (f32::MAX, f32::MIN, f32::MAX, f32::MIN);
        for i in 0..n {
            min_x = min_x.min(xs[i]);
            max_x = max_x.max(xs[i]);
            min_y = min_y.min(ys[i]);
            max_y = max_y.max(ys[i]);
        }
        let span_x = (max_x - min_x).max(1e-3);
        let span_y = (max_y - min_y).max(1e-3);
        let pad = max_r + 2.0;
        let avail_w = (cw - 2.0 * pad).max(1.0);
        let avail_h = (ch - 2.0 * pad - font_size).max(1.0);
        let scale = (avail_w / span_x).min(avail_h / span_y);
        let off_x = pad + (avail_w - span_x * scale) / 2.0;
        let off_y = pad + (avail_h - span_y * scale) / 2.0;
        for i in 0..n {
            xs[i] = off_x + (xs[i] - min_x) * scale;
            ys[i] = off_y + (ys[i] - min_y) * scale;
        }

        // ── Edges (drawn first, under the nodes) ──────────────────────────────
        for &(a, b) in &edges {
            content.line(Line {
                color: Some(self.grid_stroke_color),
                stroke_width: self.grid_stroke_width.max(1.0),
                left: xs[a],
                top: ys[a],
                right: xs[b],
                bottom: ys[b],
                ..Default::default()
            });
        }

        // ── Nodes ─────────────────────────────────────────────────────────────
        for i in 0..n {
            content.circle(Circle {
                fill: Some(colors[i]),
                stroke_color: Some(self.background_color),
                stroke_width: 1.0,
                cx: xs[i],
                cy: ys[i],
                r: radii[i],
                ..Default::default()
            });
        }

        // ── Labels ────────────────────────────────────────────────────────────
        for i in 0..n {
            if names[i].is_empty() {
                continue;
            }
            content.text(Text {
                text: names[i].clone(),
                font_family: Some(self.font_family.clone()),
                font_color: Some(self.series_label_font_color),
                font_size: Some(font_size),
                font_weight: self.series_label_font_weight.clone(),
                x: Some(xs[i]),
                y: Some(ys[i] + radii[i] + font_size * 0.7),
                text_anchor: Some("middle".to_string()),
                dominant_baseline: Some("central".to_string()),
                ..Default::default()
            });
        }

        c.svg()
    }
}

#[cfg(test)]
mod tests {
    use super::{GraphChart, GraphLink, GraphNode};
    use pretty_assertions::{assert_eq, assert_ne};

    fn make_links() -> Vec<GraphLink> {
        vec![
            ("A", "B").into(),
            ("A", "C").into(),
            ("B", "C").into(),
            ("C", "D").into(),
            ("D", "E").into(),
            ("E", "A").into(),
        ]
    }

    #[test]
    fn graph_basic() {
        // Nodes auto-derived from the links; deterministic force layout.
        assert_eq!(
            include_str!("../../asset/graph_chart/basic.svg"),
            GraphChart::new(vec![], make_links()).svg().unwrap()
        );
    }

    #[test]
    fn graph_deterministic() {
        // The force layout must be reproducible run to run.
        let a = GraphChart::new(vec![], make_links()).svg().unwrap();
        let b = GraphChart::new(vec![], make_links()).svg().unwrap();
        assert_eq!(a, b, "force layout must be deterministic");
    }

    #[test]
    fn graph_circular() {
        let mut chart = GraphChart::new(vec![], make_links());
        let force = chart.svg().unwrap();
        chart.layout = Some("circular".to_string());
        let circular = chart.svg().unwrap();
        assert_ne!(force, circular, "circular layout should differ from force");
        assert!(!circular.contains("NaN"));
    }

    #[test]
    fn graph_from_json() {
        let chart = GraphChart::from_json(
            r##"{
                "title_text": "Team Network",
                "nodes": [
                    {"name": "Alice", "value": 20, "category": 0},
                    {"name": "Bob", "value": 12, "category": 0},
                    {"name": "Carol", "value": 16, "category": 1},
                    {"name": "Dave", "value": 8, "category": 1},
                    {"name": "Erin", "value": 14, "category": 2},
                    {"name": "Frank", "value": 10, "category": 2},
                    {"name": "Grace", "value": 18, "category": 1}
                ],
                "links": [
                    {"source": "Alice", "target": "Bob"},
                    {"source": "Alice", "target": "Carol"},
                    {"source": "Alice", "target": "Grace"},
                    {"source": "Bob", "target": "Dave"},
                    {"source": "Carol", "target": "Dave"},
                    {"source": "Carol", "target": "Erin"},
                    {"source": "Erin", "target": "Frank"},
                    {"source": "Frank", "target": "Grace"},
                    {"source": "Grace", "target": "Carol"},
                    {"source": "Dave", "target": "Erin"}
                ]
            }"##,
        )
        .unwrap();
        assert_eq!(
            include_str!("../../asset/graph_chart/basic_json.svg"),
            chart.svg().unwrap()
        );
    }

    #[test]
    fn graph_single_node_no_panic() {
        let chart = GraphChart::new(vec![GraphNode::from("solo")], vec![]);
        let svg = chart.svg().unwrap();
        assert!(svg.starts_with("<svg"));
        assert!(!svg.contains("NaN") && !svg.contains("inf"));
    }

    #[test]
    fn graph_empty() {
        let chart = GraphChart::new(vec![], vec![]);
        assert!(chart.svg().unwrap().starts_with("<svg"));
    }
}
