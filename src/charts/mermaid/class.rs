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

//! Mermaid `classDiagram` / `classDiagram-v2` parser and pure-SVG renderer.

use std::collections::HashMap;

use super::super::color::Color;
use super::super::component::{Line, RawPath, Rect};
use super::super::error::{Error, Result};
use super::super::font::measure_text_width_family;
use super::super::theme::get_theme;
use super::super::util::Point;
use super::MermaidChart;
use super::draw::{
    EdgeMarker, build_smooth_edge_path, clip_point_to_rect, draw_cardinality_label,
    draw_edge_label, draw_edge_marker, draw_note_box,
};
use super::layout::{
    LayeredEdgeInput, LayeredGraph, LayeredNodeInput, LayeredSubgraphInput, NoteSide, sugiyama,
};
use super::preprocess::{Direction, SourceLine};
use super::style::{
    MermaidPalette, StyleDirective, StyleProps, mix_color, resolve_style, split_inline_classes,
    try_parse_style_directive, unquote_text,
};
use super::text::{FormattedText, draw_multiline_text, format_and_measure};

#[derive(Clone, Debug)]
struct ClassMember {
    text: String,
    is_method: bool,
    is_abstract: bool,
    is_static: bool,
}

#[derive(Clone, Debug)]
struct ClassNode {
    id: String,
    display_name: Option<String>,
    annotation: Option<String>,
    fields: Vec<ClassMember>,
    methods: Vec<ClassMember>,
    is_note: bool,
    note_text: Option<String>,
    subgraph: Option<usize>,
    attach_to: Option<(usize, NoteSide)>,
    classes: Vec<String>,
    inline_style: StyleProps,
}

#[derive(Clone, Debug)]
struct ClassRelation {
    from: usize,
    to: usize,
    start_marker: EdgeMarker,
    end_marker: EdgeMarker,
    dashed: bool,
    card_left: Option<String>,
    card_right: Option<String>,
    label: Option<String>,
}

#[derive(Clone, Debug)]
struct ClassNamespace {
    title: String,
    parent: Option<usize>,
    inline_style: StyleProps,
}

#[derive(Clone, Copy, Debug)]
enum Scope {
    Namespace { idx: usize, open_line: usize },
    ClassBlock { idx: usize, open_line: usize },
}

#[derive(Debug)]
struct ParsedClassDiagram {
    direction: Direction,
    nodes: Vec<ClassNode>,
    node_map: HashMap<String, usize>,
    relations: Vec<ClassRelation>,
    namespaces: Vec<ClassNamespace>,
    namespace_map: HashMap<String, usize>,
    class_defs: HashMap<String, StyleProps>,
}

/// Converts balanced Mermaid `~T~` generic syntax into `<T>` while leaving
/// leading visibility `~` prefixes untouched.
fn format_generics(raw: &str) -> String {
    let chars: Vec<char> = raw.chars().collect();
    let mut out = String::with_capacity(raw.len());
    let mut depth = 0usize;
    let mut i = 0;

    while i < chars.len() {
        let ch = chars[i];
        if ch == '~' {
            let prev = i.checked_sub(1).map(|p| chars[p]);
            let opens_generic =
                prev.is_some_and(|p| p.is_alphanumeric() || p == '_' || p == '>') && depth == 0;
            let nested_open = depth > 0
                && i + 1 < chars.len()
                && !chars[i + 1].is_whitespace()
                && chars[i + 1] != '~'
                && prev.is_some_and(|p| p.is_alphanumeric() || p == '_');
            if opens_generic || nested_open {
                depth += 1;
                out.push('<');
            } else if depth > 0 {
                depth -= 1;
                out.push('>');
            } else {
                out.push('~');
            }
        } else {
            out.push(ch);
        }
        i += 1;
    }
    out
}

fn parse_member_line(raw: &str) -> ClassMember {
    let mut s = raw.trim().to_string();
    let mut is_abstract = false;
    let mut is_static = false;

    if s.ends_with('*') {
        is_abstract = true;
        s.pop();
        s = s.trim_end().to_string();
    } else if s.ends_with('$') {
        is_static = true;
        s.pop();
        s = s.trim_end().to_string();
    }

    // Handle classifier directly after parameter list, e.g. `+create()$ Order`.
    if let Some(rp) = s.find(')') {
        let after = &s[rp + 1..];
        if let Some(stripped) = after.strip_prefix('*') {
            is_abstract = true;
            s = format!("{}{}", &s[..=rp], stripped);
        } else if let Some(stripped) = after.strip_prefix('$') {
            is_static = true;
            s = format!("{}{}", &s[..=rp], stripped);
        }
    }

    let is_method = s.contains('(') && s.contains(')');
    let text = format_generics(s.trim());
    ClassMember {
        text,
        is_method,
        is_abstract,
        is_static,
    }
}

fn strip_generic_base(token: &str) -> (&str, Option<String>) {
    if let Some(pos) = token.find('~') {
        (&token[..pos], Some(format_generics(token)))
    } else {
        (token, None)
    }
}

