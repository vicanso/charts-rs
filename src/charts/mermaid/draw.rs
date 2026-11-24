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

//! Pure-SVG drawing helpers for Mermaid flowchart shapes, smooth cubic Bézier
//! edges, arrowheads/markers, and pill-backed edge labels.

use super::super::canvas::Canvas;
use super::super::color::Color;
use super::super::component::{Circle, Line, Polygon, RawPath, Rect};
use super::super::util::{format_float, write_float, Point};
use super::layout::EdgeRouting;
use super::style::MermaidPalette;
use super::text::{draw_multiline_text, format_and_measure, FormattedText};

/// Mermaid flowchart node shapes (classic + v11 `@{ shape: ... }` core subset).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub(crate) enum NodeShape {
    #[default]
    Rect,
    Rounded,
    Stadium,
    Subroutine,
    Cylinder,
    Circle,
    DoubleCircle,
    Diamond,
    Hexagon,
    Parallelogram,
    ParallelogramAlt,
    Trapezoid,
    TrapezoidAlt,
    Asymmetric,
    Doc,
    Docs,
    Delay,
    Triangle,
    Flag,
    SmallCircle,
    FramedCircle,
    TextOnly,
}

impl NodeShape {
    /// Whether edges may attach anywhere along the sides facing the layout flow
    /// (otherwise they converge on the side's center point). In horizontal
    /// layouts, a cylinder's left/right walls are flat vertical segments.
    pub(crate) fn spreads_ports_in(self, is_horizontal: bool) -> bool {
        match self {
            NodeShape::Rect | NodeShape::Rounded | NodeShape::Subroutine | NodeShape::TextOnly => {
                true
            }
            NodeShape::Cylinder => is_horizontal,
            _ => false,
        }
    }
}

/// Edge line stroke kind.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum EdgeStroke {
    Solid,
    Dotted,
    Thick,
    Invisible,
}

/// Edge endpoint marker kind.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum EdgeMarker {
    None,
    Arrow,
    OpenArrow,
    Circle,
    Cross,
    HollowTriangle,
    FilledDiamond,
    HollowDiamond,
    Lollipop,
    ErOnlyOne,
    ErZeroOrOne,
    ErOneOrMore,
    ErZeroOrMore,
}

fn polygon_vertices(shape: NodeShape, cx: f32, cy: f32, w: f32, h: f32) -> Option<Vec<Point>> {
    let hw = w * 0.5;
    let hh = h * 0.5;
    let skew = (h * 0.35).min(w * 0.22);
    let hex_indent = (h * 0.42).min(w * 0.25);
    let flag = (h * 0.38).min(w * 0.20);

    let pts = match shape {
        NodeShape::Diamond => vec![(cx, cy - hh), (cx + hw, cy), (cx, cy + hh), (cx - hw, cy)],
        NodeShape::Hexagon => vec![
            (cx - hw, cy),
            (cx - hw + hex_indent, cy - hh),
            (cx + hw - hex_indent, cy - hh),
            (cx + hw, cy),
            (cx + hw - hex_indent, cy + hh),
            (cx - hw + hex_indent, cy + hh),
        ],
        NodeShape::Parallelogram => vec![
            (cx - hw + skew, cy - hh),
            (cx + hw, cy - hh),
            (cx + hw - skew, cy + hh),
            (cx - hw, cy + hh),
        ],
        NodeShape::ParallelogramAlt => vec![
            (cx - hw, cy - hh),
            (cx + hw - skew, cy - hh),
            (cx + hw, cy + hh),
            (cx - hw + skew, cy + hh),
        ],
        NodeShape::Trapezoid => vec![
            (cx - hw + skew, cy - hh),
            (cx + hw - skew, cy - hh),
            (cx + hw, cy + hh),
            (cx - hw, cy + hh),
        ],
        NodeShape::TrapezoidAlt => vec![
            (cx - hw, cy - hh),
            (cx + hw, cy - hh),
            (cx + hw - skew, cy + hh),
            (cx - hw + skew, cy + hh),
        ],
        NodeShape::Asymmetric => vec![
            (cx - hw, cy - hh),
            (cx + hw, cy - hh),
            (cx + hw, cy + hh),
            (cx - hw, cy + hh),
            (cx - hw + flag, cy),
        ],
        NodeShape::Triangle => vec![(cx, cy - hh), (cx + hw, cy + hh), (cx - hw, cy + hh)],
        NodeShape::Flag => vec![
            (cx - hw, cy - hh),
            (cx + hw - flag, cy - hh),
            (cx + hw, cy),
            (cx + hw - flag, cy + hh),
            (cx - hw, cy + hh),
            (cx - hw + flag * 0.8, cy),
        ],
        _ => return None,
    };

    Some(pts.into_iter().map(Point::from).collect())
}

/// Computes the exact intersection on a rectangle centered at `(cx, cy)`.
pub(crate) fn clip_point_to_rect(cx: f32, cy: f32, w: f32, h: f32, toward: Point) -> Point {
    let dx = toward.x - cx;
    let dy = toward.y - cy;
    let hw = (w * 0.5).max(1.0);
    let hh = (h * 0.5).max(1.0);
    if dx.abs() < 1e-4 && dy.abs() < 1e-4 {
        return Point::from((cx, cy));
    }
    let sx = if dx.abs() > 1e-4 {
        hw / dx.abs()
    } else {
        f32::MAX
    };
    let sy = if dy.abs() > 1e-4 {
        hh / dy.abs()
    } else {
        f32::MAX
    };
    let t = sx.min(sy).min(1.0);
    Point::from((cx + dx * t, cy + dy * t))
}

