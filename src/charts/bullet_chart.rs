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

use super::base::{ChartBase, axis_value_params, get_y_axis_config};
use super::canvas;
use super::color::*;
use super::common::*;
use super::component::*;
use super::font::text_ellipsis;
use super::params::*;
use super::theme::{get_default_theme_name, get_theme};
use super::util::*;
use crate::charts::measure_text_width_family;

/// One measure of a bullet chart, on a row of its own: its value as a bar,
/// the value aimed at as a line across it, and bands behind them that tell
/// what is poor, what is fair and what is good.
#[derive(Clone, Debug, PartialEq)]
pub struct Bullet {
    /// Name of the measure, written to the left of its row.
    pub name: String,
    /// The value, drawn as the bar. `NIL_VALUE` draws none.
    pub value: f32,
    /// The value aimed at, drawn as a line across the bar.
    pub target: Option<f32>,
    /// Where the bands behind the bar end: the first one reaches from the
    /// start of the scale to the first of them, the next one from there to
    /// the second, and so on. Without any there is one band, all along.
    pub ranges: Vec<f32>,
    /// The start of the scale. Without one it is that of an axis of the
    /// values: 0, or below it where a value is.
    pub min: Option<f32>,
    /// The end of the scale. Without one it is that of an axis of the
    /// values: the largest of them, rounded up.
    pub max: Option<f32>,
    /// Color of the bar; without one, the first of the palette.
    pub color: Option<Color>,
}

impl Default for Bullet {
    fn default() -> Self {
        Bullet {
            name: String::new(),
            value: NIL_VALUE,
            target: None,
            ranges: vec![],
            min: None,
            max: None,
            color: None,
        }
    }
}

impl From<(&str, f32)> for Bullet {
    fn from((name, value): (&str, f32)) -> Self {
        Bullet {
            name: name.to_string(),
            value,
            ..Default::default()
        }
    }
}

impl From<(&str, f32, f32)> for Bullet {
    fn from((name, value, target): (&str, f32, f32)) -> Self {
        Bullet {
            name: name.to_string(),
            value,
            target: Some(target),
            ..Default::default()
        }
    }
}

/// A bullet from its JSON object.
fn bullet_from_value(value: &serde_json::Value) -> Option<Bullet> {
    if !value.is_object() {
        return None;
    }
    Some(Bullet {
        name: get_string_from_value(value, "name").unwrap_or_default(),
        value: get_f32_from_value(value, "value").unwrap_or(NIL_VALUE),
        target: get_f32_from_value(value, "target"),
        ranges: get_f32_slice_from_value(value, "ranges").unwrap_or_default(),
        min: get_f32_from_value(value, "min"),
        max: get_f32_from_value(value, "max"),
        color: get_color_from_value(value, "color"),
    })
}

/// The start, the end and the number of parts of a scale whose ticks are
/// round numbers: 1, 2, 2.5 or 5 times a power of ten apart. A start or an
/// end that is given is kept, and split into round parts where it can be.
/// Without them the scale starts at 0, or at a round number below the
/// smallest of `data` where that is below 0, and ends at the first round
/// number from the largest of `data` on, in four to six parts.
fn round_scale(min: Option<f32>, max: Option<f32>, data: &[f32]) -> (f32, f32, usize) {
    let (least, most) = data
        .iter()
        .fold((f32::MAX, f32::MIN), |(a, b), v| (a.min(*v), b.max(*v)));
    let start = min.unwrap_or(if data.is_empty() { 0.0 } else { least.min(0.0) });
    let mut end = max.unwrap_or(if data.is_empty() { start + 1.0 } else { most });
    if end <= start {
        end = start + 1.0;
    }
    let span = end - start;
    // The smallest round step that is at least `raw`.
    let round = |raw: f32| -> f32 {
        let power = 10_f32.powf(raw.log10().floor());
        [1.0, 2.0, 2.5, 5.0, 10.0]
            .into_iter()
            .map(|m| m * power)
            .find(|step| *step >= raw * (1.0 - 1e-4))
            .unwrap_or(raw)
    };
    if max.is_some() {
        let parts = [5, 4, 6, 10, 8, 2, 3, 7, 9]
            .into_iter()
            .find(|parts| {
                let step = span / *parts as f32;
                (round(step) - step).abs() <= step * 1e-4
            })
            .unwrap_or(5);
        return (start, end, parts);
    }
    // Of four, five and six parts, the one that ends nearest past the
    // largest value, and of those the one nearer five parts.
    let mut best: Option<(f32, f32, usize)> = None;
    let mut best_key = (f32::MAX, usize::MAX);
    for parts in [5_usize, 4, 6] {
        let step = round(span / parts as f32);
        if !step.is_finite() || step <= 0.0 {
            continue;
        }
        let from = if min.is_none() && start < 0.0 {
            (start / step).floor() * step
        } else {
            start
        };
        let count = ((end - from) / step - 1e-4).ceil().max(1.0);
        let to = from + count * step;
        let key = (to - end, (count as usize).abs_diff(5));
        if key.0 < best_key.0 - 1e-4 || (key.0 <= best_key.0 + 1e-4 && key.1 < best_key.1) {
            best_key = key;
            best = Some((from, to, count as usize));
        }
    }
    best.unwrap_or((start, end, 5))
}

