mod canvas;
mod color;
mod common;
mod component;
mod font;
mod line_chart;
mod path;
mod theme;
mod util;

pub use canvas::Canvas;
pub use color::*;
pub use common::*;
pub use component::{
    Axis, Circle, Grid, Legend, LegendCategory, Line, Polygon, Polyline, Rect, SmoothLine,
    SmoothLineFill, StraightLine, StraightLineFill, Text,
};
pub use font::{add_font, get_font, measure_text_width_family, DEFAULT_FONT_FAMILY};
pub use line_chart::LineChart;
pub use path::*;
pub use theme::Theme;
pub use util::*;

pub trait ChartBasic {
    fn fill_theme(&mut self, t: Theme) {}
}
