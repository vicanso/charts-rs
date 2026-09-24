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

use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fmt;
use std::fmt::Write as _;
use std::vec;

use super::color::*;
use super::common::*;
use super::font;
use super::measure_text_width_family;
use super::path::*;
use super::util::*;

static TAG_LINE: &str = "line";
static TAG_RECT: &str = "rect";
static TAG_POLYLINE: &str = "polyline";
static TAG_CIRCLE: &str = "circle";
static TAG_POLYGON: &str = "polygon";
static TAG_TEXT: &str = "text";
static TAG_PATH: &str = "path";
static TAG_GROUP: &str = "g";

static ATTR_HEIGHT: &str = "height";
static ATTR_WIDTH: &str = "width";
static ATTR_FONT_FAMILY: &str = "font-family";
static ATTR_FONT_SIZE: &str = "font-size";
static ATTR_FONT_WEIGHT: &str = "font-weight";
static ATTR_TRANSFORM: &str = "transform";
static ATTR_DOMINANT_BASELINE: &str = "dominant-baseline";
static ATTR_TEXT_ANCHOR: &str = "text-anchor";
static ATTR_ALIGNMENT_BASELINE: &str = "alignment-baseline";
static ATTR_STROKE_OPACITY: &str = "stroke-opacity";
static ATTR_FILL_OPACITY: &str = "fill-opacity";
static ATTR_STROKE_WIDTH: &str = "stroke-width";
static ATTR_STROKE: &str = "stroke";
static ATTR_STROKE_DASH_ARRAY: &str = "stroke-dasharray";
static ATTR_X: &str = "x";
static ATTR_Y: &str = "y";
static ATTR_FILL: &str = "fill";
static ATTR_X1: &str = "x1";
static ATTR_Y1: &str = "y1";
static ATTR_X2: &str = "x2";
static ATTR_Y2: &str = "y2";
static ATTR_RX: &str = "rx";
static ATTR_RY: &str = "ry";
static ATTR_POINTS: &str = "points";
static ATTR_CX: &str = "cx";
static ATTR_CY: &str = "cy";
static ATTR_DX: &str = "dx";
static ATTR_DY: &str = "dy";
static ATTR_R: &str = "r";
static ATTR_D: &str = "d";
static ATTR_CLASS: &str = "class";
static ATTR_STYLE: &str = "style";
static ATTR_PATH_LENGTH: &str = "pathLength";

/// Converts opacity to string value.
fn convert_opacity(color: &Color) -> String {
    if color.is_nontransparent() {
        "".to_string()
    } else {
        format_opacity(color.opacity())
    }
}

fn fill_grad_id(start: &Color, end: &Color, angle: f32) -> String {
    format!(
        "grad_{:02X}{:02X}{:02X}{:02X}_{:02X}{:02X}{:02X}{:02X}_{}",
        start.r,
        start.g,
        start.b,
        start.a,
        end.r,
        end.g,
        end.b,
        end.a,
        angle.round() as i32
    )
}

/// Renders the `<defs>` block for a gradient fill. When a render-scoped
/// `grad_seen` set is provided, a gradient id that was already emitted in this
/// document is skipped — identical gradients share one definition instead of
/// repeating an (invalid) duplicate id per shape.
fn fill_svg_defs(fill: &Fill, grad_seen: Option<&mut HashSet<String>>) -> String {
    match fill {
        Fill::Solid(_) => String::new(),
        Fill::LinearGradient {
            start_color,
            end_color,
            angle,
        } => {
            if let Some(seen) = grad_seen
                && !seen.insert(fill_grad_id(start_color, end_color, *angle))
            {
                return String::new();
            }
            let angle_rad = angle * std::f32::consts::PI / 180.0;
            let x1 = 0.5 - 0.5 * angle_rad.sin();
            let y1 = 0.5 - 0.5 * angle_rad.cos();
            let x2 = 0.5 + 0.5 * angle_rad.sin();
            let y2 = 0.5 + 0.5 * angle_rad.cos();
            let id = fill_grad_id(start_color, end_color, *angle);
            let start_opacity = if start_color.is_nontransparent() {
                String::new()
            } else {
                format!(" stop-opacity=\"{:.2}\"", start_color.opacity())
            };
            let end_opacity = if end_color.is_nontransparent() {
                String::new()
            } else {
                format!(" stop-opacity=\"{:.2}\"", end_color.opacity())
            };
            format!(
                "<defs><linearGradient id=\"{id}\" x1=\"{x1:.2}\" y1=\"{y1:.2}\" x2=\"{x2:.2}\" y2=\"{y2:.2}\" gradientUnits=\"objectBoundingBox\"><stop offset=\"0%\" stop-color=\"{}\"{start_opacity}/><stop offset=\"100%\" stop-color=\"{}\"{end_opacity}/></linearGradient></defs>",
                start_color.hex(),
                end_color.hex(),
            )
        }
    }
}

fn fill_svg_attr(fill: &Fill) -> String {
    match fill {
        Fill::Solid(c) => {
            if c.is_transparent() {
                "none".to_string()
            } else {
                c.hex()
            }
        }
        Fill::LinearGradient {
            start_color,
            end_color,
            angle,
        } => {
            format!("url(#{})", fill_grad_id(start_color, end_color, *angle))
        }
    }
}

fn fill_svg_opacity(fill: &Fill) -> String {
    match fill {
        Fill::Solid(c) => convert_opacity(c),
        Fill::LinearGradient { .. } => String::new(),
    }
}

/// Builds the `<title>` child element (escaped) for a shape's tooltip, or
/// `None` when no title is set (keeping the shape a self-closing tag).
fn title_data(title: &Option<String>) -> Option<String> {
    title
        .as_ref()
        .map(|t| format!("<title>{}</title>", encode_text(t)))
}

#[derive(Clone, PartialEq, Debug, Default)]
struct SVGTag<'a> {
    tag: &'a str,
    attrs: Vec<(&'a str, String)>,
    /// `data-*` attributes, written after the fixed ones.
    dataset: &'a [(String, String)],
    data: Option<String>,
}

/// Shared CSS for the opt-in hover tooltip: a hidden `.ct-tip` label revealed
/// when the adjacent `.ct-trigger` data shape is hovered. Charts inject this
/// (via `svg_with_style`) when `tooltip_show` is set. Works in any browser,
/// unlike the bare `<title>` element.
pub(crate) const TOOLTIP_STYLE: &str =
    ".ct-tip{opacity:0;pointer-events:none} .ct-trigger:hover+.ct-tip{opacity:1}";

pub fn generate_svg(width: f32, height: f32, x: f32, y: f32, data: String) -> String {
    let mut out = String::with_capacity(data.len() + 160);
    write_svg_open(&mut out, width, height, x, y);
    out.push_str(&data);
    write_svg_close(&mut out);
    out
}

/// Writes the opening `<svg …>` tag (and the newline after it) so a chart
/// can stream its components right behind it.
pub(crate) fn write_svg_open(out: &mut String, width: f32, height: f32, x: f32, y: f32) {
    let _ = write!(
        out,
        "<svg width=\"{}\" height=\"{}\" viewBox=\"0 0 {} {}\" xmlns=\"http://www.w3.org/2000/svg\"",
        format_float(width),
        format_float(height),
        format_float(width),
        format_float(height)
    );
    if x != 0.0 {
        let _ = write!(out, " x=\"{}\"", format_float(x));
    }
    if y != 0.0 {
        let _ = write!(out, " y=\"{}\"", format_float(y));
    }
    out.push_str(">\n");
}

/// Writes the closing `</svg>` tag.
pub(crate) fn write_svg_close(out: &mut String) {
    out.push_str("\n</svg>");
}

/// Adds accessibility metadata to an SVG produced by a chart's `svg()`: sets
/// `role="img"` on the root `<svg>` and inserts a `<title>` (and, when `desc`
/// is non-empty, a `<desc>`) as its first children, giving assistive
/// technology an accessible name. This is opt-in — chart output is unchanged
/// unless you call it — and `role="img"` is only added together with a
/// non-empty `title`, since a role without a name would hide the content.
///
/// ```rust
/// use charts_rs::{BarChart, svg_with_accessibility};
/// let chart = BarChart::new(vec![("v", vec![1.0]).into()], vec!["a".to_string()]);
/// let svg = svg_with_accessibility(&chart.svg().unwrap(), "Weekly traffic", "");
/// assert!(svg.contains(r#"role="img""#));
/// assert!(svg.contains("<title>Weekly traffic</title>"));
/// ```
pub fn svg_with_accessibility(svg: &str, title: &str, desc: &str) -> String {
    if title.is_empty() {
        return svg.to_string();
    }
    let Some(tag_end) = svg.find('>') else {
        return svg.to_string();
    };
    let (open, rest) = svg.split_at(tag_end + 1);
    // The first `>` closes the root `<svg …>` tag (attribute values are quoted
    // and contain no `>`), so this turns it into `<svg … role="img">`.
    let open = open.replacen('>', " role=\"img\">", 1);
    let mut children = format!("\n<title>{}</title>", encode_text(title));
    if !desc.is_empty() {
        children.push_str(&format!("\n<desc>{}</desc>", encode_text(desc)));
    }
    format!("{open}{children}{rest}")
}

/// Writes an SVG/XML attribute value into `out`, escaping the characters that
/// could otherwise break out of the double-quoted attribute (`&`, `<`, `>`,
/// `"`). Without this, free-form string fields sourced from JSON (e.g.
/// `stroke_dash_array`, the various `*_font_weight` values) could inject
/// arbitrary markup — a stored/reflected XSS vector for any consumer that
/// serves `svg()` output to a browser. Only does per-character work when an
/// unsafe byte is present, so the common case stays a single `push_str`.
/// Escapes a string for use as SVG/XML text content (`&`, `<`, `>`); the
/// quote characters are fine inside text nodes, matching the escaping set of
/// the `html_escape::encode_text` this replaced (byte-identical output).
fn encode_text(raw: &str) -> String {
    let mut out = String::with_capacity(raw.len());
    for c in raw.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            _ => out.push(c),
        }
    }
    out
}

fn push_escaped_attr<W: fmt::Write>(out: &mut W, raw: &str) -> fmt::Result {
    if raw.bytes().any(|b| matches!(b, b'&' | b'<' | b'>' | b'"')) {
        for c in raw.chars() {
            match c {
                '&' => out.write_str("&amp;")?,
                '<' => out.write_str("&lt;")?,
                '>' => out.write_str("&gt;")?,
                '"' => out.write_str("&quot;")?,
                _ => out.write_char(c)?,
            }
        }
        Ok(())
    } else {
        out.write_str(raw)
    }
}

// A `data-*` key reaches the markup verbatim (only values are escaped), so a
// key that would not form a valid attribute name is dropped instead.
fn is_data_key(key: &str) -> bool {
    !key.is_empty()
        && key
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'_')
}

/// Streams one element straight into the output buffer: attributes are
/// written as they are set (numbers without an intermediate string), so the
/// hot shapes of a chart cost no per-attribute allocation.
struct TagWriter<'a> {
    out: &'a mut String,
    tag: &'static str,
}

