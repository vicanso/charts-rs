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
use std::sync::Arc;

// ── ParallelChart ────────────────────────────────────────────────────────────

/// A parallel-coordinates chart: one vertical axis per dimension, and one
/// polyline per record connecting its value on each axis. It is the standard
/// view for comparing many records across many numeric dimensions at once.
///
/// Data reuses the shared model: `series_list` holds one [`Series`] per record
/// (its `data` are the values, one per dimension) and `x_axis_data` holds the
/// dimension names. Each axis is scaled independently to its own min..max.
#[charts_rs_derive::chart_common_fields]
#[derive(Clone, Debug, Default, Chart)]
pub struct ParallelChart {
    // x/y axis (required by #[derive(Chart)]); `x_axis_data` doubles as the
    // dimension names, the rest are unused in rendering.
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
    pub y_axis_hidden: bool,
    y_axis_configs: Vec<YAxisConfig>,
    grid_stroke_color: Color,
    grid_stroke_width: f32,

    // series (required by #[derive(Chart)])
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

impl ParallelChart {
    /// Creates a parallel-coordinates chart with the default theme.
    pub fn new(series_list: Vec<Series>, x_axis_data: Vec<String>) -> ParallelChart {
        ParallelChart::new_with_theme(series_list, x_axis_data, &get_default_theme_name())
    }

    /// Creates a parallel-coordinates chart with a custom theme.
    pub fn new_with_theme(
        series_list: Vec<Series>,
        x_axis_data: Vec<String>,
        theme: &str,
    ) -> ParallelChart {
        let mut c = ParallelChart {
            series_list,
            x_axis_data,
            ..Default::default()
        };
        c.fill_theme(get_theme(theme));
        c
    }

    /// Creates a parallel-coordinates chart from a JSON string.
    pub fn from_json(json: &str) -> canvas::Result<ParallelChart> {
        let mut c = ParallelChart {
            ..Default::default()
        };
        // `series_list` and `x_axis_data` are parsed by the derived fill_option.
        c.fill_option(json)?;
        Ok(c)
    }

    pub fn svg(&self) -> canvas::Result<String> {
        let mut c = Canvas::new_width_xy(self.width, self.height, self.x, self.y);

        let axis_top = self.render_header(&mut c);

        let mut content = c.child(Box {
            top: axis_top,
            ..Default::default()
        });

        let cw = content.width();
        let ch = content.height();
        if cw <= 0.0 || ch <= 0.0 || self.series_list.is_empty() {
            return c.svg();
        }

        // Number of axes: the dimension names if given, else the longest record.
        let n = if !self.x_axis_data.is_empty() {
            self.x_axis_data.len()
        } else {
            self.series_list
                .iter()
                .map(|s| s.data.len())
                .max()
                .unwrap_or(0)
        };
        if n < 2 {
            return c.svg();
        }

        let font_size = self.series_label_font_size.max(10.0);
        // Reserve room for the dimension name (top) and min/max value labels.
        let top_pad = font_size + 6.0;
        let bottom_pad = font_size + 6.0;
        let plot_top = top_pad;
        let plot_bottom = (ch - bottom_pad).max(plot_top + 1.0);
        let plot_h = plot_bottom - plot_top;
        // Inset horizontally so the first/last axis names and value labels are
        // not clipped at the edges.
        let side = (cw * 0.06).clamp(font_size, 48.0);
        let inner_w = (cw - 2.0 * side).max(1.0);

        let x_at = |j: usize| -> f32 {
            if n == 1 {
                cw / 2.0
            } else {
                side + j as f32 / (n - 1) as f32 * inner_w
            }
        };

        // Value of a record on a dimension; `None` marks a missing point.
        let val = |s: &Series, j: usize| -> Option<f32> { s.data.get(j).copied().flatten() };

        // Independent min/max per dimension.
        let mut mins = vec![f32::MAX; n];
        let mut maxs = vec![f32::MIN; n];
        for s in &self.series_list {
            for j in 0..n {
                if let Some(v) = val(s, j) {
                    mins[j] = mins[j].min(v);
                    maxs[j] = maxs[j].max(v);
                }
            }
        }
        for j in 0..n {
            // Empty or flat dimension: fall back to a unit range so the mapping
            // stays finite.
            if !mins[j].is_finite() || !maxs[j].is_finite() {
                mins[j] = 0.0;
                maxs[j] = 1.0;
            } else if (maxs[j] - mins[j]).abs() < f32::EPSILON {
                maxs[j] = mins[j] + 1.0;
            }
        }

        // Map a value on dimension `j` to a pixel y (top = max, bottom = min).
        let y_of = |j: usize, v: f32| -> f32 {
            plot_bottom - (v - mins[j]) / (maxs[j] - mins[j]) * plot_h
        };

        // ── Axes + labels ─────────────────────────────────────────────────────
        for j in 0..n {
            let x = x_at(j);
            content.line(Line {
                color: Some(self.grid_stroke_color),
                stroke_width: self.grid_stroke_width.max(1.0),
                left: x,
                top: plot_top,
                right: x,
                bottom: plot_bottom,
                ..Default::default()
            });
            // Dimension name above the axis.
            if let Some(name) = self.x_axis_data.get(j) {
                content.text(Text {
                    text: name.clone(),
                    font_family: Some(self.font_family.clone()),
                    font_color: Some(self.x_axis_font_color),
                    font_size: Some(font_size),
                    font_weight: self.x_axis_font_weight.clone(),
                    x: Some(x),
                    y: Some(plot_top - font_size * 0.6),
                    text_anchor: Some("middle".to_string()),
                    dominant_baseline: Some("central".to_string()),
                    ..Default::default()
                });
            }
            // Max at the top, min at the bottom, drawn just inside each axis end.
            // The last axis anchors its value labels to the left so they do not
            // spill past the right edge.
            let (label_x, label_anchor) = if j == n - 1 {
                (x - 3.0, "end")
            } else {
                (x + 3.0, "start")
            };
            for (value, y) in [
                (maxs[j], plot_top + font_size * 0.6),
                (mins[j], plot_bottom - font_size * 0.6),
            ] {
                content.text(Text {
                    text: format_float(value),
                    font_family: Some(self.font_family.clone()),
                    font_color: Some(self.series_label_font_color),
                    font_size: Some(font_size * 0.85),
                    x: Some(label_x),
                    y: Some(y),
                    text_anchor: Some(label_anchor.to_string()),
                    dominant_baseline: Some("central".to_string()),
                    ..Default::default()
                });
            }
        }

        // ── Record polylines ──────────────────────────────────────────────────
        for (i, s) in self.series_list.iter().enumerate() {
            let color = get_color(&self.series_colors, s.index.unwrap_or(i));
            let mut points: Vec<Point> = Vec::with_capacity(n);
            for j in 0..n {
                if let Some(v) = val(s, j) {
                    points.push((x_at(j), y_of(j, v)).into());
                }
            }
            if points.len() < 2 {
                continue;
            }
            content.polyline(Polyline {
                color: Some(color),
                stroke_width: self.series_stroke_width.max(1.0),
                points,
            });
        }

        c.svg()
    }
}

#[cfg(test)]
mod tests {
    use super::ParallelChart;
    use crate::Series;
    use pretty_assertions::assert_eq;

