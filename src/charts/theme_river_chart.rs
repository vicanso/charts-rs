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

// ── ThemeRiverChart ──────────────────────────────────────────────────────────

/// A "theme river" (streamgraph): each series is a stream whose thickness at
/// each time step encodes its value, stacked around a centered baseline so the
/// composition and its evolution over time are both easy to read.
///
/// Data reuses the shared model: `series_list` holds one [`Series`] per stream
/// and `x_axis_data` holds the time-axis labels.
#[derive(Clone, Debug, Default)]
pub struct ThemeRiverChart {
    /// The shared chart options (size, series, title/legend, axes); exposed
    /// directly on the chart through `Deref`, e.g. `chart.title_text`.
    pub base: ChartBase,
    y_axis_configs: Vec<YAxisConfig>,

    // theme-river-specific
    /// Opacity of the stream bands in `0.0..=1.0`. Default: 0.85.
    pub stream_opacity: f32,
}

impl std::ops::Deref for ThemeRiverChart {
    type Target = ChartBase;
    fn deref(&self) -> &ChartBase {
        &self.base
    }
}
impl std::ops::DerefMut for ThemeRiverChart {
    fn deref_mut(&mut self) -> &mut ChartBase {
        &mut self.base
    }
}

impl ThemeRiverChart {
    fn fill_default(&mut self) {
        if self.stream_opacity <= 0.0 {
            self.stream_opacity = 0.85;
        }
        self.stream_opacity = self.stream_opacity.min(1.0);
    }

    /// Creates a theme river chart with the default theme.
    pub fn new(series_list: Vec<Series>, x_axis_data: Vec<String>) -> ThemeRiverChart {
        ThemeRiverChart::new_with_theme(series_list, x_axis_data, &get_default_theme_name())
    }

    /// Creates a theme river chart with a custom theme.
    pub fn new_with_theme(
        series_list: Vec<Series>,
        x_axis_data: Vec<String>,
        theme: &str,
    ) -> ThemeRiverChart {
        let mut c = ThemeRiverChart {
            ..Default::default()
        };
        c.series_list = series_list;
        c.x_axis_data = x_axis_data;
        c.base.fill_theme(get_theme(theme), &mut c.y_axis_configs);
        c.fill_default();
        c
    }

    /// Creates a theme river chart from a JSON string.
    pub fn from_json(json: &str) -> canvas::Result<ThemeRiverChart> {
        let mut c = ThemeRiverChart {
            ..Default::default()
        };
        // `series_list` and `x_axis_data` are parsed by the derived fill_option.
        let value = c.base.fill_option(json, &mut c.y_axis_configs)?;
        if let Some(v) = get_f32_from_value(&value, "stream_opacity") {
            c.stream_opacity = v;
        }
        c.fill_default();
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

        let t_count = self
            .series_list
            .iter()
            .map(|s| s.data.len())
            .max()
            .unwrap_or(0);
        if t_count == 0 {
            return c.svg();
        }

        let font_size = self.series_label_font_size.max(10.0);
        // Reserve space at the bottom for the time-axis labels.
        let x_label_h = if self.x_axis_data.is_empty() {
            0.0
        } else {
            font_size + 6.0
        };
        let plot_h = (ch - x_label_h).max(1.0);

        // Value of a stream at a time step; missing points count as 0.
        let val = |s: &Series, t: usize| -> f32 {
            s.data.get(t).and_then(|v| *v).unwrap_or(0.0).max(0.0)
        };

        // Per-step totals and stacked prefix sums (`belows[i][t]` = sum of
        // series `0..i` at step `t`), computed once up front — re-summing per
        // band would be O(series² × steps).
        let mut totals = vec![0.0_f32; t_count];
        for (t, total) in totals.iter_mut().enumerate() {
            *total = self.series_list.iter().map(|s| val(s, t)).sum();
        }
        let mut belows: Vec<Vec<f32>> = Vec::with_capacity(self.series_list.len());
        let mut acc = vec![0.0_f32; t_count];
        for s in self.series_list.iter() {
            belows.push(acc.clone());
            for (t, a) in acc.iter_mut().enumerate() {
                *a += val(s, t);
            }
        }

        // Peak total across time drives the vertical scale.
        let mut max_total = 0.0_f32;
        for &total in totals.iter() {
            max_total = max_total.max(total);
        }
        if max_total <= 0.0 {
            return c.svg();
        }
        // Leave a small vertical margin so the widest point does not touch edges.
        let ky = (plot_h * 0.9) / max_total;
        let center_y = plot_h / 2.0;

        let x_at = |t: usize| -> f32 {
            if t_count == 1 {
                cw / 2.0
            } else {
                t as f32 / (t_count - 1) as f32 * cw
            }
        };

        let alpha = (self.stream_opacity.clamp(0.0, 1.0) * 255.0).round() as u8;

        // ── Stream bands (stacked, centered baseline) ─────────────────────────
        for (i, s) in self.series_list.iter().enumerate() {
            let color = get_color(&self.series_colors, s.index.unwrap_or(i)).with_alpha(alpha);
            let mut top_pts: Vec<Point> = Vec::with_capacity(t_count);
            let mut bottom_pts: Vec<Point> = Vec::with_capacity(t_count);
            for t in 0..t_count {
                let v = val(s, t);
                let base = center_y - totals[t] * ky / 2.0;
                let y_top = base + belows[i][t] * ky;
                let y_bottom = y_top + v * ky;
                let x = x_at(t);
                top_pts.push((x, y_top).into());
                bottom_pts.push((x, y_bottom).into());
            }
            // Close the band: top edge left→right, bottom edge right→left.
            bottom_pts.reverse();
            top_pts.extend(bottom_pts);
            content.polygon(Polygon {
                fill: Some(color),
                points: top_pts,
                ..Default::default()
            });
        }

        // ── Stream labels (name at each stream's widest time step) ─────────────
        for (i, s) in self.series_list.iter().enumerate() {
            if s.name.is_empty() {
                continue;
            }
            let mut best_t = 0usize;
            let mut best_v = 0.0_f32;
            for t in 0..t_count {
                let v = val(s, t);
                if v > best_v {
                    best_v = v;
                    best_t = t;
                }
            }
            // Only label bands thick enough to hold the text.
            if best_v * ky < font_size {
                continue;
            }
            let base = center_y - totals[best_t] * ky / 2.0;
            let y = base + (belows[i][best_t] + best_v / 2.0) * ky;
            // Keep the label inside the plot: anchor the first step's label to
            // the start and the last step's to the end.
            let (lx, la) = if best_t == 0 {
                (x_at(best_t) + 2.0, "start")
            } else if best_t + 1 == t_count {
                (x_at(best_t) - 2.0, "end")
            } else {
                (x_at(best_t), "middle")
            };
            content.text(Text {
                text: s.name.clone(),
                font_family: Some(self.font_family.clone()),
                font_color: Some(self.series_label_font_color),
                font_size: Some(font_size),
                font_weight: self.series_label_font_weight.clone(),
                x: Some(lx),
                y: Some(y),
                text_anchor: Some(la.to_string()),
                dominant_baseline: Some("central".to_string()),
                ..Default::default()
            });
        }

        // ── Time-axis labels ──────────────────────────────────────────────────
        if !self.x_axis_data.is_empty() {
            // Subsample so labels do not overlap (~one every ~60px).
            let max_labels = (cw / 60.0).floor().max(1.0) as usize;
            let step = t_count.div_ceil(max_labels).max(1);
            let y = plot_h + font_size * 0.8;
            for (t, label) in self.x_axis_data.iter().enumerate().take(t_count) {
                if t % step != 0 && t != t_count - 1 {
                    continue;
                }
                if label.is_empty() {
                    continue;
                }
                let anchor = if t == 0 {
                    "start"
                } else if t + 1 == t_count {
                    "end"
                } else {
                    "middle"
                };
                content.text(Text {
                    text: label.clone(),
                    font_family: Some(self.font_family.clone()),
                    font_color: Some(self.x_axis_font_color),
                    font_size: Some(font_size),
                    x: Some(x_at(t)),
                    y: Some(y),
                    text_anchor: Some(anchor.to_string()),
                    dominant_baseline: Some("central".to_string()),
                    ..Default::default()
                });
            }
        }

        c.svg()
    }
}

