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

//! Source preprocessing: BOM/newline normalization, YAML frontmatter,
//! `%%{init:...}%%` directives, `%%` comment stripping with line tracking,
//! and first-line diagram type dispatch.

use super::super::error::{Error, Result};
use super::super::theme::{THEME_DARK, THEME_LIGHT, THEME_WALDEN};

/// Layout direction for a flowchart diagram.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Direction {
    TopBottom,
    BottomTop,
    LeftRight,
    RightLeft,
}

impl Direction {
    pub(crate) fn from_token(token: &str) -> Option<Self> {
        match token.to_ascii_uppercase().as_str() {
            "TB" | "TD" => Some(Self::TopBottom),
            "BT" => Some(Self::BottomTop),
            "LR" => Some(Self::LeftRight),
            "RL" => Some(Self::RightLeft),
            _ => None,
        }
    }
}

/// Detected Mermaid diagram type and header metadata.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) enum DiagramKind {
    Flowchart(Direction),
    Sequence,
    State,
    Class,
    Er,
    Mindmap,
    Quadrant,
    Radar,
    Treemap,
    Pie {
        show_data: bool,
        inline_title: Option<String>,
    },
    Gantt,
    Sankey,
    XyChart {
        horizontal: bool,
    },
}

/// One non-empty preprocessed logical statement line with its 1-based source line number.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct SourceLine {
    pub(crate) line_no: usize,
    pub(crate) indent: usize,
    pub(crate) text: String,
}

fn leading_indent(s: &str) -> usize {
    let mut n = 0;
    for b in s.bytes() {
        match b {
            b' ' => n += 1,
            b'\t' => n += 4,
            _ => break,
        }
    }
    n
}

/// Result of preprocessing a raw Mermaid document.
#[derive(Clone, Debug)]
pub(crate) struct Preprocessed {
    pub(crate) kind: DiagramKind,
    pub(crate) title: Option<String>,
    pub(crate) acc_title: Option<String>,
    pub(crate) acc_descr: Option<String>,
    pub(crate) init_theme: Option<String>,
    pub(crate) lines: Vec<SourceLine>,
}

fn unquote_yaml(raw: &str) -> String {
    let s = raw.trim();
    if s.len() >= 2
        && ((s.starts_with('"') && s.ends_with('"')) || (s.starts_with('\'') && s.ends_with('\'')))
    {
        s[1..s.len() - 1].to_string()
    } else {
        s.to_string()
    }
}

fn map_mermaid_theme(name: &str) -> Option<String> {
    match name.trim().to_ascii_lowercase().as_str() {
        "dark" => Some(THEME_DARK.to_string()),
        "forest" => Some(THEME_WALDEN.to_string()),
        "default" | "base" | "neutral" => Some(THEME_LIGHT.to_string()),
        _ => None,
    }
}

/// Converts single-quoted JSON-like directive strings into valid JSON so
/// `serde_json` can parse directives such as `{'theme': 'dark'}`.
fn normalize_directive_json(raw: &str) -> String {
    let mut out = String::with_capacity(raw.len());
    let chars: Vec<char> = raw.chars().collect();
    let mut i = 0;
    let mut in_double = false;
    let mut in_single = false;
    while i < chars.len() {
        let ch = chars[i];
        if in_double {
            out.push(ch);
            if ch == '\\' && i + 1 < chars.len() {
                i += 1;
                out.push(chars[i]);
            } else if ch == '"' {
                in_double = false;
            }
            i += 1;
            continue;
        }
        if in_single {
            if ch == '\\' && i + 1 < chars.len() {
                let next = chars[i + 1];
                if next == '\'' {
                    out.push('\'');
                } else {
                    out.push('\\');
                    out.push(next);
                }
                i += 2;
                continue;
            }
            if ch == '\'' {
                in_single = false;
                out.push('"');
            } else if ch == '"' {
                out.push_str("\\\"");
            } else {
                out.push(ch);
            }
            i += 1;
            continue;
        }
        match ch {
            '"' => {
                in_double = true;
                out.push('"');
            }
            '\'' => {
                in_single = true;
                out.push('"');
            }
            ',' => {
                // Drop trailing commas before `}` or `]`.
                let mut j = i + 1;
                while j < chars.len() && chars[j].is_whitespace() {
                    j += 1;
                }
                if j < chars.len() && (chars[j] == '}' || chars[j] == ']') {
                    // skip trailing comma
                } else {
                    out.push(',');
                }
            }
            _ => out.push(ch),
        }
        i += 1;
    }
    out
}