/// Between the name of a row and its bands.
const NAME_GAP: f32 = 10.0;
/// Between the bands of a row and its value.
const LABEL_GAP: f32 = 8.0;
/// The bands of a row are not higher than this.
const MAX_BAND: f32 = 40.0;

/// Measures against their targets, one row each: a compact kind of gauge,
/// of which several fit where one gauge would. A row is a bar for the value,
/// a line across it for the target and bands behind them for what is poor,
/// fair and good, on a scale of its own.
///
/// The names are written as the categories of a horizontal bar chart are,
/// in the font of the x axis; the scales are drawn as its value axis is,
/// after the first of `y_axis_configs`, and below each row unless
/// `x_axis.hidden`.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct BulletChart {
    /// The shared chart options (size, series, title/legend, axes); exposed
    /// directly on the chart through `Deref`, e.g. `chart.title.text`.
    pub base: ChartBase,
    /// The scales of the rows: their format, their font and their color are
    /// those of the first of these. Their ticks are round numbers that the
    /// chart picks, between the `min` and the `max` of each bullet.
    pub y_axis_configs: Vec<YAxisConfig>,

    /// The measures, one row each, from the top down.
    pub bullets: Vec<Bullet>,
    /// Colors of the bands, from the first one on. Without them the bands
    /// are shades of the text color, the first one the darkest.
    pub range_colors: Vec<Color>,
    /// Color of the target lines; without one, that of the title.
    pub target_color: Option<Color>,
    /// Writes the value of each row to the right of it, as
    /// `series.label.formatter` formats it: `{c}` the value, `{b}` the name.
    pub label_show: bool,
}

impl BulletChart {
    /// Creates a bullet chart.
    pub fn new(bullets: Vec<Bullet>) -> BulletChart {
        BulletChart::new_with_theme(bullets, &get_default_theme_name())
    }
    /// Creates a bullet chart with custom theme.
    pub fn new_with_theme(bullets: Vec<Bullet>, theme: &str) -> BulletChart {
        let mut c = BulletChart {
            bullets,
            ..Default::default()
        };
        let theme = get_theme(theme);
        c.base.fill_theme(theme, &mut c.y_axis_configs);
        c
    }
    /// Creates a bullet chart from JSON.
    pub fn from_json(data: &str) -> canvas::Result<BulletChart> {
        let mut c = BulletChart::default();
        let data = c
            .base
            .fill_option(data, &mut c.y_axis_configs, super::schema::BULLET_FIELDS)?;
        if let Some(bullets) = data.get("bullets").and_then(|v| v.as_array()) {
            c.bullets = bullets.iter().filter_map(bullet_from_value).collect();
        }
        if let Some(colors) = get_color_slice_from_value(&data, "range_colors") {
            c.range_colors = colors;
        }
        if let Some(color) = get_color_from_value(&data, "target_color") {
            c.target_color = Some(color);
        }
        if let Some(show) = get_bool_from_value(&data, "label_show") {
            c.label_show = show;
        }
        Ok(c)
    }

