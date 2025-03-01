// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

use serde::{Deserialize, Serialize};
use snafu::{ResultExt, Snafu};
use std::fmt;
use std::vec;

use super::color::*;
use super::common::*;
use super::font;
use super::measure_text_width_family;
use super::path::*;
use super::util::*;

static TAG_SVG: &str = "svg";
static TAG_LINE: &str = "line";
static TAG_RECT: &str = "rect";
static TAG_POLYLINE: &str = "polyline";
static TAG_CIRCLE: &str = "circle";
static TAG_POLYGON: &str = "polygon";
static TAG_TEXT: &str = "text";
static TAG_PATH: &str = "path";
static TAG_GROUP: &str = "g";

static ATTR_VIEW_BOX: &str = "viewBox";
static ATTR_XMLNS: &str = "xmlns";
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

/// Converts opacity to string value.
fn convert_opacity(color: &Color) -> String {
    if color.is_nontransparent() {
        "".to_string()
    } else {
        format_float(color.opacity())
    }
}

fn format_option_float(value: Option<f32>) -> String {
    if let Some(f) = value {
        format_float(f)
    } else {
        "".to_string()
    }
}

#[derive(Clone, PartialEq, Debug, Default)]
struct SVGTag<'a> {
    tag: &'a str,
    attrs: Vec<(&'a str, String)>,
    data: Option<String>,
}

pub fn generate_svg(width: f32, height: f32, x: f32, y: f32, data: String) -> String {
    let mut attrs = vec![
        (ATTR_WIDTH, format!("{}", width)),
        (ATTR_HEIGHT, format!("{}", height)),
        (ATTR_VIEW_BOX, format!("0 0 {} {}", width, height)),
        (ATTR_XMLNS, "http://www.w3.org/2000/svg".to_string()),
    ];
    if x != 0.0 {
        attrs.push((ATTR_X, format!("{}", x)))
    }
    if y != 0.0 {
        attrs.push((ATTR_Y, format!("{}", y)))
    }
    SVGTag::new(TAG_SVG, data, attrs).to_string()
}

impl<'a> SVGTag<'a> {
    pub fn new(tag: &'a str, data: String, attrs: Vec<(&'a str, String)>) -> Self {
        Self {
            tag,
            attrs,
            data: Some(data),
        }
    }
}

impl fmt::Display for SVGTag<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.tag == TAG_GROUP {
            if let Some(ref data) = self.data {
                if data.is_empty() {
                    return write!(f, "");
                }
            }
        }
        let mut value = "<".to_string();
        value.push_str(self.tag);
        for (k, v) in self.attrs.iter() {
            if k.is_empty() || v.is_empty() {
                continue;
            }
            value.push(' ');
            value.push_str(k);
            value.push_str("=\"");
            value.push_str(v);
            value.push('\"');
        }
        if let Some(ref data) = self.data {
            value.push_str(">\n");
            value.push_str(data);
            value.push_str(&format!("\n</{}>", self.tag));
        } else {
            value.push_str("/>");
        }
        write!(f, "{}", value)
    }
}

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("Error get font: {source}"))]
    GetFont { source: font::Error },
}

pub type Result<T, E = Error> = std::result::Result<T, E>;

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
}
#[derive(Clone, PartialEq, Debug)]

pub struct Line {
    pub color: Option<Color>,
    pub stroke_width: f32,
    pub left: f32,
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
    // dash array
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
    pub fn svg(&self) -> String {
        if self.stroke_width <= 0.0 {
            return "".to_string();
        }
        let mut attrs = vec![
            (ATTR_STROKE_WIDTH, format_float(self.stroke_width)),
            (ATTR_X1, format_float(self.left)),
            (ATTR_Y1, format_float(self.top)),
            (ATTR_X2, format_float(self.right)),
            (ATTR_Y2, format_float(self.bottom)),
        ];
        if let Some(color) = self.color {
            attrs.push((ATTR_STROKE, color.hex()));
            attrs.push((ATTR_STROKE_OPACITY, convert_opacity(&color)));
        }
        if let Some(ref stroke_dash_array) = self.stroke_dash_array {
            attrs.push((ATTR_STROKE_DASH_ARRAY, stroke_dash_array.to_string()));
        }
        SVGTag {
            tag: TAG_LINE,
            attrs,
            data: None,
        }
        .to_string()
    }
}