fn parse_directive(body: &str, init_theme: &mut Option<String>) {
    let trimmed = body.trim();
    // Accept `{init: {...}}`, `init: {...}`, or `{ "theme": ... }`.
    let inner = trimmed
        .strip_prefix('{')
        .and_then(|s| s.strip_suffix('}'))
        .map(str::trim)
        .unwrap_or(trimmed);
    let json_text = if let Some((k, rest)) = inner.split_once(':') {
        let key = unquote_yaml(k);
        if key.eq_ignore_ascii_case("init") {
            rest.trim()
        } else {
            trimmed
        }
    } else {
        trimmed
    };
    let normalized = normalize_directive_json(json_text);
    if let Ok(val) = serde_json::from_str::<serde_json::Value>(&normalized)
        && let Some(theme_name) = val.get("theme").and_then(|v| v.as_str()).or_else(|| {
            val.get("init")
                .and_then(|i| i.get("theme"))
                .and_then(|v| v.as_str())
        })
        && let Some(mapped) = map_mermaid_theme(theme_name)
    {
        *init_theme = Some(mapped);
    }
}

/// Tracks whether scanning `s` toggles `in_md` (inside a `` "`...`" ``
/// Mermaid Markdown string). Returns the updated `in_md` state at end-of-line.
fn scan_markdown_string_state(s: &str, mut in_md: bool) -> bool {
    let bytes = s.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if !in_md {
            if bytes[i] == b'"' && i + 1 < bytes.len() && bytes[i + 1] == b'`' {
                in_md = true;
                i += 2;
                continue;
            }
        } else if bytes[i] == b'`' && i + 1 < bytes.len() && bytes[i + 1] == b'"' {
            in_md = false;
            i += 2;
            continue;
        }
        i += 1;
    }
    in_md
}

/// Finds `%%` outside double-quoted strings, returning the byte offset if present.
fn find_comment_start(line: &str) -> Option<usize> {
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
        if b == b'%' && i + 1 < bytes.len() && bytes[i + 1] == b'%' {
            return Some(i);
        }
        i += 1;
    }
    None
}