/// Computes the boundary intersection along the ray from a node's center toward `toward`.
pub(crate) fn clip_point_to_shape(
    shape: NodeShape,
    cx: f32,
    cy: f32,
    w: f32,
    h: f32,
    toward: Point,
) -> Point {
    let dx = toward.x - cx;
    let dy = toward.y - cy;
    let dist = dx.hypot(dy);
    if dist < 1e-4 {
        return Point::from((cx, cy));
    }

    match shape {
        NodeShape::Circle
        | NodeShape::DoubleCircle
        | NodeShape::SmallCircle
        | NodeShape::FramedCircle => {
            let r = (w.max(h) * 0.5).max(1.0);
            let scale = (r / dist).min(1.0);
            Point::from((cx + dx * scale, cy + dy * scale))
        }
        _ if polygon_vertices(shape, cx, cy, w, h).is_some() => {
            let verts = polygon_vertices(shape, cx, cy, w, h).unwrap();
            let mut best_t = f32::MAX;
            for i in 0..verts.len() {
                let a = verts[i];
                let b = verts[(i + 1) % verts.len()];
                let ex = b.x - a.x;
                let ey = b.y - a.y;
                let denom = dx * ey - dy * ex;
                if denom.abs() < 1e-6 {
                    continue;
                }
                let qx = a.x - cx;
                let qy = a.y - cy;
                let t = (qx * ey - qy * ex) / denom;
                let u = (qx * dy - qy * dx) / denom;
                if t > 0.0 && (-1e-4..=1.0001).contains(&u) {
                    best_t = best_t.min(t);
                }
            }
            if best_t.is_finite() {
                let t = best_t.min(1.0);
                Point::from((cx + dx * t, cy + dy * t))
            } else {
                clip_point_to_rect(cx, cy, w, h, toward)
            }
        }
        _ => clip_point_to_rect(cx, cy, w, h, toward),
    }
}

