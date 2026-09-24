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
use super::base::{ChartBase, axis_value_params, get_y_axis_config, mark_statistics};
use super::canvas;
use super::color::*;
use super::common::*;
use super::component::*;
use super::params::*;
use super::theme::{get_default_theme_name, get_theme};
use super::util::*;
use crate::charts::measure_text_width_family;

/// A horizontal bar chart: categories on the y axis, values on the x axis.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct HorizontalBarChart {
    /// The shared chart options (size, series, title/legend, axes); exposed
    /// directly on the chart through `Deref`, e.g. `chart.title_text`.
    pub base: ChartBase,
    // x axis

    // y axis
    /// Y axis configurations; one per axis, up to two.
    pub y_axis_configs: Vec<YAxisConfig>,

    // grid

    // series
    /// Position of the series value labels.
    pub series_label_position: Option<Position>,
}

impl std::ops::Deref for HorizontalBarChart {
    type Target = ChartBase;
    fn deref(&self) -> &ChartBase {
        &self.base
    }
}
impl std::ops::DerefMut for HorizontalBarChart {
    fn deref_mut(&mut self) -> &mut ChartBase {
        &mut self.base
    }
}

impl HorizontalBarChart {
    /// Creates a horizontal bar from json.
    pub fn from_json(data: &str) -> canvas::Result<HorizontalBarChart> {
        let mut h = HorizontalBarChart {
            ..Default::default()
        };
        let value = h.base.fill_option(
            data,
            &mut h.y_axis_configs,
            super::schema::HORIZONTAL_BAR_FIELDS,
        )?;
        if let Some(series_label_position) =
            get_position_from_value(&value, "series_label_position")
        {
            h.series_label_position = Some(series_label_position);
        }
        Ok(h)
    }
    /// Creates a horizontal bar with custom theme.
    pub fn new_with_theme(
        series_list: Vec<Series>,
        x_axis_data: Vec<String>,
        theme: &str,
    ) -> HorizontalBarChart {
        let mut h = HorizontalBarChart {
            ..Default::default()
        };
        h.series_list = series_list;
        h.x_axis_data = x_axis_data;
        let theme = get_theme(theme);
        h.base.fill_theme(theme, &mut h.y_axis_configs);
        h
    }
    /// Creates a horizontal bar with default theme.
    pub fn new(series_list: Vec<Series>, x_axis_data: Vec<String>) -> HorizontalBarChart {
        HorizontalBarChart::new_with_theme(series_list, x_axis_data, &get_default_theme_name())
    }
    /// Converts horizontal bar chart to svg.
    pub fn svg(&self) -> canvas::Result<String> {
        let mut c = Canvas::new_width_xy(self.width, self.height, self.x, self.y);

        let axis_top = self.render_header(&mut c);

        // The value axis is the base's "x axis" (bottom) and the category
        // axis its "y axis" (left), so `x_axis_*` / `y_axis_*` options keep
        // their meaning of "the axis at the bottom / on the left".
        let x_axis_height = if self.x_axis_hidden {
            0.0
        } else {
            self.x_axis_height
        };
        let axis_height = c.height() - axis_top - x_axis_height;
        // minus the height of top text area
        if axis_top > 0.0 {
            c = c.child(Box {
                top: axis_top,
                ..Default::default()
            });
        }

        let mut y_axis_width = 0.0;
        if !self.y_axis_hidden {
            let mut data = self.x_axis_data.clone();
            data.reverse();
            let mut max_width = 0.0;
            for text in data.iter() {
                if let Ok(b) =
                    measure_text_width_family(&self.font_family, self.x_axis_font_size, text)
                    && b.width() > max_width
                {
                    max_width = b.width();
                }
            }
            y_axis_width = max_width + 5.0;
            c.axis(Axis {
                position: Position::Left,
                height: axis_height,
                width: y_axis_width,
                split_number: self.x_axis_data.len(),
                font_family: self.font_family.clone(),
                stroke_color: Some(self.x_axis_stroke_color),
                name_align: Align::Center,
                name_gap: self.x_axis_name_gap,
                font_color: Some(self.x_axis_font_color),
                font_size: self.x_axis_font_size,
                font_weight: self.x_axis_font_weight.clone(),
                data,
                ..Default::default()
            });
        }

        let category_count = self.x_axis_data.len().max(1);
        // Assign each series a slot: stacked series share one. Positive and
        // negative values stack separately, away from the 0 line.
        let mut stack_slot_keys: Vec<String> = vec![];
        let mut series_slot_indices: Vec<usize> = Vec::with_capacity(self.series_list.len());
        let mut series_stack_indices: Vec<Option<usize>> =
            Vec::with_capacity(self.series_list.len());
        let mut slot_count = 0_usize;
        for series in self.series_list.iter() {
            if let Some(ref stack) = series.stack {
                let key = format!("{}_{}", stack, series.y_axis_index);
                let pos = stack_slot_keys.iter().position(|k| k == &key);
                let stack_index = pos.unwrap_or(stack_slot_keys.len());
                if pos.is_none() {
                    stack_slot_keys.push(key);
                }
                series_slot_indices.push(stack_index);
                series_stack_indices.push(Some(stack_index));
            } else {
                series_slot_indices.push(slot_count);
                series_stack_indices.push(None);
            }
            slot_count += usize::from(series.stack.is_none());
        }
        // Stacked slots are numbered first, plain series after them.
        let stack_count = stack_slot_keys.len();
        for (slot, stack) in series_slot_indices
            .iter_mut()
            .zip(series_stack_indices.iter())
        {
            if stack.is_none() {
                *slot += stack_count;
            }
        }
        slot_count += stack_count;

        // The value axis has to cover the stacked totals on both sides of 0.
        let mut data_list = vec![];
        let mut stack_sums: Vec<Vec<(f32, f32)>> =
            vec![vec![(0.0_f32, 0.0_f32); category_count]; stack_count];
        for (index, series) in self.series_list.iter().enumerate() {
            let values = series.data_values();
            match series_stack_indices[index] {
                Some(stack_index) => {
                    for (i, &v) in values.iter().enumerate() {
                        let actual_i = i.saturating_add(series.start_index);
                        if v == NIL_VALUE || actual_i >= category_count {
                            continue;
                        }
                        let entry = &mut stack_sums[stack_index][actual_i];
                        if v >= 0.0 {
                            entry.0 += v;
                        } else {
                            entry.1 += v;
                        }
                    }
                }
                None => data_list.extend(values),
            }
        }
        for sums in stack_sums.iter() {
            for &(pos, neg) in sums.iter() {
                data_list.push(pos);
                if neg < 0.0 {
                    data_list.push(neg);
                }
            }
        }
        let x_axis_config = get_y_axis_config(&self.y_axis_configs, 0);
        let x_axis_values = get_axis_values(axis_value_params(&x_axis_config, data_list, false));

        let x_axis_width = c.width() - y_axis_width;
        if !self.x_axis_hidden {
            c.child(Box {
                left: y_axis_width,
                top: axis_height,
                ..Default::default()
            })
            .axis(Axis {
                position: Position::Bottom,
                height: x_axis_height,
                width: x_axis_width,
                split_number: x_axis_config.axis_split_number,
                font_family: self.font_family.clone(),
                stroke_color: Some(x_axis_config.axis_stroke_color),
                name_align: Align::Left,
                name_gap: x_axis_config.axis_name_gap,
                font_color: Some(x_axis_config.axis_font_color),
                font_size: x_axis_config.axis_font_size,
                font_weight: x_axis_config.axis_font_weight.clone(),
                data: x_axis_values.data.clone(),
                formatter: x_axis_config.axis_formatter.clone(),
                ..Default::default()
            });
        }

        c.child(Box {
            left: y_axis_width,
            ..Default::default()
        })
        .grid(Grid {
            right: x_axis_width,
            bottom: axis_height,
            color: Some(self.grid_stroke_color),
            stroke_width: self.grid_stroke_width,
            verticals: x_axis_config.axis_split_number,
            hidden_verticals: vec![0],
            ..Default::default()
        });

        // horizontal bar
        if slot_count > 0 {
            let mut c1 = c.child(Box {
                left: y_axis_width,
                bottom: x_axis_height,
                ..Default::default()
            });
            let max_width = c1.width();
            // Row count is the number of categories (the y-axis ticks use
            // `x_axis_data.len()`), not the first series' length: a short/empty
            // first series must not divide by zero or misplace every bar.
            let unit_height = c1.height() / category_count as f32;
            let bar_chart_margin = 5.0_f32;
            let bar_chart_gap = 3.0_f32;
            let bar_chart_min_height = 1.0_f32;

            let bar_chart_margin_height = bar_chart_margin * 2.0;
            let bar_chart_gap_height = bar_chart_gap * (slot_count - 1) as f32;
            // A narrow band (many categories in a small canvas) has less room
            // than the fixed margins and gaps need, which would give a negative
            // bar height. Shrink the margins and gaps together, just enough to
            // keep each bar `bar_chart_min_height` tall, so the group stays
            // inside its band. With room to spare `scale` is 1 and the layout
            // is unchanged.
            let bar_chart_spacing = bar_chart_margin_height + bar_chart_gap_height;
            let scale = ((unit_height - bar_chart_min_height * slot_count as f32)
                / bar_chart_spacing)
                .clamp(0.0, 1.0);
            let bar_chart_margin = bar_chart_margin * scale;
            let bar_chart_gap = bar_chart_gap * scale;
            let bar_height = (unit_height - bar_chart_spacing * scale) / slot_count as f32;
            let half_bar_height = bar_height / 2.0;

            // Bars grow from the 0 line so negative values extend to the left
            // of it; with a positive `axis_min` there is no 0 and the axis
            // start is used, as before.
            let zero_x =
                (max_width - x_axis_values.get_offset_height(0.0, max_width)).clamp(0.0, max_width);
            let zero_x = if zero_x.is_finite() { zero_x } else { 0.0 };
            let value_x = |value: f32| -> f32 {
                if value == 0.0 {
                    zero_x
                } else {
                    max_width - x_axis_values.get_offset_height(value, max_width)
                }
            };
            // Running (positive, negative) sums per stack and row.
            let mut stack_acc: Vec<Vec<(f32, f32)>> =
                vec![vec![(0.0_f32, 0.0_f32); category_count]; stack_count];

            let mut series_labels_list = vec![];
            for (index, series) in self.series_list.iter().enumerate() {
                let slot_index = series_slot_indices[index];
                let stack_index = series_stack_indices[index];
                let color = get_color(&self.series_colors, series.index.unwrap_or(index));

                let mut series_labels = vec![];
                for (i, p) in series.data_values().iter().enumerate() {
                    let value = p.to_owned();
                    if value == NIL_VALUE {
                        continue;
                    }
                    // Rows are categories (bottom-up), shared by every series;
                    // a shorter series must not shift its bars onto other rows.
                    let actual_i = i.saturating_add(series.start_index);
                    if actual_i >= category_count {
                        continue;
                    }
                    let mut top =
                        unit_height * (category_count - actual_i - 1) as f32 + bar_chart_margin;
                    top += (bar_height + bar_chart_gap) * slot_index as f32;

                    let base = match stack_index {
                        Some(si) => {
                            let (pos, neg) = stack_acc[si][actual_i];
                            if value >= 0.0 { pos } else { neg }
                        }
                        None => 0.0,
                    };
                    let x_base = value_x(base);
                    let x = value_x(base + value);
                    if let Some(si) = stack_index {
                        let acc = &mut stack_acc[si][actual_i];
                        if value >= 0.0 {
                            acc.0 += value;
                        } else {
                            acc.1 += value;
                        }
                    }

                    let fill_color = series
                        .colors
                        .as_ref()
                        .and_then(|colors| colors.get(i).copied().flatten())
                        .unwrap_or(color);
                    let label = self.format_series_label(series, actual_i, value);
                    let tip = if self.tooltip_show {
                        Some(format!("{}: {}", series.name, label))
                    } else {
                        None
                    };
                    let mut classes: Vec<&str> = vec![];
                    if self.animation.is_some() {
                        classes.push("bar-anim");
                    }
                    if tip.is_some() {
                        classes.push("ct-trigger");
                    }
                    let class = if classes.is_empty() {
                        None
                    } else {
                        Some(classes.join(" "))
                    };
                    // Negative bars grow leftwards from the 0 line.
                    let style = self.animation.as_ref().map(|a| {
                        let origin = if value < 0.0 { "right" } else { "left" };
                        format!(
                            "transform-origin:{origin} center;animation-delay:{}ms",
                            actual_i as u32 * a.delay
                        )
                    });
                    c1.rect(Rect {
                        fill: Some(fill_color.into()),
                        left: x.min(x_base),
                        top,
                        width: (x - x_base).abs(),
                        height: bar_height,
                        title: tip.clone(),
                        class,
                        style,
                        dataset: self.point_dataset(series, actual_i, value),
                        ..Default::default()
                    });
                    if let Some(text) = tip {
                        c1.text(Text {
                            text,
                            class: Some("ct-tip".to_string()),
                            font_family: Some(self.font_family.clone()),
                            font_color: Some(self.series_label_font_color),
                            font_size: Some(self.series_label_font_size),
                            x: Some(x),
                            y: Some(top + half_bar_height),
                            dy: Some(-4.0),
                            text_anchor: Some("end".to_string()),
                            ..Default::default()
                        });
                    }
                    series_labels.push(SeriesLabel {
                        point: (x, top + half_bar_height).into(),
                        text: label,
                    })
                }
                if series.label_show {
                    series_labels_list.push(series_labels);
                }
            }

            let series_label_position = self
                .series_label_position
                .clone()
                .unwrap_or(Position::Right);
            for series_labels in series_labels_list.iter() {
                for series_label in series_labels.iter() {
                    let mut dy = None;
                    let mut dx = Some(3.0);
                    let mut x = Some(series_label.point.x);
                    if let Ok(value) = measure_text_width_family(
                        &self.font_family,
                        self.series_label_font_size,
                        &series_label.text,
                    ) {
                        dy = Some(value.height() / 2.0 - 2.0);
                        if series_label_position == Position::Inside {
                            dx = None;
                            let offset = series_label.point.x - value.width();
                            if offset <= 0.0 {
                                x = Some(1.0);
                            } else {
                                x = Some(offset / 2.0);
                            }
                        } else if series_label_position == Position::Left {
                            x = Some(0.0);
                            dx = Some(-value.width());
                        }
                    }
                    c1.text(Text {
                        text: series_label.text.clone(),
                        dx,
                        dy,
                        font_family: Some(self.font_family.clone()),
                        font_color: Some(self.series_label_font_color),
                        font_size: Some(self.series_label_font_size),
                        font_weight: self.series_label_font_weight.clone(),
                        x,
                        y: Some(series_label.point.y),
                        ..Default::default()
                    });
                }
            }
        }

        // Mark lines and areas run vertically at their value.
        if slot_count > 0 {
            let mut c1 = c.child(Box {
                left: y_axis_width,
                bottom: x_axis_height,
                ..Default::default()
            });
            let max_width = c1.width();
            let max_height = c1.height();
            for (index, series) in self.series_list.iter().enumerate() {
                if series.mark_lines.is_empty() && series.mark_areas.is_empty() {
                    continue;
                }
                let color = get_color(&self.series_colors, series.index.unwrap_or(index));
                let values: Vec<f32> = series
                    .data_values()
                    .into_iter()
                    .filter(|v| *v != NIL_VALUE)
                    .collect();
                let stat = mark_statistics(&values);
                let value_x =
                    |value: f32| max_width - x_axis_values.get_offset_height(value, max_width);
                for mark_area in series.mark_areas.iter() {
                    let (Some(from), Some(to)) = (stat(&mark_area.from), stat(&mark_area.to))
                    else {
                        continue;
                    };
                    let (x_from, x_to) = (value_x(from), value_x(to));
                    c1.rect(Rect {
                        fill: Some(color.with_alpha(40).into()),
                        left: x_from.min(x_to),
                        top: 0.0,
                        width: (x_from - x_to).abs(),
                        height: max_height,
                        ..Default::default()
                    });
                }
                for mark_line in series.mark_lines.iter() {
                    let Some(value) = stat(&mark_line.category) else {
                        continue;
                    };
                    let x = value_x(value);
                    c1.circle(Circle {
                        stroke_color: Some(color),
                        fill: Some(color),
                        cx: x,
                        cy: max_height - 3.0,
                        r: 3.5,
                        ..Default::default()
                    });
                    c1.line(Line {
                        color: Some(color),
                        left: x,
                        top: 4.0,
                        right: x,
                        bottom: max_height - 8.0,
                        stroke_dash_array: Some("4,2".to_string()),
                        ..Default::default()
                    });
                    c1.text(Text {
                        text: format_float(value),
                        font_family: Some(self.font_family.clone()),
                        font_size: Some(self.series_label_font_size),
                        font_color: Some(self.series_label_font_color),
                        x: Some(x),
                        y: Some(self.series_label_font_size),
                        dx: Some(4.0),
                        ..Default::default()
                    });
                }
            }
        }

        let mut css = String::new();
        if let Some(ref anim) = self.animation {
            css.push_str(&format!(
                "@keyframes hbar-grow{{from{{transform:scaleX(0)}}to{{transform:scaleX(1)}}}} \
                 .bar-anim{{transform-box:fill-box;animation:hbar-grow {}ms {} both}} ",
                anim.duration,
                anim.safe_easing()
            ));
        }
        if self.tooltip_show {
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
    use super::HorizontalBarChart;
    use crate::{Align, NIL_VALUE, Position};
    #[test]
    fn horizontal_bar_chart_basic() {
        let mut horizontal_bar_chart = HorizontalBarChart::new(
            vec![
                (
                    "2011",
                    vec![18203.0, 23489.0, 29034.0, 104970.0, 131744.0, 630230.0],
                )
                    .into(),
                (
                    "2012",
                    vec![19325.0, 23438.0, 31000.0, 121594.0, 134141.0, 681807.0],
                )
                    .into(),
            ],
            vec![
                "Brazil".to_string(),
                "Indonesia".to_string(),
                "USA".to_string(),
                "India".to_string(),
                "China".to_string(),
                "World".to_string(),
            ],
        );
        horizontal_bar_chart.title_text = "World Population".to_string();
        horizontal_bar_chart.series_label_formatter = "{t}".to_string();
        horizontal_bar_chart.margin.right = 15.0;
        horizontal_bar_chart.series_list[0].label_show = true;
        horizontal_bar_chart.title_align = Align::Left;
        assert_snapshot!(
            "horizontal_bar_chart/basic.svg",
            horizontal_bar_chart.svg().unwrap()
        );
    }

    #[test]
    fn horizontal_bar_chart_inside() {
        let mut horizontal_bar_chart = HorizontalBarChart::new(
            vec![
                (
                    "2011",
                    vec![18203.0, 23489.0, 29034.0, 104970.0, 131744.0, 630230.0],
                )
                    .into(),
                (
                    "2012",
                    vec![19325.0, 23438.0, 31000.0, 121594.0, 134141.0, 681807.0],
                )
                    .into(),
            ],
            vec![
                "Brazil".to_string(),
                "Indonesia".to_string(),
                "USA".to_string(),
                "India".to_string(),
                "China".to_string(),
                "World".to_string(),
            ],
        );
        horizontal_bar_chart.title_text = "World Population".to_string();
        horizontal_bar_chart.series_label_formatter = "{t}".to_string();
        horizontal_bar_chart.margin.right = 15.0;
        horizontal_bar_chart.series_list[0].label_show = true;
        horizontal_bar_chart.title_align = Align::Left;
        horizontal_bar_chart.series_label_position = Some(Position::Inside);
        assert_snapshot!(
            "horizontal_bar_chart/basic_label_inside.svg",
            horizontal_bar_chart.svg().unwrap()
        );
    }

    #[test]
    fn horizontal_bar_chart_nil_value() {
        let mut horizontal_bar_chart = HorizontalBarChart::new(
            vec![
                (
                    "2011",
                    vec![18203.0, 23489.0, NIL_VALUE, 104970.0, 131744.0, 630230.0],
                )
                    .into(),
                (
                    "2012",
                    vec![19325.0, 23438.0, 31000.0, 121594.0, NIL_VALUE, 681807.0],
                )
                    .into(),
            ],
            vec![
                "Brazil".to_string(),
                "Indonesia".to_string(),
                "USA".to_string(),
                "India".to_string(),
                "China".to_string(),
                "World".to_string(),
            ],
        );
        horizontal_bar_chart.title_text = "World Population".to_string();
        horizontal_bar_chart.margin.right = 15.0;
        horizontal_bar_chart.series_list[0].label_show = true;
        horizontal_bar_chart.title_align = Align::Left;
        assert_snapshot!(
            "horizontal_bar_chart/nil_value.svg",
            horizontal_bar_chart.svg().unwrap()
        );
    }

    #[test]
    fn horizontal_bar_chart_tooltip() {
        let chart = HorizontalBarChart::from_json(
            r#"{"tooltip_show": true, "series_list": [{"name": "A", "data": [1]}], "x_axis_data": ["x"]}"#,
        )
        .unwrap();
        let svg = chart.svg().unwrap();
        assert!(svg.contains("<title>A: 1</title>"), "missing hbar title");
        assert!(svg.contains(r#"class="ct-tip""#), "missing hover label");
        assert!(
            svg.contains(".ct-trigger:hover+.ct-tip"),
            "missing hover css"
        );
        let off = HorizontalBarChart::from_json(
            r#"{"series_list": [{"name": "A", "data": [1]}], "x_axis_data": ["x"]}"#,
        )
        .unwrap();
        let off_svg = off.svg().unwrap();
        assert!(!off_svg.contains("<title>"));
        assert!(!off_svg.contains("ct-tip"));
    }

    // An empty first series must not divide by zero (row count comes from the
    // categories now), so later series still render finite bars.
    #[test]
    fn empty_first_series_no_inf() {
        let chart = HorizontalBarChart::from_json(
            r#"{"series_list":[{"name":"A","data":[]},{"name":"B","data":[10,20,30]}],"x_axis_data":["x","y","z"]}"#,
        )
        .unwrap();
        let svg = chart.svg().unwrap();
        assert!(!svg.contains("inf"), "empty first series must not emit inf");
        assert!(!svg.contains("NaN"), "empty first series must not emit NaN");
    }
}
