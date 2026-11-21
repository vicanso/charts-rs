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
use super::super::util::{Point, format_float, write_float};
use super::style::MermaidPalette;
use super::text::{FormattedText, draw_multiline_text, format_and_measure};

/// All 14 Mermaid flowchart node shapes.
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
        NodeShape::Circle | NodeShape::DoubleCircle => {
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
                NodeShape::Rect => None,
                NodeShape::Rounded => Some(6.0),
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

    draw_multiline_text(
        c,
        cx,
        cy,
        label,
        font_family,
        font_size,
        text_color,
        "middle",
        false,
    );
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
            let arrow_len = (9.0 + stroke_width).min(13.0);
            let half_w = (4.2 + stroke_width * 0.4).min(6.5);
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
    let pad_x = 6.0;
    let pad_y = 3.0;
    let w = label.width + pad_x * 2.0;
    let h = label.height + pad_y * 2.0;
    c.rect(Rect {
        left: mid.x - w * 0.5,
        top: mid.y - h * 0.5,
        width: w,
        height: h,
        rx: Some(4.0),
        ry: Some(4.0),
        fill: Some(palette.edge_label_bg.into()),
        color: None,
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
        stroke_width: 1.2,
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
