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
use super::sunburst_chart::{SunburstData, parse_node};
use super::theme::{get_default_theme_name, get_theme};
use super::util::*;
use crate::charts::measure_text_width_family;

/// Hierarchical node data for the tree chart — identical to the sunburst
/// model (`{ name, value, children, color }`), so the two share a data layer.
pub type TreeData = SunburstData;

// ── Internal layout node ─────────────────────────────────────────────────────

struct TNode {
    label: String,
    color: Color,
    depth: usize,
    /// Total value of the node (own value, or the sum of its leaves).
    value: f32,
    /// Position along the cross axis (leaf order); parents sit at the midpoint
    /// of their children.
    cross: f32,
    parent: Option<usize>,
    is_leaf: bool,
}

/// Total value of a node: its own value when a leaf, else the sum of all
/// descendant leaf values (mirrors `SunburstData::total`).
fn node_total(node: &TreeData) -> f32 {
    if node.children.is_empty() {
        node.value.max(0.0)
    } else {
        node.children.iter().map(node_total).sum()
    }
}

/// Straight segments used to approximate each curved link.
const LINK_SEGMENTS: usize = 20;

/// Samples a cubic Bézier link from `(x0, y0)` to `(x1, y1)`. `horizontal`
/// places the end tangents along x (LR trees); otherwise along y (TB trees),
/// giving the smooth S-shaped connector used by node-link trees.
fn sample_curve(x0: f32, y0: f32, x1: f32, y1: f32, horizontal: bool) -> Vec<Point> {
    let (c1x, c1y, c2x, c2y) = if horizontal {
        let xm = (x0 + x1) / 2.0;
        (xm, y0, xm, y1)
    } else {
        let ym = (y0 + y1) / 2.0;
        (x0, ym, x1, ym)
    };
    let mut points = Vec::with_capacity(LINK_SEGMENTS + 1);
    for i in 0..=LINK_SEGMENTS {
        let t = i as f32 / LINK_SEGMENTS as f32;
        let mt = 1.0 - t;
        let b0 = mt * mt * mt;
        let b1 = 3.0 * mt * mt * t;
        let b2 = 3.0 * mt * t * t;
        let b3 = t * t * t;
        let x = b0 * x0 + b1 * c1x + b2 * c2x + b3 * x1;
        let y = b0 * y0 + b1 * c1y + b2 * c2y + b3 * y1;
        points.push((x, y).into());
    }
    points
}

/// Recursively lays out a node and its descendants, appending to `nodes` and
/// returning the new node's index. Leaves are assigned successive `cross`
/// slots; a parent's `cross` is the midpoint of its children.
#[allow(clippy::too_many_arguments)]
fn place(
    node: &TreeData,
    depth: usize,
    parent: Option<usize>,
    base: Color,
    formatter: &str,
    grand_total: f32,
    nodes: &mut Vec<TNode>,
    leaf: &mut f32,
) -> usize {
    let color = node.color.unwrap_or(base);
    let value = node_total(node);
    let label = if formatter.is_empty() {
        node.name.clone()
    } else {
        LabelOption {
            series_name: node.name.clone(),
            category_name: node.name.clone(),
            value,
            percentage: if grand_total > 0.0 {
                value / grand_total
            } else {
                0.0
            },
            formatter: formatter.to_string(),
        }
        .format()
    };
    let idx = nodes.len();
    nodes.push(TNode {
        label,
        color,
        depth,
        value: node.total(),
        cross: 0.0,
        parent,
        is_leaf: node.children.is_empty(),
    });
    if node.children.is_empty() {
        nodes[idx].cross = *leaf;
        *leaf += 1.0;
    } else {
        let mut first = f32::MAX;
        let mut last = f32::MIN;
        for child in &node.children {
            let cidx = place(
                child,
                depth + 1,
                Some(idx),
                color,
                formatter,
                grand_total,
                nodes,
                leaf,
            );
            first = first.min(nodes[cidx].cross);
            last = last.max(nodes[cidx].cross);
        }
        nodes[idx].cross = (first + last) / 2.0;
    }
    idx
}

