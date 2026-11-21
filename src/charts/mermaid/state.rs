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

//! Mermaid `stateDiagram` / `stateDiagram-v2` parser and pure-SVG renderer.

use std::collections::{HashMap, HashSet};

use super::super::color::Color;
use super::super::component::{Circle, Line, Rect};
use super::super::error::{Error, Result};
use super::super::theme::get_theme;
use super::super::util::Point;
use super::MermaidChart;
use super::draw::{
    EdgeMarker, NodeShape, build_smooth_edge_path, clip_point_to_rect, clip_point_to_shape,
    draw_edge_label, draw_edge_marker, draw_node, draw_note_box,
};
use super::layout::{
    LayeredEdgeInput, LayeredGraph, LayeredNodeInput, LayeredSubgraphInput, NoteSide, sugiyama,
};
use super::preprocess::{Direction, SourceLine};
use super::style::{
    MermaidPalette, StyleDirective, StyleProps, resolve_style, split_inline_classes,
    try_parse_style_directive, unquote_text,
};
use super::text::{FormattedText, draw_multiline_text, format_and_measure};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum StateKind {
    Regular,
    Start,
    End,
    Choice,
    Fork,
    Join,
    Note,
}

#[derive(Clone, Debug)]
struct StateNode {
    id: String,
    label: Option<String>,
    descriptions: Vec<String>,
    kind: StateKind,
    subgraph: Option<usize>,
    attach_to: Option<(usize, NoteSide)>,
    classes: Vec<String>,
    inline_style: StyleProps,
}

#[derive(Clone, Copy, Debug)]
enum StateEndpoint {
    Node(usize),
    Subgraph(usize),
}

#[derive(Clone, Debug)]
struct StateEdge {
    from: StateEndpoint,
    to: StateEndpoint,
    label: Option<String>,
    dashed: bool,
}

#[derive(Clone, Debug)]
struct StateSubgraph {
    title: String,
    parent: Option<usize>,
    concurrent_region: bool,
    region_index: usize,
    inline_style: StyleProps,
}

#[derive(Clone, Copy, Debug)]
struct CompositeFrame {
    composite_sg: usize,
    active_sg: usize,
    has_concurrent_regions: bool,
    region_count: usize,
    open_line: usize,
}

#[derive(Debug)]
struct ParsedStateDiagram {
    direction: Direction,
    nodes: Vec<StateNode>,
    node_map: HashMap<String, usize>,
    edges: Vec<StateEdge>,
    subgraphs: Vec<StateSubgraph>,
    subgraph_map: HashMap<String, usize>,
    class_defs: HashMap<String, StyleProps>,
    region_starts: HashMap<Option<usize>, usize>,
    region_ends: HashMap<Option<usize>, usize>,
}

impl ParsedStateDiagram {
    fn new() -> Self {
        Self {
            direction: Direction::TopBottom,
            nodes: Vec::new(),
            node_map: HashMap::new(),
            edges: Vec::new(),
            subgraphs: Vec::new(),
            subgraph_map: HashMap::new(),
            class_defs: HashMap::new(),
            region_starts: HashMap::new(),
            region_ends: HashMap::new(),
        }
    }

    fn ensure_state(
        &mut self,
        raw_id: &str,
        label: Option<String>,
        kind: Option<StateKind>,
        current_sg: Option<usize>,
    ) -> usize {
        let (clean_id, classes) = split_inline_classes(raw_id);
        let idx = if let Some(&existing) = self.node_map.get(clean_id) {
            existing
        } else {
            let new_idx = self.nodes.len();
            self.node_map.insert(clean_id.to_string(), new_idx);
            self.nodes.push(StateNode {
                id: clean_id.to_string(),
                label: None,
                descriptions: Vec::new(),
                kind: StateKind::Regular,
                subgraph: current_sg,
                attach_to: None,
                classes: Vec::new(),
                inline_style: StyleProps::default(),
            });
            new_idx
        };

        let n = &mut self.nodes[idx];
        if let Some(l) = label {
            n.label = Some(l);
        }
        if let Some(k) = kind {
            n.kind = k;
        }
        for cls in classes {
            if !n.classes.contains(&cls) {
                n.classes.push(cls);
            }
        }
        if n.subgraph.is_none() && current_sg.is_some() {
            n.subgraph = current_sg;
        }
        idx
    }

    fn ensure_pseudo_star(&mut self, is_start: bool, current_sg: Option<usize>) -> usize {
        let map = if is_start {
            &mut self.region_starts
        } else {
            &mut self.region_ends
        };
        if let Some(&idx) = map.get(&current_sg) {
            return idx;
        }
        let idx = self.nodes.len();
        let tag = match current_sg {
            Some(sg) => format!("sg_{sg}"),
            None => "root".to_string(),
        };
        let id = if is_start {
            format!("__state_start_{tag}_{idx}")
        } else {
            format!("__state_end_{tag}_{idx}")
        };
        self.nodes.push(StateNode {
            id,
            label: None,
            descriptions: Vec::new(),
            kind: if is_start {
                StateKind::Start
            } else {
                StateKind::End
            },
            subgraph: current_sg,
            attach_to: None,
            classes: Vec::new(),
            inline_style: StyleProps::default(),
        });
        map.insert(current_sg, idx);
        idx
    }