    fn make() -> ParallelChart {
        ParallelChart::new(
            vec![
                Series::new("Laptop A".to_string(), vec![1200.0, 1.4, 8.0, 512.0, 92.0]),
                Series::new("Laptop B".to_string(), vec![950.0, 1.8, 6.0, 256.0, 78.0]),
                Series::new(
                    "Laptop C".to_string(),
                    vec![1800.0, 1.2, 12.0, 1024.0, 96.0],
                ),
                Series::new("Laptop D".to_string(), vec![700.0, 2.1, 4.0, 256.0, 65.0]),
            ],
            vec![
                "Price".to_string(),
                "Weight".to_string(),
                "Cores".to_string(),
                "Storage".to_string(),
                "Score".to_string(),
            ],
        )
    }

    #[test]
    fn parallel_basic() {
        assert_eq!(
            include_str!("../../asset/parallel_chart/basic.svg"),
            make().svg().unwrap()
        );
    }

    #[test]
    fn parallel_from_json() {
        let chart = ParallelChart::from_json(
            r##"{
                "title_text": "Car Specs",
                "legend_show": false,
                "x_axis_data": ["MPG", "Cylinders", "Horsepower", "Weight", "Price"],
                "series_list": [
                    {"name": "Sedan", "data": [32, 4, 180, 3200, 28]},
                    {"name": "SUV", "data": [24, 6, 280, 4500, 42]},
                    {"name": "Sports", "data": [21, 8, 450, 3400, 75]},
                    {"name": "Hatchback", "data": [38, 4, 150, 2800, 22]}
                ]
            }"##,
        )
        .unwrap();
        assert_eq!(
            include_str!("../../asset/parallel_chart/basic_json.svg"),
            chart.svg().unwrap()
        );
    }

    #[test]
    fn parallel_flat_dimension_no_panic() {
        // A dimension where every record has the same value must not divide by 0.
        let chart = ParallelChart::new(
            vec![
                Series::new("a".to_string(), vec![5.0, 5.0]),
                Series::new("b".to_string(), vec![5.0, 9.0]),
            ],
            vec!["d0".to_string(), "d1".to_string()],
        );
        let svg = chart.svg().unwrap();
        assert!(!svg.contains("NaN") && !svg.contains("inf"));
    }

    #[test]
    fn parallel_empty() {
        let chart = ParallelChart::new(vec![], vec![]);
        assert!(chart.svg().unwrap().starts_with("<svg"));
    }
}