impl<'a> TagWriter<'a> {
    fn open(out: &'a mut String, tag: &'static str) -> Self {
        out.push('<');
        out.push_str(tag);
        TagWriter { out, tag }
    }
    fn float(&mut self, key: &str, value: f32) -> &mut Self {
        self.out.push(' ');
        self.out.push_str(key);
        self.out.push_str("=\"");
        write_float(self.out, value);
        self.out.push('"');
        self
    }
    fn opt_float(&mut self, key: &str, value: Option<f32>) -> &mut Self {
        if let Some(value) = value {
            self.float(key, value);
        }
        self
    }
    /// A value that is known to be safe markup (hex colors, keywords).
    fn raw(&mut self, key: &str, value: &str) -> &mut Self {
        if !value.is_empty() {
            self.out.push(' ');
            self.out.push_str(key);
            self.out.push_str("=\"");
            self.out.push_str(value);
            self.out.push('"');
        }
        self
    }
    /// A free-form value, escaped for the attribute.
    fn text(&mut self, key: &str, value: &str) -> &mut Self {
        if !value.is_empty() {
            self.out.push(' ');
            self.out.push_str(key);
            self.out.push_str("=\"");
            let _ = push_escaped_attr(self.out, value);
            self.out.push('"');
        }
        self
    }
    fn opt_text(&mut self, key: &str, value: Option<&String>) -> &mut Self {
        if let Some(value) = value {
            self.text(key, value);
        }
        self
    }
    fn color(&mut self, key: &str, opacity_key: &str, color: Option<Color>) -> &mut Self {
        if let Some(color) = color {
            self.raw(key, &color.hex());
            self.raw(opacity_key, &convert_opacity(&color));
        }
        self
    }
    fn dataset(&mut self, dataset: &[(String, String)]) -> &mut Self {
        for (k, v) in dataset.iter() {
            if v.is_empty() || !is_data_key(k) {
                continue;
            }
            self.out.push_str(" data-");
            self.out.push_str(k);
            self.out.push_str("=\"");
            let _ = push_escaped_attr(self.out, v);
            self.out.push('"');
        }
        self
    }
    /// Closes a childless element.
    fn close(self) {
        self.out.push_str("/>");
    }
    /// Closes the element around `data` (already escaped markup / text).
    fn close_with(self, data: &str) {
        self.out.push_str(">\n");
        self.out.push_str(data);
        self.out.push_str("\n</");
        self.out.push_str(self.tag);
        self.out.push('>');
    }
    /// Closes the element with an optional `<title>` child.
    fn close_with_title(self, title: &Option<String>) {
        match title_data(title) {
            Some(data) => self.close_with(&data),
            None => self.close(),
        }
    }
}

impl fmt::Display for SVGTag<'_> {
    // Streams straight into the output (a chart-level shared buffer via
    // `write!`), instead of assembling an intermediate String per tag.
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.tag == TAG_GROUP
            && let Some(ref data) = self.data
            && data.is_empty()
        {
            return Ok(());
        }
        f.write_char('<')?;
        f.write_str(self.tag)?;
        for (k, v) in self.attrs.iter() {
            if k.is_empty() || v.is_empty() {
                continue;
            }
            f.write_char(' ')?;
            f.write_str(k)?;
            f.write_str("=\"")?;
            push_escaped_attr(f, v)?;
            f.write_char('"')?;
        }
        for (k, v) in self.dataset.iter() {
            if v.is_empty() || !is_data_key(k) {
                continue;
            }
            f.write_str(" data-")?;
            f.write_str(k)?;
            f.write_str("=\"")?;
            push_escaped_attr(f, v)?;
            f.write_char('"')?;
        }
        if let Some(ref data) = self.data {
            f.write_str(">\n")?;
            f.write_str(data)?;
            f.write_str("\n</")?;
            f.write_str(self.tag)?;
            f.write_char('>')
        } else {
            f.write_str("/>")
        }
    }
}

// Crate-level result type (see `error.rs`).
use super::error::Result;

pub enum Component {
    Arrow(Arrow),
    Bubble(Bubble),
    Line(Line),
    Rect(Rect),
    Polyline(Polyline),
    Circle(Circle),
    Polygon(Polygon),
    Text(Text),
    SmoothLine(SmoothLine),
    StraightLine(StraightLine),
    SmoothLineFill(SmoothLineFill),
    StraightLineFill(StraightLineFill),
    Grid(Grid),
    Axis(Axis),
    Legend(Legend),
    Pie(Pie),
    /// A filled band between two smooth curves.
    SmoothBand(SmoothBand),
}
#[derive(Clone, PartialEq, Debug)]

/// A straight line segment between two points.
pub struct Line {
    /// Stroke color.
    pub color: Option<Color>,
    /// Stroke width.
    pub stroke_width: f32,
    /// X coordinate of the start point.
    pub left: f32,
    /// Y coordinate of the start point.
    pub top: f32,
    /// X coordinate of the end point.
    pub right: f32,
    /// Y coordinate of the end point.
    pub bottom: f32,
    // dash array
    /// SVG stroke dash array.
    pub stroke_dash_array: Option<String>,
}

impl Default for Line {
    fn default() -> Self {
        Line {
            color: None,
            stroke_width: 1.0,
            left: 0.0,
            top: 0.0,
            right: 0.0,
            bottom: 0.0,
            stroke_dash_array: None,
        }
    }
}

impl Line {
    /// Renders the component to an SVG fragment.
    pub fn svg(&self) -> String {
        let mut out = String::new();
        self.write_svg(&mut out);
        out
    }
    pub(crate) fn write_svg(&self, out: &mut String) {
        if self.stroke_width <= 0.0 {
            return;
        }
        let mut w = TagWriter::open(out, TAG_LINE);
        w.float(ATTR_STROKE_WIDTH, self.stroke_width);
        self.write_coordinates(&mut w);
        w.color(ATTR_STROKE, ATTR_STROKE_OPACITY, self.color)
            .opt_text(ATTR_STROKE_DASH_ARRAY, self.stroke_dash_array.as_ref());
        w.close();
    }
    /// Writes only the end points, for lines inside a `<g>` that carries the
    /// stroke attributes.
    fn write_bare(&self, out: &mut String) {
        let mut w = TagWriter::open(out, TAG_LINE);
        self.write_coordinates(&mut w);
        w.close();
    }
    fn write_coordinates(&self, w: &mut TagWriter<'_>) {
        w.float(ATTR_X1, self.left)
            .float(ATTR_Y1, self.top)
            .float(ATTR_X2, self.right)
            .float(ATTR_Y2, self.bottom);
    }
}

#[derive(Clone, PartialEq, Debug, Default)]
/// A rectangle, optionally rounded and gradient-filled.
pub struct Rect {
    /// Stroke color.
    pub color: Option<Color>,
    /// Fill (solid or gradient).
    pub fill: Option<Fill>,
    /// Left (x) coordinate.
    pub left: f32,
    /// Top (y) coordinate.
    pub top: f32,
    /// Width.
    pub width: f32,
    /// Height.
    pub height: f32,
    /// Corner radius on the x axis.
    pub rx: Option<f32>,
    /// Corner radius on the y axis.
    pub ry: Option<f32>,
    /// CSS class attribute of the SVG element.
    pub class: Option<String>,
    /// Inline style attribute of the SVG element.
    pub style: Option<String>,
    /// Optional native `<title>` child (hover tooltip / accessible name).
    pub title: Option<String>,
    /// `data-*` attributes for the SVG element: `("date", "2026-01-05")` is
    /// written as `data-date="2026-01-05"`. Keys are limited to ASCII letters,
    /// digits, `-` and `_`; anything else is skipped, as are empty values.
    /// Empty by default, so charts that set none render exactly as before.
    pub dataset: Vec<(String, String)>,
}
impl Rect {
    /// Renders the component to an SVG fragment.
    pub fn svg(&self) -> String {
        let mut out = String::new();
        self.write_svg(&mut out, None);
        out
    }
    pub(crate) fn write_svg(&self, out: &mut String, grad_seen: Option<&mut HashSet<String>>) {
        if let Some(fill) = &self.fill {
            out.push_str(&fill_svg_defs(fill, grad_seen));
        }
        let mut w = TagWriter::open(out, TAG_RECT);
        self.write_geometry(&mut w);
        w.color(ATTR_STROKE, ATTR_STROKE_OPACITY, self.color);
        if let Some(fill) = &self.fill {
            let fill_attr = fill_svg_attr(fill);
            w.raw(ATTR_FILL, &fill_attr);
            if fill_attr != "none" {
                w.raw(ATTR_FILL_OPACITY, &fill_svg_opacity(fill));
            }
        }
        w.opt_text(ATTR_CLASS, self.class.as_ref())
            .opt_text(ATTR_STYLE, self.style.as_ref())
            .dataset(&self.dataset);
        w.close_with_title(&self.title);
    }
    /// Writes only the geometry, for rects inside a `<g>` that carries the
    /// paint attributes.
    fn write_bare(&self, out: &mut String) {
        let mut w = TagWriter::open(out, TAG_RECT);
        self.write_geometry(&mut w);
        w.close();
    }
    fn write_geometry(&self, w: &mut TagWriter<'_>) {
        w.float(ATTR_X, self.left)
            .float(ATTR_Y, self.top)
            .float(ATTR_WIDTH, self.width)
            .float(ATTR_HEIGHT, self.height)
            .opt_float(ATTR_RX, self.rx)
            .opt_float(ATTR_RY, self.ry);
    }
}

#[derive(Clone, PartialEq, Debug)]
/// A polyline through a list of points.
pub struct Polyline {
    /// Stroke color.
    pub color: Option<Color>,
    /// Stroke width.
    pub stroke_width: f32,
    /// The points of the line.
    pub points: Vec<Point>,
}

impl Default for Polyline {
    fn default() -> Self {
        Polyline {
            color: None,
            stroke_width: 1.0,
            points: vec![],
        }
    }
}

impl Polyline {
    /// Renders the component to an SVG fragment.
    pub fn svg(&self) -> String {
        let mut out = String::new();
        self.write_svg(&mut out);
        out
    }
    pub(crate) fn write_svg(&self, out: &mut String) {
        if self.stroke_width <= 0.0 {
            return;
        }
        let mut points = String::with_capacity(self.points.len() * 10);
        for (i, p) in self.points.iter().enumerate() {
            if i > 0 {
                points.push(' ');
            }
            points.push_str(&format_float(p.x));
            points.push(',');
            points.push_str(&format_float(p.y));
        }
        let mut w = TagWriter::open(out, TAG_POLYLINE);
        w.raw(ATTR_FILL, "none")
            .float(ATTR_STROKE_WIDTH, self.stroke_width)
            .raw(ATTR_POINTS, &points)
            .color(ATTR_STROKE, ATTR_STROKE_OPACITY, self.color);
        w.close();
    }
}

#[derive(Clone, PartialEq, Debug)]
/// A circle.
pub struct Circle {
    /// Stroke color.
    pub stroke_color: Option<Color>,
    /// Fill color.
    pub fill: Option<Color>,
    /// Stroke width.
    pub stroke_width: f32,
    /// Center x coordinate.
    pub cx: f32,
    /// Center y coordinate.
    pub cy: f32,
    /// Radius.
    pub r: f32,
    /// Optional native `<title>` child (hover tooltip / accessible name).
    pub title: Option<String>,
    /// Optional CSS class (used for the hover-tooltip trigger).
    pub class: Option<String>,
    /// `data-*` attributes, see [`Rect::dataset`].
    pub dataset: Vec<(String, String)>,
}

impl Default for Circle {
    fn default() -> Self {
        Circle {
            stroke_color: None,
            fill: None,
            stroke_width: 1.0,
            cx: 0.0,
            cy: 0.0,
            r: 3.0,
            title: None,
            class: None,
            dataset: vec![],
        }
    }
}

impl Circle {
    /// Renders the component to an SVG fragment.
    pub fn svg(&self) -> String {
        let mut out = String::new();
        self.write_svg(&mut out);
        out
    }
    pub(crate) fn write_svg(&self, out: &mut String) {
        let mut w = TagWriter::open(out, TAG_CIRCLE);
        self.write_geometry(&mut w);
        // A stroke width without a stroke color would be dead weight.
        if self.stroke_color.is_some() {
            w.float(ATTR_STROKE_WIDTH, self.stroke_width);
        }
        w.color(ATTR_STROKE, ATTR_STROKE_OPACITY, self.stroke_color);
        match self.fill {
            Some(color) => {
                w.raw(ATTR_FILL_OPACITY, &convert_opacity(&color));
                w.raw(ATTR_FILL, &color.hex());
            }
            None => {
                w.raw(ATTR_FILL, "none");
            }
        }
        w.opt_text(ATTR_CLASS, self.class.as_ref())
            .dataset(&self.dataset);
        w.close_with_title(&self.title);
    }
    /// Writes only the geometry, for circles inside a `<g>` that carries the
    /// paint attributes.
    fn write_bare(&self, out: &mut String) {
        let mut w = TagWriter::open(out, TAG_CIRCLE);
        self.write_geometry(&mut w);
        w.close();
    }
    fn write_geometry(&self, w: &mut TagWriter<'_>) {
        w.float(ATTR_CX, self.cx)
            .float(ATTR_CY, self.cy)
            .float(ATTR_R, self.r);
    }
}

