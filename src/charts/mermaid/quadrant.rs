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

//! Mermaid `quadrantChart` parser and pure-SVG renderer.

use std::collections::HashMap;

use super::super::color::Color;
use super::super::component::{Circle, Line, Rect, Text};
use super::super::error::{Error, Result};
use super::super::theme::get_theme;
use super::super::util::format_float;
use super::MermaidChart;
use super::preprocess::SourceLine;
use super::style::{MermaidPalette, mix_color, parse_css_color, unquote_text};
use super::text::{draw_multiline_text, format_and_measure};

#[derive(Clone, Debug, Default)]
struct QuadrantPointStyle {
    color: Option<Color>,
    stroke_color: Option<Color>,
    stroke_width: Option<f32>,
    radius: Option<f32>,
}

impl QuadrantPointStyle {
    fn merge_from(&mut self, other: &Self) {
        if other.color.is_some() {
            self.color = other.color;
        }
        if other.stroke_color.is_some() {
            self.stroke_color = other.stroke_color;
        }
        if other.stroke_width.is_some() {
            self.stroke_width = other.stroke_width;
        }
        if other.radius.is_some() {
            self.radius = other.radius;
        }
    }
}

#[derive(Clone, Debug)]
struct QuadrantPoint {
    label: String,
    x: f32,
    y: f32,
    classes: Vec<String>,
    inline_style: QuadrantPointStyle,
}

#[derive(Debug, Default)]
struct ParsedQuadrant {
    title: Option<String>,
    x_left: Option<String>,
    x_right: Option<String>,
    y_bottom: Option<String>,
    y_top: Option<String>,
    quadrants: [Option<String>; 4],
    class_defs: HashMap<String, QuadrantPointStyle>,
    points: Vec<QuadrantPoint>,
}

fn parse_point_attrs(raw: &str, line_no: usize) -> Result<QuadrantPointStyle> {
    let mut style = QuadrantPointStyle::default();
    let mut normalized = String::with_capacity(raw.len());
    for ch in raw.chars() {
        normalized.push(if ch == ',' { ' ' } else { ch });
    }

    // Rejoin `key:` followed by whitespace so `key: value` becomes `key:value`.
    let tokens: Vec<&str> = normalized.split_whitespace().collect();
    let mut i = 0;
    while i < tokens.len() {
        let tok = tokens[i];
        let (k, v) = if let Some((left, right)) = tok.split_once(':') {
            if right.is_empty() && i + 1 < tokens.len() {
                i += 1;
                (left.trim(), tokens[i].trim())
            } else {
                (left.trim(), right.trim())
            }
        } else {
            return Err(Error::Mermaid {
                line: Some(line_no),
                message: format!("invalid quadrant style attribute `{tok}`"),
            });
        };

        let key_lower = k.to_ascii_lowercase();
        match key_lower.as_str() {
            "color" | "fill" => {
                style.color = Some(parse_css_color(v).ok_or_else(|| Error::Mermaid {
                    line: Some(line_no),
                    message: format!("invalid color `{v}` in quadrant attribute"),
                })?);
            }
            "stroke-color" | "stroke" => {
                style.stroke_color = Some(parse_css_color(v).ok_or_else(|| Error::Mermaid {
                    line: Some(line_no),
                    message: format!("invalid stroke-color `{v}` in quadrant attribute"),
                })?);
            }
            "stroke-width" => {
                let num_str = v.strip_suffix("px").unwrap_or(v);
                let w: f32 = num_str.parse().map_err(|_| Error::Mermaid {
                    line: Some(line_no),
                    message: format!("invalid stroke-width `{v}` in quadrant attribute"),
                })?;
                if !w.is_finite() || w < 0.0 {
                    return Err(Error::Mermaid {
                        line: Some(line_no),
                        message: format!("invalid stroke-width `{v}`"),
                    });
                }
                style.stroke_width = Some(w);
            }
            "radius" => {
                let num_str = v.strip_suffix("px").unwrap_or(v);
                let r: f32 = num_str.parse().map_err(|_| Error::Mermaid {
                    line: Some(line_no),
                    message: format!("invalid radius `{v}` in quadrant attribute"),
                })?;
                if !r.is_finite() || r <= 0.0 {
                    return Err(Error::Mermaid {
                        line: Some(line_no),
                        message: format!("quadrant point radius must be > 0, got `{v}`"),
                    });
                }
                style.radius = Some(r);
            }
            _ => {}
        }
        i += 1;
    }

    Ok(style)
}

