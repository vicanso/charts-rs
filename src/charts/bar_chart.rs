use super::canvas;
use super::color::*;
use super::common::*;
use super::component::*;
use super::params::*;
use super::theme::{get_default_theme, get_theme, Theme, DEFAULT_Y_AXIS_WIDTH};
use super::util::*;
use super::Canvas;
use super::Chart;
use crate::charts::measure_text_width_family;
use charts_rs_derive::Chart;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, Default, Chart)]
pub struct BarChart {
    pub width: f32,
    pub height: f32,
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
    pub sub_title_margin: Option<Box>,
    pub sub_title_align: Align,
    pub sub_title_height: f32,

    // legend
    pub legend_font_size: f32,
    pub legend_font_color: Color,
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
    pub x_axis_name_gap: f32,
    pub x_axis_name_rotate: f32,
    pub x_boundary_gap: Option<bool>,

    // y axis
    pub y_axis_configs: Vec<YAxisConfig>,

    // grid
    pub grid_stroke_color: Color,
    pub grid_stroke_width: f32,

    // series
    pub series_stroke_width: f32,
    pub series_label_font_color: Color,
    pub series_label_font_size: f32,
    pub series_colors: Vec<Color>,
    pub series_symbol: Option<Symbol>,
    pub series_smooth: bool,
    pub series_fill: bool,
}