/// Draws a Mermaid flowchart node in pure SVG.
#[allow(clippy::too_many_arguments)]
pub(crate) fn draw_node(
    c: &mut Canvas,
    shape: NodeShape,
    cx: f32,
    cy: f32,
    w: f32,
    h: f32,
    fill: Color,
    stroke: Color,
    stroke_width: f32,
    stroke_dasharray: Option<String>,
    text_color: Color,
    label: &FormattedText,
    font_family: &str,
    font_size: f32,
) {
    let x = cx - w * 0.5;
    let y = cy - h * 0.5;

    match shape {
        NodeShape::Rect | NodeShape::Rounded | NodeShape::Stadium => {
            let rx = match shape {
                NodeShape::Rect => Some(super::style::NODE_RADIUS * 0.5),
                NodeShape::Rounded => Some(super::style::NODE_RADIUS * 1.5),
                NodeShape::Stadium => Some((h * 0.5).min(w * 0.5)),
                _ => None,
            };
            c.rect(Rect {
                left: x,
                top: y,
                width: w,
                height: h,
                rx,
                ry: rx,
                fill: Some(fill.into()),
                color: Some(stroke),
                stroke_width: Some(stroke_width),
                stroke_dash_array: stroke_dasharray,
                ..Default::default()
            });
        }
        NodeShape::Subroutine => {
            let inset = 8.0_f32.min(w * 0.15);
            c.rect(Rect {
                left: x,
                top: y,
                width: w,
                height: h,
                fill: Some(fill.into()),
                color: Some(stroke),
                stroke_width: Some(stroke_width),
                stroke_dash_array: stroke_dasharray.clone(),
                ..Default::default()
            });
            for lx in [x + inset, x + w - inset] {
                c.line(Line {
                    color: Some(stroke),
                    stroke_width,
                    left: lx,
                    top: y,
                    right: lx,
                    bottom: y + h,
                    stroke_dash_array: stroke_dasharray.clone(),
                });
            }
        }
        NodeShape::Cylinder => {
            let ry = (h * 0.14).clamp(5.0, 10.0);
            let rx = w * 0.5;
            let top_y = y + ry;
            let bot_y = y + h - ry;
            let (xs, rx_s, ry_s, xw_s, top_s, bot_s) = (
                format_float(x),
                format_float(rx),
                format_float(ry),
                format_float(x + w),
                format_float(top_y),
                format_float(bot_y),
            );
            // Body with curved top & bottom caps.
            let body_d = format!(
                "M{xs} {top_s} A {rx_s} {ry_s} 0 0 1 {xw_s} {top_s} L {xw_s} {bot_s} A {rx_s} {ry_s} 0 0 1 {xs} {bot_s} Z"
            );
            c.raw_path(RawPath {
                d: body_d,
                fill: Some(fill),
                stroke: Some(stroke),
                stroke_width,
                stroke_dash_array: stroke_dasharray.clone(),
                ..Default::default()
            });
            // Inner lid crease arc across the top of the cylinder.
            let lid_d = format!("M {xs} {top_s} A {rx_s} {ry_s} 0 0 0 {xw_s} {top_s}");
            c.raw_path(RawPath {
                d: lid_d,
                fill: None,
                stroke: Some(stroke),
                stroke_width,
                stroke_dash_array: stroke_dasharray,
                ..Default::default()
            });
        }
        NodeShape::Circle | NodeShape::DoubleCircle => {
            let r = w.max(h) * 0.5;
            c.circle(Circle {
                cx,
                cy,
                r,
                fill: Some(fill),
                stroke_color: Some(stroke),
                stroke_width,
                stroke_dash_array: stroke_dasharray.clone(),
                ..Default::default()
            });
            if shape == NodeShape::DoubleCircle {
                c.circle(Circle {
                    cx,
                    cy,
                    r: (r - 5.0).max(2.0),
                    fill: None,
                    stroke_color: Some(stroke),
                    stroke_width,
                    stroke_dash_array: stroke_dasharray,
                    ..Default::default()
                });
            }
        }
        NodeShape::SmallCircle => {
            let r = (w.min(h) * 0.5).max(6.0);
            c.circle(Circle {
                cx,
                cy,
                r,
                fill: Some(stroke),
                stroke_color: Some(stroke),
                stroke_width,
                ..Default::default()
            });
        }
        NodeShape::FramedCircle => {
            let r = (w.min(h) * 0.5).max(8.0);
            c.circle(Circle {
                cx,
                cy,
                r,
                fill: Some(fill),
                stroke_color: Some(stroke),
                stroke_width,
                stroke_dash_array: stroke_dasharray,
                ..Default::default()
            });
            let d = r * 0.68;
            for (dx, dy) in [(-d, -d), (-d, d)] {
                c.line(Line {
                    color: Some(stroke),
                    stroke_width,
                    left: cx + dx,
                    top: cy + dy,
                    right: cx - dx,
                    bottom: cy - dy,
                    ..Default::default()
                });
            }
        }
        NodeShape::Delay => {
            let r = (h * 0.5).min(w * 0.45);
            let right_flat = x + w - r;
            let d = format!(
                "M{} {} L{} {} A{} {} 0 0 1 {} {} L{} {} Z",
                format_float(x),
                format_float(y),
                format_float(right_flat),
                format_float(y),
                format_float(r),
                format_float(h * 0.5),
                format_float(right_flat),
                format_float(y + h),
                format_float(x),
                format_float(y + h),
            );
            c.raw_path(RawPath {
                d,
                fill: Some(fill),
                stroke: Some(stroke),
                stroke_width,
                stroke_dash_array: stroke_dasharray,
                stroke_linejoin: Some("round"),
                ..Default::default()
            });
        }
        NodeShape::Doc | NodeShape::Docs => {
            let doc_path = |ox: f32, oy: f32, dw: f32, dh: f32| -> String {
                let wave = (dh * 0.16).clamp(5.0, 9.0);
                let bot = oy + dh - wave;
                format!(
                    "M{} {} L{} {} L{} {} C{} {}, {} {}, {} {} Z",
                    format_float(ox),
                    format_float(oy),
                    format_float(ox + dw),
                    format_float(oy),
                    format_float(ox + dw),
                    format_float(bot),
                    format_float(ox + dw * 0.65),
                    format_float(bot - wave * 1.4),
                    format_float(ox + dw * 0.35),
                    format_float(oy + dh + wave * 0.6),
                    format_float(ox),
                    format_float(bot),
                )
            };
            if shape == NodeShape::Docs {
                let step = 4.0;
                let fw = (w - step * 2.0).max(20.0);
                let fh = (h - step * 2.0).max(20.0);
                for layer in [2.0_f32, 1.0, 0.0] {
                    c.raw_path(RawPath {
                        d: doc_path(x + layer * step, y + (2.0 - layer) * step, fw, fh),
                        fill: Some(fill),
                        stroke: Some(stroke),
                        stroke_width,
                        stroke_dash_array: stroke_dasharray.clone(),
                        stroke_linejoin: Some("round"),
                        ..Default::default()
                    });
                }
            } else {
                c.raw_path(RawPath {
                    d: doc_path(x, y, w, h),
                    fill: Some(fill),
                    stroke: Some(stroke),
                    stroke_width,
                    stroke_dash_array: stroke_dasharray,
                    stroke_linejoin: Some("round"),
                    ..Default::default()
                });
            }
        }
        NodeShape::TextOnly => {}
        _ => {
            if let Some(points) = polygon_vertices(shape, cx, cy, w, h) {
                if stroke_dasharray.is_some() || (stroke_width - 1.0).abs() > 0.05 {
                    let mut d = String::new();
                    for (i, p) in points.iter().enumerate() {
                        d.push(if i == 0 { 'M' } else { 'L' });
                        write_float(&mut d, p.x);
                        d.push(' ');
                        write_float(&mut d, p.y);
                        d.push(' ');
                    }
                    d.push('Z');
                    c.raw_path(RawPath {
                        d,
                        fill: Some(fill),
                        stroke: Some(stroke),
                        stroke_width,
                        stroke_dash_array: stroke_dasharray,
                        stroke_linejoin: Some("round"),
                        ..Default::default()
                    });
                } else {
                    c.polygon(Polygon {
                        points,
                        fill: Some(fill),
                        color: Some(stroke),
                        ..Default::default()
                    });
                }
            }
        }
    }

    let text_y = match shape {
        NodeShape::Triangle => cy + h * 0.16,
        NodeShape::Cylinder => cy + (h * 0.14).clamp(5.0, 10.0) * 0.3,
        _ => cy,
    };
    draw_multiline_text(
        c,
        cx,
        text_y,
        label,
        font_family,
        font_size,
        text_color,
        "middle",
        false,
    );
}

