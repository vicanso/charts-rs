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

use super::canvas;
use super::color::*;
use super::common::*;
use super::component::*;
use super::params::*;
use super::theme::{get_default_theme_name, get_theme, Theme, DEFAULT_Y_AXIS_WIDTH};
use super::util::*;
use super::Canvas;
use crate::charts::measure_text_width_family;
use charts_rs_derive::Chart;
use std::sync::Arc;

/// One data series for a box plot.
///
/// Each entry in `data` encodes one box as `[min, q1, median, q3, max]`.
#[derive(Clone, Debug, Default)]
pub struct BoxPlotSeries {
    pub name: String,
    /// `[min, q1, median, q3, max]` per x-axis category.
    pub data: Vec<[f32; 5]>,
    pub index: Option<usize>,
}

#[derive(Clone, Debug, Default, Chart)]
pub struct BoxPlotChart {
    pub width: f32,
    pub height: f32,
    pub x: f32,
    pub y: f32,
    pub margin: Box,
    pub series_list: Vec<Series>,
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
    pub y_axis_configs: Vec<YAxisConfig>,

    // grid
    pub grid_stroke_color: Color,
    pub grid_stroke_width: f32,

    // series (required by #[derive(Chart)])
    pub series_stroke_width: f32,
    pub series_label_font_color: Color,
    pub series_label_font_size: f32,
    pub series_label_font_weight: Option<String>,
    pub series_label_formatter: String,
    pub series_colors: Vec<Color>,
    pub series_symbol: Option<Symbol>,
    pub series_smooth: bool,
    pub series_fill: bool,

    // box plot specific
    pub box_series: Vec<BoxPlotSeries>,
}

impl BoxPlotChart {
    fn fill_default(&mut self) {
        // Sync series_list from box_series so legend and color-cycling work.
        if self.series_list.is_empty() {
            for (i, bs) in self.box_series.iter().enumerate() {
                let mut s = Series::new(bs.name.clone(), vec![]);
                s.index = Some(bs.index.unwrap_or(i));
                self.series_list.push(s);
            }
        }
        if self.y_axis_configs[0].axis_stroke_color.is_zero() {
            self.y_axis_configs[0].axis_stroke_color = self.x_axis_stroke_color;
        }
    }

    pub fn new_with_theme(
        box_series: Vec<BoxPlotSeries>,
        x_axis_data: Vec<String>,
        theme: &str,
    ) -> BoxPlotChart {
        let mut c = BoxPlotChart {
            box_series,
            x_axis_data,
            ..Default::default()
        };
        c.fill_theme(get_theme(theme));
        c.fill_default();
        c
    }

    pub fn new(box_series: Vec<BoxPlotSeries>, x_axis_data: Vec<String>) -> BoxPlotChart {
        BoxPlotChart::new_with_theme(box_series, x_axis_data, &get_default_theme_name())
    }

    pub fn from_json(json: &str) -> canvas::Result<BoxPlotChart> {
        let mut c = BoxPlotChart {
            ..Default::default()
        };
        let value = c.fill_option(json)?;
        // Parse box_series array
        if let Some(arr) = value.get("box_series").and_then(|v| v.as_array()) {
            for (i, item) in arr.iter().enumerate() {
                let name = get_string_from_value(item, "name").unwrap_or_default();
                let index = get_f32_from_value(item, "index").map(|v| v as usize);
                let mut data: Vec<[f32; 5]> = vec![];
                if let Some(rows) = item.get("data").and_then(|v| v.as_array()) {
                    for row in rows {
                        if let Some(vals) = row.as_array() {
                            if vals.len() >= 5 {
                                let f = |i: usize| vals[i].as_f64().unwrap_or(0.0) as f32;
                                data.push([f(0), f(1), f(2), f(3), f(4)]);
                            }
                        }
                    }
                }
                c.box_series.push(BoxPlotSeries {
                    name,
                    data,
                    index: index.or(Some(i)),
                });
            }
        }
        c.fill_default();
        Ok(c)
    }

