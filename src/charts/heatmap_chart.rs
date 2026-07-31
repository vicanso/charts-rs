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

use super::Canvas;
use super::base::ChartBase;
use super::canvas;
use super::color::*;
use super::common::*;
use super::component::*;
use super::font::measure_max_text_width_family;
use super::params::*;
use super::theme::{get_default_theme_name, get_theme};
use super::util::*;
use crate::charts::measure_text_width_family;

/// One heatmap cell: a flat grid `index` plus its value.
#[derive(Clone, Debug, Default)]
pub struct HeatmapData {
    /// Flat index of the cell in the grid.
    pub index: usize,
    /// Value of the cell.
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

/// The heatmap cells plus the value range and its color mapping.
#[derive(Clone, Debug, Default)]
pub struct HeatmapSeries {
    /// The heatmap cells.
    pub data: Vec<HeatmapData>,
    /// Lower bound of the value range.
    pub min: f32,
    /// Upper bound of the value range.
    pub max: f32,
    /// Color mapped to the lower bound.
    pub min_color: Color,
    /// Color mapped to the upper bound.
    pub max_color: Color,
    /// Label font color on low-value cells.
    pub min_font_color: Color,
    /// Label font color on high-value cells.
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
        let get_value = |max: u8, min: u8| {
            let offset = max.abs_diff(min);
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

/// A heatmap over an x/y category grid, coloring cells by value.
#[derive(Clone, Debug, Default)]
pub struct HeatmapChart {
    /// The shared chart options (size, series, title/legend, axes); exposed
    /// directly on the chart through `Deref`, e.g. `chart.title_text`.
    pub base: ChartBase,
    // no use, but for derive chart
    /// The heatmap cell data and color mapping.
    pub series: HeatmapSeries,

    // title

    // sub title

    // legend

    // x axis

    // y axis
    /// Labels of the y axis.
    pub y_axis_data: Vec<String>,
    y_axis_configs: Vec<YAxisConfig>,
    // grid

    // series
}

impl std::ops::Deref for HeatmapChart {
    type Target = ChartBase;
    fn deref(&self) -> &ChartBase {
        &self.base
    }
}
impl std::ops::DerefMut for HeatmapChart {
    fn deref_mut(&mut self) -> &mut ChartBase {
        &mut self.base
    }
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
        let value = h.base.fill_option(data, &mut h.y_axis_configs)?;
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
        Ok(h)
    }
    /// Creates a heatmap chart with default theme.
    pub fn new(
        series_data: Vec<(usize, f32)>,
        x_axis_data: Vec<String>,
        y_axis_data: Vec<String>,
    ) -> HeatmapChart {
        HeatmapChart::new_with_theme(
            series_data,
            x_axis_data,
            y_axis_data,
            &get_default_theme_name(),
        )
    }
    /// Creates a heatmap chart with custom theme.
    pub fn new_with_theme(
        series_data: Vec<(usize, f32)>,
        x_axis_data: Vec<String>,
        y_axis_data: Vec<String>,
        theme: &str,
    ) -> HeatmapChart {
        let mut h = HeatmapChart {
            y_axis_data,
            ..Default::default()
        };
        h.x_axis_data = x_axis_data;
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
        h.base.fill_theme(theme, &mut h.y_axis_configs);
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

        let mut x_axis_height = self.x_axis_height;
        if self.x_axis_hidden {
            x_axis_height = 0.0;
        }
        let axis_top = self.render_header(&mut c);
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
                &self.y_axis_configs,
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
                let index = i * self.x_axis_data.len() + j;
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
                    fill: Some(color.into()),
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
