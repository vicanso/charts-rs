use super::canvas;
use super::color::*;
use super::common::*;
use super::component::*;
use super::params::*;
use super::theme::{get_default_theme, get_theme, Theme, DEFAULT_Y_AXIS_WIDTH};
use super::util::*;
use super::Canvas;
use crate::charts::measure_text_width_family;
use charts_rs_derive::Chart;

#[derive(Clone, Debug, Default, Chart)]
pub struct LineChart {
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

impl LineChart {
    /// Creates a line chart from json.
    pub fn from_json(data: &str) -> canvas::Result<LineChart> {
        let mut l = LineChart {
            ..Default::default()
        };
        let value = l.fill_option(data)?;
        if let Some(x_axis_hidden) = get_bool_from_value(&value, "x_axis_hidden") {
            l.x_axis_hidden = x_axis_hidden;
        }
        if let Some(y_axis_hidden) = get_bool_from_value(&value, "y_axis_hidden") {
            l.y_axis_hidden = y_axis_hidden;
        }
        Ok(l)
    }
    /// Creates a line chart with custom theme.
    pub fn new_with_theme(
        series_list: Vec<Series>,
        x_axis_data: Vec<String>,
        theme: &str,
    ) -> LineChart {
        let mut l = LineChart {
            series_list,
            x_axis_data,
            ..Default::default()
        };
        let theme = get_theme(theme);
        l.fill_theme(theme);
        l
    }
    /// Creates a line chart with default theme.
    pub fn new(series_list: Vec<Series>, x_axis_data: Vec<String>) -> LineChart {
        LineChart::new_with_theme(series_list, x_axis_data, &get_default_theme())
    }
    fn render_mark_line(
        &self,
        c: Canvas,
        series_list: &[Series],
        y_axis_values_list: &[&AxisValues],
        max_height: f32,
    ) {
        let mut c = c;
        for (index, series) in series_list.iter().enumerate() {
            if series.mark_lines.is_empty() {
                continue;
            }
            let y_axis_values = if series.y_axis_index >= y_axis_values_list.len() {
                y_axis_values_list[0]
            } else {
                y_axis_values_list[series.y_axis_index]
            };
            let color = get_color(&self.series_colors, series.index.unwrap_or(index));
            let values: Vec<_> = series
                .data
                .iter()
                .filter(|x| *x.to_owned() != NIL_VALUE)
                .map(|x| x.to_owned())
                .collect();
            let mut sum = 0.0;
            let mut min = f32::MAX;
            let mut max = f32::MIN;
            for value in values.iter() {
                let v = *value;
                if v == NIL_VALUE {
                    continue;
                }
                sum += v;
                if v > max {
                    max = v;
                }
                if v < min {
                    min = v;
                }
            }
            let average = sum / values.len() as f32;
            for mark_line in series.mark_lines.iter() {
                let value = match mark_line.category {
                    MarkLineCategory::Average => average,
                    MarkLineCategory::Max => max,
                    MarkLineCategory::Min => min,
                };
                let y = y_axis_values.get_offset_height(value, max_height);
                let arrow_width = 10.0;
                c.circle(Circle {
                    stroke_color: Some(color),
                    fill: Some(color),
                    cx: 3.0,
                    cy: y,
                    r: 3.5,
                    ..Default::default()
                });
                c.line(Line {
                    color: Some(color),
                    left: 8.0,
                    top: y,
                    right: c.width() - arrow_width,
                    bottom: y,
                    stroke_dash_array: Some("4,2".to_string()),
                    ..Default::default()
                });
                c.arrow(Arrow {
                    x: c.width() - arrow_width,
                    y,
                    stroke_color: color,
                    ..Arrow::default()
                });
                let line_height = 20.0;
                c.text(Text {
                    text: format_float(value),
                    font_family: Some(self.font_family.clone()),
                    font_size: Some(self.series_label_font_size),
                    line_height: Some(line_height),
                    font_color: Some(self.series_label_font_color),
                    x: Some(c.width() + 2.0),
                    y: Some(y - line_height / 2.0 + 1.0),
                    ..Default::default()
                });
            }
        }
    }
    /// Converts line chart to svg.
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
        // get the max height of title and legend
        let axis_top = if legend_height > title_height {
            legend_height
        } else {
            title_height
        };

        let (left_y_axis_values, mut left_y_axis_width) = self.get_y_axis_values(0);
        if self.y_axis_hidden {
            left_y_axis_width = 0.0;
        }
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