    pub fn svg(&self) -> canvas::Result<String> {
        let mut c = Canvas::new_width_xy(self.width, self.height, self.x, self.y);
        self.render_background(c.child(Box::default()));
        let mut x_axis_height = self.x_axis_height;
        if self.x_axis_hidden {
            x_axis_height = 0.0;
        }
        c.margin = self.margin.clone();

        let title_height = self.render_title(c.child(Box::default()));
        let legend_height = self.render_legend(c.child(Box::default()));
        let axis_top = title_height.max(legend_height);

        // Collect all values to build y-axis range
        let mut all_values: Vec<f32> = vec![];
        for bs in &self.box_series {
            for entry in &bs.data {
                all_values.push(entry[0]); // min
                all_values.push(entry[4]); // max
            }
        }
        if all_values.is_empty() {
            return c.svg();
        }

        let y_axis_config = self.get_y_axis_config(0);
        let y_axis_values = get_axis_values(AxisValueParams {
            data_list: all_values,
            split_number: y_axis_config.axis_split_number,
            reverse: Some(true),
            min: y_axis_config.axis_min,
            max: y_axis_config.axis_max,
            ..Default::default()
        });

        let y_axis_width = if self.y_axis_hidden {
            0.0
        } else if let Some(w) = y_axis_config.axis_width {
            w
        } else {
            let formatter = y_axis_config.axis_formatter.clone().unwrap_or_default();
            let label = format_string(&y_axis_values.data[0], &formatter);
            measure_text_width_family(&self.font_family, y_axis_config.axis_font_size, &label)
                .map(|b| b.width() + 5.0)
                .unwrap_or(DEFAULT_Y_AXIS_WIDTH)
        };

        let axis_height = c.height() - x_axis_height - axis_top;
        let axis_width = c.width() - y_axis_width;

        if axis_top > 0.0 {
            c = c.child(Box {
                top: axis_top,
                ..Default::default()
            });
        }

        // Grid
        self.render_grid(
            c.child(Box {
                left: y_axis_width,
                ..Default::default()
            }),
            axis_width,
            axis_height,
        );

        // Y axis
        if !self.y_axis_hidden {
            self.render_y_axis(
                c.child(Box::default()),
                y_axis_values.data.clone(),
                axis_height,
                y_axis_width,
                0,
            );
        }

        // X axis
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

        let num_cats = self.x_axis_data.len().max(
            self.box_series
                .iter()
                .map(|bs| bs.data.len())
                .max()
                .unwrap_or(0),
        );
        if num_cats == 0 {
            return c.svg();
        }

        let num_series = self.box_series.len();
        let col_w = axis_width / num_cats as f32;
        // step = spacing between adjacent box centres; box_w = rendered width (80% of step)
        let total_boxes_w = col_w * 0.6_f32;
        let box_step = if num_series > 0 {
            total_boxes_w / num_series as f32
        } else {
            total_boxes_w
        };
        let box_w = box_step * 0.8;
        // Cap width for whisker line
        let cap_half = box_w * 0.3;
        let stroke_w = self.series_stroke_width.max(1.0);

        let mut data_c = c.child(Box {
            left: y_axis_width,
            ..Default::default()
        });

        for (si, bs) in self.box_series.iter().enumerate() {
            let color = get_color(&self.series_colors, bs.index.unwrap_or(si));
            let fill_color = color.with_alpha(80);

            for (ci, entry) in bs.data.iter().enumerate() {
                if ci >= num_cats {
                    break;
                }
                let [v_min, v_q1, v_med, v_q3, v_max] = *entry;

                // Centre x of this box
                let cat_cx = col_w * (ci as f32 + 0.5);
                let series_offset = (si as f32 - (num_series as f32 - 1.0) / 2.0) * box_step;
                let cx = cat_cx + series_offset;

                let y_min = y_axis_values.get_offset_height(v_min, axis_height);
                let y_q1 = y_axis_values.get_offset_height(v_q1, axis_height);
                let y_med = y_axis_values.get_offset_height(v_med, axis_height);
                let y_q3 = y_axis_values.get_offset_height(v_q3, axis_height);
                let y_max = y_axis_values.get_offset_height(v_max, axis_height);

                let box_left = cx - box_w / 2.0;
                let box_top = y_q3; // Q3 is higher value → lower y pixel
                let box_height = (y_q1 - y_q3).abs();

                // IQR box (Q1..Q3)
                data_c.rect(Rect {
                    fill: Some(Fill::Solid(fill_color)),
                    color: Some(color),
                    left: box_left,
                    top: box_top,
                    width: box_w,
                    height: box_height,
                    ..Default::default()
                });

                // Median line
                data_c.line(Line {
                    color: Some(color),
                    stroke_width: stroke_w + 1.0,
                    left: box_left,
                    right: box_left + box_w,
                    top: y_med,
                    bottom: y_med,
                    ..Default::default()
                });

                // Upper whisker Q3 → max
                data_c.line(Line {
                    color: Some(color),
                    stroke_width: stroke_w,
                    left: cx,
                    right: cx,
                    top: y_max,
                    bottom: y_q3,
                    ..Default::default()
                });

                // Lower whisker min → Q1
                data_c.line(Line {
                    color: Some(color),
                    stroke_width: stroke_w,
                    left: cx,
                    right: cx,
                    top: y_q1,
                    bottom: y_min,
                    ..Default::default()
                });

                // Upper cap at max
                data_c.line(Line {
                    color: Some(color),
                    stroke_width: stroke_w,
                    left: cx - cap_half,
                    right: cx + cap_half,
                    top: y_max,
                    bottom: y_max,
                    ..Default::default()
                });

                // Lower cap at min
                data_c.line(Line {
                    color: Some(color),
                    stroke_width: stroke_w,
                    left: cx - cap_half,
                    right: cx + cap_half,
                    top: y_min,
                    bottom: y_min,
                    ..Default::default()
                });
            }
        }

        c.svg()
    }
}

