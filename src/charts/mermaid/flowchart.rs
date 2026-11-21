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

//! Mermaid `flowchart` / `graph` parser and renderer.

use std::collections::HashMap;

use super::super::component::{RawPath, Rect};
use super::super::error::{Error, Result};
use super::super::theme::get_theme;
use super::MermaidChart;
use super::draw::{
    EdgeMarker, EdgeStroke, NodeShape, build_smooth_edge_path, clip_point_to_rect,
    clip_point_to_shape, draw_edge_label, draw_edge_marker, draw_node,
};
use super::layout::{
    LayeredEdgeInput, LayeredGraph, LayeredNodeInput, LayeredSubgraphInput, sugiyama,
};
use super::preprocess::{Direction, SourceLine};
use super::style::{MermaidPalette, StyleProps, parse_style_props, unquote_text};
use super::text::{FormattedText, draw_multiline_text, format_and_measure};

#[derive(Clone, Debug)]
struct FcNode {
    id: String,
    label: Option<String>,
    shape: Option<NodeShape>,
    subgraph: Option<usize>,
    classes: Vec<String>,
    inline_style: StyleProps,
}

#[derive(Clone, Debug)]
enum Endpoint {
    Node(usize),
    Subgraph(usize),
}

#[derive(Clone, Debug)]
struct FcEdge {
    from: Endpoint,
    to: Endpoint,
    stroke: EdgeStroke,
    start_marker: EdgeMarker,
    end_marker: EdgeMarker,
    label: Option<String>,
    min_len: usize,
}

#[derive(Clone, Debug)]
struct FcSubgraph {
    id: String,
    title: String,
    parent: Option<usize>,
    inline_style: StyleProps,
}

struct ParsedFlowchart {
    direction: Direction,
    nodes: Vec<FcNode>,
    node_map: HashMap<String, usize>,
    edges: Vec<FcEdge>,
    subgraphs: Vec<FcSubgraph>,
    subgraph_map: HashMap<String, usize>,
    class_defs: HashMap<String, StyleProps>,
    link_styles: Vec<(Option<usize>, StyleProps)>, // None => default
}

impl ParsedFlowchart {
    fn new(direction: Direction) -> Self {
        Self {
            direction,
            nodes: Vec::new(),
            node_map: HashMap::new(),
            edges: Vec::new(),
            subgraphs: Vec::new(),
            subgraph_map: HashMap::new(),
            class_defs: HashMap::new(),
            link_styles: Vec::new(),
        }
    }

    fn ensure_node(
        &mut self,
        id: &str,
        label: Option<String>,
        shape: Option<NodeShape>,
        classes: &[String],
        current_subgraph: Option<usize>,
    ) -> usize {
        if let Some(&idx) = self.node_map.get(id) {
            let n = &mut self.nodes[idx];
            if let Some(l) = label {
                n.label = Some(l);
            }
            if let Some(s) = shape {
                n.shape = Some(s);
            }
            for cls in classes {
                if !n.classes.contains(cls) {
                    n.classes.push(cls.clone());
                }
            }
            if n.subgraph.is_none() && current_subgraph.is_some() {
                n.subgraph = current_subgraph;
            }
            return idx;
        }
        let idx = self.nodes.len();
        self.node_map.insert(id.to_string(), idx);
        self.nodes.push(FcNode {
            id: id.to_string(),
            label,
            shape,
            subgraph: current_subgraph,
            classes: classes.to_vec(),
            inline_style: StyleProps::default(),
        });
        idx
    }

    fn resolve_endpoint(&mut self, token: &ParsedNodeToken, current_sg: Option<usize>) -> Endpoint {
        if token.label.is_none()
            && token.shape.is_none()
            && let Some(&sg_idx) = self.subgraph_map.get(&token.id)
        {
            return Endpoint::Subgraph(sg_idx);
        }
        let idx = self.ensure_node(
            &token.id,
            token.label.clone(),
            token.shape,
            &token.classes,
            current_sg,
        );
        Endpoint::Node(idx)
    }
}

#[derive(Clone, Debug)]
struct ParsedNodeToken {
    id: String,
    label: Option<String>,
    shape: Option<NodeShape>,
    classes: Vec<String>,
}

/// Splits a statement line by `;` outside quotes and shape delimiters.
fn split_statements(line: &str) -> Vec<&str> {
    let mut out = Vec::new();
    let bytes = line.as_bytes();
    let mut start = 0;
    let mut i = 0;
    let mut in_quotes = false;
    let mut bracket = 0i32;
    let mut paren = 0i32;
    let mut brace = 0i32;

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
        match b {
            b'"' => in_quotes = true,
            b'[' => bracket += 1,
            b']' => bracket = (bracket - 1).max(0),
            b'(' => paren += 1,
            b')' => paren = (paren - 1).max(0),
            b'{' => brace += 1,
            b'}' => brace = (brace - 1).max(0),
            b';' if bracket == 0 && paren == 0 && brace == 0 => {
                let s = line[start..i].trim();
                if !s.is_empty() {
                    out.push(s);
                }
                start = i + 1;
            }
            _ => {}
        }
        i += 1;
    }
    let tail = line[start..].trim();
    if !tail.is_empty() {
        out.push(tail);
    }
    out
}

fn unquote_label(raw: &str) -> String {
    let t = raw.trim();
    if t.len() >= 2 && t.starts_with('"') && t.ends_with('"') {
        t[1..t.len() - 1].replace("\\\"", "\"")
    } else {
        t.to_string()
    }
}

struct ParsedEdgeOp {
    stroke: EdgeStroke,
    start_marker: EdgeMarker,
    end_marker: EdgeMarker,
    label: Option<String>,
    min_len: usize,
}

impl ParsedEdgeOp {
    fn new(
        stroke: EdgeStroke,
        start_marker: EdgeMarker,
        end_marker: EdgeMarker,
        label: Option<String>,
        min_len: usize,
    ) -> Self {
        Self {
            stroke,
            start_marker,
            end_marker,
            label,
            min_len,
        }
    }
}

/// Scanner over a single flowchart statement for nodes, chains, `&` groups and edges.
struct StmtScanner<'a> {
    src: &'a str,
    pos: usize,
    line_no: usize,
}

impl<'a> StmtScanner<'a> {
    fn new(src: &'a str, line_no: usize) -> Self {
        Self {
            src,
            pos: 0,
            line_no,
        }
    }

