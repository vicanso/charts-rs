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
static ATTR_D: &str = "d";

fn convert_opacity(color: &Color) -> String {
    if color.is_nontransparent() {
        "".to_string()
    } else {
        format_float(color.opacity())
    }
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

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("Error get font: {source}"))]
    GetFont { source: font::Error },
}

pub type Result<T, E = Error> = std::result::Result<T, E>;

pub enum Component {
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
}
#[derive(Clone, PartialEq, Debug)]

pub struct Line {
    pub color: Option<Color>,
    pub stroke_width: f64,
    pub left: f64,
    pub top: f64,
    pub right: f64,
    pub bottom: f64,
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

#[derive(Clone, PartialEq, Debug)]
pub struct Polyline {
    pub color: Option<Color>,
    pub stroke_width: f64,
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
    pub stroke_width: f64,
    pub cx: f64,
    pub cy: f64,
    pub r: f64,
}

impl Default for Circle {
    fn default() -> Self {
        Circle {
            stroke_color: None,
            fill: None,
            stroke_width: 1.0,
            cx: 0.0,
            cy: 0.0,
            r: 0.0,
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
pub struct Text {
    pub text: String,
    pub font_family: Option<String>,
    pub font_size: Option<f64>,
    pub font_color: Option<Color>,
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

struct BaseLine {
    pub color: Option<Color>,
    pub points: Vec<Point>,
    pub stroke_width: f64,
    pub symbol: Option<Symbol>,
    pub is_smooth: bool,
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
            arr.join(" ")
        };

        let mut attrs = vec![
            (ATTR_FILL, "none".to_string()),
            (ATTR_D, path),
            (ATTR_STROKE_WIDTH, format_float(self.stroke_width)),
        ];
        if let Some(color) = self.color {
            attrs.push((ATTR_STROKE, color.hex()));
            attrs.push((ATTR_STROKE_OPACITY, convert_opacity(&color)));
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
            }
        } else {
            "".to_string()
        };

        if symbol_svg.is_empty() {
            line_svg
        } else {
            SVGTag {
                tag: TAG_GROUP,
                data: Some(vec![line_svg, symbol_svg].join("\n")),
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
    pub stroke_width: f64,
    pub symbol: Option<Symbol>,
}

impl Default for SmoothLine {
    fn default() -> Self {
        SmoothLine {
            color: None,
            points: vec![],
            stroke_width: 1.0,
            symbol: Some(Symbol::Circle(2.0, None)),
        }
    }
}

impl SmoothLine {
    pub fn svg(&self) -> String {
        BaseLine {
            color: self.color,
            points: self.points.clone(),
            stroke_width: self.stroke_width,
            symbol: self.symbol.clone(),
            is_smooth: true,
        }
        .svg()
    }
}

#[derive(Clone, PartialEq, Debug)]
pub struct SmoothLineFill {
    pub fill: Color,
    pub points: Vec<Point>,
    pub bottom: f64,
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
        let fill_path = vec![
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
    pub points: Vec<Point>,
    pub stroke_width: f64,
    pub symbol: Option<Symbol>,
}

impl Default for StraightLine {
    fn default() -> Self {
        StraightLine {
            color: None,
            points: vec![],
            stroke_width: 1.0,
            symbol: Some(Symbol::Circle(2.0, None)),
        }
    }
}

impl StraightLine {
    pub fn svg(&self) -> String {
        BaseLine {
            color: self.color,
            points: self.points.clone(),
            stroke_width: self.stroke_width,
            symbol: self.symbol.clone(),
            is_smooth: false,
        }
        .svg()
    }
}

#[derive(Clone, PartialEq, Debug, Default)]
pub struct StraightLineFill {
    pub fill: Color,
    pub points: Vec<Point>,
    pub bottom: f64,
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
    pub left: f64,
    pub top: f64,
    pub right: f64,
    pub bottom: f64,
    pub color: Option<Color>,
    pub stroke_width: f64,
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
            let unit = (self.right - self.left) / (self.verticals) as f64;
            for index in 0..=self.verticals {
                if self.hidden_verticals.contains(&index) {
                    continue;
                }
                let x = self.left + unit * index as f64;
                points.push((x, self.top, x, self.bottom));
            }
        }
        if self.horizontals != 0 {
            let unit = (self.bottom - self.top) / (self.horizontals) as f64;
            for index in 0..=self.horizontals {
                if self.hidden_horizontals.contains(&index) {
                    continue;
                }
                let y = self.top + unit * index as f64;
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
    pub font_size: f64,
    pub font_family: String,
    pub font_color: Option<Color>,
    pub data: Vec<String>,
    pub name_gap: f64,
    pub name_align: Align,
    pub name_rotate: f64,
    pub stroke_color: Option<Color>,
    pub left: f64,
    pub top: f64,
    pub width: f64,
    pub height: f64,
    pub tick_length: f64,
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
            font_color: None,
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
                Position::Left => {
                    let x = left + width;
                    (x, top, x, top + height)
                }
                Position::Top => {
                    let y = top + height;
                    (left, y, left + width, y)
                }
                Position::Right => {
                    let y = top + height;
                    (left, top, left, y)
                }
                Position::Bottom => (left, top, left + width, top),
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

        if !is_transparent {
            let unit = axis_length / self.split_number as f64;
            let tick_interval = self.tick_interval;
            let tick_start = self.tick_start;
            for i in 0..=self.split_number {
                if i < tick_start {
                    continue;
                }
                let index = if i > tick_start { i - tick_start } else { i };
                if i != tick_start && (tick_interval != 0 && index % tick_interval != 0) {
                    continue;
                }

                let values = match self.position {
                    Position::Left => {
                        let y = top + unit * i as f64;
                        let x = left + width;
                        (x, y, x - tick_length, y)
                    }
                    Position::Top => {
                        let x = left + unit * i as f64;
                        let y = top + height;
                        (x, y - tick_length, x, y)
                    }
                    Position::Right => {
                        let y = top + unit * i as f64;
                        (left, y, left + tick_length, y)
                    }
                    Position::Bottom => {
                        let x = left + unit * i as f64;
                        (x, top, x, top + tick_length)
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
        let font_size = self.font_size;
        let name_rotate = self.name_rotate / std::f64::consts::FRAC_PI_2 * 180.0;
        if font_size > 0.0 && !self.data.is_empty() {
            let name_gap = self.name_gap;
            let f = font::get_font(&self.font_family).context(GetFontSnafu)?;
            let mut data_len = self.data.len();
            let is_name_align_start = self.name_align == Align::Left;
            if is_name_align_start {
                data_len -= 1;
            }
            let unit = axis_length / data_len as f64;
            for (index, text) in self.data.iter().enumerate() {
                let b = font::measure_text(&f, font_size, text);
                let mut unit_offset = unit * index as f64 + unit / 2.0;
                if is_name_align_start {
                    unit_offset -= unit / 2.0;
                }
                let text_width = b.width();

                let values = match self.position {
                    Position::Left => {
                        let x = left + width - text_width - name_gap;
                        let y = top + unit_offset + font_size / 2.0;
                        (x, y)
                    }
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
                };
                let mut transform = None;
                if name_rotate != 0.0 {
                    let x = (values.0 + b.width() / 2.0) as i32;
                    let y = (values.1 - b.height()) as i32;
                    let a = name_rotate as i32;
                    transform = Some(format!("rotate({a},{x},{y})"));
                }

                text_data.push(
                    Text {
                        text: text.to_string(),
                        font_family: Some(self.font_family.clone()),
                        font_size: Some(self.font_size),
                        font_color: self.font_color,
                        x: Some(values.0),
                        y: Some(values.1),
                        transform,
                        ..Default::default()
                    }
                    .svg(),
                );
            }
        };
        Ok(SVGTag {
            tag: TAG_GROUP,
            data: Some(
                vec![
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

pub(crate) static LEGEND_WIDTH: f64 = 25.0;
pub(crate) static LEGEND_HEIGHT: f64 = 20.0;
pub(crate) static LEGEND_TEXT_MARGIN: f64 = 3.0;
pub(crate) static LEGEND_MARGIN: f64 = 8.0;

pub(crate) fn measure_legends(
    font_family: &str,
    font_size: f64,
    legends: &[&str],
) -> (f64, Vec<f64>) {
    let widths: Vec<f64> = legends
        .iter()
        .map(|item| {
            let text_box = measure_text_width_family(font_family, font_size, item.to_owned())
                .unwrap_or_default();
            text_box.width() + LEGEND_WIDTH + LEGEND_TEXT_MARGIN
        })
        .collect();
    let width: f64 = widths.iter().sum();
    let margin = LEGEND_MARGIN * (legends.len() - 1) as f64;

    (width + margin, widths)
}

#[derive(Clone, PartialEq, Debug)]
pub struct Legend {
    pub text: String,
    pub font_size: f64,
    pub font_family: String,
    pub font_color: Option<Color>,
    pub stroke_color: Option<Color>,
    pub fill: Option<Color>,
    pub left: f64,
    pub top: f64,
}
impl Legend {
    pub fn svg(&self) -> String {
        let stroke_width = 2.0;
        let line_svg = Line {
            stroke_width,
            color: self.stroke_color,
            left: self.left,
            top: self.top + LEGEND_HEIGHT / 2.0,
            right: self.left + LEGEND_WIDTH,
            bottom: self.top + LEGEND_HEIGHT / 2.0,
            ..Default::default()
        }
        .svg();
        let circle_svg = Circle {
            stroke_width,
            stroke_color: self.stroke_color,
            fill: self.fill,
            cx: self.left + LEGEND_WIDTH / 2.0,
            cy: self.top + LEGEND_HEIGHT / 2.0,
            r: 5.5,
            ..Default::default()
        }
        .svg();
        let text_svg = Text {
            text: self.text.clone(),
            font_family: Some(self.font_family.clone()),
            font_color: self.font_color,
            font_size: Some(self.font_size),
            x: Some(self.left + LEGEND_WIDTH + LEGEND_TEXT_MARGIN),
            y: Some(self.top + self.font_size),
            font_weight: Some("bold".to_string()),
            ..Default::default()
        }
        .svg();
        SVGTag {
            tag: TAG_GROUP,
            data: Some(vec![line_svg, circle_svg, text_svg].join("\n")),
            ..Default::default()
        }
        .to_string()
    }
}