impl BarChart {
    pub fn from_json(data: &str) -> canvas::Result<BarChart> {
        let data: serde_json::Value = serde_json::from_str(data)?;
        let mut b = BarChart {
            ..Default::default()
        };
        b.fill_option(data)?;
        Ok(b)
    }
    fn fill_option(&mut self, data: serde_json::Value) -> canvas::Result<()> {
        let series_list = get_series_list_from_value(&data).unwrap_or_default();
        if series_list.is_empty() {
            return Err(canvas::Error::Params {
                message: "series list can not be empty".to_string(),
            });
        }
        let x_axis_data = get_string_slice_from_value(&data, "x_axis_data").unwrap_or_default();
        if x_axis_data.is_empty() {
            return Err(canvas::Error::Params {
                message: "x axis list can not be empty".to_string(),
            });
        }
        let theme = get_string_from_value(&data, "theme").unwrap_or_default();
        self.fill_theme(get_theme(&theme));
        self.series_list = series_list;
        self.x_axis_data = x_axis_data;

        if let Some(width) = get_f32_from_value(&data, "width") {
            self.width = width;
        }
        if let Some(height) = get_f32_from_value(&data, "height") {
            self.height = height;
        }
        if let Some(margin) = get_margin_from_value(&data, "margin") {
            self.margin = margin;
        }
        if let Some(font_family) = get_string_from_value(&data, "font_family") {
            self.font_family = font_family;
        }
        if let Some(title_text) = get_string_from_value(&data, "title_text") {
            self.title_text = title_text;
        }
        if let Some(title_font_size) = get_f32_from_value(&data, "title_font_size") {
            self.title_font_size = title_font_size;
        }
        if let Some(title_font_color) = get_color_from_value(&data, "title_font_color") {
            self.title_font_color = title_font_color;
        }
        if let Some(title_font_weight) = get_string_from_value(&data, "title_font_weight") {
            self.title_font_weight = Some(title_font_weight);
        }
        if let Some(title_margin) = get_margin_from_value(&data, "title_margin") {
            self.title_margin = Some(title_margin);
        }
        if let Some(title_align) = get_align_from_value(&data, "title_align") {
            self.title_align = title_align;
        }
        if let Some(title_height) = get_f32_from_value(&data, "title_height") {
            self.title_height = title_height;
        }

        if let Some(sub_title_text) = get_string_from_value(&data, "sub_title_text") {
            self.sub_title_text = sub_title_text;
        }
        if let Some(sub_title_font_size) = get_f32_from_value(&data, "sub_title_font_size") {
            self.sub_title_font_size = sub_title_font_size;
        }
        if let Some(sub_title_font_color) = get_color_from_value(&data, "sub_title_font_color") {
            self.sub_title_font_color = sub_title_font_color;
        }
        if let Some(sub_title_margin) = get_margin_from_value(&data, "sub_title_margin") {
            self.sub_title_margin = Some(sub_title_margin);
        }
        if let Some(sub_title_align) = get_align_from_value(&data, "sub_title_align") {
            self.sub_title_align = sub_title_align;
        }
        if let Some(sub_title_height) = get_f32_from_value(&data, "sub_title_height") {
            self.sub_title_height = sub_title_height;
        }

        if let Some(legend_font_size) = get_f32_from_value(&data, "legend_font_size") {
            self.legend_font_size = legend_font_size;
        }
        if let Some(legend_font_color) = get_color_from_value(&data, "legend_font_color") {
            self.legend_font_color = legend_font_color;
        }
        if let Some(legend_align) = get_align_from_value(&data, "legend_align") {
            self.legend_align = legend_align;
        }
        if let Some(legend_margin) = get_margin_from_value(&data, "legend_margin") {
            self.legend_margin = Some(legend_margin);
        }
        if let Some(legend_category) = get_legend_category_from_value(&data, "legend_category") {
            self.legend_category = legend_category;
        }
        if let Some(legend_show) = get_bool_from_value(&data, "legend_show") {
            self.legend_show = Some(legend_show);
        }

        if let Some(x_axis_height) = get_f32_from_value(&data, "x_axis_height") {
            self.x_axis_height = x_axis_height;
        }
        if let Some(x_axis_stroke_color) = get_color_from_value(&data, "x_axis_stroke_color") {
            self.x_axis_stroke_color = x_axis_stroke_color;
        }
        if let Some(x_axis_font_size) = get_f32_from_value(&data, "x_axis_font_size") {
            self.x_axis_font_size = x_axis_font_size;
        }
        if let Some(x_axis_font_color) = get_color_from_value(&data, "x_axis_font_color") {
            self.x_axis_font_color = x_axis_font_color;
        }
        if let Some(x_axis_name_gap) = get_f32_from_value(&data, "x_axis_name_gap") {
            self.x_axis_name_gap = x_axis_name_gap;
        }
        if let Some(x_axis_name_rotate) = get_f32_from_value(&data, "x_axis_name_rotate") {
            self.x_axis_name_rotate = x_axis_name_rotate;
        }
        if let Some(x_boundary_gap) = get_bool_from_value(&data, "x_boundary_gap") {
            self.x_boundary_gap = Some(x_boundary_gap);
        }

        if let Some(y_axis_configs) = get_y_axis_configs_from_value(&data, "y_axis_configs") {
            self.y_axis_configs = y_axis_configs;
        }

        if let Some(grid_stroke_color) = get_color_from_value(&data, "grid_stroke_color") {
            self.grid_stroke_color = grid_stroke_color;
        }
        if let Some(grid_stroke_width) = get_f32_from_value(&data, "grid_stroke_width") {
            self.grid_stroke_width = grid_stroke_width;
        }

        if let Some(series_stroke_width) = get_f32_from_value(&data, "series_stroke_width") {
            self.series_stroke_width = series_stroke_width;
        }
        if let Some(series_label_font_color) =
            get_color_from_value(&data, "series_label_font_color")
        {
            self.series_label_font_color = series_label_font_color;
        }
        if let Some(series_label_font_size) = get_f32_from_value(&data, "series_label_font_size") {
            self.series_label_font_size = series_label_font_size;
        }
        if let Some(series_colors) = get_color_slice_from_value(&data, "series_colors") {
            self.series_colors = series_colors;
        }
        if let Some(series_smooth) = get_bool_from_value(&data, "series_smooth") {
            self.series_smooth = series_smooth;
        }
        if let Some(series_fill) = get_bool_from_value(&data, "series_fill") {
            self.series_fill = series_fill;
        }

        Ok(())
    }
    pub fn new_with_theme(
        mut series_list: Vec<Series>,
        x_axis_data: Vec<String>,
        theme: &str,
    ) -> BarChart {
        let mut series_index: usize = 0;
        // bar chart 可能同时支持两种图
        // 因此先计算index
        series_list.iter_mut().for_each(|item| {
            item.index = Some(series_index);
            series_index += 1;
        });
        let mut b = BarChart {
            series_list,
            x_axis_data,
            ..Default::default()
        };
        let theme = get_theme(theme);
        b.fill_theme(theme);
        b
    }
    pub fn new(series_list: Vec<Series>, x_axis_data: Vec<String>) -> BarChart {
        BarChart::new_with_theme(series_list, x_axis_data, &get_default_theme())
    }
    pub fn svg(&self) -> canvas::Result<String> {
        let mut c = Canvas::new(self.width, self.height);

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

        let (left_y_axis_values, left_y_axis_width) = self.get_y_axis_values(0);
        let mut exist_right_y_axis = false;
        for series in self.series_list.iter() {
            if series.y_axis_index != 0 {
                exist_right_y_axis = true;
            }
        }
        let mut right_y_axis_values = AxisValues::default();
        let mut right_y_axis_width = 0.0_f32;
        if exist_right_y_axis {
            (right_y_axis_values, right_y_axis_width) = self.get_y_axis_values(1);
        }

        let axis_height = c.height() - self.x_axis_height - axis_top;
        let axis_width = c.width() - left_y_axis_width - right_y_axis_width;
        // 减去顶部文本区域
        if axis_top > 0.0 {
            c = c.child(Box {
                top: axis_top,
                ..Default::default()
            });
        }

        self.render_grid(
            c.child(Box {
                left: left_y_axis_width,
                ..Default::default()
            }),
            axis_width,
            axis_height,
        );

        // y axis
        self.render_y_axis(
            c.child(Box::default()),
            left_y_axis_values.data.clone(),
            axis_height,
            left_y_axis_width,
            0,
        );
        if right_y_axis_width > 0.0 {
            self.render_y_axis(
                c.child(Box {
                    left: c.width() - right_y_axis_width,
                    ..Default::default()
                }),
                right_y_axis_values.data.clone(),
                axis_height,
                right_y_axis_width,
                1,
            );
        }

        // x axis
        self.render_x_axis(
            c.child(Box {
                top: c.height() - self.x_axis_height,
                left: left_y_axis_width,
                right: right_y_axis_width,
                ..Default::default()
            }),
            self.x_axis_data.clone(),
            axis_width,
        );

        // bar point
        let max_height = c.height() - self.x_axis_height;
        let mut bar_series_list = vec![];
        let mut line_series_list = vec![];
        self.series_list.iter().for_each(|item| {
            if let Some(ref cat) = item.category {
                if *cat == SeriesCategory::Line {
                    line_series_list.push(item);
                    return;
                }
            }
            bar_series_list.push(item);
        });

        let y_axis_values_list = vec![&left_y_axis_values, &right_y_axis_values];
        let mut bar_series_labels_list = self.render_bar(
            c.child(Box {
                left: left_y_axis_width,
                right: right_y_axis_width,
                ..Default::default()
            }),
            &bar_series_list,
            &y_axis_values_list,
            max_height,
        );

        let mut line_series_labels_list = self.render_line(
            c.child(Box {
                left: left_y_axis_width,
                right: right_y_axis_width,
                ..Default::default()
            }),
            &line_series_list,
            &y_axis_values_list,
            max_height,
            axis_height,
        );

        bar_series_labels_list.append(&mut line_series_labels_list);

        self.render_series_label(
            c.child(Box {
                left: left_y_axis_width,
                right: right_y_axis_width,
                ..Default::default()
            }),
            bar_series_labels_list,
        );

        c.svg()
    }
}