    fn rest(&self) -> &'a str {
        &self.src[self.pos..]
    }

    fn skip_ws(&mut self) {
        while let Some(ch) = self.rest().chars().next() {
            if ch.is_whitespace() {
                self.pos += ch.len_utf8();
            } else {
                break;
            }
        }
    }

    fn starts_edge_at(&self, offset: usize) -> bool {
        let s = &self.src[offset..];
        s.starts_with("-->")
            || s.starts_with("---")
            || s.starts_with("-.")
            || s.starts_with("--")
            || s.starts_with("->")
            || s.starts_with("==")
            || s.starts_with("~~~")
            || s.starts_with("<--")
            || s.starts_with("<==")
            || s.starts_with("<-.")
            || s.starts_with("o--")
            || s.starts_with("x--")
    }

    fn read_delimited(&mut self, open: &str, close: &str) -> Result<Option<String>> {
        if !self.rest().starts_with(open) {
            return Ok(None);
        }
        let open_pos = self.pos;
        self.pos += open.len();
        let content_start = self.pos;
        let mut in_quotes = false;
        while self.pos < self.src.len() {
            let rest = self.rest();
            if in_quotes {
                if rest.starts_with("\\\"") {
                    self.pos += 2;
                    continue;
                }
                if rest.starts_with('"') {
                    in_quotes = false;
                }
                let ch = rest.chars().next().unwrap();
                self.pos += ch.len_utf8();
                continue;
            }
            if rest.starts_with('"') {
                in_quotes = true;
                self.pos += 1;
                continue;
            }
            if rest.starts_with(close) {
                let inner = &self.src[content_start..self.pos];
                self.pos += close.len();
                return Ok(Some(unquote_label(inner)));
            }
            let ch = rest.chars().next().unwrap();
            self.pos += ch.len_utf8();
        }
        Err(Error::Mermaid {
            line: Some(self.line_no),
            message: format!(
                "unclosed node shape delimiter `{open}` starting near `{}`",
                &self.src[open_pos..]
            ),
        })
    }

    fn read_v11_shape_block(&mut self) -> Result<(Option<NodeShape>, Option<String>)> {
        let open_pos = self.pos;
        self.pos += 2; // consume `@{`
        let start = self.pos;
        let bytes = self.src.as_bytes();
        let mut in_quote: Option<u8> = None;
        let mut depth = 1usize;
        let mut end = None;
        while self.pos < bytes.len() {
            let b = bytes[self.pos];
            if let Some(q) = in_quote {
                if b == b'\\' && self.pos + 1 < bytes.len() {
                    self.pos += 2;
                    continue;
                }
                if b == q {
                    in_quote = None;
                }
                self.pos += 1;
                continue;
            }
            match b {
                b'"' | b'\'' => in_quote = Some(b),
                b'{' => depth += 1,
                b'}' => {
                    depth -= 1;
                    if depth == 0 {
                        end = Some(self.pos);
                        self.pos += 1;
                        break;
                    }
                }
                _ => {}
            }
            self.pos += 1;
        }
        let Some(end_pos) = end else {
            return Err(Error::Mermaid {
                line: Some(self.line_no),
                message: format!("unclosed v11 shape block starting near `{}`", &self.src[open_pos..]),
            });
        };
        let body = &self.src[start..end_pos];
        let mut shape: Option<NodeShape> = None;
        let mut label: Option<String> = None;

        // Split top-level comma-separated `key: value` pairs outside quotes.
        let mut entries = Vec::new();
        let mut cur_start = 0usize;
        let mut q: Option<u8> = None;
        let bbody = body.as_bytes();
        let mut i = 0;
        while i < bbody.len() {
            let b = bbody[i];
            if let Some(quote) = q {
                if b == b'\\' && i + 1 < bbody.len() {
                    i += 2;
                    continue;
                }
                if b == quote {
                    q = None;
                }
                i += 1;
                continue;
            }
            if b == b'"' || b == b'\'' {
                q = Some(b);
            } else if b == b',' {
                entries.push(body[cur_start..i].trim());
                cur_start = i + 1;
            }
            i += 1;
        }
        let tail = body[cur_start..].trim();
        if !tail.is_empty() {
            entries.push(tail);
        }

        for entry in entries {
            if entry.is_empty() {
                continue;
            }
            let Some((raw_k, raw_v)) = entry.split_once(':') else {
                return Err(Error::Mermaid {
                    line: Some(self.line_no),
                    message: format!("invalid v11 node property `{entry}` (expected `key: value`)"),
                });
            };
            let key = unquote_text(raw_k.trim()).to_ascii_lowercase();
            let val = unquote_text(raw_v.trim());
            match key.as_str() {
                "shape" => {
                    shape = Some(parse_v11_shape(&val, self.line_no)?);
                }
                "label" => {
                    label = Some(val);
                }
                _ => {}
            }
        }
        if matches!(shape, Some(NodeShape::SmallCircle | NodeShape::FramedCircle))
            && label.is_none()
        {
            label = Some(String::new());
        }
        Ok((shape, label))
    }

    /// Reads `[/...]` or `[\...]` where the closing delimiter may be either
    /// `/]` or `\]` (Parallelogram vs Trapezoid).
    fn read_slash_delimited(&mut self, open: &str) -> Result<(NodeShape, String)> {
        let open_pos = self.pos;
        self.pos += open.len();
        let content_start = self.pos;
        let mut in_quotes = false;
        while self.pos < self.src.len() {
            let rest = self.rest();
            if in_quotes {
                if rest.starts_with("\\\"") {
                    self.pos += 2;
                    continue;
                }
                if rest.starts_with('"') {
                    in_quotes = false;
                }
                let ch = rest.chars().next().unwrap();
                self.pos += ch.len_utf8();
                continue;
            }
            if rest.starts_with('"') {
                in_quotes = true;
                self.pos += 1;
                continue;
            }
            if rest.starts_with("/]") || rest.starts_with("\\]") {
                let inner = &self.src[content_start..self.pos];
                let shape = match (open, &rest[..2]) {
                    ("[/", "/]") => NodeShape::Parallelogram,
                    ("[\\", "\\]") => NodeShape::ParallelogramAlt,
                    ("[/", "\\]") => NodeShape::Trapezoid,
                    _ => NodeShape::TrapezoidAlt,
                };
                self.pos += 2;
                return Ok((shape, unquote_label(inner)));
            }
            let ch = rest.chars().next().unwrap();
            self.pos += ch.len_utf8();
        }
        Err(Error::Mermaid {
            line: Some(self.line_no),
            message: format!(
                "unclosed node shape delimiter `{open}` starting near `{}`",
                &self.src[open_pos..]
            ),
        })
    }

    fn parse_node_ref(&mut self) -> Result<ParsedNodeToken> {
        self.skip_ws();
        let id_start = self.pos;
        while let Some(ch) = self.rest().chars().next() {
            if ch.is_alphanumeric() || ch == '_' || ch == '.' {
                self.pos += ch.len_utf8();
            } else if ch == '-' {
                if self.starts_edge_at(self.pos) {
                    break;
                }
                self.pos += 1;
            } else {
                break;
            }
        }
        if self.pos == id_start {
            return Err(Error::Mermaid {
                line: Some(self.line_no),
                message: format!("expected node identifier near `{}`", self.rest()),
            });
        }
        let id = self.src[id_start..self.pos].to_string();

        // Optional shape delimiter (longest openers first).
        let delimiters: &[(&str, &str, NodeShape)] = &[
            ("(((", ")))", NodeShape::DoubleCircle),
            ("((", "))", NodeShape::Circle),
            ("([", "])", NodeShape::Stadium),
            ("[[", "]]", NodeShape::Subroutine),
            ("[(", ")]", NodeShape::Cylinder),
            ("{{", "}}", NodeShape::Hexagon),
            ("[", "]", NodeShape::Rect),
            ("(", ")", NodeShape::Rounded),
            ("{", "}", NodeShape::Diamond),
            (">", "]", NodeShape::Asymmetric),
        ];

        let mut shape = None;
        let mut label = None;
        if self.rest().starts_with("@{") {
            let (v11_shape, v11_label) = self.read_v11_shape_block()?;
            shape = v11_shape;
            label = v11_label;
        } else if self.rest().starts_with("[/") {
            let (s, l) = self.read_slash_delimited("[/")?;
            shape = Some(s);
            label = Some(l);
        } else if self.rest().starts_with("[\\") {
            let (s, l) = self.read_slash_delimited("[\\")?;
            shape = Some(s);
            label = Some(l);
        } else {
            for &(open, close, kind) in delimiters {
                if self.rest().starts_with(open) {
                    let text = self.read_delimited(open, close)?.unwrap_or_default();
                    shape = Some(kind);
                    label = Some(text);
                    break;
                }
            }
        }

        // Optional `:::className` inline class attachment.
        let mut classes = Vec::new();
        while self.rest().starts_with(":::") {
            self.pos += 3;
            let cls_start = self.pos;
            while let Some(ch) = self.rest().chars().next() {
                if ch.is_alphanumeric() || ch == '_' || ch == '-' {
                    if ch == '-' && self.starts_edge_at(self.pos) {
                        break;
                    }
                    self.pos += ch.len_utf8();
                } else {
                    break;
                }
            }
            if self.pos > cls_start {
                classes.push(self.src[cls_start..self.pos].to_string());
            }
        }

        Ok(ParsedNodeToken {
            id,
            label,
            shape,
            classes,
        })
    }

    fn parse_node_group(&mut self) -> Result<Vec<ParsedNodeToken>> {
        let mut nodes = vec![self.parse_node_ref()?];
        loop {
            let saved = self.pos;
            self.skip_ws();
            if self.rest().starts_with('&') {
                self.pos += 1;
                nodes.push(self.parse_node_ref()?);
            } else {
                self.pos = saved;
                break;
            }
        }
        Ok(nodes)
    }

    fn read_pipe_label(&mut self) -> Result<Option<String>> {
        let saved = self.pos;
        self.skip_ws();
        if !self.rest().starts_with('|') {
            self.pos = saved;
            return Ok(None);
        }
        self.pos += 1;
        let start = self.pos;
        while self.pos < self.src.len() {
            let rest = self.rest();
            if rest.starts_with('|') {
                let raw = &self.src[start..self.pos];
                self.pos += 1;
                return Ok(Some(unquote_label(raw)));
            }
            let ch = rest.chars().next().unwrap();
            self.pos += ch.len_utf8();
        }
        Err(Error::Mermaid {
            line: Some(self.line_no),
            message: "unclosed `|...|` edge label".to_string(),
        })
    }

    fn parse_edge_op(&mut self) -> Result<Option<ParsedEdgeOp>> {
        self.skip_ws();
        if self.rest().is_empty() {
            return Ok(None);
        }
        let rest = self.rest();

        // Invisible links `~~~`
        if rest.starts_with("~~~") {
            let mut len = 0usize;
            while self.rest().starts_with('~') {
                self.pos += 1;
                len += 1;
            }
            return Ok(Some(ParsedEdgeOp::new(
                EdgeStroke::Invisible,
                EdgeMarker::None,
                EdgeMarker::None,
                None,
                len.saturating_sub(2).max(1),
            )));
        }

        // Dotted links: `<-.->`, `-.->`, `-.-`, `-. text .->`, `-. text .-`
        if rest.starts_with("<-.") || rest.starts_with("-.") {
            let start_arrow = if rest.starts_with('<') {
                self.pos += 1;
                EdgeMarker::Arrow
            } else {
                EdgeMarker::None
            };
            self.pos += 2; // `-.`
            // Check pure `-.->` / `-.-`
            if self.rest().starts_with("->") {
                self.pos += 2;
                let label = self.read_pipe_label()?;
                return Ok(Some(ParsedEdgeOp::new(
                    EdgeStroke::Dotted,
                    start_arrow,
                    EdgeMarker::Arrow,
                    label,
                    1,
                )));
            }
            if self.rest().starts_with('-') {
                self.pos += 1;
                if self.rest().starts_with('>') {
                    self.pos += 1;
                    let label = self.read_pipe_label()?;
                    return Ok(Some(ParsedEdgeOp::new(
                        EdgeStroke::Dotted,
                        start_arrow,
                        EdgeMarker::Arrow,
                        label,
                        1,
                    )));
                }
                let label = self.read_pipe_label()?;
                return Ok(Some(ParsedEdgeOp::new(
                    EdgeStroke::Dotted,
                    start_arrow,
                    EdgeMarker::None,
                    label,
                    1,
                )));
            }
            // Inline text `-. text .->` or `-. text .-`
            if let Some(end_rel) = self.rest().find(".->") {
                let txt = unquote_label(&self.rest()[..end_rel]);
                self.pos += end_rel + 3;
                return Ok(Some(ParsedEdgeOp::new(
                    EdgeStroke::Dotted,
                    start_arrow,
                    EdgeMarker::Arrow,
                    (!txt.is_empty()).then_some(txt),
                    1,
                )));
            }
            if let Some(end_rel) = self.rest().find(".-") {
                let txt = unquote_label(&self.rest()[..end_rel]);
                self.pos += end_rel + 2;
                return Ok(Some(ParsedEdgeOp::new(
                    EdgeStroke::Dotted,
                    start_arrow,
                    EdgeMarker::None,
                    (!txt.is_empty()).then_some(txt),
                    1,
                )));
            }
            return Err(Error::Mermaid {
                line: Some(self.line_no),
                message: format!("malformed dotted edge near `{rest}`"),
            });
        }

        // Thick links: `<==>`, `==>`, `===`, `== text ==>`, `== text ===`
        if rest.starts_with("<==") || rest.starts_with("==") {
            let start_marker = if rest.starts_with('<') {
                self.pos += 1;
                EdgeMarker::Arrow
            } else {
                EdgeMarker::None
            };
            let mut eq_count = 0usize;
            while self.rest().starts_with('=') {
                self.pos += 1;
                eq_count += 1;
            }
            if self.rest().starts_with('>') {
                self.pos += 1;
                let label = self.read_pipe_label()?;
                return Ok(Some(ParsedEdgeOp::new(
                    EdgeStroke::Thick,
                    start_marker,
                    EdgeMarker::Arrow,
                    label,
                    eq_count.saturating_sub(1).max(1),
                )));
            }
            // If `===` without text (followed by whitespace or node)
            if eq_count >= 3
                && (self.rest().is_empty() || self.rest().starts_with(char::is_whitespace))
            {
                let label = self.read_pipe_label()?;
                return Ok(Some(ParsedEdgeOp::new(
                    EdgeStroke::Thick,
                    start_marker,
                    EdgeMarker::None,
                    label,
                    eq_count.saturating_sub(2).max(1),
                )));
            }
            // Inline label `== text ==>` or `== text ===`
            if let Some(end_rel) = self.rest().find("==>") {
                let txt = unquote_label(&self.rest()[..end_rel]);
                self.pos += end_rel + 3;
                return Ok(Some(ParsedEdgeOp::new(
                    EdgeStroke::Thick,
                    start_marker,
                    EdgeMarker::Arrow,
                    (!txt.is_empty()).then_some(txt),
                    1,
                )));
            }
            if let Some(end_rel) = self.rest().find("===") {
                let txt = unquote_label(&self.rest()[..end_rel]);
                self.pos += end_rel + 3;
                return Ok(Some(ParsedEdgeOp::new(
                    EdgeStroke::Thick,
                    start_marker,
                    EdgeMarker::None,
                    (!txt.is_empty()).then_some(txt),
                    1,
                )));
            }
            return Err(Error::Mermaid {
                line: Some(self.line_no),
                message: format!("malformed thick edge near `{rest}`"),
            });
        }

        // Solid links: `<-->`, `o--o`, `x--x`, `-->`, `---`, `--o`, `--x`, `-- text -->`, `->`
        let start_marker = if rest.starts_with("<--") {
            self.pos += 1;
            EdgeMarker::Arrow
        } else if rest.starts_with("o--") {
            self.pos += 1;
            EdgeMarker::Circle
        } else if rest.starts_with("x--") {
            self.pos += 1;
            EdgeMarker::Cross
        } else {
            EdgeMarker::None
        };

        if self.rest().starts_with('-') {
            let mut dash_count = 0usize;
            while self.rest().starts_with('-') {
                self.pos += 1;
                dash_count += 1;
            }
            let end_marker = if self.rest().starts_with('>') {
                self.pos += 1;
                Some(EdgeMarker::Arrow)
            } else if self.rest().starts_with('o') && dash_count >= 2 {
                self.pos += 1;
                Some(EdgeMarker::Circle)
            } else if self.rest().starts_with('x') && dash_count >= 2 {
                self.pos += 1;
                Some(EdgeMarker::Cross)
            } else {
                None
            };
            if let Some(end_m) = end_marker {
                let label = self.read_pipe_label()?;
                let min_len = dash_count.saturating_sub(1).max(1);
                return Ok(Some(ParsedEdgeOp::new(
                    EdgeStroke::Solid,
                    start_marker,
                    end_m,
                    label,
                    min_len,
                )));
            }
            if dash_count >= 3 {
                let label = self.read_pipe_label()?;
                let min_len = dash_count.saturating_sub(2).max(1);
                return Ok(Some(ParsedEdgeOp::new(
                    EdgeStroke::Solid,
                    start_marker,
                    EdgeMarker::None,
                    label,
                    min_len,
                )));
            }
            if dash_count == 2 {
                // Inline label `-- text -->` or `-- text ---`
                if let Some(end_rel) = self.rest().find("-->") {
                    let txt = unquote_label(&self.rest()[..end_rel]);
                    self.pos += end_rel + 3;
                    return Ok(Some(ParsedEdgeOp::new(
                        EdgeStroke::Solid,
                        start_marker,
                        EdgeMarker::Arrow,
                        (!txt.is_empty()).then_some(txt),
                        1,
                    )));
                }
                if let Some(end_rel) = self.rest().find("---") {
                    let txt = unquote_label(&self.rest()[..end_rel]);
                    self.pos += end_rel + 3;
                    return Ok(Some(ParsedEdgeOp::new(
                        EdgeStroke::Solid,
                        start_marker,
                        EdgeMarker::None,
                        (!txt.is_empty()).then_some(txt),
                        1,
                    )));
                }
            }
        }

        Err(Error::Mermaid {
            line: Some(self.line_no),
            message: format!("unexpected syntax in flowchart near `{}`", self.rest()),
        })
    }
}

