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
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, Default, Chart)]
pub struct CandlestickChart {
    pub width: f32,
    pub height: f32,
    pub x: f32,
    pub y: f32,
    pub margin: Box,
    // [open price1, close price1, lowest price1, highest price1, open price2, close price2, ...]
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
    fn fill_default(&mut self) {
        if self.candlestick_up_color.is_zero() {
            self.candlestick_up_color = (236, 0, 0).into();
        }
        if self.candlestick_up_border_color.is_zero() {
            self.candlestick_up_border_color = (138, 0, 0).into();
        }
        if self.candlestick_down_color.is_zero() {
            self.candlestick_down_color = (0, 218, 60).into();
        }
        if self.candlestick_down_border_color.is_zero() {
            self.candlestick_down_border_color = (0, 143, 40).into();
        }
    }
    /// Creates a candlestick chart from json.
    pub fn from_json(data: &str) -> canvas::Result<CandlestickChart> {
        let mut c = CandlestickChart {
            ..Default::default()
        };
        let value = c.fill_option(data)?;
        if let Some(value) = get_color_from_value(&value, "candlestick_up_color") {
            c.candlestick_up_color = value;
        }
        if let Some(value) = get_color_from_value(&value, "candlestick_up_border_color") {
            c.candlestick_up_border_color = value;
        }
        if let Some(value) = get_color_from_value(&value, "candlestick_down_color") {
            c.candlestick_down_color = value;
        }
        if let Some(value) = get_color_from_value(&value, "candlestick_down_border_color") {
            c.candlestick_down_border_color = value;
        }
        c.fill_default();
        Ok(c)
    }
    /// Creates a candlestick chart with custom theme.
    pub fn new_with_theme(
        mut series_list: Vec<Series>,
        x_axis_data: Vec<String>,
        theme: &str,
    ) -> CandlestickChart {
        // set the index of series
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
        c.fill_default();
        c
    }
    /// Creates a candlestick chart with default theme.
    pub fn new(series_list: Vec<Series>, x_axis_data: Vec<String>) -> CandlestickChart {
        CandlestickChart::new_with_theme(series_list, x_axis_data, &get_default_theme())
    }
    /// Converts candlestick chart to svg.
    pub fn svg(&self) -> canvas::Result<String> {
        let mut c = Canvas::new_width_xy(self.width, self.height, self.x, self.y);

        self.render_background(c.child(Box::default()));
        c.margin = self.margin.clone();

        let title_height = self.render_title(c.child(Box::default()));

        let legend_height = self.render_legend(c.child(Box::default()));
        // get the max height of title and legend
        let axis_top = if legend_height > title_height {
            legend_height
        } else {
            title_height
        };

        let (left_y_axis_values, left_y_axis_width) = self.get_y_axis_values(0);

        let axis_height = c.height() - self.x_axis_height - axis_top;
        let axis_width = c.width() - left_y_axis_width;
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
            // split the series point to chunk
            // [open, close, lowest, highest]
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
                // fall
                if chunk[0] > chunk[1] {
                    fill = self.candlestick_down_color;
                    border_color = self.candlestick_down_border_color;
                } else if chunk[0] == chunk[1] {
                    border_color = Color::transparent();
                }

                let line_left = half_chunk_width + chunk_width * index as f32 - 1.0;
                c.child(Box {
                    left: left_y_axis_width,
                    ..Default::default()
                })
                .line(Line {
                    color: Some(fill),
                    stroke_width: 1.0,
                    left: line_left,
                    top: lowest.min(highest),
                    right: line_left,
                    bottom: lowest.max(highest),
                    ..Default::default()
                });

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
            }
        }
        let mut line_series_list = vec![];
        self.series_list.iter().for_each(|item| {
            if let Some(ref cat) = item.category {
                if *cat == SeriesCategory::Line {
                    line_series_list.push(item);
                }
            }
        });

        let y_axis_values_list = vec![&left_y_axis_values];
        let max_height = c.height() - self.x_axis_height;
        let line_series_labels_list = self.render_line(
            c.child(Box {
                left: left_y_axis_width,
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
                ..Default::default()
            }),
            line_series_labels_list,
        );

        c.svg()
    }
}