impl ParsedClassDiagram {
    fn new() -> Self {
        Self {
            direction: Direction::TopBottom,
            nodes: Vec::new(),
            node_map: HashMap::new(),
            relations: Vec::new(),
            namespaces: Vec::new(),
            namespace_map: HashMap::new(),
            class_defs: HashMap::new(),
        }
    }

    fn ensure_class(
        &mut self,
        raw_tok: &str,
        explicit_display: Option<String>,
        current_ns: Option<usize>,
    ) -> usize {
        let (clean_tok, classes) = split_inline_classes(raw_tok);
        let formatted = format_generics(clean_tok.trim());
        let (base_id, generic_display) = strip_generic_base(clean_tok.trim());

        let key = if self.node_map.contains_key(&formatted) {
            formatted.clone()
        } else if generic_display.is_none() && self.node_map.contains_key(base_id) {
            base_id.to_string()
        } else {
            formatted.clone()
        };

        let idx = if let Some(&existing) = self.node_map.get(&key) {
            existing
        } else if let Some(&base_existing) = self.node_map.get(base_id)
            && generic_display.is_some()
        {
            base_existing
        } else {
            let new_idx = self.nodes.len();
            self.node_map.insert(key.clone(), new_idx);
            if generic_display.is_some() && !self.node_map.contains_key(base_id) {
                self.node_map.insert(base_id.to_string(), new_idx);
            }
            self.nodes.push(ClassNode {
                id: key,
                display_name: generic_display.or_else(|| Some(clean_tok.trim().to_string())),
                annotation: None,
                fields: Vec::new(),
                methods: Vec::new(),
                is_note: false,
                note_text: None,
                subgraph: current_ns,
                attach_to: None,
                classes: Vec::new(),
                inline_style: StyleProps::default(),
            });
            new_idx
        };

        let node = &mut self.nodes[idx];
        if let Some(d) = explicit_display {
            node.display_name = Some(d);
        }
        for cls in classes {
            if !node.classes.contains(&cls) {
                node.classes.push(cls);
            }
        }
        if node.subgraph.is_none() && current_ns.is_some() {
            node.subgraph = current_ns;
        }
        idx
    }
}

fn extract_leading_quoted(s: &str) -> (Option<String>, &str) {
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
                let card = after_q[..i].replace("\\\"", "\"");
                return (Some(card), after_q[i + 1..].trim());
            }
            i += 1;
        }
    }
    (None, t)
}

fn extract_trailing_quoted(s: &str) -> (&str, Option<String>) {
    let t = s.trim_end();
    if let Some(before_end) = t.strip_suffix('"')
        && let Some(open) = before_end.rfind('"')
        && open > 0
        && before_end[..open].ends_with(char::is_whitespace)
    {
        let card = before_end[open + 1..].replace("\\\"", "\"");
        return (before_end[..open].trim(), Some(card));
    }
    (t, None)
}

fn parse_left_rel_marker(raw: &str) -> (&str, EdgeMarker) {
    let s = raw.trim_end();
    if let Some(p) = s.strip_suffix("<|") {
        (p.trim_end(), EdgeMarker::HollowTriangle)
    } else if let Some(p) = s.strip_suffix('*') {
        (p.trim_end(), EdgeMarker::FilledDiamond)
    } else if let Some(p) = s.strip_suffix("()") {
        (p.trim_end(), EdgeMarker::Lollipop)
    } else if let Some(p) = s.strip_suffix('<') {
        (p.trim_end(), EdgeMarker::OpenArrow)
    } else if let Some(p) = s.strip_suffix('o')
        && (p.is_empty() || p.ends_with(char::is_whitespace))
    {
        (p.trim_end(), EdgeMarker::HollowDiamond)
    } else {
        (s, EdgeMarker::None)
    }
}

fn parse_right_rel_marker(raw: &str) -> (EdgeMarker, &str) {
    let s = raw.trim_start();
    if let Some(r) = s.strip_prefix("|>") {
        (EdgeMarker::HollowTriangle, r.trim_start())
    } else if let Some(r) = s.strip_prefix('*') {
        (EdgeMarker::FilledDiamond, r.trim_start())
    } else if let Some(r) = s.strip_prefix("()") {
        (EdgeMarker::Lollipop, r.trim_start())
    } else if let Some(r) = s.strip_prefix('>') {
        (EdgeMarker::OpenArrow, r.trim_start())
    } else if let Some(r) = s.strip_prefix('o')
        && (r.is_empty() || r.starts_with(char::is_whitespace))
    {
        (EdgeMarker::HollowDiamond, r.trim_start())
    } else {
        (EdgeMarker::None, s)
    }
}

