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

//! Compact SVG output: the same picture in fewer bytes.
//!
//! [`compact_svg`] rewrites a chart's SVG the way an optimizer such as svgo
//! would, without changing what is drawn:
//!
//! - whitespace between elements and around text content is dropped;
//! - `line`, `rect` (without corner radius), `polygon` and `polyline` become
//!   paths, path data is written with relative coordinates, `h`/`v`
//!   shorthands and minimal separators, and adjacent paths that share their
//!   attributes are merged (grid lines, axis ticks);
//! - attributes shared by every child of a group move onto the group, and a
//!   font family shared by every text moves onto the root;
//! - default values (`stroke-width="1"`, `x="0"`), leading zeros (`.45`) and
//!   long hex colors (`#fff`) are shortened; attribute-less groups are
//!   removed.
//!
//! `data-*` attributes, classes, `<title>` tooltips and `<style>` blocks are
//! kept as they are. The rewriter understands the markup this crate emits
//! (elements, attributes in double quotes, text, no comments or CDATA); other
//! SVG is passed through where it is not understood.

use std::fmt::Write;

/// Rewrites `svg` into a compact form that renders the same; see the module
/// documentation for what is changed.
pub fn compact_svg(svg: &str) -> String {
    let Some(mut root) = parse(svg) else {
        return svg.to_string();
    };
    hoist_font_family(&mut root);
    compact_node(&mut root, &Inherited::default());
    let mut out = String::with_capacity(svg.len());
    serialize(&root, &mut out);
    out
}

#[derive(Debug, Clone)]
enum Node {
    Element(Element),
    /// Raw markup between tags (already escaped), kept verbatim.
    Text(String),
}

#[derive(Debug, Clone)]
struct Element {
    name: String,
    attrs: Vec<(String, String)>,
    children: Vec<Node>,
}

impl Element {
    fn attr(&self, name: &str) -> Option<&str> {
        self.attrs
            .iter()
            .find(|(k, _)| k == name)
            .map(|(_, v)| v.as_str())
    }
    fn remove_attr(&mut self, name: &str) -> Option<String> {
        let pos = self.attrs.iter().position(|(k, _)| k == name)?;
        Some(self.attrs.remove(pos).1)
    }
    fn set_attr(&mut self, name: &str, value: String) {
        if let Some(slot) = self.attrs.iter_mut().find(|(k, _)| k == name) {
            slot.1 = value;
        } else {
            self.attrs.push((name.to_string(), value));
        }
    }
    fn child_elements(&self) -> impl Iterator<Item = &Element> {
        self.children.iter().filter_map(|c| match c {
            Node::Element(e) => Some(e),
            Node::Text(_) => None,
        })
    }
    fn is_only_elements(&self) -> bool {
        self.children.iter().all(|c| matches!(c, Node::Element(_)))
    }
}

// ---------------------------------------------------------------------------
// Parsing: just enough XML for the markup this crate produces.