    fn resolve_endpoint(
        &mut self,
        raw_tok: &str,
        is_source: bool,
        current_sg: Option<usize>,
    ) -> Result<StateEndpoint> {
        let (clean, classes) = split_inline_classes(raw_tok);
        if clean == "[*]" {
            return Ok(StateEndpoint::Node(
                self.ensure_pseudo_star(is_source, current_sg),
            ));
        }
        if classes.is_empty()
            && let Some(&sg_idx) = self.subgraph_map.get(clean)
        {
            return Ok(StateEndpoint::Subgraph(sg_idx));
        }
        Ok(StateEndpoint::Node(
            self.ensure_state(raw_tok, None, None, current_sg),
        ))
    }
}

fn find_transition_op(line: &str) -> Option<(usize, usize)> {
    let bytes = line.as_bytes();
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
        if line[i..].starts_with("-->") {
            return Some((i, 3));
        }
        if line[i..].starts_with("->") {
            return Some((i, 2));
        }
        i += 1;
    }
    None
}

fn split_label_colon(raw: &str) -> (&str, Option<&str>) {
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
        // Treat `:` as separator only when not part of `:::`.
        if b == b':' && !raw[i..].starts_with(":::") {
            return (raw[..i].trim(), Some(raw[i + 1..].trim()));
        }
        i += 1;
    }
    (raw.trim(), None)
}

fn parse_composite_header(raw: &str, line_no: usize) -> Result<(String, String)> {
    let r = raw.trim();
    if r.is_empty() {
        return Err(Error::Mermaid {
            line: Some(line_no),
            message: "expected state name before `{`".to_string(),
        });
    }
    if let Some((before_as, after_as)) = split_as_keyword(r) {
        let title = unquote_text(before_as);
        let (id, _) = split_inline_classes(after_as);
        if id.is_empty() {
            return Err(Error::Mermaid {
                line: Some(line_no),
                message: "expected state identifier after `as`".to_string(),
            });
        }
        return Ok((id.to_string(), title));
    }
    if r.starts_with('"') {
        let title = unquote_text(r);
        return Ok((title.clone(), title));
    }
    let (id, _) = split_inline_classes(r);
    Ok((id.to_string(), id.to_string()))
}

fn split_as_keyword(input: &str) -> Option<(&str, &str)> {
    let bytes = input.as_bytes();
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
        if i > 0
            && input[..i].ends_with(char::is_whitespace)
            && input[i..].len() >= 3
            && input[i..i + 2].eq_ignore_ascii_case("as")
            && input[i + 2..].starts_with(char::is_whitespace)
        {
            return Some((input[..i].trim(), input[i + 2..].trim()));
        }
        i += 1;
    }
    None
}

fn parse_note_header(trimmed: &str) -> Option<(NoteSide, &str, Option<&str>)> {
    let rest = trimmed.strip_prefix("note")?.trim_start();
    let (side, after_side) = if let Some(r) = rest.strip_prefix("left") {
        (NoteSide::Left, r.trim_start())
    } else {
        let r = rest.strip_prefix("right")?;
        (NoteSide::Right, r.trim_start())
    };
    let after_of = after_side.strip_prefix("of")?.trim();
    if let Some((target, body)) = after_of.split_once(':') {
        Some((side, target.trim(), Some(body.trim())))
    } else if !after_of.is_empty() {
        Some((side, after_of, None))
    } else {
        None
    }
}

struct PendingTransition {
    lhs: String,
    rhs: String,
    label: Option<String>,
    dashed: bool,
    sg_at_stmt: Option<usize>,
}

enum PendingStyle {
    ClassAssign {
        ids: Vec<String>,
        classes: Vec<String>,
        sg_at_stmt: Option<usize>,
    },
    NodeStyle {
        ids: Vec<String>,
        props: StyleProps,
        sg_at_stmt: Option<usize>,
    },
}

