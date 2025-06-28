// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

mod bar_chart;
mod candlestick_chart;
mod canvas;
mod color;
mod common;
mod component;
#[cfg(feature = "image-encoder")]
mod encoder;
mod font;
mod heatmap_chart;
mod horizontal_bar_chart;
mod line_chart;
mod multi_chart;
mod params;
mod path;
mod pie_chart;
mod radar_chart;
mod scatter_chart;
mod table_chart;
mod theme;
mod util;

pub use bar_chart::BarChart;
pub use canvas::Canvas;
pub use canvas::Error as CanvasError;
pub use canvas::Result as CanvasResult;
pub use color::*;
pub use common::*;
pub use component::{
    Axis, Circle, Grid, Legend, LegendCategory, Line, Pie, Polygon, Polyline, Rect, SmoothLine,
    SmoothLineFill, StraightLine, StraightLineFill, Text,
};
#[cfg(feature = "image-encoder")]
pub(crate) use encoder::get_or_init_fontdb;
#[cfg(feature = "image-encoder")]
pub use encoder::Error as EncoderError;
#[cfg(feature = "image-encoder")]
pub use encoder::*;

pub use candlestick_chart::CandlestickChart;
pub use font::Error as FontError;
pub use font::{
    get_font, get_font_families, get_or_try_init_fonts, measure_text_width_family,
    DEFAULT_FONT_DATA, DEFAULT_FONT_FAMILY,
};
pub use heatmap_chart::{HeatmapChart, HeatmapData, HeatmapSeries};
pub use horizontal_bar_chart::HorizontalBarChart;
pub use line_chart::LineChart;
pub use multi_chart::{ChildChart, MultiChart};
pub use path::*;
pub use pie_chart::PieChart;
pub use radar_chart::{RadarChart, RadarIndicator};
pub use scatter_chart::ScatterChart;
pub use table_chart::{TableCellStyle, TableChart};
pub use theme::Theme;
pub use theme::{add_theme, get_theme, list_theme_name, THEME_ANT, THEME_DARK, THEME_GRAFANA};
pub use util::*;
