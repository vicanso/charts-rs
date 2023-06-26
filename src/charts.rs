mod bar_chart;
mod canvas;
mod color;
mod common;
mod component;
mod encoder;
mod font;
mod horizontal_bar_chart;
mod line_chart;
mod path;
mod pie_chart;
mod radar_chart;
mod theme;
mod util;

pub use bar_chart::BarChart;
pub use canvas::Canvas;
pub use color::*;
pub use common::*;
pub use component::{
    Axis, Circle, Grid, Legend, LegendCategory, Line, Pie, Polygon, Polyline, Rect, SmoothLine,
    SmoothLineFill, StraightLine, StraightLineFill, Text,
};
pub use encoder::svg_to_png;
pub use font::{add_font, get_font, measure_text_width_family, DEFAULT_FONT_FAMILY};
pub use line_chart::LineChart;
pub use path::*;
pub use pie_chart::PieChart;
pub use radar_chart::RadarChart;
pub use theme::Theme;
pub use theme::{THEME_ANT, THEME_DARK, THEME_GRAFANA};
pub use util::*;

pub trait Chart {
    fn fill_theme(&mut self, t: Theme);
    fn get_y_axis_config(&self, index: usize) -> YAxisConfig;
    fn get_y_axis_values(&self, y_axis_index: usize) -> (AxisValues, f32);
    fn render_background(&self, c: Canvas);
    fn render_title(&self, c: Canvas) -> f32;
    fn render_legend(&self, c: Canvas) -> f32;
    fn render_grid(&self, c: Canvas, axis_width: f32, axis_height: f32);
    fn render_y_axis(
        &self,
        c: Canvas,
        data: Vec<String>,
        axis_height: f32,
        axis_width: f32,
        axis_index: usize,
    );
    fn render_x_axis(&self, c: Canvas, data: Vec<String>, axis_width: f32);
    fn render_series_label(&self, c: Canvas, series_labels_list: Vec<Vec<SeriesLabel>>);
    fn render_bar(
        &self,
        c: Canvas,
        series_list: &[&Series],
        y_axis_values: &[&AxisValues],
        max_height: f32,
    ) -> Vec<Vec<SeriesLabel>>;
    fn render_line(
        &self,
        c: Canvas,
        series_list: &[&Series],
        y_axis_values: &[&AxisValues],
        max_height: f32,
        axis_height: f32,
    ) -> Vec<Vec<SeriesLabel>>;
}
