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

//! Theme-derived Mermaid palette and inline CSS style (`style` / `classDef` /
//! `linkStyle`) parsing.

use std::collections::HashMap;

use super::super::color::Color;
use super::super::error::{Error, Result};
use super::super::theme::Theme;

/// Shared visual constants for the native Mermaid engines (flat, thin-stroke
/// style). Inline `style` / `classDef` overrides still take precedence.
pub(crate) const NODE_STROKE_WIDTH: f32 = 1.0;
pub(crate) const EDGE_STROKE_WIDTH: f32 = 1.25;
pub(crate) const EDGE_THICK_STROKE_WIDTH: f32 = 2.2;
pub(crate) const NODE_RADIUS: f32 = 6.0;
pub(crate) const SUBGRAPH_RADIUS: f32 = 8.0;
pub(crate) const SUBGRAPH_STROKE_WIDTH: f32 = 1.0;

/// Whether nodes are tinted with the theme accent (otherwise neutral fg mix).
const USE_ACCENT_NODES: bool = true;

/// Harmonious colors derived from a `charts-rs` [`Theme`].
///
/// Built from a two-color foundation: every UI color is a ratio mix between
/// the opaque background (`bg`) and the foreground text color (`fg`), with an
/// optional accent tint used only for node fills / strokes.
#[derive(Clone, Debug)]
pub(crate) struct MermaidPalette {
    pub(crate) is_light: bool,
    pub(crate) bg: Color,
    pub(crate) opaque_bg: Color,
    pub(crate) fg: Color,
    pub(crate) node_fill: Color,
    pub(crate) node_stroke: Color,
    pub(crate) edge_stroke: Color,
    pub(crate) text_color: Color,
    pub(crate) muted_text: Color,
    pub(crate) edge_label_bg: Color,
    pub(crate) edge_label_stroke: Color,
    pub(crate) header_fill: Color,
    pub(crate) subgraph_stroke: Color,
    pub(crate) note_fill: Color,
    pub(crate) note_stroke: Color,
    pub(crate) activation_fill: Color,
    pub(crate) accent: Color,
}

pub(crate) fn mix_color(a: Color, b: Color, t: f32) -> Color {
    let t = t.clamp(0.0, 1.0);
    let mix = |ca: u8, cb: u8| -> u8 { (ca as f32 + (cb as f32 - ca as f32) * t).round() as u8 };
    Color {
        r: mix(a.r, b.r),
        g: mix(a.g, b.g),
        b: mix(a.b, b.b),
        a: mix(a.a, b.a),
    }
}