#[cfg(test)]
mod tests {
    use super::BarChart;
    use crate::{
        svg_to_png, Box, LegendCategory, Series, SeriesCategory, THEME_ANT, THEME_DARK,
        THEME_GRAFANA,
    };
    use pretty_assertions::assert_eq;
    #[test]
    fn bar_chart_basic() {
        let mut bar_chart = BarChart::new(
            vec![
                Series::new(
                    "Email".to_string(),
                    vec![120.0, 132.0, 101.0, 134.0, 90.0, 230.0, 210.0],
                ),
                Series::new(
                    "Union Ads".to_string(),
                    vec![220.0, 182.0, 191.0, 234.0, 290.0, 330.0, 310.0],
                ),
                Series::new(
                    "Direct".to_string(),
                    vec![320.0, 332.0, 301.0, 334.0, 390.0, 330.0, 320.0],
                ),
                Series::new(
                    "Search Engine".to_string(),
                    vec![820.0, 932.0, 901.0, 934.0, 1290.0, 1330.0, 1320.0],
                ),
            ],
            vec![
                "Mon".to_string(),
                "Tue".to_string(),
                "Wed".to_string(),
                "Thu".to_string(),
                "Fri".to_string(),
                "Sat".to_string(),
                "Sun".to_string(),
            ],
        );
        bar_chart.y_axis_configs[0].axis_width = Some(55.0);
        bar_chart.title_text = "Bar Chart".to_string();
        bar_chart.legend_margin = Some(Box {
            top: 35.0,
            bottom: 10.0,
            ..Default::default()
        });
        bar_chart.y_axis_configs[0].axis_formatter = Some("{c} ml".to_string());
        bar_chart.series_list[0].label_show = true;
        assert_eq!(
            include_str!("../../asset/bar_chart/basic.svg"),
            bar_chart.svg().unwrap()
        );
    }
    #[test]
    fn bar_chart_basic_dark() {
        let mut bar_chart = BarChart::new_with_theme(
            vec![
                Series::new(
                    "Email".to_string(),
                    vec![120.0, 132.0, 101.0, 134.0, 90.0, 230.0, 210.0],
                ),
                Series::new(
                    "Union Ads".to_string(),
                    vec![220.0, 182.0, 191.0, 234.0, 290.0, 330.0, 310.0],
                ),
                Series::new(
                    "Direct".to_string(),
                    vec![320.0, 332.0, 301.0, 334.0, 390.0, 330.0, 320.0],
                ),
                Series::new(
                    "Search Engine".to_string(),
                    vec![820.0, 932.0, 901.0, 934.0, 1290.0, 1330.0, 1320.0],
                ),
            ],
            vec![
                "Mon".to_string(),
                "Tue".to_string(),
                "Wed".to_string(),
                "Thu".to_string(),
                "Fri".to_string(),
                "Sat".to_string(),
                "Sun".to_string(),
            ],
            THEME_DARK,
        );
        bar_chart.y_axis_configs[0].axis_width = Some(55.0);
        bar_chart.title_text = "Bar Chart".to_string();
        bar_chart.legend_margin = Some(Box {
            top: 35.0,
            bottom: 10.0,
            ..Default::default()
        });
        bar_chart.y_axis_configs[0].axis_formatter = Some("{c} ml".to_string());
        bar_chart.series_list[0].label_show = true;
        assert_eq!(
            include_str!("../../asset/bar_chart/basic_dark.svg"),
            bar_chart.svg().unwrap()
        );
    }

