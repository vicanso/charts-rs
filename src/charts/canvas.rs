use super::component::{
    generate_svg, Axis, Circle, Component, Grid, Legend, Line, Polygon, Polyline, Rect, SmoothLine,
    SmoothLineFill, StraightLine, StraightLineFill, Text, LEGEND_MARGIN, LEGEND_WIDTH,
};

use super::{measure_text_width_family, util::*};
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
    pub fn line(&mut self, line: Line) -> Box {
        let mut c = line;
        c.left += self.margin.left;
        c.right += self.margin.left;
        c.top += self.margin.top;
        c.bottom += self.margin.bottom;
        let b = Box {
            left: c.left,
            top: c.top,
            right: c.right,
            bottom: c.bottom,
        };
        self.append(Component::Line(c));
        b
    }
    pub fn rect(&mut self, rect: Rect) -> Box {
        let mut c = rect;
        c.left += self.margin.left;
        c.top += self.margin.top;
        let b = Box {
            left: c.left,
            top: c.top,
            right: c.left + c.width,
            bottom: c.top + c.height,
        };
        self.append(Component::Rect(c));
        b
    }
    pub fn polyline(&mut self, polyline: Polyline) -> Box {
        let mut c = polyline;
        for p in c.points.iter_mut() {
            p.x += self.margin.left;
            p.y += self.margin.top;
        }
        let b = get_box_of_points(&c.points);

        self.append(Component::Polyline(c));
        b
    }
    pub fn circle(&mut self, circle: Circle) -> Box {
        let mut c = circle;
        c.cx += self.margin.left;
        c.cy += self.margin.top;
        let b = Box {
            left: c.cx - c.r,
            top: c.cy - c.r,
            right: c.cx + c.r,
            bottom: c.cy + c.r,
        };
        self.append(Component::Circle(c));
        b
    }
    pub fn polygon(&mut self, polygon: Polygon) -> Box {
        let mut c = polygon;
        for p in c.points.iter_mut() {
            p.x += self.margin.left;
            p.y += self.margin.top
        }
        let b = get_box_of_points(&c.points);
        self.append(Component::Polygon(c));
        b
    }
    pub fn text(&mut self, text: Text) -> Box {
        let font_family = text.font_family.clone().unwrap_or_default();
        let font_size = text.font_size.unwrap_or_default();
        let is_bold = text.font_weight.clone().is_some();
        let mut c = text;

        if let Some(x) = c.x {
            c.x = Some(x + self.margin.left);
        } else {
            c.x = Some(self.margin.left);
        }
        if let Some(y) = c.y {
            c.y = Some(y + self.margin.top);
        } else {
            c.y = Some(self.margin.top);
        }
        let mut b = Box {
            left: c.x.unwrap_or_default(),
            top: c.y.unwrap_or_default(),
            ..Default::default()
        };
        if !font_family.is_empty() && font_size > 0.0 {
            if let Ok(result) = measure_text_width_family(&font_family, font_size, &c.text, is_bold)
            {
                b.right = b.left + result.width();
                b.bottom = b.top + result.height();
            }
        }
        self.append(Component::Text(c));
        b
    }
    pub fn smooth_line(&mut self, line: SmoothLine) -> Box {
        let mut c = line;
        for p in c.points.iter_mut() {
            p.x += self.margin.left;
            p.y += self.margin.top
        }
        let b = get_box_of_points(&c.points);
        self.append(Component::SmoothLine(c));
        b
    }
    pub fn straight_line(&mut self, line: StraightLine) -> Box {
        let mut c = line;
        for p in c.points.iter_mut() {
            p.x += self.margin.left;
            p.y += self.margin.top
        }
        let b = get_box_of_points(&c.points);
        self.append(Component::StraightLine(c));
        b
    }
    pub fn smooth_line_fill(&mut self, fill: SmoothLineFill) -> Box {
        let mut c = fill;
        for p in c.points.iter_mut() {
            p.x += self.margin.left;
            p.y += self.margin.top
        }
        c.bottom += self.margin.top;
        let b = get_box_of_points(&c.points);
        self.append(Component::SmoothLineFill(c));
        b
    }
    pub fn straight_line_fill(&mut self, fill: StraightLineFill) -> Box {
        let mut c = fill;
        for p in c.points.iter_mut() {
            p.x += self.margin.left;
            p.y += self.margin.top
        }
        c.bottom += self.margin.top;
        let b = get_box_of_points(&c.points);
        self.append(Component::StraightLineFill(c));
        b
    }
    pub fn grid(&mut self, grip: Grid) -> Box {
        let mut c = grip;
        c.left += self.margin.left;
        c.right += self.margin.left;
        c.top += self.margin.top;
        c.bottom += self.margin.top;
        let b = Box {
            left: c.left,
            top: c.top,
            right: c.right,
            bottom: c.bottom,
        };
        self.append(Component::Grid(c));
        b
    }
    pub fn axis(&mut self, axis: Axis) -> Box {
        let mut c = axis;
        c.left += self.margin.left;
        c.top += self.margin.top;
        let b = Box {
            left: c.left,
            top: c.top,
            right: c.left + c.width,
            bottom: c.top + c.height,
        };
        self.append(Component::Axis(c));
        b
    }
    pub fn legend(&mut self, legend: Legend) -> Box {
        let mut c = legend;
        c.left += self.margin.left;
        c.top += self.margin.top;
        let measurement = measure_text_width_family(&c.font_family, c.font_size, &c.text, false)
            .unwrap_or_default();
        let b = Box {
            left: c.left,
            top: c.top,
            right: c.left + measurement.width() + LEGEND_WIDTH + LEGEND_MARGIN,
            bottom: c.top + measurement.height(),
        };
        self.append(Component::Legend(c));
        b
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
                Component::Legend(c) => c.svg(),
            };
            data.push(value);
        }
        Ok(generate_svg(self.width, self.height, data.join("\n")))
    }
}
