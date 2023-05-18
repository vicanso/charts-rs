use super::util::*;
use std::fmt;

#[derive(Clone, Copy, PartialEq, Debug, Default)]
pub struct QuadraticBezier {
    pub x1: f32,
    pub y1: f32,
    pub x2: f32,
    pub y2: f32,
}
impl fmt::Display for QuadraticBezier {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let m = format!("{} {}", format_float(self.x1), format_float(self.y1));
        let x = (self.x1 + self.x1) / 2.0;
        let y = self.y1 + (self.y2 - self.y1) / 2.0;
        let q = format!("{} {}", format_float(x), format_float(y));
        let end = format!("{} {}", format_float(self.x2), format_float(self.y2));
        write!(f, "M{} Q{}, {}", m, q, end)
    }
}

#[derive(Clone, Copy, PartialEq, Debug, Default)]
struct ControlPoint {
    left: Option<Point>,
    right: Option<Point>,
}

// http://scaledinnovation.com/analytics/splines/aboutSplines.html
fn get_control_points(
    p: &Point,
    left: Option<&Point>,
    right: Option<&Point>,
    t: f32,
) -> ControlPoint {
    let x0 = left.unwrap_or(p).x;
    let y0 = left.unwrap_or(p).y;
    let x1 = p.x;
    let y1 = p.y;
    let x2 = right.unwrap_or(p).x;
    let y2 = right.unwrap_or(p).y;

    let d01 = ((x1 - x0).powf(2.0) + (y1 - y0).powf(2.0)).sqrt();
    let d12 = ((x2 - x1).powf(2.0) + (y2 - y1).powf(2.0)).sqrt();
    // scaling factor for triangle Ta
    let fa = t * d01 / (d01 + d12);
    // ditto for Tb, simplifies to fb=t-fa
    let fb = t * d12 / (d01 + d12);
    // x2-x0 is the width of triangle T
    let p1x = x1 - fa * (x2 - x0);
    // y2-y0 is the height of T
    let p1y = y1 - fa * (y2 - y0);
    let p2x = x1 + fb * (x2 - x0);
    let p2y = y1 + fb * (y2 - y0);

    let mut cpl = None;
    let mut cpr = None;
    if left.is_some() {
        cpl = Some((p1x, p1y).into());
    }
    if right.is_some() {
        cpr = Some((p2x, p2y).into());
    }
    ControlPoint {
        left: cpl,
        right: cpr,
    }
}

#[derive(Clone, PartialEq, Debug, Default)]
pub struct SmoothCurve {
    pub points: Vec<Point>,
    pub close: bool,
}
impl fmt::Display for SmoothCurve {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let tension = 0.25;

        let close = self.close;
        let count = self.points.len();
        let mut control_points = vec![];
        for (index, point) in self.points.iter().enumerate() {
            let mut left = None;
            let mut right = None;
            if index >= 1 {
                left = Some(&self.points[index - 1]);
            } else if close {
                // 对于第一个点的上一节点则为last
                left = self.points.last();
            }
            if index + 1 < count {
                right = Some(&self.points[index + 1]);
            } else if close {
                // 最后一个点的下一节点则为first
                right = self.points.first()
            }
            control_points.push(get_control_points(point, left, right, tension));
        }

        let mut arr = vec![];
        for (index, point) in self.points.iter().enumerate() {
            if index == 0 {
                arr.push(format!(
                    "M{},{}",
                    format_float(point.x),
                    format_float(point.y)
                ));
            }
            let cp1 = control_points[index].right;
            let mut cp2 = None;
            if let Some(value) = control_points.get(index + 1) {
                cp2 = value.left;
            } else if close {
                // 最的一个点
                cp2 = control_points[0].left;
            }
            let mut next_point = self.points.get(index + 1);
            // 如果是close的才需要处理最后一个点
            // 如果非最后一个点
            if close && index == count - 1 {
                next_point = self.points.first();
            }
            if let Some(next_point_value) = next_point {
                let next_point = format!(
                    "{} {}",
                    format_float(next_point_value.x),
                    format_float(next_point_value.y)
                );
                if let Some(cp1_value) = cp1 {
                    if let Some(cp2_value) = cp2 {
                        let c1 = format!(
                            "{} {}",
                            format_float(cp1_value.x),
                            format_float(cp1_value.y)
                        );
                        let c2 = format!(
                            "{} {}",
                            format_float(cp2_value.x),
                            format_float(cp2_value.y)
                        );
                        arr.push(format!("C{}, {}, {}", c1, c2, next_point));
                        continue;
                    }
                }
                let p = cp1.unwrap_or(cp2.unwrap_or_default());

                let q = format!("{} {}", format_float(p.x), format_float(p.y));
                arr.push(format!("Q{}, {}", q, next_point));
            }
        }
        write!(f, "{}", arr.join(" "))
    }
}