    #[test]
    fn bar_chart_basic_ant() {
        let mut bar_chart = BarChart::new_with_theme(
            vec![
                Series::new(
                    "Email".to_string(),
                    vec![120.0, 132.0, 101.0, 134.0, 90.0, 230.0, 210.0],
                ),
                Series::new(
                    "Union Ads".to_string(),
                    vec![220.0, 182.0, 191.0, 234.0, 290.0, 330.0, 310.0],
                ),
                Series::new(
                    "Direct".to_string(),
                    vec![320.0, 332.0, 301.0, 334.0, 390.0, 330.0, 320.0],
                ),
                Series::new(
                    "Search Engine".to_string(),
                    vec![820.0, 932.0, 901.0, 934.0, 1290.0, 1330.0, 1320.0],
                ),
            ],
            vec![
                "Mon".to_string(),
                "Tue".to_string(),
                "Wed".to_string(),
                "Thu".to_string(),
                "Fri".to_string(),
                "Sat".to_string(),
                "Sun".to_string(),
            ],
            THEME_ANT,
        );
        bar_chart.y_axis_configs[0].axis_width = Some(55.0);
        bar_chart.title_text = "Bar Chart".to_string();
        bar_chart.legend_margin = Some(Box {
            top: 35.0,
            bottom: 10.0,
            ..Default::default()
        });
        bar_chart.y_axis_configs[0].axis_formatter = Some("{c} ml".to_string());
        bar_chart.series_list[0].label_show = true;
        assert_eq!(
            include_str!("../../asset/bar_chart/basic_ant.svg"),
            bar_chart.svg().unwrap()
        );
    }