#[derive(Clone, PartialEq, Debug)]
pub struct Arrow {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub stroke_color: Color,
}
impl Arrow {
    pub fn default() -> Self {
        Arrow {
            x: 0.0,
            y: 0.0,
            width: 10.0,
            stroke_color: Color::default(),
        }
    }
    pub fn svg(&self) -> String {
        let x_offset = self.width / 2.0;
        let y_offset = self.width / 2.0;
        let points = vec![
            Point {
                x: self.x,
                y: self.y,
            },
            Point {
                x: self.x - x_offset,
                y: self.y - y_offset,
            },
            Point {
                x: self.x + self.width,
                y: self.y,
            },
            Point {
                x: self.x - x_offset,
                y: self.y + y_offset,
            },
        ];
        StraightLine {
            color: Some(self.stroke_color),
            fill: Some(self.stroke_color),
            points,
            close: true,
            symbol: None,
            ..Default::default()
        }
        .svg()
    }
}

#[derive(Clone, PartialEq, Debug, Default)]
/// A closed polygon through a list of points.
pub struct Polygon {
    /// Stroke color.
    pub color: Option<Color>,
    /// Fill color.
    pub fill: Option<Color>,
    /// Optional gradient fill; when set it overrides `fill`.
    pub gradient: Option<Fill>,
    /// The vertices of the polygon.
    pub points: Vec<Point>,
    /// CSS class attribute of the SVG element.
    pub class: Option<String>,
    /// Inline style attribute of the SVG element.
    pub style: Option<String>,
    /// Optional native `<title>` child (hover tooltip / accessible name).
    pub title: Option<String>,
    /// `data-*` attributes, see [`Rect::dataset`].
    pub dataset: Vec<(String, String)>,
}

impl Polygon {
    /// Renders the component to an SVG fragment.
    pub fn svg(&self) -> String {
        let mut out = String::new();
        self.write_svg(&mut out, None);
        out
    }
    pub(crate) fn write_svg(&self, out: &mut String, grad_seen: Option<&mut HashSet<String>>) {
        if self.points.is_empty() {
            return;
        }
        let mut points = String::with_capacity(self.points.len() * 10);
        for (i, p) in self.points.iter().enumerate() {
            if i > 0 {
                points.push(' ');
            }
            points.push_str(&format_float(p.x));
            points.push(',');
            points.push_str(&format_float(p.y));
        }
        if let Some(ref fill) = self.gradient {
            out.push_str(&fill_svg_defs(fill, grad_seen));
        }
        let mut w = TagWriter::open(out, TAG_POLYGON);
        w.raw(ATTR_POINTS, &points)
            .color(ATTR_STROKE, ATTR_STROKE_OPACITY, self.color);
        if let Some(ref fill) = self.gradient {
            w.raw(ATTR_FILL, &fill_svg_attr(fill));
            w.raw(ATTR_FILL_OPACITY, &fill_svg_opacity(fill));
        } else {
            w.color(ATTR_FILL, ATTR_FILL_OPACITY, self.fill);
        }
        w.opt_text(ATTR_CLASS, self.class.as_ref())
            .opt_text(ATTR_STYLE, self.style.as_ref())
            .dataset(&self.dataset);
        w.close_with_title(&self.title);
    }
    /// Writes only the points, for polygons inside a `<g>` that carries the
    /// paint attributes.
    fn write_bare(&self, out: &mut String) {
        let mut points = String::with_capacity(self.points.len() * 10);
        for (i, p) in self.points.iter().enumerate() {
            if i > 0 {
                points.push(' ');
            }
            write_float(&mut points, p.x);
            points.push(',');
            write_float(&mut points, p.y);
        }
        let mut w = TagWriter::open(out, TAG_POLYGON);
        w.raw(ATTR_POINTS, &points);
        w.close();
    }
}

#[derive(Clone, PartialEq, Debug, Default)]
pub struct Bubble {
    pub r: f32,
    pub x: f32,
    pub y: f32,
    pub fill: Color,
}

impl Bubble {
    pub fn svg(&self) -> String {
        let x = format_float(self.x);
        let y = format_float(self.y);
        let r = format_float(self.r);

        let first = get_pie_point(self.x, self.y, self.r, -140.0);
        let last = get_pie_point(self.x, self.y, self.r, 140.0);

        let mut path_list = vec![
            format!("M {},{}", format_float(first.x), format_float(first.y)),
            format!("A {r},{r} 0,0,1 {},{y}", format_float(self.x - self.r)),
            format!("A {r},{r} 0,0,1 {},{y}", format_float(self.x + self.r)),
            format!(
                "A {r},{r} 0,0,1 {},{}",
                format_float(last.x),
                format_float(last.y)
            ),
            format!("L {x},{}", format_float(self.y + self.r * 1.5)),
        ];

        path_list.push("Z".to_string());

        let attrs = vec![
            (ATTR_D, path_list.join(" ")),
            (ATTR_FILL, self.fill.hex()),
            (ATTR_FILL_OPACITY, convert_opacity(&self.fill)),
        ];
        SVGTag {
            tag: TAG_PATH,
            attrs,
            ..Default::default()
        }
        .to_string()
    }
}

#[derive(Clone, PartialEq, Debug, Default)]
/// A text element.
pub struct Text {
    /// The text content.
    pub text: String,
    /// Font family.
    pub font_family: Option<String>,
    /// Font size.
    pub font_size: Option<f32>,
    /// Font color.
    pub font_color: Option<Color>,
    /// Line height.
    pub line_height: Option<f32>,
    /// X coordinate.
    pub x: Option<f32>,
    /// Y coordinate.
    pub y: Option<f32>,
    /// X offset.
    pub dx: Option<f32>,
    /// Y offset.
    pub dy: Option<f32>,
    /// Font weight, e.g. `"bold"`.
    pub font_weight: Option<String>,
    /// SVG transform attribute.
    pub transform: Option<String>,
    /// SVG dominant-baseline attribute.
    pub dominant_baseline: Option<String>,
    /// SVG text-anchor attribute.
    pub text_anchor: Option<String>,
    /// SVG alignment-baseline attribute.
    pub alignment_baseline: Option<String>,
    /// CSS class attribute of the SVG element.
    pub class: Option<String>,
}

impl Text {
    /// Renders the component to an SVG fragment.
    pub fn svg(&self) -> String {
        let mut out = String::new();
        self.write_svg(&mut out);
        out
    }
    pub(crate) fn write_svg(&self, out: &mut String) {
        if self.text.is_empty() {
            return;
        }
        let mut w = TagWriter::open(out, TAG_TEXT);
        w.opt_float(ATTR_FONT_SIZE, self.font_size)
            .opt_float(ATTR_X, self.x)
            .opt_float(ATTR_Y, self.y)
            .opt_float(ATTR_DX, self.dx)
            .opt_float(ATTR_DY, self.dy)
            .opt_text(ATTR_FONT_WEIGHT, self.font_weight.as_ref())
            .opt_text(ATTR_TRANSFORM, self.transform.as_ref())
            .opt_text(ATTR_DOMINANT_BASELINE, self.dominant_baseline.as_ref())
            .opt_text(ATTR_TEXT_ANCHOR, self.text_anchor.as_ref())
            .opt_text(ATTR_ALIGNMENT_BASELINE, self.alignment_baseline.as_ref())
            .opt_text(ATTR_FONT_FAMILY, self.font_family.as_ref())
            .color(ATTR_FILL, ATTR_FILL_OPACITY, self.font_color)
            .opt_text(ATTR_CLASS, self.class.as_ref());
        w.close_with(&encode_text(&self.text));
    }
}

/// Opens the `<g>` that carries the paint shared by every symbol of a line,
/// so the per-point shapes only carry their geometry.
fn open_symbol_group(
    out: &mut String,
    stroke_width: Option<f32>,
    stroke: Option<Color>,
    fill: Option<Color>,
) {
    let mut w = TagWriter::open(out, TAG_GROUP);
    if let Some(stroke_width) = stroke_width {
        w.float(ATTR_STROKE_WIDTH, stroke_width);
    }
    w.color(ATTR_STROKE, ATTR_STROKE_OPACITY, stroke);
    match fill {
        Some(color) => {
            w.raw(ATTR_FILL, &color.hex());
            w.raw(ATTR_FILL_OPACITY, &convert_opacity(&color));
        }
        None => {
            w.raw(ATTR_FILL, "none");
        }
    }
    w.out.push_str(">\n");
}

fn generate_circle_symbol(points: &[Point], c: Circle) -> String {
    if points.is_empty() {
        return String::new();
    }
    let mut out = String::with_capacity(points.len() * 40);
    open_symbol_group(
        &mut out,
        c.stroke_color.map(|_| c.stroke_width),
        c.stroke_color,
        c.fill,
    );
    for (i, p) in points.iter().enumerate() {
        if i > 0 {
            out.push('\n');
        }
        Circle {
            cx: p.x,
            cy: p.y,
            r: c.r,
            ..Default::default()
        }
        .write_bare(&mut out);
    }
    out.push_str("\n</g>");
    out
}

fn generate_rect_symbol(
    points: &[Point],
    r: f32,
    fill: Option<Color>,
    stroke: Option<Color>,
) -> String {
    if points.is_empty() {
        return String::new();
    }
    let mut out = String::with_capacity(points.len() * 50);
    open_symbol_group(&mut out, None, stroke, fill);
    for (i, p) in points.iter().enumerate() {
        if i > 0 {
            out.push('\n');
        }
        Rect {
            left: p.x - r,
            top: p.y - r,
            width: r * 2.0,
            height: r * 2.0,
            ..Default::default()
        }
        .write_bare(&mut out);
    }
    out.push_str("\n</g>");
    out
}

fn generate_polygon_symbols(
    points: &[Point],
    fill: Option<Color>,
    stroke: Option<Color>,
    shape: impl Fn(&Point) -> Vec<Point>,
) -> String {
    if points.is_empty() {
        return String::new();
    }
    let mut out = String::with_capacity(points.len() * 50);
    open_symbol_group(&mut out, None, stroke, fill);
    for (i, p) in points.iter().enumerate() {
        if i > 0 {
            out.push('\n');
        }
        Polygon {
            points: shape(p),
            ..Default::default()
        }
        .write_bare(&mut out);
    }
    out.push_str("\n</g>");
    out
}

fn generate_triangle_symbol(
    points: &[Point],
    r: f32,
    fill: Option<Color>,
    stroke: Option<Color>,
) -> String {
    // Equilateral triangle pointing upward; r = circumradius.
    generate_polygon_symbols(points, fill, stroke, |p| {
        vec![
            (p.x, p.y - r).into(),
            (p.x + r * 0.866, p.y + r * 0.5).into(),
            (p.x - r * 0.866, p.y + r * 0.5).into(),
        ]
    })
}

fn generate_diamond_symbol(
    points: &[Point],
    r: f32,
    fill: Option<Color>,
    stroke: Option<Color>,
) -> String {
    generate_polygon_symbols(points, fill, stroke, |p| {
        vec![
            (p.x, p.y - r).into(),
            (p.x + r, p.y).into(),
            (p.x, p.y + r).into(),
            (p.x - r, p.y).into(),
        ]
    })
}

#[derive(Clone, PartialEq, Debug)]
/// A pie / donut slice.
pub struct Pie {
    /// Fill (solid or gradient).
    pub fill: Fill,
    /// Stroke color.
    pub stroke_color: Option<Color>,
    /// Center x coordinate.
    pub cx: f32,
    /// Center y coordinate.
    pub cy: f32,
    /// Outer radius.
    pub r: f32,
    /// Inner radius; a value above zero renders a donut slice.
    pub ir: f32,
    /// Start angle in degrees.
    pub start_angle: f32,
    /// Sweep angle in degrees.
    pub delta: f32,
    /// Corner radius of the outer edge.
    pub border_radius: f32,
    /// CSS class attribute of the SVG element.
    pub class: Option<String>,
    /// Inline style attribute of the SVG element.
    pub style: Option<String>,
    /// Optional native `<title>` child (hover tooltip / accessible name).
    pub title: Option<String>,
    /// `data-*` attributes, see [`Rect::dataset`].
    pub dataset: Vec<(String, String)>,
}

impl Default for Pie {
    fn default() -> Self {
        Pie {
            fill: Fill::Solid((0, 0, 0).into()),
            stroke_color: None,
            cx: 0.0,
            cy: 0.0,
            r: 250.0,
            ir: 60.0,
            start_angle: 0.0,
            delta: 0.0,
            border_radius: 8.0,
            class: None,
            style: None,
            title: None,
            dataset: vec![],
        }
    }
}

