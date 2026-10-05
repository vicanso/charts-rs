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

use super::base::{ChartBase, XAxisMode, axis_value_params, get_y_axis_config};
use super::canvas;
use super::color::*;
use super::common::*;
use super::component::*;
use super::params::*;
use super::theme::{get_default_theme_name, get_theme};
use super::util::*;
use super::x_axis::{decimals_of, format_number, nice_step};

/// Upper bound on the number of bins, however small a `bin_width` is asked
/// for: more bars than this are not readable, only expensive.
const MAX_BINS: usize = 1000;

/// Equal-width bins over a range: `count` bins of `width`, the first one
/// starting at `start`.
#[derive(Clone, Copy, Debug, PartialEq)]
struct Bins {
    start: f64,
    width: f64,
    count: usize,
}

impl Bins {
    /// The left edge of bin `i` (and the right edge of the one before it).
    fn edge(&self, i: usize) -> f64 {
        self.start + self.width * i as f64
    }
    fn end(&self) -> f64 {
        self.edge(self.count)
    }
    /// The bin `value` falls in: every bin holds its left edge, and the last
    /// one its right edge too. Values outside the range belong to no bin.
    fn index(&self, value: f64) -> Option<usize> {
        // A hair of slack, so a value on the last edge is not lost to
        // rounding in `start + width * count`.
        let slack = self.width * 1e-9;
        if value < self.start - slack || value > self.end() + slack {
            return None;
        }
        let index = ((value - self.start) / self.width).floor().max(0.0) as usize;
        Some(index.min(self.count - 1))
    }
}

/// A histogram: how the values of a sample are distributed, as the number
/// of values falling into each of a row of equal-width bins.
///
/// The `data` of a series is the sample itself (the raw values, in any
/// order), not one value per category. Several series share the same bins
/// and are drawn over each other, slightly transparent — or on top of each
/// other when they share a `stack` name.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct HistogramChart {
    /// The shared chart options (size, series, title/legend, axes); exposed
    /// directly on the chart through `Deref`, e.g. `chart.title.text`.
    pub base: ChartBase,
    /// Configuration of the y (count) axis.
    pub y_axis_configs: Vec<YAxisConfig>,

    // histogram-specific
    /// Number of bins. `0` (the default) derives it from the sample size and
    /// rounds the bin edges to round numbers.
    pub bin_count: usize,
    /// Width of a bin; takes precedence over `bin_count`. The edges are
    /// multiples of it.
    pub bin_width: Option<f32>,
    /// Shows each bar as the share of its series' sample, in percent,
    /// instead of a count — so samples of different sizes compare.
    pub percent: bool,
    /// Gap between adjacent bars in pixels. Default: 1.
    pub bar_gap: Option<f32>,
}

impl std::ops::Deref for HistogramChart {
    type Target = ChartBase;
    fn deref(&self) -> &ChartBase {
        &self.base
    }
}
impl std::ops::DerefMut for HistogramChart {
    fn deref_mut(&mut self) -> &mut ChartBase {
        &mut self.base
    }
}

impl HistogramChart {
    /// Creates a histogram with the default theme. Each series carries one
    /// sample in its `data`.
    pub fn new(series_list: Vec<Series>) -> HistogramChart {
        HistogramChart::new_with_theme(series_list, &get_default_theme_name())
    }

    /// Creates a histogram with a custom theme.
    pub fn new_with_theme(series_list: Vec<Series>, theme: &str) -> HistogramChart {
        let mut c = HistogramChart {
            ..Default::default()
        };
        c.series_list = series_list;
        c.base.fill_theme(get_theme(theme), &mut c.y_axis_configs);
        c
    }

    /// Creates a histogram from a JSON string.
    pub fn from_json(json: &str) -> canvas::Result<HistogramChart> {
        let mut c = HistogramChart {
            ..Default::default()
        };
        let value =
            c.base
                .fill_option(json, &mut c.y_axis_configs, super::schema::HISTOGRAM_FIELDS)?;
        if let Some(v) = get_usize_from_value(&value, "bin_count") {
            c.bin_count = v;
        }
        if let Some(v) = get_f32_from_value(&value, "bin_width") {
            c.bin_width = Some(v);
        }
        if let Some(v) = get_bool_from_value(&value, "percent") {
            c.percent = v;
        }
        if let Some(v) = get_f32_from_value(&value, "bar_gap") {
            c.bar_gap = Some(v);
        }
        Ok(c)
    }

