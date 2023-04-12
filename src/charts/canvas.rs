use super::component::{generate_svg, Circle, Component, Line, Polygon, Polyline, Rect, Text};
use super::util::*;
use std::cell::RefCell;
use std::rc::Rc;

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
        if let Some(x) = c.x  {
            c.x = Some(x + self.margin.left);
        }
        if let Some(y) = c.y {
            c.y = Some(y + self.margin.top);
        }
        self.append(Component::Text(c))
    }
    pub fn append(&mut self, component: Component) {
        let mut components = self.components.borrow_mut();
        components.push(component);
    }
    pub fn svg(&self) -> String {
        let mut data = vec![];
        for c in self.components.borrow().iter() {
            let value = match c {
                Component::Line(line) => line.svg(),
                Component::Rect(rect) => rect.svg(),
                Component::Polyline(polyline) => polyline.svg(),
                Component::Circle(circle) => circle.svg(),
                Component::Polygon(polygon) => polygon.svg(),
                Component::Text(text) => text.svg(),
            };
            data.push(value);
        }
        generate_svg(self.width, self.height, data.join("\n"))
    }
}

// /// Node's kind.
// #[allow(missing_docs)]
// #[derive(Clone, Debug)]
// pub enum NodeKind {
//     Group(Group),
//     Path(Path),
//     Image(Image),
//     Text(Text),
// }

// fn draw_line<S: BackendStyle>(
//     &mut self,
//     from: BackendCoord,
//     to: BackendCoord,
//     style: &S,
// ) -> Result<(), DrawingErrorKind<Self::ErrorType>> {
//     if style.color().alpha == 0.0 {
//         return Ok(());
//     }
//     self.open_tag(
//         SVGTag::Line,
//         &[
//             ("opacity", &make_svg_opacity(style.color())),
//             ("stroke", &make_svg_color(style.color())),
//             ("stroke-width", &format!("{}", style.stroke_width())),
//             ("x1", &format!("{}", from.0)),
//             ("y1", &format!("{}", from.1)),
//             ("x2", &format!("{}", to.0)),
//             ("y2", &format!("{}", to.1)),
//         ],
//         true,
//     );
//     Ok(())
// }