impl Pie {
    /// Renders the component to an SVG fragment.
    pub fn svg(&self) -> String {
        self.svg_with_grad_seen(None)
    }
    pub(crate) fn svg_with_grad_seen(&self, grad_seen: Option<&mut HashSet<String>>) -> String {
        let r = self.r;
        let r_str = format_float(r);

        let ir = self.ir;
        let ir_str = format_float(ir);

        let mut path_list = vec![];
        let mut border_radius = self.border_radius;
        if border_radius != 0.0 && self.r - self.ir < border_radius {
            border_radius = 2.0;
        }
        let border_radius_str = format_float(border_radius);
        let border_angle = 2.0_f32;
        let start_angle = self.start_angle;
        let end_angle = start_angle + self.delta;

        // 左下角第一个点
        if self.ir == 0.0 {
            path_list.push(format!(
                "M{},{}",
                format_float(self.cx),
                format_float(self.cy),
            ));
        } else {
            let point = get_pie_point(self.cx, self.cy, self.ir + border_radius, start_angle);
            path_list.push(format!(
                "M{},{}",
                format_float(point.x),
                format_float(point.y)
            ));
        }

        // 左侧直线
        let point = get_pie_point(self.cx, self.cy, self.r - border_radius, start_angle);
        path_list.push(format!(
            "L{},{}",
            format_float(point.x),
            format_float(point.y)
        ));

        // 左上圆角
        let point = get_pie_point(self.cx, self.cy, self.r, start_angle + border_angle);
        path_list.push(format!(
            "A{border_radius_str} {border_radius_str} 0 0 1 {},{}",
            format_float(point.x),
            format_float(point.y)
        ));

        // 大圆弧
        // 如果过大，要先划一半
        if self.delta > 180.0 {
            let point = get_pie_point(
                self.cx,
                self.cy,
                self.r,
                self.start_angle + 180.0 - border_angle,
            );
            path_list.push(format!(
                "A{r_str} {r_str} 0 0 1 {},{}",
                format_float(point.x),
                format_float(point.y)
            ));
        }

        let point = get_pie_point(self.cx, self.cy, self.r, end_angle - border_angle);
        path_list.push(format!(
            "A{r_str} {r_str} 0 0 1 {},{}",
            format_float(point.x),
            format_float(point.y)
        ));

        // 右上圆角
        let point = get_pie_point(self.cx, self.cy, self.r - border_radius, end_angle);
        path_list.push(format!(
            "A{border_radius_str} {border_radius_str} 0 0 1 {},{}",
            format_float(point.x),
            format_float(point.y)
        ));

        // 右侧直线
        let point = get_pie_point(self.cx, self.cy, self.ir + border_radius, end_angle);
        path_list.push(format!(
            "L{},{}",
            format_float(point.x),
            format_float(point.y)
        ));

        if self.ir > 0.0 {
            // 右下圆角
            let point = get_pie_point(self.cx, self.cy, self.ir, end_angle - border_angle);
            path_list.push(format!(
                "A{border_radius_str} {border_radius_str} 0 0 1 {},{}",
                format_float(point.x),
                format_float(point.y)
            ));

            // 小圆弧
            // 如果过大，要先划一半
            if self.delta > 180.0 {
                let point = get_pie_point(self.cx, self.cy, self.ir, end_angle - 180.0);
                path_list.push(format!(
                    "A{ir_str} {ir_str} 0 0 0 {},{}",
                    format_float(point.x),
                    format_float(point.y)
                ));
            }

            let point = get_pie_point(self.cx, self.cy, self.ir, start_angle + border_angle);
            path_list.push(format!(
                "A{ir_str} {ir_str} 0 0 0 {},{}",
                format_float(point.x),
                format_float(point.y)
            ));

            // 左下圆角
            let point = get_pie_point(self.cx, self.cy, self.ir + border_radius, start_angle);
            path_list.push(format!(
                "A{border_radius_str} {border_radius_str} 0 0 1 {},{}",
                format_float(point.x),
                format_float(point.y)
            ));
        }

        path_list.push("Z".to_string());

        let defs = fill_svg_defs(&self.fill, grad_seen);
        let mut attrs = vec![
            (ATTR_D, path_list.join(" ")),
            (ATTR_FILL, fill_svg_attr(&self.fill)),
            (ATTR_FILL_OPACITY, fill_svg_opacity(&self.fill)),
        ];
        if let Some(color) = self.stroke_color {
            attrs.push((ATTR_STROKE, color.hex()));
            attrs.push((ATTR_STROKE_OPACITY, convert_opacity(&color)));
        }
        if let Some(ref class) = self.class {
            attrs.push((ATTR_CLASS, class.clone()));
        }
        if let Some(ref style) = self.style {
            attrs.push((ATTR_STYLE, style.clone()));
        }
        let element = SVGTag {
            tag: TAG_PATH,
            attrs,
            dataset: &self.dataset,
            data: title_data(&self.title),
        }
        .to_string();
        if defs.is_empty() {
            element
        } else {
            format!("{defs}{element}")
        }
    }
}

struct BaseLine<'a> {
    pub color: Option<Color>,
    pub fill: Option<Color>,
    pub points: &'a [Point],
    pub stroke_width: f32,
    pub symbol: Option<Symbol>,
    pub is_smooth: bool,
    pub close: bool,
    pub stroke_dash_array: Option<String>,
    pub class: Option<String>,
    pub path_length: Option<f32>,
}

impl<'a> BaseLine<'a> {
    pub fn svg(&self) -> String {
        if self.points.is_empty() || self.stroke_width <= 0.0 {
            return "".to_string();
        }
        let path = if self.is_smooth {
            SmoothCurve {
                points: self.points.to_vec(),
                ..Default::default()
            }
            .to_string()
        } else {
            let mut arr = vec![];
            for (index, p) in self.points.iter().enumerate() {
                let mut action = "L";
                if index == 0 {
                    action = "M"
                }
                arr.push(format!(
                    "{} {} {}",
                    action,
                    format_float(p.x),
                    format_float(p.y)
                ));
            }
            if self.close {
                arr.push('Z'.to_string());
            }
            arr.join(" ")
        };

        let mut attrs = vec![
            (ATTR_D, path),
            (ATTR_STROKE_WIDTH, format_float(self.stroke_width)),
        ];
        if let Some(fill) = self.fill {
            attrs.push((ATTR_FILL, fill.hex()));
            attrs.push((ATTR_FILL_OPACITY, convert_opacity(&fill)));
        } else {
            attrs.push((ATTR_FILL, "none".to_string()));
        }

        if let Some(color) = self.color {
            attrs.push((ATTR_STROKE, color.hex()));
            attrs.push((ATTR_STROKE_OPACITY, convert_opacity(&color)));
        }
        if let Some(stroke_dash_array) = &self.stroke_dash_array {
            attrs.push((ATTR_STROKE_DASH_ARRAY, stroke_dash_array.to_string()));
        }
        if let Some(pl) = self.path_length {
            attrs.push((ATTR_PATH_LENGTH, format_float(pl)));
        }
        if let Some(ref class) = self.class {
            attrs.push((ATTR_CLASS, class.clone()));
        }
        let line_svg = SVGTag {
            tag: TAG_PATH,
            attrs,
            ..Default::default()
        }
        .to_string();
        let symbol_svg = if let Some(ref symbol) = self.symbol {
            match symbol {
                Symbol::Circle(r, fill) => generate_circle_symbol(
                    self.points,
                    Circle {
                        stroke_color: self.color,
                        fill: fill.to_owned(),
                        stroke_width: self.stroke_width,
                        r: r.to_owned(),
                        ..Default::default()
                    },
                ),
                Symbol::Rect(r, fill) => {
                    generate_rect_symbol(self.points, *r, fill.to_owned(), self.color)
                }
                Symbol::Triangle(r, fill) => {
                    generate_triangle_symbol(self.points, *r, fill.to_owned(), self.color)
                }
                Symbol::Diamond(r, fill) => {
                    generate_diamond_symbol(self.points, *r, fill.to_owned(), self.color)
                }
                Symbol::None => "".to_string(),
            }
        } else {
            "".to_string()
        };

        if symbol_svg.is_empty() {
            line_svg
        } else {
            SVGTag {
                tag: TAG_GROUP,
                data: Some([line_svg, symbol_svg].join("\n")),
                ..Default::default()
            }
            .to_string()
        }
    }
}

#[derive(Clone, PartialEq, Debug)]
/// A smooth curve through a list of points.
pub struct SmoothLine {
    /// Stroke color.
    pub color: Option<Color>,
    /// The points the curve passes through.
    pub points: Vec<Point>,
    /// Stroke width.
    pub stroke_width: f32,
    /// Marker drawn on each point.
    pub symbol: Option<Symbol>,
    /// SVG stroke dash array.
    pub stroke_dash_array: Option<String>,
    /// CSS class attribute of the SVG element.
    pub class: Option<String>,
    /// SVG pathLength attribute (used by animations).
    pub path_length: Option<f32>,
}

impl Default for SmoothLine {
    fn default() -> Self {
        SmoothLine {
            color: None,
            points: vec![],
            stroke_width: 1.0,
            symbol: Some(Symbol::Circle(2.0, None)),
            stroke_dash_array: None,
            class: None,
            path_length: None,
        }
    }
}

impl SmoothLine {
    /// Renders the component to an SVG fragment.
    pub fn svg(&self) -> String {
        BaseLine {
            color: self.color,
            fill: None,
            points: &self.points,
            stroke_width: self.stroke_width,
            symbol: self.symbol.clone(),
            is_smooth: true,
            close: false,
            stroke_dash_array: self.stroke_dash_array.clone(),
            class: self.class.clone(),
            path_length: self.path_length,
        }
        .svg()
    }
}

#[derive(Clone, PartialEq, Debug)]
/// The filled area under a smooth curve.
pub struct SmoothLineFill {
    /// Fill (solid or gradient).
    pub fill: Fill,
    /// The points the curve passes through.
    pub points: Vec<Point>,
    /// Y coordinate of the baseline the fill closes to.
    pub bottom: f32,
}

impl Default for SmoothLineFill {
    fn default() -> Self {
        SmoothLineFill {
            fill: Fill::Solid((255, 255, 255, 255).into()),
            points: vec![],
            bottom: 0.0,
        }
    }
}

impl SmoothLineFill {
    /// Renders the component to an SVG fragment.
    pub fn svg(&self) -> String {
        self.svg_with_grad_seen(None)
    }
    pub(crate) fn svg_with_grad_seen(&self, grad_seen: Option<&mut HashSet<String>>) -> String {
        if self.points.is_empty() || self.fill.is_transparent() {
            return "".to_string();
        }
        let mut path = SmoothCurve {
            points: self.points.clone(),
            ..Default::default()
        }
        .to_string();

        let last = self.points[self.points.len() - 1];
        let first = self.points[0];
        let fill_path = [
            format!("M {} {}", format_float(last.x), format_float(last.y)),
            format!("L {} {}", format_float(last.x), format_float(self.bottom)),
            format!("L {} {}", format_float(first.x), format_float(self.bottom)),
            format!("L {} {}", format_float(first.x), format_float(first.y)),
        ]
        .join(" ");
        path.push_str(&fill_path);

        let defs = fill_svg_defs(&self.fill, grad_seen);
        let attrs = vec![
            (ATTR_D, path),
            (ATTR_FILL, fill_svg_attr(&self.fill)),
            (ATTR_FILL_OPACITY, fill_svg_opacity(&self.fill)),
        ];

        let element = SVGTag {
            tag: TAG_PATH,
            attrs,
            ..Default::default()
        }
        .to_string();
        if defs.is_empty() {
            element
        } else {
            format!("{defs}{element}")
        }
    }
}

/// A filled band between two smooth curves, e.g. one stream of a theme
/// river: the top edge runs left to right, the bottom edge back.
#[derive(Clone, PartialEq, Debug, Default)]
pub struct SmoothBand {
    /// Points of the upper edge, left to right.
    pub top: Vec<Point>,
    /// Points of the lower edge, left to right (same x positions).
    pub bottom: Vec<Point>,
    /// Fill color.
    pub fill: Option<Color>,
    /// CSS class attribute of the SVG element.
    pub class: Option<String>,
    /// Optional native `<title>` child (hover tooltip / accessible name).
    pub title: Option<String>,
    /// `data-*` attributes, see [`Rect::dataset`].
    pub dataset: Vec<(String, String)>,
}