impl MermaidPalette {
    pub(crate) fn from_theme(theme: &Theme, bg_override: Option<Color>) -> Self {
        let bg = bg_override.unwrap_or(theme.background_color);
        let is_light = if bg.is_transparent() {
            theme.is_light
        } else {
            bg.is_light()
        };
        let opaque_bg = if bg.is_transparent() {
            if is_light {
                Color::white()
            } else {
                (24, 24, 27).into()
            }
        } else {
            bg.with_alpha(255)
        };

        let accent = theme
            .series
            .colors
            .iter()
            .copied()
            .find(|c| !c.is_transparent() && !c.is_zero())
            .unwrap_or((84, 112, 198).into());

        let fg: Color =
            if !theme.title.font.color.is_zero() && !theme.title.font.color.is_transparent() {
                theme.title.font.color.with_alpha(255)
            } else if is_light {
                (32, 37, 46).into()
            } else {
                (235, 240, 248).into()
            };
        let ui_fg = if !is_light {
            let fg_luma = 0.299 * fg.r as f32 + 0.587 * fg.g as f32 + 0.114 * fg.b as f32;
            if fg_luma < 190.0 {
                mix_color(fg, (235, 238, 245).into(), 0.45)
            } else {
                fg
            }
        } else {
            fg
        };
        let m = |t: f32| mix_color(opaque_bg, fg, t);
        let mui = |t: f32| mix_color(opaque_bg, ui_fg, t);

        let (node_fill, node_stroke) = if USE_ACCENT_NODES {
            if is_light {
                (
                    mix_color(opaque_bg, accent, 0.08),
                    mix_color(mix_color(opaque_bg, accent, 0.62), fg, 0.12),
                )
            } else {
                (
                    mix_color(opaque_bg, accent, 0.16),
                    mix_color(mix_color(opaque_bg, accent, 0.70), fg, 0.15),
                )
            }
        } else if is_light {
            (m(0.04), m(0.38))
        } else {
            (m(0.07), m(0.40))
        };

        let edge_stroke = mui(0.55);
        let muted_text = if is_light { mui(0.60) } else { mui(0.62) };
        let edge_label_stroke = if is_light { m(0.12) } else { m(0.18) };
        let header_fill = if USE_ACCENT_NODES {
            if is_light {
                mix_color(opaque_bg, accent, 0.14)
            } else {
                mix_color(opaque_bg, accent, 0.26)
            }
        } else if is_light {
            m(0.07)
        } else {
            m(0.11)
        };
        let subgraph_stroke = if is_light { mui(0.18) } else { mui(0.22) };

        let note_base: Color = (250, 204, 21).into();
        let note_fill = if is_light {
            mix_color(opaque_bg, note_base, 0.14)
        } else {
            mix_color(opaque_bg, (180, 140, 30).into(), 0.20)
        };
        let note_stroke = mix_color(mix_color(opaque_bg, (202, 138, 4).into(), 0.70), fg, 0.10);
        let activation_fill = if is_light {
            mix_color(opaque_bg, accent, 0.18)
        } else {
            mix_color(opaque_bg, accent, 0.28)
        };

        Self {
            is_light,
            bg,
            opaque_bg,
            fg,
            node_fill,
            node_stroke,
            edge_stroke,
            text_color: fg,
            muted_text,
            edge_label_bg: opaque_bg,
            edge_label_stroke,
            header_fill,
            subgraph_stroke,
            note_fill,
            note_stroke,
            activation_fill,
            accent,
        }
    }

    pub(crate) fn subgraph_fill(&self, depth: usize) -> Color {
        let d = depth.min(4) as f32;
        if self.is_light {
            mix_color(self.opaque_bg, self.fg, 0.02 + 0.015 * d)
        } else {
            mix_color(self.opaque_bg, self.fg, 0.03 + 0.02 * d)
        }
    }

    pub(crate) fn box_fill(&self) -> Color {
        if self.is_light {
            mix_color(self.opaque_bg, self.fg, 0.035)
        } else {
            mix_color(self.opaque_bg, self.fg, 0.05)
        }
    }
}

/// Inline CSS properties supported on Mermaid nodes, edges and classes.
#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct StyleProps {
    pub(crate) fill: Option<Color>,
    pub(crate) stroke: Option<Color>,
    pub(crate) stroke_width: Option<f32>,
    pub(crate) stroke_dasharray: Option<String>,
    pub(crate) color: Option<Color>,
}

impl StyleProps {
    pub(crate) fn merge_from(&mut self, other: &StyleProps) {
        if other.fill.is_some() {
            self.fill = other.fill;
        }
        if other.stroke.is_some() {
            self.stroke = other.stroke;
        }
        if other.stroke_width.is_some() {
            self.stroke_width = other.stroke_width;
        }
        if other.stroke_dasharray.is_some() {
            self.stroke_dasharray = other.stroke_dasharray.clone();
        }
        if other.color.is_some() {
            self.color = other.color;
        }
    }
}

/// Parses a Mermaid CSS color string (`#rgb`, `#rrggbb`, `rgb()`, `rgba()`,
/// `"none"`, `"transparent"`, or a standard CSS named color).
pub(crate) fn parse_css_color(raw: &str) -> Option<Color> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return None;
    }
    if trimmed.eq_ignore_ascii_case("none") || trimmed.eq_ignore_ascii_case("transparent") {
        return Some(Color::transparent());
    }
    let lowered_fn;
    let candidate = if trimmed.len() >= 4
        && (trimmed[..4].eq_ignore_ascii_case("rgb(")
            || trimmed.to_ascii_lowercase().starts_with("rgba("))
    {
        lowered_fn = trimmed.to_ascii_lowercase();
        lowered_fn.as_str()
    } else {
        trimmed
    };
    if let Some(c) = Color::parse(candidate) {
        return Some(c);
    }
    named_css_color(trimmed)
}

