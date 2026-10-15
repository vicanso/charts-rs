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
use super::color::*;
use super::common::*;
use super::component::*;
use super::params::*;
use super::theme::{get_default_theme_name, get_theme};
use super::util::*;
use crate::charts::measure_text_width_family;

/// One radar axis: its name and the values at the center and the rim.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct RadarIndicator {
    /// Name of the indicator axis.
    pub name: String,
    /// Value at the center of this axis. A value below it is drawn at the
    /// center. Default 0, so an axis that only sets `max` is unchanged.
    pub min: f32,
    /// Value at the outer end of this axis. Raised to the largest finite
    /// value of the data when that is larger.
    pub max: f32,
}
impl From<(&str, f32)> for RadarIndicator {
    fn from(val: (&str, f32)) -> Self {
        RadarIndicator {
            name: val.0.to_string(),
            max: val.1,
            min: 0.0,
        }
    }
}

/// The web of a radar chart.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum RadarShape {
    /// Concentric polygons, one side per indicator.
    #[default]
    Polygon,
    /// Concentric circles. The spokes stay straight lines.
    Circle,
}

fn get_radar_indicator_list_from_value(value: &serde_json::Value) -> Option<Vec<RadarIndicator>> {
    if let Some(data) = value.get("indicators")
        && let Some(arr) = data.as_array()
    {
        let mut indicators = vec![];
        for item in arr.iter() {
            let name = get_string_from_value(item, "name").unwrap_or_default();
            let max = get_f32_from_value(item, "max").unwrap_or_default();
            let min = get_f32_from_value(item, "min").unwrap_or_default();
            if !name.is_empty() {
                indicators.push(RadarIndicator { name, max, min });
            }
        }
        return Some(indicators);
    }
    None
}