    #[test]
    fn bar_chart_basic_grafana() {
        let mut bar_chart = BarChart::new_with_theme(
            vec![
                Series::new(
                    "Email".to_string(),
                    vec![120.0, 132.0, 101.0, 134.0, 90.0, 230.0, 210.0],
                ),
                Series::new(
                    "Union Ads".to_string(),
                    vec![220.0, 182.0, 191.0, 234.0, 290.0, 330.0, 310.0],
                ),
                Series::new(
                    "Direct".to_string(),
                    vec![320.0, 332.0, 301.0, 334.0, 390.0, 330.0, 320.0],
                ),
                Series::new(
                    "Search Engine".to_string(),
                    vec![820.0, 932.0, 901.0, 934.0, 1290.0, 1330.0, 1320.0],
                ),
            ],
            vec![
                "Mon".to_string(),
                "Tue".to_string(),
                "Wed".to_string(),
                "Thu".to_string(),
                "Fri".to_string(),
                "Sat".to_string(),
                "Sun".to_string(),
            ],
            THEME_GRAFANA,
        );
        bar_chart.y_axis_configs[0].axis_width = Some(55.0);
        bar_chart.title_text = "Bar Chart".to_string();
        bar_chart.legend_margin = Some(Box {
            top: 35.0,
            bottom: 10.0,
            ..Default::default()
        });
        bar_chart.y_axis_configs[0].axis_formatter = Some("{c} ml".to_string());
        bar_chart.series_list[0].label_show = true;
        assert_eq!(
            include_str!("../../asset/bar_chart/basic_grafana.svg"),
            bar_chart.svg().unwrap()
        );
    }
    #[test]
    fn bar_chart_line_mixin() {
        let mut bar_chart = BarChart::new(
            vec![
                Series::new(
                    "Email".to_string(),
                    vec![120.0, 132.0, 101.0, 134.0, 90.0, 230.0, 210.0],
                ),
                Series::new(
                    "Union Ads".to_string(),
                    vec![220.0, 182.0, 191.0, 234.0, 290.0, 330.0, 310.0],
                ),
                Series::new(
                    "Direct".to_string(),
                    vec![320.0, 332.0, 301.0, 334.0, 390.0, 330.0, 320.0],
                ),
                Series::new(
                    "Search Engine".to_string(),
                    vec![820.0, 932.0, 901.0, 934.0, 1290.0, 1330.0, 1320.0],
                ),
            ],
            vec![
                "Mon".to_string(),
                "Tue".to_string(),
                "Wed".to_string(),
                "Thu".to_string(),
                "Fri".to_string(),
                "Sat".to_string(),
                "Sun".to_string(),
            ],
        );
        bar_chart.series_list[0].category = Some(SeriesCategory::Line);
        bar_chart.y_axis_configs[0].axis_width = Some(55.0);
        bar_chart.title_text = "Bar Chart".to_string();
        bar_chart.legend_margin = Some(Box {
            top: 35.0,
            bottom: 10.0,
            ..Default::default()
        });
        bar_chart.legend_category = LegendCategory::Rect;
        bar_chart.y_axis_configs[0].axis_formatter = Some("{c} ml".to_string());
        bar_chart.series_list[0].label_show = true;
        bar_chart.series_list[3].label_show = true;

        assert_eq!(
            include_str!("../../asset/bar_chart/line_mixin.svg"),
            bar_chart.svg().unwrap()
        );

        let buf = svg_to_png(&bar_chart.svg().unwrap()).unwrap();
        std::fs::write("./asset/line_mixin.png", buf).unwrap();
    }

    #[test]
    fn bar_chart_two_y_axis() {
        let mut bar_chart = BarChart::new(
            vec![
                Series::new(
                    "Evaporation".to_string(),
                    vec![
                        2.0, 4.9, 7.0, 23.2, 25.6, 76.7, 135.6, 162.2, 32.6, 20.0, 6.4, 3.3,
                    ],
                ),
                Series::new(
                    "Precipitation".to_string(),
                    vec![
                        2.6, 5.9, 9.0, 26.4, 28.7, 70.7, 175.6, 182.2, 48.7, 18.8, 6.0, 2.3,
                    ],
                ),
                Series::new(
                    "Temperature".to_string(),
                    vec![
                        2.0, 2.2, 3.3, 4.5, 6.3, 10.2, 20.3, 23.4, 23.0, 16.5, 12.0, 6.2,
                    ],
                ),
            ],
            vec![
                "Mon".to_string(),
                "Tue".to_string(),
                "Wed".to_string(),
                "Thu".to_string(),
                "Fri".to_string(),
                "Sat".to_string(),
                "Sun".to_string(),
            ],
        );
        bar_chart.series_list[2].category = Some(SeriesCategory::Line);
        bar_chart.series_list[2].y_axis_index = 1;

        bar_chart.y_axis_configs[0].axis_width = Some(55.0);
        bar_chart.title_text = "Bar Chart".to_string();
        bar_chart.legend_margin = Some(Box {
            top: 35.0,
            bottom: 10.0,
            ..Default::default()
        });
        bar_chart.legend_category = LegendCategory::Rect;
        bar_chart.y_axis_configs[0].axis_formatter = Some("{c} ml".to_string());
        bar_chart
            .y_axis_configs
            .push(bar_chart.y_axis_configs[0].clone());
        bar_chart.y_axis_configs[1].axis_formatter = Some("{c} °C".to_string());
        assert_eq!(
            include_str!("../../asset/bar_chart/two_y_axis.svg"),
            bar_chart.svg().unwrap()
        );
    }
}