        let axis_height = c.height() - x_axis_height - axis_top;
        let axis_width = c.width() - left_y_axis_width - right_y_axis_width;
        // minus the height of top text area
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
        if left_y_axis_width > 0.0 {
            self.render_y_axis(
                c.child(Box::default()),
                left_y_axis_values.data.clone(),
                axis_height,
                left_y_axis_width,
                0,
            );
        }
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
        if !self.x_axis_hidden {
            self.render_x_axis(
                c.child(Box {
                    top: c.height() - x_axis_height,
                    left: left_y_axis_width,
                    right: right_y_axis_width,
                    ..Default::default()
                }),
                self.x_axis_data.clone(),
                axis_width,
            );
        }

        // line point
        let y_axis_values_list = vec![&left_y_axis_values, &right_y_axis_values];
        let max_height = c.height() - x_axis_height;
        let line_series_list: Vec<&Series> = self.series_list.iter().collect();
        let series_labels_list = self.render_line(
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
        self.render_series_label(
            c.child(Box {
                left: left_y_axis_width,
                right: right_y_axis_width,
                ..Default::default()
            }),
            series_labels_list,
        );

        self.render_mark_line(
            c.child(Box {
                left: left_y_axis_width,
                right: right_y_axis_width,
                ..Default::default()
            }),
            &self.series_list,
            &y_axis_values_list,
            max_height,
        );
        c.svg()
    }
}

#[cfg(test)]
mod tests {
    use super::LineChart;
    use crate::{Align, Box, MarkLine, MarkLineCategory, MarkPoint, MarkPointCategory, NIL_VALUE};
    use pretty_assertions::assert_eq;
    #[test]
    fn line_chart_basic() {
        let mut line_chart = LineChart::new(
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
        line_chart.series_list[0].stroke_dash_array = Some("4,2".to_string());
        line_chart.margin.right = 50.0;
        line_chart.title_text = "Stacked Area Chart".to_string();
        line_chart.sub_title_text = "Hello World".to_string();
        line_chart.legend_margin = Some(Box {
            top: 50.0,
            bottom: 10.0,
            ..Default::default()
        });
        line_chart.series_list[3].mark_lines = vec![MarkLine {
            category: MarkLineCategory::Average,
        }];
        line_chart.series_list[3].label_show = true;
        line_chart.series_list[2].mark_points = vec![
            MarkPoint {
                category: MarkPointCategory::Max,
            },
            MarkPoint {
                category: MarkPointCategory::Min,
            },
        ];
        assert_eq!(
            include_str!("../../asset/line_chart/basic.svg"),
            line_chart.svg().unwrap()
        );
    }

    #[test]
    fn line_chart_nil_value() {
        let mut line_chart = LineChart::new(
            vec![
                (
                    "Email",
                    vec![120.0, NIL_VALUE, 101.0, 134.0, 90.0, 230.0, 210.0],
                )
                    .into(),
                (
                    "Union Ads",
                    vec![220.0, 182.0, 191.0, NIL_VALUE, 290.0, 330.0, 310.0],
                )
                    .into(),
                (
                    "Direct",
                    vec![320.0, 332.0, NIL_VALUE, 334.0, 390.0, 330.0, 320.0],
                )
                    .into(),
                (
                    "Search Engine",
                    vec![820.0, 932.0, 901.0, 934.0, 1290.0, NIL_VALUE, 1320.0],
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
        line_chart.title_text = "Stacked Area Chart".to_string();
        line_chart.sub_title_text = "Hello World".to_string();
        line_chart.legend_margin = Some(Box {
            top: 50.0,
            bottom: 10.0,
            ..Default::default()
        });
        line_chart.series_list[3].label_show = true;
        assert_eq!(
            include_str!("../../asset/line_chart/nil_value.svg"),
            line_chart.svg().unwrap()
        );
    }

    #[test]
    fn line_chart_align_left() {
        let mut line_chart = LineChart::new(
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
        line_chart.title_text = "Stacked Area Chart".to_string();
        line_chart.sub_title_text = "Hello World".to_string();
        line_chart.legend_margin = Some(Box {
            top: 50.0,
            bottom: 10.0,
            ..Default::default()
        });
        line_chart.x_boundary_gap = Some(false);
        line_chart.margin = (5.0, 5.0, 15.0, 5.0).into();
        assert_eq!(
            include_str!("../../asset/line_chart/boundary_gap.svg"),
            line_chart.svg().unwrap()
        );
    }
    #[test]
    fn line_chart_fill() {
        let mut line_chart = LineChart::new(
            vec![(
                "Search Engine",
                vec![820.0, 932.0, 901.0, 934.0, 1290.0, 1330.0, 1320.0],
            )
                .into()],
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
        line_chart.series_fill = true;
        line_chart.series_smooth = true;
        line_chart.title_text = "Stacked Area Chart".to_string();
        line_chart.sub_title_text = "Hello World".to_string();
        line_chart.legend_margin = Some(Box {
            top: 50.0,
            bottom: 10.0,
            ..Default::default()
        });
        assert_eq!(
            include_str!("../../asset/line_chart/smooth_fill.svg"),
            line_chart.svg().unwrap()
        );
    }
    #[test]
    fn line_chart_legend_align_right() {
        let mut line_chart = LineChart::new(
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
        line_chart.title_text = "Stacked Area Chart".to_string();
        line_chart.sub_title_text = "Hello World".to_string();
        line_chart.title_align = Align::Left;
        line_chart.legend_align = Align::Right;

        line_chart.x_boundary_gap = Some(false);
        line_chart.margin = (5.0, 5.0, 15.0, 5.0).into();
        assert_eq!(
            include_str!("../../asset/line_chart/legend_align_right.svg"),
            line_chart.svg().unwrap()
        );
    }

    #[test]
    fn line_chart_two_y_axis() {
        let mut line_chart = LineChart::new(
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
        line_chart.title_text = "Stacked Area Chart".to_string();
        line_chart.sub_title_text = "Hello World".to_string();
        line_chart.legend_margin = Some(Box {
            top: 50.0,
            bottom: 10.0,
            ..Default::default()
        });
        line_chart.series_list[3].y_axis_index = 1;
        let mut y_axis_config = line_chart.y_axis_configs[0].clone();
        y_axis_config.axis_font_color = "#ee6666".into();
        line_chart.y_axis_configs.push(y_axis_config);

        assert_eq!(
            include_str!("../../asset/line_chart/two_y_axis.svg"),
            line_chart.svg().unwrap()
        );
    }

    #[test]
    fn line_chart_value_count_unequal() {
        let mut line_chart = LineChart::new(
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
        line_chart.series_list[0].start_index = 1;
        line_chart.title_text = "Stacked Area Chart".to_string();
        line_chart.sub_title_text = "Hello World".to_string();
        line_chart.legend_margin = Some(Box {
            top: 50.0,
            bottom: 10.0,
            ..Default::default()
        });
        line_chart.series_list[3].label_show = true;
        assert_eq!(
            include_str!("../../asset/line_chart/value_count_unequal.svg"),
            line_chart.svg().unwrap()
        );
    }

    #[test]
    fn line_chart_no_axis() {
        let mut line_chart = LineChart::new(
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
        line_chart.series_list[0].stroke_dash_array = Some("4,2".to_string());
        line_chart.margin.right = 50.0;
        line_chart.title_text = "Stacked Area Chart".to_string();
        line_chart.sub_title_text = "Hello World".to_string();
        line_chart.legend_margin = Some(Box {
            top: 50.0,
            bottom: 10.0,
            ..Default::default()
        });
        line_chart.series_list[3].mark_lines = vec![MarkLine {
            category: MarkLineCategory::Average,
        }];
        line_chart.series_list[3].label_show = true;
        line_chart.series_list[2].mark_points = vec![
            MarkPoint {
                category: MarkPointCategory::Max,
            },
            MarkPoint {
                category: MarkPointCategory::Min,
            },
        ];
        line_chart.x_axis_hidden = true;
        line_chart.y_axis_hidden = true;
        assert_eq!(
            include_str!("../../asset/line_chart/no_axis.svg"),
            line_chart.svg().unwrap()
        );
    }

    #[test]
    fn line_chart_small_value() {
        let mut line_chart = LineChart::new(
            vec![(
                "latency",
                vec![
                    1.12, 1.18, 1.65, 1.87, 1.92, 1.43, 1.65, 0.83, 0.65, 0.12, 1.1, 0.87,
                ],
            )
                .into()],
            vec![
                "01".to_string(),
                "02".to_string(),
                "03".to_string(),
                "04".to_string(),
                "05".to_string(),
                "06".to_string(),
                "07".to_string(),
                "08".to_string(),
                "09".to_string(),
                "10".to_string(),
                "11".to_string(),
                "12".to_string(),
            ],
        );
        line_chart.title_text = "Request Latency".to_string();
        line_chart.legend_margin = Some(Box {
            top: 50.0,
            bottom: 10.0,
            ..Default::default()
        });
        assert_eq!(
            include_str!("../../asset/line_chart/small_value.svg"),
            line_chart.svg().unwrap()
        );
    }
}
