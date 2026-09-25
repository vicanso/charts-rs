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

use super::base::ChartBase;
use super::canvas;
use super::common::*;
use super::component::*;
use super::theme::{get_default_theme_name, get_theme};

/// A line chart. Supports smooth curves, area fill, stacking and mark
/// points/lines.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct LineChart {
    /// The shared chart options (size, series, title/legend, axes); exposed
    /// directly on the chart through `Deref`, e.g. `chart.title_text`.
    pub base: ChartBase,
    // x axis

    // y axis
    /// Y axis configurations; one per axis, up to two.
    pub y_axis_configs: Vec<YAxisConfig>,
    // grid

    // series
}

impl std::ops::Deref for LineChart {
    type Target = ChartBase;
    fn deref(&self) -> &ChartBase {
        &self.base
    }
}
impl std::ops::DerefMut for LineChart {
    fn deref_mut(&mut self) -> &mut ChartBase {
        &mut self.base
    }
}

impl LineChart {
    /// Creates a line chart from json.
    pub fn from_json(data: &str) -> canvas::Result<LineChart> {
        let mut l = LineChart {
            ..Default::default()
        };
        l.base.fill_option(data, &mut l.y_axis_configs, &[])?;
        Ok(l)
    }
    /// Creates a line chart with custom theme.
    pub fn new_with_theme(
        series_list: Vec<Series>,
        x_axis_data: Vec<String>,
        theme: &str,
    ) -> LineChart {
        let mut l = LineChart {
            ..Default::default()
        };
        l.series_list = series_list;
        l.x_axis_data = x_axis_data;
        let theme = get_theme(theme);
        l.base.fill_theme(theme, &mut l.y_axis_configs);
        l
    }
    /// Creates a line chart with default theme.
    pub fn new(series_list: Vec<Series>, x_axis_data: Vec<String>) -> LineChart {
        LineChart::new_with_theme(series_list, x_axis_data, &get_default_theme_name())
    }
    /// Converts line chart to svg.
    pub fn svg(&self) -> canvas::Result<String> {
        let c = self.new_canvas();
        let layout = self.layout_cartesian(c, &self.y_axis_configs);
        let c = layout.canvas.clone();
        let axis_height = layout.axis_height;

        // line point
        let y_axis_values_list = layout.y_axis_values();
        let max_height = layout.max_height;
        let line_series_list: Vec<&Series> = self.series_list.iter().collect();
        let series_labels_list = self.render_line(
            layout.plot(),
            &line_series_list,
            &y_axis_values_list,
            max_height,
            axis_height,
            self.x_axis_data.len(),
            self.animation.as_ref(),
            self.tooltip_show,
        );
        self.render_series_label(layout.plot(), series_labels_list);

        self.render_mark_line(
            layout.plot(),
            &line_series_list,
            &y_axis_values_list,
            max_height,
        );

        let mut css = String::new();
        if let Some(ref anim) = self.animation {
            let series_count = self.series_list.len();
            css.push_str("@keyframes line-draw{from{stroke-dashoffset:1}to{stroke-dashoffset:0}}");
            for i in 0..series_count {
                let delay = i as u32 * anim.delay;
                css.push_str(&format!(
                    " .line-anim-{}{{stroke-dasharray:1;stroke-dashoffset:1;\
                     animation:line-draw {}ms {} {}ms forwards}}",
                    i,
                    anim.duration,
                    anim.safe_easing(),
                    delay
                ));
            }
        }
        if self.tooltip_show {
            if !css.is_empty() {
                css.push(' ');
            }
            css.push_str(TOOLTIP_STYLE);
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
    use super::LineChart;
    use crate::{Align, Box, MarkLine, MarkLineCategory, MarkPoint, MarkPointCategory, NIL_VALUE};
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
        assert_snapshot!("line_chart/basic.svg", line_chart.svg().unwrap());
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
        assert_snapshot!("line_chart/nil_value.svg", line_chart.svg().unwrap());
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
        assert_snapshot!("line_chart/boundary_gap.svg", line_chart.svg().unwrap());
    }
    #[test]
    fn line_chart_fill() {
        let mut line_chart = LineChart::new(
            vec![
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
        line_chart.series_fill = true;
        line_chart.series_smooth = true;
        line_chart.title_text = "Stacked Area Chart".to_string();
        line_chart.sub_title_text = "Hello World".to_string();
        line_chart.legend_margin = Some(Box {
            top: 50.0,
            bottom: 10.0,
            ..Default::default()
        });
        assert_snapshot!("line_chart/smooth_fill.svg", line_chart.svg().unwrap());
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
        assert_snapshot!(
            "line_chart/legend_align_right.svg",
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

        assert_snapshot!("line_chart/two_y_axis.svg", line_chart.svg().unwrap());
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
        assert_snapshot!(
            "line_chart/value_count_unequal.svg",
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
        assert_snapshot!("line_chart/no_axis.svg", line_chart.svg().unwrap());
    }

    #[test]
    fn line_chart_small_value() {
        let mut line_chart = LineChart::new(
            vec![
                (
                    "latency",
                    vec![
                        1.12, 1.18, 1.65, 1.87, 1.92, 1.43, 1.65, 0.83, 0.65, 0.12, 1.1, 0.87,
                    ],
                )
                    .into(),
            ],
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
        assert_snapshot!("line_chart/small_value.svg", line_chart.svg().unwrap());
    }

    #[test]
    fn line_chart_tooltip() {
        let chart = LineChart::from_json(
            r#"{"tooltip_show": true, "series_list": [{"name": "A", "data": [1, 2]}], "x_axis_data": ["x", "y"]}"#,
        )
        .unwrap();
        let svg = chart.svg().unwrap();
        assert!(svg.contains("<title>A: 1</title>"), "missing line title");
        assert!(svg.contains(r#"class="ct-tip""#), "missing hover label");
        assert!(
            svg.contains(".ct-trigger:hover+.ct-tip"),
            "missing hover css"
        );
        let off = LineChart::from_json(
            r#"{"series_list": [{"name": "A", "data": [1, 2]}], "x_axis_data": ["x", "y"]}"#,
        )
        .unwrap();
        let off_svg = off.svg().unwrap();
        assert!(!off_svg.contains("<title>"));
        assert!(!off_svg.contains("ct-tip"));
    }

    // Empty data + a mark line exercises three guards at once: the mark-line
    // average (`sum / 0`), the x-axis split (`axis_length / 0`), and the y-axis
    // value range (all-sentinel min/max). None may leak NaN/inf.
    #[test]
    fn empty_data_and_mark_line_no_nan() {
        let chart = LineChart::from_json(
            r#"{"series_list":[{"name":"A","data":[],"mark_lines":[{"category":"average"}]}],"x_axis_data":[]}"#,
        )
        .unwrap();
        let svg = chart.svg().unwrap();
        assert!(!svg.contains("NaN"), "empty data must not emit NaN");
        assert!(!svg.contains("inf"), "empty data must not emit inf");
    }
}
