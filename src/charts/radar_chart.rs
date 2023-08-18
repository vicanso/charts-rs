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

#[derive(Clone, Debug, Default)]
pub struct RadarIndicator {
    pub name: String,
    pub max: f32,
}
impl From<(&str, f32)> for RadarIndicator {
    fn from(val: (&str, f32)) -> Self {
        RadarIndicator {
            name: val.0.to_string(),
            max: val.1,
        }
    }
}

fn get_radar_indicator_list_from_value(value: &serde_json::Value) -> Option<Vec<RadarIndicator>> {
    if let Some(data) = value.get("indicators") {
        if let Some(arr) = data.as_array() {
            let mut indicators = vec![];
            for item in arr.iter() {
                let name = get_string_from_value(item, "name").unwrap_or_default();
                let max = get_f32_from_value(item, "max").unwrap_or_default();
                if !name.is_empty() {
                    indicators.push(RadarIndicator { name, max });
                }
            }
            return Some(indicators);
        }
    }
    None
}

#[derive(Clone, Debug, Default, Chart)]
pub struct RadarChart {
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

    // indicators
    pub indicators: Vec<RadarIndicator>,
}

impl RadarChart {
    /// Creates a radar chart from json.
    pub fn from_json(data: &str) -> canvas::Result<RadarChart> {
        let mut r = RadarChart {
            ..Default::default()
        };
        let data = r.fill_option(data)?;
        if let Some(indicators) = get_radar_indicator_list_from_value(&data) {
            r.indicators = indicators;
        }
        Ok(r)
    }
    /// Creates a radar chart with custom theme.
    pub fn new_with_theme(
        series_list: Vec<Series>,
        indicators: Vec<RadarIndicator>,
        theme: &str,
    ) -> RadarChart {
        let mut r = RadarChart {
            series_list,
            indicators,
            ..Default::default()
        };
        let theme = get_theme(theme);
        r.fill_theme(theme);
        r
    }
    /// Creates a radar chart with default theme.
    pub fn new(series_list: Vec<Series>, indicators: Vec<RadarIndicator>) -> RadarChart {
        RadarChart::new_with_theme(series_list, indicators, &get_default_theme())
    }
    /// Converts bar chart to svg.
    pub fn svg(&self) -> canvas::Result<String> {
        if self.indicators.len() < 3 {
            return Err(canvas::Error::Params {
                message: "The count of indicator should be >= 3".to_string(),
            });
        }
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
        if axis_top > 0.0 {
            c = c.child(Box {
                top: axis_top,
                ..Default::default()
            });
        }

        let mut max_values: Vec<f32> = vec![0.0; self.indicators.len()];
        for series in self.series_list.iter() {
            for (index, item) in series.data.iter().enumerate() {
                if index < max_values.len() && *item > max_values[index] {
                    max_values[index] = *item
                }
            }
        }

        let mut indicators = self.indicators.clone();
        for (index, item) in indicators.iter_mut().enumerate() {
            if item.max < max_values[index] {
                item.max = max_values[index];
            }
        }

        let offset = 40.0;
        let r = c.height() / 2.0 - offset;
        let angle = 360.0 / indicators.len() as f32;
        let cx = c.width() / 2.0;
        let cy = c.height() / 2.0;
        let round_count = 5;
        for i in 1..=round_count {
            let ir = r / round_count as f32 * i as f32;
            let mut points = vec![];
            for index in 0..indicators.len() {
                points.push(get_pie_point(cx, cy, ir, angle * index as f32));
            }
            c.straight_line(StraightLine {
                color: Some(self.grid_stroke_color),
                points,
                stroke_width: self.grid_stroke_width,
                symbol: None,
                close: true,
                ..Default::default()
            });
        }
        for (index, item) in indicators.iter().enumerate() {
            let current_angle = angle * index as f32;
            let p = get_pie_point(cx, cy, r, current_angle);
            let mut x = p.x;
            let mut y = p.y;
            let x_offset = 3.0;
            if let Ok(measurement) = measure_text_width_family(
                &self.font_family,
                self.series_label_font_size,
                &item.name,
            ) {
                if current_angle < 10.0 || (360.0 - current_angle) < 10.0 {
                    y -= 5.0;
                } else if (current_angle - 180.0).abs() < 10.0 {
                    y += measurement.height();
                } else if p.y > cy {
                    let x_angle = if current_angle <= 180.0 {
                        current_angle - 90.0
                    } else {
                        270.0 - current_angle
                    };
                    let y_offset = (x_angle / 180.0).cos() * (measurement.height() / 2.0);
                    y += y_offset;
                }

                if current_angle == 0.0 || current_angle == 180.0 {
                    x -= measurement.width() / 2.0;
                } else if current_angle < 180.0 {
                    x += x_offset;
                } else {
                    x -= measurement.width() + x_offset;
                }
            }
            c.text(Text {
                text: item.name.clone(),
                font_size: Some(self.series_label_font_size),
                font_family: Some(self.font_family.clone()),
                font_color: Some(self.series_label_font_color),
                x: Some(x),
                y: Some(y),
                ..Default::default()
            });
            c.child(Box::default()).line(Line {
                color: Some(self.grid_stroke_color),
                stroke_width: self.grid_stroke_width,
                left: p.x,
                top: p.y,
                right: cx,
                bottom: cy,
            });
        }

        for (index, series) in self.series_list.iter().enumerate() {
            let color = *self
                .series_colors
                .get(series.index.unwrap_or(index))
                .unwrap_or_else(|| &self.series_colors[0]);
            let mut points = vec![];
            for (i, item) in indicators.iter().enumerate() {
                if let Some(value) = series.data.get(i) {
                    let mut ir = if item.max <= 0.0 {
                        0.0
                    } else {
                        *value / item.max * r
                    };
                    if ir > r {
                        ir = r;
                    }
                    let p = get_pie_point(cx, cy, ir, angle * i as f32);
                    points.push(p);
                }
            }
            c.straight_line(StraightLine {
                color: Some(color),
                fill: Some(color.with_alpha(50)),
                points: points.clone(),
                stroke_width: self.series_stroke_width,
                close: true,
                ..Default::default()
            });
        }

        c.svg()
    }
}

