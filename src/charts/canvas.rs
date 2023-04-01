use std::rc::Rc;
use tiny_skia::{Pixmap, Transform};
use usvg::{
    AspectRatio, Fill, Group, Node, NodeExt, NodeKind, Path, PathData, Rect, Size, Stroke, Tree,
    TreeWriting, ViewBox, XmlOptions,
};

use super::util::*;

#[derive(Clone, Debug, Default)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

impl From<(f64, f64)> for Point {
    fn from(value: (f64, f64)) -> Self {
        Point {
            x: value.0,
            y: value.1,
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct Circle {
    pub cx: f64,
    pub cy: f64,
    pub r: f64,
}
impl From<(f64, f64, f64)> for Circle {
    fn from(value: (f64, f64, f64)) -> Self {
        Circle {
            cx: value.0,
            cy: value.1,
            r: value.2,
        }
    }
}

#[derive(Clone)]
pub struct Canvas {
    // TODO 增加
    // 完整chart的style
    tree: Rc<Tree>,

    // margin
    margin: Point,
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
            margin: Point { x: 0.0, y: 0.0 },
        })
    }
    fn append_kind(&self, kind: NodeKind) {
        self.tree.root.append_kind(kind);
    }
    pub fn child(&self, margin: Point) -> Self {
        let tree = Rc::clone(&self.tree);
        let mut m = margin.clone();
        m.x += self.margin.x;
        m.y += self.margin.y;

        Canvas { tree, margin: m }
    }
    pub fn line(&self, points: Vec<Point>, stroke: Stroke) -> Result<()> {
        let mut line = PathData::new();
        for (index, point) in points.iter().enumerate() {
            let x = point.x + self.margin.x;
            let y = point.y + self.margin.y;
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
        let rect = new_rect(value.0, value.1, value.2, value.3)?;
        self.append_kind(NodeKind::Path(Path {
            fill: Some(fill),
            data: Rc::new(PathData::from_rect(rect)),
            ..Path::default()
        }));
        Ok(())
    }
    pub fn circles(&self, circles: Vec<Circle>, stroke: Stroke, fill: Fill) -> Result<()> {
        for item in circles.iter() {
            let path = new_circle_path(item.cx, item.cy, item.r);
            self.append_kind(NodeKind::Path(Path {
                fill: Some(fill.clone()),
                stroke: Some(stroke.clone()),
                data: Rc::new(path),
                ..Path::default()
            }));
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