fn find_point_colon(line: &str) -> Option<usize> {
    let bytes = line.as_bytes();
    let mut in_quotes = false;
    for (i, &b) in bytes.iter().enumerate() {
        if b == b'"' {
            in_quotes = !in_quotes;
        } else if !in_quotes && b == b':' {
            let after = line[i + 1..].trim_start();
            if after.starts_with('[') {
                return Some(i);
            }
        }
    }
    None
}

fn split_axis_pair(raw: &str) -> (Option<String>, Option<String>) {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return (None, None);
    }
    if let Some((left, right)) = trimmed.split_once("-->") {
        let l = unquote_text(left.trim());
        let r = unquote_text(right.trim());
        return ((!l.is_empty()).then_some(l), (!r.is_empty()).then_some(r));
    }
    (Some(unquote_text(trimmed)), None)
}

fn parse_quadrant(lines: &[SourceLine]) -> Result<ParsedQuadrant> {
    let mut q = ParsedQuadrant::default();
    let mut in_acc_descr = false;

    for sl in lines {
        let trimmed = sl.text.trim();
        if trimmed.is_empty() {
            continue;
        }
        if in_acc_descr {
            if trimmed.contains('}') {
                in_acc_descr = false;
            }
            continue;
        }
        if trimmed.starts_with("accTitle") {
            continue;
        }
        if trimmed.starts_with("accDescr") {
            if trimmed.contains('{') && !trimmed.contains('}') {
                in_acc_descr = true;
            }
            continue;
        }

        if let Some(rest) = trimmed.strip_prefix("title")
            && (rest.is_empty() || rest.starts_with(char::is_whitespace))
        {
            let t = unquote_text(rest.trim());
            if !t.is_empty() {
                q.title = Some(t);
            }
            continue;
        }
        if let Some(rest) = trimmed.strip_prefix("x-axis")
            && rest.starts_with(char::is_whitespace)
        {
            let (l, r) = split_axis_pair(rest);
            q.x_left = l;
            q.x_right = r;
            continue;
        }
        if let Some(rest) = trimmed.strip_prefix("y-axis")
            && rest.starts_with(char::is_whitespace)
        {
            let (b, t) = split_axis_pair(rest);
            q.y_bottom = b;
            q.y_top = t;
            continue;
        }

        let mut handled_quad = false;
        for (idx, quad) in q.quadrants.iter_mut().enumerate() {
            let prefix = format!("quadrant-{}", idx + 1);
            if let Some(rest) = trimmed.strip_prefix(&prefix)
                && (rest.is_empty() || rest.starts_with(char::is_whitespace))
            {
                let label = unquote_text(rest.trim());
                if !label.is_empty() {
                    *quad = Some(label);
                }
                handled_quad = true;
                break;
            }
        }
        if handled_quad {
            continue;
        }

        if let Some(rest) = trimmed.strip_prefix("classDef")
            && rest.starts_with(char::is_whitespace)
        {
            let mut parts = rest.trim().splitn(2, char::is_whitespace);
            let name = parts.next().unwrap_or("").trim();
            let attrs = parts.next().unwrap_or("").trim();
            if name.is_empty() {
                return Err(Error::Mermaid {
                    line: Some(sl.line_no),
                    message: "classDef missing class name".to_string(),
                });
            }
            let props = parse_point_attrs(attrs, sl.line_no)?;
            q.class_defs
                .entry(name.to_string())
                .or_default()
                .merge_from(&props);
            continue;
        }

        let Some(colon_idx) = find_point_colon(trimmed) else {
            return Err(Error::Mermaid {
                line: Some(sl.line_no),
                message: format!("invalid quadrant statement `{trimmed}`"),
            });
        };
        let lhs = trimmed[..colon_idx].trim();
        let rhs = trimmed[colon_idx + 1..].trim();

        let (raw_label, classes) = if let Some((name, cls_str)) = lhs.split_once(":::") {
            let cls = cls_str
                .split(',')
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .map(str::to_string)
                .collect();
            (name.trim(), cls)
        } else {
            (lhs, Vec::new())
        };
        let label = unquote_text(raw_label);
        if label.is_empty() {
            return Err(Error::Mermaid {
                line: Some(sl.line_no),
                message: "quadrant point label cannot be empty".to_string(),
            });
        }

        let after_open = rhs.strip_prefix('[').ok_or_else(|| Error::Mermaid {
            line: Some(sl.line_no),
            message: "expected `[x, y]` coordinates in quadrant point".to_string(),
        })?;
        let close_idx = after_open.find(']').ok_or_else(|| Error::Mermaid {
            line: Some(sl.line_no),
            message: "unclosed `[x, y]` coordinates in quadrant point".to_string(),
        })?;
        let coords_str = &after_open[..close_idx];
        let trailing_attrs = after_open[close_idx + 1..].trim();

        let coords: Vec<&str> = coords_str.split(',').collect();
        if coords.len() != 2 {
            return Err(Error::Mermaid {
                line: Some(sl.line_no),
                message: format!("expected `[x, y]`, got `[{coords_str}]`"),
            });
        }
        let x: f32 = coords[0].trim().parse().map_err(|_| Error::Mermaid {
            line: Some(sl.line_no),
            message: format!("invalid x coordinate `{}`", coords[0].trim()),
        })?;
        let y: f32 = coords[1].trim().parse().map_err(|_| Error::Mermaid {
            line: Some(sl.line_no),
            message: format!("invalid y coordinate `{}`", coords[1].trim()),
        })?;
        if !x.is_finite() || !(0.0..=1.0).contains(&x) {
            return Err(Error::Mermaid {
                line: Some(sl.line_no),
                message: format!("quadrant x coordinate `{x}` must be within 0.0..=1.0"),
            });
        }
        if !y.is_finite() || !(0.0..=1.0).contains(&y) {
            return Err(Error::Mermaid {
                line: Some(sl.line_no),
                message: format!("quadrant y coordinate `{y}` must be within 0.0..=1.0"),
            });
        }

        let inline_style = if trailing_attrs.is_empty() {
            QuadrantPointStyle::default()
        } else {
            parse_point_attrs(trailing_attrs, sl.line_no)?
        };

        q.points.push(QuadrantPoint {
            label,
            x,
            y,
            classes,
            inline_style,
        });
    }

    Ok(q)
}

