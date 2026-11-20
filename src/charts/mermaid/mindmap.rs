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

//! Mermaid `mindmap` parser and pure-SVG bilateral tree renderer.

use std::collections::HashMap;
use std::fmt::Write as _;

use super::super::color::Color;
use super::super::component::{Circle, RawPath, Rect};
use super::super::error::{Error, Result};
use super::super::theme::get_theme;
use super::super::util::{format_float, write_float};
use super::MermaidChart;
use super::preprocess::SourceLine;
use super::style::{
    MermaidPalette, StyleDirective, StyleProps, mix_color, resolve_style,
    try_parse_style_directive, unquote_text,
};
use super::text::{FormattedText, draw_multiline_text, format_and_measure};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum MindShape {
    Default,
    Square,
    Rounded,
    Circle,
    Hexagon,
    Cloud,
    Bang,
}

#[derive(Clone, Debug)]
struct MindNode {
    id: Option<String>,
    label: String,
    shape: MindShape,
    parent: Option<usize>,
    children: Vec<usize>,
    depth: usize,
    classes: Vec<String>,
    inline_style: StyleProps,
}

#[derive(Debug)]
struct ParsedMindmap {
    title: Option<String>,
    nodes: Vec<MindNode>,
    class_defs: HashMap<String, StyleProps>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Side {
    Right,
    Left,
}

fn is_valid_id(s: &str) -> bool {
    !s.is_empty()
        && s.chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '-' | '.'))
}

fn strip_markdown_quotes(raw: &str) -> String {
    let mut s = unquote_text(raw.trim());
    if let Some(inner) = s.strip_prefix('`').and_then(|r| r.strip_suffix('`')) {
        s = inner.trim().replace("**", "").replace("__", "");
    }
    s
}

fn strip_icons(raw: &str) -> String {
    let mut out = raw.to_string();
    while let Some(start) = out.find("::icon(") {
        let rest = &out[start + "::icon(".len()..];
        let mut depth = 1usize;
        let mut end_rel = None;
        for (idx, ch) in rest.char_indices() {
            match ch {
                '(' => depth += 1,
                ')' => {
                    depth -= 1;
                    if depth == 0 {
                        end_rel = Some(idx + 1);
                        break;
                    }
                }
                _ => {}
            }
        }
        if let Some(rel) = end_rel {
            let remove_end = start + "::icon(".len() + rel;
            out.replace_range(start..remove_end, " ");
        } else {
            out.truncate(start);
            break;
        }
    }
    out.trim().to_string()
}

fn split_mind_classes(raw: &str) -> (&str, Vec<String>) {
    let bytes = raw.as_bytes();
    let mut in_quotes = false;
    let mut in_ticks = false;
    let mut i = 0;
    let mut found = None;
    while i + 2 < bytes.len() {
        let ch = bytes[i];
        if ch == b'"' && !in_ticks {
            in_quotes = !in_quotes;
        } else if ch == b'`' && !in_quotes {
            in_ticks = !in_ticks;
        } else if !in_quotes && !in_ticks && bytes[i..].starts_with(b":::") {
            found = Some(i);
            break;
        }
        i += 1;
    }
    let Some(pos) = found else {
        return (raw.trim(), Vec::new());
    };
    let head = raw[..pos].trim();
    let tail = raw[pos + 3..].trim();
    let classes = tail
        .split([',', ' ', '\t'])
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string)
        .collect();
    (head, classes)
}

fn parse_mind_node_spec(raw: &str) -> (Option<String>, MindShape, String) {
    let cleaned = strip_icons(raw);
    let node_part = cleaned.trim();

    let delims: &[(&str, &str, MindShape)] = &[
        ("))", "((", MindShape::Bang),
        ("(((", ")))", MindShape::Circle),
        ("((", "))", MindShape::Circle),
        ("{{", "}}", MindShape::Hexagon),
        (")", "(", MindShape::Cloud),
        ("(", ")", MindShape::Rounded),
        ("[", "]", MindShape::Square),
    ];

    for &(open, close, shape) in delims {
        if let Some(before_close) = node_part.strip_suffix(close)
            && let Some(open_idx) = before_close.find(open)
        {
            let id_prefix = before_close[..open_idx].trim();
            if open_idx == 0 || is_valid_id(id_prefix) {
                let inner = before_close[open_idx + open.len()..].trim();
                if !inner.is_empty() {
                    let id = if open_idx > 0 {
                        Some(id_prefix.to_string())
                    } else {
                        None
                    };
                    return (id, shape, strip_markdown_quotes(inner));
                }
            }
        }
    }

    (None, MindShape::Default, strip_markdown_quotes(node_part))
}