pub(crate) fn preprocess(raw: &str) -> Result<Preprocessed> {
    let cleaned = raw
        .trim_start_matches('\u{feff}')
        .replace("\r\n", "\n")
        .replace('\r', "\n");

    let raw_lines: Vec<(usize, &str)> = cleaned
        .split('\n')
        .enumerate()
        .map(|(i, s)| (i + 1, s))
        .collect();

    let mut idx = 0;
    while idx < raw_lines.len() && raw_lines[idx].1.trim().is_empty() {
        idx += 1;
    }

    let mut fm_title: Option<String> = None;
    let mut init_theme: Option<String> = None;

    // Optional YAML frontmatter at the very top (`--- ... ---`).
    if idx < raw_lines.len() && raw_lines[idx].1.trim() == "---" {
        let start_line = raw_lines[idx].0;
        idx += 1;
        let mut closed = false;
        let mut in_config = false;
        while idx < raw_lines.len() {
            let line = raw_lines[idx].1;
            let trimmed = line.trim();
            idx += 1;
            if trimmed == "---" || trimmed == "..." {
                closed = true;
                break;
            }
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }
            let indented = line.starts_with(' ') || line.starts_with('\t');
            if !indented {
                in_config = false;
                if let Some((k, v)) = trimmed.split_once(':') {
                    let key = k.trim();
                    let val = v.trim();
                    if key.eq_ignore_ascii_case("title") && !val.is_empty() {
                        fm_title = Some(unquote_yaml(val));
                    } else if key.eq_ignore_ascii_case("config") && val.is_empty() {
                        in_config = true;
                    }
                }
            } else if in_config
                && let Some((k, v)) = trimmed.split_once(':')
                && k.trim().eq_ignore_ascii_case("theme")
                && let Some(mapped) = map_mermaid_theme(&unquote_yaml(v))
            {
                init_theme = Some(mapped);
            }
        }
        if !closed {
            return Err(Error::Mermaid {
                line: Some(start_line),
                message: "unclosed YAML frontmatter (`---`)".to_string(),
            });
        }
    }

    // Collect logical non-empty lines while extracting `%%{...}%%` directives,
    // stripping `%%` comments, and joining multiline `` "`...`" `` Markdown strings.
    let mut lines: Vec<SourceLine> = Vec::new();
    let mut in_directive = false;
    let mut directive_start_line = 0;
    let mut directive_buf = String::new();
    let mut in_md_string = false;
    let mut md_start_line = 0;
    let mut md_indent = 0;
    let mut md_buf = String::new();

    while idx < raw_lines.len() {
        let (line_no, mut slice) = raw_lines[idx];
        idx += 1;

        if in_md_string {
            md_buf.push('\n');
            md_buf.push_str(slice.trim());
            if !scan_markdown_string_state(slice, true) {
                lines.push(SourceLine {
                    line_no: md_start_line,
                    indent: md_indent,
                    text: std::mem::take(&mut md_buf),
                });
                in_md_string = false;
            }
            continue;
        }

        loop {
            if in_directive {
                if let Some(end) = slice.find("}%%") {
                    directive_buf.push_str(&slice[..end]);
                    parse_directive(&directive_buf, &mut init_theme);
                    directive_buf.clear();
                    in_directive = false;
                    slice = &slice[end + 3..];
                    continue;
                } else {
                    directive_buf.push_str(slice);
                    directive_buf.push('\n');
                    break;
                }
            }

            if let Some(comment_pos) = find_comment_start(slice) {
                let before = slice[..comment_pos].trim();
                let indent = leading_indent(slice);
                let comment = &slice[comment_pos..];
                if let Some(dir_rest) = comment.strip_prefix("%%{") {
                    if !before.is_empty() {
                        lines.push(SourceLine {
                            line_no,
                            indent,
                            text: before.to_string(),
                        });
                    }
                    in_directive = true;
                    directive_start_line = line_no;
                    directive_buf.clear();
                    slice = dir_rest;
                    continue;
                }
                if !before.is_empty() {
                    lines.push(SourceLine {
                        line_no,
                        indent,
                        text: before.to_string(),
                    });
                }
                break;
            }

            let indent = leading_indent(slice);
            let trimmed = slice.trim();
            if !trimmed.is_empty() {
                if scan_markdown_string_state(trimmed, false) {
                    in_md_string = true;
                    md_start_line = line_no;
                    md_indent = indent;
                    md_buf.clear();
                    md_buf.push_str(trimmed);
                } else {
                    lines.push(SourceLine {
                        line_no,
                        indent,
                        text: trimmed.to_string(),
                    });
                }
            }
            break;
        }
    }

    if in_md_string && !md_buf.is_empty() {
        lines.push(SourceLine {
            line_no: md_start_line,
            indent: md_indent,
            text: md_buf,
        });
    }

    if in_directive {
        return Err(Error::Mermaid {
            line: Some(directive_start_line),
            message: "unclosed `%%{...}%%` directive".to_string(),
        });
    }

    let Some(first) = lines.first().cloned() else {
        return Err(Error::Mermaid {
            line: None,
            message: "empty Mermaid diagram".to_string(),
        });
    };

    // Split optional `;` trailing statements on the header line (e.g. `flowchart LR; A-->B`).
    let (header_text, trailing) = match first.text.split_once(';') {
        Some((h, t)) => (h.trim(), Some(t.trim())),
        None => (first.text.as_str(), None),
    };

    let mut raw_body: Vec<SourceLine> = Vec::with_capacity(lines.len());
    if let Some(t) = trailing
        && !t.is_empty()
    {
        raw_body.push(SourceLine {
            line_no: first.line_no,
            indent: 0,
            text: t.to_string(),
        });
    }
    raw_body.extend(lines.into_iter().skip(1));

    // Extract accessibility directives (`accTitle:`, `accDescr:`, `accDescr { ... }`)
    // centrally so every diagram engine inherits SVG `<title>`/`<desc>` support.
    let (body, acc_title, acc_descr) = extract_accessibility(raw_body)?;

    let mut parts = header_text.split_whitespace();
    let keyword = parts.next().unwrap_or_default();

    let kind = match keyword.to_ascii_lowercase().as_str() {
        "flowchart" | "graph" => {
            let dir = match parts.next() {
                Some(tok) => Direction::from_token(tok).ok_or_else(|| Error::Mermaid {
                    line: Some(first.line_no),
                    message: format!(
                        "unknown flowchart direction `{tok}` (expected one of: TB, TD, BT, LR, RL)"
                    ),
                })?,
                None => Direction::TopBottom,
            };
            if let Some(extra) = parts.next() {
                return Err(Error::Mermaid {
                    line: Some(first.line_no),
                    message: format!("unexpected token `{extra}` in flowchart header"),
                });
            }
            DiagramKind::Flowchart(dir)
        }
        "sequencediagram" => DiagramKind::Sequence,
        "statediagram" | "statediagram-v2" => DiagramKind::State,
        "classdiagram" | "classdiagram-v2" => DiagramKind::Class,
        "erdiagram" => DiagramKind::Er,
        "mindmap" => DiagramKind::Mindmap,
        "quadrantchart" => DiagramKind::Quadrant,
        "radar-beta" | "radar" => DiagramKind::Radar,
        "treemap-beta" | "treemap" => DiagramKind::Treemap,
        "pie" => {
            let rest = header_text[keyword.len()..].trim();
            let (show_data, after_show) = if let Some(r) = rest.strip_prefix("showData") {
                if r.is_empty() || r.starts_with(char::is_whitespace) {
                    (true, r.trim())
                } else {
                    (false, rest)
                }
            } else {
                (false, rest)
            };
            let inline_title = if let Some(t) = after_show.strip_prefix("title") {
                if t.is_empty() || t.starts_with(char::is_whitespace) {
                    let t = t.trim();
                    (!t.is_empty()).then(|| t.to_string())
                } else {
                    return Err(Error::Mermaid {
                        line: Some(first.line_no),
                        message: format!("unexpected token `{after_show}` in pie header"),
                    });
                }
            } else if after_show.is_empty() {
                None
            } else {
                return Err(Error::Mermaid {
                    line: Some(first.line_no),
                    message: format!("unexpected token `{after_show}` in pie header"),
                });
            };
            DiagramKind::Pie {
                show_data,
                inline_title,
            }
        }
        "gantt" => DiagramKind::Gantt,
        "sankey-beta" | "sankey" => DiagramKind::Sankey,
        "xychart-beta" | "xychart" => {
            let horizontal = match parts.next() {
                Some(tok) if tok.eq_ignore_ascii_case("horizontal") => true,
                Some(tok) => {
                    return Err(Error::Mermaid {
                        line: Some(first.line_no),
                        message: format!("unexpected token `{tok}` in xychart header"),
                    });
                }
                None => false,
            };
            DiagramKind::XyChart { horizontal }
        }
        _ => {
            let hint = suggest_diagram_kind(keyword)
                .map(|s| format!(" (did you mean `{s}`?)"))
                .unwrap_or_default();
            return Err(Error::Mermaid {
                line: Some(first.line_no),
                message: format!("unsupported Mermaid diagram type `{keyword}`{hint}"),
            });
        }
    };

    Ok(Preprocessed {
        kind,
        title: fm_title,
        acc_title,
        acc_descr,
        init_theme,
        lines: body,
    })
}