fn parse(svg: &str) -> Option<Element> {
    let mut stack: Vec<Element> = vec![Element {
        name: String::new(),
        attrs: vec![],
        children: vec![],
    }];
    let bytes = svg.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] != b'<' {
            let end = svg[i..].find('<').map(|p| i + p).unwrap_or(bytes.len());
            let text = &svg[i..end];
            if !text.trim().is_empty() {
                stack
                    .last_mut()?
                    .children
                    .push(Node::Text(text.to_string()));
            }
            i = end;
            continue;
        }
        if svg[i..].starts_with("</") {
            let end = i + svg[i..].find('>')?;
            let name = svg[i + 2..end].trim();
            let element = stack.pop()?;
            if element.name != name {
                return None;
            }
            stack.last_mut()?.children.push(Node::Element(element));
            i = end + 1;
            continue;
        }
        if svg[i..].starts_with("<?") || svg[i..].starts_with("<!") {
            // Declarations and comments are not part of this crate's output.
            return None;
        }
        // Start tag: name, then attributes up to `>` or `/>`.
        let mut j = i + 1;
        while j < bytes.len()
            && !bytes[j].is_ascii_whitespace()
            && bytes[j] != b'>'
            && bytes[j] != b'/'
        {
            j += 1;
        }
        let name = svg[i + 1..j].to_string();
        let mut attrs = vec![];
        let self_closing;
        loop {
            while j < bytes.len() && bytes[j].is_ascii_whitespace() {
                j += 1;
            }
            if j >= bytes.len() {
                return None;
            }
            if bytes[j] == b'>' {
                self_closing = false;
                j += 1;
                break;
            }
            if bytes[j] == b'/' {
                self_closing = true;
                j = j + 1 + svg[j + 1..].find('>')? + 1;
                break;
            }
            let eq = j + svg[j..].find('=')?;
            let key = svg[j..eq].trim().to_string();
            let quote = eq + 1 + svg[eq + 1..].find('"')?;
            let close = quote + 1 + svg[quote + 1..].find('"')?;
            attrs.push((key, svg[quote + 1..close].to_string()));
            j = close + 1;
        }
        let element = Element {
            name,
            attrs,
            children: vec![],
        };
        if self_closing {
            stack.last_mut()?.children.push(Node::Element(element));
        } else {
            stack.push(element);
        }
        i = j;
    }
    let mut document = stack.pop()?;
    if !stack.is_empty() || document.children.len() != 1 {
        return None;
    }
    match document.children.pop()? {
        Node::Element(root) => Some(root),
        Node::Text(_) => None,
    }
}

fn serialize(element: &Element, out: &mut String) {
    out.push('<');
    out.push_str(&element.name);
    for (k, v) in element.attrs.iter() {
        out.push(' ');
        out.push_str(k);
        out.push_str("=\"");
        out.push_str(v);
        out.push('"');
    }
    if element.children.is_empty() {
        out.push_str("/>");
        return;
    }
    out.push('>');
    for child in element.children.iter() {
        match child {
            Node::Element(e) => serialize(e, out),
            Node::Text(t) => out.push_str(t.trim()),
        }
    }
    out.push_str("</");
    out.push_str(&element.name);
    out.push('>');
}

// ---------------------------------------------------------------------------
// Rewrites.

/// Presentation values in effect from the ancestors, to know when an
/// attribute equals what the element would inherit anyway.
#[derive(Default, Clone)]
struct Inherited {
    stroke_width: Option<String>,
}

const INHERITABLE: &[&str] = &[
    "fill",
    "fill-opacity",
    "stroke",
    "stroke-opacity",
    "stroke-width",
    "stroke-dasharray",
    "font-family",
    "font-size",
    "font-weight",
    "text-anchor",
    "dominant-baseline",
];

fn compact_node(element: &mut Element, inherited: &Inherited) {
    // Children first (bottom-up), so groups see already-compacted shapes.
    let mut scope = inherited.clone();
    if let Some(width) = element.attr("stroke-width") {
        scope.stroke_width = Some(width.to_string());
    }
    for child in element.children.iter_mut() {
        if let Node::Element(e) = child {
            compact_node(e, &scope);
        }
    }
    collapse_groups(element);
    shape_to_path(element);
    hoist_to_group(element);
    merge_paths(element);
    tidy_attrs(element, inherited);
}

/// Splices the children of attribute-less `<g>` children into `element`.
fn collapse_groups(element: &mut Element) {
    let mut children = Vec::with_capacity(element.children.len());
    for child in element.children.drain(..) {
        match child {
            Node::Element(e) if e.name == "g" && e.attrs.is_empty() => {
                children.extend(e.children);
            }
            other => children.push(other),
        }
    }
    element.children = children;
}