#[cfg(test)]
mod tests {
    use super::RadarChart;
    use crate::Series;
    use pretty_assertions::assert_eq;

    #[test]
    fn radar_basic() {
        let radar_chart = RadarChart::new(
            vec![
                (
                    "Allocated Budget",
                    vec![4200.0, 3000.0, 20000.0, 35000.0, 50000.0, 18000.0],
                )
                    .into(),
                (
                    "Actual Spending",
                    vec![5000.0, 14000.0, 28000.0, 26000.0, 42000.0, 21000.0],
                )
                    .into(),
            ],
            vec![
                ("Sales", 6500.0).into(),
                ("Administration", 16000.0).into(),
                ("Information Technology", 30000.0).into(),
                ("Customer Support", 38000.0).into(),
                ("Development", 52000.0).into(),
                ("Marketing", 25000.0).into(),
            ],
        );
        assert_eq!(
            include_str!("../../asset/radar_chart/basic.svg"),
            radar_chart.svg().unwrap()
        );
    }

    #[test]
    fn radar_seven_basic() {
        let radar_chart = RadarChart::new(
            vec![
                Series::new(
                    "Allocated Budget".to_string(),
                    vec![4200.0, 3000.0, 20000.0, 35000.0, 50000.0, 18000.0, 9000.0],
                ),
                Series::new(
                    "Actual Spending".to_string(),
                    vec![5000.0, 14000.0, 28000.0, 26000.0, 42000.0, 21000.0, 7000.0],
                ),
            ],
            vec![
                ("Sales", 6500.0).into(),
                ("Administration", 16000.0).into(),
                ("Information Technology", 30000.0).into(),
                ("Customer Support", 38000.0).into(),
                ("Development", 52000.0).into(),
                ("Marketing", 25000.0).into(),
                ("Online", 10000.0).into(),
            ],
        );

        assert_eq!(
            include_str!("../../asset/radar_chart/seven_points.svg"),
            radar_chart.svg().unwrap()
        );
    }

    #[test]
    fn radar_five_points() {
        let radar_chart = RadarChart::new(
            vec![
                Series::new(
                    "Allocated Budget".to_string(),
                    vec![4200.0, 3000.0, 20000.0, 35000.0, 50000.0],
                ),
                Series::new(
                    "Actual Spending".to_string(),
                    vec![5000.0, 14000.0, 28000.0, 26000.0, 42000.0],
                ),
            ],
            vec![
                ("Sales", 6500.0).into(),
                ("Administration", 16000.0).into(),
                ("Information Technology", 30000.0).into(),
                ("Customer Support", 38000.0).into(),
                ("Development", 52000.0).into(),
            ],
        );

        assert_eq!(
            include_str!("../../asset/radar_chart/five_points.svg"),
            radar_chart.svg().unwrap()
        );
    }

    #[test]
    fn radar_four_points() {
        let radar_chart = RadarChart::new(
            vec![
                Series::new(
                    "Allocated Budget".to_string(),
                    vec![4200.0, 3000.0, 20000.0, 35000.0],
                ),
                Series::new(
                    "Actual Spending".to_string(),
                    vec![5000.0, 14000.0, 28000.0, 26000.0],
                ),
            ],
            vec![
                ("Sales", 6500.0).into(),
                ("Administration", 16000.0).into(),
                ("Information Technology", 30000.0).into(),
                ("Customer Support", 38000.0).into(),
            ],
        );

        assert_eq!(
            include_str!("../../asset/radar_chart/four_points.svg"),
            radar_chart.svg().unwrap()
        );
    }

    #[test]
    fn radar_three_points() {
        let radar_chart = RadarChart::new(
            vec![
                Series::new(
                    "Allocated Budget".to_string(),
                    vec![4200.0, 3000.0, 20000.0],
                ),
                Series::new(
                    "Actual Spending".to_string(),
                    vec![5000.0, 14000.0, 28000.0],
                ),
            ],
            vec![
                ("Sales", 6500.0).into(),
                ("Administration", 16000.0).into(),
                ("Information Technology", 30000.0).into(),
            ],
        );

        assert_eq!(
            include_str!("../../asset/radar_chart/three_points.svg"),
            radar_chart.svg().unwrap()
        );
    }
}
