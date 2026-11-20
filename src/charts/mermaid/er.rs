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

//! Mermaid `erDiagram` parser and pure-SVG renderer.

use std::collections::HashMap;

use super::super::color::Color;
use super::super::component::{Line, RawPath, Rect};
use super::super::error::{Error, Result};
use super::super::font::measure_text_width_family;
use super::super::theme::get_theme;
use super::MermaidChart;
use super::draw::{
    EdgeMarker, build_smooth_edge_path, clip_point_to_rect, draw_edge_label, draw_edge_marker,
};
use super::layout::{LayeredEdgeInput, LayeredGraph, LayeredNodeInput, sugiyama};
use super::preprocess::{Direction, SourceLine};
use super::style::{
    MermaidPalette, StyleDirective, StyleProps, mix_color, resolve_style, split_inline_classes,
    try_parse_style_directive, unquote_text,
};
use super::text::{FormattedText, draw_multiline_text, format_and_measure};

#[derive(Clone, Debug)]
struct ErAttr {
    attr_type: String,
    name: String,
    keys: Option<String>,
    comment: Option<String>,
}

#[derive(Clone, Debug)]
struct ErEntity {
    id: String,
    display_name: Option<String>,
    attrs: Vec<ErAttr>,
    classes: Vec<String>,
    inline_style: StyleProps,
}

#[derive(Clone, Debug)]
struct ErRelation {
    from: usize,
    to: usize,
    left_marker: EdgeMarker,
    right_marker: EdgeMarker,
    dashed: bool,
    label: Option<String>,
}

#[derive(Debug)]
struct ParsedErDiagram {
    direction: Direction,
    entities: Vec<ErEntity>,
    entity_map: HashMap<String, usize>,
    relations: Vec<ErRelation>,
    class_defs: HashMap<String, StyleProps>,
}

impl ParsedErDiagram {
    fn new() -> Self {
        Self {
            direction: Direction::TopBottom,
            entities: Vec::new(),
            entity_map: HashMap::new(),
            relations: Vec::new(),
            class_defs: HashMap::new(),
        }
    }

    fn ensure_entity(&mut self, raw_tok: &str, display_opt: Option<String>) -> usize {
        let (clean, classes) = split_inline_classes(raw_tok);
        let id = unquote_text(clean);
        let idx = if let Some(&existing) = self.entity_map.get(&id) {
            existing
        } else {
            let new_idx = self.entities.len();
            self.entity_map.insert(id.clone(), new_idx);
            self.entities.push(ErEntity {
                id,
                display_name: None,
                attrs: Vec::new(),
                classes: Vec::new(),
                inline_style: StyleProps::default(),
            });
            new_idx
        };

        let ent = &mut self.entities[idx];
        if let Some(d) = display_opt {
            ent.display_name = Some(d);
        }
        for cls in classes {
            if !ent.classes.contains(&cls) {
                ent.classes.push(cls);
            }
        }
        idx
    }
}

fn parse_entity_header(raw: &str, line_no: usize) -> Result<(&str, Option<String>)> {
    let r = raw.trim();
    if r.is_empty() {
        return Err(Error::Mermaid {
            line: Some(line_no),
            message: "expected entity name".to_string(),
        });
    }
    if let Some(lb) = r.find('[')
        && let Some(rb) = r.rfind(']')
        && rb > lb
    {
        let id_part = r[..lb].trim();
        let display = unquote_text(&r[lb + 1..rb]);
        let suffix = r[rb + 1..].trim();
        if !suffix.is_empty() && suffix.starts_with(":::") {
            // Re-attach trailing inline classes to ID for downstream splitting.
            return Err(Error::Mermaid {
                line: Some(line_no),
                message: format!("place `:::` on entity name rather than after alias in `{r}`"),
            });
        }
        return Ok((id_part, Some(display)));
    }
    Ok((r, None))
}

