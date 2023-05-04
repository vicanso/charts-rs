mod canvas;
mod color;
mod common;
mod component;
mod font;
mod line_chart;
mod path;
mod util;

pub use canvas::Canvas;
pub use color::*;
pub use common::*;
pub use component::{
    Axis, Circle, Grid, Line, Polygon, Polyline, Rect, SmoothLine, SmoothLineFill, StraightLine,
    StraightLineFill, Text,
};
pub use font::{add_font, get_font, measure_text_width_family};
pub use line_chart::LineChart;
pub use path::*;
pub use util::*;

// pub fn measure_text(font_family: &str, font_size: f64, text: &str) -> Box {
// }