fn parse_state_diagram(lines: &[SourceLine]) -> Result<ParsedStateDiagram> {
    let mut st = ParsedStateDiagram::new();
    let mut frames: Vec<CompositeFrame> = Vec::new();
    let mut pending_note: Option<(NoteSide, String, usize, Vec<String>)> = None;
    let mut pending_transitions: Vec<PendingTransition> = Vec::new();
    let mut pending_styles: Vec<PendingStyle> = Vec::new();
    let mut in_acc_descr_block = false;

    for sl in lines {
        let trimmed = sl.text.trim();
        if trimmed.is_empty() {
            continue;
        }

        if let Some((side, target_raw, start_line, mut note_lines)) = pending_note.take() {
            if trimmed.eq_ignore_ascii_case("end note") {
                let current_sg = frames.last().map(|f| f.active_sg);
                let target_idx = st.ensure_state(&target_raw, None, None, current_sg);
                let note_sg = st.nodes[target_idx].subgraph;
                let note_idx = st.nodes.len();
                let note_text = note_lines.join("\n");
                st.nodes.push(StateNode {
                    id: format!("__note_{note_idx}"),
                    label: Some(note_text),
                    descriptions: Vec::new(),
                    kind: StateKind::Note,
                    subgraph: note_sg,
                    attach_to: Some((target_idx, side)),
                    classes: Vec::new(),
                    inline_style: StyleProps::default(),
                });
            } else {
                note_lines.push(sl.text.trim().to_string());
                pending_note = Some((side, target_raw, start_line, note_lines));
            }
            continue;
        }

        if in_acc_descr_block {
            if trimmed.contains('}') {
                in_acc_descr_block = false;
            }
            continue;
        }
        if trimmed.starts_with("accTitle") || trimmed.starts_with("click ") {
            continue;
        }
        if trimmed.starts_with("accDescr") {
            if trimmed.contains('{') && !trimmed.contains('}') {
                in_acc_descr_block = true;
            }
            continue;
        }

        if trimmed == "}" {
            if frames.pop().is_none() {
                return Err(Error::Mermaid {
                    line: Some(sl.line_no),
                    message: "unexpected `}` with no matching composite state `{`".to_string(),
                });
            }
            continue;
        }

        if trimmed == "--" {
            let Some(top) = frames.last_mut() else {
                return Err(Error::Mermaid {
                    line: Some(sl.line_no),
                    message:
                        "concurrent divider `--` must appear inside a composite state `{ ... }`"
                            .to_string(),
                });
            };
            if !top.has_concurrent_regions {
                // Move existing items directly under `composite_sg` into region 0.
                let r0_idx = st.subgraphs.len();
                st.subgraphs.push(StateSubgraph {
                    title: String::new(),
                    parent: Some(top.composite_sg),
                    concurrent_region: true,
                    region_index: 0,
                    inline_style: StyleProps::default(),
                });
                let old_sg = top.composite_sg;
                for node in &mut st.nodes {
                    if node.subgraph == Some(old_sg) {
                        node.subgraph = Some(r0_idx);
                    }
                }
                for (i, sg) in st.subgraphs.iter_mut().enumerate() {
                    if i != r0_idx && sg.parent == Some(old_sg) {
                        sg.parent = Some(r0_idx);
                    }
                }
                if let Some(s) = st.region_starts.remove(&Some(old_sg)) {
                    st.region_starts.insert(Some(r0_idx), s);
                }
                if let Some(e) = st.region_ends.remove(&Some(old_sg)) {
                    st.region_ends.insert(Some(r0_idx), e);
                }
                for tr in &mut pending_transitions {
                    if tr.sg_at_stmt == Some(old_sg) {
                        tr.sg_at_stmt = Some(r0_idx);
                    }
                }
                for ps in &mut pending_styles {
                    let sg_ref = match ps {
                        PendingStyle::ClassAssign { sg_at_stmt, .. } => sg_at_stmt,
                        PendingStyle::NodeStyle { sg_at_stmt, .. } => sg_at_stmt,
                    };
                    if *sg_ref == Some(old_sg) {
                        *sg_ref = Some(r0_idx);
                    }
                }
                top.has_concurrent_regions = true;
                top.region_count = 1;
            }
            let r_idx = st.subgraphs.len();
            st.subgraphs.push(StateSubgraph {
                title: String::new(),
                parent: Some(top.composite_sg),
                concurrent_region: true,
                region_index: top.region_count,
                inline_style: StyleProps::default(),
            });
            top.region_count += 1;
            top.active_sg = r_idx;
            continue;
        }

        let current_sg = frames.last().map(|f| f.active_sg);

        if let Some(rest) = trimmed.strip_prefix("direction")
            && rest.starts_with(char::is_whitespace)
        {
            let dir_tok = rest.trim();
            let dir = Direction::from_token(dir_tok).ok_or_else(|| Error::Mermaid {
                line: Some(sl.line_no),
                message: format!("unknown direction `{dir_tok}`"),
            })?;
            if frames.is_empty() {
                st.direction = dir;
            }
            continue;
        }

        if let Some(directive) = try_parse_style_directive(trimmed, sl.line_no)? {
            match directive {
                StyleDirective::ClassDef(name, props) => {
                    st.class_defs.entry(name).or_default().merge_from(&props);
                }
                StyleDirective::ClassAssign { ids, classes } => {
                    pending_styles.push(PendingStyle::ClassAssign {
                        ids,
                        classes,
                        sg_at_stmt: current_sg,
                    });
                }
                StyleDirective::NodeStyle { ids, props } => {
                    pending_styles.push(PendingStyle::NodeStyle {
                        ids,
                        props,
                        sg_at_stmt: current_sg,
                    });
                }
            }
            continue;
        }

        if let Some((side, target_raw, body_opt)) = parse_note_header(trimmed) {
            if let Some(body) = body_opt {
                let target_idx = st.ensure_state(target_raw, None, None, current_sg);
                let note_sg = st.nodes[target_idx].subgraph;
                let note_idx = st.nodes.len();
                st.nodes.push(StateNode {
                    id: format!("__note_{note_idx}"),
                    label: Some(unquote_text(body)),
                    descriptions: Vec::new(),
                    kind: StateKind::Note,
                    subgraph: note_sg,
                    attach_to: Some((target_idx, side)),
                    classes: Vec::new(),
                    inline_style: StyleProps::default(),
                });
            } else {
                pending_note = Some((side, target_raw.to_string(), sl.line_no, Vec::new()));
            }
            continue;
        }

        if let Some(rest) = trimmed.strip_prefix("state")
            && rest.starts_with(char::is_whitespace)
        {
            let rest = rest.trim();
            if let Some(before_brace) = rest.strip_suffix('{') {
                let (id, title) = parse_composite_header(before_brace, sl.line_no)?;
                let sg_idx = st.subgraphs.len();
                st.subgraph_map.insert(id, sg_idx);
                st.subgraphs.push(StateSubgraph {
                    title,
                    parent: current_sg,
                    concurrent_region: false,
                    region_index: 0,
                    inline_style: StyleProps::default(),
                });
                frames.push(CompositeFrame {
                    composite_sg: sg_idx,
                    active_sg: sg_idx,
                    has_concurrent_regions: false,
                    region_count: 0,
                    open_line: sl.line_no,
                });
                continue;
            }
            if let Some(left) = rest.find("<<")
                && let Some(right) = rest[left..].find(">>")
            {
                let id_part = rest[..left].trim();
                let stereo = rest[left + 2..left + right].trim().to_ascii_lowercase();
                let kind = match stereo.as_str() {
                    "choice" => StateKind::Choice,
                    "fork" => StateKind::Fork,
                    "join" => StateKind::Join,
                    _ => {
                        return Err(Error::Mermaid {
                            line: Some(sl.line_no),
                            message: format!("unknown state stereotype `<<{stereo}>>`"),
                        });
                    }
                };
                if id_part.is_empty() {
                    return Err(Error::Mermaid {
                        line: Some(sl.line_no),
                        message: "expected state id before stereotype".to_string(),
                    });
                }
                st.ensure_state(id_part, None, Some(kind), current_sg);
                continue;
            }
            if let Some((desc_raw, alias_raw)) = split_as_keyword(rest) {
                let label = unquote_text(desc_raw);
                st.ensure_state(alias_raw, Some(label), None, current_sg);
                continue;
            }
            st.ensure_state(rest, None, None, current_sg);
            continue;
        }

        if let Some((op_pos, op_len)) = find_transition_op(trimmed) {
            let lhs = trimmed[..op_pos].trim();
            let rhs_all = trimmed[op_pos + op_len..].trim();
            if lhs.is_empty() || rhs_all.is_empty() {
                return Err(Error::Mermaid {
                    line: Some(sl.line_no),
                    message: format!("incomplete transition `{trimmed}`"),
                });
            }
            let (rhs, label_opt) = split_label_colon(rhs_all);
            if rhs.is_empty() {
                return Err(Error::Mermaid {
                    line: Some(sl.line_no),
                    message: format!("missing target state in `{trimmed}`"),
                });
            }
            pending_transitions.push(PendingTransition {
                lhs: lhs.to_string(),
                rhs: rhs.to_string(),
                label: label_opt.filter(|l| !l.is_empty()).map(unquote_text),
                dashed: false,
                sg_at_stmt: current_sg,
            });
            continue;
        }

        let (head, desc_opt) = split_label_colon(trimmed);
        if let Some(desc) = desc_opt
            && !head.is_empty()
            && !head.contains(char::is_whitespace)
        {
            if let Some(&sg_idx) = st.subgraph_map.get(head) {
                if !desc.is_empty() {
                    let subtitle = unquote_text(desc);
                    if st.subgraphs[sg_idx].title == head {
                        st.subgraphs[sg_idx].title = format!("{head}: {subtitle}");
                    } else {
                        st.subgraphs[sg_idx].title =
                            format!("{}\n{}", st.subgraphs[sg_idx].title, subtitle);
                    }
                }
            } else {
                let idx = st.ensure_state(head, None, None, current_sg);
                if !desc.is_empty() {
                    st.nodes[idx].descriptions.push(unquote_text(desc));
                }
            }
            continue;
        }

        if !trimmed.contains(char::is_whitespace) {
            st.ensure_state(trimmed, None, None, current_sg);
            continue;
        }

        return Err(Error::Mermaid {
            line: Some(sl.line_no),
            message: format!("invalid stateDiagram statement `{trimmed}`"),
        });
    }

    if let Some((_, _, start_line, _)) = pending_note {
        return Err(Error::Mermaid {
            line: Some(start_line),
            message: "unclosed `note` block (missing `end note`)".to_string(),
        });
    }
    if let Some(frame) = frames.last() {
        return Err(Error::Mermaid {
            line: Some(frame.open_line),
            message: "unclosed composite state block (missing `}`)".to_string(),
        });
    }

    // Pass 2: resolve transition and style targets now that all composite state blocks are known.
    for tr in pending_transitions {
        let from = st.resolve_endpoint(&tr.lhs, true, tr.sg_at_stmt)?;
        let to = st.resolve_endpoint(&tr.rhs, false, tr.sg_at_stmt)?;
        st.edges.push(StateEdge {
            from,
            to,
            label: tr.label,
            dashed: tr.dashed,
        });
    }
    for ps in pending_styles {
        match ps {
            PendingStyle::ClassAssign {
                ids,
                classes,
                sg_at_stmt,
            } => {
                for id in ids {
                    if let Some(&sg_idx) = st.subgraph_map.get(&id) {
                        for cls in &classes {
                            if let Some(def) = st.class_defs.get(cls) {
                                st.subgraphs[sg_idx].inline_style.merge_from(def);
                            }
                        }
                    } else {
                        let idx = st.ensure_state(&id, None, None, sg_at_stmt);
                        for cls in &classes {
                            if !st.nodes[idx].classes.contains(cls) {
                                st.nodes[idx].classes.push(cls.clone());
                            }
                        }
                    }
                }
            }
            PendingStyle::NodeStyle {
                ids,
                props,
                sg_at_stmt,
            } => {
                for id in ids {
                    if let Some(&sg_idx) = st.subgraph_map.get(&id) {
                        st.subgraphs[sg_idx].inline_style.merge_from(&props);
                    } else {
                        let idx = st.ensure_state(&id, None, None, sg_at_stmt);
                        st.nodes[idx].inline_style.merge_from(&props);
                    }
                }
            }
        }
    }

    Ok(st)
}