fn find_rel_stem(line: &str) -> Option<(usize, bool)> {
    let bytes = line.as_bytes();
    let mut in_quotes = false;
    let mut bracket = 0i32;
    let mut tilde = 0usize;
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
        if b == b'[' {
            bracket += 1;
        } else if b == b']' {
            bracket = (bracket - 1).max(0);
        } else if b == b'~' {
            if tilde == 0 && i > 0 {
                tilde += 1;
            } else if tilde > 0 {
                tilde = 0;
            }
        } else if bracket == 0 && tilde == 0 {
            if line[i..].starts_with("--") && !line[i..].starts_with("---") {
                return Some((i, false));
            }
            if line[i..].starts_with("..") && !line[i..].starts_with("...") {
                return Some((i, true));
            }
        }
        i += 1;
    }
    None
}

fn split_label_outside_quotes(s: &str) -> (&str, Option<String>) {
    let bytes = s.as_bytes();
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
        if b == b':' && !s[i..].starts_with(":::") {
            let lbl = unquote_text(&s[i + 1..]);
            return (
                s[..i].trim(),
                (!lbl.is_empty()).then_some(format_generics(&lbl)),
            );
        }
        i += 1;
    }
    (s.trim(), None)
}

#[derive(Debug)]
struct ParsedRel<'a> {
    left_cls: &'a str,
    right_cls: &'a str,
    start_marker: EdgeMarker,
    end_marker: EdgeMarker,
    dashed: bool,
    card_left: Option<String>,
    card_right: Option<String>,
    label: Option<String>,
}

fn try_parse_relation(trimmed: &str) -> Option<ParsedRel<'_>> {
    let (main_part, label) = split_label_outside_quotes(trimmed);
    let (stem_pos, dashed) = find_rel_stem(main_part)?;
    let (lhs_after_marker, start_marker) = parse_left_rel_marker(&main_part[..stem_pos]);
    let (end_marker, rhs_after_marker) = parse_right_rel_marker(&main_part[stem_pos + 2..]);
    let (left_cls_raw, card_left) = extract_trailing_quoted(lhs_after_marker);
    let (card_right, right_cls_raw) = extract_leading_quoted(rhs_after_marker);

    let left_cls = left_cls_raw.trim();
    let right_cls = right_cls_raw.trim();
    if left_cls.is_empty() || right_cls.is_empty() {
        return None;
    }
    // Ensure endpoints look like class identifiers (with optional `~...~` and `:::cls`) rather than arbitrary sentences.
    if left_cls.contains(char::is_whitespace) && !left_cls.contains('~') {
        return None;
    }
    if right_cls.contains(char::is_whitespace) && !right_cls.contains('~') {
        return None;
    }

    Some(ParsedRel {
        left_cls,
        right_cls,
        start_marker,
        end_marker,
        dashed,
        card_left,
        card_right,
        label,
    })
}

fn parse_class_decl_header(rest: &str, line_no: usize) -> Result<(&str, Option<String>)> {
    let r = rest.trim();
    if r.is_empty() {
        return Err(Error::Mermaid {
            line: Some(line_no),
            message: "expected class name after `class`".to_string(),
        });
    }
    if let Some(lb) = r.find('[')
        && let Some(rb) = r.rfind(']')
        && rb > lb
    {
        let id = r[..lb].trim();
        let display = unquote_text(&r[lb + 1..rb]);
        return Ok((id, Some(display)));
    }
    Ok((r, None))
}

