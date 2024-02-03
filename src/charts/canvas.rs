use super::component::{
    generate_svg, Arrow, Axis, Bubble, Circle, Component, Grid, Legend, Line, Pie, Polygon,
    Polyline, Rect, SmoothLine, SmoothLineFill, StraightLine, StraightLineFill, Text, LEGEND_WIDTH,
};

use super::{measure_text_width_family, util::*};
use snafu::{ResultExt, Snafu};
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("Error to svg: {source}"))]
    ToSVG { source: super::component::Error },
    #[snafu(display("Params is invalid: {message}"))]
    Params { message: String },
    #[snafu(display("Json is invalid: {source}"))]
    Json { source: serde_json::Error },
    #[snafu(display("Font is invalid: {source}"))]
    Font { source: super::FontError },
}

impl From<serde_json::Error> for Error {
    fn from(value: serde_json::Error) -> Self {
        Error::Json { source: value }
    }
}

impl From<super::FontError> for Error {
    fn from(value: super::FontError) -> Self {
        Error::Font { source: value }
    }
}

pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Clone)]
pub struct Canvas {
    pub width: f32,
    pub height: f32,
    pub x: f32,
    pub y: f32,
    pub components: Rc<RefCell<Vec<Component>>>,
    pub margin: Box,
}

impl Canvas {
    pub fn new(width: f32, height: f32) -> Self {
        Canvas::new_width_xy(width, height, 0.0, 0.0)
    }
    pub fn new_width_xy(width: f32, height: f32, x: f32, y: f32) -> Self {
        Canvas {
            width,
            height,
            x,
            y,
            components: Rc::new(RefCell::new(vec![])),
            margin: Box::default(),
        }
    }
    pub fn width(&self) -> f32 {
        self.width - self.margin.left - self.margin.right
    }
    pub fn height(&self) -> f32 {
        self.height - self.margin.top - self.margin.bottom
    }
    pub fn child_left_top(&self, margin: Box) -> Self {
        let mut m = margin;
        m.left += self.margin.left;
        m.top += self.margin.top;
        Canvas {
            width: self.width,
            height: self.height,
            components: Rc::clone(&self.components),
            margin: m,
            x: self.x,
            y: self.y,
        }
    }
    pub fn child(&self, margin: Box) -> Self {
        let mut m = margin;
        m.left += self.margin.left;
        m.top += self.margin.top;
        m.right += self.margin.right;
        m.bottom += self.margin.bottom;
        Canvas {
            width: self.width,
            height: self.height,
            components: Rc::clone(&self.components),
            margin: m,
            x: self.x,
            y: self.y,
        }
    }
    pub fn arrow(&mut self, arrow: Arrow) -> Box {
        let mut c = arrow;
        c.x += self.margin.left;
        c.y += self.margin.top;
        self.append(Component::Arrow(c));
        let mut b = self.margin.clone();
        b.right = b.left + 10.0;
        b.bottom = b.top;
        b
    }
    pub fn line(&mut self, line: Line) -> Box {
        let mut c = line;
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
            if let Ok(result) = measure_text_width_family(&font_family, font_size, &c.text) {
                b.right = b.left + result.width();
                b.bottom = b.top + result.height();
            }
            let line_height = c.line_height.unwrap_or_default();
            // 设置了行高
            if line_height > font_size {
                c.dy = Some(c.dy.unwrap_or_default() + line_height / 2.0);
                c.dominant_baseline = Some("middle".to_string());
                b.bottom = b.top + line_height;
            }
        }