fn extract_accessibility(
    lines: Vec<SourceLine>,
) -> Result<(Vec<SourceLine>, Option<String>, Option<String>)> {
    let mut out = Vec::with_capacity(lines.len());
    let mut acc_title: Option<String> = None;
    let mut acc_descr: Option<String> = None;
    let mut in_descr_block = false;
    let mut descr_start_line = 0usize;
    let mut descr_parts: Vec<String> = Vec::new();

    for sl in lines {
        let trimmed = sl.text.trim();
        if in_descr_block {
            if let Some(before_close) = trimmed.strip_suffix('}') {
                let piece = before_close.trim();
                if !piece.is_empty() {
                    descr_parts.push(piece.to_string());
                }
                let joined = descr_parts.join("\n");
                if !joined.is_empty() {
                    acc_descr = Some(joined);
                }
                descr_parts.clear();
                in_descr_block = false;
            } else if !trimmed.is_empty() {
                descr_parts.push(trimmed.to_string());
            }
            continue;
        }

        if let Some(rest) = trimmed.strip_prefix("accTitle") {
            let after = rest.trim_start();
            if let Some(val) = after.strip_prefix(':') {
                let t = val.trim();
                if !t.is_empty() {
                    acc_title = Some(t.to_string());
                }
                continue;
            }
        }

        if let Some(rest) = trimmed.strip_prefix("accDescr") {
            let after = rest.trim_start();
            if let Some(val) = after.strip_prefix(':') {
                let d = val.trim();
                if !d.is_empty() {
                    acc_descr = Some(d.to_string());
                }
                continue;
            }
            if let Some(after_brace) = after.strip_prefix('{') {
                let inner = after_brace.trim();
                if let Some(before_close) = inner.strip_suffix('}') {
                    let d = before_close.trim();
                    if !d.is_empty() {
                        acc_descr = Some(d.to_string());
                    }
                } else {
                    in_descr_block = true;
                    descr_start_line = sl.line_no;
                    descr_parts.clear();
                    if !inner.is_empty() {
                        descr_parts.push(inner.to_string());
                    }
                }
                continue;
            }
        }

        out.push(sl);
    }

    if in_descr_block {
        return Err(Error::Mermaid {
            line: Some(descr_start_line),
            message: "unclosed `accDescr { ... }` block".to_string(),
        });
    }

    Ok((out, acc_title, acc_descr))
}

