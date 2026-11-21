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

//! Mermaid `sequenceDiagram` parser, 2D column/row layout and pure-SVG renderer.

use std::collections::HashMap;

use super::super::canvas::Canvas;
use super::super::color::Color;
use super::super::component::{Circle, Line, Polygon, RawPath, Rect};
use super::super::error::{Error, Result};
use super::super::theme::get_theme;
use super::super::util::{Point, format_float};
use super::MermaidChart;
use super::preprocess::SourceLine;
use super::style::{MermaidPalette, mix_color, parse_css_color};
use super::text::{FormattedText, draw_multiline_text, format_and_measure};

const PARTICIPANT_PAD_X: f32 = 16.0;
const PARTICIPANT_MIN_W: f32 = 84.0;
const BASE_COL_GAP: f32 = 38.0;
const ACT_WIDTH: f32 = 10.0;
const ACT_OFFSET: f32 = 5.0;
const BLOCK_INSET_X: f32 = 14.0;
const BLOCK_HEADER_H: f32 = 24.0;
const SELF_LOOP_W: f32 = 38.0;
const SELF_LOOP_H: f32 = 24.0;
const CANVAS_PAD: f32 = 24.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ParticipantKind {
    Participant,
    Actor,
}

#[derive(Clone, Debug)]
struct SeqParticipant {
    label: String,
    kind: ParticipantKind,
    box_idx: Option<usize>,
    created: bool,
}