/// The link between two nodes around a center: a curve that leaves the
/// first one along its spoke and arrives at the other one along its own.
/// The nodes are given as `(angle, radius)`.
fn radial_curve(center: (f32, f32), from: (f32, f32), to: (f32, f32)) -> Vec<Point> {
    const SEGMENTS: usize = 16;
    let at = |(angle, radius): (f32, f32)| get_pie_point(center.0, center.1, radius, angle);
    let middle = (from.1 + to.1) / 2.0;
    let (p0, p1, p2, p3) = (at(from), at((from.0, middle)), at((to.0, middle)), at(to));
    (0..=SEGMENTS)
        .map(|i| {
            let t = i as f32 / SEGMENTS as f32;
            let u = 1.0 - t;
            let (a, b, c, d) = (u * u * u, 3.0 * u * u * t, 3.0 * u * t * t, t * t * t);
            (
                a * p0.x + b * p1.x + c * p2.x + d * p3.x,
                a * p0.y + b * p1.y + c * p2.y + d * p3.y,
            )
                .into()
        })
        .collect()
}

// ── TreeChart ────────────────────────────────────────────────────────────────

/// A tree diagram of hierarchical data as linked nodes.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct TreeChart {
    /// The shared chart options (size, series, title/legend, axes); exposed
    /// directly on the chart through `Deref`, e.g. `chart.title_text`.
    pub base: ChartBase,
    y_axis_configs: Vec<YAxisConfig>,

    // tree-specific
    /// Hierarchy roots. Multiple roots form a forest laid out side by side.
    pub series_data: Vec<TreeData>,
    /// Layout orientation: where the root is, and which way the tree grows.
    /// `"LR"` (default, root on the left), `"RL"` (root on the right),
    /// `"TB"` (root on top) or `"BT"` (root at the bottom).
    pub orient: Option<String>,
    /// Radius of the node circle in pixels. Default: 6.0.
    pub symbol_size: f32,
    /// `"orthogonal"` (the default) lays the levels out side by side, by
    /// `orient`; `"radial"` puts the root in the middle and every level on
    /// a circle around it.
    pub layout: Option<String>,
    /// Shape of the links: `"curve"` (the default) or `"polyline"`, with
    /// square corners.
    pub edge_shape: Option<String>,
}

impl std::ops::Deref for TreeChart {
    type Target = ChartBase;
    fn deref(&self) -> &ChartBase {
        &self.base
    }
}
impl std::ops::DerefMut for TreeChart {
    fn deref_mut(&mut self) -> &mut ChartBase {
        &mut self.base
    }
}

impl TreeChart {
    fn fill_default(&mut self) {
        if self.symbol_size <= 0.0 {
            self.symbol_size = 6.0;
        }
    }

    /// Creates a tree chart with the default theme.
    pub fn new(series_data: Vec<TreeData>) -> TreeChart {
        TreeChart::new_with_theme(series_data, &get_default_theme_name())
    }

    /// Creates a tree chart with a custom theme.
    pub fn new_with_theme(series_data: Vec<TreeData>, theme: &str) -> TreeChart {
        let mut c = TreeChart {
            series_data,
            ..Default::default()
        };
        c.base.fill_theme(get_theme(theme), &mut c.y_axis_configs);
        c.fill_default();
        c
    }

    /// Creates a tree chart from a JSON string.
    pub fn from_json(json: &str) -> canvas::Result<TreeChart> {
        let mut c = TreeChart {
            ..Default::default()
        };
        let value = c
            .base
            .fill_option(json, &mut c.y_axis_configs, super::schema::TREE_FIELDS)?;
        if let Some(arr) = value.get("series_data").and_then(|v| v.as_array()) {
            c.series_data = arr.iter().filter_map(parse_node).collect();
        }
        if let Some(s) = get_string_from_value(&value, "orient") {
            c.orient = Some(s);
        }
        if let Some(v) = get_f32_from_value(&value, "symbol_size") {
            c.symbol_size = v;
        }
        if let Some(s) = get_string_from_value(&value, "layout") {
            c.layout = Some(s);
        }
        if let Some(s) = get_string_from_value(&value, "edge_shape") {
            c.edge_shape = Some(s);
        }
        c.fill_default();
        Ok(c)
    }