    /// The scale of a row: from 0, or from below it where a value is, to
    /// past the largest of its value, its target and the ends of its bands,
    /// unless the bullet says where it starts or ends. Its ticks are round
    /// numbers.
    fn scale_of(&self, bullet: &Bullet, config: &YAxisConfig) -> AxisValues {
        let mut data: Vec<f32> = bullet.ranges.clone();
        data.push(bullet.value);
        data.extend(bullet.target);
        data.retain(|value| value.is_finite() && *value != NIL_VALUE);
        let (min, max, parts) = round_scale(
            bullet.min.filter(|min| min.is_finite()),
            bullet.max.filter(|max| max.is_finite()),
            &data,
        );
        let mut params = axis_value_params(config, data, false);
        params.min = Some(min);
        params.max = Some(max);
        params.split_number = parts;
        params.scale = AxisScale::Linear;
        params.inverse = false;
        get_axis_values(params)
    }

    /// Renders the chart to an SVG string.
    pub fn svg(&self) -> canvas::Result<String> {
        let mut c = self.new_canvas();
        let axis_top = self.render_header(&mut c);
        let mut content = c.child(Box {
            top: axis_top,
            ..Default::default()
        });
        if self.bullets.is_empty() {
            self.render_empty_text(content);
            return c.svg();
        }
        let (width, height) = (content.width(), content.height());
        if width <= 0.0 || height <= 0.0 {
            return c.svg();
        }
        let config = get_y_axis_config(&self.y_axis_configs, 0);
        let measure = |text: &str, font_size: f32| -> f32 {
            measure_text_width_family(&self.font_family, font_size, text)
                .map(|b| b.width())
                .unwrap_or_default()
        };

        // The names to the left, as wide as the widest of them and a third
        // of the chart at the most; the values, when they are written, to
        // the right; the bands in between.
        let name_font = &self.x_axis.font;
        let name_width = self
            .bullets
            .iter()
            .map(|bullet| measure(&bullet.name, name_font.size))
            .fold(0.0_f32, f32::max)
            .min(width / 3.0);
        let name_column = if name_width > 0.0 {
            name_width + NAME_GAP
        } else {
            0.0
        };
        let label_font = &self.series.label.font;
        let labels: Vec<String> = self
            .bullets
            .iter()
            .map(|bullet| {
                let shown =
                    self.label_show && bullet.value.is_finite() && bullet.value != NIL_VALUE;
                if shown {
                    format_series_label(
                        &self.series.label.formatter,
                        bullet.value,
                        &bullet.name,
                        &bullet.name,
                    )
                } else {
                    String::new()
                }
            })
            .collect();
        let label_width = labels
            .iter()
            .map(|label| measure(label, label_font.size))
            .fold(0.0_f32, f32::max);
        let label_column = if label_width > 0.0 {
            label_width + LABEL_GAP
        } else {
            0.0
        };
        let plot_left = name_column;
        let plot_width = width - name_column - label_column;
        if plot_width <= 0.0 {
            return c.svg();
        }

        // A row each, as high as the plot shares out. Below its bands is its
        // scale, where there is room for one.
        let row = height / self.bullets.len() as f32;
        let axis_height = if self.x_axis.hidden {
            0.0
        } else {
            config.font.size + config.name_gap + 5.0
        };
        let axis_height = if row - axis_height < 8.0 {
            0.0
        } else {
            axis_height
        };
        let band = ((row - axis_height) * 0.7).min(MAX_BAND);
        if band <= 0.0 {
            return c.svg();
        }
        let bar_color = get_color(&self.series.colors, 0);
        let target_color = self.target_color.unwrap_or(self.title.font.color);
        let shade = self.x_axis.font.color;

        for (index, bullet) in self.bullets.iter().enumerate() {
            let band_top = row * index as f32 + (row - axis_height - band) / 2.0;
            let middle = band_top + band / 2.0;
            let scale = self.scale_of(bullet, &config);
            // Where a value is along the row, kept on it.
            let x_of = |value: f32| -> f32 {
                plot_left
                    + (plot_width - scale.get_offset_height(value, plot_width))
                        .clamp(0.0, plot_width)
            };
            let on_scale = |value: f32| {
                value.is_finite()
                    && value != NIL_VALUE
                    && value >= scale.min.min(scale.max)
                    && value <= scale.max.max(scale.min)
            };

            // The bands: from the start of the scale to the end of the first
            // range, from there to the end of the next one, and so on.
            let mut ends: Vec<f32> = bullet
                .ranges
                .iter()
                .copied()
                .filter(|end| end.is_finite())
                .collect();
            ends.sort_by(f32::total_cmp);
            if ends.is_empty() {
                ends.push(scale.max);
            }
            let count = ends.len();
            let mut from = scale.min;
            for (k, end) in ends.into_iter().enumerate() {
                let fill = if self.range_colors.is_empty() {
                    // From a third of the text color for the first band down
                    // to an eighth for the last.
                    let share = if count > 1 {
                        k as f32 / (count - 1) as f32
                    } else {
                        1.0
                    };
                    shade.with_alpha((84.0 - share * 52.0) as u8)
                } else {
                    get_color(&self.range_colors, k)
                };
                let (left, right) = (x_of(from), x_of(end));
                if right > left {
                    content.rect(Rect {
                        fill: Some(fill.into()),
                        left,
                        top: band_top,
                        width: right - left,
                        height: band,
                        ..Default::default()
                    });
                }
                from = end;
            }

            // The bar, from 0 where 0 is on the scale, from its start where
            // it is not.
            if bullet.value.is_finite() && bullet.value != NIL_VALUE {
                let base = 0.0_f32.clamp(scale.min.min(scale.max), scale.max.max(scale.min));
                let (start, end) = (x_of(base), x_of(bullet.value));
                let thickness = (band / 3.0).max(1.0);
                let title = self
                    .tooltip
                    .show
                    .then(|| format!("{}: {}", bullet.name, format_float(bullet.value)));
                let mut dataset = vec![
                    ("name".to_string(), bullet.name.clone()),
                    ("value".to_string(), format_float(bullet.value)),
                ];
                if let Some(target) = bullet.target.filter(|t| t.is_finite()) {
                    dataset.push(("target".to_string(), format_float(target)));
                }
                content.rect(Rect {
                    fill: Some(bullet.color.unwrap_or(bar_color).into()),
                    left: start.min(end),
                    top: middle - thickness / 2.0,
                    width: (end - start).abs(),
                    height: thickness,
                    title: title.clone(),
                    class: title.as_ref().map(|_| "ct-trigger".to_string()),
                    dataset,
                    ..Default::default()
                });
                if let Some(text) = title {
                    content.text(Text {
                        text,
                        class: Some("ct-tip".to_string()),
                        font_weight: self.tooltip.font.weight.clone(),
                        font_family: Some(self.font_family.clone()),
                        font_color: Some(self.tooltip_font_color(label_font.color)),
                        font_size: Some(self.tooltip_font_size(label_font.size)),
                        x: Some(end),
                        y: Some(band_top),
                        dy: Some(-4.0),
                        text_anchor: Some("middle".to_string()),
                        ..Default::default()
                    });
                }
            }

            // The target, across the bar and most of the bands.
            if let Some(target) = bullet.target.filter(|target| on_scale(*target)) {
                let x = x_of(target);
                content.line(Line {
                    color: Some(target_color),
                    stroke_width: (band * 0.08).max(2.0),
                    left: x,
                    top: band_top + band * 0.15,
                    right: x,
                    bottom: band_top + band * 0.85,
                    ..Default::default()
                });
            }

            // The name, cut to its column, and the value.
            if name_column > 0.0 && !bullet.name.is_empty() {
                content.text(Text {
                    text: text_ellipsis(
                        &self.font_family,
                        name_font.size,
                        &bullet.name,
                        name_width,
                    ),
                    font_family: Some(self.font_family.clone()),
                    font_size: Some(name_font.size),
                    font_color: Some(name_font.color),
                    font_weight: name_font.weight.clone(),
                    x: Some(name_width),
                    y: Some(middle),
                    text_anchor: Some("end".to_string()),
                    dominant_baseline: Some("central".to_string()),
                    ..Default::default()
                });
            }
            if !labels[index].is_empty() {
                content.text(Text {
                    text: labels[index].clone(),
                    font_family: Some(self.font_family.clone()),
                    font_size: Some(label_font.size),
                    font_color: Some(label_font.color),
                    font_weight: label_font.weight.clone(),
                    x: Some(plot_left + plot_width + LABEL_GAP),
                    y: Some(middle),
                    dominant_baseline: Some("central".to_string()),
                    ..Default::default()
                });
            }

            // The scale, below the bands.
            if axis_height > 0.0 && !scale.data.is_empty() {
                content
                    .child(Box {
                        left: plot_left,
                        top: band_top + band,
                        ..Default::default()
                    })
                    .axis(Axis {
                        position: Position::Bottom,
                        height: axis_height,
                        width: plot_width,
                        split_number: scale.data.len().saturating_sub(1).max(1),
                        font_family: self.font_family.clone(),
                        stroke_color: Some(config.stroke_color),
                        stroke_width: config.stroke_width.unwrap_or(1.0),
                        name_align: Align::Left,
                        name_gap: config.name_gap,
                        font_color: Some(config.font.color),
                        font_size: config.font.size,
                        font_weight: config.font.weight.clone(),
                        data: scale.data.clone(),
                        formatter: config.formatter.clone(),
                        ..Default::default()
                    });
            }
        }

        if self.tooltip.show {
            c.svg_with_style(TOOLTIP_STYLE)
        } else {
            c.svg()
        }
    }
}

