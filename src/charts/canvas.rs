use super::component::{
    generate_svg, Axis, Circle, Component, Grid, Line, Polygon, Polyline, Rect, SmoothLine,
    SmoothLineFill, StraightLine, StraightLineFill, Text,
};

use super::util::*;
use snafu::{ResultExt, Snafu};
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("Error to svg: {source}"))]
    ToSVG { source: super::component::Error },
}

pub type Result<T, E = Error> = std::result::Result<T, E>;

pub struct Canvas {
    pub width: f64,
    pub height: f64,
    pub components: Rc<RefCell<Vec<Component>>>,
    pub margin: Box,
}

impl Canvas {
    pub fn new(width: f64, height: f64) -> Self {
        Canvas {
            width,
            height,
            components: Rc::new(RefCell::new(vec![])),
            margin: Box::default(),
        }
    }
    pub fn width(&self) -> f64 {
        self.width - self.margin.left - self.margin.right
    }
    pub fn height(&self) -> f64 {
        self.height - self.margin.top - self.margin.bottom
    }
    pub fn child(&self, margin: Box) -> Self {
        let mut m = self.margin.clone();
        m.left += margin.left;
        m.top += margin.top;
        m.right += margin.right;
        m.bottom += margin.bottom;
        Canvas {
            width: self.width,
            height: self.height,
            components: Rc::clone(&self.components),
            margin: m,
        }
    }
    pub fn line(&mut self, line: Line) {
        let mut c = line;
        c.left += self.margin.left;
        c.right += self.margin.left;
        c.top += self.margin.top;
        c.bottom += self.margin.bottom;
        self.append(Component::Line(c));
    }
    pub fn rect(&mut self, rect: Rect) {
        let mut c = rect;
        c.left += self.margin.left;
        c.top += self.margin.top;
        self.append(Component::Rect(c))
    }
    pub fn polyline(&mut self, polyline: Polyline) {
        let mut c = polyline;
        for p in c.points.iter_mut() {
            p.x += self.margin.left;
            p.y += self.margin.top
        }

        self.append(Component::Polyline(c))
    }
    pub fn circle(&mut self, circle: Circle) {
        let mut c = circle;
        c.cx += self.margin.left;
        c.cy += self.margin.top;
        self.append(Component::Circle(c))
    }
    pub fn polygon(&mut self, polygon: Polygon) {
        let mut c = polygon;
        for p in c.points.iter_mut() {
            p.x += self.margin.left;
            p.y += self.margin.top
        }
        self.append(Component::Polygon(c))
    }
    pub fn text(&mut self, text: Text) {
        let mut c = text;
        if let Some(x) = c.x {
            c.x = Some(x + self.margin.left);
        }
        if let Some(y) = c.y {
            c.y = Some(y + self.margin.top);
        }
        self.append(Component::Text(c))
    }
    pub fn smooth_line(&mut self, line: SmoothLine) {
        let mut c = line;
        for p in c.points.iter_mut() {
            p.x += self.margin.left;
            p.y += self.margin.top
        }
        self.append(Component::SmoothLine(c))
    }
    pub fn straight_line(&mut self, line: StraightLine) {
        let mut c = line;
        for p in c.points.iter_mut() {
            p.x += self.margin.left;
            p.y += self.margin.top
        }
        self.append(Component::StraightLine(c))
    }
    pub fn smooth_line_fill(&mut self, fill: SmoothLineFill) {
        let mut c = fill;
        for p in c.points.iter_mut() {
            p.x += self.margin.left;
            p.y += self.margin.top
        }
        c.bottom += self.margin.top;
        self.append(Component::SmoothLineFill(c))
    }
    pub fn straight_line_fill(&mut self, fill: StraightLineFill) {
        let mut c = fill;
        for p in c.points.iter_mut() {
            p.x += self.margin.left;
            p.y += self.margin.top
        }
        c.bottom += self.margin.top;
        self.append(Component::StraightLineFill(c))
    }
    pub fn grid(&mut self, grip: Grid) {
        let mut c = grip;
        c.left += self.margin.left;
        c.right += self.margin.left;
        c.top += self.margin.top;
        c.bottom += self.margin.top;
        self.append(Component::Grid(c))
    }
    pub fn axis(&mut self, axis: Axis) {
        let mut c = axis;
        c.left += self.margin.left;
        c.top += self.margin.top;
        self.append(Component::Axis(c))
    }
    pub fn append(&mut self, component: Component) {
        let mut components = self.components.borrow_mut();
        components.push(component);
    }
    pub fn svg(&self) -> Result<String> {
        let mut data = vec![];
        for c in self.components.borrow().iter() {
            let value = match c {
                Component::Line(c) => c.svg(),
                Component::Rect(c) => c.svg(),
                Component::Polyline(c) => c.svg(),
                Component::Circle(c) => c.svg(),
                Component::Polygon(c) => c.svg(),
                Component::Text(c) => c.svg(),
                Component::SmoothLine(c) => c.svg(),
                Component::StraightLine(c) => c.svg(),
                Component::SmoothLineFill(c) => c.svg(),
                Component::StraightLineFill(c) => c.svg(),
                Component::Grid(c) => c.svg(),
                Component::Axis(c) => c.svg().context(ToSVGSnafu)?,
            };
            data.push(value);
        }
        Ok(generate_svg(self.width, self.height, data.join("\n")))
    }
}