fn parse_class_diagram(lines: &[SourceLine]) -> Result<ParsedClassDiagram> {
    let mut cd = ParsedClassDiagram::new();
    let mut scopes: Vec<Scope> = Vec::new();
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
        if trimmed.starts_with("accTitle")
            || trimmed.starts_with("link ")
            || trimmed.starts_with("click ")
            || trimmed.starts_with("callback ")
        {
            continue;
        }
        if trimmed.starts_with("accDescr") {
            if trimmed.contains('{') && !trimmed.contains('}') {
                in_acc_descr_block = true;
            }
            continue;
        }

        if trimmed == "}" {
            if scopes.pop().is_none() {
                return Err(Error::Mermaid {
                    line: Some(sl.line_no),
                    message: "unexpected `}` with no matching `{`".to_string(),
                });
            }
            continue;
        }

        // Inside `class Foo { ... }` body: parse annotation or class member lines.
        if let Some(Scope::ClassBlock { idx, .. }) = scopes.last().copied() {
            if trimmed.starts_with("<<") && trimmed.ends_with(">>") && trimmed.len() > 4 {
                cd.nodes[idx].annotation = Some(trimmed.to_string());
            } else {
                let member = parse_member_line(trimmed);
                if member.is_method {
                    cd.nodes[idx].methods.push(member);
                } else {
                    cd.nodes[idx].fields.push(member);
                }
            }
            continue;
        }

        let current_ns = scopes.iter().rev().find_map(|s| match s {
            Scope::Namespace { idx, .. } => Some(*idx),
            Scope::ClassBlock { .. } => None,
        });

        if let Some(rest) = trimmed.strip_prefix("direction")
            && rest.starts_with(char::is_whitespace)
        {
            let dir_tok = rest.trim();
            let dir = Direction::from_token(dir_tok).ok_or_else(|| Error::Mermaid {
                line: Some(sl.line_no),
                message: format!("unknown direction `{dir_tok}`"),
            })?;
            if scopes.is_empty() {
                cd.direction = dir;
            }
            continue;
        }

        if let Some(rest) = trimmed.strip_prefix("class")
            && rest.starts_with(char::is_whitespace)
            && {
                let r = rest.trim();
                r.ends_with('{')
                    || r.contains('[')
                    || r.contains('~')
                    || !r.contains(char::is_whitespace)
            }
        {
            let rest = rest.trim();
            if let Some(header) = rest.strip_suffix('{') {
                let (id_raw, display_opt) = parse_class_decl_header(header, sl.line_no)?;
                let idx = cd.ensure_class(id_raw, display_opt, current_ns);
                scopes.push(Scope::ClassBlock {
                    idx,
                    open_line: sl.line_no,
                });
            } else {
                let (id_raw, display_opt) = parse_class_decl_header(rest, sl.line_no)?;
                cd.ensure_class(id_raw, display_opt, current_ns);
            }
            continue;
        }

        if let Some(directive) = try_parse_style_directive(trimmed, sl.line_no)? {
            match directive {
                StyleDirective::ClassDef(name, props) => {
                    cd.class_defs.entry(name).or_default().merge_from(&props);
                }
                StyleDirective::ClassAssign { ids, classes } => {
                    for id in ids {
                        let idx = cd.ensure_class(&id, None, current_ns);
                        for cls in &classes {
                            if !cd.nodes[idx].classes.contains(cls) {
                                cd.nodes[idx].classes.push(cls.clone());
                            }
                        }
                    }
                }
                StyleDirective::NodeStyle { ids, props } => {
                    for id in ids {
                        if let Some(&ns_idx) = cd.namespace_map.get(&id) {
                            cd.namespaces[ns_idx].inline_style.merge_from(&props);
                        } else {
                            let idx = cd.ensure_class(&id, None, current_ns);
                            cd.nodes[idx].inline_style.merge_from(&props);
                        }
                    }
                }
            }
            continue;
        }

        if let Some(rest) = trimmed.strip_prefix("namespace")
            && rest.starts_with(char::is_whitespace)
        {
            let rest = rest.trim();
            let Some(name_raw) = rest.strip_suffix('{').map(str::trim) else {
                return Err(Error::Mermaid {
                    line: Some(sl.line_no),
                    message: "expected `{` after `namespace <name>`".to_string(),
                });
            };
            if name_raw.is_empty() {
                return Err(Error::Mermaid {
                    line: Some(sl.line_no),
                    message: "empty namespace name".to_string(),
                });
            }
            let ns_title = unquote_text(name_raw);
            let ns_idx = cd.namespaces.len();
            cd.namespace_map.insert(ns_title.clone(), ns_idx);
            cd.namespaces.push(ClassNamespace {
                title: ns_title,
                parent: current_ns,
                inline_style: StyleProps::default(),
            });
            scopes.push(Scope::Namespace {
                idx: ns_idx,
                open_line: sl.line_no,
            });
            continue;
        }

        if let Some(rest) = trimmed.strip_prefix("note")
            && rest.starts_with(char::is_whitespace)
        {
            let r = rest.trim();
            if let Some(after_for) = r
                .strip_prefix("for")
                .filter(|x| x.starts_with(char::is_whitespace))
            {
                let after_for = after_for.trim();
                let Some((target_raw, note_body)) = after_for.split_once(char::is_whitespace)
                else {
                    return Err(Error::Mermaid {
                        line: Some(sl.line_no),
                        message: "expected `note for <ClassName> \"text\"`".to_string(),
                    });
                };
                let target_idx = cd.ensure_class(target_raw.trim(), None, current_ns);
                let note_ns = cd.nodes[target_idx].subgraph;
                let note_idx = cd.nodes.len();
                cd.nodes.push(ClassNode {
                    id: format!("__note_{note_idx}"),
                    display_name: None,
                    annotation: None,
                    fields: Vec::new(),
                    methods: Vec::new(),
                    is_note: true,
                    note_text: Some(unquote_text(note_body.trim())),
                    subgraph: note_ns,
                    attach_to: Some((target_idx, NoteSide::Right)),
                    classes: Vec::new(),
                    inline_style: StyleProps::default(),
                });
            } else {
                let note_idx = cd.nodes.len();
                cd.nodes.push(ClassNode {
                    id: format!("__note_{note_idx}"),
                    display_name: None,
                    annotation: None,
                    fields: Vec::new(),
                    methods: Vec::new(),
                    is_note: true,
                    note_text: Some(unquote_text(r)),
                    subgraph: current_ns,
                    attach_to: None,
                    classes: Vec::new(),
                    inline_style: StyleProps::default(),
                });
            }
            continue;
        }

        if trimmed.starts_with("<<")
            && let Some(end) = trimmed.find(">>")
        {
            let ann = trimmed[..=end + 1].to_string();
            let target = trimmed[end + 2..].trim();
            if target.is_empty() {
                return Err(Error::Mermaid {
                    line: Some(sl.line_no),
                    message: "expected class name after annotation".to_string(),
                });
            }
            let idx = cd.ensure_class(target, None, current_ns);
            cd.nodes[idx].annotation = Some(ann);
            continue;
        }

        if let Some(rel) = try_parse_relation(trimmed) {
            let from = cd.ensure_class(rel.left_cls, None, current_ns);
            let to = cd.ensure_class(rel.right_cls, None, current_ns);
            cd.relations.push(ClassRelation {
                from,
                to,
                start_marker: rel.start_marker,
                end_marker: rel.end_marker,
                dashed: rel.dashed,
                card_left: rel.card_left,
                card_right: rel.card_right,
                label: rel.label,
            });
            continue;
        }

        if let Some((cls_raw, member_raw)) = trimmed.split_once(':')
            && !trimmed.contains(":::")
        {
            let cls_tok = cls_raw.trim();
            let mem_raw = member_raw.trim();
            if cls_tok.is_empty() || cls_tok.contains(char::is_whitespace) {
                return Err(Error::Mermaid {
                    line: Some(sl.line_no),
                    message: format!("invalid class member syntax `{trimmed}`"),
                });
            }
            let idx = cd.ensure_class(cls_tok, None, current_ns);
            if mem_raw.starts_with("<<") && mem_raw.ends_with(">>") && mem_raw.len() > 4 {
                cd.nodes[idx].annotation = Some(mem_raw.to_string());
            } else if !mem_raw.is_empty() {
                let m = parse_member_line(mem_raw);
                if m.is_method {
                    cd.nodes[idx].methods.push(m);
                } else {
                    cd.nodes[idx].fields.push(m);
                }
            }
            continue;
        }

        return Err(Error::Mermaid {
            line: Some(sl.line_no),
            message: format!("invalid classDiagram statement `{trimmed}`"),
        });
    }

    if let Some(scope) = scopes.last() {
        let (line, kind) = match *scope {
            Scope::Namespace { open_line, .. } => (open_line, "namespace"),
            Scope::ClassBlock { open_line, .. } => (open_line, "class"),
        };
        return Err(Error::Mermaid {
            line: Some(line),
            message: format!("unclosed `{kind}` block (missing `}}`)"),
        });
    }

    Ok(cd)
}