/// Turns simple shapes into paths (which then take relative coordinates
/// and can be merged) and compacts every path's data.
fn shape_to_path(element: &mut Element) {
    let (name, attrs) = (element.name.as_str(), &mut element.attrs);
    let d = match name {
        "line" => {
            let (x1, y1, x2, y2) = (
                take_number(attrs, "x1"),
                take_number(attrs, "y1"),
                take_number(attrs, "x2"),
                take_number(attrs, "y2"),
            );
            Some(vec![Seg::M(x1, y1), Seg::L(x2, y2)])
        }
        "rect" if element_has_no_radius(attrs) => {
            let (x, y, w, h) = (
                take_number(attrs, "x"),
                take_number(attrs, "y"),
                take_number(attrs, "width"),
                take_number(attrs, "height"),
            );
            Some(vec![
                Seg::M(x, y),
                Seg::L(x + w, y),
                Seg::L(x + w, y + h),
                Seg::L(x, y + h),
                Seg::Z,
            ])
        }
        "polygon" | "polyline" => {
            let points = attrs
                .iter()
                .position(|(k, _)| k == "points")
                .map(|p| attrs.remove(p).1)
                .unwrap_or_default();
            let mut segs = vec![];
            for (i, (x, y)) in parse_points(&points).into_iter().enumerate() {
                segs.push(if i == 0 { Seg::M(x, y) } else { Seg::L(x, y) });
            }
            if name == "polygon" {
                segs.push(Seg::Z);
            }
            Some(segs)
        }
        "path" => attrs
            .iter()
            .position(|(k, _)| k == "d")
            .map(|p| attrs.remove(p).1)
            .and_then(|d| parse_path(&d)),
        _ => None,
    };
    if let Some(segs) = d {
        element.name = "path".to_string();
        // `d` first, like the rest of the crate's paths.
        element
            .attrs
            .insert(0, ("d".to_string(), write_path(&segs)));
    }
}

fn element_has_no_radius(attrs: &[(String, String)]) -> bool {
    !attrs
        .iter()
        .any(|(k, v)| (k == "rx" || k == "ry") && v != "0")
}

fn take_number(attrs: &mut Vec<(String, String)>, name: &str) -> Dec {
    attrs
        .iter()
        .position(|(k, _)| k == name)
        .map(|p| attrs.remove(p).1)
        .and_then(|v| Dec::parse(&v))
        .unwrap_or_default()
}

/// Moves attributes shared by every child element onto the group.
fn hoist_to_group(element: &mut Element) {
    if element.name != "g" || element.children.len() < 2 || !element.is_only_elements() {
        return;
    }
    for attr in INHERITABLE {
        let mut shared: Option<&str> = None;
        let mut all = true;
        for child in element.child_elements() {
            match child.attr(attr) {
                Some(v) if shared.is_none_or(|s| s == v) => shared = Some(v),
                _ => {
                    all = false;
                    break;
                }
            }
        }
        if let (true, Some(value)) = (all, shared) {
            let value = value.to_string();
            for child in element.children.iter_mut() {
                if let Node::Element(e) = child {
                    e.remove_attr(attr);
                }
            }
            element.set_attr(attr, value);
        }
    }
}

/// Puts a font family shared by every `<text>` on the root element.
fn hoist_font_family(root: &mut Element) {
    fn collect<'a>(e: &'a Element, out: &mut Vec<Option<&'a str>>) {
        if e.name == "text" {
            out.push(e.attr("font-family"));
        }
        for child in e.child_elements() {
            collect(child, out);
        }
    }
    fn strip(e: &mut Element) {
        if e.name == "text" {
            e.remove_attr("font-family");
        }
        for child in e.children.iter_mut() {
            if let Node::Element(c) = child {
                strip(c);
            }
        }
    }
    let mut families = vec![];
    collect(root, &mut families);
    let Some(Some(first)) = families.first() else {
        return;
    };
    if families.iter().all(|f| *f == Some(first)) {
        let family = first.to_string();
        strip(root);
        root.set_attr("font-family", family);
    }
}

