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
pub struct CandlestickChart {
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

    pub candlestick_up_color: Color,
    pub candlestick_up_border_color: Color,
    pub candlestick_down_color: Color,
    pub candlestick_down_border_color: Color,
}

impl CandlestickChart {
    /// New a candlestick chart with custom theme
    pub fn new_with_theme(
        mut series_list: Vec<Series>,
        x_axis_data: Vec<String>,
        theme: &str,
    ) -> CandlestickChart {
        // candlestick data: open, close, lowest, highest
        // 因此先计算index
        series_list
            .iter_mut()
            .enumerate()
            .for_each(|(index, item)| {
                item.index = Some(index);
            });
        let mut c = CandlestickChart {
            series_list,
            x_axis_data,
            ..Default::default()
        };
        let theme = get_theme(theme);
        c.fill_theme(theme);
        c.candlestick_up_color = (236, 0, 0).into();
        c.candlestick_up_border_color = (138, 0, 0).into();
        c.candlestick_down_color = (0, 218, 60).into();
        c.candlestick_down_border_color = (0, 143, 40).into();
        c
    }
    /// New a candlestick chart with default theme.
    pub fn new(series_list: Vec<Series>, x_axis_data: Vec<String>) -> CandlestickChart {
        CandlestickChart::new_with_theme(series_list, x_axis_data, &get_default_theme())
    }
    /// Converts candlestick chart to svg.
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

        let axis_height = c.height() - self.x_axis_height - axis_top;
        let axis_width = c.width() - left_y_axis_width;
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

        // x axis
        self.render_x_axis(
            c.child(Box {
                top: c.height() - self.x_axis_height,
                left: left_y_axis_width,
                ..Default::default()
            }),
            self.x_axis_data.clone(),
            axis_width,
        );
        let chunk_width = axis_width / self.x_axis_data.len() as f32;
        let half_chunk_width = chunk_width / 2.0;
        for series in self.series_list.iter() {
            if series.category.is_some() {
                continue;
            }
            let chunks = series.data.chunks(4);

            for (index, chunk) in chunks.enumerate() {
                if chunk.len() != 4 {
                    continue;
                }

                let open = left_y_axis_values.get_offset_height(chunk[0], axis_height);
                let close = left_y_axis_values.get_offset_height(chunk[1], axis_height);
                let lowest = left_y_axis_values.get_offset_height(chunk[2], axis_height);
                let highest = left_y_axis_values.get_offset_height(chunk[3], axis_height);
                let mut fill = self.candlestick_up_color;
                let mut border_color = self.candlestick_up_border_color;
                // 跌
                if chunk[0] > chunk[1] {
                    fill = self.candlestick_down_color;
                    border_color = self.candlestick_down_border_color;
                } else if chunk[0] == chunk[1] {
                    border_color = Color::transparent();
                }

                c.child(Box {
                    left: left_y_axis_width,
                    ..Default::default()
                })
                .rect(Rect {
                    color: Some(border_color),
                    fill: Some(fill),
                    left: half_chunk_width / 2.0 + chunk_width * index as f32 - 1.0,
                    top: open.min(close),
                    width: half_chunk_width,
                    height: (open.max(close) - open.min(close)).max(1.0),
                    ..Default::default()
                });
                c.child(Box {
                    left: left_y_axis_width,
                    ..Default::default()
                })
                .line(Line {
                    color: Some(fill),
                    stroke_width: 1.0,
                    left: half_chunk_width + chunk_width * index as f32,
                    top: lowest.min(highest),
                    right: half_chunk_width + chunk_width * index as f32,
                    bottom: lowest.max(highest),
                });
            }
        }

        c.svg()
    }
}

#[cfg(test)]
mod tests {
    use super::CandlestickChart;
    use pretty_assertions::assert_eq;
    #[test]
    fn candlestick_chart_basic() {
        let mut candlestick_chart = CandlestickChart::new(
            vec![(
                "",
                vec![
                    2320.26, 2320.26, 2287.3, 2362.94, 2300.0, 2291.3, 2288.26, 2308.38, 2295.35,
                    2346.5, 2295.35, 2346.92, 2347.22, 2358.98, 2337.35, 2363.8, 2360.75, 2382.48,
                    2347.89, 2383.76, 2383.43, 2385.42, 2371.23, 2391.82, 2377.41, 2419.02,
                    2369.57, 2421.15, 2425.92, 2428.15, 2417.58, 2440.38, 2411.0, 2433.13, 2403.3,
                    2437.42, 2432.68, 2434.48, 2427.7, 2441.73, 2430.69, 2418.53, 2394.22, 2433.89,
                    2416.62, 2432.4, 2414.4, 2443.03, 2441.91, 2421.56, 2415.43, 2444.8, 2420.26,
                    2382.91, 2373.53, 2427.07, 2383.49, 2397.18, 2370.61, 2397.94, 2378.82,
                    2325.95, 2309.17, 2378.82, 2322.94, 2314.16, 2308.76, 2330.88, 2320.62,
                    2325.82, 2315.01, 2338.78, 2313.74, 2293.34, 2289.89, 2340.71, 2297.77,
                    2313.22, 2292.03, 2324.63, 2322.32, 2365.59, 2308.92, 2366.16,
                ],
            )
                .into()],
            vec![
                "2013/1/24".to_string(),
                "2013/1/25".to_string(),
                "2013/1/28".to_string(),
                "2013/1/29".to_string(),
                "2013/1/30".to_string(),
                "2013/1/31".to_string(),
                "2013/2/1".to_string(),
                "2013/2/4".to_string(),
                "2013/2/5".to_string(),
                "2013/2/6".to_string(),
                "2013/2/7".to_string(),
                "2013/2/8".to_string(),
                "2013/2/18".to_string(),
                "2013/2/19".to_string(),
                "2013/2/20".to_string(),
                "2013/2/21".to_string(),
                "2013/2/22".to_string(),
                "2013/2/25".to_string(),
                "2013/2/26".to_string(),
                "2013/2/27".to_string(),
                "2013/2/28".to_string(),
            ],
        );
        candlestick_chart.y_axis_configs[0].axis_min = Some(2200.0);
        candlestick_chart.y_axis_configs[0].axis_max = Some(2500.0);
        candlestick_chart.y_axis_configs[0].axis_formatter = Some("{t}".to_string());
        println!("{}", candlestick_chart.svg().unwrap());
        assert_eq!(
            include_str!("../../asset/candlestick_chart/basic.svg"),
            candlestick_chart.svg().unwrap()
        );
    }
}