impl SmoothBand {
    /// Renders the component to an SVG fragment.
    pub fn svg(&self) -> String {
        if self.top.len() < 2 || self.bottom.len() < 2 {
            return String::new();
        }
        let mut d = SmoothCurve {
            points: self.top.clone(),
            ..Default::default()
        }
        .to_string();
        let mut bottom = self.bottom.clone();
        bottom.reverse();
        let bottom_d = SmoothCurve {
            points: bottom,
            ..Default::default()
        }
        .to_string();
        // The lower curve starts with a move; join it with a line instead.
        let (start, rest) = bottom_d.split_once(' ').unwrap_or((&bottom_d, ""));
        d.push_str(" L");
        d.push_str(&start.trim_start_matches('M').replace(',', " "));
        if !rest.is_empty() {
            d.push(' ');
            d.push_str(rest);
        }
        d.push_str(" Z");
        let mut attrs = vec![(ATTR_D, d)];
        if let Some(color) = self.fill {
            attrs.push((ATTR_FILL, color.hex()));
            attrs.push((ATTR_FILL_OPACITY, convert_opacity(&color)));
        }
        if let Some(ref class) = self.class {
            attrs.push((ATTR_CLASS, class.clone()));
        }
        SVGTag {
            tag: TAG_PATH,
            attrs,
            dataset: &self.dataset,
            data: title_data(&self.title),
        }
        .to_string()
    }
}

#[derive(Clone, PartialEq, Debug)]
/// A series line drawn with straight segments.
pub struct StraightLine {
    /// Stroke color.
    pub color: Option<Color>,
    /// Fill color.
    pub fill: Option<Color>,
    /// The points of the line.
    pub points: Vec<Point>,
    /// Stroke width.
    pub stroke_width: f32,
    /// Marker drawn on each point.
    pub symbol: Option<Symbol>,
    /// Closes the path back to the first point.
    pub close: bool,
    /// SVG stroke dash array.
    pub stroke_dash_array: Option<String>,
    /// CSS class attribute of the SVG element.
    pub class: Option<String>,
    /// SVG pathLength attribute (used by animations).
    pub path_length: Option<f32>,
}

impl Default for StraightLine {
    fn default() -> Self {
        StraightLine {
            color: None,
            fill: None,
            points: vec![],
            stroke_width: 1.0,
            symbol: Some(Symbol::Circle(2.0, None)),
            close: false,
            stroke_dash_array: None,
            class: None,
            path_length: None,
        }
    }
}

impl StraightLine {
    /// Renders the component to an SVG fragment.
    pub fn svg(&self) -> String {
        BaseLine {
            color: self.color,
            fill: self.fill,
            points: &self.points,
            stroke_width: self.stroke_width,
            symbol: self.symbol.clone(),
            is_smooth: false,
            close: self.close,
            stroke_dash_array: self.stroke_dash_array.clone(),
            class: self.class.clone(),
            path_length: self.path_length,
        }
        .svg()
    }
}

#[derive(Clone, PartialEq, Debug, Default)]
/// The filled area under a straight-segment line.
pub struct StraightLineFill {
    /// Fill (solid or gradient).
    pub fill: Fill,
    /// The points of the line.
    pub points: Vec<Point>,
    /// Y coordinate of the baseline the fill closes to.
    pub bottom: f32,
    /// Closes the path back to the first point.
    pub close: bool,
}

impl StraightLineFill {
    /// Renders the component to an SVG fragment.
    pub fn svg(&self) -> String {
        self.svg_with_grad_seen(None)
    }
    pub(crate) fn svg_with_grad_seen(&self, grad_seen: Option<&mut HashSet<String>>) -> String {
        if self.points.is_empty() || self.fill.is_transparent() {
            return "".to_string();
        }
        let mut points = Vec::with_capacity(self.points.len() + 3);
        points.extend_from_slice(&self.points);
        let last = self.points[self.points.len() - 1];
        let first = self.points[0];
        points.push((last.x, self.bottom).into());
        points.push((first.x, self.bottom).into());
        points.push(first);
        let mut arr = vec![];
        for (index, p) in points.iter().enumerate() {
            let mut action = "L";
            if index == 0 {
                action = "M"
            }
            arr.push(format!(
                "{} {} {}",
                action,
                format_float(p.x),
                format_float(p.y)
            ));
        }
        if self.close {
            arr.push('Z'.to_string());
        }
        let defs = fill_svg_defs(&self.fill, grad_seen);
        let attrs = vec![
            (ATTR_D, arr.join(" ")),
            (ATTR_FILL, fill_svg_attr(&self.fill)),
            (ATTR_FILL_OPACITY, fill_svg_opacity(&self.fill)),
        ];

        let element = SVGTag {
            tag: TAG_PATH,
            attrs,
            ..Default::default()
        }
        .to_string();
        if defs.is_empty() {
            element
        } else {
            format!("{defs}{element}")
        }
    }
}

#[derive(Clone, PartialEq, Debug, Default)]
/// The chart grid: evenly spaced vertical and horizontal lines.
pub struct Grid {
    /// Left (x) coordinate.
    pub left: f32,
    /// Top (y) coordinate.
    pub top: f32,
    /// Right (x) coordinate.
    pub right: f32,
    /// Bottom (y) coordinate.
    pub bottom: f32,
    /// Stroke color.
    pub color: Option<Color>,
    /// Stroke width.
    pub stroke_width: f32,
    /// Number of vertical grid lines.
    pub verticals: usize,
    /// Indexes of vertical lines to hide.
    pub hidden_verticals: Vec<usize>,
    /// Number of horizontal grid lines.
    pub horizontals: usize,
    /// Indexes of horizontal lines to hide.
    pub hidden_horizontals: Vec<usize>,
}

impl Grid {
    /// Renders the component to an SVG fragment.
    pub fn svg(&self) -> String {
        if (self.verticals == 0 && self.horizontals == 0) || self.stroke_width <= 0.0 {
            return "".to_string();
        }
        let mut points = vec![];
        if self.verticals != 0 {
            let unit = (self.right - self.left) / (self.verticals) as f32;
            for index in 0..=self.verticals {
                if self.hidden_verticals.contains(&index) {
                    continue;
                }
                let x = self.left + unit * index as f32;
                points.push((x, self.top, x, self.bottom));
            }
        }
        if self.horizontals != 0 {
            let unit = (self.bottom - self.top) / (self.horizontals) as f32;
            for index in 0..=self.horizontals {
                if self.hidden_horizontals.contains(&index) {
                    continue;
                }
                let y = self.top + unit * index as f32;
                points.push((self.left, y, self.right, y));
            }
        }
        // The stroke lives on the group; each line only carries its ends.
        let mut out = String::with_capacity(points.len() * 40 + 64);
        let mut w = TagWriter::open(&mut out, TAG_GROUP);
        w.color(ATTR_STROKE, ATTR_STROKE_OPACITY, self.color)
            .float(ATTR_STROKE_WIDTH, self.stroke_width);
        let mut lines = String::with_capacity(points.len() * 40);
        for (left, top, right, bottom) in points.iter() {
            Line {
                left: *left,
                top: *top,
                right: *right,
                bottom: *bottom,
                ..Default::default()
            }
            .write_bare(&mut lines);
        }
        w.close_with(&lines);
        out
    }
}

#[derive(Clone, PartialEq, Debug)]
/// An axis: its line, tick marks and labels.
pub struct Axis {
    /// Which side of the chart the axis sits on.
    pub position: Position,
    /// Number of intervals the axis splits into.
    pub split_number: usize,
    /// Label font size.
    pub font_size: f32,
    /// Label font family.
    pub font_family: String,
    /// Label font color.
    pub font_color: Option<Color>,
    /// Label font weight, e.g. `"bold"`.
    pub font_weight: Option<String>,
    /// The axis labels.
    pub data: Vec<String>,
    /// Label format, supporting `{c}` value and `{t}` thousands.
    pub formatter: Option<String>,
    /// Gap between the axis line and its labels.
    pub name_gap: f32,
    /// Alignment of the labels.
    pub name_align: Align,
    /// Rotation of the labels, in radians.
    pub name_rotate: f32,
    /// Stroke color of the axis line and ticks.
    pub stroke_color: Option<Color>,
    /// Left (x) coordinate.
    pub left: f32,
    /// Top (y) coordinate.
    pub top: f32,
    /// Width.
    pub width: f32,
    /// Height.
    pub height: f32,
    /// Length of the tick marks.
    pub tick_length: f32,
    /// Index of the first tick.
    pub tick_start: usize,
    /// Interval between ticks.
    pub tick_interval: usize,
    /// What to do with labels that do not fit side by side (horizontal
    /// axes only).
    pub label_overflow: AxisLabelOverflow,
}
impl Default for Axis {
    fn default() -> Self {
        Axis {
            position: Position::Bottom,
            split_number: 0,
            font_size: 14.0,
            font_family: font::DEFAULT_FONT_FAMILY.to_string(),
            data: vec![],
            formatter: None,
            font_color: None,
            font_weight: None,
            stroke_color: None,
            name_gap: 5.0,
            name_rotate: 0.0,
            name_align: Align::Center,
            left: 0.0,
            top: 0.0,
            width: 0.0,
            height: 0.0,
            tick_length: 5.0,
            tick_start: 0,
            tick_interval: 0,
            label_overflow: AxisLabelOverflow::Thin,
        }
    }
}

/// A `<line>` with only its end points, for use inside a stroked `<g>`.
fn bare_line((left, top, right, bottom): (f32, f32, f32, f32)) -> String {
    let mut out = String::with_capacity(48);
    Line {
        left,
        top,
        right,
        bottom,
        ..Default::default()
    }
    .write_bare(&mut out);
    out
}