#[derive(Clone, Debug)]
struct SeqBox {
    title: String,
    color: Option<Color>,
    members: Vec<usize>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ArrowHead {
    Filled,
    Open,
    Cross,
    Async,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct ArrowSpec {
    dashed: bool,
    bidirectional: bool,
    head: ArrowHead,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum NotePos {
    LeftOf,
    RightOf,
    Over,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum BlockKind {
    Alt,
    Opt,
    Loop,
    Par,
    Critical,
    Break,
    Rect,
}

impl BlockKind {
    fn keyword(&self) -> &'static str {
        match self {
            Self::Alt => "alt",
            Self::Opt => "opt",
            Self::Loop => "loop",
            Self::Par => "par",
            Self::Critical => "critical",
            Self::Break => "break",
            Self::Rect => "rect",
        }
    }
}

#[derive(Clone, Debug)]
enum SeqEvent {
    Message {
        from: usize,
        to: usize,
        arrow: ArrowSpec,
        text: String,
        activate_to: bool,
        deactivate_from: bool,
    },
    Note {
        pos: NotePos,
        from: usize,
        to: usize,
        text: String,
    },
    Activate(usize),
    Deactivate(usize),
    Create(usize),
    Destroy(usize),
    BlockStart {
        kind: BlockKind,
        label: String,
        rect_color: Option<Color>,
    },
    BlockDivider {
        label: String,
    },
    BlockEnd,
}

struct ParsedSequence {
    title: Option<String>,
    autonumber: bool,
    participants: Vec<SeqParticipant>,
    participant_map: HashMap<String, usize>,
    boxes: Vec<SeqBox>,
    events: Vec<SeqEvent>,
}

impl ParsedSequence {
    fn new() -> Self {
        Self {
            title: None,
            autonumber: false,
            participants: Vec::new(),
            participant_map: HashMap::new(),
            boxes: Vec::new(),
            events: Vec::new(),
        }
    }

    fn ensure_participant(
        &mut self,
        id: &str,
        label: Option<&str>,
        kind: Option<ParticipantKind>,
        box_idx: Option<usize>,
    ) -> usize {
        if let Some(&idx) = self.participant_map.get(id) {
            let p = &mut self.participants[idx];
            if let Some(l) = label {
                p.label = l.to_string();
            }
            if let Some(k) = kind {
                p.kind = k;
            }
            if p.box_idx.is_none() && box_idx.is_some() {
                p.box_idx = box_idx;
            }
            return idx;
        }
        let idx = self.participants.len();
        self.participant_map.insert(id.to_string(), idx);
        self.participants.push(SeqParticipant {
            label: label.unwrap_or(id).to_string(),
            kind: kind.unwrap_or(ParticipantKind::Participant),
            box_idx,
            created: false,
        });
        if let Some(b_idx) = box_idx
            && !self.boxes[b_idx].members.contains(&idx)
        {
            self.boxes[b_idx].members.push(idx);
        }
        idx
    }
}

fn parse_participant_decl(rest: &str, line_no: usize) -> Result<(String, Option<String>)> {
    let r = rest.trim();
    if r.is_empty() {
        return Err(Error::Mermaid {
            line: Some(line_no),
            message: "expected participant identifier".to_string(),
        });
    }
    if let Some((id, label)) = r.split_once(" as ") {
        let id = id.trim();
        let label = label.trim().trim_matches('"');
        if id.is_empty() {
            return Err(Error::Mermaid {
                line: Some(line_no),
                message: "empty participant identifier before `as`".to_string(),
            });
        }
        Ok((id.to_string(), Some(label.to_string())))
    } else {
        Ok((r.to_string(), None))
    }
}

fn parse_box_header(rest: &str) -> (Option<Color>, String) {
    let trimmed = rest.trim();
    if trimmed.is_empty() {
        return (None, String::new());
    }
    // Check `rgb(...)` or `rgba(...)` prefix first.
    let lower = trimmed.to_ascii_lowercase();
    if (lower.starts_with("rgb(") || lower.starts_with("rgba("))
        && let Some(end_p) = trimmed.find(')')
    {
        let color_str = &trimmed[..=end_p];
        if let Some(c) = parse_css_color(color_str) {
            return (Some(c), trimmed[end_p + 1..].trim().to_string());
        }
    }
    let mut parts = trimmed.splitn(2, char::is_whitespace);
    let first = parts.next().unwrap_or_default();
    let tail = parts.next().unwrap_or_default().trim();
    if let Some(c) = parse_css_color(first) {
        (Some(c), tail.to_string())
    } else {
        (None, trimmed.to_string())
    }
}

fn find_message_arrow(line: &str) -> Option<(usize, usize, ArrowSpec)> {
    const ARROWS: &[(&str, ArrowSpec)] = &[
        (
            "<<-->>",
            ArrowSpec {
                dashed: true,
                bidirectional: true,
                head: ArrowHead::Filled,
            },
        ),
        (
            "<<->>",
            ArrowSpec {
                dashed: false,
                bidirectional: true,
                head: ArrowHead::Filled,
            },
        ),
        (
            "-->>",
            ArrowSpec {
                dashed: true,
                bidirectional: false,
                head: ArrowHead::Filled,
            },
        ),
        (
            "->>",
            ArrowSpec {
                dashed: false,
                bidirectional: false,
                head: ArrowHead::Filled,
            },
        ),
        (
            "-->",
            ArrowSpec {
                dashed: true,
                bidirectional: false,
                head: ArrowHead::Open,
            },
        ),
        (
            "->",
            ArrowSpec {
                dashed: false,
                bidirectional: false,
                head: ArrowHead::Open,
            },
        ),
        (
            "--x",
            ArrowSpec {
                dashed: true,
                bidirectional: false,
                head: ArrowHead::Cross,
            },
        ),
        (
            "-x",
            ArrowSpec {
                dashed: false,
                bidirectional: false,
                head: ArrowHead::Cross,
            },
        ),
        (
            "--)",
            ArrowSpec {
                dashed: true,
                bidirectional: false,
                head: ArrowHead::Async,
            },
        ),
        (
            "-)",
            ArrowSpec {
                dashed: false,
                bidirectional: false,
                head: ArrowHead::Async,
            },
        ),
    ];

    // Match before any `:` (text separator) so arrows inside message text aren't matched.
    let head_part = match line.find(':') {
        Some(pos) => &line[..pos],
        None => line,
    };
    let mut best: Option<(usize, usize, ArrowSpec)> = None;
    for &(pat, spec) in ARROWS {
        if let Some(pos) = head_part.find(pat) {
            let candidate = (pos, pat.len(), spec);
            let replace = match best {
                None => true,
                Some((bpos, blen, _)) => pos < bpos || (pos == bpos && pat.len() > blen),
            };
            if replace {
                best = Some(candidate);
            }
        }
    }
    best
}

#[derive(Clone, Copy)]
enum OpenFrame {
    Box(usize),
    Block(BlockKind, usize),
}

fn parse_sequence(lines: &[SourceLine]) -> Result<ParsedSequence> {
    let mut seq = ParsedSequence::new();
    let mut frames: Vec<OpenFrame> = Vec::new();
    let mut current_box: Option<usize> = None;
    let mut in_acc_descr_block = false;

    for sl in lines {
        let text = sl.text.trim();
        if text.is_empty() {
            continue;
        }
        if in_acc_descr_block {
            if text.contains('}') {
                in_acc_descr_block = false;
            }
            continue;
        }
        let lower = text.to_ascii_lowercase();
        if lower.starts_with("acctitle")
            || lower.starts_with("link ")
            || lower.starts_with("links ")
            || lower.starts_with("properties ")
        {
            continue;
        }
        if lower.starts_with("accdescr") {
            if text.contains('{') && !text.contains('}') {
                in_acc_descr_block = true;
            }
            continue;
        }

        if text.eq_ignore_ascii_case("autonumber") {
            seq.autonumber = true;
            continue;
        }
        if let Some(rest) = text.strip_prefix("title")
            && (rest.is_empty() || rest.starts_with(char::is_whitespace) || rest.starts_with(':'))
        {
            let t = rest.trim().trim_start_matches(':').trim();
            if !t.is_empty() {
                seq.title = Some(t.to_string());
            }
            continue;
        }

        if text == "end" {
            match frames.pop() {
                Some(OpenFrame::Box(_)) => {
                    current_box = None;
                }
                Some(OpenFrame::Block(_, _)) => {
                    seq.events.push(SeqEvent::BlockEnd);
                }
                None => {
                    return Err(Error::Mermaid {
                        line: Some(sl.line_no),
                        message: "unexpected `end` in sequenceDiagram".to_string(),
                    });
                }
            }
            continue;
        }

        if let Some(rest) = text.strip_prefix("box")
            && (rest.is_empty() || rest.starts_with(char::is_whitespace))
        {
            let (color, title) = parse_box_header(rest);
            let box_idx = seq.boxes.len();
            seq.boxes.push(SeqBox {
                title,
                color,
                members: Vec::new(),
            });
            current_box = Some(box_idx);
            frames.push(OpenFrame::Box(sl.line_no));
            continue;
        }

        // `create participant A as Alice` / `create actor U`
        let (is_create, decl_text) = if let Some(rest) = text.strip_prefix("create")
            && rest.starts_with(char::is_whitespace)
        {
            (true, rest.trim())
        } else {
            (false, text)
        };

        if let Some(rest) = decl_text.strip_prefix("participant")
            && rest.starts_with(char::is_whitespace)
        {
            let (id, label) = parse_participant_decl(rest, sl.line_no)?;
            let idx = seq.ensure_participant(
                &id,
                label.as_deref(),
                Some(ParticipantKind::Participant),
                current_box,
            );
            if is_create {
                seq.participants[idx].created = true;
                seq.events.push(SeqEvent::Create(idx));
            }
            continue;
        }
        if let Some(rest) = decl_text.strip_prefix("actor")
            && rest.starts_with(char::is_whitespace)
        {
            let (id, label) = parse_participant_decl(rest, sl.line_no)?;
            let idx = seq.ensure_participant(
                &id,
                label.as_deref(),
                Some(ParticipantKind::Actor),
                current_box,
            );
            if is_create {
                seq.participants[idx].created = true;
                seq.events.push(SeqEvent::Create(idx));
            }
            continue;
        }

        if let Some(rest) = text.strip_prefix("destroy")
            && rest.starts_with(char::is_whitespace)
        {
            let id = rest.trim();
            let idx = seq.ensure_participant(id, None, None, None);
            seq.events.push(SeqEvent::Destroy(idx));
            continue;
        }

        if let Some(rest) = text.strip_prefix("activate")
            && rest.starts_with(char::is_whitespace)
        {
            let id = rest.trim();
            let idx = seq.ensure_participant(id, None, None, None);
            seq.events.push(SeqEvent::Activate(idx));
            continue;
        }
        if let Some(rest) = text.strip_prefix("deactivate")
            && rest.starts_with(char::is_whitespace)
        {
            let id = rest.trim();
            let idx = seq.ensure_participant(id, None, None, None);
            seq.events.push(SeqEvent::Deactivate(idx));
            continue;
        }

        // Notes: `Note left of A: ...`, `Note right of A: ...`, `Note over A[,B]: ...`
        if lower.starts_with("note ") {
            let after_note = text[4..].trim();
            let Some((placement, note_body)) = after_note.split_once(':') else {
                return Err(Error::Mermaid {
                    line: Some(sl.line_no),
                    message: "expected `:` in Note statement".to_string(),
                });
            };
            let p_trim = placement.trim();
            let p_lower = p_trim.to_ascii_lowercase();
            let (pos, target_str) = if p_lower.starts_with("left of ") {
                (NotePos::LeftOf, p_trim[8..].trim())
            } else if p_lower.starts_with("right of ") {
                (NotePos::RightOf, p_trim[9..].trim())
            } else if p_lower.starts_with("over ") {
                (NotePos::Over, p_trim[5..].trim())
            } else {
                return Err(Error::Mermaid {
                    line: Some(sl.line_no),
                    message: format!("unsupported Note placement `{p_trim}`"),
                });
            };
            let (from_id, to_id) = match target_str.split_once(',') {
                Some((a, b)) => (a.trim(), b.trim()),
                None => (target_str, target_str),
            };
            let from = seq.ensure_participant(from_id, None, None, None);
            let to = seq.ensure_participant(to_id, None, None, None);
            seq.events.push(SeqEvent::Note {
                pos,
                from,
                to,
                text: note_body.trim().to_string(),
            });
            continue;
        }

        // Control blocks & dividers.
        let mut matched_block = false;
        for (kw, kind) in [
            ("alt", BlockKind::Alt),
            ("opt", BlockKind::Opt),
            ("loop", BlockKind::Loop),
            ("par", BlockKind::Par),
            ("critical", BlockKind::Critical),
            ("break", BlockKind::Break),
            ("rect", BlockKind::Rect),
        ] {
            if let Some(rest) = text.strip_prefix(kw)
                && (rest.is_empty() || rest.starts_with(char::is_whitespace))
            {
                let rest = rest.trim();
                let rect_color = if kind == BlockKind::Rect {
                    parse_css_color(rest)
                } else {
                    None
                };
                let label = if kind == BlockKind::Rect {
                    String::new()
                } else {
                    rest.to_string()
                };
                frames.push(OpenFrame::Block(kind, sl.line_no));
                seq.events.push(SeqEvent::BlockStart {
                    kind,
                    label,
                    rect_color,
                });
                matched_block = true;
                break;
            }
        }
        if matched_block {
            continue;
        }

        for div_kw in ["else", "and", "option"] {
            if let Some(rest) = text.strip_prefix(div_kw)
                && (rest.is_empty() || rest.starts_with(char::is_whitespace))
            {
                let in_block = frames
                    .iter()
                    .rev()
                    .any(|f| matches!(f, OpenFrame::Block(_, _)));
                if !in_block {
                    return Err(Error::Mermaid {
                        line: Some(sl.line_no),
                        message: format!("unexpected `{div_kw}` outside control block"),
                    });
                }
                seq.events.push(SeqEvent::BlockDivider {
                    label: rest.trim().to_string(),
                });
                matched_block = true;
                break;
            }
        }
        if matched_block {
            continue;
        }

        // Messages: `<from> <arrow>[+|-] <to>: <text>`
        if let Some((pos, len, arrow)) = find_message_arrow(text) {
            let from_id = text[..pos].trim();
            if from_id.is_empty() {
                return Err(Error::Mermaid {
                    line: Some(sl.line_no),
                    message: "missing sender participant in message".to_string(),
                });
            }
            let mut after = &text[pos + len..];
            let mut activate_to = false;
            let mut deactivate_from = false;
            if let Some(rest) = after.strip_prefix('+') {
                activate_to = true;
                after = rest;
            } else if let Some(rest) = after.strip_prefix('-') {
                deactivate_from = true;
                after = rest;
            }
            let (to_raw, msg_text) = match after.split_once(':') {
                Some((t, m)) => (t.trim(), m.trim()),
                None => (after.trim(), ""),
            };
            if to_raw.is_empty() {
                return Err(Error::Mermaid {
                    line: Some(sl.line_no),
                    message: "missing receiver participant in message".to_string(),
                });
            }
            let from = seq.ensure_participant(from_id, None, None, None);
            let to = seq.ensure_participant(to_raw, None, None, None);
            seq.events.push(SeqEvent::Message {
                from,
                to,
                arrow,
                text: msg_text.to_string(),
                activate_to,
                deactivate_from,
            });
            continue;
        }

        return Err(Error::Mermaid {
            line: Some(sl.line_no),
            message: format!("unrecognized sequenceDiagram statement `{text}`"),
        });
    }

    if let Some(frame) = frames.last() {
        let (what, line_no) = match frame {
            OpenFrame::Box(l) => ("box", *l),
            OpenFrame::Block(kind, l) => (kind.keyword(), *l),
        };
        return Err(Error::Mermaid {
            line: Some(line_no),
            message: format!("unclosed `{what}` block (missing `end`)"),
        });
    }

    Ok(seq)
}

#[derive(Clone, Debug)]
struct ActivationSpan {
    participant: usize,
    level: usize,
    start_y: f32,
    end_y: f32,
}

#[derive(Clone, Debug)]
struct LaidOutBlock {
    kind: BlockKind,
    label: String,
    rect_color: Option<Color>,
    depth: usize,
    left: f32,
    right: f32,
    top: f32,
    bottom: f32,
    dividers: Vec<(f32, String)>,
}

enum PositionedEvent {
    Message {
        from: usize,
        to: usize,
        arrow: ArrowSpec,
        y: f32,
        text: FormattedText,
        created_target: Option<usize>,
    },
    Note {
        pos: NotePos,
        from: usize,
        to: usize,
        top_y: f32,
        height: f32,
        text: FormattedText,
    },
    Destroy {
        participant: usize,
        y: f32,
    },
}

fn draw_actor_icon(c: &mut Canvas, cx: f32, top_y: f32, fill: Color, stroke: Color) {
    let head_r = 9.0;
    let head_cy = top_y + head_r + 2.0;
    c.circle(Circle {
        cx,
        cy: head_cy,
        r: head_r,
        fill: Some(fill),
        stroke_color: Some(stroke),
        stroke_width: 1.6,
        ..Default::default()
    });
    let neck_y = head_cy + head_r;
    let hip_y = neck_y + 15.0;
    let arm_y = neck_y + 5.0;
    let foot_y = hip_y + 12.0;
    for (x1, y1, x2, y2) in [
        (cx, neck_y, cx, hip_y),
        (cx - 12.0, arm_y, cx + 12.0, arm_y),
        (cx, hip_y, cx - 10.0, foot_y),
        (cx, hip_y, cx + 10.0, foot_y),
    ] {
        c.line(Line {
            color: Some(stroke),
            stroke_width: 1.6,
            left: x1,
            top: y1,
            right: x2,
            bottom: y2,
            ..Default::default()
        });
    }
}

fn draw_participant_box(
    c: &mut Canvas,
    p: &SeqParticipant,
    ft: &FormattedText,
    pos: (f32, f32),
    size: (f32, f32),
    palette: &MermaidPalette,
    font: (&str, f32),
) {
    let (cx, top_y) = pos;
    let (box_w, box_h) = size;
    let (font_family, font_size) = font;
    match p.kind {
        ParticipantKind::Participant => {
            c.rect(Rect {
                left: cx - box_w * 0.5,
                top: top_y,
                width: box_w,
                height: box_h,
                rx: Some(4.0),
                ry: Some(4.0),
                fill: Some(palette.node_fill.into()),
                color: Some(palette.node_stroke),
                stroke_width: Some(1.6),
                ..Default::default()
            });
            draw_multiline_text(
                c,
                cx,
                top_y + box_h * 0.5,
                ft,
                font_family,
                font_size,
                palette.text_color,
                "middle",
                false,
            );
        }
        ParticipantKind::Actor => {
            draw_actor_icon(c, cx, top_y, palette.node_fill, palette.node_stroke);
            draw_multiline_text(
                c,
                cx,
                top_y + 48.0 + ft.height * 0.5,
                ft,
                font_family,
                font_size,
                palette.text_color,
                "middle",
                false,
            );
        }
    }
}

fn draw_seq_arrow_head(
    c: &mut Canvas,
    tip_x: f32,
    y: f32,
    dir_x: f32, // +1.0 pointing right, -1.0 pointing left
    head: ArrowHead,
    color: Color,
) {
    let len = 9.0;
    let half_h = 4.5;
    let bx = tip_x - dir_x * len;
    match head {
        ArrowHead::Filled => {
            c.polygon(Polygon {
                points: vec![
                    Point::from((tip_x, y)),
                    Point::from((bx, y - half_h)),
                    Point::from((bx, y + half_h)),
                ],
                fill: Some(color),
                color: None,
                ..Default::default()
            });
        }
        ArrowHead::Open => {
            for dy in [-half_h, half_h] {
                c.line(Line {
                    color: Some(color),
                    stroke_width: 1.6,
                    left: bx,
                    top: y + dy,
                    right: tip_x,
                    bottom: y,
                    ..Default::default()
                });
            }
        }
        ArrowHead::Cross => {
            let cx = tip_x - dir_x * 5.0;
            let r = 4.5;
            for (dy1, dy2) in [(-r, r), (r, -r)] {
                c.line(Line {
                    color: Some(color),
                    stroke_width: 1.8,
                    left: cx - r,
                    top: y + dy1,
                    right: cx + r,
                    bottom: y + dy2,
                    ..Default::default()
                });
            }
        }
        ArrowHead::Async => {
            c.line(Line {
                color: Some(color),
                stroke_width: 1.6,
                left: bx,
                top: y - half_h,
                right: tip_x,
                bottom: y,
                ..Default::default()
            });
        }
    }
}

pub(crate) fn render(
    chart: &MermaidChart,
    theme_name: &str,
    fm_title: Option<&str>,
    lines: &[SourceLine],
) -> Result<(String, f32, f32)> {
    let seq = parse_sequence(lines)?;
    if seq.participants.is_empty() {
        return Err(Error::Mermaid {
            line: lines.first().map(|l| l.line_no),
            message: "sequenceDiagram has no participants or messages".to_string(),
        });
    }

    let theme = get_theme(theme_name);
    let bg_override = chart
        .has_custom_background()
        .then_some(chart.base.background_color);
    let palette = MermaidPalette::from_theme(&theme, bg_override);
    let font_family = chart.resolved_font_family(&theme);
    let font_size = 13.5_f32;
    let msg_font_size = 12.5_f32;
    let note_font_size = 12.0_f32;

    let n = seq.participants.len();
    let has_actor = seq
        .participants
        .iter()
        .any(|p| p.kind == ParticipantKind::Actor);

    let p_labels: Vec<FormattedText> = seq
        .participants
        .iter()
        .map(|p| format_and_measure(&p.label, &font_family, font_size))
        .collect();
    let p_widths: Vec<f32> = p_labels
        .iter()
        .map(|ft| (ft.width + PARTICIPANT_PAD_X * 2.0).max(PARTICIPANT_MIN_W))
        .collect();
    let max_label_h = p_labels.iter().map(|ft| ft.height).fold(18.0_f32, f32::max);
    let box_h = (max_label_h + 18.0).max(38.0);
    let participant_slot_h = if has_actor {
        (48.0 + max_label_h + 6.0).max(box_h)
    } else {
        box_h
    };

    // Pre-measure event texts and compute column distances.
    let mut dists: Vec<f32> = (0..n.saturating_sub(1))
        .map(|i| (p_widths[i] + p_widths[i + 1]) * 0.5 + BASE_COL_GAP)
        .collect();

    let mut msg_counter = 0usize;
    let mut measured_events: Vec<Option<FormattedText>> = Vec::with_capacity(seq.events.len());
    let mut left_overhang = p_widths[0] * 0.5;
    let mut right_overhang = p_widths[n - 1] * 0.5;
    let mut max_depth = 0usize;
    let mut cur_depth = 0usize;

    for ev in &seq.events {
        match ev {
            SeqEvent::BlockStart { .. } => {
                cur_depth += 1;
                max_depth = max_depth.max(cur_depth);
                measured_events.push(None);
            }
            SeqEvent::BlockEnd => {
                cur_depth = cur_depth.saturating_sub(1);
                measured_events.push(None);
            }
            SeqEvent::Message { from, to, text, .. } => {
                let num = if seq.autonumber {
                    msg_counter += 1;
                    Some(msg_counter)
                } else {
                    None
                };
                let full_text = match num {
                    Some(k) if !text.is_empty() => format!("{k}. {text}"),
                    Some(k) => format!("{k}"),
                    None => text.clone(),
                };
                let ft = format_and_measure(&full_text, &font_family, msg_font_size);
                if from != to {
                    let lo = (*from).min(*to);
                    let hi = (*from).max(*to);
                    let current: f32 = dists[lo..hi].iter().sum();
                    let create_extra = if seq.participants[*to].created {
                        p_widths[*to] * 0.5
                    } else {
                        0.0
                    };
                    let needed = ft.width + 44.0 + create_extra;
                    if needed > current {
                        let extra = (needed - current) / (hi - lo) as f32;
                        for d in &mut dists[lo..hi] {
                            *d += extra;
                        }
                    }
                } else if *from == n - 1 {
                    right_overhang = right_overhang.max(SELF_LOOP_W + ft.width + 20.0);
                } else {
                    let needed = SELF_LOOP_W + ft.width + 28.0;
                    if dists[*from] < needed {
                        dists[*from] = needed;
                    }
                }
                measured_events.push(Some(ft));
            }
            SeqEvent::Note {
                pos,
                from,
                to,
                text,
            } => {
                let ft = format_and_measure(text, &font_family, note_font_size);
                let note_w = (ft.width + 24.0).max(60.0);
                match pos {
                    NotePos::LeftOf => {
                        if *from == 0 {
                            left_overhang = left_overhang.max(note_w + 24.0);
                        } else if dists[*from - 1] < note_w + 30.0 {
                            dists[*from - 1] = note_w + 30.0;
                        }
                    }
                    NotePos::RightOf => {
                        if *from == n - 1 {
                            right_overhang = right_overhang.max(note_w + 24.0);
                        } else if dists[*from] < note_w + 30.0 {
                            dists[*from] = note_w + 30.0;
                        }
                    }
                    NotePos::Over => {
                        let lo = (*from).min(*to);
                        let hi = (*from).max(*to);
                        if lo == hi {
                            if lo == 0 {
                                left_overhang = left_overhang.max(note_w * 0.5);
                            }
                            if lo == n - 1 {
                                right_overhang = right_overhang.max(note_w * 0.5);
                            }
                        } else {
                            let span: f32 = dists[lo..hi].iter().sum();
                            if span + 40.0 < note_w {
                                let extra = (note_w - span - 40.0) / (hi - lo) as f32;
                                for d in &mut dists[lo..hi] {
                                    *d += extra;
                                }
                            }
                        }
                    }
                }
                measured_events.push(Some(ft));
            }
            _ => measured_events.push(None),
        }
    }

    let box_header_pad = if seq.boxes.iter().any(|b| !b.title.is_empty()) {
        26.0
    } else if !seq.boxes.is_empty() {
        10.0
    } else {
        0.0
    };
    let block_pad = max_depth as f32 * BLOCK_INSET_X + if max_depth > 0 { 12.0 } else { 0.0 };
    left_overhang += block_pad;
    right_overhang += block_pad;

    let mut col_x = vec![0.0_f32; n];
    col_x[0] = CANVAS_PAD + left_overhang;
    for i in 0..dists.len() {
        col_x[i + 1] = col_x[i] + dists[i];
    }
    let content_w_inner = col_x[n - 1] + right_overhang + CANVAS_PAD;

    // Row Y layout.
    let top_participant_y = CANVAS_PAD + box_header_pad;
    let lifeline_top = top_participant_y + participant_slot_h;
    let mut y = lifeline_top + 22.0;

    let mut act_stacks: Vec<Vec<f32>> = vec![Vec::new(); n];
    let mut act_spans: Vec<ActivationSpan> = Vec::new();
    let mut active_blocks: Vec<LaidOutBlock> = Vec::new();
    let mut finished_blocks: Vec<LaidOutBlock> = Vec::new();
    let mut positioned_events: Vec<PositionedEvent> = Vec::new();
    let mut created_box_top: Vec<Option<f32>> = vec![None; n];
    let mut lifeline_start: Vec<f32> = vec![lifeline_top; n];
    let mut destroyed_y: Vec<Option<f32>> = vec![None; n];
    let mut pending_create: Option<usize> = None;
    let mut pending_destroy: Option<usize> = None;

    let participant_h_of = |p_idx: usize| -> f32 {
        if seq.participants[p_idx].kind == ParticipantKind::Actor {
            participant_slot_h
        } else {
            box_h
        }
    };

    let start_act = |p: usize, start_y: f32, stacks: &mut Vec<Vec<f32>>| {
        stacks[p].push(start_y);
    };
    let end_act =
        |p: usize, end_y: f32, stacks: &mut Vec<Vec<f32>>, spans: &mut Vec<ActivationSpan>| {
            if let Some(start_y) = stacks[p].pop() {
                spans.push(ActivationSpan {
                    participant: p,
                    level: stacks[p].len(),
                    start_y,
                    end_y: end_y.max(start_y + 10.0),
                });
            }
        };

    for (ev, ft_opt) in seq.events.iter().zip(measured_events) {
        match ev {
            SeqEvent::BlockStart {
                kind,
                label,
                rect_color,
            } => {
                let depth = active_blocks.len();
                let inset = depth as f32 * BLOCK_INSET_X;
                let left = CANVAS_PAD + inset;
                let right = content_w_inner - CANVAS_PAD - inset;
                active_blocks.push(LaidOutBlock {
                    kind: *kind,
                    label: label.clone(),
                    rect_color: *rect_color,
                    depth,
                    left,
                    right,
                    top: y,
                    bottom: y,
                    dividers: Vec::new(),
                });
                y += if *kind == BlockKind::Rect {
                    12.0
                } else {
                    BLOCK_HEADER_H + 10.0
                };
            }
            SeqEvent::BlockDivider { label } => {
                if let Some(b) = active_blocks.last_mut() {
                    b.dividers.push((y, label.clone()));
                }
                y += BLOCK_HEADER_H + 8.0;
            }
            SeqEvent::BlockEnd => {
                y += 10.0;
                if let Some(mut b) = active_blocks.pop() {
                    b.bottom = y;
                    finished_blocks.push(b);
                }
                y += 12.0;
            }
            SeqEvent::Activate(p) => {
                start_act(*p, y, &mut act_stacks);
            }
            SeqEvent::Deactivate(p) => {
                end_act(*p, y, &mut act_stacks, &mut act_spans);
            }
            SeqEvent::Create(p) => {
                if let Some(prev) = pending_create.replace(*p) {
                    let ph = participant_h_of(prev);
                    created_box_top[prev] = Some(y);
                    lifeline_start[prev] = y + ph;
                    y += ph + 14.0;
                }
            }
            SeqEvent::Destroy(p) => {
                if let Some(prev) = pending_destroy.replace(*p) {
                    let dy = y + 6.0;
                    destroyed_y[prev] = Some(dy);
                    positioned_events.push(PositionedEvent::Destroy {
                        participant: prev,
                        y: dy,
                    });
                    y += 14.0;
                }
            }
            SeqEvent::Message {
                from,
                to,
                arrow,
                activate_to,
                deactivate_from,
                ..
            } => {
                let ft =
                    ft_opt.unwrap_or_else(|| format_and_measure("", &font_family, msg_font_size));
                let creating = pending_create.take();
                let created_target = creating.filter(|&cp| cp == *to && from != to);
                if from == to {
                    let msg_y = if let Some(cp) = creating {
                        let cp_h = participant_h_of(cp);
                        let my = (y + 8.0).max(y + cp_h * 0.5);
                        let btop = my - cp_h * 0.5;
                        created_box_top[cp] = Some(btop);
                        lifeline_start[cp] = btop + cp_h;
                        my
                    } else {
                        y + 8.0
                    };
                    let loop_h = (ft.height + 10.0).max(SELF_LOOP_H);
                    if *activate_to {
                        start_act(*to, msg_y, &mut act_stacks);
                    }
                    if *deactivate_from {
                        end_act(*from, msg_y + loop_h, &mut act_stacks, &mut act_spans);
                    }
                    positioned_events.push(PositionedEvent::Message {
                        from: *from,
                        to: *to,
                        arrow: *arrow,
                        y: msg_y,
                        text: ft,
                        created_target: None,
                    });
                    if let Some(dp) = pending_destroy.take() {
                        let dy = msg_y + loop_h;
                        destroyed_y[dp] = Some(dy);
                        positioned_events.push(PositionedEvent::Destroy {
                            participant: dp,
                            y: dy,
                        });
                    }
                    let mut next_y = msg_y + loop_h + 18.0;
                    if let Some(cp) = creating
                        && let Some(btop) = created_box_top[cp]
                    {
                        next_y = next_y.max(btop + participant_h_of(cp) + 14.0);
                    }
                    y = next_y;
                } else {
                    let text_h = if ft.lines.iter().all(|l| l.text.is_empty()) {
                        4.0
                    } else {
                        ft.height + 6.0
                    };
                    let msg_y = if let Some(cp) = creating {
                        let cp_h = participant_h_of(cp);
                        let my = (y + text_h + 6.0).max(y + cp_h * 0.5);
                        let btop = my - cp_h * 0.5;
                        created_box_top[cp] = Some(btop);
                        lifeline_start[cp] = btop + cp_h;
                        my
                    } else {
                        y + text_h + 6.0
                    };
                    if *activate_to {
                        start_act(*to, msg_y, &mut act_stacks);
                    }
                    if *deactivate_from {
                        end_act(*from, msg_y, &mut act_stacks, &mut act_spans);
                    }
                    positioned_events.push(PositionedEvent::Message {
                        from: *from,
                        to: *to,
                        arrow: *arrow,
                        y: msg_y,
                        text: ft,
                        created_target,
                    });
                    if let Some(dp) = pending_destroy.take() {
                        destroyed_y[dp] = Some(msg_y);
                        positioned_events.push(PositionedEvent::Destroy {
                            participant: dp,
                            y: msg_y,
                        });
                    }
                    let mut next_y = msg_y + 18.0;
                    if let Some(cp) = creating
                        && let Some(btop) = created_box_top[cp]
                    {
                        next_y = next_y.max(btop + participant_h_of(cp) + 14.0);
                    }
                    y = next_y;
                }
            }
            SeqEvent::Note { pos, from, to, .. } => {
                let ft =
                    ft_opt.unwrap_or_else(|| format_and_measure("", &font_family, note_font_size));
                let note_h = (ft.height + 14.0).max(30.0);
                positioned_events.push(PositionedEvent::Note {
                    pos: *pos,
                    from: *from,
                    to: *to,
                    top_y: y,
                    height: note_h,
                    text: ft,
                });
                y += note_h + 16.0;
            }
        }
    }

    // Flush any trailing `create` or `destroy` not followed by a message.
    if let Some(cp) = pending_create.take() {
        let ph = participant_h_of(cp);
        created_box_top[cp] = Some(y);
        lifeline_start[cp] = y + ph;
        y += ph + 14.0;
    }
    if let Some(dp) = pending_destroy.take() {
        let dy = y + 6.0;
        destroyed_y[dp] = Some(dy);
        positioned_events.push(PositionedEvent::Destroy {
            participant: dp,
            y: dy,
        });
        y += 14.0;
    }

    let lifeline_bottom = y + 10.0;
    for p in 0..n {
        while !act_stacks[p].is_empty() {
            end_act(p, lifeline_bottom - 6.0, &mut act_stacks, &mut act_spans);
        }
    }

    let layout_h = lifeline_bottom + participant_slot_h + CANVAS_PAD;
    let layout_w = content_w_inner.max(120.0);

    let title_text = if !chart.base.title.text.is_empty() {
        Some(chart.base.title.text.clone())
    } else {
        seq.title.clone().or_else(|| fm_title.map(str::to_string))
    };
    let header_h = super::compute_header_height(chart, title_text.as_deref(), &theme);
    let content_w = (layout_w + chart.base.margin.left + chart.base.margin.right).max(1.0);
    let content_h =
        (layout_h + header_h + chart.base.margin.top + chart.base.margin.bottom).max(1.0);

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

    let mut content_canvas = canvas.child(super::super::util::Box {
        left: chart.base.margin.left,
        top: chart.base.margin.top + header_h,
        right: chart.base.margin.right,
        bottom: chart.base.margin.bottom,
    });

    // 1. Participant `box` groups behind everything.
    for b in &seq.boxes {
        if b.members.is_empty() {
            continue;
        }
        let mut min_x = f32::MAX;
        let mut max_x = f32::MIN;
        for &m in &b.members {
            min_x = min_x.min(col_x[m] - p_widths[m] * 0.5);
            max_x = max_x.max(col_x[m] + p_widths[m] * 0.5);
        }
        let bx = min_x - 8.0;
        let bw = (max_x - min_x) + 16.0;
        let by = top_participant_y - box_header_pad;
        let bh = lifeline_bottom + participant_slot_h + 6.0 - by;
        let fill = b.color.unwrap_or_else(|| palette.box_fill());
        content_canvas.rect(Rect {
            left: bx,
            top: by,
            width: bw,
            height: bh,
            rx: Some(4.0),
            ry: Some(4.0),
            fill: Some(fill.into()),
            color: Some(palette.subgraph_stroke),
            stroke_width: Some(1.0),
            ..Default::default()
        });
        if !b.title.is_empty() {
            let title_ft = format_and_measure(&b.title, &font_family, 12.5);
            draw_multiline_text(
                &mut content_canvas,
                bx + bw * 0.5,
                by + 13.0,
                &title_ft,
                &font_family,
                12.5,
                palette.text_color,
                "middle",
                true,
            );
        }
    }

    // 2. `rect` highlight blocks behind lifelines.
    for b in &finished_blocks {
        if b.kind != BlockKind::Rect {
            continue;
        }
        let fill = b
            .rect_color
            .unwrap_or_else(|| palette.subgraph_fill(b.depth));
        content_canvas.rect(Rect {
            left: b.left,
            top: b.top,
            width: (b.right - b.left).max(10.0),
            height: (b.bottom - b.top).max(10.0),
            fill: Some(fill.into()),
            color: None,
            ..Default::default()
        });
    }

    // 3. Dashed lifelines.
    for i in 0..n {
        let start_y = lifeline_start[i];
        let end_y = destroyed_y[i].unwrap_or(lifeline_bottom);
        if end_y > start_y {
            content_canvas.line(Line {
                color: Some(palette.edge_stroke),
                stroke_width: 1.2,
                left: col_x[i],
                top: start_y,
                right: col_x[i],
                bottom: end_y,
                stroke_dash_array: Some("5,4".to_string()),
            });
        }
    }

    // 4. Activation bars.
    for span in &act_spans {
        let cx = col_x[span.participant] + span.level as f32 * ACT_OFFSET;
        content_canvas.rect(Rect {
            left: cx - ACT_WIDTH * 0.5,
            top: span.start_y,
            width: ACT_WIDTH,
            height: span.end_y - span.start_y,
            rx: Some(2.0),
            ry: Some(2.0),
            fill: Some(palette.activation_fill.into()),
            color: Some(palette.node_stroke),
            stroke_width: Some(1.2),
            ..Default::default()
        });
    }

    // 5. Control-flow frames (`alt`, `opt`, `loop`, `par`, `critical`, `break`).
    finished_blocks.sort_by_key(|b| b.depth);
    for b in &finished_blocks {
        if b.kind == BlockKind::Rect {
            continue;
        }
        let w = (b.right - b.left).max(20.0);
        let h = (b.bottom - b.top).max(20.0);
        content_canvas.rect(Rect {
            left: b.left,
            top: b.top,
            width: w,
            height: h,
            fill: Some(Color::transparent().into()),
            color: Some(palette.subgraph_stroke),
            stroke_width: Some(1.4),
            ..Default::default()
        });

        let kw = b.kind.keyword();
        let kw_ft = format_and_measure(kw, &font_family, 12.0);
        let tab_w = (kw_ft.width + 20.0).max(46.0).min(w);
        let tab_h = 20.0;
        let fold = 7.0_f32.min(tab_w * 0.25);
        let tab_pts = vec![
            Point::from((b.left, b.top)),
            Point::from((b.left + tab_w, b.top)),
            Point::from((b.left + tab_w, b.top + tab_h - fold)),
            Point::from((b.left + tab_w - fold, b.top + tab_h)),
            Point::from((b.left, b.top + tab_h)),
        ];
        content_canvas.polygon(Polygon {
            points: tab_pts,
            fill: Some(mix_color(palette.node_fill, palette.opaque_bg, 0.25)),
            color: Some(palette.subgraph_stroke),
            ..Default::default()
        });
        draw_multiline_text(
            &mut content_canvas,
            b.left + tab_w * 0.5 - 2.0,
            b.top + tab_h * 0.5,
            &kw_ft,
            &font_family,
            12.0,
            palette.text_color,
            "middle",
            true,
        );

        if !b.label.is_empty() {
            let cond = format!("[{}]", b.label);
            let cond_ft = format_and_measure(&cond, &font_family, 12.0);
            let label_x = b.left + tab_w + 10.0;
            let label_y = b.top + tab_h * 0.5;
            content_canvas.rect(Rect {
                left: label_x - 4.0,
                top: label_y - cond_ft.height * 0.5 - 2.0,
                width: cond_ft.width + 8.0,
                height: cond_ft.height + 4.0,
                rx: Some(3.0),
                ry: Some(3.0),
                fill: Some(palette.edge_label_bg.into()),
                color: None,
                ..Default::default()
            });
            draw_multiline_text(
                &mut content_canvas,
                label_x,
                label_y,
                &cond_ft,
                &font_family,
                12.0,
                palette.text_color,
                "start",
                true,
            );
        }

        for (div_y, div_label) in &b.dividers {
            content_canvas.line(Line {
                color: Some(palette.subgraph_stroke),
                stroke_width: 1.2,
                left: b.left,
                top: *div_y,
                right: b.right,
                bottom: *div_y,
                stroke_dash_array: Some("5,4".to_string()),
            });
            if !div_label.is_empty() {
                let cond = format!("[{div_label}]");
                let div_ft = format_and_measure(&cond, &font_family, 12.0);
                let label_x = b.left + 12.0;
                let label_y = *div_y + 12.0;
                content_canvas.rect(Rect {
                    left: label_x - 4.0,
                    top: label_y - div_ft.height * 0.5 - 2.0,
                    width: div_ft.width + 8.0,
                    height: div_ft.height + 4.0,
                    rx: Some(3.0),
                    ry: Some(3.0),
                    fill: Some(palette.edge_label_bg.into()),
                    color: None,
                    ..Default::default()
                });
                draw_multiline_text(
                    &mut content_canvas,
                    label_x,
                    label_y,
                    &div_ft,
                    &font_family,
                    12.0,
                    palette.text_color,
                    "start",
                    true,
                );
            }
        }
    }

    // 6. Messages and destroy markers.
    for pe in &positioned_events {
        match pe {
            PositionedEvent::Destroy { participant, y } => {
                let cx = col_x[*participant];
                let r = 8.0;
                for (dx, dy) in [(-r, -r), (-r, r)] {
                    content_canvas.line(Line {
                        color: Some(palette.node_stroke),
                        stroke_width: 2.2,
                        left: cx + dx,
                        top: *y + dy,
                        right: cx - dx,
                        bottom: *y - dy,
                        ..Default::default()
                    });
                }
            }
            PositionedEvent::Message {
                from,
                to,
                arrow,
                y,
                text,
                created_target,
            } => {
                let dash = arrow.dashed.then(|| "5,4".to_string());
                if from == to {
                    let x = col_x[*from];
                    let loop_h = (text.height + 10.0).max(SELF_LOOP_H);
                    let xs = format_float(x);
                    let x2s = format_float(x + SELF_LOOP_W);
                    let y1s = format_float(*y);
                    let y2s = format_float(*y + loop_h);
                    let d = format!("M{xs} {y1s} L{x2s} {y1s} L{x2s} {y2s} L{xs} {y2s}");
                    content_canvas.raw_path(RawPath {
                        d,
                        fill: None,
                        stroke: Some(palette.edge_stroke),
                        stroke_width: 1.6,
                        stroke_dash_array: dash,
                        stroke_linejoin: Some("round"),
                        ..Default::default()
                    });
                    draw_seq_arrow_head(
                        &mut content_canvas,
                        x,
                        *y + loop_h,
                        -1.0,
                        arrow.head,
                        palette.edge_stroke,
                    );
                    draw_multiline_text(
                        &mut content_canvas,
                        x + SELF_LOOP_W + 8.0,
                        *y + loop_h * 0.5,
                        text,
                        &font_family,
                        msg_font_size,
                        palette.text_color,
                        "start",
                        false,
                    );
                } else {
                    let x1 = col_x[*from];
                    let mut x2 = col_x[*to];
                    let dir = if x2 >= x1 { 1.0 } else { -1.0 };
                    if *created_target == Some(*to) {
                        x2 -= dir * (p_widths[*to] * 0.5);
                    }
                    content_canvas.line(Line {
                        color: Some(palette.edge_stroke),
                        stroke_width: 1.6,
                        left: x1,
                        top: *y,
                        right: x2,
                        bottom: *y,
                        stroke_dash_array: dash,
                    });
                    draw_seq_arrow_head(
                        &mut content_canvas,
                        x2,
                        *y,
                        dir,
                        arrow.head,
                        palette.edge_stroke,
                    );
                    if arrow.bidirectional {
                        draw_seq_arrow_head(
                            &mut content_canvas,
                            x1,
                            *y,
                            -dir,
                            arrow.head,
                            palette.edge_stroke,
                        );
                    }
                    let label_cy = *y - text.height * 0.5 - 5.0;
                    draw_multiline_text(
                        &mut content_canvas,
                        (x1 + x2) * 0.5,
                        label_cy,
                        text,
                        &font_family,
                        msg_font_size,
                        palette.text_color,
                        "middle",
                        false,
                    );
                }
            }
            PositionedEvent::Note { .. } => {}
        }
    }

    // 7. Top, inline-created & bottom participant boxes / actors.
    for (i, p) in seq.participants.iter().enumerate() {
        let header_y = created_box_top[i].unwrap_or(top_participant_y);
        draw_participant_box(
            &mut content_canvas,
            p,
            &p_labels[i],
            (col_x[i], header_y),
            (p_widths[i], box_h),
            &palette,
            (&font_family, font_size),
        );
        if destroyed_y[i].is_none() {
            draw_participant_box(
                &mut content_canvas,
                p,
                &p_labels[i],
                (col_x[i], lifeline_bottom),
                (p_widths[i], box_h),
                &palette,
                (&font_family, font_size),
            );
        }
    }

    // 8. Notes on top so they stay crisp over lifelines.
    for pe in &positioned_events {
        if let PositionedEvent::Note {
            pos,
            from,
            to,
            top_y,
            height,
            text,
        } = pe
        {
            let natural_w = (text.width + 24.0).max(60.0);
            let (nx, nw) = match pos {
                NotePos::LeftOf => (col_x[*from] - 12.0 - natural_w, natural_w),
                NotePos::RightOf => (col_x[*from] + 12.0, natural_w),
                NotePos::Over => {
                    let lo = (*from).min(*to);
                    let hi = (*from).max(*to);
                    if lo == hi {
                        (col_x[lo] - natural_w * 0.5, natural_w)
                    } else {
                        let span_w = (col_x[hi] - col_x[lo] + 40.0).max(natural_w);
                        let mid = (col_x[lo] + col_x[hi]) * 0.5;
                        (mid - span_w * 0.5, span_w)
                    }
                }
            };
            let fold = 8.0_f32.min(nw * 0.25);
            let pts = vec![
                Point::from((nx, *top_y)),
                Point::from((nx + nw - fold, *top_y)),
                Point::from((nx + nw, *top_y + fold)),
                Point::from((nx + nw, *top_y + *height)),
                Point::from((nx, *top_y + *height)),
            ];
            content_canvas.polygon(Polygon {
                points: pts,
                fill: Some(palette.note_fill),
                color: Some(palette.note_stroke),
                ..Default::default()
            });
            // Folded corner crease lines.
            content_canvas.line(Line {
                color: Some(palette.note_stroke),
                stroke_width: 1.0,
                left: nx + nw - fold,
                top: *top_y,
                right: nx + nw - fold,
                bottom: *top_y + fold,
                ..Default::default()
            });
            content_canvas.line(Line {
                color: Some(palette.note_stroke),
                stroke_width: 1.0,
                left: nx + nw - fold,
                top: *top_y + fold,
                right: nx + nw,
                bottom: *top_y + fold,
                ..Default::default()
            });
            draw_multiline_text(
                &mut content_canvas,
                nx + nw * 0.5,
                *top_y + *height * 0.5,
                text,
                &font_family,
                note_font_size,
                palette.text_color,
                "middle",
                false,
            );
        }
    }

    Ok((canvas.svg()?, out_w, out_h))
}