fn parse_mindmap(lines: &[SourceLine]) -> Result<ParsedMindmap> {
    let mut title = None;
    let mut nodes: Vec<MindNode> = Vec::new();
    let mut class_defs: HashMap<String, StyleProps> = HashMap::new();
    let mut pending_node_classes: Vec<(String, Vec<String>)> = Vec::new();
    let mut pending_node_styles: Vec<(String, StyleProps)> = Vec::new();
    let mut stack: Vec<(usize, usize)> = Vec::new();
    let mut root_indent = 0usize;
    let mut in_acc_descr = false;
    let mut last_node_idx: Option<usize> = None;

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
        if trimmed.starts_with("::icon(") {
            continue;
        }
        if let Some(cls_tail) = trimmed.strip_prefix(":::") {
            if let Some(idx) = last_node_idx {
                for c in cls_tail
                    .split([',', ' ', '\t'])
                    .map(str::trim)
                    .filter(|s| !s.is_empty())
                {
                    if !nodes[idx].classes.iter().any(|x| x == c) {
                        nodes[idx].classes.push(c.to_string());
                    }
                }
            }
            continue;
        }

        if nodes.is_empty()
            && let Some(rest) = trimmed.strip_prefix("title")
            && (rest.is_empty() || rest.starts_with(char::is_whitespace))
        {
            let t = rest.trim();
            if !t.is_empty() {
                title = Some(strip_markdown_quotes(t));
            }
            continue;
        }

        if let Some(directive) = try_parse_style_directive(trimmed, sl.line_no)? {
            match directive {
                StyleDirective::ClassDef(name, props) => {
                    class_defs.entry(name).or_default().merge_from(&props);
                }
                StyleDirective::ClassAssign { ids, classes } => {
                    for id in ids {
                        pending_node_classes.push((id, classes.clone()));
                    }
                }
                StyleDirective::NodeStyle { ids, props } => {
                    for id in ids {
                        pending_node_styles.push((id, props.clone()));
                    }
                }
            }
            continue;
        }

        let (head, inline_classes) = split_mind_classes(trimmed);
        let (id, shape, label) = parse_mind_node_spec(head);
        if label.is_empty() {
            return Err(Error::Mermaid {
                line: Some(sl.line_no),
                message: "empty mindmap node label".to_string(),
            });
        }

        let idx = nodes.len();
        let (parent, depth) = if idx == 0 {
            root_indent = sl.indent;
            stack.push((sl.indent, idx));
            (None, 0)
        } else {
            while stack.last().is_some_and(|&(pi, _)| pi >= sl.indent) {
                stack.pop();
            }
            if stack.is_empty() || sl.indent <= root_indent {
                return Err(Error::Mermaid {
                    line: Some(sl.line_no),
                    message: "mindmap allows only a single root node".to_string(),
                });
            }
            let p = stack.last().unwrap().1;
            let d = nodes[p].depth + 1;
            nodes[p].children.push(idx);
            stack.push((sl.indent, idx));
            (Some(p), d)
        };

        nodes.push(MindNode {
            id,
            label,
            shape,
            parent,
            children: Vec::new(),
            depth,
            classes: inline_classes,
            inline_style: StyleProps::default(),
        });
        last_node_idx = Some(idx);
    }

    if nodes.is_empty() {
        return Err(Error::Mermaid {
            line: lines.first().map(|l| l.line_no),
            message: "mindmap has no nodes".to_string(),
        });
    }

    for (target_id, classes) in pending_node_classes {
        for n in &mut nodes {
            if n.id.as_deref() == Some(target_id.as_str()) {
                for c in &classes {
                    if !n.classes.contains(c) {
                        n.classes.push(c.clone());
                    }
                }
            }
        }
    }
    for (target_id, props) in pending_node_styles {
        for n in &mut nodes {
            if n.id.as_deref() == Some(target_id.as_str()) {
                n.inline_style.merge_from(&props);
            }
        }
    }

    Ok(ParsedMindmap {
        title,
        nodes,
        class_defs,
    })
}