impl std::ops::Deref for BulletChart {
    type Target = ChartBase;
    fn deref(&self) -> &ChartBase {
        &self.base
    }
}
impl std::ops::DerefMut for BulletChart {
    fn deref_mut(&mut self) -> &mut ChartBase {
        &mut self.base
    }
}

#[cfg(test)]
mod tests {
    use super::round_scale;

    #[test]
    fn a_scale_ends_at_a_round_number() {
        // From 0, in four to six round parts, ending at or just past the
        // largest value: of those, the one that ends nearest, and then the
        // one nearer five parts.
        assert_eq!(
            (0.0, 2500.0, 5),
            round_scale(None, None, &[1820.0, 2000.0, 2400.0])
        );
        assert_eq!((0.0, 30.0, 6), round_scale(None, None, &[12.0, 10.0, 30.0]));
        assert_eq!((0.0, 100.0, 5), round_scale(None, None, &[92.0, 95.0]));
        assert_eq!((0.0, 1.0, 5), round_scale(None, None, &[0.42, 0.9]));
        // Below 0 where a value is, at a round number.
        assert_eq!(
            (-20.0, 40.0, 6),
            round_scale(None, None, &[-12.0, 30.0, 40.0])
        );
        // Nothing to go by: from 0 to 1.
        assert_eq!((0.0, 1.0, 5), round_scale(None, None, &[]));
    }

    #[test]
    fn a_given_scale_is_kept_and_split_where_it_can_be() {
        assert_eq!((0.0, 100.0, 5), round_scale(None, Some(100.0), &[92.0]));
        assert_eq!(
            (50.0, 100.0, 5),
            round_scale(Some(50.0), Some(100.0), &[72.0])
        );
        assert_eq!((0.0, 7.0, 7), round_scale(None, Some(7.0), &[3.0]));
        assert_eq!((0.0, 12.0, 6), round_scale(None, Some(12.0), &[3.0]));
        // One that cannot be split into round parts is split into five.
        assert_eq!((0.0, 13.0, 5), round_scale(None, Some(13.0), &[3.0]));
        // An end before the start is a scale of one.
        assert_eq!((10.0, 11.0, 5), round_scale(Some(10.0), Some(5.0), &[3.0]));
        // A start that is given is where it starts, values below it or not.
        assert_eq!(
            (20.0, 100.0, 4),
            round_scale(Some(20.0), None, &[-5.0, 100.0])
        );
    }
}