fn parse_attr_line(line: &str, line_no: usize) -> Result<ErAttr> {
    let trimmed = line.trim();
    let (main, comment) = if let Some(without_end) = trimmed.strip_suffix('"') {
        if let Some(open) = without_end.rfind('"')
            && (open == 0 || without_end[..open].ends_with(char::is_whitespace))
        {
            (
                without_end[..open].trim(),
                Some(without_end[open + 1..].replace("\\\"", "\"")),
            )
        } else {
            (trimmed, None)
        }
    } else {
        (trimmed, None)
    };

    let tokens: Vec<&str> = main.split_whitespace().collect();
    if tokens.len() < 2 {
        return Err(Error::Mermaid {
            line: Some(line_no),
            message: format!("expected `<type> <name> [PK|FK|UK]` in entity block, got `{line}`"),
        });
    }

    let mut keys_list: Vec<String> = Vec::new();
    for &tok in &tokens[2..] {
        for part in tok.split(',') {
            let k = part.trim().to_ascii_uppercase();
            if !k.is_empty() {
                keys_list.push(k);
            }
        }
    }
    let keys = (!keys_list.is_empty()).then(|| keys_list.join(", "));

    Ok(ErAttr {
        attr_type: tokens[0].to_string(),
        name: tokens[1].to_string(),
        keys,
        comment,
    })
}

fn split_er_label(raw: &str) -> (&str, Option<String>) {
    let bytes = raw.as_bytes();
    let mut in_quotes = false;
    let mut i = 0;
    while i < bytes.len() {
        let b = bytes[i];
        if in_quotes {
            if b == b'\\' && i + 1 < bytes.len() {
                i += 2;
                continue;
            }
            if b == b'"' {
                in_quotes = false;
            }
            i += 1;
            continue;
        }
        if b == b'"' {
            in_quotes = true;
            i += 1;
            continue;
        }
        if b == b':' && !raw[i..].starts_with(":::") {
            let lbl = unquote_text(&raw[i + 1..]);
            return (raw[..i].trim(), (!lbl.is_empty()).then_some(lbl));
        }
        i += 1;
    }
    (raw.trim(), None)
}

fn parse_er_symbol(sym: &str) -> Option<EdgeMarker> {
    match sym {
        "||" => Some(EdgeMarker::ErOnlyOne),
        "|o" | "o|" => Some(EdgeMarker::ErZeroOrOne),
        "}|" | "|{" => Some(EdgeMarker::ErOneOrMore),
        "}o" | "o{" => Some(EdgeMarker::ErZeroOrMore),
        _ => None,
    }
}

fn parse_er_phrase(phrase: &str) -> Option<EdgeMarker> {
    let norm: Vec<&str> = phrase.split_whitespace().collect();
    let joined = norm.join(" ").to_ascii_lowercase();
    match joined.as_str() {
        "only one" | "1" => Some(EdgeMarker::ErOnlyOne),
        "zero or one" | "one or zero" | "0..1" => Some(EdgeMarker::ErZeroOrOne),
        "one or more" | "one or many" | "1+" | "1..*" => Some(EdgeMarker::ErOneOrMore),
        "zero or more" | "zero or many" | "0+" | "0..*" => Some(EdgeMarker::ErZeroOrMore),
        _ => None,
    }
}

struct ParsedErRel<'a> {
    left_ent: &'a str,
    right_ent: &'a str,
    left_marker: EdgeMarker,
    right_marker: EdgeMarker,
    dashed: bool,
    label: Option<String>,
}

fn take_first_entity(s: &str) -> Option<(&str, &str)> {
    let t = s.trim_start();
    if let Some(after_q) = t.strip_prefix('"') {
        let bytes = after_q.as_bytes();
        let mut i = 0;
        while i < bytes.len() {
            if bytes[i] == b'\\' && i + 1 < bytes.len() {
                i += 2;
                continue;
            }
            if bytes[i] == b'"' {
                let end = t.len() - after_q.len() + i + 1;
                return Some((t[..end].trim(), t[end..].trim()));
            }
            i += 1;
        }
        return None;
    }
    let idx = t.find(char::is_whitespace)?;
    Some((t[..idx].trim(), t[idx..].trim()))
}

