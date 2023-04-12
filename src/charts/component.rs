use std::fmt;

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

#[derive(Clone, PartialEq, Debug, Default)]
struct SVGTag {
    tag: String,
    attrs: Vec<(String, String)>,
    data: Option<String>,
}

pub fn generate_svg(width: f64, height: f64, data: String) -> String {
    SVGTag::new(
        TAG_SVG.to_string(),
        data,
        vec![
            (ATTR_WIDTH.to_string(), format!("{}", width)),
            (ATTR_HEIGHT.to_string(), format!("{}", height)),
            (
                ATTR_VIEW_BOX.to_string(),
                format!("0 0 {} {}", width, height),
            ),
            (
                ATTR_XMLNS.to_string(),
                "http://www.w3.org/2000/svg".to_string(),
            ),
        ],
    )
    .to_string()
}

impl SVGTag {
    pub fn new(tag: String, data: String, attrs: Vec<(String, String)>) -> Self {
        Self {
            tag,
            attrs,
            data: Some(data),
        }
    }
}

impl fmt::Display for SVGTag {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut value = "<".to_string();
        value.push_str(&self.tag);
        for (k, v) in self.attrs.iter() {
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
        let mut attrs = vec![
            (
                ATTR_STROKE_WIDTH.to_string(),
                format_float(self.stroke_width),
            ),
            (ATTR_STROKE.to_string(), color.hex()),
            (ATTR_X1.to_string(), format_float(self.left)),
            (ATTR_Y1.to_string(), format_float(self.top)),
            (ATTR_X2.to_string(), format_float(self.right)),
            (ATTR_Y2.to_string(), format_float(self.bottom)),
        ];
        if !color.is_nontransparent() {
            attrs.push((
                ATTR_STROKE_OPACITY.to_string(),
                format_float(color.opacity()),
            ));
        }
        SVGTag {
            tag: TAG_LINE.to_string(),
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
            (ATTR_X.to_string(), format_float(self.left)),
            (ATTR_Y.to_string(), format_float(self.top)),
            (ATTR_WIDTH.to_string(), format_float(self.width)),
            (ATTR_HEIGHT.to_string(), format_float(self.height)),
        ];

        if let Some(color) = self.color {
            attrs.push((ATTR_STROKE.to_string(), color.hex()));
            if !color.is_nontransparent() {
                attrs.push((
                    ATTR_STROKE_OPACITY.to_string(),
                    format_float(color.opacity()),
                ))
            }
        }
        if let Some(color) = self.fill {
            attrs.push((ATTR_FILL.to_string(), color.hex()));
            if !color.is_nontransparent() {
                attrs.push((ATTR_FILL_OPACITY.to_string(), format_float(color.opacity())))
            }
        }

        if let Some(rx) = self.rx {
            attrs.push((ATTR_RX.to_string(), format_float(rx)));
        }
        if let Some(ry) = self.ry {
            attrs.push((ATTR_RY.to_string(), format_float(ry)));
        }
        SVGTag {
            tag: TAG_RECT.to_string(),
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
        let mut attrs = vec![
            (ATTR_FILL.to_string(), "none".to_string()),
            (ATTR_STROKE.to_string(), self.color.hex()),
            (
                ATTR_STROKE_WIDTH.to_string(),
                format_float(self.stroke_width),
            ),
            (ATTR_POINTS.to_string(), points.join(" ")),
        ];
        if !self.color.is_nontransparent() {
            attrs.push((
                ATTR_STROKE_OPACITY.to_string(),
                format_float(self.color.opacity()),
            ));
        }

        SVGTag {
            tag: TAG_POLYLINE.to_string(),
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
            (ATTR_CX.to_string(), format_float(self.cx)),
            (ATTR_CY.to_string(), format_float(self.cy)),
            (ATTR_R.to_string(), format_float(self.r)),
            (
                ATTR_STROKE_WIDTH.to_string(),
                format_float(self.stroke_width),
            ),
        ];
        if let Some(color) = self.color {
            attrs.push((ATTR_STROKE.to_string(), color.hex()));
            if !color.is_nontransparent() {
                attrs.push((
                    ATTR_STROKE_OPACITY.to_string(),
                    format_float(color.opacity()),
                ));
            }
        }
        let mut fill = "none".to_string();
        if let Some(color) = self.fill {
            fill = color.hex();
            if !color.is_nontransparent() {
                attrs.push((ATTR_FILL_OPACITY.to_string(), format_float(color.opacity())));
            }
        }
        attrs.push((ATTR_FILL.to_string(), fill));

        SVGTag {
            tag: TAG_CIRCLE.to_string(),
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
        let mut attrs = vec![(ATTR_POINTS.to_string(), points.join(" "))];
        if let Some(color) = self.color {
            attrs.push((ATTR_STROKE.to_string(), color.hex()));
            if !color.is_nontransparent() {
                attrs.push((
                    ATTR_STROKE_OPACITY.to_string(),
                    format_float(color.opacity()),
                ));
            }
        }
        if let Some(color) = self.fill {
            attrs.push((ATTR_FILL.to_string(), color.hex()));
            if !color.is_nontransparent() {
                attrs.push((ATTR_FILL_OPACITY.to_string(), format_float(color.opacity())));
            }
        }
        SVGTag {
            tag: TAG_POLYGON.to_string(),
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
            (ATTR_FONT_FAMILY.to_string(), self.font_family.clone()),
            (ATTR_FONT_SIZE.to_string(), format_float(self.font_size)),
        ];
        if let Some(value) = self.x {
            attrs.push((ATTR_X.to_string(), format_float(value)));
        }
        if let Some(value) = self.y {
            attrs.push((ATTR_Y.to_string(), format_float(value)));
        }
        if let Some(value) = self.dx {
            attrs.push((ATTR_DX.to_string(), format_float(value)));
        }
        if let Some(value) = self.dy {
            attrs.push((ATTR_DY.to_string(), format_float(value)));
        }
        if let Some(ref value) = self.font_weight {
            attrs.push((ATTR_FONT_WEIGHT.to_string(), value.clone()));
        }
        if let Some(ref value) = self.transform {
            attrs.push((ATTR_TRANSFORM.to_string(), value.clone()));
        }
        if let Some(fill) = self.fill {
            attrs.push((ATTR_FILL.to_string(), fill.hex()));
            if !fill.is_nontransparent() {
                attrs.push((ATTR_OPACITY.to_string(), format_float(fill.opacity())))
            }
        }

        SVGTag {
            tag: TAG_TEXT.to_string(),
            attrs,
            data: Some(self.text.clone()),
        }
        .to_string()
    }
}