fn fallback_branch_palette() -> [Color; 7] {
    [
        Color::from("#3b82f6"),
        Color::from("#10b981"),
        Color::from("#f59e0b"),
        Color::from("#8b5cf6"),
        Color::from("#ec4899"),
        Color::from("#06b6d4"),
        Color::from("#ef4444"),
    ]
}

fn contrast_text(bg: Color) -> Color {
    let luminance = 0.2126 * (bg.r as f32) + 0.7152 * (bg.g as f32) + 0.0722 * (bg.b as f32);
    if luminance < 145.0 {
        Color::from("#f8fafc")
    } else {
        Color::from("#1e293b")
    }
}

fn build_cloud_path(cx: f32, cy: f32, w: f32, h: f32) -> String {
    let rx = w * 0.5;
    let ry = h * 0.5;
    let pts: &[(f32, f32)] = &[
        (-0.82, 0.08),
        (-0.76, -0.58),
        (-0.32, -0.82),
        (0.18, -0.86),
        (0.66, -0.64),
        (0.84, -0.10),
        (0.80, 0.46),
        (0.42, 0.80),
        (-0.08, 0.84),
        (-0.56, 0.68),
    ];
    let n = pts.len();
    let mut d = String::with_capacity(256);
    let p0 = (cx + pts[0].0 * rx, cy + pts[0].1 * ry);
    let _ = write!(d, "M {} {}", format_float(p0.0), format_float(p0.1));
    for i in 0..n {
        let a = pts[i];
        let b = pts[(i + 1) % n];
        let mx = (a.0 + b.0) * 0.5;
        let my = (a.1 + b.1) * 0.5;
        let len = (mx * mx + my * my).sqrt().max(0.01);
        let bulge = 1.24;
        let kx = cx + mx / len * bulge * rx;
        let ky = cy + my / len * bulge * ry;
        let bx = cx + b.0 * rx;
        let by = cy + b.1 * ry;
        let _ = write!(
            d,
            " Q {} {} {} {}",
            format_float(kx),
            format_float(ky),
            format_float(bx),
            format_float(by)
        );
    }
    d.push('Z');
    d
}

fn build_bang_path(cx: f32, cy: f32, w: f32, h: f32) -> String {
    let rx = w * 0.5;
    let ry = h * 0.5;
    let spikes = 14usize;
    let mut d = String::with_capacity(320);
    for i in 0..spikes {
        let a0 = (i as f32) / (spikes as f32) * std::f32::consts::TAU;
        let a_mid = ((i as f32) + 0.5) / (spikes as f32) * std::f32::consts::TAU;
        let a1 = ((i + 1) as f32) / (spikes as f32) * std::f32::consts::TAU;
        let inner_r = 0.82;
        let ctrl_r = 1.16;
        let x0 = cx + rx * inner_r * a0.cos();
        let y0 = cy + ry * inner_r * a0.sin();
        let kx = cx + rx * ctrl_r * a_mid.cos();
        let ky = cy + ry * ctrl_r * a_mid.sin();
        let x1 = cx + rx * inner_r * a1.cos();
        let y1 = cy + ry * inner_r * a1.sin();
        if i == 0 {
            let _ = write!(d, "M {} {}", format_float(x0), format_float(y0));
        }
        let _ = write!(
            d,
            " Q {} {} {} {}",
            format_float(kx),
            format_float(ky),
            format_float(x1),
            format_float(y1)
        );
    }
    d.push('Z');
    d
}