#[cfg(test)]
mod tests {
    use super::ThemeRiverChart;
    use crate::Series;
    use pretty_assertions::assert_eq;

    fn make() -> ThemeRiverChart {
        // Streams are labeled inline, so the legend is turned off.
        let mut c = ThemeRiverChart::new(
            vec![
                Series::new("Rust".to_string(), vec![10.0, 14.0, 20.0, 28.0, 35.0, 40.0]),
                Series::new("Go".to_string(), vec![18.0, 22.0, 24.0, 26.0, 27.0, 30.0]),
                Series::new("Zig".to_string(), vec![2.0, 4.0, 7.0, 12.0, 20.0, 26.0]),
                Series::new("C++".to_string(), vec![30.0, 28.0, 26.0, 25.0, 24.0, 22.0]),
            ],
            vec![
                "2020".to_string(),
                "2021".to_string(),
                "2022".to_string(),
                "2023".to_string(),
                "2024".to_string(),
                "2025".to_string(),
            ],
        );
        c.legend_show = Some(false);
        c
    }

    #[test]
    fn theme_river_basic() {
        assert_eq!(
            include_str!("../../asset/theme_river_chart/basic.svg"),
            make().svg().unwrap()
        );
    }

    #[test]
    fn theme_river_from_json() {
        let chart = ThemeRiverChart::from_json(
            r##"{
                "title_text": "Traffic Sources",
                "legend_show": false,
                "x_axis_data": ["Jan", "Feb", "Mar", "Apr", "May", "Jun"],
                "series_list": [
                    {"name": "Search", "data": [40, 44, 50, 58, 62, 70]},
                    {"name": "Social", "data": [20, 26, 30, 28, 35, 40]},
                    {"name": "Direct", "data": [30, 30, 28, 32, 30, 34]},
                    {"name": "Referral", "data": [10, 12, 14, 16, 18, 20]}
                ]
            }"##,
        )
        .unwrap();
        assert_eq!(
            include_str!("../../asset/theme_river_chart/basic_json.svg"),
            chart.svg().unwrap()
        );
    }

    #[test]
    fn theme_river_all_zero_no_panic() {
        let chart = ThemeRiverChart::new(
            vec![Series::new("Z".to_string(), vec![0.0, 0.0])],
            vec!["a".to_string(), "b".to_string()],
        );
        assert!(chart.svg().unwrap().starts_with("<svg"));
    }

    #[test]
    fn theme_river_empty() {
        let chart = ThemeRiverChart::new(vec![], vec![]);
        assert!(chart.svg().unwrap().starts_with("<svg"));
    }
}