#[cfg(test)]
mod tests {
    use super::{BoxPlotChart, BoxPlotSeries};
    use pretty_assertions::assert_eq;

    fn make_box_plot() -> BoxPlotChart {
        BoxPlotChart::new(
            vec![
                BoxPlotSeries {
                    name: "Series A".to_string(),
                    data: vec![
                        [3.0, 10.0, 18.0, 28.0, 40.0],
                        [5.0, 14.0, 22.0, 32.0, 45.0],
                        [1.0, 8.0, 15.0, 24.0, 35.0],
                        [6.0, 12.0, 20.0, 30.0, 42.0],
                    ],
                    index: None,
                },
                BoxPlotSeries {
                    name: "Series B".to_string(),
                    data: vec![
                        [5.0, 13.0, 21.0, 31.0, 43.0],
                        [2.0, 9.0, 17.0, 26.0, 38.0],
                        [4.0, 11.0, 19.0, 29.0, 41.0],
                        [7.0, 15.0, 23.0, 33.0, 46.0],
                    ],
                    index: None,
                },
            ],
            vec![
                "Category A".to_string(),
                "Category B".to_string(),
                "Category C".to_string(),
                "Category D".to_string(),
            ],
        )
    }

    #[test]
    fn box_plot_chart_basic() {
        assert_eq!(
            include_str!("../../asset/box_plot_chart/basic.svg"),
            make_box_plot().svg().unwrap()
        );
    }

    #[test]
    fn box_plot_chart_basic_json() {
        let chart = BoxPlotChart::from_json(
            r##"{
                "title_text": "Box Plot",
                "x_axis_data": ["Cat A", "Cat B", "Cat C"],
                "box_series": [
                    {
                        "name": "Group 1",
                        "data": [
                            [3, 10, 18, 28, 40],
                            [5, 14, 22, 32, 45],
                            [1,  8, 15, 24, 35]
                        ]
                    },
                    {
                        "name": "Group 2",
                        "data": [
                            [5, 13, 21, 31, 43],
                            [2,  9, 17, 26, 38],
                            [4, 11, 19, 29, 41]
                        ]
                    }
                ]
            }"##,
        )
        .unwrap();
        assert_eq!(
            include_str!("../../asset/box_plot_chart/basic_json.svg"),
            chart.svg().unwrap()
        );
    }
}
