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

//! Pure-SVG multiline label preprocessing, entity decoding and `ttf-parser`
//! text measurement.

use super::super::canvas::Canvas;
use super::super::color::Color;
use super::super::component::{Text, TextSpan};
use super::super::font::measure_text_width_family;

/// One processed line of label text.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct FormattedLine {
    pub(crate) text: String,
    pub(crate) bold: bool,
    pub(crate) italic: bool,
    pub(crate) spans: Vec<TextSpan>,
}

/// Measured multiline text block ready for SVG emission.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct FormattedText {
    pub(crate) lines: Vec<FormattedLine>,
    pub(crate) width: f32,
    pub(crate) height: f32,
    pub(crate) line_height: f32,
}

/// Splits `<br>`, `<br/>`, `<br />`, `\n` and escaped `\\n` into raw lines.
fn split_lines(raw: &str) -> Vec<String> {
    let mut normalized = String::with_capacity(raw.len());
    let chars: Vec<char> = raw.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        if chars[i] == '\\' && i + 1 < chars.len() && chars[i + 1] == 'n' {
            normalized.push('\n');
            i += 2;
            continue;
        }
        if chars[i] == '<' {
            // Match `<br>`, `<br/>`, `<br />` case-insensitively.
            let rest: String = chars[i..].iter().take(8).collect();
            let lower = rest.to_ascii_lowercase();
            if let Some(close) = lower.find('>') {
                let inner = lower[1..close].trim().trim_end_matches('/').trim();
                if inner == "br" {
                    normalized.push('\n');
                    i += close + 1;
                    continue;
                }
            }
        }
        normalized.push(chars[i]);
        i += 1;
    }
    normalized
        .split('\n')
        .map(|s| s.trim().to_string())
        .collect()
}

fn decode_entities(raw: &str) -> String {
    let mut out = String::with_capacity(raw.len());
    let bytes = raw.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'&'
            && let Some(semi_rel) = raw[i + 1..].find(';')
            && semi_rel <= 12
        {
            let entity = &raw[i + 1..i + 1 + semi_rel];
            let replacement = match entity {
                "amp" => Some("&".to_string()),
                "lt" => Some("<".to_string()),
                "gt" => Some(">".to_string()),
                "quot" => Some("\"".to_string()),
                "apos" => Some("'".to_string()),
                "nbsp" => Some(" ".to_string()),
                _ if entity.starts_with('#') => {
                    let num = &entity[1..];
                    let code = if let Some(hex) = num.strip_prefix(['x', 'X']) {
                        u32::from_str_radix(hex, 16).ok()
                    } else {
                        num.parse::<u32>().ok()
                    };
                    code.and_then(char::from_u32).map(|c| c.to_string())
                }
                _ => None,
            };
            if let Some(rep) = replacement {
                out.push_str(&rep);
                i += semi_rel + 2;
                continue;
            }
        }
        // Mermaid's `#quot;` / `#35;` escape form.
        if bytes[i] == b'#'
            && let Some(semi_rel) = raw[i + 1..].find(';')
            && (2..=8).contains(&semi_rel)
        {
            let entity = &raw[i + 1..i + 1 + semi_rel];
            let replacement = match entity {
                "quot" => Some("\"".to_string()),
                "amp" => Some("&".to_string()),
                "lt" => Some("<".to_string()),
                "gt" => Some(">".to_string()),
                "apos" => Some("'".to_string()),
                _ if entity.bytes().all(|b| b.is_ascii_digit()) => entity
                    .parse::<u32>()
                    .ok()
                    .and_then(char::from_u32)
                    .map(|c| c.to_string()),
                _ => None,
            };
            if let Some(rep) = replacement {
                out.push_str(&rep);
                i += semi_rel + 2;
                continue;
            }
        }
        let ch = raw[i..].chars().next().unwrap_or_default();
        out.push(ch);
        i += ch.len_utf8();
    }
    out
}

fn is_inline_html_tag(tag_name: &str) -> bool {
    let bare = tag_name.strip_prefix('/').unwrap_or(tag_name);
    matches!(
        bare,
        "b" | "strong"
            | "i"
            | "em"
            | "u"
            | "s"
            | "del"
            | "ins"
            | "mark"
            | "small"
            | "sub"
            | "sup"
            | "code"
            | "span"
            | "a"
            | "font"
            | "div"
            | "p"
    )
}

fn is_fa_icon_token(tok: &str) -> bool {
    let Some((prefix, rest)) = tok.split_once(':') else {
        return false;
    };
    let valid_prefix = matches!(
        prefix.to_ascii_lowercase().as_str(),
        "fa" | "fas" | "fab" | "far" | "fal" | "fad"
    );
    if !valid_prefix {
        return false;
    }
    let lower_rest = rest.to_ascii_lowercase();
    let Some(name) = lower_rest.strip_prefix("fa-") else {
        return false;
    };
    !name.is_empty()
        && name
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
}

