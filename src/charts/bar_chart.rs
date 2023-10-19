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
    pub series_label_font_weight: Option<String>,
    pub series_label_formatter: String,
    pub series_colors: Vec<Color>,
    pub series_symbol: Option<Symbol>,
    pub series_smooth: bool,
    pub series_fill: bool,
}

impl BarChart {
    /// Creates a bar chart from json.
    pub fn from_json(data: &str) -> canvas::Result<BarChart> {
        let mut b = BarChart {
            ..Default::default()
        };
        b.fill_option(data)?;
        Ok(b)
    }
    /// Creates a bar chart with custom theme.
    pub fn new_with_theme(
        mut series_list: Vec<Series>,
        x_axis_data: Vec<String>,
        theme: &str,
    ) -> BarChart {
        // bar chart 可能同时支持两种图
        // 因此先计算index
        series_list
            .iter_mut()
            .enumerate()
            .for_each(|(index, item)| {
                item.index = Some(index);
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
    /// Creates a bar chart with default theme.
    pub fn new(series_list: Vec<Series>, x_axis_data: Vec<String>) -> BarChart {
        BarChart::new_with_theme(series_list, x_axis_data, &get_default_theme())
    }
    /// Converts bar chart to svg.
    pub fn svg(&self) -> canvas::Result<String> {
        let mut c = Canvas::new_width_xy(self.width, self.height, self.x, self.y);

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
            self.x_axis_data.len(),
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
            self.x_axis_data.len(),
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
        Box, LegendCategory, SeriesCategory, NIL_VALUE, THEME_ANT, THEME_DARK, THEME_GRAFANA,
    };
    use pretty_assertions::assert_eq;
    #[test]
    fn bar_chart_basic() {
        let mut bar_chart = BarChart::new(
            vec![
                (
                    "Email",
                    vec![120.0, 132.0, 101.0, 134.0, 90.0, 230.0, 210.0],
                )
                    .into(),
                (
                    "Union Ads",
                    vec![220.0, 182.0, 191.0, 234.0, 290.0, 330.0, 310.0],
                )
                    .into(),
                (
                    "Direct",
                    vec![320.0, 332.0, 301.0, 334.0, 390.0, 330.0, 320.0],
                )
                    .into(),
                (
                    "Search Engine",
                    vec![820.0, 932.0, 901.0, 934.0, 1290.0, 1330.0, 1320.0],
                )
                    .into(),
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
        bar_chart.legend_category = LegendCategory::RoundRect;
        bar_chart.y_axis_configs[0].axis_formatter = Some("{c} ml".to_string());
        bar_chart.series_list[0].label_show = true;
        bar_chart.series_list[0].colors = Some(vec![None, Some("#a90000".into())]);
        assert_eq!(
            include_str!("../../asset/bar_chart/basic.svg"),
            bar_chart.svg().unwrap()
        );
    }
    #[test]
    fn bar_chart_basic_dark() {
        let mut bar_chart = BarChart::new_with_theme(
            vec![
                (
                    "Email",
                    vec![120.0, 132.0, 101.0, 134.0, 90.0, 230.0, 210.0],
                )
                    .into(),
                (
                    "Union Ads",
                    vec![220.0, 182.0, 191.0, 234.0, 290.0, 330.0, 310.0],
                )
                    .into(),
                (
                    "Direct",
                    vec![320.0, 332.0, 301.0, 334.0, 390.0, 330.0, 320.0],
                )
                    .into(),
                (
                    "Search Engine",
                    vec![820.0, 932.0, 901.0, 934.0, 1290.0, 1330.0, 1320.0],
                )
                    .into(),
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
        bar_chart.legend_category = LegendCategory::Circle;
        assert_eq!(
            include_str!("../../asset/bar_chart/basic_dark.svg"),
            bar_chart.svg().unwrap()
        );
    }

    #[test]
    fn bar_chart_basic_ant() {
        let mut bar_chart = BarChart::new_with_theme(
            vec![
                (
                    "Email",
                    vec![120.0, 132.0, 101.0, 134.0, 90.0, 230.0, 210.0],
                )
                    .into(),
                (
                    "Union Ads",
                    vec![220.0, 182.0, 191.0, 234.0, 290.0, 330.0, 310.0],
                )
                    .into(),
                (
                    "Direct",
                    vec![320.0, 332.0, 301.0, 334.0, 390.0, 330.0, 320.0],
                )
                    .into(),
                (
                    "Search Engine",
                    vec![820.0, 932.0, 901.0, 934.0, 1290.0, 1330.0, 1320.0],
                )
                    .into(),
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
                (
                    "Email",
                    vec![120.0, 132.0, 101.0, 134.0, 90.0, 230.0, 210.0],
                )
                    .into(),
                (
                    "Union Ads",
                    vec![220.0, 182.0, 191.0, 234.0, 290.0, 330.0, 310.0],
                )
                    .into(),
                (
                    "Direct",
                    vec![320.0, 332.0, 301.0, 334.0, 390.0, 330.0, 320.0],
                )
                    .into(),
                (
                    "Search Engine",
                    vec![820.0, 932.0, 901.0, 934.0, 1290.0, 1330.0, 1320.0],
                )
                    .into(),
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
    fn bar_chart_y_axis_min_max() {
        let mut bar_chart = BarChart::new_with_theme(
            vec![
                (
                    "Email",
                    vec![120.0, 132.0, 101.0, 134.0, 90.0, 230.0, 210.0],
                )
                    .into(),
                (
                    "Union Ads",
                    vec![220.0, 182.0, 191.0, 234.0, 290.0, 330.0, 310.0],
                )
                    .into(),
                (
                    "Direct",
                    vec![320.0, 332.0, 301.0, 334.0, 390.0, 330.0, 320.0],
                )
                    .into(),
                (
                    "Search Engine",
                    vec![820.0, 932.0, 901.0, 934.0, 1290.0, 1330.0, 1320.0],
                )
                    .into(),
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
        bar_chart.y_axis_configs[0].axis_max = Some(1500.0);
        bar_chart.title_text = "Bar Chart".to_string();
        bar_chart.legend_margin = Some(Box {
            top: 35.0,
            bottom: 10.0,
            ..Default::default()
        });
        bar_chart.y_axis_configs[0].axis_formatter = Some("{c} ml".to_string());
        bar_chart.series_list[0].label_show = true;
        assert_eq!(
            include_str!("../../asset/bar_chart/y_axis_min_max.svg"),
            bar_chart.svg().unwrap()
        );
    }

    #[test]
    fn bar_chart_line_mixin() {
        let mut bar_chart = BarChart::new(
            vec![
                (
                    "Email",
                    vec![120.0, 132.0, 101.0, 134.0, 90.0, 230.0, 210.0],
                )
                    .into(),
                (
                    "Union Ads",
                    vec![220.0, 182.0, 191.0, 234.0, 290.0, 330.0, 310.0],
                )
                    .into(),
                (
                    "Direct",
                    vec![320.0, 332.0, 301.0, 334.0, 390.0, 330.0, 320.0],
                )
                    .into(),
                (
                    "Search Engine",
                    vec![820.0, 932.0, 901.0, 934.0, 1290.0, 1330.0, 1320.0],
                )
                    .into(),
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

        #[cfg(feature = "image")]
        {
            use crate::svg_to_png;
            let buf = svg_to_png(&bar_chart.svg().unwrap()).unwrap();
            std::fs::write("./asset/line_mixin.png", buf).unwrap();
        }
    }

    #[test]
    fn bar_chart_two_y_axis() {
        let mut bar_chart = BarChart::new(
            vec![
                ("Evaporation", vec![2.0, 4.9, 7.0, 23.2, 25.6, 76.7, 135.6]).into(),
                (
                    "Precipitation",
                    vec![2.6, 5.9, 9.0, 26.4, 28.7, 70.7, 175.6],
                )
                    .into(),
                ("Temperature", vec![2.0, 2.2, 3.3, 4.5, 6.3, 10.2, 20.3]).into(),
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

    #[test]
    fn bar_chart_value_count_unequal() {
        let mut bar_chart = BarChart::new(
            vec![
                ("Email", vec![120.0, 132.0, 101.0, 134.0, 90.0, 230.0]).into(),
                (
                    "Union Ads",
                    vec![220.0, 182.0, 191.0, 234.0, 290.0, 330.0, 310.0],
                )
                    .into(),
                (
                    "Direct",
                    vec![320.0, 332.0, 301.0, 334.0, 390.0, 330.0, 320.0],
                )
                    .into(),
                (
                    "Search Engine",
                    vec![820.0, 932.0, 901.0, 934.0, 1290.0, 1330.0, 1320.0],
                )
                    .into(),
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
        bar_chart.series_list[0].start_index = 1;
        assert_eq!(
            include_str!("../../asset/bar_chart/value_count_unequal.svg"),
            bar_chart.svg().unwrap()
        );
    }

    #[test]
    fn bar_chart_nil_value() {
        let mut bar_chart = BarChart::new(
            vec![
                (
                    "Email",
                    vec![120.0, NIL_VALUE, 132.0, 101.0, 134.0, 90.0, 230.0],
                )
                    .into(),
                (
                    "Union Ads",
                    vec![220.0, 182.0, 191.0, NIL_VALUE, 290.0, 330.0, 310.0],
                )
                    .into(),
                (
                    "Direct",
                    vec![320.0, 332.0, 301.0, 334.0, 390.0, NIL_VALUE, 320.0],
                )
                    .into(),
                (
                    "Search Engine",
                    vec![NIL_VALUE, 932.0, 901.0, 934.0, 1290.0, 1330.0, 1320.0],
                )
                    .into(),
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
            include_str!("../../asset/bar_chart/nil_value.svg"),
            bar_chart.svg().unwrap()
        );
    }
}