/// Splits CSS declarations separated by `,` or `;` while keeping commas
/// inside `rgb(...)` / `rgba(...)` intact.
pub(crate) fn parse_style_props(raw: &str) -> StyleProps {
    let mut props = StyleProps::default();
    let mut decls = Vec::new();
    let mut start = 0;
    let mut paren_depth = 0usize;
    for (i, ch) in raw.char_indices() {
        match ch {
            '(' => paren_depth += 1,
            ')' => paren_depth = paren_depth.saturating_sub(1),
            ',' | ';' if paren_depth == 0 => {
                decls.push(&raw[start..i]);
                start = i + 1;
            }
            _ => {}
        }
    }
    decls.push(&raw[start..]);

    for decl in decls {
        let Some((key, val)) = decl.split_once(':') else {
            continue;
        };
        let key = key.trim().to_ascii_lowercase();
        let val = val.trim();
        match key.as_str() {
            "fill" => {
                if let Some(c) = parse_css_color(val) {
                    props.fill = Some(c);
                }
            }
            "stroke" => {
                if let Some(c) = parse_css_color(val) {
                    props.stroke = Some(c);
                }
            }
            "color" => {
                if let Some(c) = parse_css_color(val) {
                    props.color = Some(c);
                }
            }
            "stroke-width" => {
                let num = val.strip_suffix("px").unwrap_or(val).trim();
                if let Ok(w) = num.parse::<f32>()
                    && w.is_finite()
                    && w >= 0.0
                {
                    props.stroke_width = Some(w);
                }
            }
            "stroke-dasharray" if !val.is_empty() => {
                let normalized: Vec<&str> = val.split_whitespace().collect();
                props.stroke_dasharray = Some(normalized.join(","));
            }
            _ => {}
        }
    }
    props
}