pub(crate) fn render(
    chart: &MermaidChart,
    theme_name: &str,
    fm_title: Option<&str>,
    lines: &[SourceLine],
) -> Result<(String, f32, f32)> {
    let mut cd = parse_class_diagram(lines)?;
    if cd.nodes.is_empty() && cd.namespaces.is_empty() {
        return Err(Error::Mermaid {
            line: lines.first().map(|l| l.line_no),
            message: "classDiagram has no classes".to_string(),
        });
    }

    for ns_idx in 0..cd.namespaces.len() {
        let has_member = cd.nodes.iter().any(|n| n.subgraph == Some(ns_idx))
            || cd.namespaces.iter().any(|s| s.parent == Some(ns_idx));
        if !has_member {
            cd.nodes.push(ClassNode {
                id: format!("__ns_anchor_{ns_idx}"),
                display_name: None,
                annotation: None,
                fields: Vec::new(),
                methods: Vec::new(),
                is_note: false,
                note_text: None,
                subgraph: Some(ns_idx),
                attach_to: None,
                classes: Vec::new(),
                inline_style: StyleProps::default(),
            });
        }
    }

    let theme = get_theme(theme_name);
    let bg_override = chart
        .has_custom_background()
        .then_some(chart.base.background_color);
    let palette = MermaidPalette::from_theme(&theme, bg_override);
    let font_family = chart.resolved_font_family(&theme);
    let header_font_size = 13.5_f32;
    let ann_font_size = 11.5_f32;
    let member_font_size = 12.0_f32;
    let edge_font_size = 12.0_f32;
    let card_font_size = 11.0_f32;
    let note_font_size = 12.0_f32;
    let ns_font_size = 13.0_f32;
    let row_h = 20.0_f32;
    let pad_x = 14.0_f32;

    struct MeasuredClass {
        is_anchor: bool,
        w: f32,
        h: f32,
        header_h: f32,
        fields_h: f32,
        ann_ft: Option<FormattedText>,
        name_ft: FormattedText,
        note_ft: Option<FormattedText>,
    }

    let mut measured = Vec::with_capacity(cd.nodes.len());
    let mut layout_nodes = Vec::with_capacity(cd.nodes.len());

    for n in &cd.nodes {
        let is_anchor = n.id.starts_with("__ns_anchor_");
        if is_anchor {
            layout_nodes.push(LayeredNodeInput {
                width: 40.0,
                height: 20.0,
                subgraph: n.subgraph,
                attach_to: None,
            });
            measured.push(MeasuredClass {
                is_anchor: true,
                w: 40.0,
                h: 20.0,
                header_h: 20.0,
                fields_h: 0.0,
                ann_ft: None,
                name_ft: format_and_measure("", &font_family, header_font_size),
                note_ft: None,
            });
            continue;
        }
        if n.is_note {
            let nft = format_and_measure(
                n.note_text.as_deref().unwrap_or(""),
                &font_family,
                note_font_size,
            );
            let w = (nft.width + 26.0).max(68.0);
            let h = (nft.height + 16.0).max(34.0);
            layout_nodes.push(LayeredNodeInput {
                width: w,
                height: h,
                subgraph: n.subgraph,
                attach_to: n.attach_to,
            });
            measured.push(MeasuredClass {
                is_anchor: false,
                w,
                h,
                header_h: 0.0,
                fields_h: 0.0,
                ann_ft: None,
                name_ft: format_and_measure("", &font_family, header_font_size),
                note_ft: Some(nft),
            });
            continue;
        }

        let display = n.display_name.as_deref().unwrap_or(&n.id);
        let name_ft = format_and_measure(display, &font_family, header_font_size);
        let ann_ft = n
            .annotation
            .as_deref()
            .map(|a| format_and_measure(a, &font_family, ann_font_size));

        let mut max_w = name_ft.width + pad_x * 2.0;
        if let Some(ref aft) = ann_ft {
            max_w = max_w.max(aft.width + pad_x * 2.0);
        }
        for m in n.fields.iter().chain(n.methods.iter()) {
            let mw = measure_text_width_family(&font_family, member_font_size, &m.text)
                .map(|b| b.width())
                .unwrap_or_else(|_| m.text.chars().count() as f32 * member_font_size * 0.6);
            max_w = max_w.max(mw + pad_x * 2.0);
        }
        let w = max_w.max(108.0);

        let header_h = if let Some(ref aft) = ann_ft {
            aft.height + name_ft.height + 16.0
        } else {
            name_ft.height + 16.0
        };
        let fields_h = if n.fields.is_empty() {
            0.0
        } else {
            n.fields.len() as f32 * row_h + 10.0
        };
        let methods_h = if n.methods.is_empty() {
            0.0
        } else {
            n.methods.len() as f32 * row_h + 10.0
        };
        let h = header_h + fields_h + methods_h;

        layout_nodes.push(LayeredNodeInput {
            width: w,
            height: h,
            subgraph: n.subgraph,
            attach_to: None,
        });
        measured.push(MeasuredClass {
            is_anchor: false,
            w,
            h,
            header_h,
            fields_h,
            ann_ft,
            name_ft,
            note_ft: None,
        });
    }

    let mut ns_labels = Vec::with_capacity(cd.namespaces.len());
    let mut layout_subgraphs = Vec::with_capacity(cd.namespaces.len());
    for ns in &cd.namespaces {
        let ft = format_and_measure(&ns.title, &font_family, ns_font_size);
        layout_subgraphs.push(LayeredSubgraphInput {
            parent: ns.parent,
            title_width: ft.width,
            title_height: ft.height,
            concurrent_region: false,
        });
        ns_labels.push(ft);
    }

    let layout_edges: Vec<LayeredEdgeInput> = cd
        .relations
        .iter()
        .map(|r| LayeredEdgeInput {
            from: r.from,
            to: r.to,
            has_label: r.label.is_some() || r.card_left.is_some() || r.card_right.is_some(),
            min_len: 1,
        })
        .collect();

    let layout = sugiyama::layout(&LayeredGraph {
        direction: cd.direction,
        rank_sep_extra: 32.0,
        node_sep_extra: 0.0,
        nodes: layout_nodes,
        edges: layout_edges,
        subgraphs: layout_subgraphs,
    });

    let title_text = if !chart.base.title.text.is_empty() {
        Some(chart.base.title.text.clone())
    } else {
        fm_title.map(str::to_string)
    };
    let header_h = super::compute_header_height(chart, title_text.as_deref(), &theme);
    let content_w = (layout.width + chart.base.margin.left + chart.base.margin.right).max(1.0);
    let content_h =
        (layout.height + header_h + chart.base.margin.top + chart.base.margin.bottom).max(1.0);

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

    // 1. Namespaces.
    for (ns_idx, ns) in cd.namespaces.iter().enumerate() {
        let b = layout.subgraphs[ns_idx];
        if !b.visible {
            continue;
        }
        let fill = ns
            .inline_style
            .fill
            .unwrap_or_else(|| palette.subgraph_fill(0));
        let stroke = ns.inline_style.stroke.unwrap_or(palette.subgraph_stroke);
        let stroke_w = ns.inline_style.stroke_width.unwrap_or(1.2);
        let text_color = ns.inline_style.color.unwrap_or(palette.text_color);

        cc.rect(Rect {
            left: b.x,
            top: b.y,
            width: b.width,
            height: b.height,
            rx: Some(6.0),
            ry: Some(6.0),
            fill: Some(fill.into()),
            color: Some(stroke),
            stroke_width: Some(stroke_w),
            stroke_dash_array: ns.inline_style.stroke_dasharray.clone(),
            ..Default::default()
        });
        let title_ft = &ns_labels[ns_idx];
        draw_multiline_text(
            &mut cc,
            b.x + b.width * 0.5,
            b.y + 12.0 + title_ft.height * 0.5,
            title_ft,
            &font_family,
            ns_font_size,
            text_color,
            "middle",
            true,
        );
    }

    // 2. Dashed note attachment connectors.
    for (u, n) in cd.nodes.iter().enumerate() {
        if n.is_note
            && let Some((target, _)) = n.attach_to
        {
            let np = layout.nodes[u];
            let tp = layout.nodes[target];
            let p0 = clip_point_to_rect(
                np.cx,
                np.cy,
                measured[u].w,
                measured[u].h,
                Point::from((tp.cx, tp.cy)),
            );
            let p1 = clip_point_to_rect(
                tp.cx,
                tp.cy,
                measured[target].w,
                measured[target].h,
                Point::from((np.cx, np.cy)),
            );
            cc.line(Line {
                color: Some(palette.note_stroke),
                stroke_width: 1.2,
                left: p0.x,
                top: p0.y,
                right: p1.x,
                bottom: p1.y,
                stroke_dash_array: Some("4,3".to_string()),
            });
        }
    }

    // 3. UML relationships (pass 1: paths & markers; pass 2: cardinality & edge labels).
    let mut deferred_cards: Vec<(Point, (f32, f32), &str)> = Vec::new();
    let mut deferred_labels: Vec<(Point, FormattedText)> = Vec::new();
    for (rel_idx, r) in cd.relations.iter().enumerate() {
        let routed = &layout.edges[rel_idx];
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
            r.start_marker,
            pts[0],
            (-start_dir.0, -start_dir.1),
            palette.edge_stroke,
            palette.opaque_bg,
            1.7,
        );
        draw_edge_marker(
            &mut cc,
            r.end_marker,
            *pts.last().unwrap(),
            end_dir,
            palette.edge_stroke,
            palette.opaque_bg,
            1.7,
        );
        if let Some(ref cl) = r.card_left {
            deferred_cards.push((pts[0], (-start_dir.0, -start_dir.1), cl.as_str()));
        }
        if let Some(ref cr) = r.card_right {
            deferred_cards.push((*pts.last().unwrap(), end_dir, cr.as_str()));
        }
        if let Some(ref lbl) = r.label {
            let ft = format_and_measure(lbl, &font_family, edge_font_size);
            deferred_labels.push((mid, ft));
        }
    }
    for (pt, dir, card) in deferred_cards {
        draw_cardinality_label(
            &mut cc,
            pt,
            dir,
            card,
            &palette,
            palette.text_color,
            &font_family,
            card_font_size,
        );
    }
    for (mid, ft) in deferred_labels {
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

    // 4. Class boxes and Note cards.
    for (u, n) in cd.nodes.iter().enumerate() {
        let m = &measured[u];
        if m.is_anchor {
            continue;
        }
        let pos = layout.nodes[u];
        if let Some(ref nft) = m.note_ft {
            draw_note_box(
                &mut cc,
                pos.cx,
                pos.cy,
                m.w,
                m.h,
                nft,
                &palette,
                &font_family,
                note_font_size,
            );
            continue;
        }

        let eff_style = resolve_style(&n.classes, &n.inline_style, &cd.class_defs);
        let fill = eff_style.fill.unwrap_or(palette.node_fill);
        let stroke = eff_style.stroke.unwrap_or(palette.node_stroke);
        let stroke_w = eff_style.stroke_width.unwrap_or(1.6);
        let text_color = eff_style.color.unwrap_or(palette.text_color);
        let header_fill = mix_color(fill, stroke, 0.14);

        let x = pos.cx - m.w * 0.5;
        let y = pos.cy - m.h * 0.5;

        // Card body fill.
        cc.rect(Rect {
            left: x,
            top: y,
            width: m.w,
            height: m.h,
            rx: Some(4.0),
            ry: Some(4.0),
            fill: Some(fill.into()),
            color: None,
            ..Default::default()
        });

        // Header compartment shading.
        cc.rect(Rect {
            left: x,
            top: y,
            width: m.w,
            height: m.header_h,
            rx: Some(4.0),
            ry: Some(4.0),
            fill: Some(header_fill.into()),
            color: None,
            ..Default::default()
        });
        if !n.fields.is_empty() || !n.methods.is_empty() {
            cc.rect(Rect {
                left: x,
                top: y + m.header_h - 4.0,
                width: m.w,
                height: 4.0,
                fill: Some(header_fill.into()),
                color: None,
                ..Default::default()
            });
        }

        // Annotation + Class name.
        if let Some(ref aft) = m.ann_ft {
            let ann_color = mix_color(text_color, palette.accent, 0.35);
            draw_multiline_text(
                &mut cc,
                pos.cx,
                y + 7.0 + aft.height * 0.5,
                aft,
                &font_family,
                ann_font_size,
                ann_color,
                "middle",
                false,
            );
            draw_multiline_text(
                &mut cc,
                pos.cx,
                y + 9.0 + aft.height + m.name_ft.height * 0.5,
                &m.name_ft,
                &font_family,
                header_font_size,
                text_color,
                "middle",
                true,
            );
        } else {
            draw_multiline_text(
                &mut cc,
                pos.cx,
                y + m.header_h * 0.5,
                &m.name_ft,
                &font_family,
                header_font_size,
                text_color,
                "middle",
                true,
            );
        }

        let draw_members =
            |cc: &mut super::super::canvas::Canvas, members: &[ClassMember], top_y: f32| {
                for (i, member) in members.iter().enumerate() {
                    let cy = top_y + 5.0 + (i as f32 + 0.5) * row_h;
                    let ft = format_and_measure(&member.text, &font_family, member_font_size);
                    super::text::draw_multiline_text_styled(
                        cc,
                        x + pad_x,
                        cy,
                        &ft,
                        &font_family,
                        member_font_size,
                        text_color,
                        "start",
                        false,
                        member.is_abstract,
                    );
                    if member.is_static {
                        let measured_w = ft
                            .lines
                            .first()
                            .and_then(|l| {
                                super::super::font::measure_text_width_family(
                                    &font_family,
                                    member_font_size,
                                    &l.text,
                                )
                                .ok()
                                .map(|b| b.width())
                            })
                            .unwrap_or(ft.width);
                        if measured_w > 0.0 {
                            let uy = cy + member_font_size * 0.48;
                            cc.line(Line {
                                color: Some(text_color),
                                stroke_width: 1.0,
                                left: x + pad_x,
                                top: uy,
                                right: x + pad_x + measured_w,
                                bottom: uy,
                                ..Default::default()
                            });
                        }
                    }
                }
            };

        let mut cursor_y = y + m.header_h;
        if !n.fields.is_empty() {
            cc.line(Line {
                color: Some(stroke),
                stroke_width: 1.0,
                left: x,
                top: cursor_y,
                right: x + m.w,
                bottom: cursor_y,
                ..Default::default()
            });
            draw_members(&mut cc, &n.fields, cursor_y);
            cursor_y += m.fields_h;
        }
        if !n.methods.is_empty() {
            cc.line(Line {
                color: Some(stroke),
                stroke_width: 1.0,
                left: x,
                top: cursor_y,
                right: x + m.w,
                bottom: cursor_y,
                ..Default::default()
            });
            draw_members(&mut cc, &n.methods, cursor_y);
        }

        // Crisp outer card border drawn last (transparent fill avoids black SVG default).
        cc.rect(Rect {
            left: x,
            top: y,
            width: m.w,
            height: m.h,
            rx: Some(4.0),
            ry: Some(4.0),
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
    fn converts_generics_without_touching_package_visibility() {
        assert_eq!("List<Order>", format_generics("List~Order~"));
        assert_eq!(
            "Map<String, List<Item>>",
            format_generics("Map~String, List~Item~~")
        );
        assert_eq!("~int pkgCount", format_generics("~int pkgCount"));
    }

    #[test]
    fn renders_all_uml_relations_compartments_generics_namespace_and_notes() {
        let src = r#"
classDiagram
    namespace Commerce {
        class Payable {
            <<interface>>
            +pay(amount Decimal) bool*
        }
        class Order {
            +List~OrderItem~ items
            -OrderStatus status
            +create()$ Order
            +total() Decimal
        }
        class OrderItem {
            +String sku
        }
    }
    class PaymentService {
        <<service>>
        +pay(amount Decimal) bool
    }
    Payable <|.. PaymentService : implements
    Order "1" *-- "1..*" OrderItem : contains
    Order ..> PaymentService : uses
    Order o-- Customer
    Order ()-- Auditable
    Service -- Repo
    A .. B
    note for Order "Aggregate root"
"#;
        let prep = preprocess(src).unwrap();
        let chart = MermaidChart::new(src);
        let (svg, w, h) = render(&chart, "light", prep.title.as_deref(), &prep.lines).unwrap();
        assert!(svg.starts_with("<svg") && w > 100.0 && h > 100.0);
        assert!(svg.contains("Commerce"));
        assert!(svg.contains("List&lt;OrderItem&gt;"));
        assert!(svg.contains("&lt;&lt;interface&gt;&gt;"));
        assert!(svg.contains("+create() Order"));
        assert!(svg.contains("Aggregate root"));
        assert!(svg.contains("fill=\"none\""));
    }

    #[test]
    fn reports_line_numbered_errors_in_class_diagram() {
        let lines = preprocess("classDiagram\n  class Foo {\n    +int x")
            .unwrap()
            .lines;
        let err = parse_class_diagram(&lines).unwrap_err().to_string();
        assert!(err.contains("line 2"), "{err}");
    }
}
