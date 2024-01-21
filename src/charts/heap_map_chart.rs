use super::canvas;
use super::color::*;
use super::common::*;
use super::component::*;
use super::font::measure_max_text_width_family;
use super::params::*;
use super::theme::{get_default_theme, get_theme, Theme, DEFAULT_Y_AXIS_WIDTH};
use super::util::*;
use super::Canvas;
use super::Chart;
use crate::charts::measure_text_width_family;
use charts_rs_derive::Chart;

#[derive(Clone, Debug, Default)]
pub struct HeapMapData {
    pub index: usize,
    pub value: f32,
}

impl From<(usize, f32)> for HeapMapData {
    fn from(value: (usize, f32)) -> Self {
        HeapMapData {
            index: value.0,
            value: value.1,
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct HeapMapSeries {
    pub data: Vec<HeapMapData>,
    pub min: f32,
    pub max: f32,
    pub min_color: Color,
    pub max_color: Color,
    pub background_colors: Vec<Color>,
}

impl HeapMapSeries {
    fn get_color(&self, value: f32) -> Color {
        if value < self.min {
            return self.min_color;
        }
        if value > self.max {
            return self.max_color;
        }
        let percent = (value - self.min) / (self.max - self.min);
        let get_value = |max, min| {
            let offset = if max > min {
                max - min
            } else {
                min - max
            };
            let offset = (offset as f32 * percent) as u8;
            if max > min {
                min + offset
            } else {
                min - offset
            }
        };
        Color {
            r: get_value(self.max_color.r, self.min_color.r),
            g: get_value(self.max_color.g, self.min_color.g),
            b: get_value(self.max_color.b, self.min_color.b),
            a: get_value(self.max_color.a, self.min_color.a),
        }
    }
}

#[derive(Clone, Debug, Default, Chart)]
pub struct HeapMapChart {
    pub width: f32,
    pub height: f32,
    pub x: f32,
    pub y: f32,
    pub margin: Box,
    // heap map使用新的数据结构
    series_list: Vec<Series>,
    pub series: HeapMapSeries,
    pub font_family: String,
    pub background_color: Color,
    pub is_light: bool,

    // title
    pub title_text: String,
    pub title_font_size: f32,
    pub title_font_color: Color,
    pub title_font_weight: Option<String>,
    pub title_margin: Option<Box>,
    pub title_align: Align,
    pub title_height: f32,

    // sub title
    pub sub_title_text: String,
    pub sub_title_font_size: f32,
    pub sub_title_font_color: Color,
    pub sub_title_font_weight: Option<String>,
    pub sub_title_margin: Option<Box>,
    pub sub_title_align: Align,
    pub sub_title_height: f32,

    // legend
    pub legend_font_size: f32,
    pub legend_font_color: Color,
    pub legend_font_weight: Option<String>,
    pub legend_align: Align,
    pub legend_margin: Option<Box>,
    pub legend_category: LegendCategory,
    pub legend_show: Option<bool>,

    // x axis
    pub x_axis_data: Vec<String>,
    pub x_axis_height: f32,
    pub x_axis_stroke_color: Color,
    pub x_axis_font_size: f32,
    pub x_axis_font_color: Color,
    pub x_axis_font_weight: Option<String>,
    pub x_axis_name_gap: f32,
    pub x_axis_name_rotate: f32,
    pub x_axis_margin: Option<Box>,
    pub x_boundary_gap: Option<bool>,

    // y axis
    pub y_axis_data: Vec<String>,
    y_axis_configs: Vec<YAxisConfig>,

    // grid
    grid_stroke_color: Color,
    grid_stroke_width: f32,

    // series
    pub series_stroke_width: f32,
    pub series_label_font_color: Color,
    pub series_label_font_size: f32,
    pub series_label_font_weight: Option<String>,
    pub series_label_formatter: String,
    pub series_colors: Vec<Color>,
    pub series_symbol: Option<Symbol>,
    pub series_smooth: bool,
    pub series_fill: bool,
}

impl HeapMapChart {
    fn fill_default(&mut self) {
        if self.y_axis_configs[0].axis_stroke_color.is_zero() {
            self.y_axis_configs[0].axis_stroke_color = self.x_axis_stroke_color;
        }
        self.y_axis_configs[0].axis_name_align = Some(Align::Center);
        self.y_axis_configs[0].axis_split_number += 1;
        if self.series.max_color.is_zero() {
            self.series.max_color = (191, 68, 76).into();
        }
        if self.series.min_color.is_zero() {
            self.series.min_color = (240, 217, 156).into();
        }
        if self.series.background_colors.is_empty() {
            self.series.background_colors = vec![
                (210, 219, 238).into(),
                (245, 247, 250).into(),
            ];
        }
    }
    /// Creates a heap map chart from json.
    pub fn from_json(data: &str) -> canvas::Result<HeapMapChart> {
        let mut h = HeapMapChart {
            ..Default::default()
        };
        h.fill_option(data)?;
        h.fill_default();
        Ok(h)
    }
    /// Creates a heap map chart with default theme.
    pub fn new(
        series_list: Vec<Series>,
        x_axis_data: Vec<String>,
        y_axis_data: Vec<String>,
    ) -> HeapMapChart {
        HeapMapChart::new_with_theme(series_list, x_axis_data, y_axis_data, &get_default_theme())
    }
    /// Creates a heap map chart with custom theme.
    pub fn new_with_theme(
        series_list: Vec<Series>,
        x_axis_data: Vec<String>,
        y_axis_data: Vec<String>,
        theme: &str,
    ) -> HeapMapChart {
        let mut h = HeapMapChart {
            series_list,
            x_axis_data,
            y_axis_data,
            ..Default::default()
        };
        let theme = get_theme(theme);
        h.fill_theme(theme);
        h.fill_default();
        h
    }
    /// Converts heap map chart to svg.
    pub fn svg(&self) -> canvas::Result<String> {
        let mut c = Canvas::new_width_xy(self.width, self.height, self.x, self.y);

        if self.x_axis_data.is_empty() || self.y_axis_data.is_empty() {
            return Err(canvas::Error::Params {
                message: "x axis or y axis can not be empty".to_string(),
            });
        }

        self.render_background(c.child(Box::default()));

        c.margin = self.margin.clone();

        let title_height = self.render_title(c.child(Box::default()));

        let legend_height = self.render_legend(c.child(Box::default()));
        // title 与 legend 取较高的值
        let axis_top = if legend_height > title_height {
            legend_height
        } else {
            title_height
        };
        let axis_height = c.height() - self.x_axis_height - axis_top;

        let max_text_width_box = measure_max_text_width_family(
            &self.font_family,
            self.y_axis_configs[0].axis_font_size,
            self.y_axis_data.iter().map(|item| item.as_str()).collect(),
        )?;
        // 减去顶部文本区域
        if axis_top > 0.0 {
            c = c.child(Box {
                top: axis_top,
                ..Default::default()
            });
        }
        let y_axis_width = max_text_width_box.width() + self.margin.left;
        // y axis
        let mut y_axis_data = self.y_axis_data.clone();
        y_axis_data.reverse();
        self.render_y_axis(
            c.child(Box::default()),
            y_axis_data,
            axis_height,
            y_axis_width,
            0,
        );
        let axis_width = c.width() - y_axis_width;
        // x axis
        self.render_x_axis(
            c.child(Box {
                top: c.height() - self.x_axis_height,
                left: y_axis_width,
                ..Default::default()
            }),
            self.x_axis_data.clone(),
            axis_width,
        );
        let mut data = vec![None; self.x_axis_data.len() * self.y_axis_data.len()];
        for item in self.series.data.iter() {
            if item.index < data.len() {
                data[item.index] = Some(item.value);
            }
        }

        let x_unit = axis_width / self.x_axis_data.len() as f32;
        let y_unit = axis_height / self.y_axis_data.len() as f32;
        let mut c1 = c.child(Box{
            left: y_axis_width,
            ..Default::default()
        });
        let background_colors = self.series.background_colors.clone();
        for i in 0..self.y_axis_data.len() {
            for j in 0..self.x_axis_data.len() {
                let index = i * self.y_axis_data.len() + j;
                let x = x_unit * j as f32;
                let y = y_unit * i as f32;
                let color = if let Some(value) = data[index] {
                    self.series.get_color(value)
                } else if background_colors.is_empty() {
                    Color::white()
                } else {
                    let mut color_index = j;
                    if i % 2 != 0 {
                        color_index += 1;
                    }
                    color_index %= background_colors.len();
                   
                    background_colors[color_index]
                };
                c1.rect(Rect {
                    color: Some(color),
                    fill: Some(color),
                    left: x,
                    top: y,
                    width: x_unit,
                    height: y_unit,
                    ..Default::default()
                });
            }
        }

        c.svg()
    }
}

#[cfg(test)]
mod tests {
    use super::HeapMapChart;
    use pretty_assertions::assert_eq;

    #[test]
    fn heap_map_chart_basic() {
        let x_axis_data = vec![
            "12a", "1a", "2a", "3a", "4a", "5a", "6a", "7a", "8a", "9a", "10a", "11a", "12p", "1p",
            "2p", "3p", "4p", "5p", "6p", "7p", "8p", "9p", "10p", "11p",
        ]
        .iter()
        .map(|item| item.to_string())
        .collect();
        let y_axis_data = vec![
            "Saturday",
            "Friday",
            "Thursday",
            "Wednesday",
            "Tuesday",
            "Monday",
            "Sunday",
        ]
        .iter()
        .map(|item| item.to_string())
        .collect();
        let mut heap_map_chart = HeapMapChart::new(
            vec![("Punch Card", vec![0.0, 0.0, 5.0]).into()],
            x_axis_data,
            y_axis_data,
        );
        heap_map_chart.width = 800.0;
        heap_map_chart.series.data = vec![
            (0, 9.0).into(),
            (1, 3.0).into(),
            (7, 3.0).into(),
            (12, 3.0).into(),
        ];
        heap_map_chart.series.max = 10.0;

        println!("{}", heap_map_chart.svg().unwrap());
        assert_eq!("", heap_map_chart.svg().unwrap());
    }
}