fn take_last_entity(s: &str) -> Option<(&str, &str)> {
    let t = s.trim_end();
    if let Some(before_q) = t.strip_suffix('"') {
        let open = before_q.rfind('"')?;
        return Some((t[..open].trim(), t[open..].trim()));
    }
    let idx = t.rfind(char::is_whitespace)?;
    Some((t[..idx].trim(), t[idx..].trim()))
}

fn try_parse_er_relation(trimmed: &str) -> Option<ParsedErRel<'_>> {
    let (main, label) = split_er_label(trimmed);
    let (left_ent, after_left) = take_first_entity(main)?;
    let (middle_part, right_ent) = take_last_entity(after_left)?;
    if left_ent.is_empty() || right_ent.is_empty() || middle_part.is_empty() {
        return None;
    }

    // 1. Try symbolic notation (`||--o{`, `}|..||`, etc.).
    let compact: String = middle_part.split_whitespace().collect();
    for (stem, dashed) in [("--", false), ("..", true)] {
        if let Some((ls, rs)) = compact.split_once(stem)
            && let Some(lm) = parse_er_symbol(ls)
            && let Some(rm) = parse_er_symbol(rs)
        {
            return Some(ParsedErRel {
                left_ent,
                right_ent,
                left_marker: lm,
                right_marker: rm,
                dashed,
                label,
            });
        }
    }

    // 2. Try natural-language cardinality aliases (`only one to zero or more`).
    let lower = middle_part.to_ascii_lowercase();
    for (conn, dashed) in [(" optionally to ", true), (" to ", false)] {
        if let Some(pos) = lower.find(conn) {
            let lp = middle_part[..pos].trim();
            let rp = middle_part[pos + conn.len()..].trim();
            if let Some(lm) = parse_er_phrase(lp)
                && let Some(rm) = parse_er_phrase(rp)
            {
                return Some(ParsedErRel {
                    left_ent,
                    right_ent,
                    left_marker: lm,
                    right_marker: rm,
                    dashed,
                    label,
                });
            }
        }
    }

    None
}

fn parse_er_diagram(lines: &[SourceLine]) -> Result<ParsedErDiagram> {
    let mut er = ParsedErDiagram::new();
    let mut open_entity: Option<(usize, usize)> = None; // (entity_idx, open_line)
    let mut in_acc_descr_block = false;

    for sl in lines {
        let trimmed = sl.text.trim();
        if trimmed.is_empty() {
            continue;
        }
        if in_acc_descr_block {
            if trimmed.contains('}') {
                in_acc_descr_block = false;
            }
            continue;
        }
        if trimmed.starts_with("accTitle") {
            continue;
        }
        if trimmed.starts_with("accDescr") {
            if trimmed.contains('{') && !trimmed.contains('}') {
                in_acc_descr_block = true;
            }
            continue;
        }

        if let Some((ent_idx, _)) = open_entity {
            if trimmed == "}" {
                open_entity = None;
            } else {
                let attr = parse_attr_line(trimmed, sl.line_no)?;
                er.entities[ent_idx].attrs.push(attr);
            }
            continue;
        }

        if trimmed == "}" {
            return Err(Error::Mermaid {
                line: Some(sl.line_no),
                message: "unexpected `}` with no open entity `{`".to_string(),
            });
        }

        if let Some(rest) = trimmed.strip_prefix("direction")
            && rest.starts_with(char::is_whitespace)
        {
            let dir_tok = rest.trim();
            er.direction = Direction::from_token(dir_tok).ok_or_else(|| Error::Mermaid {
                line: Some(sl.line_no),
                message: format!("unknown direction `{dir_tok}`"),
            })?;
            continue;
        }

        if let Some(directive) = try_parse_style_directive(trimmed, sl.line_no)? {
            match directive {
                StyleDirective::ClassDef(name, props) => {
                    er.class_defs.entry(name).or_default().merge_from(&props);
                }
                StyleDirective::ClassAssign { ids, classes } => {
                    for id in ids {
                        let idx = er.ensure_entity(&id, None);
                        for cls in &classes {
                            if !er.entities[idx].classes.contains(cls) {
                                er.entities[idx].classes.push(cls.clone());
                            }
                        }
                    }
                }
                StyleDirective::NodeStyle { ids, props } => {
                    for id in ids {
                        let idx = er.ensure_entity(&id, None);
                        er.entities[idx].inline_style.merge_from(&props);
                    }
                }
            }
            continue;
        }

        if let Some(header) = trimmed.strip_suffix('{') {
            let (id_part, display_opt) = parse_entity_header(header, sl.line_no)?;
            let idx = er.ensure_entity(id_part, display_opt);
            open_entity = Some((idx, sl.line_no));
            continue;
        }

        if let Some(rel) = try_parse_er_relation(trimmed) {
            let from = er.ensure_entity(rel.left_ent, None);
            let to = er.ensure_entity(rel.right_ent, None);
            er.relations.push(ErRelation {
                from,
                to,
                left_marker: rel.left_marker,
                right_marker: rel.right_marker,
                dashed: rel.dashed,
                label: rel.label,
            });
            continue;
        }

        if trimmed.contains('[') && trimmed.ends_with(']') && !trimmed.contains(char::is_whitespace)
        {
            let (id_part, display_opt) = parse_entity_header(trimmed, sl.line_no)?;
            er.ensure_entity(id_part, display_opt);
            continue;
        }

        if !trimmed.contains(char::is_whitespace) {
            er.ensure_entity(trimmed, None);
            continue;
        }

        return Err(Error::Mermaid {
            line: Some(sl.line_no),
            message: format!("invalid erDiagram statement `{trimmed}`"),
        });
    }

    if let Some((_, open_line)) = open_entity {
        return Err(Error::Mermaid {
            line: Some(open_line),
            message: "unclosed entity block (missing `}`)".to_string(),
        });
    }

    Ok(er)
}