fn named_css_color(name: &str) -> Option<Color> {
    let lower = name.to_ascii_lowercase();
    let rgb = match lower.as_str() {
        "aliceblue" => (240, 248, 255),
        "antiquewhite" => (250, 235, 215),
        "aqua" | "cyan" => (0, 255, 255),
        "aquamarine" => (127, 255, 212),
        "azure" => (240, 255, 255),
        "beige" => (245, 245, 220),
        "bisque" => (255, 228, 196),
        "black" => (0, 0, 0),
        "blanchedalmond" => (255, 235, 205),
        "blue" => (0, 0, 255),
        "blueviolet" => (138, 43, 226),
        "brown" => (165, 42, 42),
        "burlywood" => (222, 184, 135),
        "cadetblue" => (95, 158, 160),
        "chartreuse" => (127, 255, 0),
        "chocolate" => (210, 105, 30),
        "coral" => (255, 127, 80),
        "cornflowerblue" => (100, 149, 237),
        "cornsilk" => (255, 248, 220),
        "crimson" => (220, 20, 60),
        "darkblue" => (0, 0, 139),
        "darkcyan" => (0, 139, 139),
        "darkgoldenrod" => (184, 134, 11),
        "darkgray" | "darkgrey" => (169, 169, 169),
        "darkgreen" => (0, 100, 0),
        "darkkhaki" => (189, 183, 107),
        "darkmagenta" => (139, 0, 139),
        "darkolivegreen" => (85, 107, 47),
        "darkorange" => (255, 140, 0),
        "darkorchid" => (153, 50, 204),
        "darkred" => (139, 0, 0),
        "darksalmon" => (233, 150, 122),
        "darkseagreen" => (143, 188, 143),
        "darkslateblue" => (72, 61, 139),
        "darkslategray" | "darkslategrey" => (47, 79, 79),
        "darkturquoise" => (0, 206, 209),
        "darkviolet" => (148, 0, 211),
        "deeppink" => (255, 20, 147),
        "deepskyblue" => (0, 191, 255),
        "dimgray" | "dimgrey" => (105, 105, 105),
        "dodgerblue" => (30, 144, 255),
        "firebrick" => (178, 34, 34),
        "floralwhite" => (255, 250, 240),
        "forestgreen" => (34, 139, 34),
        "fuchsia" | "magenta" => (255, 0, 255),
        "gainsboro" => (220, 220, 220),
        "ghostwhite" => (248, 248, 255),
        "gold" => (255, 215, 0),
        "goldenrod" => (218, 165, 32),
        "gray" | "grey" => (128, 128, 128),
        "green" => (0, 128, 0),
        "greenyellow" => (173, 255, 47),
        "honeydew" => (240, 255, 240),
        "hotpink" => (255, 105, 180),
        "indianred" => (205, 92, 92),
        "indigo" => (75, 0, 130),
        "ivory" => (255, 255, 240),
        "khaki" => (240, 230, 140),
        "lavender" => (230, 230, 250),
        "lavenderblush" => (255, 240, 245),
        "lawngreen" => (124, 252, 0),
        "lemonchiffon" => (255, 250, 205),
        "lightblue" => (173, 216, 230),
        "lightcoral" => (240, 128, 128),
        "lightcyan" => (224, 255, 255),
        "lightgoldenrodyellow" => (250, 250, 210),
        "lightgray" | "lightgrey" => (211, 211, 211),
        "lightgreen" => (144, 238, 144),
        "lightpink" => (255, 182, 193),
        "lightsalmon" => (255, 160, 122),
        "lightseagreen" => (32, 178, 170),
        "lightskyblue" => (135, 206, 250),
        "lightslategray" | "lightslategrey" => (119, 136, 153),
        "lightsteelblue" => (176, 196, 222),
        "lightyellow" => (255, 255, 224),
        "lime" => (0, 255, 0),
        "limegreen" => (50, 205, 50),
        "linen" => (250, 240, 230),
        "maroon" => (128, 0, 0),
        "mediumaquamarine" => (102, 205, 170),
        "mediumblue" => (0, 0, 205),
        "mediumorchid" => (186, 85, 211),
        "mediumpurple" => (147, 112, 219),
        "mediumseagreen" => (60, 179, 113),
        "mediumslateblue" => (123, 104, 238),
        "mediumspringgreen" => (0, 250, 154),
        "mediumturquoise" => (72, 209, 204),
        "mediumvioletred" => (199, 21, 133),
        "midnightblue" => (25, 25, 112),
        "mintcream" => (245, 255, 250),
        "mistyrose" => (255, 228, 225),
        "moccasin" => (255, 228, 181),
        "navajowhite" => (255, 222, 173),
        "navy" => (0, 0, 128),
        "oldlace" => (253, 245, 230),
        "olive" => (128, 128, 0),
        "olivedrab" => (107, 142, 35),
        "orange" => (255, 165, 0),
        "orangered" => (255, 69, 0),
        "orchid" => (218, 112, 214),
        "palegoldenrod" => (238, 232, 170),
        "palegreen" => (152, 251, 152),
        "paleturquoise" => (175, 238, 238),
        "palevioletred" => (219, 112, 147),
        "papayawhip" => (255, 239, 213),
        "peachpuff" => (255, 218, 185),
        "peru" => (205, 133, 63),
        "pink" => (255, 192, 203),
        "plum" => (221, 160, 221),
        "powderblue" => (176, 224, 230),
        "purple" => (128, 0, 128),
        "rebeccapurple" => (102, 51, 153),
        "red" => (255, 0, 0),
        "rosybrown" => (188, 143, 143),
        "royalblue" => (65, 105, 225),
        "saddlebrown" => (139, 69, 19),
        "salmon" => (250, 128, 114),
        "sandybrown" => (244, 164, 96),
        "seagreen" => (46, 139, 87),
        "seashell" => (255, 245, 238),
        "sienna" => (160, 82, 45),
        "silver" => (192, 192, 192),
        "skyblue" => (135, 206, 235),
        "slateblue" => (106, 90, 205),
        "slategray" | "slategrey" => (112, 128, 144),
        "snow" => (255, 250, 250),
        "springgreen" => (0, 255, 127),
        "steelblue" => (70, 130, 180),
        "tan" => (210, 180, 140),
        "teal" => (0, 128, 128),
        "thistle" => (216, 191, 216),
        "tomato" => (255, 99, 71),
        "turquoise" => (64, 224, 208),
        "violet" => (238, 130, 238),
        "wheat" => (245, 222, 179),
        "white" => (255, 255, 255),
        "whitesmoke" => (245, 245, 245),
        "yellow" => (255, 255, 0),
        "yellowgreen" => (154, 205, 50),
        _ => return None,
    };
    Some(rgb.into())
}