/// Merges runs of adjacent childless `<path>` siblings that share every
/// attribute but `d`.
fn merge_paths(element: &mut Element) {
    let mut merged: Vec<Node> = Vec::with_capacity(element.children.len());
    for child in element.children.drain(..) {
        if let (Node::Element(cur), Some(Node::Element(prev))) = (&child, merged.last_mut())
            && cur.name == "path"
            && prev.name == "path"
            && cur.children.is_empty()
            && prev.children.is_empty()
            && same_attrs_but_d(&cur.attrs, &prev.attrs)
            && let (Some(d_cur), Some(d_prev)) = (cur.attr("d"), prev.attr("d"))
        {
            let joined = format!("{d_prev}{d_cur}");
            prev.set_attr("d", joined);
            continue;
        }
        merged.push(child);
    }
    element.children = merged;
}

fn same_attrs_but_d(a: &[(String, String)], b: &[(String, String)]) -> bool {
    let strip = |attrs: &[(String, String)]| -> Vec<(String, String)> {
        attrs.iter().filter(|(k, _)| k != "d").cloned().collect()
    };
    strip(a) == strip(b)
}

/// Drops defaults, shortens colors and numbers.
fn tidy_attrs(element: &mut Element, inherited: &Inherited) {
    let is_rect = element.name == "rect";
    element.attrs.retain(|(k, v)| {
        !matches!(
            (k.as_str(), v.as_str()),
            ("fill-opacity", "1") | ("stroke-opacity", "1")
        ) && !(is_rect && (k == "x" || k == "y") && v == "0")
            && !(k == "stroke-width" && v == "1" && inherited.stroke_width.is_none())
    });
    for (k, v) in element.attrs.iter_mut() {
        if let Some(short) = short_hex(v) {
            *v = short;
        } else if k != "d"
            && k != "points"
            && !k.starts_with("data-")
            && let Some(n) = Dec::parse(v)
        {
            *v = n.to_string();
        }
    }
}

fn short_hex(value: &str) -> Option<String> {
    let hex = value.strip_prefix('#')?;
    if hex.len() != 6 || !hex.bytes().all(|b| b.is_ascii_hexdigit()) {
        return None;
    }
    let b = hex.as_bytes();
    if b[0] == b[1] && b[2] == b[3] && b[4] == b[5] {
        Some(format!("#{}{}{}", b[0] as char, b[2] as char, b[4] as char))
    } else {
        None
    }
}

// ---------------------------------------------------------------------------
// Fixed-point decimals: coordinates are re-expressed relative to each other
// without any rounding, so the geometry is exactly the original.

/// A decimal number as `mantissa / 10^scale`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
struct Dec {
    mantissa: i64,
    scale: u32,
}

impl Dec {
    fn parse(text: &str) -> Option<Dec> {
        let text = text.trim();
        let (negative, digits) = match text.strip_prefix('-') {
            Some(rest) => (true, rest),
            None => (false, text.strip_prefix('+').unwrap_or(text)),
        };
        if digits.is_empty() || !digits.bytes().all(|b| b.is_ascii_digit() || b == b'.') {
            return None;
        }
        let (int, frac) = digits.split_once('.').unwrap_or((digits, ""));
        if int.is_empty() && frac.is_empty() || frac.contains('.') {
            return None;
        }
        let scale = frac.len() as u32;
        let mantissa: i64 = format!("{int}{frac}").parse().ok()?;
        Some(Dec {
            mantissa: if negative { -mantissa } else { mantissa },
            scale,
        })
    }
    fn rescale(self, scale: u32) -> Dec {
        Dec {
            mantissa: self.mantissa * 10_i64.pow(scale - self.scale),
            scale,
        }
    }
    fn is_zero(self) -> bool {
        self.mantissa == 0
    }
}

impl std::ops::Add for Dec {
    type Output = Dec;
    fn add(self, other: Dec) -> Dec {
        let scale = self.scale.max(other.scale);
        Dec {
            mantissa: self.rescale(scale).mantissa + other.rescale(scale).mantissa,
            scale,
        }
    }
}