pub(crate) fn render(
    chart: &MermaidChart,
    theme_name: &str,
    fm_title: Option<&str>,
    lines: &[SourceLine],
) -> Result<(String, f32, f32)> {
    let mm = parse_mindmap(lines)?;
    let theme = get_theme(theme_name);
    let bg_override = chart
        .has_custom_background(&theme)
        .then_some(chart.base.background_color);
    let palette = MermaidPalette::from_theme(&theme, bg_override);
    let font_family = chart.resolved_font_family(&theme);

    let title_text = if !chart.base.title.text.is_empty() {
        Some(chart.base.title.text.clone())
    } else {
        fm_title.map(str::to_string).or(mm.title.clone())
    };
    let header_h = super::compute_header_height(chart, title_text.as_deref(), &theme);

    struct MeasuredNode {
        ft: FormattedText,
        font_size: f32,
        w: f32,
        h: f32,
        extra_pad: f32,
    }

    let mut measured = Vec::with_capacity(mm.nodes.len());
    for n in &mm.nodes {
        let font_size = match n.depth {
            0 => 15.0,
            1 => 13.5,
            _ => 12.5,
        };
        let ft = format_and_measure(&n.label, &font_family, font_size);
        let (pad_x, pad_y) = match n.depth {
            0 => (18.0, 12.0),
            1 => (14.0, 9.0),
            _ => (12.0, 7.0),
        };
        let (w, h, extra_pad) = match n.shape {
            MindShape::Default | MindShape::Square | MindShape::Rounded => (
                (ft.width + pad_x * 2.0).max(58.0),
                (ft.height + pad_y * 2.0).max(32.0),
                2.0,
            ),
            MindShape::Hexagon => (
                (ft.width + pad_x * 2.0 + 22.0).max(74.0),
                (ft.height + pad_y * 2.0 + 4.0).max(36.0),
                2.0,
            ),
            MindShape::Circle => {
                let d = (ft.width.max(ft.height) + 26.0).max(62.0);
                (d, d, 2.0)
            }
            MindShape::Cloud => (
                (ft.width + pad_x * 2.0 + 24.0).max(106.0),
                (ft.height + pad_y * 2.0 + 18.0).max(56.0),
                10.0,
            ),
            MindShape::Bang => (
                (ft.width + pad_x * 2.0 + 26.0).max(102.0),
                (ft.height + pad_y * 2.0 + 20.0).max(58.0),
                10.0,
            ),
        };
        measured.push(MeasuredNode {
            ft,
            font_size,
            w,
            h,
            extra_pad,
        });
    }

    let v_gap = 14.0_f32;
    let branch_gap = 22.0_f32;
    let root_h_gap = 46.0_f32;
    let child_h_gap = 36.0_f32;

    let mut subtree_h = vec![0.0_f32; mm.nodes.len()];
    for idx in (0..mm.nodes.len()).rev() {
        let ch = &mm.nodes[idx].children;
        if ch.is_empty() {
            subtree_h[idx] = measured[idx].h;
        } else {
            let sum_c: f32 = ch.iter().map(|&c| subtree_h[c]).sum();
            let gap = if idx == 0 { branch_gap } else { v_gap };
            let total_c = sum_c + gap * (ch.len().saturating_sub(1) as f32);
            subtree_h[idx] = measured[idx].h.max(total_c);
        }
    }

    struct PlaceCtx<'a> {
        nodes: &'a [MindNode],
        measured: &'a [MeasuredNode],
        subtree_h: &'a [f32],
        h_gap: f32,
        v_gap: f32,
        pos_cx: Vec<f32>,
        pos_cy: Vec<f32>,
        branch_idx: Vec<usize>,
    }

    fn place_subtree(
        ctx: &mut PlaceCtx<'_>,
        u: usize,
        parent_outer_x: f32,
        top_y: f32,
        side: Side,
        top_branch_idx: usize,
    ) {
        ctx.branch_idx[u] = top_branch_idx;
        let w = ctx.measured[u].w;
        let cx = match side {
            Side::Right => parent_outer_x + ctx.h_gap + w * 0.5,
            Side::Left => parent_outer_x - ctx.h_gap - w * 0.5,
        };
        ctx.pos_cx[u] = cx;
        let cy = top_y + ctx.subtree_h[u] * 0.5;
        ctx.pos_cy[u] = cy;

        let ch = &ctx.nodes[u].children;
        if ch.is_empty() {
            return;
        }
        let sum_c: f32 =
            ch.iter().map(|&c| ctx.subtree_h[c]).sum::<f32>() + ctx.v_gap * (ch.len() - 1) as f32;
        let mut cursor_y = cy - sum_c * 0.5;
        let next_outer_x = match side {
            Side::Right => cx + w * 0.5,
            Side::Left => cx - w * 0.5,
        };
        for &c in ch {
            place_subtree(ctx, c, next_outer_x, cursor_y, side, top_branch_idx);
            cursor_y += ctx.subtree_h[c] + ctx.v_gap;
        }
    }

    let mut ctx = PlaceCtx {
        nodes: &mm.nodes,
        measured: &measured,
        subtree_h: &subtree_h,
        h_gap: child_h_gap,
        v_gap,
        pos_cx: vec![0.0_f32; mm.nodes.len()],
        pos_cy: vec![0.0_f32; mm.nodes.len()],
        branch_idx: vec![0usize; mm.nodes.len()],
    };

    let root_children = &mm.nodes[0].children;
    let right_count = root_children.len().div_ceil(2);
    let right_branches = &root_children[..right_count];
    let left_branches = &root_children[right_count..];

    let stack_height = |branches: &[usize]| -> f32 {
        if branches.is_empty() {
            0.0
        } else {
            branches.iter().map(|&b| subtree_h[b]).sum::<f32>()
                + branch_gap * (branches.len() - 1) as f32
        }
    };

    let right_total_h = stack_height(right_branches);
    let mut cursor_y = -right_total_h * 0.5;
    for (bi, &b) in right_branches.iter().enumerate() {
        place_subtree(
            &mut ctx,
            b,
            measured[0].w * 0.5 + (root_h_gap - child_h_gap),
            cursor_y,
            Side::Right,
            bi,
        );
        cursor_y += subtree_h[b] + branch_gap;
    }

    let left_total_h = stack_height(left_branches);
    let mut cursor_y = -left_total_h * 0.5;
    for (bi, &b) in left_branches.iter().enumerate() {
        place_subtree(
            &mut ctx,
            b,
            -measured[0].w * 0.5 - (root_h_gap - child_h_gap),
            cursor_y,
            Side::Left,
            right_count + bi,
        );
        cursor_y += subtree_h[b] + branch_gap;
    }

    let PlaceCtx {
        pos_cx,
        pos_cy,
        branch_idx,
        ..
    } = ctx;

    let mut min_x = f32::INFINITY;
    let mut max_x = f32::NEG_INFINITY;
    let mut min_y = f32::INFINITY;
    let mut max_y = f32::NEG_INFINITY;
    for (i, m) in measured.iter().enumerate() {
        let hw = m.w * 0.5 + m.extra_pad;
        let hh = m.h * 0.5 + m.extra_pad;
        min_x = min_x.min(pos_cx[i] - hw);
        max_x = max_x.max(pos_cx[i] + hw);
        min_y = min_y.min(pos_cy[i] - hh);
        max_y = max_y.max(pos_cy[i] + hh);
    }

    let pad = 24.0_f32;
    let graph_w = (max_x - min_x + pad * 2.0).max(200.0);
    let graph_h = (max_y - min_y + pad * 2.0).max(120.0);
    let shift_x = pad - min_x;
    let shift_y = pad - min_y;

    let content_w = (graph_w + chart.base.margin.left + chart.base.margin.right).max(1.0);
    let content_h =
        (graph_h + header_h + chart.base.margin.top + chart.base.margin.bottom).max(1.0);
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

    let mut cc = canvas.child(super::super::util::Box {
        left: chart.base.margin.left,
        top: chart.base.margin.top + header_h,
        right: chart.base.margin.right,
        bottom: chart.base.margin.bottom,
    });

    let branch_colors = if theme.series.colors.is_empty() {
        fallback_branch_palette().to_vec()
    } else {
        theme.series.colors.clone()
    };

    // 1. S-shaped cubic Bézier links (parent -> child).
    for (idx, node) in mm.nodes.iter().enumerate().skip(1) {
        let Some(p) = node.parent else { continue };
        let b_color = branch_colors[branch_idx[idx] % branch_colors.len()];
        let fade = ((node.depth.saturating_sub(1) as f32) * 0.14).min(0.45);
        let link_color = mix_color(b_color, palette.text_color, fade * 0.35);
        let stroke_w = match node.depth {
            1 => 2.6,
            2 => 2.0,
            _ => 1.5,
        };

        let px = pos_cx[p] + shift_x;
        let py = pos_cy[p] + shift_y;
        let cx = pos_cx[idx] + shift_x;
        let cy = pos_cy[idx] + shift_y;
        let going_right = cx >= px;

        let x1 = if going_right {
            px + measured[p].w * 0.38
        } else {
            px - measured[p].w * 0.38
        };
        let x2 = if going_right {
            cx - measured[idx].w * 0.44
        } else {
            cx + measured[idx].w * 0.44
        };
        let mx1 = x1 + (x2 - x1) * 0.55;
        let mx2 = x1 + (x2 - x1) * 0.45;

        let mut d = String::with_capacity(96);
        d.push('M');
        write_float(&mut d, x1);
        d.push(',');
        write_float(&mut d, py);
        d.push_str(" C");
        write_float(&mut d, mx1);
        d.push(',');
        write_float(&mut d, py);
        d.push(' ');
        write_float(&mut d, mx2);
        d.push(',');
        write_float(&mut d, cy);
        d.push(' ');
        write_float(&mut d, x2);
        d.push(',');
        write_float(&mut d, cy);

        cc.raw_path(RawPath {
            d,
            fill: None,
            stroke: Some(link_color),
            stroke_width: stroke_w,
            stroke_linecap: Some("round"),
            stroke_linejoin: Some("round"),
            ..Default::default()
        });
    }

    // 2. Nodes (shapes + crisp labels).
    for (idx, node) in mm.nodes.iter().enumerate() {
        let m = &measured[idx];
        let cx = pos_cx[idx] + shift_x;
        let cy = pos_cy[idx] + shift_y;
        let eff_style = resolve_style(&node.classes, &node.inline_style, &mm.class_defs);
        let b_color = branch_colors[branch_idx[idx] % branch_colors.len()];

        let default_fill = if node.depth == 0 {
            palette.accent
        } else if node.depth == 1 {
            mix_color(
                b_color,
                palette.opaque_bg,
                if palette.is_light { 0.80 } else { 0.60 },
            )
        } else {
            mix_color(
                b_color,
                palette.opaque_bg,
                if palette.is_light { 0.90 } else { 0.72 },
            )
        };
        let default_stroke = if node.depth == 0 {
            mix_color(palette.accent, palette.text_color, 0.20)
        } else {
            mix_color(
                b_color,
                palette.opaque_bg,
                if palette.is_light { 0.25 } else { 0.15 },
            )
        };

        let fill = eff_style.fill.unwrap_or(default_fill);
        let stroke = eff_style.stroke.unwrap_or(default_stroke);
        let stroke_w = eff_style
            .stroke_width
            .unwrap_or(if node.depth <= 1 { 2.0 } else { 1.4 });
        let text_color = eff_style.color.unwrap_or_else(|| contrast_text(fill));
        let left = cx - m.w * 0.5;
        let top = cy - m.h * 0.5;

        match node.shape {
            MindShape::Default => {
                cc.rect(Rect {
                    left,
                    top,
                    width: m.w,
                    height: m.h,
                    rx: Some(8.0),
                    ry: Some(8.0),
                    fill: Some(fill.into()),
                    color: Some(stroke),
                    stroke_width: Some(stroke_w),
                    stroke_dash_array: eff_style.stroke_dasharray,
                    ..Default::default()
                });
            }
            MindShape::Square => {
                cc.rect(Rect {
                    left,
                    top,
                    width: m.w,
                    height: m.h,
                    rx: Some(3.0),
                    ry: Some(3.0),
                    fill: Some(fill.into()),
                    color: Some(stroke),
                    stroke_width: Some(stroke_w),
                    stroke_dash_array: eff_style.stroke_dasharray,
                    ..Default::default()
                });
            }
            MindShape::Rounded => {
                let r = (m.h * 0.45).min(18.0);
                cc.rect(Rect {
                    left,
                    top,
                    width: m.w,
                    height: m.h,
                    rx: Some(r),
                    ry: Some(r),
                    fill: Some(fill.into()),
                    color: Some(stroke),
                    stroke_width: Some(stroke_w),
                    stroke_dash_array: eff_style.stroke_dasharray,
                    ..Default::default()
                });
            }
            MindShape::Circle => {
                cc.circle(Circle {
                    cx,
                    cy,
                    r: m.w * 0.5,
                    fill: Some(fill),
                    stroke_color: Some(stroke),
                    stroke_width: stroke_w,
                    ..Default::default()
                });
            }
            MindShape::Hexagon => {
                let dx = (m.h * 0.28).clamp(10.0, 18.0);
                let d = format!(
                    "M {} {} L {} {} L {} {} L {} {} L {} {} L {} {} Z",
                    format_float(left + dx),
                    format_float(top),
                    format_float(left + m.w - dx),
                    format_float(top),
                    format_float(left + m.w),
                    format_float(cy),
                    format_float(left + m.w - dx),
                    format_float(top + m.h),
                    format_float(left + dx),
                    format_float(top + m.h),
                    format_float(left),
                    format_float(cy),
                );
                cc.raw_path(RawPath {
                    d,
                    fill: Some(fill),
                    stroke: Some(stroke),
                    stroke_width: stroke_w,
                    stroke_dash_array: eff_style.stroke_dasharray,
                    stroke_linejoin: Some("round"),
                    ..Default::default()
                });
            }
            MindShape::Cloud => {
                cc.raw_path(RawPath {
                    d: build_cloud_path(cx, cy, m.w, m.h),
                    fill: Some(fill),
                    stroke: Some(stroke),
                    stroke_width: stroke_w,
                    stroke_dash_array: eff_style.stroke_dasharray,
                    stroke_linejoin: Some("round"),
                    ..Default::default()
                });
            }
            MindShape::Bang => {
                cc.raw_path(RawPath {
                    d: build_bang_path(cx, cy, m.w, m.h),
                    fill: Some(fill),
                    stroke: Some(stroke),
                    stroke_width: stroke_w,
                    stroke_dash_array: eff_style.stroke_dasharray,
                    stroke_linejoin: Some("round"),
                    ..Default::default()
                });
            }
        }

        draw_multiline_text(
            &mut cc,
            cx,
            cy,
            &m.ft,
            &font_family,
            m.font_size,
            text_color,
            "middle",
            node.depth <= 1,
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
    fn renders_all_seven_mindmap_shapes_and_styles() {
        let src = r#"
mindmap
  root((Core Vision))
    )Cloud Branch(
      [Square Item]
      "`Markdown **Bold**`"
    {{Hexagon Topic}}
      ))Bang Alert((
      (Rounded Leaf)
        ::icon(fa fa-bolt)
      StyledLeaf:::urgent
  classDef urgent fill:#dc2626,color:#ffffff,stroke:#991b1b
"#;
        let prep = preprocess(src).unwrap();
        let chart = MermaidChart::new(src);
        let (svg, w, h) = render(&chart, "light", prep.title.as_deref(), &prep.lines).unwrap();
        assert!(svg.starts_with("<svg") && w > 200.0 && h > 150.0);
        assert!(svg.contains("Core Vision"));
        assert!(svg.contains("Cloud Branch"));
        assert!(svg.contains("Bang Alert"));
        assert!(svg.contains("Markdown"));
        assert!(!svg.contains("fa-bolt"));
    }

    #[test]
    fn rejects_multiple_roots_with_line_number() {
        let src = "mindmap\n  RootOne\n    Child\n  RootTwo";
        let prep = preprocess(src).unwrap();
        let err = parse_mindmap(&prep.lines).unwrap_err().to_string();
        assert!(err.contains("line 4"), "{err}");
    }
}