/// Converts routed waypoints into an SVG path (`d`) for the given routing
/// style, returning `(path_d, midpoint, start_out_tangent, end_in_tangent)`.
pub(crate) fn build_routed_path(
    pts: &[Point],
    routing: EdgeRouting,
) -> (String, Point, (f32, f32), (f32, f32)) {
    match routing {
        EdgeRouting::Orthogonal => build_orthogonal_edge_path(pts, ORTHO_CORNER_RADIUS),
        EdgeRouting::Spline => build_smooth_edge_path(pts),
        EdgeRouting::Polyline => build_linear_edge_path(pts),
    }
}

const ORTHO_CORNER_RADIUS: f32 = 6.0;

/// Builds an axis-aligned polyline path whose corners are rounded with
/// quadratic curves of at most `radius` (shrunk on short segments). The
/// midpoint is measured by arc length.
pub(crate) fn build_orthogonal_edge_path(
    pts: &[Point],
    radius: f32,
) -> (String, Point, (f32, f32), (f32, f32)) {
    let n = pts.len();
    if n < 2 {
        return (String::new(), Point::default(), (1.0, 0.0), (1.0, 0.0));
    }
    let seg_len = |a: Point, b: Point| (b.x - a.x).hypot(b.y - a.y);
    let mut d = String::new();
    d.push('M');
    write_float(&mut d, pts[0].x);
    d.push(' ');
    write_float(&mut d, pts[0].y);
    for i in 1..n - 1 {
        let (p0, p1, p2) = (pts[i - 1], pts[i], pts[i + 1]);
        let l_in = seg_len(p0, p1);
        let l_out = seg_len(p1, p2);
        let r = radius.min(l_in * 0.5).min(l_out * 0.5);
        if r < 0.5 || l_in < 1e-3 || l_out < 1e-3 {
            d.push_str(" L");
            write_float(&mut d, p1.x);
            d.push(' ');
            write_float(&mut d, p1.y);
            continue;
        }
        let a = Point::from((
            p1.x - (p1.x - p0.x) / l_in * r,
            p1.y - (p1.y - p0.y) / l_in * r,
        ));
        let b = Point::from((
            p1.x + (p2.x - p1.x) / l_out * r,
            p1.y + (p2.y - p1.y) / l_out * r,
        ));
        d.push_str(" L");
        write_float(&mut d, a.x);
        d.push(' ');
        write_float(&mut d, a.y);
        d.push_str(" Q");
        write_float(&mut d, p1.x);
        d.push(' ');
        write_float(&mut d, p1.y);
        d.push(',');
        write_float(&mut d, b.x);
        d.push(' ');
        write_float(&mut d, b.y);
    }
    d.push_str(" L");
    write_float(&mut d, pts[n - 1].x);
    d.push(' ');
    write_float(&mut d, pts[n - 1].y);

    let total: f32 = pts.windows(2).map(|w| seg_len(w[0], w[1])).sum();
    let mut mid = pts[0];
    let mut acc = 0.0_f32;
    for w in pts.windows(2) {
        let l = seg_len(w[0], w[1]);
        if acc + l >= total * 0.5 && l > 1e-4 {
            let t = (total * 0.5 - acc) / l;
            mid = Point::from((
                w[0].x + (w[1].x - w[0].x) * t,
                w[0].y + (w[1].y - w[0].y) * t,
            ));
            break;
        }
        acc += l;
    }
    let start_dir = (pts[1].x - pts[0].x, pts[1].y - pts[0].y);
    let end_dir = (pts[n - 1].x - pts[n - 2].x, pts[n - 1].y - pts[n - 2].y);
    (d, mid, start_dir, end_dir)
}

/// Geometry an edge endpoint attaches to.
#[derive(Clone, Copy, Debug)]
pub(crate) enum EdgeEnd {
    /// A node of the given shape centered at `(cx, cy)`.
    Shape {
        shape: NodeShape,
        cx: f32,
        cy: f32,
        w: f32,
        h: f32,
    },
    /// A subgraph / composite container box.
    Box { x: f32, y: f32, w: f32, h: f32 },
}

/// Parametric entry `t` of segment `a -> b` into the rectangle, if `a` lies
/// outside and the segment reaches the rectangle.
fn segment_rect_entry(a: Point, b: Point, x: f32, y: f32, w: f32, h: f32) -> Option<f32> {
    let inside = |p: Point| p.x >= x && p.x <= x + w && p.y >= y && p.y <= y + h;
    if inside(a) {
        return None;
    }
    let (dx, dy) = (b.x - a.x, b.y - a.y);
    let mut t0 = 0.0_f32;
    let mut t1 = 1.0_f32;
    for (p, q) in [
        (-dx, a.x - x),
        (dx, x + w - a.x),
        (-dy, a.y - y),
        (dy, y + h - a.y),
    ] {
        if p.abs() < 1e-6 {
            if q < 0.0 {
                return None;
            }
        } else {
            let r = q / p;
            if p < 0.0 {
                t0 = t0.max(r);
            } else {
                t1 = t1.min(r);
            }
        }
    }
    (t0 <= t1).then_some(t0)
}