fn measure_str_w(font_family: &str, font_size: f32, s: &str) -> f32 {
    if s.is_empty() {
        return 0.0;
    }
    measure_text_width_family(font_family, font_size, s)
        .map(|b| b.width())
        .unwrap_or_else(|_| s.chars().count() as f32 * font_size * 0.6)
}

pub(crate) fn render(
    chart: &MermaidChart,
    theme_name: &str,
    fm_title: Option<&str>,
    lines: &[SourceLine],
) -> Result<(String, f32, f32)> {
    let er = parse_er_diagram(lines)?;
    if er.entities.is_empty() {
        return Err(Error::Mermaid {
            line: lines.first().map(|l| l.line_no),
            message: "erDiagram has no entities".to_string(),
        });
    }

    let theme = get_theme(theme_name);
    let bg_override = chart
        .has_custom_background(&theme)
        .then_some(chart.base.background_color);
    let palette = MermaidPalette::from_theme(&theme, bg_override);
    let font_family = chart.resolved_font_family(&theme);
    let title_font_size = 13.5_f32;
    let attr_font_size = 11.5_f32;
    let key_font_size = 10.0_f32;
    let edge_font_size = 12.0_f32;
    let header_h = 30.0_f32;
    let row_h = 24.0_f32;
    let pad_x = 12.0_f32;
    let col_gap = 12.0_f32;

    struct MeasuredEntity {
        title_ft: FormattedText,
        w: f32,
        h: f32,
        type_col_w: f32,
        name_col_w: f32,
        key_col_w: f32,
    }

    let mut measured = Vec::with_capacity(er.entities.len());
    let mut layout_nodes = Vec::with_capacity(er.entities.len());

    for ent in &er.entities {
        let display = ent.display_name.as_deref().unwrap_or(&ent.id);
        let title_ft = format_and_measure(display, &font_family, title_font_size);

        if ent.attrs.is_empty() {
            let w = (title_ft.width + 32.0).max(98.0);
            let h = 40.0_f32;
            layout_nodes.push(LayeredNodeInput {
                width: w,
                height: h,
                subgraph: None,
                attach_to: None,
            });
            measured.push(MeasuredEntity {
                title_ft,
                w,
                h,
                type_col_w: 0.0,
                name_col_w: 0.0,
                key_col_w: 0.0,
            });
            continue;
        }

        let mut max_type_w = 0.0_f32;
        let mut max_name_w = 0.0_f32;
        let mut max_key_w = 0.0_f32;
        let mut max_comment_w = 0.0_f32;

        for a in &ent.attrs {
            max_type_w = max_type_w.max(measure_str_w(&font_family, attr_font_size, &a.attr_type));
            max_name_w = max_name_w.max(measure_str_w(&font_family, attr_font_size, &a.name));
            if let Some(ref k) = a.keys {
                let kw = measure_str_w(&font_family, key_font_size, k) + 12.0;
                max_key_w = max_key_w.max(kw);
            }
            if let Some(ref c) = a.comment {
                let cw = measure_str_w(&font_family, attr_font_size, c);
                max_comment_w = max_comment_w.max(cw);
            }
        }

        let mut cols_w = max_type_w + col_gap + max_name_w;
        if max_key_w > 0.0 {
            cols_w += col_gap + max_key_w;
        }
        if max_comment_w > 0.0 {
            cols_w += col_gap + max_comment_w;
        }
        let w = (cols_w + pad_x * 2.0).max(title_ft.width + 28.0).max(120.0);
        let h = header_h + ent.attrs.len() as f32 * row_h;

        layout_nodes.push(LayeredNodeInput {
            width: w,
            height: h,
            subgraph: None,
            attach_to: None,
        });
        measured.push(MeasuredEntity {
            title_ft,
            w,
            h,
            type_col_w: max_type_w,
            name_col_w: max_name_w,
            key_col_w: max_key_w,
        });
    }

    let layout_edges: Vec<LayeredEdgeInput> = er
        .relations
        .iter()
        .map(|r| LayeredEdgeInput {
            from: r.from,
            to: r.to,
            has_label: r.label.is_some(),
            min_len: 1,
        })
        .collect();

    let layout = sugiyama::layout(&LayeredGraph {
        direction: er.direction,
        rank_sep_extra: 24.0,
        nodes: layout_nodes,
        edges: layout_edges,
        subgraphs: vec![],
    });

    let title_text = if !chart.base.title.text.is_empty() {
        Some(chart.base.title.text.clone())
    } else {
        fm_title.map(str::to_string)
    };
    let top_header_h = super::compute_header_height(chart, title_text.as_deref(), &theme);
    let content_w = (layout.width + chart.base.margin.left + chart.base.margin.right).max(1.0);
    let content_h =
        (layout.height + top_header_h + chart.base.margin.top + chart.base.margin.bottom).max(1.0);

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
        top: chart.base.margin.top + top_header_h,
        right: chart.base.margin.right,
        bottom: chart.base.margin.bottom,
    });

    // 1. Relationship lines and Crow's Foot markers.
    for (idx, r) in er.relations.iter().enumerate() {
        let routed = &layout.edges[idx];
        if routed.waypoints.len() < 2 {
            continue;
        }
        let mut pts = routed.waypoints.clone();
        if !routed.self_loop {
            let n_pts = pts.len();
            let fp = layout.nodes[r.from];
            let tp = layout.nodes[r.to];
            pts[0] =
                clip_point_to_rect(fp.cx, fp.cy, measured[r.from].w, measured[r.from].h, pts[1]);
            pts[n_pts - 1] = clip_point_to_rect(
                tp.cx,
                tp.cy,
                measured[r.to].w,
                measured[r.to].h,
                pts[n_pts - 2],
            );
        }

        let (d, mid, start_dir, end_dir) = build_smooth_edge_path(&pts);
        cc.raw_path(RawPath {
            d,
            fill: None,
            stroke: Some(palette.edge_stroke),
            stroke_width: 1.7,
            stroke_dash_array: r.dashed.then(|| "5,4".to_string()),
            stroke_linecap: Some("round"),
            stroke_linejoin: Some("round"),
            ..Default::default()
        });
        draw_edge_marker(
            &mut cc,
            r.left_marker,
            pts[0],
            (-start_dir.0, -start_dir.1),
            palette.edge_stroke,
            palette.opaque_bg,
            1.7,
        );
        draw_edge_marker(
            &mut cc,
            r.right_marker,
            *pts.last().unwrap(),
            end_dir,
            palette.edge_stroke,
            palette.opaque_bg,
            1.7,
        );
        if let Some(ref lbl) = r.label {
            let ft = format_and_measure(lbl, &font_family, edge_font_size);
            draw_edge_label(
                &mut cc,
                mid,
                &ft,
                &palette,
                palette.text_color,
                &font_family,
                edge_font_size,
            );
        }
    }

    // 2. Entity cards and multi-column aligned attribute tables.
    for (u, ent) in er.entities.iter().enumerate() {
        let m = &measured[u];
        let pos = layout.nodes[u];
        let eff_style = resolve_style(&ent.classes, &ent.inline_style, &er.class_defs);
        let base_fill = eff_style.fill.unwrap_or(palette.opaque_bg);
        let stroke = eff_style.stroke.unwrap_or(palette.node_stroke);
        let stroke_w = eff_style.stroke_width.unwrap_or(1.6);
        let text_color = eff_style.color.unwrap_or(palette.text_color);

        let x = pos.cx - m.w * 0.5;
        let y = pos.cy - m.h * 0.5;

        if ent.attrs.is_empty() {
            let card_fill = eff_style.fill.unwrap_or(palette.node_fill);
            cc.rect(Rect {
                left: x,
                top: y,
                width: m.w,
                height: m.h,
                rx: Some(5.0),
                ry: Some(5.0),
                fill: Some(card_fill.into()),
                color: Some(stroke),
                stroke_width: Some(stroke_w),
                stroke_dash_array: eff_style.stroke_dasharray,
                ..Default::default()
            });
            draw_multiline_text(
                &mut cc,
                pos.cx,
                pos.cy,
                &m.title_ft,
                &font_family,
                title_font_size,
                text_color,
                "middle",
                true,
            );
            continue;
        }

        let header_fill = mix_color(palette.node_fill, stroke, 0.16);
        let zebra_fill = mix_color(
            base_fill,
            stroke,
            if palette.is_light { 0.05 } else { 0.09 },
        );
        let sep_color = mix_color(stroke, base_fill, 0.55);
        let muted_color = mix_color(text_color, base_fill, 0.32);
        let badge_fill = mix_color(
            base_fill,
            palette.accent,
            if palette.is_light { 0.16 } else { 0.28 },
        );

        // Card base background.
        cc.rect(Rect {
            left: x,
            top: y,
            width: m.w,
            height: m.h,
            rx: Some(5.0),
            ry: Some(5.0),
            fill: Some(base_fill.into()),
            color: None,
            ..Default::default()
        });

        // Header fill band with square bottom edges.
        cc.rect(Rect {
            left: x,
            top: y,
            width: m.w,
            height: header_h,
            rx: Some(5.0),
            ry: Some(5.0),
            fill: Some(header_fill.into()),
            color: None,
            ..Default::default()
        });
        cc.rect(Rect {
            left: x,
            top: y + header_h - 5.0,
            width: m.w,
            height: 5.0,
            fill: Some(header_fill.into()),
            color: None,
            ..Default::default()
        });
        draw_multiline_text(
            &mut cc,
            pos.cx,
            y + header_h * 0.5,
            &m.title_ft,
            &font_family,
            title_font_size,
            text_color,
            "middle",
            true,
        );

        let x_type = x + pad_x;
        let x_name = x_type + m.type_col_w + col_gap;
        let x_key = x_name + m.name_col_w + col_gap;
        let x_comment = x_key
            + if m.key_col_w > 0.0 {
                m.key_col_w + col_gap
            } else {
                0.0
            };

        for (i, attr) in ent.attrs.iter().enumerate() {
            let row_y = y + header_h + i as f32 * row_h;
            let cy = row_y + row_h * 0.5;
            if i % 2 == 1 {
                cc.rect(Rect {
                    left: x,
                    top: row_y,
                    width: m.w,
                    height: row_h,
                    fill: Some(zebra_fill.into()),
                    color: None,
                    ..Default::default()
                });
            }
            cc.line(Line {
                color: Some(if i == 0 { stroke } else { sep_color }),
                stroke_width: if i == 0 { 1.1 } else { 0.7 },
                left: x,
                top: row_y,
                right: x + m.w,
                bottom: row_y,
                ..Default::default()
            });

            let t_ft = format_and_measure(&attr.attr_type, &font_family, attr_font_size);
            draw_multiline_text(
                &mut cc,
                x_type,
                cy,
                &t_ft,
                &font_family,
                attr_font_size,
                muted_color,
                "start",
                false,
            );
            let n_ft = format_and_measure(&attr.name, &font_family, attr_font_size);
            draw_multiline_text(
                &mut cc,
                x_name,
                cy,
                &n_ft,
                &font_family,
                attr_font_size,
                text_color,
                "start",
                false,
            );
            if let Some(ref k) = attr.keys {
                let k_ft = format_and_measure(k, &font_family, key_font_size);
                let bw = k_ft.width + 10.0;
                let bh = 16.0;
                cc.rect(Rect {
                    left: x_key,
                    top: cy - bh * 0.5,
                    width: bw,
                    height: bh,
                    rx: Some(3.0),
                    ry: Some(3.0),
                    fill: Some(badge_fill.into()),
                    color: None,
                    ..Default::default()
                });
                draw_multiline_text(
                    &mut cc,
                    x_key + bw * 0.5,
                    cy,
                    &k_ft,
                    &font_family,
                    key_font_size,
                    text_color,
                    "middle",
                    true,
                );
            }
            if let Some(ref comment) = attr.comment {
                let c_ft = format_and_measure(comment, &font_family, attr_font_size);
                draw_multiline_text(
                    &mut cc,
                    x_comment,
                    cy,
                    &c_ft,
                    &font_family,
                    attr_font_size,
                    muted_color,
                    "start",
                    false,
                );
            }
        }

        // Crisp outer card border drawn last (transparent fill avoids black SVG default).
        cc.rect(Rect {
            left: x,
            top: y,
            width: m.w,
            height: m.h,
            rx: Some(5.0),
            ry: Some(5.0),
            fill: Some(Color::transparent().into()),
            color: Some(stroke),
            stroke_width: Some(stroke_w),
            stroke_dash_array: eff_style.stroke_dasharray,
            ..Default::default()
        });
    }

    let svg = canvas.svg()?;
    Ok((svg, out_w, out_h))
}

