use std::fmt;
use substring::Substring;

use super::color::*;
use super::util::*;

static TAG_SVG: &str = "svg";
static TAG_LINE: &str = "line";
static TAG_RECT: &str = "rect";
static TAG_POLYLINE: &str = "polyline";
static TAG_CIRCLE: &str = "circle";
static TAG_POLYGON: &str = "polygon";
static TAG_TEXT: &str = "text";

static ATTR_VIEW_BOX: &str = "viewBox";
static ATTR_XMLNS: &str = "xmlns";
static ATTR_HEIGHT: &str = "height";
static ATTR_WIDTH: &str = "width";
static ATTR_FONT_FAMILY: &str = "font-family";
static ATTR_FONT_SIZE: &str = "font-size";
static ATTR_FONT_WEIGHT: &str = "font-weight";
static ATTR_TRANSFORM: &str = "transform";
static ATTR_OPACITY: &str = "opacity";
static ATTR_STROKE_OPACITY: &str = "stroke-opacity";
static ATTR_FILL_OPACITY: &str = "fill-opacity";
static ATTR_STROKE_WIDTH: &str = "stroke-width";
static ATTR_STROKE: &str = "stroke";
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

fn convert_opacity(color: &Color) -> String {
    if color.is_nontransparent() {
        "".to_string()
    } else {
        format_float(color.opacity())
    }
}
fn format_float(value: f64) -> String {
    let str = format!("{:.1}", value);
    if str.ends_with(".0") {
        return str.substring(0, str.len() - 2).to_string();
    }
    str
}
fn format_option_float(value: Option<f64>) -> String {
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

pub fn generate_svg(width: f64, height: f64, data: String) -> String {
    SVGTag::new(
        TAG_SVG,
        data,
        vec![
            (ATTR_WIDTH, format!("{}", width)),
            (ATTR_HEIGHT, format!("{}", height)),
            (ATTR_VIEW_BOX, format!("0 0 {} {}", width, height)),
            (ATTR_XMLNS, "http://www.w3.org/2000/svg".to_string()),
        ],
    )
    .to_string()
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

impl<'a> fmt::Display for SVGTag<'a> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
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

pub enum Component {
    Line(Line),
    Rect(Rect),
    Polyline(Polyline),
    Circle(Circle),
    Polygon(Polygon),
    Text(Text),
}
#[derive(Clone, PartialEq, Debug, Default)]

pub struct Line {
    pub color: Color,
    pub stroke_width: f64,
    pub left: f64,
    pub top: f64,
    pub right: f64,
    pub bottom: f64,
}

impl Line {
    pub fn svg(&self) -> String {
        let color = &self.color;
        if color.is_transparent() {
            return "".to_string();
        }
        let attrs = vec![
            (ATTR_STROKE_WIDTH, format_float(self.stroke_width)),
            (ATTR_STROKE, color.hex()),
            (ATTR_X1, format_float(self.left)),
            (ATTR_Y1, format_float(self.top)),
            (ATTR_X2, format_float(self.right)),
            (ATTR_Y2, format_float(self.bottom)),
            (ATTR_STROKE_OPACITY, convert_opacity(color)),
        ];
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
    pub left: f64,
    pub top: f64,
    pub width: f64,
    pub height: f64,
    pub rx: Option<f64>,
    pub ry: Option<f64>,
}
impl Rect {
    pub fn svg(&self) -> String {
        if self.color.is_none() && self.fill.is_none() {
            return "".to_string();
        }
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
            attrs.push((ATTR_FILL, color.hex()));
            attrs.push((ATTR_FILL_OPACITY, convert_opacity(&color)));
        }

        SVGTag {
            tag: TAG_RECT,
            attrs,
            data: None,
        }
        .to_string()
    }
}

#[derive(Clone, PartialEq, Debug, Default)]
pub struct Polyline {
    pub color: Color,
    pub stroke_width: f64,
    pub points: Vec<Point>,
}

impl Polyline {
    pub fn svg(&self) -> String {
        if self.color.is_transparent() {
            return "".to_string();
        }
        let points: Vec<String> = self
            .points
            .iter()
            .map(|p| format!("{},{}", format_float(p.x), format_float(p.y)))
            .collect();
        let attrs = vec![
            (ATTR_FILL, "none".to_string()),
            (ATTR_STROKE, self.color.hex()),
            (ATTR_STROKE_WIDTH, format_float(self.stroke_width)),
            (ATTR_POINTS, points.join(" ")),
            (ATTR_STROKE_OPACITY, convert_opacity(&self.color)),
        ];

        SVGTag {
            tag: TAG_POLYLINE,
            attrs,
            data: None,
        }
        .to_string()
    }
}

#[derive(Clone, PartialEq, Debug, Default)]
pub struct Circle {
    pub color: Option<Color>,
    pub fill: Option<Color>,
    pub stroke_width: f64,
    pub cx: f64,
    pub cy: f64,
    pub r: f64,
}

impl Circle {
    pub fn svg(&self) -> String {
        let mut attrs = vec![
            (ATTR_CX, format_float(self.cx)),
            (ATTR_CY, format_float(self.cy)),
            (ATTR_R, format_float(self.r)),
            (ATTR_STROKE_WIDTH, format_float(self.stroke_width)),
        ];
        if let Some(color) = self.color {
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

#[derive(Clone, PartialEq, Debug, Default)]
pub struct Polygon {
    pub color: Option<Color>,
    pub fill: Option<Color>,
    pub points: Vec<Point>,
}

impl Polygon {
    pub fn svg(&self) -> String {
        if self.fill.is_none() && self.color.is_none() {
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
pub struct Text {
    pub text: String,
    pub font_family: String,
    pub font_size: f64,
    pub fill: Option<Color>,
    pub x: Option<f64>,
    pub y: Option<f64>,
    pub dx: Option<f64>,
    pub dy: Option<f64>,
    pub font_weight: Option<String>,
    pub transform: Option<String>,
}

impl Text {
    pub fn svg(&self) -> String {
        if self.text.is_empty() {
            return "".to_string();
        }
        let mut attrs = vec![
            (ATTR_FONT_FAMILY, self.font_family.clone()),
            (ATTR_FONT_SIZE, format_float(self.font_size)),
            (ATTR_X, format_option_float(self.x)),
            (ATTR_Y, format_option_float(self.y)),
            (ATTR_DX, format_option_float(self.dx)),
            (ATTR_DY, format_option_float(self.dy)),
            (
                ATTR_FONT_WEIGHT,
                self.font_weight.clone().unwrap_or_default(),
            ),
            (ATTR_TRANSFORM, self.transform.clone().unwrap_or_default()),
        ];
        if let Some(fill) = self.fill {
            attrs.push((ATTR_FILL, fill.hex()));
            attrs.push((ATTR_OPACITY, convert_opacity(&fill)));
        }

        SVGTag {
            tag: TAG_TEXT,
            attrs,
            data: Some(self.text.clone()),
        }
        .to_string()
    }
}