#[derive(Clone, Copy)]
struct RectBox {
    x0: f32,
    y0: f32,
    x1: f32,
    y1: f32,
}

impl RectBox {
    fn overlap_area(&self, other: &Self) -> f32 {
        let ox = (self.x1.min(other.x1) - self.x0.max(other.x0)).max(0.0);
        let oy = (self.y1.min(other.y1) - self.y0.max(other.y0)).max(0.0);
        ox * oy
    }
}

pub(crate) fn render(
    chart: &MermaidChart,
    theme_name: &str,
    fm_title: Option<&str>,
    lines: &[SourceLine],
) -> Result<(String, f32, f32)> {
    let q = parse_quadrant(lines)?;
    let theme = get_theme(theme_name);
    let bg_override = chart
        .has_custom_background()
        .then_some(chart.base.background_color);
    let palette = MermaidPalette::from_theme(&theme, bg_override);
    let font_family = chart.resolved_font_family(&theme);

    let title_text = if !chart.base.title.text.is_empty() {
        Some(chart.base.title.text.clone())
    } else {
        fm_title.map(str::to_string).or(q.title.clone())
    };
    let header_h = super::compute_header_height(chart, title_text.as_deref(), &theme);

    let content_w = if chart.width > 0.0 {
        chart.width
    } else {
        640.0
    };
    let content_h = if chart.height > 0.0 {
        chart.height
    } else {
        500.0
    };

    let (mut canvas, out_w, out_h) = super::create_scaled_canvas(chart, content_w, content_h);
    if !palette.bg.is_transparent() {
        canvas.rect(Rect {
            fill: Some(palette.bg.into()),
            left: 0.0,
            top: 0.0,
            width: content_w,
            height: content_h,
            ..Default::default()
        });
    }
    super::draw_header(&mut canvas, chart, title_text.as_deref(), &theme, content_w);

    let ox = chart.base.margin.left;
    let oy = chart.base.margin.top + header_h;
    let mut cc = canvas.child(super::super::util::Box {
        left: ox,
        top: oy,
        right: chart.base.margin.right,
        bottom: chart.base.margin.bottom,
    });

    let avail_w = (content_w - chart.base.margin.left - chart.base.margin.right).max(200.0);
    let avail_h =
        (content_h - header_h - chart.base.margin.top - chart.base.margin.bottom).max(180.0);

    let has_y_axis = q.y_bottom.is_some() || q.y_top.is_some();
    let has_x_axis = q.x_left.is_some() || q.x_right.is_some();

    let plot_left = if has_y_axis { 52.0 } else { 24.0 };
    let plot_right = avail_w - 24.0;
    let plot_top = 14.0;
    let plot_bottom = avail_h - if has_x_axis { 42.0 } else { 22.0 };
    let plot_w = (plot_right - plot_left).max(120.0);
    let plot_h = (plot_bottom - plot_top).max(120.0);
    let mid_x = plot_left + plot_w * 0.5;
    let mid_y = plot_top + plot_h * 0.5;

    let series_palette = if theme.series.colors.is_empty() {
        vec![
            Color::from("#10b981"),
            Color::from("#3b82f6"),
            Color::from("#f59e0b"),
            Color::from("#ef4444"),
        ]
    } else {
        theme.series.colors.clone()
    };

    let tint = if palette.is_light { 0.11 } else { 0.18 };
    // Standard Mermaid quadrants: 1=top-right, 2=top-left, 3=bottom-left, 4=bottom-right.
    let quad_rects: [(usize, f32, f32, f32, f32); 4] = [
        (0, mid_x, plot_top, plot_w * 0.5, plot_h * 0.5),
        (1, plot_left, plot_top, plot_w * 0.5, plot_h * 0.5),
        (2, plot_left, mid_y, plot_w * 0.5, plot_h * 0.5),
        (3, mid_x, mid_y, plot_w * 0.5, plot_h * 0.5),
    ];

    let mut occupied: Vec<RectBox> = Vec::with_capacity(q.points.len() * 2 + 4);
    let quad_label_color = mix_color(palette.text_color, palette.bg, 0.25);

    for (q_idx, qx, qy, qw, qh) in quad_rects {
        let base_tint = series_palette[q_idx % series_palette.len()];
        let q_fill = mix_color(palette.opaque_bg, base_tint, tint);
        cc.rect(Rect {
            left: qx,
            top: qy,
            width: qw,
            height: qh,
            fill: Some(q_fill.into()),
            color: None,
            ..Default::default()
        });

        if let Some(ref label) = q.quadrants[q_idx] {
            let ft = format_and_measure(label, &font_family, 13.0);
            let lx = qx + qw * 0.5;
            let ly = qy + 18.0;
            draw_multiline_text(
                &mut cc,
                lx,
                ly,
                &ft,
                &font_family,
                13.0,
                quad_label_color,
                "middle",
                true,
            );
            occupied.push(RectBox {
                x0: lx - ft.width * 0.5 - 4.0,
                y0: ly - ft.height * 0.5 - 4.0,
                x1: lx + ft.width * 0.5 + 4.0,
                y1: ly + ft.height * 0.5 + 4.0,
            });
        }
    }

    let axis_stroke = mix_color(palette.node_stroke, palette.bg, 0.25);
    cc.line(Line {
        left: mid_x,
        top: plot_top,
        right: mid_x,
        bottom: plot_bottom,
        color: Some(axis_stroke),
        stroke_width: 1.0,
        stroke_dash_array: Some("4,4".to_string()),
    });
    cc.line(Line {
        left: plot_left,
        top: mid_y,
        right: plot_right,
        bottom: mid_y,
        color: Some(axis_stroke),
        stroke_width: 1.0,
        stroke_dash_array: Some("4,4".to_string()),
    });

    // Outer frame must explicitly specify transparent fill to avoid SVG default black fill.
    cc.rect(Rect {
        left: plot_left,
        top: plot_top,
        width: plot_w,
        height: plot_h,
        fill: Some(Color::transparent().into()),
        color: Some(palette.node_stroke),
        stroke_width: Some(1.0),
        ..Default::default()
    });

    // X-axis labels.
    let axis_label_color = palette.text_color;
    let x_label_y = plot_bottom + 22.0;
    match (&q.x_left, &q.x_right) {
        (Some(l), Some(r)) => {
            let ft_l = format_and_measure(l, &font_family, 13.0);
            draw_multiline_text(
                &mut cc,
                plot_left + plot_w * 0.25,
                x_label_y,
                &ft_l,
                &font_family,
                13.0,
                axis_label_color,
                "middle",
                true,
            );
            let ft_r = format_and_measure(r, &font_family, 13.0);
            draw_multiline_text(
                &mut cc,
                plot_left + plot_w * 0.75,
                x_label_y,
                &ft_r,
                &font_family,
                13.0,
                axis_label_color,
                "middle",
                true,
            );
        }
        (Some(l), None) => {
            let ft_l = format_and_measure(l, &font_family, 13.0);
            draw_multiline_text(
                &mut cc,
                mid_x,
                x_label_y,
                &ft_l,
                &font_family,
                13.0,
                axis_label_color,
                "middle",
                true,
            );
        }
        (None, Some(r)) => {
            let ft_r = format_and_measure(r, &font_family, 13.0);
            draw_multiline_text(
                &mut cc,
                mid_x,
                x_label_y,
                &ft_r,
                &font_family,
                13.0,
                axis_label_color,
                "middle",
                true,
            );
        }
        (None, None) => {}
    }

    // Y-axis labels rotated -90 around the canvas-offset pivot.
    let y_label_x = plot_left - 24.0;
    let draw_rotated_y = |cc: &mut super::super::Canvas, label: &str, local_y: f32| {
        let abs_x = ox + y_label_x;
        let abs_y = oy + local_y;
        cc.text_unmeasured(Text {
            text: label.to_string(),
            font_family: Some(font_family.clone()),
            font_size: Some(13.0),
            font_color: Some(axis_label_color),
            font_weight: Some("bold".to_string()),
            x: Some(y_label_x),
            y: Some(local_y),
            text_anchor: Some("middle".to_string()),
            dominant_baseline: Some("central".to_string()),
            transform: Some(format!(
                "rotate(-90 {} {})",
                format_float(abs_x),
                format_float(abs_y)
            )),
            ..Default::default()
        });
    };
    match (&q.y_bottom, &q.y_top) {
        (Some(b), Some(t)) => {
            draw_rotated_y(&mut cc, b, mid_y + plot_h * 0.25);
            draw_rotated_y(&mut cc, t, plot_top + plot_h * 0.25);
        }
        (Some(b), None) => draw_rotated_y(&mut cc, b, mid_y),
        (None, Some(t)) => draw_rotated_y(&mut cc, t, mid_y),
        (None, None) => {}
    }

    // Resolve styles and record point marker bounding boxes first.
    struct ResolvedPoint {
        px: f32,
        py: f32,
        r: f32,
        fill: Color,
        stroke: Color,
        stroke_w: f32,
        label: String,
    }
    let mut resolved = Vec::with_capacity(q.points.len());
    for pt in &q.points {
        let mut eff = QuadrantPointStyle::default();
        for cls in &pt.classes {
            if let Some(def) = q.class_defs.get(cls) {
                eff.merge_from(def);
            }
        }
        eff.merge_from(&pt.inline_style);

        let px = plot_left + pt.x * plot_w;
        let py = plot_bottom - pt.y * plot_h;
        let r = eff.radius.unwrap_or(6.5);
        let fill = eff.color.unwrap_or(palette.accent);
        let stroke = eff.stroke_color.unwrap_or(palette.opaque_bg);
        let stroke_w = eff.stroke_width.unwrap_or(1.8);

        occupied.push(RectBox {
            x0: px - r - 2.0,
            y0: py - r - 2.0,
            x1: px + r + 2.0,
            y1: py + r + 2.0,
        });
        resolved.push(ResolvedPoint {
            px,
            py,
            r,
            fill,
            stroke,
            stroke_w,
            label: pt.label.clone(),
        });
    }

    for rp in &resolved {
        cc.circle(Circle {
            cx: rp.px,
            cy: rp.py,
            r: rp.r,
            fill: Some(rp.fill),
            stroke_color: Some(rp.stroke),
            stroke_width: rp.stroke_w,
            ..Default::default()
        });

        let ft = format_and_measure(&rp.label, &font_family, 12.0);
        let pad_gap = 6.0;
        let candidates = [
            // Above
            (
                rp.px,
                rp.py - rp.r - pad_gap - ft.height * 0.5,
                "middle",
                rp.px - ft.width * 0.5,
            ),
            // Below
            (
                rp.px,
                rp.py + rp.r + pad_gap + ft.height * 0.5,
                "middle",
                rp.px - ft.width * 0.5,
            ),
            // Right
            (
                rp.px + rp.r + pad_gap,
                rp.py,
                "start",
                rp.px + rp.r + pad_gap,
            ),
            // Left
            (
                rp.px - rp.r - pad_gap,
                rp.py,
                "end",
                rp.px - rp.r - pad_gap - ft.width,
            ),
        ];

        let mut best_idx = 0;
        let mut best_score = f32::INFINITY;
        for (i, &(_lx, ly, _anchor, box_x0)) in candidates.iter().enumerate() {
            let box_y0 = ly - ft.height * 0.5;
            let cand_box = RectBox {
                x0: box_x0,
                y0: box_y0,
                x1: box_x0 + ft.width,
                y1: box_y0 + ft.height,
            };
            let mut score = 0.0_f32;
            for occ in &occupied {
                score += cand_box.overlap_area(occ) * 10.0;
            }
            if cand_box.x0 < plot_left + 4.0 {
                score += (plot_left + 4.0 - cand_box.x0) * 50.0;
            }
            if cand_box.x1 > plot_right - 4.0 {
                score += (cand_box.x1 - (plot_right - 4.0)) * 50.0;
            }
            if cand_box.y0 < plot_top + 4.0 {
                score += (plot_top + 4.0 - cand_box.y0) * 50.0;
            }
            if cand_box.y1 > plot_bottom - 4.0 {
                score += (cand_box.y1 - (plot_bottom - 4.0)) * 50.0;
            }
            if score < best_score {
                best_score = score;
                best_idx = i;
                if score == 0.0 {
                    break;
                }
            }
        }

        let (lx, ly, anchor, box_x0) = candidates[best_idx];
        let box_y0 = ly - ft.height * 0.5;
        occupied.push(RectBox {
            x0: box_x0 - 2.0,
            y0: box_y0 - 2.0,
            x1: box_x0 + ft.width + 2.0,
            y1: box_y0 + ft.height + 2.0,
        });
        draw_multiline_text(
            &mut cc,
            lx,
            ly,
            &ft,
            &font_family,
            12.0,
            palette.text_color,
            anchor,
            false,
        );
    }

    let svg = canvas.svg()?;
    Ok((svg, out_w, out_h))
}