/// Truncates `pts` where the polyline first enters the box (target side).
fn truncate_at_box_entry(pts: &mut Vec<Point>, x: f32, y: f32, w: f32, h: f32) -> bool {
    for i in 0..pts.len().saturating_sub(1) {
        let (a, b) = (pts[i], pts[i + 1]);
        if let Some(t) = segment_rect_entry(a, b, x, y, w, h) {
            let q = Point::from((a.x + (b.x - a.x) * t, a.y + (b.y - a.y) * t));
            pts.truncate(i + 1);
            pts.push(q);
            return true;
        }
    }
    false
}

/// Snaps routed waypoints onto the endpoint geometry. Legacy polyline routes
/// (through node centers) clip node shapes toward the neighbor waypoint; port
/// routes keep their side ports and only clip non-flat shapes along the port
/// axis. Container box endpoints always truncate where the routed path enters
/// the box border (falling back to center-ward clipping if already inside).
pub(crate) fn attach_edge_ends(
    pts: &mut Vec<Point>,
    routing: EdgeRouting,
    from: EdgeEnd,
    to: EdgeEnd,
    is_horizontal: bool,
) {
    if pts.len() < 2 {
        return;
    }
    let legacy = routing == EdgeRouting::Polyline;
    let clip_box = |b: EdgeEnd, toward: Point| -> Point {
        match b {
            EdgeEnd::Shape {
                shape,
                cx,
                cy,
                w,
                h,
            } => clip_point_to_shape(shape, cx, cy, w, h, toward),
            EdgeEnd::Box { x, y, w, h } => {
                clip_point_to_rect(x + w * 0.5, y + h * 0.5, w, h, toward)
            }
        }
    };
    // Source end.
    match from {
        EdgeEnd::Shape { shape, .. } if legacy || !shape.spreads_ports_in(is_horizontal) => {
            pts[0] = clip_box(from, pts[1]);
        }
        EdgeEnd::Box { x, y, w, h } => {
            pts.reverse();
            if !truncate_at_box_entry(pts, x, y, w, h) {
                let n = pts.len();
                pts[n - 1] = clip_box(from, pts[n - 2]);
            }
            pts.reverse();
        }
        _ => {}
    }
    // Target end.
    let n = pts.len();
    match to {
        EdgeEnd::Shape { shape, .. } if legacy || !shape.spreads_ports_in(is_horizontal) => {
            pts[n - 1] = clip_box(to, pts[n - 2]);
        }
        EdgeEnd::Box { x, y, w, h } if !truncate_at_box_entry(pts, x, y, w, h) => {
            let n = pts.len();
            pts[n - 1] = clip_box(to, pts[n - 2]);
        }
        _ => {}
    }
}

pub(crate) fn build_linear_edge_path(pts: &[Point]) -> (String, Point, (f32, f32), (f32, f32)) {
    if pts.len() < 2 {
        return (String::new(), Point::default(), (1.0, 0.0), (1.0, 0.0));
    }
    let mut d = String::new();
    d.push('M');
    write_float(&mut d, pts[0].x);
    d.push(' ');
    write_float(&mut d, pts[0].y);
    for p in &pts[1..] {
        d.push_str(" L");
        write_float(&mut d, p.x);
        d.push(' ');
        write_float(&mut d, p.y);
    }
    let n = pts.len();
    let mid = if n % 2 == 1 {
        pts[n / 2]
    } else {
        let a = pts[n / 2 - 1];
        let b = pts[n / 2];
        Point::from(((a.x + b.x) * 0.5, (a.y + b.y) * 0.5))
    };
    let start_dir = (pts[1].x - pts[0].x, pts[1].y - pts[0].y);
    let end_dir = (pts[n - 1].x - pts[n - 2].x, pts[n - 1].y - pts[n - 2].y);
    (d, mid, start_dir, end_dir)
}

