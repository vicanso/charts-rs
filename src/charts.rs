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

pub trait Chart {
    fn fill_theme(&mut self, t: Theme);
    fn render_background(&self, c: Canvas);
    fn render_title(&self, c: Canvas) -> f32;
    fn render_legend(&self, c: Canvas) -> f32;
    fn render_grid(&self, c: Canvas, axis_width: f32, axis_height: f32);
    fn render_y_axis(&self, c: Canvas, data: Vec<String>, axis_height: f32);
    fn render_x_axis(&self, c: Canvas, data: Vec<String>, axis_width: f32);
    fn render_lines(
        &self,
        c: Canvas,
        series_list: &[Series],
        y_axis_values: &AxisValues,
        max_height: f32,
        axis_height: f32,
    );
}