    /// The bins for samples spanning `min..=max`, the largest of them `n`
    /// values. `x_axis.min` / `x_axis.max` fix the range instead.
    fn bins(&self, min: f64, max: f64, n: usize) -> Bins {
        let fixed_min = self.x_axis.min.filter(|v| v.is_finite());
        let mut lo = fixed_min.unwrap_or(min);
        let mut hi = self.x_axis.max.filter(|v| v.is_finite()).unwrap_or(max);
        if !lo.is_finite() || !hi.is_finite() {
            (lo, hi) = (0.0, 1.0);
        }
        // A sample of one value (or none) still gets a bin around it.
        if hi <= lo {
            lo -= 0.5;
            hi = lo + 1.0;
        }
        // Bins `width` wide whose edges are multiples of it, unless the
        // start is fixed.
        let aligned = |width: f64| -> Bins {
            let start = match fixed_min {
                Some(start) => start,
                None => (lo / width + 1e-9).floor() * width,
            };
            let count = ((hi - start) / width - 1e-9).ceil().max(1.0);
            Bins {
                start,
                width,
                count: count as usize,
            }
        };
        if let Some(width) = self.bin_width.map(f64::from)
            && width.is_finite()
            && width > 0.0
            && (hi - lo) / width <= MAX_BINS as f64
        {
            return aligned(width);
        }
        if self.bin_count > 0 {
            let count = self.bin_count.min(MAX_BINS);
            return Bins {
                start: lo,
                width: (hi - lo) / count as f64,
                count,
            };
        }
        // Sturges' rule for the number of bins, then the bin width rounded
        // up to 1, 2 or 5 × 10ⁿ so the edges are round numbers.
        let target = (n.max(1) as f64).log2().ceil() + 1.0;
        aligned(nice_step((hi - lo) / target))
    }