fn edit_distance(a: &str, b: &str) -> usize {
    let a: Vec<char> = a.chars().collect();
    let b: Vec<char> = b.chars().collect();
    let mut prev: Vec<usize> = (0..=b.len()).collect();
    let mut curr = vec![0usize; b.len() + 1];
    for (i, &ca) in a.iter().enumerate() {
        curr[0] = i + 1;
        for (j, &cb) in b.iter().enumerate() {
            let cost = usize::from(ca != cb);
            curr[j + 1] = (prev[j + 1] + 1).min(curr[j] + 1).min(prev[j] + cost);
        }
        std::mem::swap(&mut prev, &mut curr);
    }
    prev[b.len()]
}

fn suggest_diagram_kind(keyword: &str) -> Option<&'static str> {
    const KNOWN: &[&str] = &[
        "flowchart",
        "graph",
        "sequenceDiagram",
        "stateDiagram",
        "stateDiagram-v2",
        "classDiagram",
        "classDiagram-v2",
        "erDiagram",
        "mindmap",
        "quadrantChart",
        "radar-beta",
        "treemap-beta",
        "pie",
        "gantt",
        "sankey-beta",
        "xychart-beta",
    ];
    let lower = keyword.to_ascii_lowercase();
    if lower.is_empty() {
        return None;
    }
    // Check prefix match for common shorthand (e.g. `sequence`, `quadrant`).
    if lower.len() >= 4 {
        for &cand in KNOWN {
            if cand.to_ascii_lowercase().starts_with(&lower) {
                return Some(cand);
            }
        }
    }
    let max_dist = (lower.len() / 2).clamp(1, 3);
    let mut best: Option<(&'static str, usize)> = None;
    for &cand in KNOWN {
        let d = edit_distance(&lower, &cand.to_ascii_lowercase());
        if d <= max_dist && best.is_none_or(|(_, bd)| d < bd) {
            best = Some((cand, d));
        }
    }
    best.map(|(cand, _)| cand)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_frontmatter_and_init_directive() {
        let src = r#"
---
title: "Service Flow"
config:
  theme: dark
---
%%{init: {'theme': 'forest', 'flowchart': {'curve': 'basis'}}}%%
flowchart LR; A --> B %% inline comment
"#;
        let p = preprocess(src).unwrap();
        assert_eq!(Some("Service Flow".to_string()), p.title);
        assert_eq!(Some(THEME_WALDEN.to_string()), p.init_theme);
        assert_eq!(DiagramKind::Flowchart(Direction::LeftRight), p.kind);
        assert_eq!(1, p.lines.len());
        assert_eq!("A --> B", p.lines[0].text);
        assert_eq!(8, p.lines[0].line_no);
    }

    #[test]
    fn rejects_unsupported_diagram_with_line() {
        let err = preprocess("\n\ngitGraph\n  commit").unwrap_err();
        let msg = err.to_string();
        assert!(msg.contains("line 3"), "{msg}");
        assert!(msg.contains("unsupported"), "{msg}");
    }

    #[test]
    fn detects_v2_diagram_headers() {
        assert_eq!(
            DiagramKind::State,
            preprocess("stateDiagram\n  [*] --> Idle").unwrap().kind
        );
        assert_eq!(
            DiagramKind::State,
            preprocess("stateDiagram-v2\n  [*] --> Idle").unwrap().kind
        );
        assert_eq!(
            DiagramKind::Class,
            preprocess("classDiagram\n  class Foo").unwrap().kind
        );
        assert_eq!(
            DiagramKind::Class,
            preprocess("classDiagram-v2\n  class Foo").unwrap().kind
        );
        assert_eq!(
            DiagramKind::Er,
            preprocess("erDiagram\n  A ||--o{ B : rel").unwrap().kind
        );
    }

    #[test]
    fn detects_v3_diagram_headers_and_indent() {
        let mm = preprocess("mindmap\n  root((R))\n\tChild").unwrap();
        assert_eq!(DiagramKind::Mindmap, mm.kind);
        assert_eq!(2, mm.lines[0].indent);
        assert_eq!(4, mm.lines[1].indent);

        assert_eq!(
            DiagramKind::Quadrant,
            preprocess("quadrantChart\n  x-axis Low --> High")
                .unwrap()
                .kind
        );
        assert_eq!(
            DiagramKind::Radar,
            preprocess("radar-beta\n  axis a, b").unwrap().kind
        );
        assert_eq!(
            DiagramKind::Radar,
            preprocess("radar\n  axis a, b").unwrap().kind
        );
        assert_eq!(
            DiagramKind::Treemap,
            preprocess("treemap-beta\n  \"A\": 10").unwrap().kind
        );
        assert_eq!(
            DiagramKind::Treemap,
            preprocess("treemap\n  \"A\": 10").unwrap().kind
        );
    }
}
