use std::rc::Rc;
use tiny_skia::{Pixmap, Transform};
use usvg::{
    AspectRatio, Fill, Group, Node, NodeExt, NodeKind, Path, PathData, Rect, Size, Stroke, Tree,
    TreeWriting, ViewBox, XmlOptions,
};

use super::color::Color;
use super::util::*;

#[derive(Clone)]
pub struct Canvas {
    // TODO 增加
    // 完整chart的style
    tree: Rc<Tree>,
    // margin
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
            hidden_verticals: vec![0],
            horizontals: value.1,
            hidden_horizontals: vec![0],
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
    pub fn rect(&self, value: (f64, f64, f64, f64), fill: Fill) -> Result<()> {
        let rect = new_rect(
            self.margin.left + value.0,
            self.margin.top + value.1,
            value.2,
            value.3,
        )?;
        self.append_kind(NodeKind::Path(Path {
            fill: Some(fill),
            data: Rc::new(PathData::from_rect(rect)),
            ..Path::default()
        }));
        Ok(())
    }
    pub fn circles(&self, circles: Vec<Circle>, stroke: Stroke, fill: Fill) -> Result<()> {
        for item in circles.iter() {
            let path = new_circle_path(
                self.margin.left + item.cx,
                self.margin.top + item.cy,
                item.r,
            );
            self.append_kind(NodeKind::Path(Path {
                fill: Some(fill.clone()),
                stroke: Some(stroke.clone()),
                data: Rc::new(path),
                ..Path::default()
            }));
        }
        Ok(())
    }
    pub fn grid(&self, opt: GridOption, color: Color) -> Result<()> {
        // 垂直线
        let width = self.width();
        let height = self.height();

        let stroke = new_stroke(1.0, color);
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
        if opt.horizontals != 0 {
            let unit = height / ((opt.horizontals - 1) as f64);
            let mut y = 0.0;
            for i in 0..opt.horizontals {
                let points = vec![
                    (0.0, y).into(),
                    (width, y).into(),
                ];
                y += unit;
                if opt.hidden_horizontals.contains(&i) {
                    continue;
                }
                self.line(points, stroke.clone())?;
            }
        }
        Ok(())
    }
    pub fn to_svg(&self) -> String {
        self.tree.to_string(&XmlOptions::default())
    }
    pub fn to_png(&self) -> Result<Vec<u8>> {
        let size = self.tree.size.to_screen_size();
        let map = Pixmap::new(size.width(), size.height());
        if map.is_none() {
            return Err(Error {
                message: "new pixmap fail".to_string(),
            });
        }
        // 已保证不会为空
        let mut pixmap = map.unwrap();

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
