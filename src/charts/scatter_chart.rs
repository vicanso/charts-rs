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
pub struct ScatterChart {
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
    pub x_axis_config: YAxisConfig,
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

    // symbol
    pub series_symbol_sizes: Vec<f32>,
}

impl ScatterChart {
    pub fn from_json(data: &str) -> canvas::Result<ScatterChart> {
        let mut s = ScatterChart {
            ..Default::default()
        };
        let value = s.fill_option(data)?;
        s.fill_default();

        if let Some(series_symbol_sizes) = get_f32_slice_from_value(&value, "series_symbol_sizes") {
            s.series_symbol_sizes = series_symbol_sizes;
        }
        let theme = get_string_from_value(&value, "theme").unwrap_or_default();
        if let Some(x_axis_config) = value.get("x_axis_config") {
            s.x_axis_config = get_y_axis_config_from_value(&get_theme(&theme), x_axis_config);
        }
        Ok(s)
    }
    /// New a scatter chart with  theme.
    pub fn new_with_theme(series_list: Vec<Series>, theme: &str) -> ScatterChart {
        let mut s = ScatterChart {
            series_list,
            ..Default::default()
        };
        let theme = get_theme(theme);
        s.fill_theme(theme);
        s.fill_default();

        s
    }
    pub fn fill_default(&mut self) {
        if self.y_axis_configs[0].axis_stroke_color.is_zero() {
            self.y_axis_configs[0].axis_stroke_color = self.x_axis_stroke_color;
        }
        if self.x_axis_config.axis_split_number == 0 {
            self.x_axis_config = self.y_axis_configs[0].clone();
        }
        self.x_boundary_gap = Some(false);
    }
    /// New a scatter chart with default theme.
    pub fn new(series_list: Vec<Series>) -> ScatterChart {
        ScatterChart::new_with_theme(series_list, &get_default_theme())
    }
    /// Converts scatter chart to svg.
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

        let y_axis_config = self.get_y_axis_config(0);

        let mut y_axis_data_list = vec![];
        let mut x_axis_data_list = vec![];
        for series in self.series_list.iter() {
            for (index, data) in series.data.iter().enumerate() {
                if index % 2 == 0 {
                    x_axis_data_list.push(*data);
                } else {
                    y_axis_data_list.push(*data);
                }
            }
        }
        let y_axis_values = get_axis_values(AxisValueParams {
            data_list: y_axis_data_list,
            split_number: y_axis_config.axis_split_number,
            reverse: Some(true),
            min: y_axis_config.axis_min,
            max: y_axis_config.axis_max,
            thousands_format: false,
        });
        let y_axis_width = if let Some(value) = y_axis_config.axis_width {
            value
        } else {
            let y_axis_formatter = &y_axis_config.axis_formatter.clone().unwrap_or_default();
            let str = format_string(&y_axis_values.data[0], y_axis_formatter);
            if let Ok(b) =
                measure_text_width_family(&self.font_family, y_axis_config.axis_font_size, &str)
            {
                b.width() + 5.0
            } else {
                DEFAULT_Y_AXIS_WIDTH
            }
        };

        let axis_height = c.height() - self.x_axis_height - axis_top;
        let axis_width = c.width() - y_axis_width;
        // 减去顶部文本区域
        if axis_top > 0.0 {
            c = c.child(Box {
                top: axis_top,
                ..Default::default()
            });
        }

        // grid
        self.render_grid(
            c.child(Box {
                left: y_axis_width,
                ..Default::default()
            }),
            axis_width,
            axis_height,
        );
        let x_axis_width = c.width() - y_axis_width;
        c.child(Box {
            left: y_axis_width,
            ..Default::default()
        })
        .grid(Grid {
            right: x_axis_width,
            bottom: axis_height,
            color: Some(self.grid_stroke_color),
            stroke_width: self.grid_stroke_width,
            verticals: y_axis_config.axis_split_number,
            hidden_verticals: vec![0],
            ..Default::default()
        });

        // y axis
        self.render_y_axis(
            c.child(Box::default()),
            y_axis_values.data.clone(),
            axis_height,
            y_axis_width,
            0,
        );

