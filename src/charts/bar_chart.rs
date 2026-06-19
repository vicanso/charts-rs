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
use super::canvas;
use super::color::*;
use super::common::*;
use super::component::*;
use super::params::*;
use super::theme::{DEFAULT_Y_AXIS_WIDTH, Theme, get_default_theme_name, get_theme};
use super::util::*;
use crate::charts::measure_text_width_family;
use charts_rs_derive::Chart;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[charts_rs_derive::chart_common_fields]
#[derive(Serialize, Deserialize, Clone, Debug, Default, Chart)]
pub struct BarChart {
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

    pub radius: Option<f32>,
    pub animation: Option<AnimationConfig>,
    /// When `true`, every bar gets a hover tooltip (`series: value`): a
    /// CSS-revealed label that works in any browser, plus a native `<title>`
    /// for accessibility. Default: false; output is unchanged when off.
    pub tooltip_show: bool,
}

impl BarChart {
    /// Creates a bar chart from json.
    pub fn from_json(data: &str) -> canvas::Result<BarChart> {
        let mut b = BarChart {
            ..Default::default()
        };
        let value = b.fill_option(data)?;
        if let Some(x_axis_hidden) = get_bool_from_value(&value, "x_axis_hidden") {
            b.x_axis_hidden = x_axis_hidden;
        }
        if let Some(y_axis_hidden) = get_bool_from_value(&value, "y_axis_hidden") {
            b.y_axis_hidden = y_axis_hidden;
        }
        if let Some(radius) = get_f32_from_value(&value, "radius") {
            b.radius = Some(radius);
        }
        if let Some(anim) = value.get("animation")
            && !anim.is_null()
        {
            let mut config = AnimationConfig::default();
            if let Some(d) = get_usize_from_value(anim, "duration") {
                config.duration = d as u32;
            }
            if let Some(e) = get_string_from_value(anim, "easing") {
                config.easing = e;
            }
            if let Some(d) = get_usize_from_value(anim, "delay") {
                config.delay = d as u32;
            }
            b.animation = Some(config);
        }
        if let Some(v) = get_bool_from_value(&value, "tooltip_show") {
            b.tooltip_show = v;
        }
        Ok(b)
    }
    /// Creates a bar chart with custom theme.
    pub fn new_with_theme(
        mut series_list: Vec<Series>,
        x_axis_data: Vec<String>,
        theme: &str,
    ) -> BarChart {
        // bar chart supports line and bar,
        // so sets the index first
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
        BarChart::new_with_theme(series_list, x_axis_data, &get_default_theme_name())
    }
    /// Converts bar chart to svg.
    pub fn svg(&self) -> canvas::Result<String> {
        let mut c = Canvas::new_width_xy(self.width, self.height, self.x, self.y);

        let mut x_axis_height = self.x_axis_height;
        if self.x_axis_hidden {
            x_axis_height = 0.0;
        }
        let axis_top = self.render_header(&mut c);

        let (left_y_axis_values, mut left_y_axis_width) = self.get_y_axis_values(0);
        if self.y_axis_hidden {
            left_y_axis_width = 0.0;
        }
        let mut exist_right_y_axis = false;
        // check the right y axis
        for series in self.series_list.iter() {
            if series.y_axis_index != 0 {
                exist_right_y_axis = true;
            }
        }
        let mut right_y_axis_values = AxisValues::default();
        let mut right_y_axis_width = 0.0_f32;
        if !self.y_axis_hidden && exist_right_y_axis {
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
        // render right y axis
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

        // bar point
        let max_height = c.height() - x_axis_height;
        let mut bar_series_list = vec![];
        let mut line_series_list = vec![];
        // filter line and bar series points
        self.series_list.iter().for_each(|item| {
            if let Some(ref cat) = item.category
                && *cat == SeriesCategory::Line
            {
                line_series_list.push(item);
                return;
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
            self.radius,
            self.animation.as_ref(),
            self.tooltip_show,
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
            None,
            self.tooltip_show,
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

        let mut css = String::new();
        if let Some(ref anim) = self.animation {
            css.push_str(&format!(
                "@keyframes bar-grow{{from{{transform:scaleY(0)}}to{{transform:scaleY(1)}}}} \
                 .bar-anim{{transform-box:fill-box;transform-origin:center bottom;\
                 animation:bar-grow {}ms {} both}} ",
                anim.duration, anim.easing
            ));
        }
        if self.tooltip_show {
            // Hidden hover labels, revealed when the adjacent bar is hovered.
            css.push_str(
                ".ct-tip{opacity:0;pointer-events:none} \
                 .ct-trigger:hover+.ct-tip{opacity:1}",
            );
        }
        if css.is_empty() {
            c.svg()
        } else {
            c.svg_with_style(&css)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::BarChart;
    use crate::{
        Box, LegendCategory, NIL_VALUE, Series, SeriesCategory, THEME_ANT, THEME_DARK,
        THEME_GRAFANA,
    };
    use pretty_assertions::assert_eq;
    #[test]
    fn bar_chart_basic() {
        let mut bar_chart = BarChart::new(
            vec![
                (
                    "Email & Instagram",
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
                "Mon & Mo".to_string(),
                "Tue".to_string(),
                "Wed".to_string(),
                "Thu".to_string(),
                "Fri".to_string(),
                "Sat".to_string(),
                "Sun".to_string(),
            ],
        );
        bar_chart.y_axis_configs[0].axis_width = Some(55.0);
        bar_chart.title_text = "Bar & Line Chart".to_string();
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
        bar_chart.radius = Some(5.0);
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

        #[cfg(feature = "image-encoder")]
        {
            use crate::svg_to_jpeg;
            let buf = svg_to_jpeg(&bar_chart.svg().unwrap()).unwrap();
            std::fs::write("./asset/image/line_mixin.jpeg", buf).unwrap();
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

    // The nullable `Option<f32>` API must render identically to the legacy
    // `NIL_VALUE` sentinel path: `None` is equivalent to a missing point.
    #[test]
    fn bar_chart_nil_value_nullable_api() {
        let n = None;
        let mut bar_chart = BarChart::new(
            vec![
                (
                    "Email",
                    vec![
                        Some(120.0),
                        n,
                        Some(132.0),
                        Some(101.0),
                        Some(134.0),
                        Some(90.0),
                        Some(230.0),
                    ],
                )
                    .into(),
                (
                    "Union Ads",
                    vec![
                        Some(220.0),
                        Some(182.0),
                        Some(191.0),
                        n,
                        Some(290.0),
                        Some(330.0),
                        Some(310.0),
                    ],
                )
                    .into(),
                (
                    "Direct",
                    vec![
                        Some(320.0),
                        Some(332.0),
                        Some(301.0),
                        Some(334.0),
                        Some(390.0),
                        n,
                        Some(320.0),
                    ],
                )
                    .into(),
                Series::new_nullable(
                    "Search Engine".to_string(),
                    vec![
                        n,
                        Some(932.0),
                        Some(901.0),
                        Some(934.0),
                        Some(1290.0),
                        Some(1330.0),
                        Some(1320.0),
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

    #[test]
    fn bar_chart_no_axis() {
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
        bar_chart.x_axis_hidden = true;
        bar_chart.y_axis_hidden = true;
        bar_chart.title_text = "Bar Chart".to_string();
        bar_chart.legend_margin = Some(Box {
            top: 35.0,
            bottom: 10.0,
            ..Default::default()
        });

        assert_eq!(
            include_str!("../../asset/bar_chart/no_axis.svg"),
            bar_chart.svg().unwrap()
        );
    }

    #[test]
    fn bar_chart_custom_label_formatter() {
        let mut bar_chart = BarChart::new(
            vec![("values", vec![0.0, 0.25, 0.5, 0.75, 1.0]).into()],
            vec!["1".into(), "2".into(), "3".into(), "4".into(), "5".into()],
        );

        bar_chart.series_list[0].label_show = true;
        bar_chart.series_label_formatter = "{:.1}".to_string();

        assert_eq!(
            include_str!("../../asset/bar_chart/custom_label_formatter.svg").trim(),
            bar_chart.svg().unwrap()
        );
    }

    #[test]
    fn test_render_legend_center() {
        let bar_chart = BarChart::from_json(
            r###"{
  "legend_align": "center",
  "series_list": [
    {
      "name": "None",
      "label_show": true,
      "data": [
        44
      ]
    },
        {
      "name": "M1 - End of Inception",
      "label_show": true,
      "data": [
        78
      ]
    },
        {
      "name": "M2 - End of Elaboration",
      "label_show": true,
      "data": [
        26
      ]
    },
        {
      "name": "M3 - End of Construction",
      "label_show": true,
      "data": [
        3
      ]
    }
  ],
  "type": "bar",
  "x_axis_data": [
    "Milestones"
  ]
}"###,
        )
        .unwrap();
        assert_eq!(
            include_str!("../../asset/bar_chart/legend_center.svg").trim(),
            bar_chart.svg().unwrap()
        );
    }

    #[test]
    fn bar_chart_animation() {
        let chart = BarChart::from_json(
            r###"{
                "width": 400, "height": 300,
                "series_list": [{"name": "A", "data": [10.0, 20.0, 30.0]}],
                "x_axis_data": ["Mon", "Tue", "Wed"],
                "animation": {"duration": 800, "easing": "ease-out", "delay": 50}
            }"###,
        )
        .unwrap();
        let svg = chart.svg().unwrap();
        assert!(svg.contains("bar-grow"), "missing @keyframes bar-grow");
        assert!(svg.contains(".bar-anim"), "missing .bar-anim class rule");
        assert!(
            svg.contains("transform-origin:center bottom"),
            "missing transform-origin"
        );
        assert!(svg.contains("800ms ease-out"), "missing duration/easing");
        assert!(
            svg.contains(r#"class="bar-anim""#),
            "missing class attr on bar"
        );
        assert!(
            svg.contains("animation-delay:0ms"),
            "missing delay for col 0"
        );
        assert!(
            svg.contains("animation-delay:50ms"),
            "missing delay for col 1"
        );
        assert!(
            svg.contains("animation-delay:100ms"),
            "missing delay for col 2"
        );
    }

    #[test]
    fn line_chart_animation() {
        use super::super::line_chart::LineChart;
        let chart = LineChart::from_json(
            r###"{
                "width": 400, "height": 300,
                "series_list": [
                    {"name": "A", "data": [10.0, 20.0]},
                    {"name": "B", "data": [15.0, 25.0]}
                ],
                "x_axis_data": ["Mon", "Tue"],
                "animation": {"duration": 1200, "easing": "linear", "delay": 200}
            }"###,
        )
        .unwrap();
        let svg = chart.svg().unwrap();
        assert!(svg.contains("line-draw"), "missing @keyframes line-draw");
        assert!(
            svg.contains(".line-anim-0"),
            "missing .line-anim-0 class rule"
        );
        assert!(
            svg.contains(".line-anim-1"),
            "missing .line-anim-1 class rule"
        );
        assert!(
            svg.contains("0ms forwards"),
            "missing 0ms delay for series 0"
        );
        assert!(
            svg.contains("200ms forwards"),
            "missing 200ms delay for series 1"
        );
        assert!(svg.contains("pathLength"), "missing pathLength attribute");
        assert!(
            svg.contains(r#"class="line-anim-0""#),
            "missing class attr on line"
        );
    }

    #[test]
    fn bar_chart_tooltip() {
        let chart = BarChart::from_json(
            r#"{"tooltip_show": true, "series_list": [{"name": "A", "data": [1, 2]}], "x_axis_data": ["x", "y"]}"#,
        )
        .unwrap();
        let svg = chart.svg().unwrap();
        // Native <title> for accessibility.
        assert!(svg.contains("<title>A: 1</title>"), "missing bar title");
        // CSS hover tooltip: trigger class, hidden label, and reveal rule.
        assert!(
            svg.contains(r#"class="ct-trigger""#),
            "missing trigger class"
        );
        assert!(svg.contains(r#"class="ct-tip""#), "missing hover label");
        assert!(
            svg.contains(".ct-trigger:hover+.ct-tip"),
            "missing hover css rule"
        );
        // Off by default: byte-for-byte free of tooltip markup.
        let off = BarChart::from_json(
            r#"{"series_list": [{"name": "A", "data": [1, 2]}], "x_axis_data": ["x", "y"]}"#,
        )
        .unwrap();
        let off_svg = off.svg().unwrap();
        assert!(!off_svg.contains("<title>"));
        assert!(!off_svg.contains("ct-tip"));
        assert!(!off_svg.contains("ct-trigger"));
    }
}