fn parse_subgraph_header(rest: &str, line_no: usize) -> Result<(String, String)> {
    let r = rest.trim();
    if r.is_empty() {
        return Err(Error::Mermaid {
            line: Some(line_no),
            message: "expected subgraph identifier or title".to_string(),
        });
    }
    if r.starts_with('"') {
        let title = unquote_label(r);
        let id = title.clone();
        return Ok((id, title));
    }
    if let Some(bracket) = r.find('[')
        && r.ends_with(']')
    {
        let id = r[..bracket].trim();
        let title = unquote_label(&r[bracket + 1..r.len() - 1]);
        let id = if id.is_empty() {
            title.clone()
        } else {
            id.to_string()
        };
        return Ok((id, title));
    }
    if let Some((first, tail)) = r.split_once(char::is_whitespace) {
        let tail = tail.trim();
        let title = if tail.starts_with('[') && tail.ends_with(']') {
            unquote_label(&tail[1..tail.len() - 1])
        } else {
            unquote_label(tail)
        };
        return Ok((first.to_string(), title));
    }
    Ok((r.to_string(), r.to_string()))
}

fn parse_flowchart(direction: Direction, lines: &[SourceLine]) -> Result<ParsedFlowchart> {
    let mut fc = ParsedFlowchart::new(direction);
    let mut sg_stack: Vec<(usize, usize)> = Vec::new(); // (subgraph_idx, opening line_no)
    let mut in_acc_descr_block = false;

    for sl in lines {
        for stmt in split_statements(&sl.text) {
            let trimmed = stmt.trim();
            if trimmed.is_empty() {
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

            if trimmed == "end" {
                if sg_stack.pop().is_none() {
                    return Err(Error::Mermaid {
                        line: Some(sl.line_no),
                        message: "unexpected `end` with no matching `subgraph`".to_string(),
                    });
                }
                continue;
            }

            if let Some(rest) = trimmed.strip_prefix("subgraph")
                && rest.starts_with(char::is_whitespace)
            {
                let (id, title) = parse_subgraph_header(rest, sl.line_no)?;
                let parent = sg_stack.last().map(|&(idx, _)| idx);
                let sg_idx = fc.subgraphs.len();
                fc.subgraph_map.insert(id.clone(), sg_idx);
                fc.subgraphs.push(FcSubgraph {
                    id,
                    title,
                    parent,
                    inline_style: StyleProps::default(),
                });
                sg_stack.push((sg_idx, sl.line_no));
                continue;
            }

            if let Some(rest) = trimmed.strip_prefix("direction")
                && rest.starts_with(char::is_whitespace)
            {
                let dir_tok = rest.trim();
                let dir = Direction::from_token(dir_tok).ok_or_else(|| Error::Mermaid {
                    line: Some(sl.line_no),
                    message: format!("unknown direction `{dir_tok}`"),
                })?;
                if sg_stack.is_empty() {
                    fc.direction = dir;
                }
                continue;
            }

            if let Some(rest) = trimmed.strip_prefix("classDef")
                && rest.starts_with(char::is_whitespace)
            {
                let rest = rest.trim();
                let Some((cls_name, css)) = rest.split_once(char::is_whitespace) else {
                    return Err(Error::Mermaid {
                        line: Some(sl.line_no),
                        message: "expected `classDef <name> <styles>`".to_string(),
                    });
                };
                let props = parse_style_props(css.trim());
                fc.class_defs
                    .entry(cls_name.trim().to_string())
                    .or_default()
                    .merge_from(&props);
                continue;
            }

            if let Some(rest) = trimmed.strip_prefix("class")
                && rest.starts_with(char::is_whitespace)
            {
                let rest = rest.trim();
                let Some((ids_raw, cls_raw)) = rest.split_once(char::is_whitespace) else {
                    return Err(Error::Mermaid {
                        line: Some(sl.line_no),
                        message: "expected `class <nodeIds> <className>`".to_string(),
                    });
                };
                let cls_names: Vec<String> = cls_raw
                    .split(',')
                    .map(str::trim)
                    .filter(|s| !s.is_empty())
                    .map(str::to_string)
                    .collect();
                let current_sg = sg_stack.last().map(|&(idx, _)| idx);
                for id in ids_raw.split(',').map(str::trim).filter(|s| !s.is_empty()) {
                    let idx = fc.ensure_node(id, None, None, &cls_names, current_sg);
                    for cls in &cls_names {
                        if !fc.nodes[idx].classes.contains(cls) {
                            fc.nodes[idx].classes.push(cls.clone());
                        }
                    }
                }
                continue;
            }

            if let Some(rest) = trimmed.strip_prefix("style")
                && rest.starts_with(char::is_whitespace)
            {
                let rest = rest.trim();
                let Some((id_raw, css)) = rest.split_once(char::is_whitespace) else {
                    return Err(Error::Mermaid {
                        line: Some(sl.line_no),
                        message: "expected `style <id> <styles>`".to_string(),
                    });
                };
                let id = id_raw.trim();
                let props = parse_style_props(css.trim());
                if let Some(&sg_idx) = fc.subgraph_map.get(id) {
                    fc.subgraphs[sg_idx].inline_style.merge_from(&props);
                } else {
                    let current_sg = sg_stack.last().map(|&(idx, _)| idx);
                    let idx = fc.ensure_node(id, None, None, &[], current_sg);
                    fc.nodes[idx].inline_style.merge_from(&props);
                }
                continue;
            }

            if let Some(rest) = trimmed.strip_prefix("linkStyle")
                && rest.starts_with(char::is_whitespace)
            {
                let rest = rest.trim();
                let Some((target_raw, css)) = rest.split_once(char::is_whitespace) else {
                    return Err(Error::Mermaid {
                        line: Some(sl.line_no),
                        message: "expected `linkStyle <index|default> <styles>`".to_string(),
                    });
                };
                let target_raw = target_raw.trim();
                let props = parse_style_props(css.trim());
                if target_raw.eq_ignore_ascii_case("default") {
                    fc.link_styles.push((None, props));
                } else {
                    for part in target_raw
                        .split(',')
                        .map(str::trim)
                        .filter(|s| !s.is_empty())
                    {
                        let idx: usize = part.parse().map_err(|_| Error::Mermaid {
                            line: Some(sl.line_no),
                            message: format!("invalid link index `{part}` in linkStyle"),
                        })?;
                        fc.link_styles.push((Some(idx), props.clone()));
                    }
                }
                continue;
            }

            // Node and/or chained edge statement (`A & B --> C --> D`).
            let current_sg = sg_stack.last().map(|&(idx, _)| idx);
            let mut scanner = StmtScanner::new(trimmed, sl.line_no);
            let mut lhs = scanner.parse_node_group()?;
            let mut had_edge = false;

            while let Some(op) = scanner.parse_edge_op()? {
                had_edge = true;
                let rhs = scanner.parse_node_group()?;
                for l in &lhs {
                    let from_ep = fc.resolve_endpoint(l, current_sg);
                    for r in &rhs {
                        let to_ep = fc.resolve_endpoint(r, current_sg);
                        fc.edges.push(FcEdge {
                            from: from_ep.clone(),
                            to: to_ep,
                            stroke: op.stroke,
                            start_marker: op.start_marker,
                            end_marker: op.end_marker,
                            label: op.label.clone(),
                            min_len: op.min_len,
                        });
                    }
                }
                lhs = rhs;
            }

            if !had_edge {
                for tok in &lhs {
                    fc.resolve_endpoint(tok, current_sg);
                }
            }
        }
    }

    if let Some(&(sg_idx, line_no)) = sg_stack.last() {
        return Err(Error::Mermaid {
            line: Some(line_no),
            message: format!(
                "unclosed `subgraph {}` (missing `end`)",
                fc.subgraphs[sg_idx].id
            ),
        });
    }

    Ok(fc)
}

fn parse_v11_shape(name: &str, line_no: usize) -> Result<NodeShape> {
    match name.trim().to_ascii_lowercase().as_str() {
        "rect" | "process" | "rectangle" => Ok(NodeShape::Rect),
        "rounded" | "event" => Ok(NodeShape::Rounded),
        "stadium" | "terminal" | "pill" => Ok(NodeShape::Stadium),
        "subproc" | "fr-rect" | "subprocess" => Ok(NodeShape::Subroutine),
        "cyl" | "cylinder" | "database" | "db" => Ok(NodeShape::Cylinder),
        "circle" | "circ" => Ok(NodeShape::Circle),
        "dbl-circ" | "double-circle" | "doublecircle" => Ok(NodeShape::DoubleCircle),
        "diam" | "diamond" | "decision" => Ok(NodeShape::Diamond),
        "hex" | "hexagon" | "prepare" => Ok(NodeShape::Hexagon),
        "lean-r" | "in-out" => Ok(NodeShape::Parallelogram),
        "lean-l" | "out-in" => Ok(NodeShape::ParallelogramAlt),
        "trap-b" | "priority" => Ok(NodeShape::Trapezoid),
        "trap-t" | "manual" => Ok(NodeShape::TrapezoidAlt),
        "odd" | "asym" | "asymmetric" => Ok(NodeShape::Asymmetric),
        "doc" | "document" => Ok(NodeShape::Doc),
        "docs" | "documents" => Ok(NodeShape::Docs),
        "delay" | "half-rounded-rectangle" => Ok(NodeShape::Delay),
        "tri" | "triangle" | "extract" => Ok(NodeShape::Triangle),
        "flag" | "notch-pent" => Ok(NodeShape::Flag),
        "sm-circ" | "start" => Ok(NodeShape::SmallCircle),
        "f-circ" | "junction" => Ok(NodeShape::FramedCircle),
        "text" => Ok(NodeShape::TextOnly),
        other => Err(Error::Mermaid {
            line: Some(line_no),
            message: format!("unknown flowchart v11 shape `{other}`"),
        }),
    }
}

fn node_box_size(shape: NodeShape, label: &FormattedText) -> (f32, f32) {
    let text_w = label.width.max(16.0);
    let text_h = label.height.max(18.0);
    match shape {
        NodeShape::Diamond => (
            (text_w * 1.55 + 32.0).max(56.0),
            (text_h * 1.55 + 28.0).max(52.0),
        ),
        NodeShape::Circle | NodeShape::DoubleCircle => {
            let side = text_w.max(text_h)
                + 26.0
                + if shape == NodeShape::DoubleCircle {
                    10.0
                } else {
                    0.0
                };
            let side = side.max(46.0);
            (side, side)
        }
        NodeShape::SmallCircle => (20.0, 20.0),
        NodeShape::FramedCircle => (24.0, 24.0),
        NodeShape::Triangle => (
            (text_w * 1.6 + 34.0).max(66.0),
            (text_h * 1.6 + 32.0).max(56.0),
        ),
        NodeShape::Doc => ((text_w + 30.0).max(62.0), (text_h + 28.0).max(46.0)),
        NodeShape::Docs => ((text_w + 38.0).max(70.0), (text_h + 34.0).max(52.0)),
        NodeShape::Delay => ((text_w + 36.0).max(66.0), (text_h + 20.0).max(38.0)),
        NodeShape::Flag => ((text_w + 42.0).max(74.0), (text_h + 20.0).max(38.0)),
        NodeShape::TextOnly => ((text_w + 12.0).max(24.0), (text_h + 10.0).max(24.0)),
        NodeShape::Hexagon => ((text_w + 42.0).max(72.0), (text_h + 22.0).max(40.0)),
        NodeShape::Cylinder => ((text_w + 30.0).max(60.0), (text_h + 32.0).max(48.0)),
        NodeShape::Subroutine => ((text_w + 38.0).max(68.0), (text_h + 20.0).max(38.0)),
        NodeShape::Parallelogram
        | NodeShape::ParallelogramAlt
        | NodeShape::Trapezoid
        | NodeShape::TrapezoidAlt => ((text_w + 40.0).max(70.0), (text_h + 20.0).max(38.0)),
        NodeShape::Asymmetric => ((text_w + 38.0).max(68.0), (text_h + 20.0).max(38.0)),
        NodeShape::Stadium => ((text_w + 34.0).max(64.0), (text_h + 20.0).max(38.0)),
        NodeShape::Rect | NodeShape::Rounded => {
            ((text_w + 28.0).max(56.0), (text_h + 20.0).max(38.0))
        }
    }
}

/// Returns `true` if the line segment `p0 -> p1` intersects the axis-aligned
/// rectangle `[rx0, rx1] x [ry0, ry1]`.
fn segment_intersects_rect(
    p0: super::super::util::Point,
    p1: super::super::util::Point,
    rx0: f32,
    ry0: f32,
    rx1: f32,
    ry1: f32,
) -> bool {
    let dx = p1.x - p0.x;
    let dy = p1.y - p0.y;
    let mut t0 = 0.0_f32;
    let mut t1 = 1.0_f32;
    for (p, q) in [
        (-dx, p0.x - rx0),
        (dx, rx1 - p0.x),
        (-dy, p0.y - ry0),
        (dy, ry1 - p0.y),
    ] {
        if p.abs() < 1e-6 {
            if q < 0.0 {
                return false;
            }
        } else {
            let r = q / p;
            if p < 0.0 {
                if r > t1 {
                    return false;
                }
                t0 = t0.max(r);
            } else {
                if r < t0 {
                    return false;
                }
                t1 = t1.min(r);
            }
        }
    }
    t0 <= t1
}

pub(crate) fn render(
    chart: &MermaidChart,
    theme_name: &str,
    fm_title: Option<&str>,
    direction: Direction,
    lines: &[SourceLine],
) -> Result<(String, f32, f32)> {
    let mut fc = parse_flowchart(direction, lines)?;
    if fc.nodes.is_empty() && fc.subgraphs.is_empty() {
        return Err(Error::Mermaid {
            line: lines.first().map(|l| l.line_no),
            message: "flowchart has no nodes".to_string(),
        });
    }

    // Create invisible anchor nodes for empty subgraphs so they still lay out.
    let mut sg_anchor_nodes: HashMap<usize, usize> = HashMap::new();
    for sg_idx in 0..fc.subgraphs.len() {
        let has_member = fc.nodes.iter().any(|n| n.subgraph == Some(sg_idx))
            || fc.subgraphs.iter().any(|s| s.parent == Some(sg_idx));
        if !has_member {
            let anchor_idx = fc.nodes.len();
            fc.nodes.push(FcNode {
                id: format!("__sg_anchor_{sg_idx}"),
                label: Some(String::new()),
                shape: Some(NodeShape::Rect),
                subgraph: Some(sg_idx),
                classes: vec![],
                inline_style: StyleProps::default(),
            });
            sg_anchor_nodes.insert(sg_idx, anchor_idx);
        }
    }

    // Pick a representative layout node inside each subgraph for edges targeting a subgraph.
    let mut sg_rep_node: HashMap<usize, usize> = sg_anchor_nodes.clone();
    for sg_idx in (0..fc.subgraphs.len()).rev() {
        if let Some(u) = fc.nodes.iter().position(|n| n.subgraph == Some(sg_idx)) {
            sg_rep_node.entry(sg_idx).or_insert(u);
        } else {
            for (child_idx, child) in fc.subgraphs.iter().enumerate() {
                if child.parent == Some(sg_idx)
                    && let Some(&rep) = sg_rep_node.get(&child_idx)
                {
                    sg_rep_node.insert(sg_idx, rep);
                    break;
                }
            }
        }
    }

    let theme = get_theme(theme_name);
    let bg_override = chart
        .has_custom_background()
        .then_some(chart.base.background_color);
    let palette = MermaidPalette::from_theme(&theme, bg_override);
    let font_family = chart.resolved_font_family(&theme);
    let font_size = 14.0_f32;
    let edge_font_size = 12.0_f32;
    let sg_font_size = 13.0_f32;

    // Measure node labels and subgraph titles.
    let mut node_labels = Vec::with_capacity(fc.nodes.len());
    let mut node_sizes = Vec::with_capacity(fc.nodes.len());
    let mut layout_nodes = Vec::with_capacity(fc.nodes.len());

    for n in &fc.nodes {
        let is_anchor = n.id.starts_with("__sg_anchor_");
        let shape = n.shape.unwrap_or(NodeShape::Rect);
        let text = n.label.as_deref().unwrap_or(&n.id);
        let ft = format_and_measure(text, &font_family, font_size);
        let (w, h) = if is_anchor {
            (40.0, 20.0)
        } else {
            node_box_size(shape, &ft)
        };
        node_labels.push(ft);
        node_sizes.push((w, h));
        layout_nodes.push(LayeredNodeInput {
            width: w,
            height: h,
            subgraph: n.subgraph,
            attach_to: None,
        });
    }

    let mut sg_labels = Vec::with_capacity(fc.subgraphs.len());
    let mut layout_subgraphs = Vec::with_capacity(fc.subgraphs.len());
    for sg in &fc.subgraphs {
        let ft = format_and_measure(&sg.title, &font_family, sg_font_size);
        layout_subgraphs.push(LayeredSubgraphInput {
            parent: sg.parent,
            title_width: ft.width,
            title_height: ft.height,
            concurrent_region: false,
        });
        sg_labels.push(ft);
    }

    let mut layout_edges = Vec::with_capacity(fc.edges.len());
    for e in &fc.edges {
        let from = match e.from {
            Endpoint::Node(u) => u,
            Endpoint::Subgraph(sg) => *sg_rep_node.get(&sg).unwrap_or(&0),
        };
        let to = match e.to {
            Endpoint::Node(v) => v,
            Endpoint::Subgraph(sg) => *sg_rep_node.get(&sg).unwrap_or(&0),
        };
        layout_edges.push(LayeredEdgeInput {
            from,
            to,
            has_label: e.label.as_ref().is_some_and(|l| !l.is_empty()),
            min_len: e.min_len,
        });
    }

    let layout = sugiyama::layout(&LayeredGraph {
        direction: fc.direction,
        rank_sep_extra: 0.0,
        node_sep_extra: 0.0,
        nodes: layout_nodes,
        edges: layout_edges,
        subgraphs: layout_subgraphs,
    });

    // Reserve header room if a title or sub_title is present.
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

    let mut content_canvas = canvas.child(super::super::util::Box {
        left: chart.base.margin.left,
        top: chart.base.margin.top + header_h,
        right: chart.base.margin.right,
        bottom: chart.base.margin.bottom,
    });

    // 1. Subgraph containers (outer-to-inner).
    let mut sg_depths: Vec<(usize, usize)> = (0..fc.subgraphs.len())
        .map(|i| {
            let mut d = 0usize;
            let mut cur = fc.subgraphs[i].parent;
            while let Some(p) = cur {
                d += 1;
                cur = fc.subgraphs[p].parent;
            }
            (d, i)
        })
        .collect();
    sg_depths.sort_by(|a, b| a.0.cmp(&b.0).then_with(|| a.1.cmp(&b.1)));

    for &(depth, sg_idx) in &sg_depths {
        let b = layout.subgraphs[sg_idx];
        if !b.visible {
            continue;
        }
        let sg = &fc.subgraphs[sg_idx];
        let fill = sg
            .inline_style
            .fill
            .unwrap_or_else(|| palette.subgraph_fill(depth));
        let stroke = sg.inline_style.stroke.unwrap_or(palette.subgraph_stroke);
        let stroke_w = sg.inline_style.stroke_width.unwrap_or(1.2);
        let text_color = sg.inline_style.color.unwrap_or(palette.text_color);

        content_canvas.rect(Rect {
            left: b.x,
            top: b.y,
            width: b.width,
            height: b.height,
            rx: Some(6.0),
            ry: Some(6.0),
            fill: Some(fill.into()),
            color: Some(stroke),
            stroke_width: Some(stroke_w),
            stroke_dash_array: sg.inline_style.stroke_dasharray.clone(),
            ..Default::default()
        });

        let title_ft = &sg_labels[sg_idx];
        if !title_ft.lines.is_empty() {
            let ty = b.y + 12.0 + title_ft.height * 0.5;
            let title_intersects_edges = |tx_left: f32, tx_right: f32| -> bool {
                let ry0 = ty - title_ft.height * 0.5 - 4.0;
                let ry1 = ty + title_ft.height * 0.5 + 4.0;
                layout.edges.iter().any(|re| {
                    re.waypoints.windows(2).any(|w| {
                        segment_intersects_rect(w[0], w[1], tx_left, ry0, tx_right, ry1)
                    })
                })
            };
            let cx = b.x + b.width * 0.5;
            let half_tw = title_ft.width * 0.5 + 6.0;
            let (tx, anchor) = if !title_intersects_edges(cx - half_tw, cx + half_tw) {
                (cx, "middle")
            } else if !title_intersects_edges(b.x + 8.0, b.x + 12.0 + title_ft.width + 6.0) {
                (b.x + 12.0, "start")
            } else if !title_intersects_edges(
                b.x + b.width - 12.0 - title_ft.width - 6.0,
                b.x + b.width - 8.0,
            ) {
                (b.x + b.width - 12.0, "end")
            } else {
                (b.x + 12.0, "start")
            };
            draw_multiline_text(
                &mut content_canvas,
                tx,
                ty,
                title_ft,
                &font_family,
                sg_font_size,
                text_color,
                anchor,
                true,
            );
        }
    }

    // Resolve linkStyle default + per-index overrides.
    let mut default_link_style = StyleProps::default();
    let mut per_link_style: HashMap<usize, StyleProps> = HashMap::new();
    for (idx_opt, props) in &fc.link_styles {
        match idx_opt {
            None => default_link_style.merge_from(props),
            Some(i) => per_link_style.entry(*i).or_default().merge_from(props),
        }
    }

    // 2. Edges (pass 1: paths & markers; pass 2: labels).
    let mut deferred_edge_labels = Vec::new();
    for (edge_idx, e) in fc.edges.iter().enumerate() {
        if e.stroke == EdgeStroke::Invisible {
            continue;
        }
        let routed = &layout.edges[edge_idx];
        if routed.waypoints.len() < 2 {
            continue;
        }

        let mut pts = routed.waypoints.clone();
        if !routed.self_loop {
            let n_pts = pts.len();
            let first_target = pts[1];
            pts[0] = match e.from {
                Endpoint::Node(u) => {
                    let pos = layout.nodes[u];
                    let (w, h) = node_sizes[u];
                    let shape = fc.nodes[u].shape.unwrap_or(NodeShape::Rect);
                    clip_point_to_shape(shape, pos.cx, pos.cy, w, h, first_target)
                }
                Endpoint::Subgraph(sg) => {
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
                Endpoint::Node(v) => {
                    let pos = layout.nodes[v];
                    let (w, h) = node_sizes[v];
                    let shape = fc.nodes[v].shape.unwrap_or(NodeShape::Rect);
                    clip_point_to_shape(shape, pos.cx, pos.cy, w, h, last_target)
                }
                Endpoint::Subgraph(sg) => {
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

        let mut style = default_link_style.clone();
        if let Some(specific) = per_link_style.get(&edge_idx) {
            style.merge_from(specific);
        }

        let base_width = match e.stroke {
            EdgeStroke::Thick => 3.2,
            EdgeStroke::Solid | EdgeStroke::Dotted => 1.8,
            EdgeStroke::Invisible => 0.0,
        };
        let stroke_color = style.stroke.unwrap_or(palette.edge_stroke);
        let stroke_width = style.stroke_width.unwrap_or(base_width);
        let dash = style
            .stroke_dasharray
            .clone()
            .or_else(|| (e.stroke == EdgeStroke::Dotted).then(|| "5,4".to_string()));

        let (d, mid, start_dir, end_dir) = build_smooth_edge_path(&pts);
        content_canvas.raw_path(RawPath {
            d,
            fill: None,
            stroke: Some(stroke_color),
            stroke_width,
            stroke_dash_array: dash,
            stroke_linecap: Some("round"),
            stroke_linejoin: Some("round"),
            ..Default::default()
        });

        draw_edge_marker(
            &mut content_canvas,
            e.start_marker,
            pts[0],
            (-start_dir.0, -start_dir.1),
            stroke_color,
            palette.opaque_bg,
            stroke_width,
        );
        draw_edge_marker(
            &mut content_canvas,
            e.end_marker,
            *pts.last().unwrap(),
            end_dir,
            stroke_color,
            palette.opaque_bg,
            stroke_width,
        );

        if let Some(ref lbl) = e.label
            && !lbl.is_empty()
        {
            let ft = format_and_measure(lbl, &font_family, edge_font_size);
            let txt_color = style.color.unwrap_or(palette.text_color);
            deferred_edge_labels.push((mid, ft, txt_color));
        }
    }
    for (mid, ft, txt_color) in deferred_edge_labels {
        draw_edge_label(
            &mut content_canvas,
            mid,
            &ft,
            &palette,
            txt_color,
            &font_family,
            edge_font_size,
        );
    }

    // 3. Nodes.
    for (u, n) in fc.nodes.iter().enumerate() {
        if n.id.starts_with("__sg_anchor_") {
            continue;
        }
        let mut style = StyleProps::default();
        for cls in &n.classes {
            if let Some(cls_style) = fc.class_defs.get(cls) {
                style.merge_from(cls_style);
            }
        }
        style.merge_from(&n.inline_style);

        let pos = layout.nodes[u];
        let (w, h) = node_sizes[u];
        let shape = n.shape.unwrap_or(NodeShape::Rect);
        let fill = style.fill.unwrap_or(palette.node_fill);
        let stroke = style.stroke.unwrap_or(palette.node_stroke);
        let stroke_w = style.stroke_width.unwrap_or(1.6);
        let txt_color = style.color.unwrap_or(palette.text_color);

        draw_node(
            &mut content_canvas,
            shape,
            pos.cx,
            pos.cy,
            w,
            h,
            fill,
            stroke,
            stroke_w,
            style.stroke_dasharray.clone(),
            txt_color,
            &node_labels[u],
            &font_family,
            font_size,
        );
    }

    Ok((canvas.svg()?, out_w, out_h))
}