impl Axis {
    /// Renders the component to an SVG fragment.
    pub fn svg(&self) -> Result<String> {
        let left = self.left;
        let top = self.top;
        let width = self.width;
        let height = self.height;
        let tick_length = self.tick_length;

        let mut attrs = vec![];
        let mut is_transparent = false;
        if let Some(color) = self.stroke_color {
            attrs.push((ATTR_STROKE, color.hex()));
            attrs.push((ATTR_STROKE_OPACITY, convert_opacity(&color)));

            is_transparent = color.is_transparent();
        }
        // The stroke width lives on the group; the axis line and ticks only
        // carry their ends.
        attrs.push((ATTR_STROKE_WIDTH, format_float(1.0)));

        let mut line_data = vec![];
        if !is_transparent {
            let values = match self.position {
                Position::Top => {
                    let y = top + height;
                    (left, y, left + width, y)
                }
                Position::Right => {
                    let y = top + height;
                    (left, top, left, y)
                }
                Position::Bottom => (left, top, left + width, top),
                _ => {
                    let x = left + width;
                    (x, top, x, top + height)
                }
            };

            line_data.push(bare_line(values));
        }

        let is_horizontal = self.position == Position::Bottom || self.position == Position::Top;

        let axis_length = if is_horizontal {
            self.width
        } else {
            self.height
        };
        let font_size = self.font_size;
        let formatter = &self.formatter.clone().unwrap_or_default();

        let mut text_list = vec![];
        let mut text_unit_count: usize = 1;
        let mut name_rotate_rad = self.name_rotate;
        if font_size > 0.0 && !self.data.is_empty() {
            text_list = self
                .data
                .iter()
                .map(|item| format_string(item, formatter))
                .collect();
            if self.position == Position::Top || self.position == Position::Bottom {
                // Measured as one string (kerning included) through the
                // per-thread cache, so a re-render of the same axis is free.
                let total_measure =
                    measure_text_width_family(&self.font_family, font_size, &text_list.join(" "))?;
                let mut total_measure_width = total_measure.width();
                let overflowing = total_measure_width > axis_length;
                match self.label_overflow {
                    // Rotate first; whatever still overlaps is thinned below.
                    AxisLabelOverflow::Rotate if overflowing && name_rotate_rad == 0.0 => {
                        name_rotate_rad = std::f32::consts::FRAC_PI_4;
                    }
                    // Every label is cut to its own slot instead of skipped.
                    AxisLabelOverflow::Ellipsis if overflowing => {
                        let slots = if self.name_align == Align::Left {
                            self.data.len().saturating_sub(1)
                        } else {
                            self.data.len()
                        }
                        .max(1);
                        let slot_width = (axis_length / slots as f32 - 4.0).max(font_size);
                        text_list = text_list
                            .iter()
                            .map(|text| {
                                font::text_ellipsis(&self.font_family, font_size, text, slot_width)
                            })
                            .collect();
                        total_measure_width = 0.0;
                    }
                    _ => {}
                }
                if name_rotate_rad != 0.0 {
                    total_measure_width *= name_rotate_rad.sin().abs();
                }
                // 位置不够
                if total_measure_width > axis_length {
                    text_unit_count += (total_measure_width / axis_length).ceil() as usize;
                }
            } else {
                // Vertical axes: labels stack by their line height, so skip
                // every n-th one when there are more than fit.
                let total_height = text_list.len() as f32 * font_size * 1.2;
                if total_height > axis_length {
                    text_unit_count += (total_height / axis_length).ceil() as usize;
                }
            }
        }

        let mut split_number = self.split_number;
        if split_number == 0 {
            split_number = self.data.len();
        }
        // Floor at 1 so an empty axis cannot divide `axis_length` by zero
        // (which would emit `NaN` tick coordinates).
        let split_number = split_number.max(1);
        if !is_transparent {
            let unit = axis_length / split_number as f32;
            let tick_interval = self.tick_interval.max(text_unit_count);
            let tick_start = self.tick_start;
            for i in 0..=split_number {
                if i < tick_start {
                    continue;
                }
                let index = if i > tick_start { i - tick_start } else { i };
                if i != tick_start && (tick_interval != 0 && index % tick_interval != 0) {
                    continue;
                }

                let values = match self.position {
                    Position::Top => {
                        let x = left + unit * i as f32;
                        let y = top + height;
                        (x, y - tick_length, x, y)
                    }
                    Position::Right => {
                        let y = top + unit * i as f32;
                        (left, y, left + tick_length, y)
                    }
                    Position::Bottom => {
                        let x = left + unit * i as f32;
                        (x, top, x, top + tick_length)
                    }
                    _ => {
                        let y = top + unit * i as f32;
                        let x = left + width;
                        (x, y, x - tick_length, y)
                    }
                };

                line_data.push(bare_line(values));
            }
        }
        let mut text_data = vec![];
        let name_rotate = name_rotate_rad / std::f32::consts::PI * 180.0;
        if !text_list.is_empty() {
            let name_gap = self.name_gap;
            let mut data_len = self.data.len();
            let is_name_align_start = self.name_align == Align::Left;
            // Only shrink when there is more than one tick — otherwise this
            // would underflow (usize) on an empty axis — and floor the divisor
            // at 1 so a single-category axis cannot produce an `inf` unit.
            if is_name_align_start && data_len > 1 {
                data_len -= 1;
            }
            let unit = axis_length / data_len.max(1) as f32;

            for (index, text) in text_list.iter().enumerate() {
                if index % text_unit_count != 0 {
                    continue;
                }
                let b = measure_text_width_family(&self.font_family, font_size, text)?;
                let mut unit_offset = unit * index as f32 + unit / 2.0;
                if is_name_align_start {
                    unit_offset -= unit / 2.0;
                }
                let text_width = b.width();

                let values = match self.position {
                    Position::Top => {
                        let y = top + height - name_gap;
                        let x = left + unit_offset - text_width / 2.0;
                        (x, y)
                    }
                    Position::Right => {
                        let x = left + name_gap;
                        let y = top + unit_offset + font_size / 2.0;
                        (x, y)
                    }
                    Position::Bottom => {
                        let y = top + font_size + name_gap;
                        let x = left + unit_offset - text_width / 2.0;
                        (x, y)
                    }
                    _ => {
                        let x = left + width - text_width - name_gap;
                        let y = top + unit_offset + font_size / 2.0 - 2.0;
                        (x, y)
                    }
                };
                let mut transform = None;
                let mut x = Some(values.0);
                let mut y = Some(values.1);
                let mut text_anchor = None;
                if name_rotate != 0.0 {
                    let w = name_rotate_rad.sin().abs() * b.width();
                    let translate_x = (values.0 + b.width() / 2.0) as i32;
                    let translate_y = (values.1 + w / 2.0) as i32;
                    text_anchor = Some("middle".to_string());

                    let a = name_rotate as i32;
                    transform = Some(format!(
                        "translate({translate_x},{translate_y}) rotate({a})"
                    ));
                    x = None;
                    y = None;
                }

                text_data.push(
                    Text {
                        text: text.to_string(),
                        font_family: Some(self.font_family.clone()),
                        font_size: Some(self.font_size),
                        font_color: self.font_color,
                        font_weight: self.font_weight.clone(),
                        x,
                        y,
                        transform,
                        text_anchor,
                        ..Default::default()
                    }
                    .svg(),
                );
            }
        };
        Ok(SVGTag {
            tag: TAG_GROUP,
            data: Some(
                [
                    SVGTag {
                        tag: TAG_GROUP,
                        attrs,
                        data: Some(line_data.join("\n")),
                        ..Default::default()
                    }
                    .to_string(),
                    text_data.join("\n"),
                ]
                .join("\n"),
            ),
            ..Default::default()
        }
        .to_string())
    }
}

pub(crate) static LEGEND_WIDTH: f32 = 25.0;
pub(crate) static LEGEND_HEIGHT: f32 = 20.0;
pub(crate) static LEGEND_TEXT_MARGIN: f32 = 3.0;
pub(crate) static LEGEND_MARGIN: f32 = 8.0;

pub(crate) fn measure_legend_widths(
    font_family: &str,
    font_size: f32,
    legends: &[&str],
) -> Vec<f32> {
    legends
        .iter()
        .map(|item| {
            let text_box = measure_text_width_family(font_family, font_size, item.to_owned())
                .unwrap_or_default();
            text_box.width() + LEGEND_WIDTH + LEGEND_TEXT_MARGIN
        })
        .collect()
}

pub(crate) fn wrap_legends_to_rows<'a>(
    font_family: &str,
    font_size: f32,
    legends: &[&'a str],
    max_width: f32,
) -> Vec<(f32, Vec<&'a str>)> {
    let widths = measure_legend_widths(font_family, font_size, legends);
    let mut rows = Vec::new();
    let mut current_row_legends = Vec::new();
    let mut current_row_width: f32 = 0.0;

    for (&legend_str, &legend_width) in legends.iter().zip(widths.iter()) {
        let cost_to_add = if current_row_legends.is_empty() {
            legend_width
        } else {
            legend_width + LEGEND_MARGIN
        };
        if current_row_width + cost_to_add > max_width && !current_row_legends.is_empty() {
            rows.push((current_row_width, std::mem::take(&mut current_row_legends)));

            current_row_legends.push(legend_str);
            current_row_width = legend_width;
        } else {
            current_row_legends.push(legend_str);
            current_row_width += cost_to_add;
        }
    }

    if !current_row_legends.is_empty() {
        rows.push((current_row_width, current_row_legends));
    }

    rows
}

// pub(crate) fn measure_legends(
//     font_family: &str,
//     font_size: f32,
//     legends: &[&str],
// ) -> (f32, Vec<f32>) {
//     let widths = measure_legend_widths(font_family, font_size, legends);
//     let width: f32 = widths.iter().sum();
//     let margin = LEGEND_MARGIN * (legends.len() - 1) as f32;

//     (width + margin, widths)
// }

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug, Default)]
/// The marker shape of a legend entry.
pub enum LegendCategory {
    #[default]
    /// The default marker: a line with a dot.
    Normal,
    /// A rounded rectangle marker.
    RoundRect,
    /// A circular marker.
    Circle,
    /// A rectangular marker.
    Rect,
}