/// Converts waypoints into a smooth cubic Bézier SVG path (`d`), returning
/// `(path_d, midpoint, start_out_tangent, end_in_tangent)`.
pub(crate) fn build_smooth_edge_path(pts: &[Point]) -> (String, Point, (f32, f32), (f32, f32)) {
    if pts.len() < 2 {
        return (String::new(), Point::default(), (1.0, 0.0), (1.0, 0.0));
    }
    if pts.len() == 2 {
        let a = pts[0];
        let b = pts[1];
        let mut d = String::new();
        d.push('M');
        write_float(&mut d, a.x);
        d.push(' ');
        write_float(&mut d, a.y);
        d.push_str(" L");
        write_float(&mut d, b.x);
        d.push(' ');
        write_float(&mut d, b.y);
        let mid = Point::from(((a.x + b.x) * 0.5, (a.y + b.y) * 0.5));
        let dir = (b.x - a.x, b.y - a.y);
        return (d, mid, dir, dir);
    }

    // Catmull-Rom to cubic Bézier conversion with tension factor.
    let tension = 0.25_f32;
    let n = pts.len();
    let mut d = String::new();
    d.push('M');
    write_float(&mut d, pts[0].x);
    d.push(' ');
    write_float(&mut d, pts[0].y);

    let mut segments: Vec<(Point, Point, Point, Point)> = Vec::with_capacity(n - 1);
    for i in 0..(n - 1) {
        let p0 = if i == 0 { pts[0] } else { pts[i - 1] };
        let p1 = pts[i];
        let p2 = pts[i + 1];
        let p3 = if i + 2 < n { pts[i + 2] } else { pts[n - 1] };

        let c1 = Point::from((
            p1.x + (p2.x - p0.x) * tension,
            p1.y + (p2.y - p0.y) * tension,
        ));
        let c2 = Point::from((
            p2.x - (p3.x - p1.x) * tension,
            p2.y - (p3.y - p1.y) * tension,
        ));
        d.push_str(" C");
        write_float(&mut d, c1.x);
        d.push(' ');
        write_float(&mut d, c1.y);
        d.push(',');
        write_float(&mut d, c2.x);
        d.push(' ');
        write_float(&mut d, c2.y);
        d.push(',');
        write_float(&mut d, p2.x);
        d.push(' ');
        write_float(&mut d, p2.y);
        segments.push((p1, c1, c2, p2));
    }

    let eval_cubic = |(p0, c1, c2, p3): (Point, Point, Point, Point), t: f32| -> Point {
        let mt = 1.0 - t;
        let b0 = mt * mt * mt;
        let b1 = 3.0 * mt * mt * t;
        let b2 = 3.0 * mt * t * t;
        let b3 = t * t * t;
        Point::from((
            b0 * p0.x + b1 * c1.x + b2 * c2.x + b3 * p3.x,
            b0 * p0.y + b1 * c1.y + b2 * c2.y + b3 * p3.y,
        ))
    };

    let mid = if segments.len().is_multiple_of(2) {
        pts[n / 2]
    } else {
        eval_cubic(segments[segments.len() / 2], 0.5)
    };

    let first_seg = segments[0];
    let start_dir = {
        let dx = first_seg.1.x - first_seg.0.x;
        let dy = first_seg.1.y - first_seg.0.y;
        if dx.hypot(dy) > 1e-3 {
            (dx, dy)
        } else {
            (pts[1].x - pts[0].x, pts[1].y - pts[0].y)
        }
    };
    let last_seg = *segments.last().unwrap();
    let end_dir = {
        let dx = last_seg.3.x - last_seg.2.x;
        let dy = last_seg.3.y - last_seg.2.y;
        if dx.hypot(dy) > 1e-3 {
            (dx, dy)
        } else {
            (pts[n - 1].x - pts[n - 2].x, pts[n - 1].y - pts[n - 2].y)
        }
    };

    (d, mid, start_dir, end_dir)
}

fn path_from_points(pts: &[Point], closed: bool) -> String {
    let mut d = String::new();
    for (i, p) in pts.iter().enumerate() {
        d.push(if i == 0 { 'M' } else { 'L' });
        write_float(&mut d, p.x);
        d.push(' ');
        write_float(&mut d, p.y);
        if i + 1 < pts.len() {
            d.push(' ');
        }
    }
    if closed {
        d.push_str(" Z");
    }
    d
}