#[cfg(test)]
mod tests {
    use super::super::preprocess::preprocess;
    use super::*;

    #[test]
    fn renders_quadrant_chart_with_axes_and_styled_points() {
        let src = r#"
quadrantChart
    title Campaign Priority Matrix
    x-axis Low Reach --> High Reach
    y-axis Low Engagement --> High Engagement
    quadrant-1 Expand Now
    quadrant-2 Promote More
    quadrant-3 Re-evaluate
    quadrant-4 Maintain
    classDef hero color:#e11d48,radius:9,stroke-color:#ffffff,stroke-width:2px
    Campaign Alpha:::hero: [0.78, 0.84]
    Campaign Beta: [0.32, 0.72] radius: 7, color: #2563eb
"#;
        let prep = preprocess(src).unwrap();
        let chart = MermaidChart::new(src);
        let (svg, w, h) = render(&chart, "light", prep.title.as_deref(), &prep.lines).unwrap();
        assert!(svg.starts_with("<svg") && w > 400.0 && h > 300.0);
        assert!(svg.contains("Campaign Priority Matrix"));
        assert!(svg.contains("Expand Now"));
        assert!(svg.contains("Campaign Alpha"));
        assert!(svg.contains("rotate(-90"));
    }

    #[test]
    fn rejects_out_of_range_quadrant_coordinates_with_line_number() {
        let src = "quadrantChart\n  BadPoint: [1.4, 0.5]";
        let prep = preprocess(src).unwrap();
        let err = parse_quadrant(&prep.lines).unwrap_err().to_string();
        assert!(err.contains("line 2"), "{err}");
    }
}