#[cfg(test)]
mod tests {
    use super::super::preprocess::preprocess;
    use super::*;

    #[test]
    fn renders_crows_foot_aliases_attributes_and_badges() {
        let src = r#"
erDiagram
    CUSTOMER["Customer"] {
        uuid id PK
        varchar email UK "unique login"
    }
    ORDER {
        uuid id PK
        uuid customer_id FK
    }
    ORDER_ITEM {
        uuid order_id PK,FK
        int qty
    }
    TAG {
        string label
    }
    CUSTOMER ||--o{ ORDER : places
    ORDER one or more to zero or more TAG : tagged
    ORDER }|..|{ ORDER_ITEM : "contains"
"#;
        let prep = preprocess(src).unwrap();
        let chart = MermaidChart::new(src);
        let (svg, w, h) = render(&chart, "light", prep.title.as_deref(), &prep.lines).unwrap();
        assert!(svg.starts_with("<svg") && w > 100.0 && h > 100.0);
        assert!(svg.contains("Customer"));
        assert!(svg.contains("unique login"));
        assert!(svg.contains("PK, FK"));
        assert!(svg.contains("fill=\"none\""));
    }

    #[test]
    fn reports_line_numbered_errors_in_er_diagram() {
        let unclosed = preprocess("erDiagram\n  USER {\n    int id PK")
            .unwrap()
            .lines;
        let err = parse_er_diagram(&unclosed).unwrap_err().to_string();
        assert!(err.contains("line 2"), "{err}");

        let bad_attr = preprocess("erDiagram\n  USER {\n    onlyone\n  }")
            .unwrap()
            .lines;
        let err = parse_er_diagram(&bad_attr).unwrap_err().to_string();
        assert!(err.contains("line 3"), "{err}");
    }
}