fn node_shape_for_state(kind: StateKind) -> NodeShape {
    match kind {
        StateKind::Start | StateKind::End => NodeShape::Circle,
        StateKind::Choice => NodeShape::Diamond,
        StateKind::Fork | StateKind::Join => NodeShape::Rect,
        StateKind::Note => NodeShape::Rect,
        StateKind::Regular => NodeShape::Rounded,
    }
}

fn in_subgraph_tree(
    subgraphs: &[StateSubgraph],
    mut node_sg: Option<usize>,
    target: usize,
) -> bool {
    let mut guard = 0;
    while let Some(idx) = node_sg {
        if idx == target {
            return true;
        }
        node_sg = subgraphs.get(idx).and_then(|s| s.parent);
        guard += 1;
        if guard > subgraphs.len() {
            break;
        }
    }
    false
}

pub(crate) fn render(
    chart: &MermaidChart,
    theme_name: &str,
    fm_title: Option<&str>,
    lines: &[SourceLine],
) -> Result<(String, f32, f32)> {
    let mut st = parse_state_diagram(lines)?;
    if st.nodes.is_empty() && st.subgraphs.is_empty() {
        return Err(Error::Mermaid {
            line: lines.first().map(|l| l.line_no),
            message: "stateDiagram has no states".to_string(),
        });
    }

    // Add invisible anchors for any empty composite state so its container renders.
    for sg_idx in 0..st.subgraphs.len() {
        let has_member = st.nodes.iter().any(|n| n.subgraph == Some(sg_idx))
            || st.subgraphs.iter().any(|s| s.parent == Some(sg_idx));
        if !has_member {
            st.nodes.push(StateNode {
                id: format!("__sg_anchor_{sg_idx}"),
                label: Some(String::new()),
                descriptions: Vec::new(),
                kind: StateKind::Regular,
                subgraph: Some(sg_idx),
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
    let font_size = 13.5_f32;
    let desc_font_size = 12.0_f32;
    let edge_font_size = 12.0_f32;
    let note_font_size = 12.0_f32;
    let sg_font_size = 13.0_f32;
    let is_horizontal = matches!(st.direction, Direction::LeftRight | Direction::RightLeft);

    struct MeasuredState {
        title: FormattedText,
        desc: Option<FormattedText>,
        header_h: f32,
        w: f32,
        h: f32,
        is_anchor: bool,
    }

    let mut measured = Vec::with_capacity(st.nodes.len());
    let mut layout_nodes = Vec::with_capacity(st.nodes.len());
    for n in &st.nodes {
        let is_anchor = n.id.starts_with("__sg_anchor_");
        let display = n.label.as_deref().unwrap_or(&n.id);
        let title_ft = format_and_measure(display, &font_family, font_size);
        let desc_ft = (!n.descriptions.is_empty()).then(|| {
            let joined = n.descriptions.join("\n");
            format_and_measure(&joined, &font_family, desc_font_size)
        });
        let (w, h, header_h) = if is_anchor {
            (40.0, 20.0, 20.0)
        } else {
            match n.kind {
                StateKind::Start => (18.0, 18.0, 18.0),
                StateKind::End => (24.0, 24.0, 24.0),
                StateKind::Choice => (28.0, 28.0, 28.0),
                StateKind::Fork | StateKind::Join => {
                    if is_horizontal {
                        (8.0, 66.0, 66.0)
                    } else {
                        (66.0, 8.0, 8.0)
                    }
                }
                StateKind::Note => {
                    let nft = format_and_measure(display, &font_family, note_font_size);
                    (
                        (nft.width + 26.0).max(60.0),
                        (nft.height + 16.0).max(32.0),
                        0.0,
                    )
                }
                StateKind::Regular => {
                    if let Some(ref d) = desc_ft {
                        let hh = title_ft.height + 14.0;
                        let dh = d.height + 14.0;
                        let w = title_ft.width.max(d.width).max(52.0) + 28.0;
                        (w, hh + dh, hh)
                    } else {
                        let w = (title_ft.width + 28.0).max(68.0);
                        let h = (title_ft.height + 18.0).max(36.0);
                        (w, h, h)
                    }
                }
            }
        };
        let note_ft = if n.kind == StateKind::Note {
            format_and_measure(display, &font_family, note_font_size)
        } else {
            title_ft
        };
        layout_nodes.push(LayeredNodeInput {
            width: w,
            height: h,
            subgraph: n.subgraph,
            attach_to: n.attach_to,
        });
        measured.push(MeasuredState {
            title: note_ft,
            desc: desc_ft,
            header_h,
            w,
            h,
            is_anchor,
        });
    }

    let mut sg_labels = Vec::with_capacity(st.subgraphs.len());
    let mut layout_subgraphs = Vec::with_capacity(st.subgraphs.len());
    for sg in &st.subgraphs {
        let ft = if sg.concurrent_region || sg.title.is_empty() {
            FormattedText {
                lines: vec![],
                width: 0.0,
                height: 0.0,
                line_height: 0.0,
            }
        } else {
            format_and_measure(&sg.title, &font_family, sg_font_size)
        };
        layout_subgraphs.push(LayeredSubgraphInput {
            parent: sg.parent,
            title_width: ft.width,
            title_height: ft.height,
            concurrent_region: sg.concurrent_region,
        });
        sg_labels.push(ft);
    }

    // Identify entry and exit layout representatives per composite subgraph so
    // transitions into/out of composite states preserve clean hierarchical ranking.
    let mut internal_in: HashSet<usize> = HashSet::new();
    let mut internal_out: HashSet<usize> = HashSet::new();
    for e in &st.edges {
        if let (StateEndpoint::Node(u), StateEndpoint::Node(v)) = (e.from, e.to) {
            for sg_idx in 0..st.subgraphs.len() {
                if in_subgraph_tree(&st.subgraphs, st.nodes[u].subgraph, sg_idx)
                    && in_subgraph_tree(&st.subgraphs, st.nodes[v].subgraph, sg_idx)
                {
                    internal_out.insert(u);
                    internal_in.insert(v);
                }
            }
        }
    }

    let mut sg_entries: Vec<Vec<usize>> = vec![Vec::new(); st.subgraphs.len()];
    let mut sg_exits: Vec<Vec<usize>> = vec![Vec::new(); st.subgraphs.len()];
    for sg_idx in 0..st.subgraphs.len() {
        let members: Vec<usize> = st
            .nodes
            .iter()
            .enumerate()
            .filter(|(_, n)| {
                n.kind != StateKind::Note && in_subgraph_tree(&st.subgraphs, n.subgraph, sg_idx)
            })
            .map(|(u, _)| u)
            .collect();
        let entries: Vec<usize> = members
            .iter()
            .copied()
            .filter(|u| !internal_in.contains(u))
            .collect();
        let exits: Vec<usize> = members
            .iter()
            .copied()
            .filter(|u| !internal_out.contains(u))
            .collect();
        sg_entries[sg_idx] = if entries.is_empty() {
            members.first().copied().into_iter().collect()
        } else {
            entries
        };
        sg_exits[sg_idx] = if exits.is_empty() {
            members.last().copied().into_iter().collect()
        } else {
            exits
        };
    }

    let mut layout_edges = Vec::with_capacity(st.edges.len() * 2);
    let mut edge_layout_slot = Vec::with_capacity(st.edges.len());
    for e in &st.edges {
        let from_list: Vec<usize> = match e.from {
            StateEndpoint::Node(u) => vec![u],
            StateEndpoint::Subgraph(sg) => sg_exits[sg].clone(),
        };
        let to_list: Vec<usize> = match e.to {
            StateEndpoint::Node(v) => vec![v],
            StateEndpoint::Subgraph(sg) => sg_entries[sg].clone(),
        };
        let primary_from = from_list.first().copied().unwrap_or(0);
        let primary_to = to_list.first().copied().unwrap_or(0);
        let slot = layout_edges.len();
        layout_edges.push(LayeredEdgeInput {
            from: primary_from,
            to: primary_to,
            has_label: e.label.as_ref().is_some_and(|l| !l.is_empty()),
            min_len: 1,
        });
        edge_layout_slot.push(slot);

        // Enforce topological ordering against every sibling region inside target/source composite.
        for &f in &from_list {
            for &t in &to_list {
                if f == primary_from && t == primary_to {
                    continue;
                }
                layout_edges.push(LayeredEdgeInput {
                    from: f,
                    to: t,
                    has_label: false,
                    min_len: 1,
                });
            }
        }
    }

    let layout = sugiyama::layout(&LayeredGraph {
        direction: st.direction,
        rank_sep_extra: 0.0,
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

    // 1. Composite state containers (outer-to-inner; skip border rects on concurrent regions).
    let mut sg_depths: Vec<(usize, usize)> = (0..st.subgraphs.len())
        .map(|i| {
            let mut d = 0usize;
            let mut cur = st.subgraphs[i].parent;
            while let Some(p) = cur {
                d += 1;
                cur = st.subgraphs[p].parent;
            }
            (d, i)
        })
        .collect();
    sg_depths.sort_by(|a, b| a.0.cmp(&b.0).then_with(|| a.1.cmp(&b.1)));

    for &(depth, sg_idx) in &sg_depths {
        let sg = &st.subgraphs[sg_idx];
        if sg.concurrent_region {
            continue;
        }
        let b = layout.subgraphs[sg_idx];
        if !b.visible {
            continue;
        }
        let fill = sg
            .inline_style
            .fill
            .unwrap_or_else(|| palette.subgraph_fill(depth));
        let stroke = sg.inline_style.stroke.unwrap_or(palette.subgraph_stroke);
        let stroke_w = sg.inline_style.stroke_width.unwrap_or(1.3);
        let text_color = sg.inline_style.color.unwrap_or(palette.text_color);

        cc.rect(Rect {
            left: b.x,
            top: b.y,
            width: b.width,
            height: b.height,
            rx: Some(8.0),
            ry: Some(8.0),
            fill: Some(fill.into()),
            color: Some(stroke),
            stroke_width: Some(stroke_w),
            stroke_dash_array: sg.inline_style.stroke_dasharray.clone(),
            ..Default::default()
        });

        let title_ft = &sg_labels[sg_idx];
        let mut header_bottom = b.y;
        if !title_ft.lines.is_empty() {
            let hh = title_ft.height + 16.0;
            header_bottom = b.y + hh;
            draw_multiline_text(
                &mut cc,
                b.x + b.width * 0.5,
                b.y + hh * 0.5,
                title_ft,
                &font_family,
                sg_font_size,
                text_color,
                "middle",
                true,
            );
            cc.line(Line {
                color: Some(stroke),
                stroke_width: 1.0,
                left: b.x,
                top: header_bottom,
                right: b.x + b.width,
                bottom: header_bottom,
                ..Default::default()
            });
        }

        // Draw dashed dividers between adjacent concurrent regions inside this composite.
        let mut child_regions: Vec<(usize, super::layout::SubgraphBox)> = st
            .subgraphs
            .iter()
            .enumerate()
            .filter(|(ci, cs)| {
                cs.parent == Some(sg_idx) && cs.concurrent_region && layout.subgraphs[*ci].visible
            })
            .map(|(ci, cs)| (cs.region_index, layout.subgraphs[ci]))
            .collect();
        child_regions.sort_by_key(|r| r.0);
        if child_regions.len() >= 2 {
            for pair in child_regions.windows(2) {
                let (_, rb1) = pair[0];
                let (_, rb2) = pair[1];
                if is_horizontal {
                    let div_y = (rb1.y + rb1.height + rb2.y) * 0.5;
                    cc.line(Line {
                        color: Some(stroke),
                        stroke_width: 1.2,
                        left: b.x + 2.0,
                        top: div_y,
                        right: b.x + b.width - 2.0,
                        bottom: div_y,
                        stroke_dash_array: Some("5,4".to_string()),
                    });
                } else {
                    let div_x = (rb1.x + rb1.width + rb2.x) * 0.5;
                    cc.line(Line {
                        color: Some(stroke),
                        stroke_width: 1.2,
                        left: div_x,
                        top: header_bottom + 2.0,
                        right: div_x,
                        bottom: b.y + b.height - 2.0,
                        stroke_dash_array: Some("5,4".to_string()),
                    });
                }
            }
        }
    }

    // 2. Dashed connector lines from attached `note` boxes to their target states.
    for (u, n) in st.nodes.iter().enumerate() {
        if n.kind == StateKind::Note
            && let Some((target, _)) = n.attach_to
        {
            let np = layout.nodes[u];
            let tp = layout.nodes[target];
            let (nw, nh) = (measured[u].w, measured[u].h);
            let (tw, th) = (measured[target].w, measured[target].h);
            let p0 = clip_point_to_rect(np.cx, np.cy, nw, nh, Point::from((tp.cx, tp.cy)));
            let p1 = clip_point_to_shape(
                node_shape_for_state(st.nodes[target].kind),
                tp.cx,
                tp.cy,
                tw,
                th,
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

    // 3. Transitions.
    for (edge_idx, e) in st.edges.iter().enumerate() {
        let routed = &layout.edges[edge_layout_slot[edge_idx]];
        if routed.waypoints.len() < 2 {
            continue;
        }
        let mut pts = routed.waypoints.clone();
        if !routed.self_loop {
            let n_pts = pts.len();
            let first_target = pts[1];
            pts[0] = match e.from {
                StateEndpoint::Node(u) => {
                    let p = layout.nodes[u];
                    clip_point_to_shape(
                        node_shape_for_state(st.nodes[u].kind),
                        p.cx,
                        p.cy,
                        measured[u].w,
                        measured[u].h,
                        first_target,
                    )
                }
                StateEndpoint::Subgraph(sg) => {
                    let b = layout.subgraphs[sg];
                    clip_point_to_rect(
                        b.x + b.width * 0.5,
                        b.y + b.height * 0.5,
                        b.width,
                        b.height,
                        first_target,
                    )
                }
            };
            let last_target = pts[n_pts - 2];
            pts[n_pts - 1] = match e.to {
                StateEndpoint::Node(v) => {
                    let p = layout.nodes[v];
                    clip_point_to_shape(
                        node_shape_for_state(st.nodes[v].kind),
                        p.cx,
                        p.cy,
                        measured[v].w,
                        measured[v].h,
                        last_target,
                    )
                }
                StateEndpoint::Subgraph(sg) => {
                    let b = layout.subgraphs[sg];
                    clip_point_to_rect(
                        b.x + b.width * 0.5,
                        b.y + b.height * 0.5,
                        b.width,
                        b.height,
                        last_target,
                    )
                }
            };
        }

        let (d, mid, _, end_dir) = build_smooth_edge_path(&pts);
        cc.raw_path(super::super::component::RawPath {
            d,
            fill: None,
            stroke: Some(palette.edge_stroke),
            stroke_width: 1.7,
            stroke_dash_array: e.dashed.then(|| "5,4".to_string()),
            stroke_linecap: Some("round"),
            stroke_linejoin: Some("round"),
            ..Default::default()
        });
        draw_edge_marker(
            &mut cc,
            EdgeMarker::Arrow,
            *pts.last().unwrap(),
            end_dir,
            palette.edge_stroke,
            palette.opaque_bg,
            1.7,
        );
        if let Some(ref lbl) = e.label
            && !lbl.is_empty()
        {
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

    // 4. State nodes, pseudo-states, and note cards.
    for (u, n) in st.nodes.iter().enumerate() {
        let m = &measured[u];
        if m.is_anchor {
            continue;
        }
        let pos = layout.nodes[u];
        let eff_style = resolve_style(&n.classes, &n.inline_style, &st.class_defs);
        let fill = eff_style.fill.unwrap_or(palette.node_fill);
        let stroke = eff_style.stroke.unwrap_or(palette.node_stroke);
        let stroke_w = eff_style.stroke_width.unwrap_or(1.6);
        let text_color = eff_style.color.unwrap_or(palette.text_color);

        match n.kind {
            StateKind::Start => {
                cc.circle(Circle {
                    cx: pos.cx,
                    cy: pos.cy,
                    r: m.w * 0.5,
                    fill: Some(palette.text_color),
                    stroke_color: Some(palette.text_color),
                    stroke_width: 1.0,
                    ..Default::default()
                });
            }
            StateKind::End => {
                let outer_r = m.w * 0.5;
                cc.circle(Circle {
                    cx: pos.cx,
                    cy: pos.cy,
                    r: outer_r,
                    fill: Some(palette.opaque_bg),
                    stroke_color: Some(palette.text_color),
                    stroke_width: 1.8,
                    ..Default::default()
                });
                cc.circle(Circle {
                    cx: pos.cx,
                    cy: pos.cy,
                    r: (outer_r * 0.58).max(4.0),
                    fill: Some(palette.text_color),
                    stroke_color: None,
                    ..Default::default()
                });
            }
            StateKind::Choice => {
                let empty = FormattedText {
                    lines: vec![],
                    width: 0.0,
                    height: 0.0,
                    line_height: 0.0,
                };
                draw_node(
                    &mut cc,
                    NodeShape::Diamond,
                    pos.cx,
                    pos.cy,
                    m.w,
                    m.h,
                    fill,
                    stroke,
                    stroke_w,
                    eff_style.stroke_dasharray,
                    text_color,
                    &empty,
                    &font_family,
                    font_size,
                );
            }
            StateKind::Fork | StateKind::Join => {
                cc.rect(Rect {
                    left: pos.cx - m.w * 0.5,
                    top: pos.cy - m.h * 0.5,
                    width: m.w,
                    height: m.h,
                    rx: Some(3.0),
                    ry: Some(3.0),
                    fill: Some(palette.text_color.into()),
                    color: None,
                    ..Default::default()
                });
            }
            StateKind::Note => {
                draw_note_box(
                    &mut cc,
                    pos.cx,
                    pos.cy,
                    m.w,
                    m.h,
                    &m.title,
                    &palette,
                    &font_family,
                    note_font_size,
                );
            }
            StateKind::Regular => {
                if let Some(ref desc) = m.desc {
                    let x = pos.cx - m.w * 0.5;
                    let y = pos.cy - m.h * 0.5;
                    cc.rect(Rect {
                        left: x,
                        top: y,
                        width: m.w,
                        height: m.h,
                        rx: Some(8.0),
                        ry: Some(8.0),
                        fill: Some(fill.into()),
                        color: None,
                        ..Default::default()
                    });
                    draw_multiline_text(
                        &mut cc,
                        pos.cx,
                        y + m.header_h * 0.5,
                        &m.title,
                        &font_family,
                        font_size,
                        text_color,
                        "middle",
                        true,
                    );
                    cc.line(Line {
                        color: Some(stroke),
                        stroke_width: 1.0,
                        left: x,
                        top: y + m.header_h,
                        right: x + m.w,
                        bottom: y + m.header_h,
                        ..Default::default()
                    });
                    draw_multiline_text(
                        &mut cc,
                        pos.cx,
                        y + m.header_h + (m.h - m.header_h) * 0.5,
                        desc,
                        &font_family,
                        desc_font_size,
                        text_color,
                        "middle",
                        false,
                    );
                    cc.rect(Rect {
                        left: x,
                        top: y,
                        width: m.w,
                        height: m.h,
                        rx: Some(8.0),
                        ry: Some(8.0),
                        fill: Some(Color::transparent().into()),
                        color: Some(stroke),
                        stroke_width: Some(stroke_w),
                        stroke_dash_array: eff_style.stroke_dasharray,
                        ..Default::default()
                    });
                } else {
                    draw_node(
                        &mut cc,
                        NodeShape::Rounded,
                        pos.cx,
                        pos.cy,
                        m.w,
                        m.h,
                        fill,
                        stroke,
                        stroke_w,
                        eff_style.stroke_dasharray,
                        text_color,
                        &m.title,
                        &font_family,
                        font_size,
                    );
                }
            }
        }
    }

    let svg = canvas.svg()?;
    Ok((svg, out_w, out_h))
}

#[cfg(test)]
mod tests {
    use super::super::preprocess::preprocess;
    use super::*;

    #[test]
    fn renders_composite_concurrent_choice_fork_notes_and_styles() {
        let src = r#"
stateDiagram-v2
    [*] --> Idle
    Idle --> Checking : submit
    state Checking <<choice>>
    Checking --> Processing : ok
    Checking --> Failed : bad
    state Fork1 <<fork>>
    Idle --> Fork1
    state Processing {
        [*] --> Pay
        Pay --> Done : paid
        --
        [*] --> Ship
        Ship --> Delivered
    }
    Processing : Payment and shipping
    note right of Idle : Waiting for order
    note left of Failed
        Multi line
        failure note
    end note
    Failed --> [*]
    classDef bad fill:#fee2e2,stroke:#dc2626
    class Failed bad
"#;
        let prep = preprocess(src).unwrap();
        let chart = MermaidChart::new(src);
        let (svg, w, h) = render(&chart, "light", prep.title.as_deref(), &prep.lines).unwrap();
        assert!(svg.starts_with("<svg") && w > 100.0 && h > 100.0);
        assert!(svg.contains("Processing: Payment and shipping"));
        assert!(
            !svg.contains("\nProcessing\n"),
            "forward reference to composite state must not spawn a duplicate regular node"
        );
        assert!(svg.contains("Waiting for order"));
        assert!(svg.contains("failure note"));
        assert!(svg.contains("#FEE2E2"));
        assert!(svg.contains("stroke-dasharray=\"5,4\""));
    }

    #[test]
    fn reports_line_numbered_errors_on_unclosed_composite_and_note() {
        let unclosed_comp = preprocess("stateDiagram-v2\n  state Active {\n    [*] --> Run")
            .unwrap()
            .lines;
        let err = parse_state_diagram(&unclosed_comp).unwrap_err().to_string();
        assert!(err.contains("line 2"), "{err}");

        let unclosed_note = preprocess("stateDiagram-v2\n  A --> B\n  note right of A\n    text")
            .unwrap()
            .lines;
        let err = parse_state_diagram(&unclosed_note).unwrap_err().to_string();
        assert!(err.contains("line 3"), "{err}");

        let stray_div = preprocess("stateDiagram-v2\n  --").unwrap().lines;
        let err = parse_state_diagram(&stray_div).unwrap_err().to_string();
        assert!(err.contains("line 2"), "{err}");
    }
}