        // x axis
        let x_axis_values = get_axis_values(AxisValueParams {
            data_list: x_axis_data_list,
            split_number: self.x_axis_config.axis_split_number,
            min: self.x_axis_config.axis_min,
            max: self.x_axis_config.axis_max,
            ..Default::default()
        });
        let x_axis_formatter = &self
            .x_axis_config
            .axis_formatter
            .clone()
            .unwrap_or_default();
        let content_width = c.width() - y_axis_width;
        let content_height = axis_height;
        self.render_x_axis(
            c.child(Box {
                top: c.height() - self.x_axis_height,
                left: y_axis_width,
                ..Default::default()
            }),
            x_axis_values
                .data
                .iter()
                .map(|item| format_string(item, x_axis_formatter))
                .collect(),
            axis_width,
        );

        // render dot
        let mut content_canvas = c.child(Box {
            left: y_axis_width,
            ..Default::default()
        });
        let default_symbol_size = 10.0_f32;
        for (index, series) in self.series_list.iter().enumerate() {
            let mut color = *self
                .series_colors
                .get(series.index.unwrap_or(index))
                .unwrap_or_else(|| &self.series_colors[0]);
            let symbol_size = self
                .series_symbol_sizes
                .get(series.index.unwrap_or(index))
                .unwrap_or(&default_symbol_size);
            color = color.with_alpha(210);
            for chunk in series.data.chunks(2) {
                if chunk.len() != 2 {
                    continue;
                }
                let x = content_width - x_axis_values.get_offset_height(chunk[0], content_width);
                let y = y_axis_values.get_offset_height(chunk[1], content_height);
                content_canvas.circle(Circle {
                    fill: Some(color),
                    cx: x,
                    cy: y,
                    r: *symbol_size,
                    ..Default::default()
                });
            }
        }

        c.svg()
    }
}

#[cfg(test)]
mod tests {
    use super::ScatterChart;
    use crate::Align;
    use pretty_assertions::assert_eq;
    #[test]
    fn scatter_chart_basic() {
        let mut scatter_chart = ScatterChart::new(vec![
            (
                "Female",
                vec![
                    161.2, 51.6, 167.5, 59.0, 159.5, 49.2, 157.0, 63.0, 155.8, 53.6, 170.0, 59.0,
                    159.1, 47.6, 166.0, 69.8, 176.2, 66.8, 160.2, 75.2, 172.5, 55.2, 170.9, 54.2,
                    172.9, 62.5, 153.4, 42.0, 160.0, 50.0, 147.2, 49.8, 168.2, 49.2, 175.0, 73.2,
                    157.0, 47.8, 167.6, 68.8, 159.5, 50.6, 175.0, 82.5, 166.8, 57.2, 176.5, 87.8,
                    170.2, 72.8,
                ],
            )
                .into(),
            (
                "Male",
                vec![
                    174.0, 65.6, 175.3, 71.8, 193.5, 80.7, 186.5, 72.6, 187.2, 78.8, 181.5, 74.8,
                    184.0, 86.4, 184.5, 78.4, 175.0, 62.0, 184.0, 81.6, 180.0, 76.6, 177.8, 83.6,
                    192.0, 90.0, 176.0, 74.6, 174.0, 71.0, 184.0, 79.6, 192.7, 93.8, 171.5, 70.0,
                    173.0, 72.4, 176.0, 85.9, 176.0, 78.8, 180.5, 77.8, 172.7, 66.2, 176.0, 86.4,
                    173.5, 81.8,
                ],
            )
                .into(),
        ]);

        scatter_chart.title_text = "Male and female height and weight distribution".to_string();
        scatter_chart.margin.right = 20.0;
        scatter_chart.title_align = Align::Left;
        scatter_chart.sub_title_text = "Data from: Heinz 2003".to_string();
        scatter_chart.sub_title_align = Align::Left;
        scatter_chart.legend_align = Align::Right;
        scatter_chart.y_axis_configs[0].axis_min = Some(40.0);
        scatter_chart.y_axis_configs[0].axis_max = Some(130.0);
        scatter_chart.y_axis_configs[0].axis_formatter = Some("{c} kg".to_string());

        scatter_chart.x_axis_config.axis_min = Some(140.0);
        scatter_chart.x_axis_config.axis_max = Some(230.0);
        scatter_chart.x_axis_config.axis_formatter = Some("{c} cm".to_string());

        scatter_chart.series_symbol_sizes = vec![6.0, 6.0];

        assert_eq!(
            include_str!("../../asset/scatter_chart/basic.svg"),
            scatter_chart.svg().unwrap()
        );
    }
}
