use super::canvas;
use super::color::*;
use super::common::*;
use super::component::*;
use super::font::measure_max_text_width_family;
use super::params::*;
use super::theme::{get_default_theme, get_theme, Theme, DEFAULT_Y_AXIS_WIDTH};
use super::util::*;
use super::Canvas;
use crate::charts::measure_text_width_family;
use charts_rs_derive::Chart;

#[derive(Clone, Debug, Default)]
pub struct HeatmapData {
    pub index: usize,
    pub value: f32,
}

impl From<(usize, f32)> for HeatmapData {
    fn from(value: (usize, f32)) -> Self {
        HeatmapData {
            index: value.0,
            value: value.1,
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct HeatmapSeries {
    pub data: Vec<HeatmapData>,
    pub min: f32,
    pub max: f32,
    pub min_color: Color,
    pub max_color: Color,
    pub min_font_color: Color,
    pub max_font_color: Color,
}

impl HeatmapSeries {
    fn get_color(&self, value: f32) -> Color {
        if value < self.min {
            return self.min_color;
        }
        if value > self.max {
            return self.max_color;
        }
        let percent = (value - self.min) / (self.max - self.min);
        let get_value = |max, min| {
            let offset = if max > min { max - min } else { min - max };
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
pub struct HeatmapChart {
    pub width: f32,
    pub height: f32,
    pub x: f32,
    pub y: f32,
    pub margin: Box,
    // no use, but for derive chart
    series_list: Vec<Series>,
    pub series: HeatmapSeries,
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
    pub x_axis_hidden: bool,
    pub x_boundary_gap: Option<bool>,

    // y axis
    pub y_axis_hidden: bool,
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

impl HeatmapChart {
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
        if self.series.min_font_color.is_zero() {
            self.series.min_font_color = (70, 70, 70).into();
        }
        if self.series.max_font_color.is_zero() {
            self.series.max_font_color = (238, 238, 238).into();
        }
        if self.series.max == 0.0 {
            let mut max = 0.0;
            for item in self.series.data.iter() {
                if item.value > max {
                    max = item.value
                }
            }
            self.series.max = max;
        }
    }
    /// Creates a heatmap chart from json.
    pub fn from_json(data: &str) -> canvas::Result<HeatmapChart> {
        let mut h = HeatmapChart {
            ..Default::default()
        };
        let value = h.fill_option(data)?;
        if let Some(y_axis_data) = get_string_slice_from_value(&value, "y_axis_data") {
            h.y_axis_data = y_axis_data;
        }
        if let Some(value) = value.get("series") {
            if let Some(min) = get_f32_from_value(value, "min") {
                h.series.min = min;
            }
            if let Some(max) = get_f32_from_value(value, "max") {
                h.series.max = max;
            }
            if let Some(min_color) = get_color_from_value(value, "min_color") {
                h.series.min_color = min_color;
            }
            if let Some(max_color) = get_color_from_value(value, "max_color") {
                h.series.max_color = max_color;
            }
            if let Some(min_font_color) = get_color_from_value(value, "min_font_color") {
                h.series.min_font_color = min_font_color;
            }
            if let Some(max_font_color) = get_color_from_value(value, "max_font_color") {
                h.series.max_font_color = max_font_color;
            }
            if let Some(data) = value.get("data") {
                let mut values = vec![];
                if let Some(arr) = data.as_array() {
                    for item in arr.iter() {
                        if let Some(arr) = item.as_array() {
                            if arr.len() != 2 {
                                continue;
                            }
                            values.push(HeatmapData {
                                index: arr[0].as_i64().unwrap_or_default() as usize,
                                value: arr[1].as_f64().unwrap_or_default() as f32,
                            });
                        }
                    }
                }
                h.series.data = values;
            }
        }
        h.fill_default();
        if let Some(x_axis_hidden) = get_bool_from_value(&value, "x_axis_hidden") {
            h.x_axis_hidden = x_axis_hidden;
        }
        if let Some(y_axis_hidden) = get_bool_from_value(&value, "y_axis_hidden") {
            h.y_axis_hidden = y_axis_hidden;
        }
        Ok(h)
    }
    /// Creates a heatmap chart with default theme.
    pub fn new(
        series_data: Vec<(usize, f32)>,
        x_axis_data: Vec<String>,
        y_axis_data: Vec<String>,
    ) -> HeatmapChart {
        HeatmapChart::new_with_theme(series_data, x_axis_data, y_axis_data, &get_default_theme())
    }
    /// Creates a heatmap chart with custom theme.
    pub fn new_with_theme(
        series_data: Vec<(usize, f32)>,
        x_axis_data: Vec<String>,
        y_axis_data: Vec<String>,
        theme: &str,
    ) -> HeatmapChart {
        let mut h = HeatmapChart {
            x_axis_data,
            y_axis_data,
            ..Default::default()
        };
        let mut max = 0.0_f32;
        let mut data = vec![];
        for item in series_data.iter() {
            if item.1 > max {
                max = item.1;
            }
            data.push((*item).into());
        }
        h.series.data = data;
        let theme = get_theme(theme);
        h.fill_theme(theme);
        h.fill_default();
        h
    }
    /// Converts heatmap chart to svg.
    pub fn svg(&self) -> canvas::Result<String> {
        let mut c = Canvas::new_width_xy(self.width, self.height, self.x, self.y);

        if self.x_axis_data.is_empty() || self.y_axis_data.is_empty() {
            return Err(canvas::Error::Params {
                message: "x axis or y axis can not be empty".to_string(),
            });
        }

        self.render_background(c.child(Box::default()));
        let mut x_axis_height = self.x_axis_height;
        if self.x_axis_hidden {
            x_axis_height = 0.0;
        }

        c.margin = self.margin.clone();

        let title_height = self.render_title(c.child(Box::default()));

        let legend_height = self.render_legend(c.child(Box::default()));
        // get the max height of title and legend
        let axis_top = if legend_height > title_height {
            legend_height
        } else {
            title_height
        };
        let axis_height = c.height() - x_axis_height - axis_top;

        // minus the height of top text area
        if axis_top > 0.0 {
            c = c.child(Box {
                top: axis_top,
                ..Default::default()
            });
        }
        let mut y_axis_width = 0.0;
        if !self.y_axis_hidden {
            let max_text_width_box = measure_max_text_width_family(
                &self.font_family,
                self.y_axis_configs[0].axis_font_size,
                self.y_axis_data.iter().map(|item| item.as_str()).collect(),
            )?;
            y_axis_width = max_text_width_box.width() + self.margin.left;
            // y axis
            let mut y_axis_data = self.y_axis_data.clone();
            y_axis_data.reverse();
            self.render_y_axis(
                c.child_left_top(Box::default()),
                y_axis_data,
                axis_height,
                y_axis_width,
                0,
            );
        }
        let axis_width = c.width() - y_axis_width;
        // x axis
        if !self.x_axis_hidden {
            self.render_x_axis(
                c.child(Box {
                    top: c.height() - x_axis_height,
                    left: y_axis_width,
                    ..Default::default()
                }),
                self.x_axis_data.clone(),
                axis_width,
            );
        }
        let mut data = vec![None; self.x_axis_data.len() * self.y_axis_data.len()];
        for item in self.series.data.iter() {
            if item.index < data.len() {
                data[item.index] = Some(item.value);
            }
        }

        let x_unit = (axis_width - 1.0) / self.x_axis_data.len() as f32;
        let y_unit = (axis_height - 1.0) / self.y_axis_data.len() as f32;
        let mut c1 = c.child(Box {
            left: y_axis_width + 1.0,
            ..Default::default()
        });
        let y_axis_count = self.y_axis_data.len();
        for i in 0..y_axis_count {
            for j in 0..self.x_axis_data.len() {
                let index = i * self.y_axis_data.len() + j;
                let x = x_unit * j as f32;
                // position of y axis starts from bottom
                let y = y_unit * (y_axis_count - i - 1) as f32;
                let mut text = "".to_string();
                let mut font_color = self.series.min_font_color;
                let color = if let Some(value) = data[index] {
                    let percent = (value - self.series.min) / (self.series.max - self.series.min);
                    if percent >= 0.8 {
                        font_color = self.series.max_font_color;
                    }

                    text = format_series_value(value, &self.series_label_formatter);
                    self.series.get_color(value)
                } else {
                    let mut color_index = j;
                    if i % 2 != 0 {
                        color_index += 1;
                    }
                    let mut color = self.background_color;
                    let offset = 20;
                    if color.is_light() {
                        color.r -= offset;
                        color.g -= offset;
                        color.b -= offset;
                    } else {
                        color.r += offset;
                        color.g += offset;
                        color.b += offset;
                    }
                    if color_index % 2 != 0 {
                        color = color.with_alpha(100);
                    }
                    color
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
                if !text.is_empty() {
                    let mut x1 = x + x_unit / 2.0;
                    let y1 = y + y_unit / 2.0;
                    if let Ok(b) = measure_text_width_family(
                        &self.font_family,
                        self.series_label_font_size,
                        &text,
                    ) {
                        x1 -= b.width() / 2.0;
                    }
                    c1.text(Text {
                        text,
                        font_family: Some(self.font_family.clone()),
                        font_color: Some(font_color),
                        font_size: Some(self.series_label_font_size),
                        font_weight: self.series_label_font_weight.clone(),
                        dominant_baseline: Some("central".to_string()),
                        x: Some(x1),
                        y: Some(y1),
                        ..Default::default()
                    });
                }
            }
        }

        c.svg()
    }
}

#[cfg(test)]
mod tests {
    use crate::THEME_DARK;

    use super::HeatmapChart;
    use pretty_assertions::assert_eq;

    #[test]
    fn heatmap_chart_basic() {
        let x_axis_data = vec![
            "12a", "1a", "2a", "3a", "4a", "5a", "6a", "7a", "8a", "9a", "10a", "11a", "12p", "1p",
            "2p", "3p", "4p", "5p", "6p", "7p", "8p", "9p", "10p", "11p",
        ]
        .iter()
        .map(|item| item.to_string())
        .collect();
        let y_axis_data = [
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
        let mut heatmap_chart = HeatmapChart::new(
            vec![
                (0, 9.0),
                (1, 3.0),
                (7, 3.0),
                (12, 3.0),
                (24, 12.0),
                (28, 10.0),
                (31, 8.0),
                (50, 4.0),
                (63, 2.0),
            ],
            x_axis_data,
            y_axis_data,
        );
        heatmap_chart.width = 800.0;
        heatmap_chart.series.max = 10.0;

        assert_eq!(
            include_str!("../../asset/heatmap_chart/basic.svg"),
            heatmap_chart.svg().unwrap()
        );
    }

    #[test]
    fn heatmap_chart_dark() {
        let x_axis_data = vec![
            "12a", "1a", "2a", "3a", "4a", "5a", "6a", "7a", "8a", "9a", "10a", "11a", "12p", "1p",
            "2p", "3p", "4p", "5p", "6p", "7p", "8p", "9p", "10p", "11p",
        ]
        .iter()
        .map(|item| item.to_string())
        .collect();
        let y_axis_data = [
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
        let mut heatmap_chart = HeatmapChart::new_with_theme(
            vec![
                (0, 9.0),
                (1, 3.0),
                (7, 3.0),
                (12, 3.0),
                (24, 12.0),
                (28, 10.0),
                (31, 8.0),
                (50, 4.0),
                (63, 2.0),
            ],
            x_axis_data,
            y_axis_data,
            THEME_DARK,
        );
        heatmap_chart.width = 800.0;
        heatmap_chart.series.max = 10.0;

        assert_eq!(
            include_str!("../../asset/heatmap_chart/basic_dark.svg"),
            heatmap_chart.svg().unwrap()
        );
    }

    #[test]
    fn heatmap_chart_no_axis() {
        let x_axis_data = vec![
            "12a", "1a", "2a", "3a", "4a", "5a", "6a", "7a", "8a", "9a", "10a", "11a", "12p", "1p",
            "2p", "3p", "4p", "5p", "6p", "7p", "8p", "9p", "10p", "11p",
        ]
        .iter()
        .map(|item| item.to_string())
        .collect();
        let y_axis_data = [
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
        let mut heatmap_chart = HeatmapChart::new(
            vec![
                (0, 9.0),
                (1, 3.0),
                (7, 3.0),
                (12, 3.0),
                (24, 12.0),
                (28, 10.0),
                (31, 8.0),
                (50, 4.0),
                (63, 2.0),
            ],
            x_axis_data,
            y_axis_data,
        );
        heatmap_chart.width = 800.0;
        heatmap_chart.series.max = 10.0;
        heatmap_chart.x_axis_hidden = true;
        heatmap_chart.y_axis_hidden = true;

        assert_eq!(
            include_str!("../../asset/heatmap_chart/no_axis.svg"),
            heatmap_chart.svg().unwrap()
        );
    }
}