/// Strips Font Awesome icon tokens (`fa:fa-car`, `fab:fa-github`, etc.) from a
/// decoded label line while leaving normal text and colons untouched.
fn strip_fa_icon_tokens(text: String) -> String {
    if !text.contains(':') {
        return text;
    }
    let mut any_stripped = false;
    let mut kept = Vec::new();
    for tok in text.split_whitespace() {
        if is_fa_icon_token(tok) {
            any_stripped = true;
        } else {
            kept.push(tok);
        }
    }
    if any_stripped {
        kept.join(" ")
    } else {
        text
    }
}

/// Strips inline HTML tags (`<b>`, `<i>`, `<span ...>`) while keeping `<` in
/// expressions such as `a < b` and generic types such as `List<T>` intact.
fn strip_inline_html(line: &str, bold_state: &mut bool, italic_state: &mut bool) -> FormattedLine {
    let mut out = String::with_capacity(line.len());
    let mut line_bold = *bold_state;
    let mut line_italic = *italic_state;
    let chars: Vec<char> = line.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        if chars[i] == '<'
            && let Some(next) = chars.get(i + 1)
            && (next.is_ascii_alphabetic() || *next == '/')
            && let Some(rel_end) = chars[i + 1..].iter().position(|&c| c == '>')
        {
            let tag: String = chars[i + 1..i + 1 + rel_end].iter().collect();
            let tag_name = tag
                .split_whitespace()
                .next()
                .unwrap_or_default()
                .trim_end_matches('/')
                .to_ascii_lowercase();
            if is_inline_html_tag(&tag_name) {
                match tag_name.as_str() {
                    "b" | "strong" => {
                        *bold_state = true;
                        line_bold = true;
                    }
                    "/b" | "/strong" => {
                        *bold_state = false;
                    }
                    "i" | "em" => {
                        *italic_state = true;
                        line_italic = true;
                    }
                    "/i" | "/em" => {
                        *italic_state = false;
                    }
                    _ => {}
                }
                i += rel_end + 2;
                continue;
            }
        }
        out.push(chars[i]);
        i += 1;
    }
    FormattedLine {
        text: strip_fa_icon_tokens(decode_entities(&out)),
        bold: line_bold,
        italic: line_italic,
        spans: Vec::new(),
    }
}

/// Parses inline Markdown emphasis (`***both***`, `**bold**`, `__bold__`,
/// `*italic*`, `_italic_`) into styled [`TextSpan`]s.
fn parse_markdown_line(line: &str) -> FormattedLine {
    let cleaned = strip_fa_icon_tokens(decode_entities(line));
    let chars: Vec<char> = cleaned.chars().collect();
    let n = chars.len();
    let mut spans: Vec<TextSpan> = Vec::new();
    let mut plain = String::new();
    let mut i = 0;

    let push_span = |spans: &mut Vec<TextSpan>, text: &str, bold: bool, italic: bool| {
        if text.is_empty() {
            return;
        }
        if let Some(last) = spans.last_mut()
            && last.bold == bold
            && last.italic == italic
        {
            last.text.push_str(text);
            return;
        }
        spans.push(TextSpan {
            text: text.to_string(),
            bold,
            italic,
        });
    };

    let find_closing = |start: usize, delim: char, count: usize| -> Option<usize> {
        let mut j = start;
        while j + count <= n {
            if (0..count).all(|k| chars[j + k] == delim) {
                // For single `_`, require not followed by alphanumeric (avoid snake_case).
                if delim == '_'
                    && count == 1
                    && j + 1 < n
                    && chars[j + 1].is_alphanumeric()
                {
                    j += 1;
                    continue;
                }
                if j > start {
                    return Some(j);
                }
            }
            j += 1;
        }
        None
    };

    while i < n {
        let ch = chars[i];
        if ch == '*' || ch == '_' {
            let prev_alnum = i > 0 && chars[i - 1].is_alphanumeric();
            let mut matched = false;
            for (count, bold, italic) in [
                (3usize, true, true),
                (2usize, true, false),
                (1usize, false, true),
            ] {
                if count == 1 && ch == '_' && prev_alnum {
                    continue;
                }
                if i + count < n
                    && (0..count).all(|k| chars[i + k] == ch)
                    && !chars[i + count].is_whitespace()
                    && let Some(end) = find_closing(i + count, ch, count)
                {
                    push_span(&mut spans, &plain, false, false);
                    plain.clear();
                    let inner: String = chars[i + count..end].iter().collect();
                    push_span(&mut spans, &inner, bold, italic);
                    i = end + count;
                    matched = true;
                    break;
                }
            }
            if matched {
                continue;
            }
        }
        plain.push(ch);
        i += 1;
    }
    push_span(&mut spans, &plain, false, false);

    let full_text: String = spans.iter().map(|s| s.text.as_str()).collect();
    let has_styled = spans.iter().any(|s| s.bold || s.italic);
    if !has_styled {
        FormattedLine {
            text: full_text,
            bold: false,
            italic: false,
            spans: Vec::new(),
        }
    } else if spans.len() == 1 {
        let s = &spans[0];
        FormattedLine {
            text: full_text,
            bold: s.bold,
            italic: s.italic,
            spans: Vec::new(),
        }
    } else {
        FormattedLine {
            text: full_text,
            bold: false,
            italic: false,
            spans,
        }
    }
}