impl std::ops::Sub for Dec {
    type Output = Dec;
    fn sub(self, other: Dec) -> Dec {
        let scale = self.scale.max(other.scale);
        Dec {
            mantissa: self.rescale(scale).mantissa - other.rescale(scale).mantissa,
            scale,
        }
    }
}

impl std::fmt::Display for Dec {
    /// Shortest form: no trailing zeros, no leading zero (`.5`, `-.5`, `12`).
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let mut mantissa = self.mantissa;
        let mut scale = self.scale;
        while scale > 0 && mantissa % 10 == 0 {
            mantissa /= 10;
            scale -= 1;
        }
        if scale == 0 {
            return write!(f, "{mantissa}");
        }
        let digits = format!("{:0width$}", mantissa.abs(), width = scale as usize + 1);
        let (int, frac) = digits.split_at(digits.len() - scale as usize);
        let int = if int == "0" { "" } else { int };
        write!(f, "{}{int}.{frac}", if mantissa < 0 { "-" } else { "" })
    }
}

// ---------------------------------------------------------------------------
// Path data.

#[derive(Debug, Clone, Copy, PartialEq)]
enum Seg {
    M(Dec, Dec),
    L(Dec, Dec),
    C(Dec, Dec, Dec, Dec, Dec, Dec),
    Q(Dec, Dec, Dec, Dec),
    /// rx, ry, rotation, large arc, sweep, x, y
    A(Dec, Dec, Dec, Dec, Dec, Dec, Dec),
    Z,
}

fn parse_points(points: &str) -> Vec<(Dec, Dec)> {
    let numbers: Vec<Dec> = tokenize_numbers(points)
        .iter()
        .filter_map(|t| Dec::parse(t))
        .collect();
    numbers
        .chunks(2)
        .filter(|c| c.len() == 2)
        .map(|c| (c[0], c[1]))
        .collect()
}

fn tokenize_numbers(text: &str) -> Vec<String> {
    let mut out = vec![];
    let mut cur = String::new();
    for ch in text.chars() {
        if ch.is_ascii_digit() || ch == '.' || ch == '-' || ch == '+' {
            // A sign or a second decimal point starts the next number.
            let starts_next =
                ((ch == '-' || ch == '+') && !cur.is_empty()) || (ch == '.' && cur.contains('.'));
            if starts_next {
                out.push(std::mem::take(&mut cur));
            }
            cur.push(ch);
        } else {
            if !cur.is_empty() {
                out.push(std::mem::take(&mut cur));
            }
            if ch.is_ascii_alphabetic() {
                out.push(ch.to_string());
            }
        }
    }
    if !cur.is_empty() {
        out.push(cur);
    }
    out
}