#[cfg(test)]
mod tests {
    use super::CandlestickChart;
    use crate::SeriesCategory;
    use pretty_assertions::assert_eq;
    #[test]
    fn candlestick_chart_basic() {
        let candlestick_chart = CandlestickChart::new(
            vec![(
                "",
                vec![
                    20.0, 34.0, 10.0, 38.0, 40.0, 35.0, 30.0, 50.0, 31.0, 38.0, 33.0, 44.0, 38.0,
                    15.0, 5.0, 42.0,
                ],
            )
                .into()],
            vec![
                "2017-10-24".to_string(),
                "2017-10-25".to_string(),
                "2017-10-26".to_string(),
                "2017-10-27".to_string(),
            ],
        );
        assert_eq!(
            include_str!("../../asset/candlestick_chart/basic.svg"),
            candlestick_chart.svg().unwrap()
        );
    }
    #[test]
    fn candlestick_chart_sh() {
        let mut candlestick_chart = CandlestickChart::new(
            vec![
                // start at sixth point
                (
                    "MA5",
                    vec![
                        2352.93, 2378.48, 2394.81, 2409.64, 2420.04, 2426.66, 2429.33, 2428.01,
                        2417.97, 2410.51, 2391.99, 2368.35, 2349.20, 2331.29, 2314.49, 2322.42,
                        2331.49, 2321.01, 2327.60, 2334.39, 2326.13, 2317.95, 2325.39, 2317.45,
                        2300.81, 2290.01, 2281.96, 2267.85, 2262.02, 2272.7, 2283.49, 2293.46,
                        2310.80, 2318.85, 2315.63, 2298.04, 2279.71, 2261.25, 2247.26, 2232.06,
                        2227.12, 2224.95, 2223.30, 2221.66, 2217.96, 2212.03, 2205.85, 2199.38,
                        2194.99, 2202.56, 2214.61, 2212.55, 2217.45, 2217.79, 2204.45,
                    ],
                )
                    .into(),
                (
                    "日K",
                    vec![
                        2320.26, 2320.26, 2287.3, 2362.94, 2300.0, 2291.3, 2288.26, 2308.38,
                        2295.35, 2346.5, 2295.35, 2346.92, 2347.22, 2358.98, 2337.35, 2363.8,
                        2360.75, 2382.48, 2347.89, 2383.76, 2383.43, 2385.42, 2371.23, 2391.82,
                        2377.41, 2419.02, 2369.57, 2421.15, 2425.92, 2428.15, 2417.58, 2440.38,
                        2411.0, 2433.13, 2403.3, 2437.42, 2432.68, 2434.48, 2427.7, 2441.73,
                        2430.69, 2418.53, 2394.22, 2433.89, 2416.62, 2432.4, 2414.4, 2443.03,
                        2441.91, 2421.56, 2415.43, 2444.8, 2420.26, 2382.91, 2373.53, 2427.07,
                        2383.49, 2397.18, 2370.61, 2397.94, 2378.82, 2325.95, 2309.17, 2378.82,
                        2322.94, 2314.16, 2308.76, 2330.88, 2320.62, 2325.82, 2315.01, 2338.78,
                        2313.74, 2293.34, 2289.89, 2340.71, 2297.77, 2313.22, 2292.03, 2324.63,
                        2322.32, 2365.59, 2308.92, 2366.16, 2364.54, 2359.51, 2330.86, 2369.65,
                        2332.08, 2273.4, 2259.25, 2333.54, 2274.81, 2326.31, 2270.1, 2328.14,
                        2333.61, 2347.18, 2321.6, 2351.44, 2340.44, 2324.29, 2304.27, 2352.02,
                        2326.42, 2318.61, 2314.59, 2333.67, 2314.68, 2310.59, 2296.58, 2320.96,
                        2309.16, 2286.6, 2264.83, 2333.29, 2282.17, 2263.97, 2253.25, 2286.33,
                        2255.77, 2270.28, 2253.31, 2276.22, 2269.31, 2278.4, 2250.0, 2312.08,
                        2267.29, 2240.02, 2239.21, 2276.05, 2244.26, 2257.43, 2232.02, 2261.31,
                        2257.74, 2317.37, 2257.42, 2317.86, 2318.21, 2324.24, 2311.6, 2330.81,
                        2321.4, 2328.28, 2314.97, 2332.0, 2334.74, 2326.72, 2319.91, 2344.89,
                        2318.58, 2297.67, 2281.12, 2319.99, 2299.38, 2301.26, 2289.0, 2323.48,
                        2273.55, 2236.3, 2232.91, 2273.55, 2238.49, 2236.62, 2228.81, 2246.87,
                        2229.46, 2234.4, 2227.31, 2243.95, 2234.9, 2227.74, 2220.44, 2253.42,
                        2232.69, 2225.29, 2217.25, 2241.34, 2196.24, 2211.59, 2180.67, 2212.59,
                        2215.47, 2225.77, 2215.47, 2234.73, 2224.93, 2226.13, 2212.56, 2233.04,
                        2236.98, 2219.55, 2217.26, 2242.48, 2218.09, 2206.78, 2204.44, 2226.26,
                        2199.91, 2181.94, 2177.39, 2204.99, 2169.63, 2194.85, 2165.78, 2196.43,
                        2195.03, 2193.8, 2178.47, 2197.51, 2181.82, 2197.6, 2175.44, 2206.03,
                        2201.12, 2244.64, 2200.58, 2250.11, 2236.4, 2242.17, 2232.26, 2245.12,
                        2242.62, 2184.54, 2182.81, 2242.62, 2187.35, 2218.32, 2184.11, 2226.12,
                        2213.19, 2199.31, 2191.85, 2224.63, 2203.89, 2177.91, 2173.86, 2210.58,
                    ],
                )
                    .into(),
            ],
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
                "2013/3/1".to_string(),
                "2013/3/4".to_string(),
                "2013/3/5".to_string(),
                "2013/3/6".to_string(),
                "2013/3/7".to_string(),
                "2013/3/8".to_string(),
                "2013/3/11".to_string(),
                "2013/3/12".to_string(),
                "2013/3/13".to_string(),
                "2013/3/14".to_string(),
                "2013/3/15".to_string(),
                "2013/3/18".to_string(),
                "2013/3/18".to_string(),
                "2013/3/20".to_string(),
                "2013/3/21".to_string(),
                "2013/3/22".to_string(),
                "2013/3/25".to_string(),
                "2013/3/26".to_string(),
                "2013/3/27".to_string(),
                "2013/3/28".to_string(),
                "2013/3/29".to_string(),
                "2013/4/1".to_string(),
                "2013/4/2".to_string(),
                "2013/4/3".to_string(),
                "2013/4/8".to_string(),
                "2013/4/9".to_string(),
                "2013/4/10".to_string(),
                "2013/4/11".to_string(),
                "2013/4/12".to_string(),
                "2013/4/15".to_string(),
                "2013/4/16".to_string(),
                "2013/4/17".to_string(),
                "2013/4/18".to_string(),
                "2013/4/19".to_string(),
                "2013/4/22".to_string(),
                "2013/4/23".to_string(),
                "2013/4/24".to_string(),
                "2013/4/25".to_string(),
                "2013/4/26".to_string(),
            ],
        );
        candlestick_chart.series_list[0].category = Some(SeriesCategory::Line);
        candlestick_chart.series_list[0].start_index = 5;
        candlestick_chart.y_axis_configs[0].axis_min = Some(2100.0);
        candlestick_chart.y_axis_configs[0].axis_max = Some(2460.0);
        candlestick_chart.y_axis_configs[0].axis_formatter = Some("{t}".to_string());
        assert_eq!(
            include_str!("../../asset/candlestick_chart/sh.svg"),
            candlestick_chart.svg().unwrap()
        );
    }
}