#[derive(Clone, PartialEq, Debug, Default)]
pub struct Rect {
    pub color: Option<Color>,
    pub fill: Option<Color>,
    pub left: f32,
    pub top: f32,
    pub width: f32,
    pub height: f32,
    pub rx: Option<f32>,
    pub ry: Option<f32>,
}
impl Rect {
    pub fn svg(&self) -> String {
        let mut attrs = vec![
            (ATTR_X, format_float(self.left)),
            (ATTR_Y, format_float(self.top)),
            (ATTR_WIDTH, format_float(self.width)),
            (ATTR_HEIGHT, format_float(self.height)),
            (ATTR_RX, format_option_float(self.rx)),
            (ATTR_RY, format_option_float(self.ry)),
        ];

        if let Some(color) = self.color {
            attrs.push((ATTR_STROKE, color.hex()));
            attrs.push((ATTR_STROKE_OPACITY, convert_opacity(&color)));
        }
        if let Some(color) = self.fill {
            if color.is_transparent() {
                attrs.push((ATTR_FILL, "none".to_string()));
            } else {
                attrs.push((ATTR_FILL, color.hex()));
                attrs.push((ATTR_FILL_OPACITY, convert_opacity(&color)));
            }
        }

        SVGTag {
            tag: TAG_RECT,
            attrs,
            data: None,
        }
        .to_string()
    }
}

#[derive(Clone, PartialEq, Debug)]
pub struct Polyline {
    pub color: Option<Color>,
    pub stroke_width: f32,
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
    pub fn svg(&self) -> String {
        if self.stroke_width <= 0.0 {
            return "".to_string();
        }
        let points: Vec<String> = self
            .points
            .iter()
            .map(|p| format!("{},{}", format_float(p.x), format_float(p.y)))
            .collect();
        let mut attrs = vec![
            (ATTR_FILL, "none".to_string()),
            (ATTR_STROKE_WIDTH, format_float(self.stroke_width)),
            (ATTR_POINTS, points.join(" ")),
        ];

        if let Some(color) = self.color {
            attrs.push((ATTR_STROKE, color.hex()));
            attrs.push((ATTR_STROKE_OPACITY, convert_opacity(&color)));
        }

        SVGTag {
            tag: TAG_POLYLINE,
            attrs,
            data: None,
        }
        .to_string()
    }
}

#[derive(Clone, PartialEq, Debug)]
pub struct Circle {
    pub stroke_color: Option<Color>,
    pub fill: Option<Color>,
    pub stroke_width: f32,
    pub cx: f32,
    pub cy: f32,
    pub r: f32,
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
        }
    }
}