/// Parses path data (absolute or relative commands, implicit repeats) into
/// absolute segments; `None` when a command is not understood.
fn parse_path(d: &str) -> Option<Vec<Seg>> {
    let tokens = tokenize_numbers(d);
    let mut segs = vec![];
    let mut i = 0;
    let mut cmd = 'M';
    let (mut cx, mut cy) = (Dec::default(), Dec::default());
    let (mut sx, mut sy) = (cx, cy);
    // Reflection points for the smooth variants.
    let mut last_c2: Option<(Dec, Dec)> = None;
    let mut last_q1: Option<(Dec, Dec)> = None;
    let mut first = true;
    while i < tokens.len() {
        let t = &tokens[i];
        if t.len() == 1 && t.chars().next().is_some_and(|c| c.is_ascii_alphabetic()) {
            cmd = t.chars().next()?;
            i += 1;
            if cmd == 'Z' || cmd == 'z' {
                segs.push(Seg::Z);
                cx = sx;
                cy = sy;
                last_c2 = None;
                last_q1 = None;
                continue;
            }
        } else if first {
            return None;
        } else if cmd == 'M' {
            cmd = 'L';
        } else if cmd == 'm' {
            cmd = 'l';
        }
        first = false;
        let relative = cmd.is_ascii_lowercase();
        let num = |i: &mut usize| -> Option<Dec> {
            let v = Dec::parse(tokens.get(*i)?)?;
            *i += 1;
            Some(v)
        };
        let abs = |v: Dec, base: Dec| if relative { base + v } else { v };
        match cmd.to_ascii_uppercase() {
            'M' => {
                let (x, y) = (num(&mut i)?, num(&mut i)?);
                let (x, y) = (abs(x, cx), abs(y, cy));
                segs.push(Seg::M(x, y));
                cx = x;
                cy = y;
                sx = x;
                sy = y;
                last_c2 = None;
                last_q1 = None;
            }
            'L' | 'H' | 'V' => {
                let (x, y) = match cmd.to_ascii_uppercase() {
                    'H' => (abs(num(&mut i)?, cx), cy),
                    'V' => (cx, abs(num(&mut i)?, cy)),
                    _ => {
                        let (x, y) = (num(&mut i)?, num(&mut i)?);
                        (abs(x, cx), abs(y, cy))
                    }
                };
                segs.push(Seg::L(x, y));
                cx = x;
                cy = y;
                last_c2 = None;
                last_q1 = None;
            }
            'C' | 'S' => {
                let (x1, y1) = if cmd.eq_ignore_ascii_case(&'C') {
                    let (x1, y1) = (num(&mut i)?, num(&mut i)?);
                    (abs(x1, cx), abs(y1, cy))
                } else {
                    match last_c2 {
                        Some((px, py)) => (cx + (cx - px), cy + (cy - py)),
                        None => (cx, cy),
                    }
                };
                let (x2, y2) = (num(&mut i)?, num(&mut i)?);
                let (x2, y2) = (abs(x2, cx), abs(y2, cy));
                let (x, y) = (num(&mut i)?, num(&mut i)?);
                let (x, y) = (abs(x, cx), abs(y, cy));
                segs.push(Seg::C(x1, y1, x2, y2, x, y));
                cx = x;
                cy = y;
                last_c2 = Some((x2, y2));
                last_q1 = None;
            }
            'Q' | 'T' => {
                let (x1, y1) = if cmd.eq_ignore_ascii_case(&'Q') {
                    let (x1, y1) = (num(&mut i)?, num(&mut i)?);
                    (abs(x1, cx), abs(y1, cy))
                } else {
                    match last_q1 {
                        Some((px, py)) => (cx + (cx - px), cy + (cy - py)),
                        None => (cx, cy),
                    }
                };
                let (x, y) = (num(&mut i)?, num(&mut i)?);
                let (x, y) = (abs(x, cx), abs(y, cy));
                segs.push(Seg::Q(x1, y1, x, y));
                cx = x;
                cy = y;
                last_q1 = Some((x1, y1));
                last_c2 = None;
            }
            'A' => {
                let (rx, ry, rot, laf, sf) = (
                    num(&mut i)?,
                    num(&mut i)?,
                    num(&mut i)?,
                    num(&mut i)?,
                    num(&mut i)?,
                );
                let (x, y) = (num(&mut i)?, num(&mut i)?);
                let (x, y) = (abs(x, cx), abs(y, cy));
                segs.push(Seg::A(rx, ry, rot, laf, sf, x, y));
                cx = x;
                cy = y;
                last_c2 = None;
                last_q1 = None;
            }
            _ => return None,
        }
    }
    Some(segs)
}