/// Draws an edge endpoint marker pointing *along* `dir` and ending at `tip`.
#[allow(clippy::too_many_arguments)]
pub(crate) fn draw_edge_marker(
    c: &mut Canvas,
    marker: EdgeMarker,
    tip: Point,
    dir: (f32, f32),
    color: Color,
    bg_fill: Color,
    stroke_width: f32,
) {
    let len = dir.0.hypot(dir.1);
    if len < 1e-4 {
        return;
    }
    let (ux, uy) = (dir.0 / len, dir.1 / len);
    let (px, py) = (-uy, ux);
    let pt_at = |back: f32, side: f32| -> Point {
        Point::from((tip.x - ux * back + px * side, tip.y - uy * back + py * side))
    };

    match marker {
        EdgeMarker::None => {}
        EdgeMarker::Arrow => {
            let arrow_len = (6.5 + stroke_width).min(11.0);
            let half_w = (3.2 + stroke_width * 0.4).min(5.5);
            let p1 = pt_at(arrow_len, half_w);
            let p2 = pt_at(arrow_len, -half_w);
            c.polygon(Polygon {
                points: vec![tip, p1, p2],
                fill: Some(color),
                color: None,
                ..Default::default()
            });
        }
        EdgeMarker::OpenArrow => {
            let pts = [pt_at(10.0, 5.5), tip, pt_at(10.0, -5.5)];
            c.raw_path(RawPath {
                d: path_from_points(&pts, false),
                fill: None,
                stroke: Some(color),
                stroke_width: stroke_width.max(1.6),
                stroke_linecap: Some("round"),
                stroke_linejoin: Some("round"),
                ..Default::default()
            });
        }
        EdgeMarker::Circle => {
            let r = 4.0;
            let center = pt_at(r, 0.0);
            c.circle(Circle {
                cx: center.x,
                cy: center.y,
                r,
                fill: Some(color),
                stroke_color: Some(color),
                stroke_width: 1.0,
                ..Default::default()
            });
        }
        EdgeMarker::Lollipop => {
            let r = 5.5;
            let center = pt_at(6.0, 0.0);
            c.circle(Circle {
                cx: center.x,
                cy: center.y,
                r,
                fill: Some(bg_fill),
                stroke_color: Some(color),
                stroke_width: 1.6,
                ..Default::default()
            });
        }
        EdgeMarker::Cross => {
            let size = 7.5;
            let center = pt_at(size + 1.0, 0.0);
            let d1x = (ux + px) * size * 0.7;
            let d1y = (uy + py) * size * 0.7;
            let d2x = (ux - px) * size * 0.7;
            let d2y = (uy - py) * size * 0.7;
            for (ax, ay) in [(d1x, d1y), (d2x, d2y)] {
                c.line(Line {
                    color: Some(color),
                    stroke_width: stroke_width.max(1.8),
                    left: center.x - ax,
                    top: center.y - ay,
                    right: center.x + ax,
                    bottom: center.y + ay,
                    ..Default::default()
                });
            }
        }
        EdgeMarker::HollowTriangle => {
            let pts = [tip, pt_at(13.0, 6.5), pt_at(13.0, -6.5)];
            c.raw_path(RawPath {
                d: path_from_points(&pts, true),
                fill: Some(bg_fill),
                stroke: Some(color),
                stroke_width: stroke_width.max(1.6),
                stroke_linejoin: Some("round"),
                ..Default::default()
            });
        }
        EdgeMarker::FilledDiamond | EdgeMarker::HollowDiamond => {
            let pts = [tip, pt_at(7.5, 5.5), pt_at(15.0, 0.0), pt_at(7.5, -5.5)];
            let fill = if marker == EdgeMarker::FilledDiamond {
                color
            } else {
                bg_fill
            };
            c.raw_path(RawPath {
                d: path_from_points(&pts, true),
                fill: Some(fill),
                stroke: Some(color),
                stroke_width: if marker == EdgeMarker::FilledDiamond {
                    1.2
                } else {
                    stroke_width.max(1.6)
                },
                stroke_linejoin: Some("round"),
                ..Default::default()
            });
        }
        EdgeMarker::ErOnlyOne => {
            for dist in [6.0_f32, 11.0] {
                let a = pt_at(dist, -6.5);
                let b = pt_at(dist, 6.5);
                c.line(Line {
                    color: Some(color),
                    stroke_width: stroke_width.max(1.6),
                    left: a.x,
                    top: a.y,
                    right: b.x,
                    bottom: b.y,
                    ..Default::default()
                });
            }
        }
        EdgeMarker::ErZeroOrOne => {
            let a = pt_at(6.0, -6.5);
            let b = pt_at(6.0, 6.5);
            c.line(Line {
                color: Some(color),
                stroke_width: stroke_width.max(1.6),
                left: a.x,
                top: a.y,
                right: b.x,
                bottom: b.y,
                ..Default::default()
            });
            let center = pt_at(14.5, 0.0);
            c.circle(Circle {
                cx: center.x,
                cy: center.y,
                r: 4.2,
                fill: Some(bg_fill),
                stroke_color: Some(color),
                stroke_width: 1.5,
                ..Default::default()
            });
        }
        EdgeMarker::ErOneOrMore | EdgeMarker::ErZeroOrMore => {
            let origin = pt_at(12.0, 0.0);
            for side in [7.5_f32, -7.5] {
                let target = pt_at(0.0, side);
                c.line(Line {
                    color: Some(color),
                    stroke_width: stroke_width.max(1.5),
                    left: origin.x,
                    top: origin.y,
                    right: target.x,
                    bottom: target.y,
                    ..Default::default()
                });
            }
            if marker == EdgeMarker::ErOneOrMore {
                let a = pt_at(12.0, -6.5);
                let b = pt_at(12.0, 6.5);
                c.line(Line {
                    color: Some(color),
                    stroke_width: stroke_width.max(1.6),
                    left: a.x,
                    top: a.y,
                    right: b.x,
                    bottom: b.y,
                    ..Default::default()
                });
            } else {
                let center = pt_at(18.0, 0.0);
                c.circle(Circle {
                    cx: center.x,
                    cy: center.y,
                    r: 4.2,
                    fill: Some(bg_fill),
                    stroke_color: Some(color),
                    stroke_width: 1.5,
                    ..Default::default()
                });
            }
        }
    }
}

/// Draws a pill-backed multiline label over an edge midpoint.
pub(crate) fn draw_edge_label(
    c: &mut Canvas,
    mid: Point,
    label: &FormattedText,
    palette: &MermaidPalette,
    text_color: Color,
    font_family: &str,
    font_size: f32,
) {
    if label.width <= 0.0 && label.height <= 0.0 {
        return;
    }
    let pad_x = 7.0;
    let pad_y = 2.5;
    let w = label.width + pad_x * 2.0;
    let h = label.height + pad_y * 2.0;
    let r = (h * 0.5).min(9.0);
    c.rect(Rect {
        left: mid.x - w * 0.5,
        top: mid.y - h * 0.5,
        width: w,
        height: h,
        rx: Some(r),
        ry: Some(r),
        fill: Some(palette.edge_label_bg.into()),
        color: Some(palette.edge_label_stroke),
        stroke_width: Some(1.0),
        ..Default::default()
    });
    draw_multiline_text(
        c,
        mid.x,
        mid.y,
        label,
        font_family,
        font_size,
        text_color,
        "middle",
        false,
    );
}