/// Strips matching double or single surrounding quotes and unescapes `\"`.
pub(crate) fn unquote_text(raw: &str) -> String {
    let t = raw.trim();
    if t.len() >= 2
        && ((t.starts_with('"') && t.ends_with('"')) || (t.starts_with('\'') && t.ends_with('\'')))
    {
        t[1..t.len() - 1].replace("\\\"", "\"")
    } else {
        t.to_string()
    }
}

/// Splits optional inline `:::cls1,cls2` class suffix from an identifier token.
pub(crate) fn split_inline_classes(token: &str) -> (&str, Vec<String>) {
    if let Some((head, cls)) = token.split_once(":::") {
        let classes = cls
            .split(',')
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(str::to_string)
            .collect();
        (head.trim(), classes)
    } else {
        (token.trim(), Vec::new())
    }
}

/// Shared `classDef`, `class` / `cssClass`, and `style` statements used across
/// state, class and ER diagrams.
pub(crate) enum StyleDirective {
    ClassDef(String, StyleProps),
    ClassAssign {
        ids: Vec<String>,
        classes: Vec<String>,
    },
    NodeStyle {
        ids: Vec<String>,
        props: StyleProps,
    },
}

pub(crate) fn try_parse_style_directive(
    trimmed: &str,
    line_no: usize,
) -> Result<Option<StyleDirective>> {
    if let Some(rest) = trimmed.strip_prefix("classDef")
        && rest.starts_with(char::is_whitespace)
    {
        let rest = rest.trim();
        let Some((cls_name, css)) = rest.split_once(char::is_whitespace) else {
            return Err(Error::Mermaid {
                line: Some(line_no),
                message: "expected `classDef <name> <styles>`".to_string(),
            });
        };
        return Ok(Some(StyleDirective::ClassDef(
            cls_name.trim().to_string(),
            parse_style_props(css.trim()),
        )));
    }

    if let Some(rest) = trimmed
        .strip_prefix("cssClass")
        .or_else(|| trimmed.strip_prefix("class"))
        && rest.starts_with(char::is_whitespace)
    {
        let rest = rest.trim();
        let Some((ids_raw, cls_raw)) = rest.split_once(char::is_whitespace) else {
            return Err(Error::Mermaid {
                line: Some(line_no),
                message: "expected `class <ids> <className>`".to_string(),
            });
        };
        let ids: Vec<String> = unquote_text(ids_raw)
            .split(',')
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(str::to_string)
            .collect();
        let classes: Vec<String> = unquote_text(cls_raw)
            .split(',')
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(str::to_string)
            .collect();
        return Ok(Some(StyleDirective::ClassAssign { ids, classes }));
    }

    if let Some(rest) = trimmed.strip_prefix("style")
        && rest.starts_with(char::is_whitespace)
    {
        let rest = rest.trim();
        let Some((ids_raw, css)) = rest.split_once(char::is_whitespace) else {
            return Err(Error::Mermaid {
                line: Some(line_no),
                message: "expected `style <id> <styles>`".to_string(),
            });
        };
        let ids: Vec<String> = ids_raw
            .split(',')
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(str::to_string)
            .collect();
        return Ok(Some(StyleDirective::NodeStyle {
            ids,
            props: parse_style_props(css.trim()),
        }));
    }

    Ok(None)
}

pub(crate) fn resolve_style(
    classes: &[String],
    inline: &StyleProps,
    defs: &HashMap<String, StyleProps>,
) -> StyleProps {
    let mut out = StyleProps::default();
    for cls in classes {
        if let Some(def) = defs.get(cls) {
            out.merge_from(def);
        }
    }
    out.merge_from(inline);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_inline_css_properties() {
        let p = parse_style_props(
            "fill:#f9f,stroke:rgb(51, 51, 51),stroke-width:3px,stroke-dasharray: 5 5,color:navy,unknown:1",
        );
        assert_eq!(Some((255, 153, 255, 255).into()), p.fill);
        assert_eq!(Some((51, 51, 51, 255).into()), p.stroke);
        assert_eq!(Some(3.0), p.stroke_width);
        assert_eq!(Some("5,5".to_string()), p.stroke_dasharray);
        assert_eq!(Some((0, 0, 128, 255).into()), p.color);
    }
}