/// A radar chart plotting each series against a ring of indicators.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct RadarChart {
    /// The shared chart options (size, series, title/legend, axes); exposed
    /// directly on the chart through `Deref`, e.g. `chart.title.text`.
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
    /// Number of concentric rings of the web; 0 means the default of 5.
    pub split_number: usize,
    /// `polygon` (the default) or `circle`.
    pub shape: RadarShape,
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
        let data = r
            .base
            .fill_option(data, &mut r.y_axis_configs, super::schema::RADAR_FIELDS)?;
        if let Some(indicators) = get_radar_indicator_list_from_value(&data) {
            r.indicators = indicators;
        }
        if let Some(split_number) = get_usize_from_value(&data, "split_number") {
            r.split_number = split_number;
        }
        if get_string_from_value(&data, "shape")
            .is_some_and(|shape| shape.eq_ignore_ascii_case("circle"))
        {
            r.shape = RadarShape::Circle;
        }
        if data.get("series_fill").is_none() {
            r.series.fill = true;
        }
        // `fill_theme` installs the line-chart marker when a theme names
        // none. A radar chart keeps its own circle of radius 2 unless the
        // theme or this document sets `series_symbol`.
        if data.get("series_symbol").is_none() {
            let theme_name =
                get_string_from_value(&data, "theme").unwrap_or_else(get_default_theme_name);
            if get_theme(&theme_name).series.symbol.is_none() {
                r.series.symbol = None;
            }
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
        let theme = get_theme(theme);
        let theme_symbol = theme.series.symbol.clone();
        r.base.fill_theme(theme, &mut r.y_axis_configs);
        r.series.fill = true;
        if theme_symbol.is_none() {
            r.series.symbol = None;
        }
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
        let mut c = self.new_canvas();

        let axis_top = self.render_header(&mut c);
        if axis_top > 0.0 {
            c = c.child(Box {
                top: axis_top,
                ..Default::default()
            });
        }

        let mut max_values: Vec<f32> = vec![0.0; self.indicators.len()];
        for series in self.series_list.iter() {
            for (index, item) in series.iter_values().enumerate() {
                // A missing point is `NIL_VALUE` and must not become the max.
                if index < max_values.len() && item != NIL_VALUE && item > max_values[index] {
                    max_values[index] = item
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
        // The web fits the shorter side so its labels stay inside a canvas
        // that is taller than wide.
        let r = (c.width().min(c.height()) / 2.0 - offset).max(1.0);
        let angle = 360.0 / indicators.len() as f32;
        let cx = c.width() / 2.0;
        let cy = c.height() / 2.0;
        let round_count = if self.split_number == 0 {
            5
        } else {
            self.split_number
        };
        for i in 1..=round_count {
            let ir = r / round_count as f32 * i as f32;
            if self.shape == RadarShape::Circle {
                c.circle(Circle {
                    stroke_color: Some(self.grid.stroke_color),
                    stroke_width: self.grid.stroke_width,
                    stroke_dash_array: self.grid.stroke_dash_array.clone(),
                    cx,
                    cy,
                    r: ir,
                    ..Default::default()
                });
            } else {
                let mut points = vec![];
                for index in 0..indicators.len() {
                    points.push(get_pie_point(cx, cy, ir, angle * index as f32));
                }
                c.straight_line(StraightLine {
                    color: Some(self.grid.stroke_color),
                    points,
                    stroke_width: self.grid.stroke_width,
                    stroke_dash_array: self.grid.stroke_dash_array.clone(),
                    symbol: None,
                    close: true,
                    ..Default::default()
                });
            }
        }
        for (index, item) in indicators.iter().enumerate() {
            let current_angle = angle * index as f32;
            let p = get_pie_point(cx, cy, r, current_angle);
            let mut x = p.x;
            let mut y = p.y;
            let x_offset = 3.0;
            if let Ok(measurement) = measure_text_width_family(
                &self.font_family,
                self.series.label.font.size,
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
                font_size: Some(self.series.label.font.size),
                font_family: Some(self.font_family.clone()),
                font_color: Some(self.series.label.font.color),
                x: Some(x),
                y: Some(y),
                ..Default::default()
            });
            c.child(Box::default()).line(Line {
                color: Some(self.grid.stroke_color),
                stroke_width: self.grid.stroke_width,
                stroke_dash_array: self.grid.stroke_dash_array.clone(),
                left: p.x,
                top: p.y,
                right: cx,
                bottom: cy,
            });
        }

        let mut label_positions = vec![];
        for (index, series) in self.series_list.iter().enumerate() {
            let color = get_color(&self.series.colors, series.index.unwrap_or(index));
            let mut points = vec![];
            for (i, (item, value)) in indicators.iter().zip(series.iter_values()).enumerate() {
                let ir = indicator_radius(item.min, item.max, value, r);
                let p = get_pie_point(cx, cy, ir, angle * i as f32);
                if series.label_show {
                    let label = format_series_label(
                        &self.series.label.formatter,
                        value,
                        &series.name,
                        &item.name,
                    );
                    label_positions.push((p, label));
                }
                points.push(p);
            }
            let fill = if series.fill.unwrap_or(self.series.fill) {
                Some(color.with_alpha(self.fill_alpha(50)))
            } else {
                None
            };
            // A series symbol wins, then the chart-wide one. `Symbol::None`
            // draws nothing. With neither set, each vertex keeps the circle
            // of radius 2 the radar has always drawn.
            let symbol = series
                .symbol
                .clone()
                .or_else(|| self.series.symbol.clone())
                .unwrap_or(Symbol::Circle(2.0, None));
            c.straight_line(StraightLine {
                color: Some(color),
                fill,
                points,
                stroke_width: self.series.stroke_width,
                symbol: Some(symbol),
                stroke_dash_array: series.stroke_dash_array.clone(),
                close: true,
                ..Default::default()
            });
        }
        for item in label_positions.iter() {
            let mut dx = None;
            let text = item.1.clone();
            let point = item.0;
            if let Ok(value) =
                measure_text_width_family(&self.font_family, self.series.label.font.size, &text)
            {
                dx = Some(-value.width() / 2.0);
            }
            c.text(Text {
                text: text.clone(),
                dy: Some(-8.0),
                dx,
                font_family: Some(self.font_family.clone()),
                font_color: Some(self.series.label.font.color),
                font_size: Some(self.series.label.font.size),
                font_weight: self.series.label.font.weight.clone(),
                x: Some(point.x),
                y: Some(point.y),
                ..Default::default()
            });
        }

        c.svg()
    }
}

/// Radius of one value. The center is `min` and the rim is `max`. A missing
/// value, a range that does not open upwards, or a value below `min` sits at
/// the center; a value above `max` sits on the rim.
fn indicator_radius(min: f32, max: f32, value: f32, radius: f32) -> f32 {
    let span = max - min;
    if span <= 0.0 || value == NIL_VALUE {
        0.0
    } else {
        ((value - min) / span).clamp(0.0, 1.0) * radius
    }
}

#[cfg(test)]
mod tests {
    use super::RadarChart;
    use crate::Series;

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
        assert_snapshot!("radar_chart/basic.svg", radar_chart.svg().unwrap());
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

        assert_snapshot!("radar_chart/seven_points.svg", radar_chart.svg().unwrap());
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

        assert_snapshot!("radar_chart/five_points.svg", radar_chart.svg().unwrap());
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

        assert_snapshot!("radar_chart/four_points.svg", radar_chart.svg().unwrap());
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

        assert_snapshot!("radar_chart/three_points.svg", radar_chart.svg().unwrap());
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

    #[test]
    fn circle_web_scales_from_min_and_fill_is_per_series() {
        let chart = RadarChart::from_json(
            r###"{
                "shape": "circle",
                "indicators": [
                    {"name": "Sales", "min": 1000, "max": 6500},
                    {"name": "Administration", "max": 16000},
                    {"name": "Information Technology", "max": 30000},
                    {"name": "Customer Support", "max": 38000},
                    {"name": "Development", "max": 52000}
                ],
                "series_list": [
                    {
                        "name": "Allocated Budget",
                        "data": [4200.0, 3000.0, 20000.0, 35000.0, 50000.0],
                        "label_show": true
                    },
                    {
                        "name": "Actual Spending",
                        "data": [5000.0, 14000.0, 28000.0, 26000.0, 42000.0],
                        "fill": false,
                        "label_show": true
                    }
                ]
            }"###,
        )
        .unwrap();
        assert_eq!(super::RadarShape::Circle, chart.shape);
        assert_eq!(5, chart.indicators.len());
        assert_eq!(1000.0, chart.indicators[0].min);
        assert_eq!(0.0, chart.indicators[1].min);

        let svg = chart.svg().unwrap();
        let grid_circles = |svg: &str| {
            svg.lines()
                .filter(|line| line.starts_with("<circle") && line.contains("stroke=\"#E0E6F2\""))
                .count()
        };
        assert_eq!(5, grid_circles(&svg), "{svg}");
        let mut polygon = chart.clone();
        polygon.shape = super::RadarShape::Polygon;
        assert_eq!(0, grid_circles(&polygon.svg().unwrap()));

        // Five axes, so each series polygon has five vertices. Sales starts
        // at 1000 rather than 0, and the outline series is not filled.
        let vertices = |path: &str| path.split(" L ").count();
        let paths: Vec<&str> = svg.split("<path ").skip(1).collect();
        assert_eq!(2, paths.len(), "{svg}");
        assert_eq!(5, vertices(paths[0]), "{}", paths[0]);
        assert!(paths[0].contains("fill-opacity"), "{}", paths[0]);
        assert_eq!(5, vertices(paths[1]), "{}", paths[1]);
        assert!(paths[1].contains("fill=\"none\""), "{}", paths[1]);
        assert!(!paths[1].contains("fill-opacity"), "{}", paths[1]);
        // Sales is the top spoke. 4200 on a scale from 1000 to 6500 is not
        // the same place as 4200 on a scale from 0.
        let spoke = svg
            .lines()
            .find(|line| line.starts_with("<line") && line.contains("stroke=\"#E0E6F2\""))
            .unwrap();
        let attr = |name: &str| {
            let key = format!("{name}=\"");
            let start = spoke.find(&key).unwrap() + key.len();
            let end = start + spoke[start..].find('"').unwrap();
            spoke[start..end].parse::<f32>().unwrap()
        };
        let (y1, y2) = (attr("y1"), attr("y2"));
        let vertex_y = paths[0]
            .split_whitespace()
            .nth(2)
            .unwrap()
            .parse::<f32>()
            .unwrap();
        let ratio = (y2 - vertex_y) / (y2 - y1);
        let expected = (4200.0 - 1000.0) / (6500.0 - 1000.0);
        assert!(
            (ratio - expected).abs() < 0.02,
            "sales vertex ratio {ratio}, expected {expected}"
        );
        for value in [
            "4200", "3000", "20000", "35000", "50000", "5000", "14000", "28000", "26000", "42000",
        ] {
            assert!(
                svg.contains(&format!("\n{value}\n")),
                "missing label {value}"
            );
        }

        let mut unfilled = chart.clone();
        unfilled.series.fill = false;
        let unfilled = unfilled.svg().unwrap();
        let paths: Vec<&str> = unfilled.split("<path ").skip(1).collect();
        assert!(paths[0].contains("fill=\"none\""), "{}", paths[0]);

        assert_eq!(
            0.0,
            super::indicator_radius(0.0, 10.0, super::NIL_VALUE, 40.0)
        );
        assert_eq!(0.0, super::indicator_radius(5.0, 5.0, 5.0, 40.0));
        assert_eq!(20.0, super::indicator_radius(0.0, 10.0, 5.0, 40.0));

        assert_snapshot!("radar_chart/circle.svg", svg);
    }

    #[test]
    fn radar_symbol_and_dash_follow_the_series() {
        let mut chart = RadarChart::new(
            vec![
                ("Filled", vec![1.0, 2.0, 3.0]).into(),
                ("Line", vec![3.0, 2.0, 1.0]).into(),
            ],
            vec![("x", 5.0).into(), ("y", 5.0).into(), ("z", 5.0).into()],
        );
        chart.legend.show = Some(false);
        let plain = chart.svg().unwrap();
        assert_eq!(6, plain.matches("r=\"2\"").count(), "{plain}");

        chart.series.symbol = Some(super::Symbol::None);
        let bare = chart.svg().unwrap();
        assert!(!bare.contains("r=\"2\""), "{bare}");

        chart.series.symbol = None;
        chart.series_list[0].symbol = Some(super::Symbol::Circle(6.0, None));
        chart.series_list[1].symbol = Some(super::Symbol::None);
        chart.series_list[1].stroke_dash_array = Some("4,2".into());
        let svg = chart.svg().unwrap();
        assert_eq!(3, svg.matches("r=\"6\"").count(), "{svg}");
        assert!(!svg.contains("r=\"2\""), "{svg}");
        let paths: Vec<&str> = svg
            .split("<path ")
            .skip(1)
            .filter(|path| path.contains("stroke-width=\"2\""))
            .collect();
        assert_eq!(2, paths.len(), "{svg}");
        assert!(!paths[0].contains("stroke-dasharray"), "{}", paths[0]);
        assert!(
            paths[1].contains("stroke-dasharray=\"4,2\""),
            "{}",
            paths[1]
        );
    }
}