/// Draws a folded-corner note box centered at `(cx, cy)`.
#[allow(clippy::too_many_arguments)]
pub(crate) fn draw_note_box(
    c: &mut Canvas,
    cx: f32,
    cy: f32,
    w: f32,
    h: f32,
    label: &FormattedText,
    palette: &MermaidPalette,
    font_family: &str,
    font_size: f32,
) {
    let x = cx - w * 0.5;
    let y = cy - h * 0.5;
    let fold = 10.0_f32.min(w * 0.25).min(h * 0.35);
    let pts = [
        Point::from((x, y)),
        Point::from((x + w - fold, y)),
        Point::from((x + w, y + fold)),
        Point::from((x + w, y + h)),
        Point::from((x, y + h)),
    ];
    c.raw_path(RawPath {
        d: path_from_points(&pts, true),
        fill: Some(palette.note_fill),
        stroke: Some(palette.note_stroke),
        stroke_width: 1.0,
        stroke_linejoin: Some("round"),
        ..Default::default()
    });
    c.line(Line {
        color: Some(palette.note_stroke),
        stroke_width: 1.0,
        left: x + w - fold,
        top: y,
        right: x + w - fold,
        bottom: y + fold,
        ..Default::default()
    });
    c.line(Line {
        color: Some(palette.note_stroke),
        stroke_width: 1.0,
        left: x + w - fold,
        top: y + fold,
        right: x + w,
        bottom: y + fold,
        ..Default::default()
    });
    draw_multiline_text(
        c,
        cx,
        cy,
        label,
        font_family,
        font_size,
        palette.text_color,
        "middle",
        false,
    );
}

/// Draws a compact cardinality / multiplicity label near an edge endpoint.
#[allow(clippy::too_many_arguments)]
pub(crate) fn draw_cardinality_label(
    c: &mut Canvas,
    tip: Point,
    dir_into_node: (f32, f32),
    text: &str,
    palette: &MermaidPalette,
    text_color: Color,
    font_family: &str,
    font_size: f32,
) {
    if text.trim().is_empty() {
        return;
    }
    let len = dir_into_node.0.hypot(dir_into_node.1);
    if len < 1e-4 {
        return;
    }
    let (ux, uy) = (dir_into_node.0 / len, dir_into_node.1 / len);
    let (px, py) = (-uy, ux);
    let ft = format_and_measure(text, font_family, font_size);
    let pos = Point::from((tip.x - ux * 24.0 + px * 13.0, tip.y - uy * 24.0 + py * 13.0));
    let w = ft.width + 8.0;
    let h = ft.height + 4.0;
    c.rect(Rect {
        left: pos.x - w * 0.5,
        top: pos.y - h * 0.5,
        width: w,
        height: h,
        rx: Some(3.0),
        ry: Some(3.0),
        fill: Some(palette.edge_label_bg.into()),
        color: None,
        ..Default::default()
    });
    draw_multiline_text(
        c,
        pos.x,
        pos.y,
        &ft,
        font_family,
        font_size,
        text_color,
        "middle",
        false,
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::MermaidChart;

    #[test]
    fn polyline_box_target_truncates_at_entry_instead_of_corner() {
        // Center of box (0,0,200,100) is (100,50). A center-ward clip toward
        // `start = (300,150)` would hit the corner (200,100), whereas truncating
        // along `start -> orig_end = (50,80)` hits the bottom edge at (~121.43, 100).
        let start = Point::from((300.0, 150.0));
        let orig_end = Point::from((50.0, 80.0));
        let mut pts = vec![start, orig_end];
        attach_edge_ends(
            &mut pts,
            EdgeRouting::Polyline,
            EdgeEnd::Shape {
                shape: NodeShape::Stadium,
                cx: start.x,
                cy: start.y,
                w: 80.0,
                h: 36.0,
            },
            EdgeEnd::Box {
                x: 0.0,
                y: 0.0,
                w: 200.0,
                h: 100.0,
            },
            false,
        );
        let p_start = pts[0];
        let p_end = pts[pts.len() - 1];
        // Crosses the bottom edge (y = 100) strictly inside x in (0, 200).
        assert!((p_end.y - 100.0).abs() < 1e-3, "end={p_end:?}");
        assert!(p_end.x > 0.0 && p_end.x < 190.0, "end={p_end:?}");
        // Lies on the original segment `start -> orig_end`.
        let dx = orig_end.x - start.x;
        let dy = orig_end.y - start.y;
        let cross = (p_end.x - start.x) * dy - (p_end.y - start.y) * dx;
        assert!(cross.abs() < 1e-2, "cross={cross}, end={p_end:?}");
        // Source end also clipped onto start node.
        assert!(p_start.x < start.x && p_start.y < start.y);
    }

    #[test]
    fn nested_subgraphs_linear_renders() {
        let chart = MermaidChart::from_json(include_str!(
            "../../../asset/mermaid_chart/flowchart_nested_subgraphs.json"
        ))
        .unwrap();
        let svg = chart.svg().unwrap();
        assert!(svg.starts_with("<svg"));
        assert!(svg.contains("Storage Engine"));
    }
}