    /// Renders the chart to an SVG string.
    pub fn svg(&self) -> canvas::Result<String> {
        let mut c = self.new_canvas();

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

        // ── Layout ────────────────────────────────────────────────────────────
        let grand_total: f32 = self.series_data.iter().map(node_total).sum();
        let mut nodes: Vec<TNode> = vec![];
        let mut leaf = 0.0_f32;
        for (i, root) in self.series_data.iter().enumerate() {
            let base = get_color(&self.series_colors, i);
            place(
                root,
                0,
                None,
                base,
                &self.series_label_formatter,
                grand_total,
                &mut nodes,
                &mut leaf,
            );
        }
        if nodes.is_empty() {
            return c.svg();
        }
        let num_leaves = leaf.max(1.0);
        let max_depth = nodes.iter().map(|n| n.depth).max().unwrap_or(0);
        let depth_span = max_depth.max(1) as f32;
        let leaf_span = (num_leaves - 1.0).max(1.0);

        let r = self.symbol_size;
        let font_size = self.series_label_font_size.max(10.0);
        let gap = r + 4.0;
        // The names are taken as the JSON checks them: whatever their case.
        let is = |option: &Option<String>, name: &str| {
            option
                .as_deref()
                .is_some_and(|v| v.eq_ignore_ascii_case(name))
        };
        let lr = !is(&self.orient, "TB") && !is(&self.orient, "BT");
        // The root on the far side: the tree grows the other way.
        let flip = is(&self.orient, "RL") || is(&self.orient, "BT");
        let radial = is(&self.layout, "radial");
        let elbow = is(&self.edge_shape, "polyline");

        // Longest labels on each side, used to reserve room so labels don't clip.
        let measure = |s: &str| {
            measure_text_width_family(&self.font_family, font_size, s)
                .map(|b| b.width())
                .unwrap_or(s.len() as f32 * font_size * 0.6)
        };
        let mut root_label = 0.0_f32; // depth-0 labels (drawn on the near side)
        let mut leaf_label = 0.0_f32; // leaf labels (drawn on the far side)
        for n in &nodes {
            if n.depth == 0 {
                root_label = root_label.max(measure(&n.label));
            }
            if n.is_leaf {
                leaf_label = leaf_label.max(measure(&n.label));
            }
        }

        // Pixel position of each node along the main (depth) and cross axes.
        let positions: Vec<(f32, f32)> = if lr {
            let mut near = root_label + gap; // left room for the root label
            let mut far = leaf_label + gap; // right room for leaf labels
            // Clamp so the plotting band keeps a positive width.
            if near + far > cw - 2.0 * r {
                near = (cw - 2.0 * r) * 0.25;
                far = (cw - 2.0 * r) * 0.25;
            }
            let main = (cw - near - far).max(1.0);
            let cross = (ch - 2.0 * (r + 2.0)).max(1.0);
            nodes
                .iter()
                .map(|n| {
                    let x = near + n.depth as f32 / depth_span * main;
                    let y = (r + 2.0) + n.cross / leaf_span * cross;
                    (x, y)
                })
                .collect()
        } else {
            let near = font_size + gap; // top room for the root label
            let far = font_size + gap; // bottom room for leaf labels
            let main = (ch - near - far).max(1.0);
            let cross = (cw - 2.0 * (r + 2.0)).max(1.0);
            nodes
                .iter()
                .map(|n| {
                    let y = near + n.depth as f32 / depth_span * main;
                    let x = (r + 2.0) + n.cross / leaf_span * cross;
                    (x, y)
                })
                .collect()
        };

        let positions: Vec<(f32, f32)> = if flip && !radial {
            positions
                .into_iter()
                .map(|(x, y)| if lr { (cw - x, y) } else { (x, ch - y) })
                .collect()
        } else {
            positions
        };
        // Around the root: the angle of every node (in degrees, clockwise
        // from 12 o'clock) and how far out its level is.
        let center = (cw / 2.0, ch / 2.0);
        let roots = nodes.iter().filter(|n| n.depth == 0).count();
        let polar: Vec<(f32, f32)> = if radial {
            let outer = (cw.min(ch) / 2.0 - leaf_label - gap - r).max(10.0);
            // Several roots share a ring of their own around the middle.
            let (inner, levels) = if roots > 1 {
                (1.0, depth_span + 1.0)
            } else {
                (0.0, depth_span)
            };
            nodes
                .iter()
                .map(|n| {
                    (
                        n.cross / num_leaves * 360.0,
                        (n.depth as f32 + inner) / levels * outer,
                    )
                })
                .collect()
        } else {
            vec![]
        };
        let positions: Vec<(f32, f32)> = if radial {
            polar
                .iter()
                .map(|(angle, radius)| {
                    let p = get_pie_point(center.0, center.1, *radius, *angle);
                    (p.x, p.y)
                })
                .collect()
        } else {
            positions
        };
        // The side the tree grows to.
        let side = if flip { -1.0 } else { 1.0 };

        // ── Links (parent → child), drawn under the nodes ─────────────────────
        for (i, n) in nodes.iter().enumerate() {
            let Some(p) = n.parent else { continue };
            let (px, py) = positions[p];
            let (cx, cy) = positions[i];
            let points = if radial {
                if elbow {
                    vec![(px, py).into(), (cx, cy).into()]
                } else {
                    radial_curve(center, polar[p], polar[i])
                }
            } else {
                let (x1, y1, x2, y2) = if lr {
                    (px + r * side, py, cx - r * side, cy)
                } else {
                    (px, py + r * side, cx, cy - r * side)
                };
                if !elbow {
                    sample_curve(x1, y1, x2, y2, lr)
                } else if lr {
                    // Half way across, then over, then on.
                    let middle = (x1 + x2) / 2.0;
                    vec![
                        (x1, y1).into(),
                        (middle, y1).into(),
                        (middle, y2).into(),
                        (x2, y2).into(),
                    ]
                } else {
                    let middle = (y1 + y2) / 2.0;
                    vec![
                        (x1, y1).into(),
                        (x1, middle).into(),
                        (x2, middle).into(),
                        (x2, y2).into(),
                    ]
                }
            };
            content.polyline(Polyline {
                color: Some(self.grid_stroke_color),
                stroke_width: self.series_stroke_width.max(1.0),
                points,
            });
        }

        // ── Nodes ─────────────────────────────────────────────────────────────
        for (i, n) in nodes.iter().enumerate() {
            let (x, y) = positions[i];
            let tooltip_text = self.tooltip_show.then(|| {
                if n.value > 0.0 {
                    format!("{}: {}", n.label, format_float(n.value))
                } else {
                    n.label.clone()
                }
            });
            content.circle(Circle {
                fill: Some(n.color),
                stroke_color: Some(self.background_color),
                stroke_width: 1.0,
                cx: x,
                cy: y,
                r,
                title: tooltip_text.clone(),
                class: tooltip_text.as_ref().map(|_| "ct-trigger".to_string()),
                dataset: vec![
                    ("name".to_string(), n.label.clone()),
                    ("value".to_string(), format_float(n.value)),
                    ("depth".to_string(), n.depth.to_string()),
                ],
            });
            if let Some(text) = tooltip_text {
                content.text(Text {
                    text,
                    class: Some("ct-tip".to_string()),
                    font_family: Some(self.font_family.clone()),
                    font_color: Some(self.series_label_font_color),
                    font_size: Some(font_size),
                    x: Some(x),
                    y: Some(y - r),
                    dy: Some(-4.0),
                    text_anchor: Some("middle".to_string()),
                    ..Default::default()
                });
            }
        }

        // ── Labels ────────────────────────────────────────────────────────────
        for (i, n) in nodes.iter().enumerate() {
            if n.label.is_empty() {
                continue;
            }
            let (x, y) = positions[i];
            if radial {
                let (angle, radius) = polar[i];
                // The node in the very middle: its name above it.
                if radius <= 0.0 {
                    content.text(Text {
                        text: n.label.clone(),
                        font_family: Some(self.font_family.clone()),
                        font_color: Some(self.series_label_font_color),
                        font_size: Some(font_size),
                        font_weight: self.series_label_font_weight.clone(),
                        x: Some(x),
                        y: Some(y - gap - font_size * 0.5),
                        text_anchor: Some("middle".to_string()),
                        dominant_baseline: Some("central".to_string()),
                        ..Default::default()
                    });
                    continue;
                }
                // Along the spoke of the node, upright: on its outside, or
                // (a node with children) towards the middle where there is
                // room for its name before the ring of its parent.
                let parent_radius = n.parent.map(|p| polar[p].1).unwrap_or(0.0);
                let inwards =
                    !n.is_leaf && radius - parent_radius >= measure(&n.label) + 2.0 * gap + r;
                let out = if inwards { -gap } else { gap };
                let at = get_pie_point(center.0, center.1, radius + out, angle);
                let right = angle.rem_euclid(360.0) < 180.0;
                let (turn, anchor) = match (right, !inwards) {
                    (true, true) => (angle - 90.0, "start"),
                    (true, false) => (angle - 90.0, "end"),
                    (false, true) => (angle + 90.0, "end"),
                    (false, false) => (angle + 90.0, "start"),
                };
                content.text(Text {
                    text: n.label.clone(),
                    font_family: Some(self.font_family.clone()),
                    font_color: Some(self.series_label_font_color),
                    font_size: Some(font_size),
                    font_weight: self.series_label_font_weight.clone(),
                    x: Some(at.x),
                    y: Some(at.y),
                    transform: Some(format!(
                        "rotate({} {} {})",
                        format_float(turn),
                        format_float(at.x + content.margin.left),
                        format_float(at.y + content.margin.top)
                    )),
                    text_anchor: Some(anchor.to_string()),
                    dominant_baseline: Some("central".to_string()),
                    ..Default::default()
                });
                continue;
            }
            let (tx, ty, anchor) = if lr {
                // A leaf is named on the far side of it, the others on the
                // side of the root.
                if n.is_leaf != flip {
                    (x + gap, y, "start")
                } else {
                    (x - gap, y, "end")
                }
            } else if n.is_leaf {
                (x, y + (gap + font_size * 0.5) * side, "middle")
            } else {
                (x, y - gap * side, "middle")
            };
            content.text(Text {
                text: n.label.clone(),
                font_family: Some(self.font_family.clone()),
                font_color: Some(self.series_label_font_color),
                font_size: Some(font_size),
                font_weight: self.series_label_font_weight.clone(),
                x: Some(tx),
                y: Some(ty),
                text_anchor: Some(anchor.to_string()),
                dominant_baseline: Some("central".to_string()),
                ..Default::default()
            });
        }

        if self.tooltip_show {
            c.svg_with_style(TOOLTIP_STYLE)
        } else {
            c.svg()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{TreeChart, TreeData};

    fn leaf(name: &str, value: f32) -> TreeData {
        TreeData {
            name: name.to_string(),
            value,
            ..Default::default()
        }
    }

    fn make_tree() -> TreeChart {
        TreeChart::new(vec![TreeData {
            name: "root".to_string(),
            children: vec![
                TreeData {
                    name: "A".to_string(),
                    children: vec![leaf("A1", 1.0), leaf("A2", 1.0)],
                    ..Default::default()
                },
                leaf("B", 2.0),
            ],
            ..Default::default()
        }])
    }

    #[test]
    fn tree_chart_basic() {
        assert_snapshot!("tree_chart/basic.svg", make_tree().svg().unwrap());
    }

    #[test]
    fn tree_chart_basic_json() {
        let chart = TreeChart::from_json(
            r##"{
                "title_text": "Tree",
                "orient": "TB",
                "series_data": [
                    {
                        "name": "root",
                        "children": [
                            {"name": "A", "children": [
                                {"name": "A1", "value": 1},
                                {"name": "A2", "value": 1}
                            ]},
                            {"name": "B", "value": 2}
                        ]
                    }
                ]
            }"##,
        )
        .unwrap();
        assert_snapshot!("tree_chart/basic_json.svg", chart.svg().unwrap());
    }

    #[test]
    fn tree_chart_label_formatter() {
        let mut chart = make_tree();
        chart.series_label_formatter = "{b}: {c}".to_string();
        let svg = chart.svg().unwrap();
        // Root total = 1 + 1 + 2 = 4; leaf A1 = 1.
        assert!(svg.contains("root: 4"), "missing formatted root label");
        assert!(svg.contains("A1: 1"), "missing formatted leaf label");
    }

    #[test]
    fn tree_chart_empty() {
        let chart = TreeChart::new(vec![]);
        assert!(chart.svg().unwrap().starts_with("<svg"));
    }
}