        self.append(Component::Text(c));
        b
    }
    pub fn pie(&mut self, pie: Pie) -> Box {
        let mut c = pie;
        c.cx += self.margin.left;
        c.cy += self.margin.top;
        let b = Box {
            left: self.margin.left,
            top: self.margin.top,
            right: self.margin.left + c.r * 2.0,
            bottom: self.margin.top + c.r * 2.0,
        };

        self.append(Component::Pie(c));
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
        let mut b = get_box_of_points(&c.points);
        b.bottom = c.bottom;
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
        let mut b = get_box_of_points(&c.points);
        b.bottom = c.bottom;
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
        let measurement =
            measure_text_width_family(&c.font_family, c.font_size, &c.text).unwrap_or_default();
        let b = Box {
            left: c.left,
            top: c.top,
            right: c.left + measurement.width() + LEGEND_WIDTH,
            bottom: c.top + measurement.height(),
        };
        self.append(Component::Legend(c));
        b
    }
    pub fn bubble(&mut self, bubble: Bubble) -> Box {
        let mut c = bubble;
        c.x += self.margin.left;
        c.y += self.margin.top;
        let b = Box {
            left: c.x - c.r,
            top: c.y - c.r,
            right: c.x + c.r,
            bottom: c.y + c.r,
        };
        self.append(Component::Bubble(c));
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
                Component::Arrow(c) => c.svg(),
                Component::Bubble(c) => c.svg(),
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
                Component::Pie(c) => c.svg(),
            };
            data.push(value);
        }
        Ok(generate_svg(
            self.width,
            self.height,
            self.x,
            self.y,
            data.join("\n"),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::Canvas;
    use crate::{
        convert_to_points, Align, Axis, Grid, Legend, LegendCategory, Line, Polyline, Rect,
        SmoothLine, SmoothLineFill, StraightLine, StraightLineFill, Symbol, Text,
        DEFAULT_FONT_FAMILY,
    };
    use pretty_assertions::assert_eq;
    #[test]
    fn canvas_width_height() {
        let mut c = Canvas::new(400.0, 300.0);

        assert_eq!("(0,0,0,0)", c.margin.to_string());
        assert_eq!(400.0, c.width());
        assert_eq!(300.0, c.height());
        c = c.child((5.0, 10.0, 15.0, 20.0).into());
        assert_eq!("(5,10,15,20)", c.margin.to_string());
        assert_eq!(380.0, c.width());
        assert_eq!(270.0, c.height());
    }
    #[test]
    fn canvas_line() {
        let mut c = Canvas::new(400.0, 300.0);
        let b = c.line(Line {
            color: Some((0, 0, 0).into()),
            left: 5.0,
            top: 5.0,
            right: 50.0,
            bottom: 20.0,
            ..Default::default()
        });
        assert_eq!("(5,5,50,20)", b.to_string());
        assert_eq!(
            r###"<svg width="400" height="300" viewBox="0 0 400 300" xmlns="http://www.w3.org/2000/svg">
<line stroke-width="1" x1="5" y1="5" x2="50" y2="20" stroke="#000000"/>
</svg>"###,
            c.svg().unwrap()
        );
    }
    #[test]
    fn canvas_rect() {
        let mut c = Canvas::new(400.0, 300.0);
        let b = c.rect(Rect {
            color: Some((0, 0, 0).into()),
            fill: Some((0, 255, 0).into()),
            left: 10.0,
            top: 10.0,
            width: 100.0,
            height: 30.0,
            rx: Some(3.0),
            ry: Some(5.0),
        });
        assert_eq!("(10,10,110,40)", b.to_string());
        assert_eq!(
            r###"<svg width="400" height="300" viewBox="0 0 400 300" xmlns="http://www.w3.org/2000/svg">
<rect x="10" y="10" width="100" height="30" rx="3" ry="5" stroke="#000000" fill="#00FF00"/>
</svg>"###,
            c.svg().unwrap()
        );
    }
    #[test]
    fn canvas_polyline() {
        let mut c = Canvas::new(400.0, 300.0);
        let b = c.polyline(Polyline {
            color: Some((0, 0, 0).into()),
            stroke_width: 1.0,
            points: convert_to_points(&vec![(1.0, 5.0), (30.0, 60.0), (50.0, 10.0), (70.0, 40.0)]),
        });
        assert_eq!("(1,5,70,60)", b.to_string());
        assert_eq!(
            r###"<svg width="400" height="300" viewBox="0 0 400 300" xmlns="http://www.w3.org/2000/svg">
<polyline fill="none" stroke-width="1" points="1,5 30,60 50,10 70,40" stroke="#000000"/>
</svg>"###,
            c.svg().unwrap()
        );
    }
    #[test]
    fn canvas_text() {
        let mut c = Canvas::new(400.0, 300.0);
        let b = c.text(Text {
            text: "Hello World!".to_string(),
            font_family: Some(DEFAULT_FONT_FAMILY.to_string()),
            font_size: Some(14.0),
            font_color: Some((0, 0, 0).into()),
            font_weight: Some("bold".to_string()),
            line_height: Some(30.0),
            ..Default::default()
        });
        assert_eq!("(0,0,79,30)", b.to_string());
        assert_eq!(
            r###"<svg width="400" height="300" viewBox="0 0 400 300" xmlns="http://www.w3.org/2000/svg">
<text font-size="14" x="0" y="0" dy="15" font-weight="bold" dominant-baseline="middle" font-family="Roboto" fill="#000000">
Hello World!
</text>
</svg>"###,
            c.svg().unwrap()
        );
    }
    #[test]
    fn canvas_smooth_line() {
        let mut c = Canvas::new(400.0, 300.0);
        let b = c.smooth_line(SmoothLine {
            color: Some((0, 0, 0).into()),
            points: convert_to_points(&vec![
                (10.0, 10.0),
                (30.0, 50.0),
                (50.0, 80.0),
                (70.0, 20.0),
                (90.0, 40.0),
            ]),
            stroke_width: 1.0,
            symbol: Some(Symbol::Circle(3.0, Some((0, 255, 0).into()))),
            ..Default::default()
        });
        assert_eq!("(10,10,90,80)", b.to_string());
        assert_eq!(
            r###"<svg width="400" height="300" viewBox="0 0 400 300" xmlns="http://www.w3.org/2000/svg">
<g>
<path d="M10,10 C15 20, 24.5 40.3, 30 50 C34.5 57.8, 46.4 82.7, 50 80 C56.4 75.2, 63.1 26.9, 70 20 C73.1 16.9, 85 35, 90 40" stroke-width="1" fill="none" stroke="#000000"/>
<circle cx="10" cy="10" r="3" stroke-width="1" stroke="#000000" fill="#00FF00"/>
<circle cx="30" cy="50" r="3" stroke-width="1" stroke="#000000" fill="#00FF00"/>
<circle cx="50" cy="80" r="3" stroke-width="1" stroke="#000000" fill="#00FF00"/>
<circle cx="70" cy="20" r="3" stroke-width="1" stroke="#000000" fill="#00FF00"/>
<circle cx="90" cy="40" r="3" stroke-width="1" stroke="#000000" fill="#00FF00"/>
</g>
</svg>"###,
            c.svg().unwrap()
        );
    }
    #[test]
    fn canvas_straight_line() {
        let mut c = Canvas::new(400.0, 300.0);
        let b = c.straight_line(StraightLine {
            color: Some((0, 0, 0).into()),
            points: convert_to_points(&vec![
                (10.0, 10.0),
                (30.0, 50.0),
                (50.0, 80.0),
                (70.0, 20.0),
                (90.0, 40.0),
            ]),
            stroke_width: 1.0,
            symbol: Some(Symbol::Circle(3.0, Some((0, 255, 0).into()))),
            ..Default::default()
        });
        assert_eq!("(10,10,90,80)", b.to_string());
        assert_eq!(
            r###"<svg width="400" height="300" viewBox="0 0 400 300" xmlns="http://www.w3.org/2000/svg">
<g>
<path d="M 10 10 L 30 50 L 50 80 L 70 20 L 90 40" stroke-width="1" fill="none" stroke="#000000"/>
<circle cx="10" cy="10" r="3" stroke-width="1" stroke="#000000" fill="#00FF00"/>
<circle cx="30" cy="50" r="3" stroke-width="1" stroke="#000000" fill="#00FF00"/>
<circle cx="50" cy="80" r="3" stroke-width="1" stroke="#000000" fill="#00FF00"/>
<circle cx="70" cy="20" r="3" stroke-width="1" stroke="#000000" fill="#00FF00"/>
<circle cx="90" cy="40" r="3" stroke-width="1" stroke="#000000" fill="#00FF00"/>
</g>
</svg>"###,
            c.svg().unwrap()
        );
    }

    #[test]
    fn canvas_smooth_line_fill() {
        let mut c = Canvas::new(400.0, 300.0);
        let b = c.smooth_line_fill(SmoothLineFill {
            fill: (0, 0, 0).into(),
            points: convert_to_points(&vec![
                (10.0, 10.0),
                (30.0, 50.0),
                (50.0, 80.0),
                (70.0, 20.0),
                (90.0, 40.0),
            ]),
            bottom: 150.0,
        });
        assert_eq!("(10,10,90,150)", b.to_string());
        assert_eq!(
            r###"<svg width="400" height="300" viewBox="0 0 400 300" xmlns="http://www.w3.org/2000/svg">
<path d="M10,10 C15 20, 24.5 40.3, 30 50 C34.5 57.8, 46.4 82.7, 50 80 C56.4 75.2, 63.1 26.9, 70 20 C73.1 16.9, 85 35, 90 40M 90 40 L 90 150 L 10 150 L 10 10" fill="#000000"/>
</svg>"###,
            c.svg().unwrap()
        );
    }
    #[test]
    fn canvas_straight_line_fill() {
        let mut c = Canvas::new(400.0, 300.0);
        let b = c.straight_line_fill(StraightLineFill {
            fill: (0, 0, 0).into(),
            points: convert_to_points(&vec![
                (10.0, 10.0),
                (30.0, 50.0),
                (50.0, 80.0),
                (70.0, 20.0),
                (90.0, 40.0),
            ]),
            bottom: 150.0,
            ..Default::default()
        });
        assert_eq!("(10,10,90,150)", b.to_string());
        assert_eq!(
            r###"<svg width="400" height="300" viewBox="0 0 400 300" xmlns="http://www.w3.org/2000/svg">
<path d="M 10 10 L 30 50 L 50 80 L 70 20 L 90 40 L 90 150 L 10 150 L 10 10" fill="#000000"/>
</svg>"###,
            c.svg().unwrap()
        );
    }
    #[test]
    fn canvas_grid() {
        let mut c = Canvas::new(400.0, 300.0);
        let b = c.grid(Grid {
            left: 10.0,
            top: 10.0,
            right: 390.0,
            bottom: 290.0,
            color: Some((0, 0, 0).into()),
            stroke_width: 1.0,
            verticals: 5,
            hidden_verticals: vec![0],
            horizontals: 6,
            hidden_horizontals: vec![6],
        });
        assert_eq!("(10,10,390,290)", b.to_string());
        assert_eq!(
            r###"<svg width="400" height="300" viewBox="0 0 400 300" xmlns="http://www.w3.org/2000/svg">
<g stroke="#000000">
<line stroke-width="1" x1="86" y1="10" x2="86" y2="290"/><line stroke-width="1" x1="162" y1="10" x2="162" y2="290"/><line stroke-width="1" x1="238" y1="10" x2="238" y2="290"/><line stroke-width="1" x1="314" y1="10" x2="314" y2="290"/><line stroke-width="1" x1="390" y1="10" x2="390" y2="290"/><line stroke-width="1" x1="10" y1="10" x2="390" y2="10"/><line stroke-width="1" x1="10" y1="56.7" x2="390" y2="56.7"/><line stroke-width="1" x1="10" y1="103.3" x2="390" y2="103.3"/><line stroke-width="1" x1="10" y1="150" x2="390" y2="150"/><line stroke-width="1" x1="10" y1="196.7" x2="390" y2="196.7"/><line stroke-width="1" x1="10" y1="243.3" x2="390" y2="243.3"/>
</g>
</svg>"###,
            c.svg().unwrap()
        );
    }
    #[test]
    fn canvas_axis() {
        let mut c = Canvas::new(400.0, 300.0);
        let b = c.axis(Axis {
            data: vec![
                "Mon".to_string(),
                "Tue".to_string(),
                "Wed".to_string(),
                "Thu".to_string(),
                "Fri".to_string(),
                "Sat".to_string(),
                "Sun".to_string(),
            ],
            left: 5.0,
            top: 5.0,
            width: 390.0,
            stroke_color: Some((0, 0, 0).into()),
            ..Default::default()
        });
        assert_eq!("(5,5,395,5)", b.to_string());
        assert_eq!(
            r###"<svg width="400" height="300" viewBox="0 0 400 300" xmlns="http://www.w3.org/2000/svg">
<g>
<g stroke="#000000">
<line stroke-width="1" x1="5" y1="5" x2="395" y2="5"/>
<line stroke-width="1" x1="5" y1="5" x2="5" y2="10"/>
<line stroke-width="1" x1="60.7" y1="5" x2="60.7" y2="10"/>
<line stroke-width="1" x1="116.4" y1="5" x2="116.4" y2="10"/>
<line stroke-width="1" x1="172.1" y1="5" x2="172.1" y2="10"/>
<line stroke-width="1" x1="227.9" y1="5" x2="227.9" y2="10"/>
<line stroke-width="1" x1="283.6" y1="5" x2="283.6" y2="10"/>
<line stroke-width="1" x1="339.3" y1="5" x2="339.3" y2="10"/>
<line stroke-width="1" x1="395" y1="5" x2="395" y2="10"/>
</g>
<text font-size="14" x="18.9" y="24" font-family="Roboto">
Mon
</text>
<text font-size="14" x="76.6" y="24" font-family="Roboto">
Tue
</text>
<text font-size="14" x="130.3" y="24" font-family="Roboto">
Wed
</text>
<text font-size="14" x="188" y="24" font-family="Roboto">
Thu
</text>
<text font-size="14" x="247.7" y="24" font-family="Roboto">
Fri
</text>
<text font-size="14" x="300.4" y="24" font-family="Roboto">
Sat
</text>
<text font-size="14" x="355.1" y="24" font-family="Roboto">
Sun
</text>
</g>
</svg>"###,
            c.svg().unwrap()
        );

        // split number
        let mut c = Canvas::new(400.0, 300.0);
        let b = c.axis(Axis {
            data: vec![
                "Mon".to_string(),
                "Tue".to_string(),
                "Wed".to_string(),
                "Thu".to_string(),
                "Fri".to_string(),
                "Sat".to_string(),
            ],
            left: 5.0,
            top: 5.0,
            width: 390.0,
            split_number: 3,
            stroke_color: Some((0, 0, 0).into()),
            ..Default::default()
        });
        assert_eq!("(5,5,395,5)", b.to_string());
        assert_eq!(
            r###"<svg width="400" height="300" viewBox="0 0 400 300" xmlns="http://www.w3.org/2000/svg">
<g>
<g stroke="#000000">
<line stroke-width="1" x1="5" y1="5" x2="395" y2="5"/>
<line stroke-width="1" x1="5" y1="5" x2="5" y2="10"/>
<line stroke-width="1" x1="135" y1="5" x2="135" y2="10"/>
<line stroke-width="1" x1="265" y1="5" x2="265" y2="10"/>
<line stroke-width="1" x1="395" y1="5" x2="395" y2="10"/>
</g>
<text font-size="14" x="23.5" y="24" font-family="Roboto">
Mon
</text>
<text font-size="14" x="90.5" y="24" font-family="Roboto">
Tue
</text>
<text font-size="14" x="153.5" y="24" font-family="Roboto">
Wed
</text>
<text font-size="14" x="220.5" y="24" font-family="Roboto">
Thu
</text>
<text font-size="14" x="289.5" y="24" font-family="Roboto">
Fri
</text>
<text font-size="14" x="351.5" y="24" font-family="Roboto">
Sat
</text>
</g>
</svg>"###,
            c.svg().unwrap()
        );

        // set tick interval
        let mut c = Canvas::new(400.0, 300.0);
        let b = c.axis(Axis {
            data: vec![
                "Mon".to_string(),
                "Tue".to_string(),
                "Wed".to_string(),
                "Thu".to_string(),
                "Fri".to_string(),
                "Sat".to_string(),
                "Sun".to_string(),
            ],
            left: 5.0,
            top: 5.0,
            width: 390.0,
            tick_interval: 2,
            stroke_color: Some((0, 0, 0).into()),
            ..Default::default()
        });
        assert_eq!("(5,5,395,5)", b.to_string());
        assert_eq!(
            r###"<svg width="400" height="300" viewBox="0 0 400 300" xmlns="http://www.w3.org/2000/svg">
<g>
<g stroke="#000000">
<line stroke-width="1" x1="5" y1="5" x2="395" y2="5"/>
<line stroke-width="1" x1="5" y1="5" x2="5" y2="10"/>
<line stroke-width="1" x1="116.4" y1="5" x2="116.4" y2="10"/>
<line stroke-width="1" x1="227.9" y1="5" x2="227.9" y2="10"/>
<line stroke-width="1" x1="339.3" y1="5" x2="339.3" y2="10"/>
</g>
<text font-size="14" x="18.9" y="24" font-family="Roboto">
Mon
</text>
<text font-size="14" x="76.6" y="24" font-family="Roboto">
Tue
</text>
<text font-size="14" x="130.3" y="24" font-family="Roboto">
Wed
</text>
<text font-size="14" x="188" y="24" font-family="Roboto">
Thu
</text>
<text font-size="14" x="247.7" y="24" font-family="Roboto">
Fri
</text>
<text font-size="14" x="300.4" y="24" font-family="Roboto">
Sat
</text>
<text font-size="14" x="355.1" y="24" font-family="Roboto">
Sun
</text>
</g>
</svg>"###,
            c.svg().unwrap()
        );

        // name align left
        let mut c = Canvas::new(400.0, 300.0);
        let b = c.axis(Axis {
            data: vec![
                "Mon".to_string(),
                "Tue".to_string(),
                "Wed".to_string(),
                "Thu".to_string(),
                "Fri".to_string(),
                "Sat".to_string(),
                "Sun".to_string(),
            ],
            left: 20.0,
            top: 5.0,
            width: 360.0,
            stroke_color: Some((0, 0, 0).into()),
            name_align: Align::Left,
            ..Default::default()
        });
        assert_eq!("(20,5,380,5)", b.to_string());
        assert_eq!(
            r###"<svg width="400" height="300" viewBox="0 0 400 300" xmlns="http://www.w3.org/2000/svg">
<g>
<g stroke="#000000">
<line stroke-width="1" x1="20" y1="5" x2="380" y2="5"/>
<line stroke-width="1" x1="20" y1="5" x2="20" y2="10"/>
<line stroke-width="1" x1="71.4" y1="5" x2="71.4" y2="10"/>
<line stroke-width="1" x1="122.9" y1="5" x2="122.9" y2="10"/>
<line stroke-width="1" x1="174.3" y1="5" x2="174.3" y2="10"/>
<line stroke-width="1" x1="225.7" y1="5" x2="225.7" y2="10"/>
<line stroke-width="1" x1="277.1" y1="5" x2="277.1" y2="10"/>
<line stroke-width="1" x1="328.6" y1="5" x2="328.6" y2="10"/>
<line stroke-width="1" x1="380" y1="5" x2="380" y2="10"/>
</g>
<text font-size="14" x="6" y="24" font-family="Roboto">
Mon
</text>
<text font-size="14" x="68" y="24" font-family="Roboto">
Tue
</text>
<text font-size="14" x="126" y="24" font-family="Roboto">
Wed
</text>
<text font-size="14" x="188" y="24" font-family="Roboto">
Thu
</text>
<text font-size="14" x="252" y="24" font-family="Roboto">
Fri
</text>
<text font-size="14" x="309" y="24" font-family="Roboto">
Sat
</text>
<text font-size="14" x="368" y="24" font-family="Roboto">
Sun
</text>
</g>
</svg>"###,
            c.svg().unwrap()
        );
    }

    #[test]
    fn canvas_legend() {
        let mut c = Canvas::new(400.0, 300.0);
        let b = c.legend(Legend {
            text: "Email".to_string(),
            font_size: 14.0,
            font_family: DEFAULT_FONT_FAMILY.to_string(),
            font_color: Some((0, 0, 0).into()),
            stroke_color: Some((0, 0, 0).into()),
            fill: Some((0, 0, 0).into()),
            left: 10.0,
            top: 10.0,
            category: LegendCategory::Normal,
            ..Default::default()
        });
        assert_eq!("(10,10,71,24)", b.to_string());
        assert_eq!(
            r###"<svg width="400" height="300" viewBox="0 0 400 300" xmlns="http://www.w3.org/2000/svg">
<g>
<line stroke-width="2" x1="10" y1="20" x2="35" y2="20" stroke="#000000"/>
<circle cx="22.5" cy="20" r="5.5" stroke-width="2" stroke="#000000" fill="#000000"/>
<text font-size="14" x="38" y="24" font-family="Roboto" fill="#000000">
Email
</text>
</g>
</svg>"###,
            c.svg().unwrap()
        );

        let mut c = Canvas::new(400.0, 300.0);
        let b = c.legend(Legend {
            text: "Email".to_string(),
            font_size: 14.0,
            font_family: DEFAULT_FONT_FAMILY.to_string(),
            font_color: Some((0, 0, 0).into()),
            stroke_color: Some((0, 0, 0).into()),
            fill: Some((0, 0, 0).into()),
            left: 10.0,
            top: 10.0,
            category: LegendCategory::Rect,
            ..Default::default()
        });
        assert_eq!("(10,10,71,24)", b.to_string());
        assert_eq!(
            r###"<svg width="400" height="300" viewBox="0 0 400 300" xmlns="http://www.w3.org/2000/svg">
<g>
<rect x="10" y="15" width="25" height="10" stroke="#000000" fill="#000000"/>
<text font-size="14" x="38" y="24" font-family="Roboto" fill="#000000">
Email
</text>
</g>
</svg>"###,
            c.svg().unwrap()
        );
    }
}