/// Writes segments as relative commands with `h`/`v` shorthands, implicit
/// repeats and the fewest separators.
fn write_path(segs: &[Seg]) -> String {
    let mut out = String::new();
    let mut writer = PathWriter::default();
    let (mut cx, mut cy) = (Dec::default(), Dec::default());
    let (mut sx, mut sy) = (cx, cy);
    for (index, seg) in segs.iter().enumerate() {
        match *seg {
            Seg::M(x, y) => {
                // Absolute moves are unambiguous after a merge and are usually
                // as short as relative ones; only the first move has no origin.
                let (dx, dy) = (x - cx, y - cy);
                if index == 0 || abs_len(&[x, y]) <= abs_len(&[dx, dy]) {
                    writer.command(&mut out, 'M');
                    writer.numbers(&mut out, &[x, y]);
                } else {
                    writer.command(&mut out, 'm');
                    writer.numbers(&mut out, &[dx, dy]);
                }
                cx = x;
                cy = y;
                sx = x;
                sy = y;
            }
            Seg::L(x, y) => {
                // Relative unless the absolute form is shorter (e.g. `H0`).
                let (dx, dy) = (x - cx, y - cy);
                let (rel, abs, rel_nums, abs_nums) = if dy.is_zero() {
                    ('h', 'H', vec![dx], vec![x])
                } else if dx.is_zero() {
                    ('v', 'V', vec![dy], vec![y])
                } else {
                    ('l', 'L', vec![dx, dy], vec![x, y])
                };
                // A repeated command letter costs nothing, so count it.
                let cost = |cmd: char, nums: &[Dec]| {
                    abs_len(nums) + usize::from(writer.last_command != Some(cmd))
                };
                if cost(abs, &abs_nums) < cost(rel, &rel_nums) {
                    writer.command(&mut out, abs);
                    writer.numbers(&mut out, &abs_nums);
                } else {
                    writer.command(&mut out, rel);
                    writer.numbers(&mut out, &rel_nums);
                }
                cx = x;
                cy = y;
            }
            Seg::C(x1, y1, x2, y2, x, y) => {
                writer.command(&mut out, 'c');
                writer.numbers(
                    &mut out,
                    &[x1 - cx, y1 - cy, x2 - cx, y2 - cy, x - cx, y - cy],
                );
                cx = x;
                cy = y;
            }
            Seg::Q(x1, y1, x, y) => {
                writer.command(&mut out, 'q');
                writer.numbers(&mut out, &[x1 - cx, y1 - cy, x - cx, y - cy]);
                cx = x;
                cy = y;
            }
            Seg::A(rx, ry, rot, laf, sf, x, y) => {
                writer.command(&mut out, 'a');
                writer.numbers(&mut out, &[rx, ry, rot, laf, sf, x - cx, y - cy]);
                cx = x;
                cy = y;
            }
            Seg::Z => {
                writer.command(&mut out, 'z');
                cx = sx;
                cy = sy;
            }
        }
    }
    out
}

fn abs_len(values: &[Dec]) -> usize {
    values.iter().map(|v| v.to_string().len()).sum()
}

#[derive(Default)]
struct PathWriter {
    last_command: Option<char>,
    /// The previous token was a number (so a separator may be needed).
    after_number: bool,
    /// The previous number contained a decimal point.
    last_had_dot: bool,
}

