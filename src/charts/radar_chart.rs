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
use super::base::ChartBase;
use super::canvas;
use super::color::*;
use super::common::*;
use super::component::*;
use super::params::*;
use super::theme::{get_default_theme_name, get_theme};
use super::util::*;
use crate::charts::measure_text_width_family;

/// One radar axis: its name and maximum value.
#[derive(Clone, Debug, Default)]
pub struct RadarIndicator {
    /// Name of the indicator axis.
    pub name: String,
    /// Maximum value of the indicator.
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
    if let Some(data) = value.get("indicators")
        && let Some(arr) = data.as_array()
    {
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
    None
}

/// A radar chart plotting each series against a ring of indicators.
#[derive(Clone, Debug, Default)]
pub struct RadarChart {
    /// The shared chart options (size, series, title/legend, axes); exposed
    /// directly on the chart through `Deref`, e.g. `chart.title_text`.
    pub base: ChartBase,
    // x axis

    // y axis
    /// Y axis configurations; one per axis, up to two.
    pub y_axis_configs: Vec<YAxisConfig>,

    // grid

    // series

    // indicators
    /// The indicator axes of the radar.
    pub indicators: Vec<RadarIndicator>,
}

impl std::ops::Deref for RadarChart {
    type Target = ChartBase;
    fn deref(&self) -> &ChartBase {
        &self.base
    }
}
impl std::ops::DerefMut for RadarChart {
    fn deref_mut(&mut self) -> &mut ChartBase {
        &mut self.base
    }
}

impl RadarChart {
    /// Creates a radar chart from json.
    pub fn from_json(data: &str) -> canvas::Result<RadarChart> {
        let mut r = RadarChart {
            ..Default::default()
        };
        let data = r.base.fill_option(data, &mut r.y_axis_configs)?;
        if let Some(indicators) = get_radar_indicator_list_from_value(&data) {
            r.indicators = indicators;
        }
        if data.get("series_fill").is_none() {
            r.series_fill = true;
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
            indicators,
            ..Default::default()
        };
        r.series_list = series_list;
        r.series_fill = true;
        let theme = get_theme(theme);
        r.base.fill_theme(theme, &mut r.y_axis_configs);
        r
    }
    /// Creates a radar chart with default theme.
    pub fn new(series_list: Vec<Series>, indicators: Vec<RadarIndicator>) -> RadarChart {
        RadarChart::new_with_theme(series_list, indicators, &get_default_theme_name())
    }
    /// Converts bar chart to svg.
    pub fn svg(&self) -> canvas::Result<String> {
        if self.indicators.len() < 3 {
            return Err(canvas::Error::Params {
                message: "The count of indicator should be >= 3".to_string(),
            });
        }
        let mut c = Canvas::new_width_xy(self.width, self.height, self.x, self.y);

        let axis_top = self.render_header(&mut c);
        if axis_top > 0.0 {
            c = c.child(Box {
                top: axis_top,
                ..Default::default()
            });
        }

        let mut max_values: Vec<f32> = vec![0.0; self.indicators.len()];
        for series in self.series_list.iter() {
            for (index, item) in series.data_values().iter().enumerate() {
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
                ..Default::default()
            });
        }

        let mut label_positions = vec![];
        for (index, series) in self.series_list.iter().enumerate() {
            let color = get_color(&self.series_colors, series.index.unwrap_or(index));
            let mut points = vec![];
            let values = series.data_values();
            for (i, item) in indicators.iter().enumerate() {
                if let Some(value) = values.get(i) {
                    // Treat a missing point (`NIL_VALUE`) or a non-positive
                    // indicator max as the center, so the sentinel `f32::MIN`
                    // cannot leak into the polygon as a huge coordinate.
                    let mut ir = if item.max <= 0.0 || *value == NIL_VALUE {
                        0.0
                    } else {
                        *value / item.max * r
                    };

                    ir = ir.clamp(0.0, r);
                    let p = get_pie_point(cx, cy, ir, angle * i as f32);
                    if series.label_show {
                        let label =
                            format_series_value(value.to_owned(), &self.series_label_formatter);
                        label_positions.push((p, label));
                    }
                    points.push(p);
                }
            }
            let fill = if self.series_fill {
                Some(color.with_alpha(50))
            } else {
                None
            };
            c.straight_line(StraightLine {
                color: Some(color),
                fill,
                points: points.clone(),
                stroke_width: self.series_stroke_width,
                close: true,
                ..Default::default()
            });
        }
        for item in label_positions.iter() {
            let mut dx = None;
            let text = item.1.clone();
            let point = item.0;
            if let Ok(value) =
                measure_text_width_family(&self.font_family, self.series_label_font_size, &text)
            {
                dx = Some(-value.width() / 2.0);
            }
            c.text(Text {
                text: text.clone(),
                dy: Some(-8.0),
                dx,
                font_family: Some(self.font_family.clone()),
                font_color: Some(self.series_label_font_color),
                font_size: Some(self.series_label_font_size),
                font_weight: self.series_label_font_weight.clone(),
                x: Some(point.x),
                y: Some(point.y),
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
        let mut radar_chart = RadarChart::new(
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
        radar_chart.series_list[0].label_show = true;

        assert_eq!(
            include_str!("../../asset/radar_chart/three_points.svg"),
            radar_chart.svg().unwrap()
        );
    }

    // A missing point (`f32::MIN` / `NIL_VALUE`) must collapse to the center
    // rather than leaking the sentinel into the polygon as a huge coordinate.
    #[test]
    fn radar_missing_point_no_overflow() {
        let radar_chart = RadarChart::new(
            vec![Series::new(
                "A".to_string(),
                vec![f32::MIN, 3000.0, 20000.0],
            )],
            vec![
                ("Sales", 6500.0).into(),
                ("Admin", 16000.0).into(),
                ("IT", 30000.0).into(),
            ],
        );
        let svg = radar_chart.svg().unwrap();
        assert!(!svg.contains("NaN"));
        // Valid coordinates are ≤ 3-4 digits; a run of 7+ digits means the
        // overflowed sentinel leaked into a coordinate.
        let mut run = 0;
        let mut max_run = 0;
        for b in svg.bytes() {
            if b.is_ascii_digit() {
                run += 1;
                max_run = max_run.max(run);
            } else {
                run = 0;
            }
        }
        assert!(max_run < 7, "overflowed radar coordinate leaked into SVG");
    }
}