#[derive(Clone, PartialEq, Debug, Default)]
/// One legend entry: its marker plus label.
pub struct Legend {
    /// The label text.
    pub text: String,
    /// Label font size.
    pub font_size: f32,
    /// Label font family.
    pub font_family: String,
    /// Label font color.
    pub font_color: Option<Color>,
    /// Label font weight, e.g. `"bold"`.
    pub font_weight: Option<String>,
    /// Stroke color of the marker.
    pub stroke_color: Option<Color>,
    /// Fill color of the marker.
    pub fill: Option<Color>,
    /// Left (x) coordinate.
    pub left: f32,
    /// Top (y) coordinate.
    pub top: f32,
    /// Marker shape of the entry.
    pub category: LegendCategory,
}
impl Legend {
    /// Renders the component to an SVG fragment.
    pub fn svg(&self) -> String {
        let stroke_width = 2.0;
        let mut data: Vec<String> = vec![];
        match self.category {
            LegendCategory::Rect => {
                let height = 10.0_f32;
                data.push(
                    Rect {
                        color: self.stroke_color,
                        fill: self.stroke_color.map(|c| c.into()),
                        left: self.left,
                        top: self.top + (LEGEND_HEIGHT - height) / 2.0,
                        width: LEGEND_WIDTH,
                        height,
                        ..Default::default()
                    }
                    .svg(),
                );
            }
            LegendCategory::RoundRect => {
                let height = 10.0_f32;
                data.push(
                    Rect {
                        color: self.stroke_color,
                        fill: self.stroke_color.map(|c| c.into()),
                        left: self.left,
                        top: self.top + (LEGEND_HEIGHT - height) / 2.0,
                        width: LEGEND_WIDTH,
                        height,
                        rx: Some(2.0),
                        ry: Some(2.0),
                        ..Default::default()
                    }
                    .svg(),
                );
            }
            LegendCategory::Circle => {
                data.push(
                    Circle {
                        stroke_width,
                        stroke_color: self.stroke_color,
                        fill: self.fill,
                        cx: self.left + LEGEND_WIDTH * 0.6,
                        cy: self.top + LEGEND_HEIGHT / 2.0,
                        r: 5.5,
                        ..Default::default()
                    }
                    .svg(),
                );
            }
            _ => {
                data.push(
                    Line {
                        stroke_width,
                        color: self.stroke_color,
                        left: self.left,
                        top: self.top + LEGEND_HEIGHT / 2.0,
                        right: self.left + LEGEND_WIDTH,
                        bottom: self.top + LEGEND_HEIGHT / 2.0,
                        ..Default::default()
                    }
                    .svg(),
                );
                data.push(
                    Circle {
                        stroke_width,
                        stroke_color: self.stroke_color,
                        fill: self.fill,
                        cx: self.left + LEGEND_WIDTH / 2.0,
                        cy: self.top + LEGEND_HEIGHT / 2.0,
                        r: 5.5,
                        ..Default::default()
                    }
                    .svg(),
                );
            }
        }
        data.push(
            Text {
                text: self.text.clone(),
                font_family: Some(self.font_family.clone()),
                font_color: self.font_color,
                font_size: Some(self.font_size),
                font_weight: self.font_weight.clone(),
                x: Some(self.left + LEGEND_WIDTH + LEGEND_TEXT_MARGIN),
                y: Some(self.top + self.font_size),
                ..Default::default()
            }
            .svg(),
        );
        SVGTag {
            tag: TAG_GROUP,
            data: Some(data.join("\n")),
            ..Default::default()
        }
        .to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::{
        Arrow, Axis, Bubble, Circle, Fill, Grid, Legend, LegendCategory, Line, Pie, Polygon,
        Polyline, Rect, SmoothLine, SmoothLineFill, StraightLine, StraightLineFill, Text,
        wrap_legends_to_rows,
    };
    use crate::{Align, Color, DEFAULT_FONT_FAMILY, Position, Symbol};
    use pretty_assertions::assert_eq;
    #[test]
    fn test_line() {
        let line = Line::default();
        assert_eq!(1.0, line.stroke_width);
        assert_eq!(None, line.color);

        assert_eq!(
            r###"<line stroke-width="1" x1="0" y1="1" x2="30" y2="5" stroke="#000000"/>"###,
            Line {
                color: Some((0, 0, 0).into()),
                stroke_width: 1.0,
                left: 0.0,
                top: 1.0,
                right: 30.0,
                bottom: 5.0,
                ..Default::default()
            }
            .svg()
        );

        assert_eq!(
            r###"<line stroke-width="1" x1="0" y1="1" x2="30" y2="5" stroke="#000000" stroke-opacity="0.5"/>"###,
            Line {
                color: Some((0, 0, 0, 128).into()),
                stroke_width: 1.0,
                left: 0.0,
                top: 1.0,
                right: 30.0,
                bottom: 5.0,
                ..Default::default()
            }
            .svg()
        );

        assert_eq!(
            r###"<line stroke-width="1" x1="0" y1="1" x2="30" y2="5"/>"###,
            Line {
                color: None,
                stroke_width: 1.0,
                left: 0.0,
                top: 1.0,
                right: 30.0,
                bottom: 5.0,
                ..Default::default()
            }
            .svg()
        );

        assert_eq!(
            r###"<line stroke-width="1" x1="30" y1="10" x2="300" y2="10" stroke="#000000" stroke-opacity="0.5" stroke-dasharray="4,2"/>"###,
            Line {
                color: Some((0, 0, 0, 128).into()),
                stroke_width: 1.0,
                left: 30.0,
                top: 10.0,
                right: 300.0,
                bottom: 10.0,
                stroke_dash_array: Some("4,2".to_string()),
            }
            .svg()
        );
    }

    #[test]
    fn test_rect_dataset() {
        assert_eq!(
            r###"<rect x="0" y="0" width="10" height="10" data-date="2026-01-05" data-value="2" data-note="a&quot;b&amp;c"/>"###,
            Rect {
                left: 0.0,
                top: 0.0,
                width: 10.0,
                height: 10.0,
                dataset: vec![
                    ("date".to_string(), "2026-01-05".to_string()),
                    ("value".to_string(), "2".to_string()),
                    // dropped: empty key, key that is not a valid attribute
                    // name, and empty value
                    ("".to_string(), "x".to_string()),
                    ("bad key".to_string(), "x".to_string()),
                    ("empty".to_string(), "".to_string()),
                    // values are escaped like any other attribute
                    ("note".to_string(), r#"a"b&c"#.to_string()),
                ],
                ..Default::default()
            }
            .svg()
        );
    }

    #[test]
    fn test_rect() {
        assert_eq!(
            r###"<rect x="0" y="0" width="50" height="20" rx="3" ry="4" stroke="#000000" fill="#FFFFFF"/>"###,
            Rect {
                color: Some((0, 0, 0).into()),
                fill: Some(Fill::Solid((255, 255, 255).into())),
                left: 0.0,
                top: 0.0,
                width: 50.0,
                height: 20.0,
                rx: Some(3.0),
                ry: Some(4.0),
                ..Default::default()
            }
            .svg()
        );

        assert_eq!(
            r###"<rect x="0" y="0" width="50" height="20" rx="3" ry="4" stroke="#000000" stroke-opacity="0.5" fill="#FFFFFF" fill-opacity="0.2"/>"###,
            Rect {
                color: Some((0, 0, 0, 128).into()),
                fill: Some(Fill::Solid((255, 255, 255, 50).into())),
                left: 0.0,
                top: 0.0,
                width: 50.0,
                height: 20.0,
                rx: Some(3.0),
                ry: Some(4.0),
                ..Default::default()
            }
            .svg()
        );

        assert_eq!(
            r###"<rect x="0" y="0" width="50" height="20"/>"###,
            Rect {
                left: 0.0,
                top: 0.0,
                width: 50.0,
                height: 20.0,
                ..Default::default()
            }
            .svg()
        );

        // Linear gradient fill
        assert_eq!(
            r###"<defs><linearGradient id="grad_FF0000FF_0000FFFF_90" x1="0.00" y1="0.50" x2="1.00" y2="0.50" gradientUnits="objectBoundingBox"><stop offset="0%" stop-color="#FF0000"/><stop offset="100%" stop-color="#0000FF"/></linearGradient></defs><rect x="0" y="0" width="100" height="50" fill="url(#grad_FF0000FF_0000FFFF_90)"/>"###,
            Rect {
                fill: Some(Fill::LinearGradient {
                    start_color: (255, 0, 0).into(),
                    end_color: (0, 0, 255).into(),
                    angle: 90.0,
                }),
                left: 0.0,
                top: 0.0,
                width: 100.0,
                height: 50.0,
                ..Default::default()
            }
            .svg()
        );
    }

    #[test]
    fn test_bubble() {
        let c = Bubble {
            r: 15.0,
            x: 50.0,
            y: 50.0,
            fill: "#7EB26D".into(),
        };

        assert_eq!(
            r###"<path d="M 40.4,61.5 A 15,15 0,0,1 35,50 A 15,15 0,0,1 65,50 A 15,15 0,0,1 59.6,61.5 L 50,72.5 Z" fill="#7EB26D"/>"###,
            c.svg()
        );
    }

    #[test]
    fn test_polyline() {
        let polyline = Polyline::default();
        assert_eq!(1.0, polyline.stroke_width);
        assert_eq!(None, polyline.color);

        assert_eq!(
            r###"<polyline fill="none" stroke-width="1" points="0,0 10,30 20,60 30,120" stroke="#000000"/>"###,
            Polyline {
                color: Some((0, 0, 0).into()),
                stroke_width: 1.0,
                points: vec![
                    (0.0, 0.0).into(),
                    (10.0, 30.0).into(),
                    (20.0, 60.0).into(),
                    (30.0, 120.0).into(),
                ]
            }
            .svg()
        );

        assert_eq!(
            r###"<polyline fill="none" stroke-width="1" points="0,0 10,30 20,60 30,120" stroke="#000000" stroke-opacity="0.5"/>"###,
            Polyline {
                color: Some((0, 0, 0, 128).into()),
                stroke_width: 1.0,
                points: vec![
                    (0.0, 0.0).into(),
                    (10.0, 30.0).into(),
                    (20.0, 60.0).into(),
                    (30.0, 120.0).into(),
                ]
            }
            .svg()
        );

        assert_eq!(
            r###"<polyline fill="none" stroke-width="1" points="0,0 10,30 20,60 30,120"/>"###,
            Polyline {
                color: None,
                stroke_width: 1.0,
                points: vec![
                    (0.0, 0.0).into(),
                    (10.0, 30.0).into(),
                    (20.0, 60.0).into(),
                    (30.0, 120.0).into(),
                ]
            }
            .svg()
        );
    }

    #[test]
    fn test_circle() {
        let c = Circle::default();
        assert_eq!(None, c.stroke_color);
        assert_eq!(None, c.fill);
        assert_eq!(1.0, c.stroke_width);
        assert_eq!(3.0, c.r);

        assert_eq!(
            r###"<circle cx="10" cy="10" r="3" stroke-width="1" stroke="#000000" fill="#FFFFFF"/>"###,
            Circle {
                stroke_color: Some((0, 0, 0).into()),
                fill: Some((255, 255, 255).into()),
                stroke_width: 1.0,
                cx: 10.0,
                cy: 10.0,
                r: 3.0,
                ..Default::default()
            }
            .svg()
        );

        assert_eq!(
            r###"<circle cx="10" cy="10" r="3" stroke-width="1" stroke="#000000" stroke-opacity="0.5" fill-opacity="0.08" fill="#FFFFFF"/>"###,
            Circle {
                stroke_color: Some((0, 0, 0, 128).into()),
                fill: Some((255, 255, 255, 20).into()),
                stroke_width: 1.0,
                cx: 10.0,
                cy: 10.0,
                r: 3.0,
                ..Default::default()
            }
            .svg()
        );

        assert_eq!(
            r###"<circle cx="10" cy="10" r="3" fill="none"/>"###,
            Circle {
                stroke_color: None,
                fill: None,
                stroke_width: 1.0,
                cx: 10.0,
                cy: 10.0,
                r: 3.0,
                ..Default::default()
            }
            .svg()
        );
    }

    #[test]
    fn test_arrow() {
        assert_eq!(
            r###"<path d="M 30 30 L 25 25 L 40 30 L 25 35 Z" stroke-width="1" fill="#7EB26D" stroke="#7EB26D"/>"###,
            Arrow {
                x: 30.0,
                y: 30.0,
                stroke_color: (126, 178, 109).into(),
                ..Arrow::default()
            }
            .svg()
        );
    }

    #[test]
    fn test_polygon() {
        assert_eq!(
            r###"<polygon points="0,0 10,30 20,60 30,20" stroke="#000000" fill="#FFFFFF"/>"###,
            Polygon {
                color: Some((0, 0, 0).into()),
                fill: Some((255, 255, 255).into()),
                points: vec![
                    (0.0, 0.0).into(),
                    (10.0, 30.0).into(),
                    (20.0, 60.0).into(),
                    (30.0, 20.0).into(),
                ],
                ..Default::default()
            }
            .svg()
        );
        assert_eq!(
            r###"<polygon points="0,0 10,30 20,60 30,20" stroke="#000000" stroke-opacity="0.5" fill="#FFFFFF" fill-opacity="0.08"/>"###,
            Polygon {
                color: Some((0, 0, 0, 128).into()),
                fill: Some((255, 255, 255, 20).into()),
                points: vec![
                    (0.0, 0.0).into(),
                    (10.0, 30.0).into(),
                    (20.0, 60.0).into(),
                    (30.0, 20.0).into(),
                ],
                ..Default::default()
            }
            .svg()
        );
        assert_eq!(
            r###"<polygon points="0,0 10,30 20,60 30,20"/>"###,
            Polygon {
                color: None,
                fill: None,
                points: vec![
                    (0.0, 0.0).into(),
                    (10.0, 30.0).into(),
                    (20.0, 60.0).into(),
                    (30.0, 20.0).into(),
                ],
                ..Default::default()
            }
            .svg()
        );
    }

    #[test]
    fn test_text() {
        assert_eq!(
            r###"<text font-size="14" x="0" y="0" dx="5" dy="5" font-weight="bold" transform="translate(-36 45.5)" font-family="Roboto" fill="#000000">
Hello World!
</text>"###,
            Text {
                text: "Hello World!".to_string(),
                font_family: Some(DEFAULT_FONT_FAMILY.to_string()),
                font_size: Some(14.0),
                font_color: Some((0, 0, 0).into()),
                x: Some(0.0),
                y: Some(0.0),
                dy: Some(5.0),
                dx: Some(5.0),
                font_weight: Some("bold".to_string()),
                transform: Some("translate(-36 45.5)".to_string()),
                ..Default::default()
            }
            .svg()
        );

        assert_eq!(
            r###"<text>
Hello World!
</text>"###,
            Text {
                text: "Hello World!".to_string(),
                ..Default::default()
            }
            .svg()
        );
    }

    #[test]
    fn test_pie() {
        let p = Pie {
            fill: Fill::Solid((0, 0, 0, 128).into()),
            stroke_color: Some((0, 0, 0).into()),
            cx: 250.0,
            cy: 250.0,
            r: 250.0,
            ir: 60.0,
            start_angle: 45.0,
            delta: 45.0,
            ..Default::default()
        };
        assert_eq!(
            r###"<path d="M298.1,201.9 L421.1,78.9 A8 8 0 0 1 432.8,79.5 A250 250 0 0 1 499.8,241.3 A8 8 0 0 1 492,250 L318,250 A8 8 0 0 1 310,247.9 A60 60 0 0 0 293.9,209.1 A8 8 0 0 1 298.1,201.9 Z" fill="#000000" fill-opacity="0.5" stroke="#000000"/>"###,
            p.svg()
        );

        let p = Pie {
            fill: Fill::Solid((0, 0, 0, 128).into()),
            stroke_color: Some((0, 0, 0).into()),
            cx: 250.0,
            cy: 250.0,
            r: 250.0,
            ir: 0.0,
            start_angle: 45.0,
            delta: 45.0,
            border_radius: 0.0,
            ..Default::default()
        };
        assert_eq!(
            r###"<path d="M250,250 L426.8,73.2 A0 0 0 0 1 432.8,79.5 A250 250 0 0 1 499.8,241.3 A0 0 0 0 1 500,250 L250,250 Z" fill="#000000" fill-opacity="0.5" stroke="#000000"/>"###,
            p.svg()
        );

        let p = Pie {
            fill: Fill::Solid((0, 0, 0, 128).into()),
            stroke_color: Some((0, 0, 0).into()),
            cx: 250.0,
            cy: 250.0,
            r: 250.0,
            ir: 0.0,
            start_angle: 45.0,
            delta: 45.0,
            ..Default::default()
        };
        assert_eq!(
            r###"<path d="M250,250 L421.1,78.9 A8 8 0 0 1 432.8,79.5 A250 250 0 0 1 499.8,241.3 A8 8 0 0 1 492,250 L258,250 Z" fill="#000000" fill-opacity="0.5" stroke="#000000"/>"###,
            p.svg()
        );

        let p = Pie {
            fill: Fill::Solid((0, 0, 0, 128).into()),
            stroke_color: Some((0, 0, 0).into()),
            cx: 150.0,
            cy: 150.0,
            r: 50.0,
            ir: 25.0,
            start_angle: 45.0,
            delta: 230.0,
            ..Default::default()
        };
        assert_eq!(
            r###"<path d="M173.3,126.7 L179.7,120.3 A8 8 0 0 1 186.6,115.9 A50 50 0 0 1 115.9,186.6 A50 50 0 0 1 100.1,147.4 A8 8 0 0 1 108.2,146.3 L117.1,147.1 A8 8 0 0 1 125,148.7 A25 25 0 0 0 174.9,152.2 A25 25 0 0 0 168.3,133 A8 8 0 0 1 173.3,126.7 Z" fill="#000000" fill-opacity="0.5" stroke="#000000"/>"###,
            p.svg()
        );
    }

    #[test]
    fn test_smooth_line() {
        let line = SmoothLine::default();
        assert_eq!(None, line.color);
        assert_eq!(1.0, line.stroke_width);
        assert_eq!(Some(Symbol::Circle(2.0, None)), line.symbol);

        assert_eq!(
            r###"<g>
<path d="M0,0 C2.5 7.5, 8.1 22.3, 10 30 C13.1 42.3, 17.7 81.1, 20 80 C22.7 78.6, 26.7 24.9, 30 20 C31.7 17.4, 37.5 42.5, 40 50" stroke-width="1" fill="none" stroke="#000000"/>
<g stroke-width="1" stroke="#000000" fill="#FFFFFF">
<circle cx="0" cy="0" r="3"/>
<circle cx="10" cy="30" r="3"/>
<circle cx="20" cy="80" r="3"/>
<circle cx="30" cy="20" r="3"/>
<circle cx="40" cy="50" r="3"/>
</g>
</g>"###,
            SmoothLine {
                color: Some((0, 0, 0).into()),
                points: vec![
                    (0.0, 0.0).into(),
                    (10.0, 30.0).into(),
                    (20.0, 80.0).into(),
                    (30.0, 20.0).into(),
                    (40.0, 50.0).into(),
                ],
                stroke_width: 1.0,
                symbol: Some(Symbol::Circle(3.0, Some((255, 255, 255).into()))),
                ..Default::default()
            }
            .svg()
        );

        assert_eq!(
            r###"<path d="M0,0 C2.5 7.5, 8.1 22.3, 10 30 C13.1 42.3, 17.7 81.1, 20 80 C22.7 78.6, 26.7 24.9, 30 20 C31.7 17.4, 37.5 42.5, 40 50" stroke-width="1" fill="none"/>"###,
            SmoothLine {
                color: None,
                points: vec![
                    (0.0, 0.0).into(),
                    (10.0, 30.0).into(),
                    (20.0, 80.0).into(),
                    (30.0, 20.0).into(),
                    (40.0, 50.0).into(),
                ],
                stroke_width: 1.0,
                symbol: None,
                ..Default::default()
            }
            .svg()
        );
    }

    #[test]
    fn test_straight_line() {
        let line = StraightLine::default();
        assert_eq!(None, line.color);
        assert_eq!(1.0, line.stroke_width);
        assert_eq!(Some(Symbol::Circle(2.0, None)), line.symbol);

        assert_eq!(
            r###"<g>
<path d="M 0 0 L 10 30 L 20 80 L 30 20 L 40 50" stroke-width="1" fill="none" stroke="#000000"/>
<g stroke-width="1" stroke="#000000" fill="none">
<circle cx="0" cy="0" r="3"/>
<circle cx="10" cy="30" r="3"/>
<circle cx="20" cy="80" r="3"/>
<circle cx="30" cy="20" r="3"/>
<circle cx="40" cy="50" r="3"/>
</g>
</g>"###,
            StraightLine {
                color: Some((0, 0, 0).into()),
                points: vec![
                    (0.0, 0.0).into(),
                    (10.0, 30.0).into(),
                    (20.0, 80.0).into(),
                    (30.0, 20.0).into(),
                    (40.0, 50.0).into(),
                ],
                stroke_width: 1.0,
                symbol: Some(Symbol::Circle(3.0, None)),
                ..Default::default()
            }
            .svg()
        );

        assert_eq!(
            r###"<path d="M 0 0 L 10 30 L 20 80 L 30 20 L 40 50" stroke-width="1" fill="none"/>"###,
            StraightLine {
                color: None,
                points: vec![
                    (0.0, 0.0).into(),
                    (10.0, 30.0).into(),
                    (20.0, 80.0).into(),
                    (30.0, 20.0).into(),
                    (40.0, 50.0).into(),
                ],
                stroke_width: 1.0,
                symbol: None,
                ..Default::default()
            }
            .svg()
        );
    }

    #[test]
    fn test_smooth_line_fill() {
        let fill = SmoothLineFill::default();
        assert_eq!(0.0, fill.bottom);
        assert_eq!(Fill::Solid((255, 255, 255, 255).into()), fill.fill);

        assert_eq!(
            r###"<path d="M0,0 C2.5 7.5, 8.1 22.3, 10 30 C13.1 42.3, 17.7 81.1, 20 80 C22.7 78.6, 26.7 24.9, 30 20 C31.7 17.4, 37.5 42.5, 40 50M 40 50 L 40 100 L 0 100 L 0 0" fill="#000000" fill-opacity="0.5"/>"###,
            SmoothLineFill {
                fill: Fill::Solid((0, 0, 0, 128).into()),
                points: vec![
                    (0.0, 0.0).into(),
                    (10.0, 30.0).into(),
                    (20.0, 80.0).into(),
                    (30.0, 20.0).into(),
                    (40.0, 50.0).into(),
                ],
                bottom: 100.0,
            }
            .svg()
        );
    }
    #[test]
    fn test_straight_line_fill() {
        let fill = StraightLineFill::default();
        assert_eq!(Fill::Solid(Color::default()), fill.fill);
        assert_eq!(0.0, fill.bottom);

        assert_eq!(
            r###"<path d="M 0 0 L 10 30 L 20 80 L 30 20 L 40 50 L 40 100 L 0 100 L 0 0" fill="#000000" fill-opacity="0.5"/>"###,
            StraightLineFill {
                fill: Fill::Solid((0, 0, 0, 128).into()),
                points: vec![
                    (0.0, 0.0).into(),
                    (10.0, 30.0).into(),
                    (20.0, 80.0).into(),
                    (30.0, 20.0).into(),
                    (40.0, 50.0).into(),
                ],
                bottom: 100.0,
                ..Default::default()
            }
            .svg()
        );
    }

    #[test]
    fn test_grid() {
        assert_eq!(
            r###"<g stroke="#000000" stroke-width="1">
<line x1="58.3" y1="10" x2="58.3" y2="300"/><line x1="106.7" y1="10" x2="106.7" y2="300"/><line x1="155" y1="10" x2="155" y2="300"/><line x1="203.3" y1="10" x2="203.3" y2="300"/><line x1="251.7" y1="10" x2="251.7" y2="300"/><line x1="10" y1="68" x2="300" y2="68"/><line x1="10" y1="126" x2="300" y2="126"/><line x1="10" y1="184" x2="300" y2="184"/><line x1="10" y1="242" x2="300" y2="242"/>
</g>"###,
            Grid {
                left: 10.0,
                top: 10.0,
                right: 300.0,
                bottom: 300.0,
                color: Some((0, 0, 0).into()),
                stroke_width: 1.0,
                verticals: 6,
                hidden_verticals: vec![0, 6],
                horizontals: 5,
                hidden_horizontals: vec![0, 5],
            }
            .svg()
        );
    }
    #[test]
    fn test_axis() {
        let a = Axis::default();
        assert_eq!(Position::Bottom, a.position);
        assert_eq!(14.0, a.font_size);
        assert_eq!(DEFAULT_FONT_FAMILY, a.font_family);
        assert_eq!(None, a.font_color);
        assert_eq!(None, a.stroke_color);
        assert_eq!(5.0, a.name_gap);
        assert_eq!(Align::Center, a.name_align);
        assert_eq!(5.0, a.tick_length);

        assert_eq!(
            r###"<g>
<g stroke="#000000" stroke-width="1">
<line x1="0" y1="50" x2="300" y2="50"/>
<line x1="0" y1="50" x2="0" y2="55"/>
<line x1="42.9" y1="50" x2="42.9" y2="55"/>
<line x1="85.7" y1="50" x2="85.7" y2="55"/>
<line x1="128.6" y1="50" x2="128.6" y2="55"/>
<line x1="171.4" y1="50" x2="171.4" y2="55"/>
<line x1="214.3" y1="50" x2="214.3" y2="55"/>
<line x1="257.1" y1="50" x2="257.1" y2="55"/>
<line x1="300" y1="50" x2="300" y2="55"/>
</g>
<text font-size="14" x="7.4" y="69" font-family="Roboto" fill="#000000">
Mon
</text>
<text font-size="14" x="52.3" y="69" font-family="Roboto" fill="#000000">
Tue
</text>
<text font-size="14" x="93.1" y="69" font-family="Roboto" fill="#000000">
Wed
</text>
<text font-size="14" x="138" y="69" font-family="Roboto" fill="#000000">
Thu
</text>
<text font-size="14" x="184.9" y="69" font-family="Roboto" fill="#000000">
Fri
</text>
<text font-size="14" x="224.7" y="69" font-family="Roboto" fill="#000000">
Sat
</text>
<text font-size="14" x="266.6" y="69" font-family="Roboto" fill="#000000">
Sun
</text>
</g>"###,
            Axis {
                position: Position::Bottom,
                split_number: 7,
                font_color: Some((0, 0, 0).into()),
                data: vec![
                    "Mon".to_string(),
                    "Tue".to_string(),
                    "Wed".to_string(),
                    "Thu".to_string(),
                    "Fri".to_string(),
                    "Sat".to_string(),
                    "Sun".to_string(),
                ],
                stroke_color: Some((0, 0, 0).into()),
                left: 0.0,
                top: 50.0,
                width: 300.0,
                height: 30.0,
                ..Default::default()
            }
            .svg()
            .unwrap()
        );
    }

    #[test]
    fn test_legend() {
        assert_eq!(
            r###"<g>
<line stroke-width="2" x1="10" y1="40" x2="35" y2="40" stroke="#000000"/>
<circle cx="22.5" cy="40" r="5.5" stroke-width="2" stroke="#000000" fill="#000000"/>
<text font-size="14" x="38" y="44" font-family="Roboto" fill="#000000">
Line
</text>
</g>"###,
            Legend {
                text: "Line".to_string(),
                font_size: 14.0,
                font_family: DEFAULT_FONT_FAMILY.to_string(),
                font_color: Some((0, 0, 0).into()),
                stroke_color: Some((0, 0, 0).into()),
                fill: Some((0, 0, 0).into()),
                left: 10.0,
                top: 30.0,
                ..Default::default()
            }
            .svg()
        );

        assert_eq!(
            r###"<g>
<rect x="10" y="35" width="25" height="10" stroke="#000000" fill="#000000"/>
<text font-size="14" x="38" y="44" font-family="Roboto" fill="#000000">
Line
</text>
</g>"###,
            Legend {
                text: "Line".to_string(),
                font_size: 14.0,
                font_family: DEFAULT_FONT_FAMILY.to_string(),
                font_color: Some((0, 0, 0).into()),
                stroke_color: Some((0, 0, 0).into()),
                fill: Some((0, 0, 0).into()),
                left: 10.0,
                top: 30.0,
                category: LegendCategory::Rect,
                ..Default::default()
            }
            .svg()
        );
    }

    #[test]
    fn test_wrap_legends_to_rows() {
        let rows = wrap_legends_to_rows(
            DEFAULT_FONT_FAMILY,
            14.0,
            &[
                "M1 - End of Inception",
                "M2 - End of Elaboration",
                "M3 - End of Construction",
                "M4 - End of Completion",
            ],
            600.0,
        );
        assert_eq!(
            rows,
            vec![
                (
                    551.0,
                    vec![
                        "M1 - End of Inception",
                        "M2 - End of Elaboration",
                        "M3 - End of Construction"
                    ]
                ),
                (181.0, vec!["M4 - End of Completion"])
            ]
        );
    }
}