impl PathWriter {
    fn command(&mut self, out: &mut String, cmd: char) {
        // Repeated commands are implicit, except a move (whose repeat would
        // mean lineto) and `z`.
        if self.last_command == Some(cmd) && !matches!(cmd, 'M' | 'm' | 'z') {
            return;
        }
        out.push(cmd);
        self.last_command = Some(cmd);
        self.after_number = false;
    }
    fn numbers(&mut self, out: &mut String, values: &[Dec]) {
        for v in values {
            let text = v.to_string();
            if self.after_number {
                let starts_signed = text.starts_with('-');
                let starts_dot = text.starts_with('.') && self.last_had_dot;
                if !starts_signed && !starts_dot {
                    out.push(' ');
                }
            }
            let _ = write!(out, "{text}");
            self.after_number = true;
            self.last_had_dot = text.contains('.');
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decimals_are_exact() {
        let a = Dec::parse("199.8").unwrap();
        let b = Dec::parse("65").unwrap();
        assert_eq!("134.8", (a - b).to_string());
        assert_eq!("264.8", (a + b).to_string());
        assert_eq!(".45", Dec::parse("0.45").unwrap().to_string());
        assert_eq!("-.5", Dec::parse("-0.50").unwrap().to_string());
        assert_eq!("12", Dec::parse("12.0").unwrap().to_string());
        assert_eq!("0", Dec::parse("0.0").unwrap().to_string());
        assert!(Dec::parse("1e3").is_none());
        assert!(Dec::parse("#fff").is_none());
    }

    #[test]
    fn path_data() {
        let segs =
            parse_path("M77.8,199.8 C97.7 194.2, 137.1 179.3, 157.4 177.2 L157.4 365 L77.8 365 Z")
                .unwrap();
        assert_eq!(
            "M77.8 199.8c19.9-5.6 59.3-20.5 79.6-22.6V365H77.8z",
            write_path(&segs)
        );
        // Implicit repeats and the `.5.5` separator rule.
        let segs = parse_path("M0 0 L1.5 0.5 L3 1 L3 1.5").unwrap();
        assert_eq!("M0 0l1.5.5L3 1v.5", write_path(&segs));
        // Relative input is understood, arcs keep their flags.
        let segs = parse_path("m10 10 h5 v5 a2 2 0 0 1 4 0").unwrap();
        assert_eq!("M10 10h5v5a2 2 0 0 1 4 0", write_path(&segs));
        // A merged second subpath starts with an absolute move.
        let segs = parse_path("M38 87 L550 87 M38 133.3 L550 133.3").unwrap();
        assert_eq!("M38 87h512M38 133.3h512", write_path(&segs));
    }

    #[test]
    fn shapes_and_groups() {
        let svg = concat!(
            "<svg width=\"10\" height=\"10\" xmlns=\"http://www.w3.org/2000/svg\">\n",
            "<rect x=\"0\" y=\"0\" width=\"10\" height=\"10\" fill=\"#FFFFFF\"/>\n",
            "<g stroke=\"#E0E6F2\" stroke-width=\"1\">\n<line x1=\"1\" y1=\"2\" x2=\"9\" y2=\"2\"/>\n",
            "<line x1=\"1\" y1=\"4\" x2=\"9\" y2=\"4\"/>\n</g>\n",
            "<g>\n<text font-size=\"14\" x=\"2\" y=\"3\" font-family=\"Roboto\" fill=\"#464646\">\nA\n</text>\n",
            "<text font-size=\"14\" x=\"2\" y=\"6\" font-family=\"Roboto\" fill=\"#464646\">\nB &amp; C\n</text>\n</g>\n",
            "<polygon points=\"1,1 3,1 3,3\" fill=\"#5470C6\" fill-opacity=\"0.45\" data-series=\"x\"/>\n",
            "<circle cx=\"5\" cy=\"5\" r=\"2\" stroke-width=\"1\" stroke=\"#000000\" fill-opacity=\"1\" fill=\"#123456\"/>\n",
            "</svg>"
        );
        let out = compact_svg(svg);
        assert_eq!(
            concat!(
                "<svg width=\"10\" height=\"10\" xmlns=\"http://www.w3.org/2000/svg\" font-family=\"Roboto\">",
                "<path d=\"M0 0h10v10H0z\" fill=\"#FFF\"/>",
                "<g stroke=\"#E0E6F2\"><path d=\"M1 2h8M1 4h8\"/></g>",
                "<g fill=\"#464646\" font-size=\"14\"><text x=\"2\" y=\"3\">A</text><text x=\"2\" y=\"6\">B &amp; C</text></g>",
                "<path d=\"M1 1h2v2z\" fill=\"#5470C6\" fill-opacity=\".45\" data-series=\"x\"/>",
                "<circle cx=\"5\" cy=\"5\" r=\"2\" stroke=\"#000\" fill=\"#123456\"/>",
                "</svg>"
            ),
            out
        );
    }

    #[test]
    fn keeps_what_it_does_not_understand() {
        assert_eq!("<!-- x --><svg/>", compact_svg("<!-- x --><svg/>"));
        assert_eq!("not svg", compact_svg("not svg"));
    }
}