/// Cleans, wraps (`<br/>` / `\n`) and measures a Mermaid label.
pub(crate) fn format_and_measure(raw: &str, font_family: &str, font_size: f32) -> FormattedText {
    let trimmed = raw.trim();
    let (is_markdown, body) = if let Some(inner) = trimmed
        .strip_prefix('`')
        .and_then(|s| s.strip_suffix('`'))
        && trimmed.len() >= 2
    {
        (true, inner)
    } else {
        (false, raw)
    };
    let raw_lines = split_lines(body);
    let mut bold_state = false;
    let mut italic_state = false;
    let mut lines = Vec::with_capacity(raw_lines.len());
    let mut max_width = 0.0_f32;
    let line_height = (font_size * 1.35).ceil();

    for raw_line in raw_lines {
        let formatted = if is_markdown {
            parse_markdown_line(&raw_line)
        } else {
            strip_inline_html(&raw_line, &mut bold_state, &mut italic_state)
        };
        if !formatted.spans.is_empty() {
            let mut w = 0.0_f32;
            for sp in &formatted.spans {
                let mut sw = measure_text_width_family(font_family, font_size, &sp.text)
                    .map(|b| b.width())
                    .unwrap_or_else(|_| sp.text.chars().count() as f32 * font_size * 0.6);
                if sp.bold {
                    sw *= 1.05;
                }
                w += sw;
            }
            max_width = max_width.max(w);
        } else if !formatted.text.is_empty() {
            let mut w = measure_text_width_family(font_family, font_size, &formatted.text)
                .map(|b| b.width())
                .unwrap_or_else(|_| formatted.text.chars().count() as f32 * font_size * 0.6);
            if formatted.bold {
                w *= 1.05;
            }
            max_width = max_width.max(w);
        }
        lines.push(formatted);
    }

    let height = if lines.is_empty() {
        0.0
    } else {
        lines.len() as f32 * line_height
    };

    FormattedText {
        lines,
        width: max_width,
        height,
        line_height,
    }
}

/// Emits centered/aligned pure-SVG `<text>` elements for a measured [`FormattedText`].
#[allow(clippy::too_many_arguments)]
pub(crate) fn draw_multiline_text(
    c: &mut Canvas,
    x: f32,
    cy: f32,
    ft: &FormattedText,
    font_family: &str,
    font_size: f32,
    color: Color,
    anchor: &str,
    default_bold: bool,
) {
    draw_multiline_text_styled(
        c,
        x,
        cy,
        ft,
        font_family,
        font_size,
        color,
        anchor,
        default_bold,
        false,
    );
}

/// Emits centered/aligned pure-SVG `<text>` elements with optional italic default.
#[allow(clippy::too_many_arguments)]
pub(crate) fn draw_multiline_text_styled(
    c: &mut Canvas,
    x: f32,
    cy: f32,
    ft: &FormattedText,
    font_family: &str,
    font_size: f32,
    color: Color,
    anchor: &str,
    default_bold: bool,
    default_italic: bool,
) {
    if ft.lines.is_empty() {
        return;
    }
    let start_y = cy - (ft.lines.len() as f32 - 1.0) * ft.line_height / 2.0;
    for (i, line) in ft.lines.iter().enumerate() {
        if line.text.is_empty() {
            continue;
        }
        let y = start_y + i as f32 * ft.line_height;
        c.text_unmeasured(Text {
            text: line.text.clone(),
            font_family: Some(font_family.to_string()),
            font_size: Some(font_size),
            font_color: Some(color),
            font_weight: if line.bold || default_bold {
                Some("bold".to_string())
            } else {
                None
            },
            font_style: if line.italic || default_italic {
                Some("italic".to_string())
            } else {
                None
            },
            x: Some(x),
            y: Some(y),
            text_anchor: Some(anchor.to_string()),
            dominant_baseline: Some("central".to_string()),
            spans: line.spans.clone(),
            ..Default::default()
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::DEFAULT_FONT_FAMILY;

    #[test]
    fn handles_breaks_html_entities_and_cjk() {
        let ft = format_and_measure(
            "<b>开始</b><br/>a &lt; b &amp; #quot;ok#quot;",
            DEFAULT_FONT_FAMILY,
            14.0,
        );
        assert_eq!(2, ft.lines.len());
        assert_eq!("开始", ft.lines[0].text);
        assert!(ft.lines[0].bold);
        assert_eq!("a < b & \"ok\"", ft.lines[1].text);
        assert!(!ft.lines[1].bold);
        assert!(ft.width > 0.0 && ft.height > 20.0);
    }
}
