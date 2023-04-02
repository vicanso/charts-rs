use std::rc::Rc;
use tiny_skia::{Pixmap, Transform};
use usvg::{
    AspectRatio, Fill, Group, Node, NodeExt, NodeKind, Opacity, Path, PathData, Stroke, Tree,
    TreeWriting, ViewBox, XmlOptions,
};

use super::color::Color;
use super::util::*;

#[derive(Clone)]
pub struct Canvas {
    // TODO 增加
    // 完整chart的style
    tree: Rc<Tree>,
    // margin of the canvas
    margin: Margin,
}

#[derive(Clone, Debug, Default)]
pub struct GridOption {
    pub verticals: usize,
    pub hidden_verticals: Vec<usize>,
    pub horizontals: usize,
    pub hidden_horizontals: Vec<usize>,
}
impl From<(usize, usize)> for GridOption {
    fn from(value: (usize, usize)) -> Self {
        GridOption {
            verticals: value.0,
            horizontals: value.1,
            ..Default::default()
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct AxisOption {
    pub position: Position,
    pub count: usize,
    pub length: f64,
}
impl From<(Position, usize)> for AxisOption {
    fn from(value: (Position, usize)) -> Self {
        AxisOption {
            position: value.0,
            count: value.1,
            length: 3.0,
        }
    }
}

impl Canvas {
    pub fn new(width: f64, height: f64) -> Result<Self> {
        let size = new_size(width, height)?;
        let tree = Rc::new(Tree {
            size,
            view_box: ViewBox {
                rect: size.to_rect(0.0, 0.0),
                aspect: AspectRatio::default(),
            },
            root: Node::new(NodeKind::Group(Group::default())),
        });

        Ok(Canvas {
            tree,
            margin: Margin::default(),
        })
    }
    pub fn new_with_margin(width: f64, height: f64, margin: Margin) -> Result<Self> {
        let mut c = Canvas::new(width, height)?;
        c.margin = margin;
        Ok(c)
    }
    fn append_kind(&self, kind: NodeKind) {
        self.tree.root.append_kind(kind);
    }
    pub fn width(&self) -> f64 {
        self.tree.size.width() - self.margin.left - self.margin.right
    }
    pub fn height(&self) -> f64 {
        self.tree.size.height() - self.margin.top - self.margin.bottom
    }
    pub fn child(&self, margin: Margin) -> Self {
        let tree = Rc::clone(&self.tree);
        let m = self.margin.add(margin);
        Canvas { tree, margin: m }
    }
    pub fn line(&self, points: Vec<Point>, stroke: Stroke) -> Result<()> {
        if stroke.opacity == Opacity::ZERO {
            return Ok(());
        }
        let mut line = PathData::new();
        for (index, point) in points.iter().enumerate() {
            let x = point.x + self.margin.left;
            let y = point.y + self.margin.top;
            if x < 0.0 || x.is_infinite() {
                return Err(Error {
                    message: "x value is invalid".to_string(),
                });
            }
            if y < 0.0 || y.is_infinite() {
                return Err(Error {
                    message: "y value is invalid".to_string(),
                });
            }
            if index == 0 {
                line.push_move_to(x, y);
            } else {
                line.push_line_to(x, y);
            }
        }
        let path = NodeKind::Path(Path {
            data: Rc::new(line),
            stroke: Some(stroke),
            ..Path::default()
        });
        self.append_kind(path);
        Ok(())
    }
    pub fn rect(&self, value: (f64, f64, f64, f64), fill: Color) -> Result<()> {
        if fill.is_transparent() {
            return Ok(());
        }
        let rect = new_rect(
            self.margin.left + value.0,
            self.margin.top + value.1,
            value.2,
            value.3,
        )?;
        self.append_kind(NodeKind::Path(Path {
            fill: Some(fill.into()),
            data: Rc::new(PathData::from_rect(rect)),
            ..Path::default()
        }));
        Ok(())
    }
    pub fn circles(&self, circles: Vec<Circle>, stroke: Stroke, fill: Color) -> Result<()> {
        for item in circles.iter() {
            let path = new_circle_path(
                self.margin.left + item.cx,
                self.margin.top + item.cy,
                item.r,
            );
            self.append_kind(NodeKind::Path(Path {
                fill: Some(fill.into()),
                stroke: Some(stroke.clone()),
                data: Rc::new(path),
                ..Path::default()
            }));
        }
        Ok(())
    }
    pub fn grid(&self, opt: GridOption, color: Color) -> Result<()> {
        if color.is_transparent() {
            return Ok(());
        }
        let width = self.width();
        let height = self.height();

        let stroke = new_stroke(1.0, color);
        // 垂直线
        if opt.verticals != 0 {
            let unit = width / ((opt.verticals - 1) as f64);
            let mut x = 0.0;
            for i in 0..opt.verticals {
                let points = vec![(x, 0.0).into(), (x, height).into()];
                x += unit;
                if opt.hidden_verticals.contains(&i) {
                    continue;
                }
                self.line(points, stroke.clone())?;
            }
        }
        // 水平线
        if opt.horizontals != 0 {
            let unit = height / ((opt.horizontals - 1) as f64);
            let mut y = 0.0;
            for i in 0..opt.horizontals {
                let points = vec![(0.0, y).into(), (width, y).into()];
                y += unit;
                if opt.hidden_horizontals.contains(&i) {
                    continue;
                }
                self.line(points, stroke.clone())?;
            }
        }
        Ok(())
    }
    pub fn axis(&self, opt: AxisOption, color: Color) -> Result<()> {
        if opt.count == 0 {
            return Err(Error {
                message: "axis count should be > 0".to_string(),
            });
        }
        if color.is_transparent() {
            return Ok(());
        }
        let stroke = new_stroke(1.0, color);
        let width = self.width();
        let height = self.height();
        let count = (opt.count - 1) as f64;
        let unit_width = width / count;
        let unit_height = height / count;
        let mut points_list: Vec<Vec<Point>> = vec![];
        let length = opt.length;
        // line的时候会计算margin
        // 因此此处无直接使用0
        let mut x = 0.0;
        let mut y = 0.0;
        match opt.position {
            Position::Left => {
                // 刻度值
                for _ in 0..opt.count {
                    points_list.push(vec![(x, y).into(), (x + length, y).into()]);
                    y += unit_height;
                }
                points_list.push(vec![(length, 0.0).into(), (length, height).into()]);
            }
            Position::Right => {
                x = width - length;

                // 刻度值
                for _ in 0..=opt.count {
                    points_list.push(vec![(x, y).into(), (x + length, y).into()]);
                    y += unit_height;
                }
                points_list.push(vec![(x, 0.0).into(), (x, height).into()]);
            }
            Position::Top => {
                // 刻度值
                for _ in 0..opt.count {
                    points_list.push(vec![(x, y).into(), (x, y + length).into()]);
                    x += unit_width;
                }
                points_list.push(vec![(0.0, length).into(), (width, length).into()]);
            }
            _ => {
                y = height - length;
                // 刻度值
                for _ in 0..opt.count {
                    points_list.push(vec![(x, y).into(), (x, y + length).into()]);
                    x += unit_width;
                }
                points_list.push(vec![
                    (0.0, height - length).into(),
                    (width, height - length).into(),
                ]);
            }
        }
        for points in points_list.iter() {
            self.line(points.to_owned(), stroke.clone())?;
        }
        Ok(())
    }
    pub fn legend_dot_line(&self, color: Color) -> Result<()> {
        let width = 28.0;
        let height = 4.0;
        self.rect((0.0, 0.0, width, height).into(), color.into())?;

        let stroke = new_stroke(1.0, color);
        self.circles(
            vec![(width / 2.0, height / 2.0, 5.0).into()],
            stroke,
            color.into(),
        )?;
        Ok(())
    }
    pub fn legend_rect(&self, color: Color) -> Result<()> {
        let width = 28.0;
        let height = 5.0;
        self.rect((0.0, 0.0, width, height).into(), color.into())?;
        Ok(())
    }
    pub fn to_svg(&self, background: Option<Color>) -> String {
        let mut svg = self.tree.to_string(&XmlOptions::default());
        if let Some(background_color) = background {
            let fill = background_color.string();
            let rect = format!(r#"    <rect width="100%" height="100%" fill="{fill}" />"#);
            let mut arr: Vec<&str> = svg.split('\n').collect();
            arr.insert(1, &rect);
            svg = arr.join("\n");
        }

        svg
    }
    pub fn to_png(&self, background: Option<Color>) -> Result<Vec<u8>> {
        let size = self.tree.size.to_screen_size();
        let map = Pixmap::new(size.width(), size.height());
        if map.is_none() {
            return Err(Error {
                message: "new pixmap fail".to_string(),
            });
        }
        // 已保证不会为空
        let mut pixmap = map.unwrap();
        if let Some(background_color) = background {
            pixmap.fill(tiny_skia::Color::from_rgba8(
                background_color.r,
                background_color.g,
                background_color.b,
                background_color.a,
            ));
        }

        // 如果render失败
        if resvg::render(
            &self.tree,
            resvg::FitTo::Original,
            Transform::default(),
            pixmap.as_mut(),
        )
        .is_none()
        {
            return Err(Error {
                message: "render pixmap fail".to_string(),
            });
        }
        let buf = pixmap.encode_png()?;
        Ok(buf)
    }
}