    /// Renders the chart to an SVG string.
    pub fn svg(&self) -> canvas::Result<String> {
        let mut c = self.new_canvas();
        let axis_top = self.render_header(&mut c);

        // The samples, without missing values.
        let samples: Vec<Vec<f64>> = self
            .series_list
            .iter()
            .map(|s| {
                s.iter_values()
                    .filter(|v| *v != NIL_VALUE)
                    .map(f64::from)
                    .collect()
            })
            .collect();
        let (mut min, mut max, mut largest) = (f64::INFINITY, f64::NEG_INFINITY, 0);
        for values in samples.iter() {
            largest = largest.max(values.len());
            for &v in values.iter() {
                min = min.min(v);
                max = max.max(v);
            }
        }
        let bins = self.bins(min, max, largest);

        // The height of every bar: a count, or the share of the sample.
        let heights: Vec<Vec<f32>> = samples
            .iter()
            .map(|values| {
                let mut counts = vec![0.0_f32; bins.count];
                for &v in values.iter() {
                    if let Some(index) = bins.index(v) {
                        counts[index] += 1.0;
                    }
                }
                if self.percent && !values.is_empty() {
                    let total = values.len() as f32;
                    counts.iter_mut().for_each(|c| *c = *c / total * 100.0);
                }
                counts
            })
            .collect();

        // The y axis covers the tallest bar, or the tallest stack.
        let mut stack_keys: Vec<&str> = vec![];
        let mut stack_totals: Vec<Vec<f32>> = vec![];
        let mut data_list: Vec<f32> = vec![];
        for (series, counts) in self.series_list.iter().zip(heights.iter()) {
            match series.stack.as_deref() {
                Some(key) => {
                    let index = stack_keys
                        .iter()
                        .position(|k| *k == key)
                        .unwrap_or_else(|| {
                            stack_keys.push(key);
                            stack_totals.push(vec![0.0; bins.count]);
                            stack_keys.len() - 1
                        });
                    for (total, count) in stack_totals[index].iter_mut().zip(counts) {
                        *total += count;
                    }
                }
                None => data_list.extend(counts),
            }
        }
        data_list.extend(stack_totals.iter().flatten());

        // Shares read as percentages unless the axis has its own format.
        let mut y_axis_configs = self.y_axis_configs.clone();
        if self.percent
            && let Some(config) = y_axis_configs.first_mut()
            && config.formatter.is_none()
        {
            config.formatter = Some("{c}%".to_string());
        }
        let y_axis_config = get_y_axis_config(&y_axis_configs, 0);
        let has_values = samples.iter().any(|values| !values.is_empty());
        let left = if has_values {
            let values = get_axis_values(axis_value_params(&y_axis_config, data_list, true));
            let width = self.y_axis_width_for(&y_axis_config, &values);
            (values, width)
        } else {
            (AxisValues::default(), 0.0)
        };

        let layout = self.layout_cartesian_with(
            c,
            &y_axis_configs,
            axis_top,
            left,
            None,
            // Ticks on the bin edges.
            XAxisMode::Range {
                min: bins.start,
                max: bins.end(),
                tick: bins.width,
            },
        );
        let c = layout.canvas.clone();
        let Some(scale) = layout.x.as_ref() else {
            return c.svg();
        };
        let y_axis_values = &layout.left;
        let max_height = layout.max_height;
        let mut plot = layout.plot();

        // Histograms drawn over each other need to show through.
        let overlaid = self
            .series_list
            .iter()
            .filter(|s| s.stack.is_none())
            .count()
            > 1;
        let edge_decimals = decimals_of(bins.width).max(decimals_of(bins.start));
        let edge_label = |i: usize| format_number(bins.edge(i), edge_decimals);
        let value_label = |value: f32| {
            if self.percent {
                format_float(value) + "%"
            } else {
                format_float(value)
            }
        };

        // Running totals of each stack, to put the next series on top.
        let mut stack_base: Vec<Vec<f32>> = vec![vec![0.0; bins.count]; stack_keys.len()];
        let mut series_labels_list = vec![];
        for (series_index, (series, counts)) in
            self.series_list.iter().zip(heights.iter()).enumerate()
        {
            let mut color = get_color(&self.series.colors, series.index.unwrap_or(series_index));
            let stack_index = series
                .stack
                .as_deref()
                .and_then(|key| stack_keys.iter().position(|k| *k == key));
            if overlaid && stack_index.is_none() {
                color = color.with_alpha(150);
            }
            let mut series_labels = vec![];
            for (i, &value) in counts.iter().enumerate() {
                if value <= 0.0 {
                    continue;
                }
                let base = stack_index.map(|s| stack_base[s][i]).unwrap_or(0.0);
                if let Some(s) = stack_index {
                    stack_base[s][i] += value;
                }
                let bin_left = scale.px(bins.edge(i));
                let bin_width = scale.px(bins.edge(i + 1)) - bin_left;
                // Narrow bins give up their gap before their bar.
                let gap = self.bar_gap.unwrap_or(1.0).clamp(0.0, bin_width / 4.0);
                let top = y_axis_values.get_offset_height(base + value, max_height);
                let bottom = y_axis_values.get_offset_height(base, max_height);

                let tooltip_text = self.tooltip.show.then(|| {
                    format!(
                        "{}: {} – {}: {}",
                        series.name,
                        edge_label(i),
                        edge_label(i + 1),
                        value_label(value)
                    )
                });
                let mut classes: Vec<&str> = vec![];
                if self.animation.is_some() {
                    classes.push("bar-anim");
                }
                if tooltip_text.is_some() {
                    classes.push("ct-trigger");
                }
                let left = bin_left + gap / 2.0;
                let width = bin_width - gap;
                plot.rect(Rect {
                    fill: Some(color.into()),
                    left,
                    top,
                    width,
                    height: bottom - top,
                    class: (!classes.is_empty()).then(|| classes.join(" ")),
                    style: self
                        .animation
                        .as_ref()
                        .map(|a| format!("animation-delay:{}ms", i as u32 * a.delay)),
                    title: tooltip_text.clone(),
                    dataset: vec![
                        ("series".to_string(), series.name.clone()),
                        ("from".to_string(), edge_label(i)),
                        ("to".to_string(), edge_label(i + 1)),
                        ("value".to_string(), format_float(value)),
                    ],
                    ..Default::default()
                });
                if let Some(text) = tooltip_text {
                    plot.text_unmeasured(Text {
                        text,
                        class: Some("ct-tip".to_string()),
                        font_weight: self.tooltip.font.weight.clone(),
                        font_family: Some(self.font_family.clone()),
                        font_color: Some(self.tooltip_font_color(self.series.label.font.color)),
                        font_size: Some(self.tooltip_font_size(self.series.label.font.size)),
                        x: Some(left + width / 2.0),
                        y: Some(top),
                        dy: Some(-6.0),
                        text_anchor: Some("middle".to_string()),
                        ..Default::default()
                    });
                }
                if series.label_show {
                    series_labels.push(SeriesLabel {
                        point: (left + width / 2.0, top).into(),
                        text: value_label(value),
                    });
                }
            }
            if series.label_show {
                series_labels_list.push(series_labels);
            }
        }
        self.render_series_label(layout.plot(), series_labels_list);

        let mut css = String::new();
        if let Some(ref anim) = self.animation {
            css.push_str(&format!(
                "@keyframes bar-grow{{from{{transform:scaleY(0)}}to{{transform:scaleY(1)}}}} \
                 .bar-anim{{transform-box:fill-box;transform-origin:center bottom;\
                 animation:bar-grow {}ms {} both}} ",
                anim.duration,
                anim.safe_easing()
            ));
        }
        if self.tooltip.show {
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
    use super::{Bins, HistogramChart};
    use crate::Series;
    use pretty_assertions::assert_eq;

    fn chart() -> HistogramChart {
        HistogramChart::new(vec![])
    }

    /// A deterministic, roughly bell-shaped sample of 120 values in 150..190.
    fn sample() -> Vec<f32> {
        (0..120)
            .map(|i| {
                let a = ((i * 37) % 101) as f32 / 100.0;
                let b = ((i * 61) % 89) as f32 / 88.0;
                let c = ((i * 17) % 53) as f32 / 52.0;
                150.0 + (a + b + c) / 3.0 * 40.0
            })
            .collect()
    }

    #[test]
    fn automatic_bins_have_round_edges() {
        // 100 values → Sturges asks for 8 bins; 0..97 / 8 = 12.1 → width 20.
        assert_eq!(
            Bins {
                start: 0.0,
                width: 20.0,
                count: 5
            },
            chart().bins(3.0, 97.0, 100)
        );
        // Edges are multiples of the width, also below zero and for fractions.
        assert_eq!(
            Bins {
                start: -10.0,
                width: 5.0,
                count: 4
            },
            chart().bins(-7.5, 9.0, 12)
        );
        let bins = chart().bins(0.12, 0.47, 30);
        assert_eq!((0.1, 0.1, 4), (bins.start, bins.width, bins.count));
        // The largest value on an edge needs no extra bin.
        assert_eq!(2, chart().bins(0.0, 10.0, 2).count);
    }

    #[test]
    fn fixed_bins() {
        let mut c = chart();
        c.bin_count = 4;
        assert_eq!(
            Bins {
                start: 1.0,
                width: 2.0,
                count: 4
            },
            c.bins(1.0, 9.0, 50)
        );
        // A width wins over a count, and aligns the edges to its multiples.
        c.bin_width = Some(5.0);
        assert_eq!(
            Bins {
                start: 0.0,
                width: 5.0,
                count: 2
            },
            c.bins(1.0, 9.0, 50)
        );
        // A fixed range.
        c.x_axis.min = Some(-5.0);
        c.x_axis.max = Some(20.0);
        assert_eq!(
            Bins {
                start: -5.0,
                width: 5.0,
                count: 5
            },
            c.bins(1.0, 9.0, 50)
        );
        // A width that would need an absurd number of bins is ignored.
        let mut c = chart();
        c.bin_width = Some(1e-9);
        assert!(c.bins(0.0, 100.0, 50).count <= 10);
        // Asking for too many bins is capped.
        c.bin_width = None;
        c.bin_count = 5_000_000;
        assert_eq!(1000, c.bins(0.0, 100.0, 50).count);
    }

    #[test]
    fn degenerate_samples_still_get_a_bin() {
        // One distinct value, and no values at all.
        let bins = chart().bins(7.0, 7.0, 3);
        assert!(bins.count >= 1 && bins.index(7.0).is_some(), "{bins:?}");
        let bins = chart().bins(f64::INFINITY, f64::NEG_INFINITY, 0);
        assert!(bins.count >= 1 && bins.width > 0.0, "{bins:?}");
    }

    #[test]
    fn values_fall_into_half_open_bins() {
        let bins = Bins {
            start: 0.0,
            width: 10.0,
            count: 3,
        };
        assert_eq!(Some(0), bins.index(0.0));
        assert_eq!(Some(0), bins.index(9.99));
        assert_eq!(Some(1), bins.index(10.0));
        // The last bin is closed on the right.
        assert_eq!(Some(2), bins.index(30.0));
        assert_eq!(None, bins.index(30.5));
        assert_eq!(None, bins.index(-0.1));
    }

    #[test]
    fn histogram_chart_basic() {
        let mut c = HistogramChart::new(vec![Series::new("Height".to_string(), sample())]);
        c.title.text = "Heights".to_string();
        c.series_list[0].label_show = true;
        assert_snapshot!("histogram_chart/basic.svg", c.svg().unwrap());
    }
}