impl Circle {
    pub fn svg(&self) -> String {
        let mut attrs = vec![
            (ATTR_CX, format_float(self.cx)),
            (ATTR_CY, format_float(self.cy)),
            (ATTR_R, format_float(self.r)),
            (ATTR_STROKE_WIDTH, format_float(self.stroke_width)),
        ];
        if let Some(color) = self.stroke_color {
            attrs.push((ATTR_STROKE, color.hex()));
            attrs.push((ATTR_STROKE_OPACITY, convert_opacity(&color)));
        }
        let mut fill = "none".to_string();
        if let Some(color) = self.fill {
            fill = color.hex();
            attrs.push((ATTR_FILL_OPACITY, convert_opacity(&color)));
        }
        attrs.push((ATTR_FILL, fill));

        SVGTag {
            tag: TAG_CIRCLE,
            attrs,
            data: None,
        }
        .to_string()
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
pub struct Polygon {
    pub color: Option<Color>,
    pub fill: Option<Color>,
    pub points: Vec<Point>,
}

impl Polygon {
    pub fn svg(&self) -> String {
        if self.points.is_empty() {
            return "".to_string();
        }
        let points: Vec<String> = self
            .points
            .iter()
            .map(|p| format!("{},{}", format_float(p.x), format_float(p.y)))
            .collect();
        let mut attrs = vec![(ATTR_POINTS, points.join(" "))];
        if let Some(color) = self.color {
            attrs.push((ATTR_STROKE, color.hex()));
            attrs.push((ATTR_STROKE_OPACITY, convert_opacity(&color)));
        }
        if let Some(color) = self.fill {
            attrs.push((ATTR_FILL, color.hex()));
            attrs.push((ATTR_FILL_OPACITY, convert_opacity(&color)));
        }
        SVGTag {
            tag: TAG_POLYGON,
            attrs,
            data: None,
        }
        .to_string()
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
pub struct Text {
    pub text: String,
    pub font_family: Option<String>,
    pub font_size: Option<f32>,
    pub font_color: Option<Color>,
    pub line_height: Option<f32>,
    pub x: Option<f32>,
    pub y: Option<f32>,
    pub dx: Option<f32>,
    pub dy: Option<f32>,
    pub font_weight: Option<String>,
    pub transform: Option<String>,
    pub dominant_baseline: Option<String>,
    pub text_anchor: Option<String>,
    pub alignment_baseline: Option<String>,
}

impl Text {
    pub fn svg(&self) -> String {
        if self.text.is_empty() {
            return "".to_string();
        }
        let mut attrs = vec![
            (ATTR_FONT_SIZE, format_option_float(self.font_size)),
            (ATTR_X, format_option_float(self.x)),
            (ATTR_Y, format_option_float(self.y)),
            (ATTR_DX, format_option_float(self.dx)),
            (ATTR_DY, format_option_float(self.dy)),
            (
                ATTR_FONT_WEIGHT,
                self.font_weight.clone().unwrap_or_default(),
            ),
            (ATTR_TRANSFORM, self.transform.clone().unwrap_or_default()),
            (
                ATTR_DOMINANT_BASELINE,
                self.dominant_baseline.clone().unwrap_or_default(),
            ),
            (
                ATTR_TEXT_ANCHOR,
                self.text_anchor.clone().unwrap_or_default(),
            ),
            (
                ATTR_ALIGNMENT_BASELINE,
                self.alignment_baseline.clone().unwrap_or_default(),
            ),
        ];
        if let Some(ref font_family) = self.font_family {
            attrs.push((ATTR_FONT_FAMILY, font_family.clone()));
        }
        if let Some(color) = self.font_color {
            attrs.push((ATTR_FILL, color.hex()));
            attrs.push((ATTR_FILL_OPACITY, convert_opacity(&color)));
        }

        SVGTag {
            tag: TAG_TEXT,
            attrs,
            data: Some(self.text.clone()),
        }
        .to_string()
    }
}

fn generate_circle_symbol(points: &[Point], c: Circle) -> String {
    let mut arr = vec![];
    for p in points.iter() {
        let mut tmp = c.clone();
        tmp.cx = p.x;
        tmp.cy = p.y;
        arr.push(tmp.svg());
    }
    arr.join("\n")
}

#[derive(Clone, PartialEq, Debug)]
pub struct Pie {
    pub fill: Color,
    pub stroke_color: Option<Color>,
    pub cx: f32,
    pub cy: f32,
    pub r: f32,
    pub ir: f32,
    pub start_angle: f32,
    pub delta: f32,
    pub border_radius: f32,
}

impl Default for Pie {
    fn default() -> Self {
        Pie {
            fill: (0, 0, 0).into(),
            stroke_color: None,
            cx: 0.0,
            cy: 0.0,
            r: 250.0,
            ir: 60.0,
            start_angle: 0.0,
            delta: 0.0,
            border_radius: 8.0,
        }
    }
}

impl Pie {
    pub fn svg(&self) -> String {
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

        let mut attrs = vec![
            (ATTR_D, path_list.join(" ")),
            (ATTR_FILL, self.fill.hex()),
            (ATTR_FILL_OPACITY, convert_opacity(&self.fill)),
        ];
        if let Some(color) = self.stroke_color {
            attrs.push((ATTR_STROKE, color.hex()));
            attrs.push((ATTR_STROKE_OPACITY, convert_opacity(&color)));
        }
        SVGTag {
            tag: TAG_PATH,
            attrs,
            ..Default::default()
        }
        .to_string()
    }
}

struct BaseLine {
    pub color: Option<Color>,
    pub fill: Option<Color>,
    pub points: Vec<Point>,
    pub stroke_width: f32,
    pub symbol: Option<Symbol>,
    pub is_smooth: bool,
    pub close: bool,
    pub stroke_dash_array: Option<String>,
}

impl BaseLine {
    pub fn svg(&self) -> String {
        if self.points.is_empty() || self.stroke_width <= 0.0 {
            return "".to_string();
        }
        let path = if self.is_smooth {
            SmoothCurve {
                points: self.points.clone(),
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
        let line_svg = SVGTag {
            tag: TAG_PATH,
            attrs,
            data: None,
        }
        .to_string();
        let symbol_svg = if let Some(ref symbol) = self.symbol {
            match symbol {
                Symbol::Circle(r, fill) => generate_circle_symbol(
                    &self.points,
                    Circle {
                        stroke_color: self.color,
                        fill: fill.to_owned(),
                        stroke_width: self.stroke_width,
                        r: r.to_owned(),
                        ..Default::default()
                    },
                ),
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
pub struct SmoothLine {
    pub color: Option<Color>,
    pub points: Vec<Point>,
    pub stroke_width: f32,
    pub symbol: Option<Symbol>,
    pub stroke_dash_array: Option<String>,
}

impl Default for SmoothLine {
    fn default() -> Self {
        SmoothLine {
            color: None,
            points: vec![],
            stroke_width: 1.0,
            symbol: Some(Symbol::Circle(2.0, None)),
            stroke_dash_array: None,
        }
    }
}

impl SmoothLine {
    pub fn svg(&self) -> String {
        BaseLine {
            color: self.color,
            fill: None,
            points: self.points.clone(),
            stroke_width: self.stroke_width,
            symbol: self.symbol.clone(),
            is_smooth: true,
            close: false,
            stroke_dash_array: self.stroke_dash_array.clone(),
        }
        .svg()
    }
}

#[derive(Clone, PartialEq, Debug)]
pub struct SmoothLineFill {
    pub fill: Color,
    pub points: Vec<Point>,
    pub bottom: f32,
}

impl Default for SmoothLineFill {
    fn default() -> Self {
        SmoothLineFill {
            fill: (255, 255, 255, 255).into(),
            points: vec![],
            bottom: 0.0,
        }
    }
}

impl SmoothLineFill {
    pub fn svg(&self) -> String {
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

        let attrs = vec![
            (ATTR_D, path),
            (ATTR_FILL, self.fill.hex()),
            (ATTR_FILL_OPACITY, convert_opacity(&self.fill)),
        ];

        SVGTag {
            tag: TAG_PATH,
            attrs,
            data: None,
        }
        .to_string()
    }
}

#[derive(Clone, PartialEq, Debug)]
pub struct StraightLine {
    pub color: Option<Color>,
    pub fill: Option<Color>,
    pub points: Vec<Point>,
    pub stroke_width: f32,
    pub symbol: Option<Symbol>,
    pub close: bool,
    pub stroke_dash_array: Option<String>,
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
        }
    }
}

impl StraightLine {
    pub fn svg(&self) -> String {
        BaseLine {
            color: self.color,
            fill: self.fill,
            points: self.points.clone(),
            stroke_width: self.stroke_width,
            symbol: self.symbol.clone(),
            is_smooth: false,
            close: self.close,
            stroke_dash_array: self.stroke_dash_array.clone(),
        }
        .svg()
    }
}

#[derive(Clone, PartialEq, Debug, Default)]
pub struct StraightLineFill {
    pub fill: Color,
    pub points: Vec<Point>,
    pub bottom: f32,
    pub close: bool,
}

impl StraightLineFill {
    pub fn svg(&self) -> String {
        if self.points.is_empty() || self.fill.is_transparent() {
            return "".to_string();
        }
        let mut points = self.points.clone();
        let last = points[self.points.len() - 1];
        let first = points[0];
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
        let attrs = vec![
            (ATTR_D, arr.join(" ")),
            (ATTR_FILL, self.fill.hex()),
            (ATTR_FILL_OPACITY, convert_opacity(&self.fill)),
        ];

        SVGTag {
            tag: TAG_PATH,
            attrs,
            data: None,
        }
        .to_string()
    }
}

#[derive(Clone, PartialEq, Debug, Default)]
pub struct Grid {
    pub left: f32,
    pub top: f32,
    pub right: f32,
    pub bottom: f32,
    pub color: Option<Color>,
    pub stroke_width: f32,
    pub verticals: usize,
    pub hidden_verticals: Vec<usize>,
    pub horizontals: usize,
    pub hidden_horizontals: Vec<usize>,
}

impl Grid {
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
        let mut data = vec![];
        for (left, top, right, bottom) in points.iter() {
            let svg = Line {
                color: None,
                stroke_width: self.stroke_width,
                left: left.to_owned(),
                top: top.to_owned(),
                right: right.to_owned(),
                bottom: bottom.to_owned(),
                ..Default::default()
            }
            .svg();
            data.push(svg);
        }

        let mut attrs = vec![];
        if let Some(color) = self.color {
            attrs.push((ATTR_STROKE, color.hex()));
            attrs.push((ATTR_STROKE_OPACITY, convert_opacity(&color)));
        }

        SVGTag {
            tag: TAG_GROUP,
            attrs,
            data: Some(data.join("")),
        }
        .to_string()
    }
}

#[derive(Clone, PartialEq, Debug)]
pub struct Axis {
    pub position: Position,
    pub split_number: usize,
    pub font_size: f32,
    pub font_family: String,
    pub font_color: Option<Color>,
    pub font_weight: Option<String>,
    pub data: Vec<String>,
    pub formatter: Option<String>,
    pub name_gap: f32,
    pub name_align: Align,
    pub name_rotate: f32,
    pub stroke_color: Option<Color>,
    pub left: f32,
    pub top: f32,
    pub width: f32,
    pub height: f32,
    pub tick_length: f32,
    pub tick_start: usize,
    pub tick_interval: usize,
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
        }
    }
}

impl Axis {
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

        let stroke_width = 1.0;

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

            line_data.push(
                Line {
                    stroke_width,
                    left: values.0,
                    top: values.1,
                    right: values.2,
                    bottom: values.3,
                    ..Default::default()
                }
                .svg(),
            )
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
        if font_size > 0.0 && !self.data.is_empty() {
            text_list = self
                .data
                .iter()
                .map(|item| format_string(item, formatter))
                .collect();
            if self.position == Position::Top || self.position == Position::Bottom {
                let f = font::get_font(&self.font_family).context(GetFontSnafu)?;
                let total_measure = font::measure_text(f, font_size, &text_list.join(" "));
                // 位置不够
                if total_measure.width() > axis_length {
                    text_unit_count += (total_measure.width() / axis_length).ceil() as usize;
                }
            }
        }

        let mut split_number = self.split_number;
        if split_number == 0 {
            split_number = self.data.len();
        }
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

                line_data.push(
                    Line {
                        stroke_width,
                        left: values.0,
                        top: values.1,
                        right: values.2,
                        bottom: values.3,
                        ..Default::default()
                    }
                    .svg(),
                );
            }
        }
        let mut text_data = vec![];
        let name_rotate = self.name_rotate / std::f32::consts::PI * 180.0;
        if !text_list.is_empty() {
            let name_gap = self.name_gap;
            let f = font::get_font(&self.font_family).context(GetFontSnafu)?;
            let mut data_len = self.data.len();
            let is_name_align_start = self.name_align == Align::Left;
            if is_name_align_start {
                data_len -= 1;
            }
            let unit = axis_length / data_len as f32;

            for (index, text) in text_list.iter().enumerate() {
                if index % text_unit_count != 0 {
                    continue;
                }
                let b = font::measure_text(f, font_size, text);
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
                    let w = self.name_rotate.sin().abs() * b.width();
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

pub(crate) fn measure_legends(
    font_family: &str,
    font_size: f32,
    legends: &[&str],
) -> (f32, Vec<f32>) {
    let widths: Vec<f32> = legends
        .iter()
        .map(|item| {
            let text_box = measure_text_width_family(font_family, font_size, item.to_owned())
                .unwrap_or_default();
            text_box.width() + LEGEND_WIDTH + LEGEND_TEXT_MARGIN
        })
        .collect();
    let width: f32 = widths.iter().sum();
    let margin = LEGEND_MARGIN * (legends.len() - 1) as f32;

    (width + margin, widths)
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Debug, Default)]
pub enum LegendCategory {
    #[default]
    Normal,
    RoundRect,
    Circle,
    Rect,
}

#[derive(Clone, PartialEq, Debug, Default)]
pub struct Legend {
    pub text: String,
    pub font_size: f32,
    pub font_family: String,
    pub font_color: Option<Color>,
    pub font_weight: Option<String>,
    pub stroke_color: Option<Color>,
    pub fill: Option<Color>,
    pub left: f32,
    pub top: f32,
    pub category: LegendCategory,
}
impl Legend {
    pub fn svg(&self) -> String {
        let stroke_width = 2.0;
        let mut data: Vec<String> = vec![];
        match self.category {
            LegendCategory::Rect => {
                let height = 10.0_f32;
                data.push(
                    Rect {
                        color: self.stroke_color,
                        fill: self.stroke_color,
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
                        fill: self.stroke_color,
                        left: self.left,
                        top: self.top + (LEGEND_HEIGHT - height) / 2.0,
                        width: LEGEND_WIDTH,
                        height,
                        rx: Some(2.0),
                        ry: Some(2.0),
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
        Arrow, Axis, Bubble, Circle, Grid, Legend, LegendCategory, Line, Pie, Polygon, Polyline,
        Rect, SmoothLine, SmoothLineFill, StraightLine, StraightLineFill, Text,
    };
    use crate::{Align, Position, Symbol, DEFAULT_FONT_FAMILY};
    use pretty_assertions::assert_eq;
    #[test]
    fn line() {
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
    fn rect() {
        assert_eq!(
            r###"<rect x="0" y="0" width="50" height="20" rx="3" ry="4" stroke="#000000" fill="#FFFFFF"/>"###,
            Rect {
                color: Some((0, 0, 0).into()),
                fill: Some((255, 255, 255).into()),
                left: 0.0,
                top: 0.0,
                width: 50.0,
                height: 20.0,
                rx: Some(3.0),
                ry: Some(4.0),
            }
            .svg()
        );

        assert_eq!(
            r###"<rect x="0" y="0" width="50" height="20" rx="3" ry="4" stroke="#000000" stroke-opacity="0.5" fill="#FFFFFF" fill-opacity="0.2"/>"###,
            Rect {
                color: Some((0, 0, 0, 128).into()),
                fill: Some((255, 255, 255, 50).into()),
                left: 0.0,
                top: 0.0,
                width: 50.0,
                height: 20.0,
                rx: Some(3.0),
                ry: Some(4.0),
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
    }

    #[test]
    fn bubble() {
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
    fn polyline() {
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
    fn circle() {
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
            }
            .svg()
        );

        assert_eq!(
            r###"<circle cx="10" cy="10" r="3" stroke-width="1" stroke="#000000" stroke-opacity="0.5" fill-opacity="0.1" fill="#FFFFFF"/>"###,
            Circle {
                stroke_color: Some((0, 0, 0, 128).into()),
                fill: Some((255, 255, 255, 20).into()),
                stroke_width: 1.0,
                cx: 10.0,
                cy: 10.0,
                r: 3.0,
            }
            .svg()
        );

        assert_eq!(
            r###"<circle cx="10" cy="10" r="3" stroke-width="1" fill="none"/>"###,
            Circle {
                stroke_color: None,
                fill: None,
                stroke_width: 1.0,
                cx: 10.0,
                cy: 10.0,
                r: 3.0,
            }
            .svg()
        );
    }

    #[test]
    fn arrow() {
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
    fn polygon() {
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
            }
            .svg()
        );
        assert_eq!(
            r###"<polygon points="0,0 10,30 20,60 30,20" stroke="#000000" stroke-opacity="0.5" fill="#FFFFFF" fill-opacity="0.1"/>"###,
            Polygon {
                color: Some((0, 0, 0, 128).into()),
                fill: Some((255, 255, 255, 20).into()),
                points: vec![
                    (0.0, 0.0).into(),
                    (10.0, 30.0).into(),
                    (20.0, 60.0).into(),
                    (30.0, 20.0).into(),
                ],
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
            }
            .svg()
        );
    }

    #[test]
    fn text() {
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
    fn pie() {
        let p = Pie {
            fill: (0, 0, 0, 128).into(),
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
            fill: (0, 0, 0, 128).into(),
            stroke_color: Some((0, 0, 0).into()),
            cx: 250.0,
            cy: 250.0,
            r: 250.0,
            ir: 0.0,
            start_angle: 45.0,
            delta: 45.0,
            border_radius: 0.0,
        };
        assert_eq!(
            r###"<path d="M250,250 L426.8,73.2 A0 0 0 0 1 432.8,79.5 A250 250 0 0 1 499.8,241.3 A0 0 0 0 1 500,250 L250,250 Z" fill="#000000" fill-opacity="0.5" stroke="#000000"/>"###,
            p.svg()
        );

        let p = Pie {
            fill: (0, 0, 0, 128).into(),
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
            fill: (0, 0, 0, 128).into(),
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
    fn smooth_line() {
        let line = SmoothLine::default();
        assert_eq!(None, line.color);
        assert_eq!(1.0, line.stroke_width);
        assert_eq!(Some(Symbol::Circle(2.0, None)), line.symbol);

        assert_eq!(
            r###"<g>
<path d="M0,0 C2.5 7.5, 8.1 22.3, 10 30 C13.1 42.3, 17.7 81.1, 20 80 C22.7 78.6, 26.7 24.9, 30 20 C31.7 17.4, 37.5 42.5, 40 50" stroke-width="1" fill="none" stroke="#000000"/>
<circle cx="0" cy="0" r="3" stroke-width="1" stroke="#000000" fill="#FFFFFF"/>
<circle cx="10" cy="30" r="3" stroke-width="1" stroke="#000000" fill="#FFFFFF"/>
<circle cx="20" cy="80" r="3" stroke-width="1" stroke="#000000" fill="#FFFFFF"/>
<circle cx="30" cy="20" r="3" stroke-width="1" stroke="#000000" fill="#FFFFFF"/>
<circle cx="40" cy="50" r="3" stroke-width="1" stroke="#000000" fill="#FFFFFF"/>
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
    fn straight_line() {
        let line = StraightLine::default();
        assert_eq!(None, line.color);
        assert_eq!(1.0, line.stroke_width);
        assert_eq!(Some(Symbol::Circle(2.0, None)), line.symbol);

        assert_eq!(
            r###"<g>
<path d="M 0 0 L 10 30 L 20 80 L 30 20 L 40 50" stroke-width="1" fill="none" stroke="#000000"/>
<circle cx="0" cy="0" r="3" stroke-width="1" stroke="#000000" fill="none"/>
<circle cx="10" cy="30" r="3" stroke-width="1" stroke="#000000" fill="none"/>
<circle cx="20" cy="80" r="3" stroke-width="1" stroke="#000000" fill="none"/>
<circle cx="30" cy="20" r="3" stroke-width="1" stroke="#000000" fill="none"/>
<circle cx="40" cy="50" r="3" stroke-width="1" stroke="#000000" fill="none"/>
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
    fn smooth_line_fill() {
        let fill = SmoothLineFill::default();
        assert_eq!(0.0, fill.bottom);
        assert_eq!("rgba(255,255,255,1.0)", fill.fill.rgba());

        assert_eq!(
            r###"<path d="M0,0 C2.5 7.5, 8.1 22.3, 10 30 C13.1 42.3, 17.7 81.1, 20 80 C22.7 78.6, 26.7 24.9, 30 20 C31.7 17.4, 37.5 42.5, 40 50M 40 50 L 40 100 L 0 100 L 0 0" fill="#000000" fill-opacity="0.5"/>"###,
            SmoothLineFill {
                fill: (0, 0, 0, 128).into(),
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
    fn straight_line_fill() {
        let fill = StraightLineFill::default();
        assert_eq!("rgba(0,0,0,0.0)", fill.fill.rgba());
        assert_eq!(0.0, fill.bottom);

        assert_eq!(
            r###"<path d="M 0 0 L 10 30 L 20 80 L 30 20 L 40 50 L 40 100 L 0 100 L 0 0" fill="#000000" fill-opacity="0.5"/>"###,
            StraightLineFill {
                fill: (0, 0, 0, 128).into(),
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
    fn grid() {
        assert_eq!(
            r###"<g stroke="#000000">
<line stroke-width="1" x1="58.3" y1="10" x2="58.3" y2="300"/><line stroke-width="1" x1="106.7" y1="10" x2="106.7" y2="300"/><line stroke-width="1" x1="155" y1="10" x2="155" y2="300"/><line stroke-width="1" x1="203.3" y1="10" x2="203.3" y2="300"/><line stroke-width="1" x1="251.7" y1="10" x2="251.7" y2="300"/><line stroke-width="1" x1="10" y1="68" x2="300" y2="68"/><line stroke-width="1" x1="10" y1="126" x2="300" y2="126"/><line stroke-width="1" x1="10" y1="184" x2="300" y2="184"/><line stroke-width="1" x1="10" y1="242" x2="300" y2="242"/>
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
    fn axis() {
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
<g stroke="#000000">
<line stroke-width="1" x1="0" y1="50" x2="300" y2="50"/>
<line stroke-width="1" x1="0" y1="50" x2="0" y2="55"/>
<line stroke-width="1" x1="42.9" y1="50" x2="42.9" y2="55"/>
<line stroke-width="1" x1="85.7" y1="50" x2="85.7" y2="55"/>
<line stroke-width="1" x1="128.6" y1="50" x2="128.6" y2="55"/>
<line stroke-width="1" x1="171.4" y1="50" x2="171.4" y2="55"/>
<line stroke-width="1" x1="214.3" y1="50" x2="214.3" y2="55"/>
<line stroke-width="1" x1="257.1" y1="50" x2="257.1" y2="55"/>
<line stroke-width="1" x1="300" y1="50" x2="300" y2="55"/>
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
    fn legend() {
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
}
