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
use super::x_axis::{ContinuousX, format_number, format_time_full};
use crate::charts::measure_text_width_family;

/// Corner radius of the bars unless set.
const DEFAULT_RADIUS: f32 = 3.0;
/// Share of the height of a row its bar takes unless `bar_height` is set.
const BAR_HEIGHT_RATIO: f32 = 0.6;
/// Room left free at both ends of the time axis, as a share of the time
/// the tasks span, so the first and the last bar do not touch its ends.
const RANGE_PADDING: f64 = 0.03;
/// Gap between the row names and the plot, and between a bar and a label
/// beside it.
const LABEL_GAP: f32 = 8.0;
/// Alpha of the part of a bar that is still to do, when its task tells its
/// progress.
const REMAINING_ALPHA: u8 = 110;

// ── Public data model ──────────────────────────────────────────────────────────

/// One task of a gantt chart: a bar from its `start` to its `end`.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct GanttTask {
    /// Name of the task.
    pub name: String,
    /// When the task starts: a unix timestamp in seconds (or a plain number
    /// with `x_axis.kind` set to `Value`).
    pub start: f64,
    /// When the task ends. A task that ends when it starts is a milestone,
    /// drawn as a diamond.
    pub end: f64,
    /// The row the task is drawn on. `None` gives the task a row of its own,
    /// named after it; tasks with the same row share it (the bookings of a
    /// room, the flights at a gate).
    pub row: Option<String>,
    /// Name of the category of the task: tasks of a category share a color,
    /// and the categories are the legend.
    pub category: Option<String>,
    /// How much of the task is done, from 0 to 1: that part of the bar is
    /// drawn in the full color, the rest lighter.
    pub progress: Option<f32>,
    /// Optional explicit color; when `None` the color is the one of the
    /// category.
    pub color: Option<Color>,
}

impl From<(&str, f64, f64)> for GanttTask {
    fn from(v: (&str, f64, f64)) -> Self {
        GanttTask {
            name: v.0.to_string(),
            start: v.1,
            end: v.2,
            ..Default::default()
        }
    }
}

impl GanttTask {
    /// The name of the row the task is drawn on.
    fn row_name(&self) -> &str {
        self.row.as_deref().unwrap_or(&self.name)
    }
    /// The start and the end of the task in order, when it has a place on
    /// the axis at all.
    fn span(&self) -> Option<(f64, f64)> {
        if !self.start.is_finite() {
            return None;
        }
        // Without an end the task is a moment: a milestone.
        let end = if self.end.is_finite() {
            self.end
        } else {
            self.start
        };
        Some((self.start.min(end), self.start.max(end)))
    }
}

// ── GanttChart ─────────────────────────────────────────────────────────────────

/// A gantt chart: tasks as bars along a time axis, one row for each task or
/// several tasks on a row — a project plan, a schedule, a timeline.
///
/// The axis is a time axis (`start` and `end` are unix seconds; JSON also
/// takes date strings); with `x_axis.kind` set to `Value` it is an axis of
/// plain numbers. `x_axis.min` / `x_axis.max` fix its range.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct GanttChart {
    /// The shared chart options (size, series, title/legend, axes); exposed
    /// directly on the chart through `Deref`, e.g. `chart.title.text`.
    pub base: ChartBase,
    /// Configuration of the axis of the rows: the font of their names.
    pub y_axis_configs: Vec<YAxisConfig>,

    // gantt-specific
    /// The tasks. The rows are taken from them, top to bottom in the order
    /// they are first met.
    pub tasks: Vec<GanttTask>,
    /// Height of the bars in pixels; `None` (the default) makes them 60% as
    /// high as their row.
    pub bar_height: Option<f32>,
    /// Corner radius of the bars. Default: 3.0.
    pub radius: f32,
    /// Writes the name of a task on its bar (or beside it when it does not
    /// fit) when the task shares its row and the row is not named after
    /// it. Default: true.
    pub label_show: bool,
    /// A moment marked by a dashed line across the chart, such as today.
    pub now: Option<f64>,
}

impl std::ops::Deref for GanttChart {
    type Target = ChartBase;
    fn deref(&self) -> &ChartBase {
        &self.base
    }
}
impl std::ops::DerefMut for GanttChart {
    fn deref_mut(&mut self) -> &mut ChartBase {
        &mut self.base
    }
}

/// A bar or a milestone, placed on the plot.
struct Placed<'a> {
    task: &'a GanttTask,
    row: usize,
    /// Left and right edge in pixels; the same for a milestone.
    left: f32,
    right: f32,
    color: Color,
}

impl GanttChart {
    fn fill_default(&mut self) {
        self.radius = DEFAULT_RADIUS;
        self.label_show = true;
    }

    /// Creates a gantt chart with the default theme.
    pub fn new(tasks: Vec<GanttTask>) -> GanttChart {
        GanttChart::new_with_theme(tasks, &get_default_theme_name())
    }

    /// Creates a gantt chart with a custom theme.
    pub fn new_with_theme(tasks: Vec<GanttTask>, theme: &str) -> GanttChart {
        let mut c = GanttChart {
            tasks,
            ..Default::default()
        };
        c.fill_default();
        c.base.fill_theme(get_theme(theme), &mut c.y_axis_configs);
        c
    }

    /// Creates a gantt chart from a JSON string.
    pub fn from_json(json: &str) -> canvas::Result<GanttChart> {
        let mut c = GanttChart {
            ..Default::default()
        };
        c.fill_default();
        let value = c
            .base
            .fill_option(json, &mut c.y_axis_configs, super::schema::GANTT_FIELDS)?;
        let moment = |item: &serde_json::Value, key: &str| -> Option<f64> {
            item.get(key).and_then(get_x_value).map(|(v, _)| v)
        };
        if let Some(arr) = value.get("tasks").and_then(|v| v.as_array()) {
            c.tasks = arr
                .iter()
                .filter_map(|item| {
                    let start = moment(item, "start")?;
                    Some(GanttTask {
                        name: get_string_from_value(item, "name").unwrap_or_default(),
                        start,
                        // A task without an end is a milestone.
                        end: moment(item, "end").unwrap_or(start),
                        row: get_string_from_value(item, "row"),
                        category: get_string_from_value(item, "category"),
                        progress: get_f32_from_value(item, "progress"),
                        color: get_color_from_value(item, "color"),
                    })
                })
                .collect();
        }
        if let Some(v) = get_f32_from_value(&value, "bar_height") {
            c.bar_height = Some(v);
        }
        if let Some(v) = get_f32_from_value(&value, "radius") {
            c.radius = v;
        }
        if let Some(v) = get_bool_from_value(&value, "label_show") {
            c.label_show = v;
        }
        if let Some(v) = moment(&value, "now") {
            c.now = Some(v);
        }
        Ok(c)
    }

    /// The axis reads as time unless it is said to be one of plain numbers.
    fn is_time(&self) -> bool {
        self.x_axis.kind != AxisType::Value
    }

    /// A moment as it is told in tooltips and `data-*` attributes.
    fn moment_text(&self, value: f64) -> String {
        if self.is_time() {
            format_time_full(value + self.x_axis.time_offset as f64 * 60.0)
        } else {
            format_number(value, 6)
        }
    }

    /// Renders the chart to an SVG string.
    pub fn svg(&self) -> canvas::Result<String> {
        let mut c = self.new_canvas();
        let mut axis_top = self.render_header(&mut c);

        // The tasks that have a place on the axis, and the rows they are on.
        let tasks: Vec<(&GanttTask, (f64, f64))> = self
            .tasks
            .iter()
            .filter_map(|task| task.span().map(|span| (task, span)))
            .collect();
        if tasks.is_empty() {
            // The series list of a gantt chart is always empty: say so here.
            self.render_empty_text(c.child(Box {
                top: axis_top,
                ..Default::default()
            }));
            return c.svg();
        }
        let mut rows: Vec<&str> = vec![];
        let mut categories: Vec<&str> = vec![];
        for (task, _) in tasks.iter() {
            if !rows.contains(&task.row_name()) {
                rows.push(task.row_name());
            }
            if let Some(category) = task.category.as_deref()
                && !categories.contains(&category)
            {
                categories.push(category);
            }
        }
        // The categories are the legend of a gantt chart.
        if !categories.is_empty() && self.legend.show.unwrap_or(true) {
            let entries: Vec<(&str, Color)> = categories
                .iter()
                .enumerate()
                .map(|(i, name)| (*name, get_color(&self.series.colors, i)))
                .collect();
            axis_top += self.render_legend_entries(
                c.child(Box {
                    top: axis_top,
                    ..Default::default()
                }),
                &entries,
            );
        }

        let x_axis_height = if self.x_axis.hidden {
            0.0
        } else {
            self.x_axis.height
        };
        let titles =
            self.reserve_axis_titles(&mut c, &self.y_axis_configs, &self.x_axis.title, false);
        let plot_height = c.height() - x_axis_height - axis_top;
        if axis_top > 0.0 {
            c = c.child(Box {
                top: axis_top,
                ..Default::default()
            });
        }
        let measure = |text: &str, font_size: f32| -> f32 {
            measure_text_width_family(&self.font_family, font_size, text)
                .map(|b| b.width())
                .unwrap_or_default()
        };

        // The names of the rows, left of the plot.
        let row_config = super::base::get_y_axis_config(&self.y_axis_configs, 0);
        let y_axis_width = if self.y_axis_hidden {
            0.0
        } else {
            rows.iter()
                .map(|row| measure(row, row_config.font.size))
                .fold(0.0, f32::max)
                + LABEL_GAP
        };
        let plot_width = c.width() - y_axis_width;
        if plot_width <= 0.0 || plot_height <= 0.0 {
            return c.svg();
        }
        self.render_axis_titles(
            &titles,
            &self.y_axis_configs,
            &self.x_axis.title,
            y_axis_width,
            axis_top,
            plot_width,
            plot_height,
        );

        // The time the tasks span, with a little room at both ends unless
        // an end is fixed.
        let (mut min, mut max) = tasks
            .iter()
            .fold((f64::INFINITY, f64::NEG_INFINITY), |(lo, hi), (_, span)| {
                (lo.min(span.0), hi.max(span.1))
            });
        if let Some(now) = self.now.filter(|v| v.is_finite()) {
            min = min.min(now);
            max = max.max(now);
        }
        let padding = if max > min {
            (max - min) * RANGE_PADDING
        } else {
            // A single moment: an hour around it, or one unit.
            if self.is_time() { 1800.0 } else { 0.5 }
        };
        min = self
            .x_axis
            .min
            .filter(|v| v.is_finite())
            .unwrap_or(min - padding);
        max = self
            .x_axis
            .max
            .filter(|v| v.is_finite())
            .unwrap_or(max + padding);
        if max <= min {
            max = min + 1.0;
        }
        let scale = ContinuousX {
            min,
            max,
            width: plot_width,
            band_width: 0.0,
            time: self.is_time(),
            tick_step: None,
        };

        let mut plot = c.child(Box {
            left: y_axis_width,
            ..Default::default()
        });
        // The grid: a line at every tick of the axis and between the rows.
        let row_height = plot_height / rows.len() as f32;
        let grid_line = |plot: &mut canvas::Canvas, from: (f32, f32), to: (f32, f32)| {
            plot.line(Line {
                color: Some(self.grid.stroke_color),
                stroke_width: self.grid.stroke_width,
                stroke_dash_array: self.grid.stroke_dash_array.clone(),
                left: from.0,
                top: from.1,
                right: to.0,
                bottom: to.1,
            });
        };
        let (ticks, _) = self.x_ticks(&scale);
        for x in ticks {
            grid_line(&mut plot, (x, 0.0), (x, plot_height));
        }
        for row in 1..rows.len() {
            let y = row_height * row as f32;
            grid_line(&mut plot, (0.0, y), (plot_width, y));
        }
        if !self.x_axis.hidden {
            self.render_continuous_x_axis(
                plot.child(Box {
                    top: plot_height,
                    ..Default::default()
                }),
                &scale,
                plot_width,
                x_axis_height,
            );
        }
        if !self.y_axis_hidden {
            for (index, row) in rows.iter().enumerate() {
                c.text_unmeasured(Text {
                    text: row.to_string(),
                    font_family: Some(self.font_family.clone()),
                    font_size: Some(row_config.font.size),
                    font_color: Some(row_config.font.color),
                    font_weight: row_config.font.weight.clone(),
                    x: Some(y_axis_width - LABEL_GAP),
                    y: Some(row_height * (index as f32 + 0.5)),
                    text_anchor: Some("end".to_string()),
                    dominant_baseline: Some("central".to_string()),
                    ..Default::default()
                });
            }
        }

        // The bars.
        let bar_height = self
            .bar_height
            .filter(|v| v.is_finite() && *v > 0.0)
            .unwrap_or(row_height * BAR_HEIGHT_RATIO)
            .min(row_height);
        let radius = if self.radius.is_finite() {
            self.radius.clamp(0.0, bar_height / 2.0)
        } else {
            DEFAULT_RADIUS.min(bar_height / 2.0)
        };
        let placed: Vec<Placed> = tasks
            .iter()
            // A task outside a fixed range is not on the chart.
            .filter(|(_, span)| span.1 >= min && span.0 <= max)
            .map(|&(task, span)| {
                let category = task
                    .category
                    .as_deref()
                    .and_then(|name| categories.iter().position(|c| *c == name));
                // Tasks without a category share the color after the last.
                let index = category.unwrap_or(categories.len());
                Placed {
                    task,
                    row: rows.iter().position(|r| *r == task.row_name()).unwrap_or(0),
                    left: scale.px(span.0).clamp(0.0, plot_width),
                    right: scale.px(span.1).clamp(0.0, plot_width),
                    color: task
                        .color
                        .unwrap_or_else(|| get_color(&self.series.colors, index)),
                }
            })
            .collect();

        // No corner to round: no attribute for it.
        let corner = (radius > 0.0).then_some(radius);
        // Half the width of the diamond of a milestone.
        let diamond = bar_height * 0.45;
        // What a shape takes of its row: a bar its span (the shortest one
        // shows as well), a milestone its diamond.
        let extent = |item: &Placed| -> (f32, f32) {
            if item.task.span().is_some_and(|(start, end)| start == end) {
                (item.left - diamond, item.left + diamond)
            } else {
                (item.left, item.left + (item.right - item.left).max(1.0))
            }
        };
        let font_size = self.series.label.font.size;
        for item in placed.iter() {
            let task = item.task;
            let top = row_height * item.row as f32 + (row_height - bar_height) / 2.0;
            let middle = top + bar_height / 2.0;
            let is_milestone = task.span().is_some_and(|(start, end)| start == end);
            let progress = task
                .progress
                .filter(|v| v.is_finite())
                .map(|v| v.clamp(0.0, 1.0));

            let (start, end) = task.span().unwrap_or_default();
            let tooltip_text = self.tooltip.show.then(|| {
                let mut text = if is_milestone {
                    format!("{}: {}", task.name, self.moment_text(start))
                } else {
                    format!(
                        "{}: {} – {}",
                        task.name,
                        self.moment_text(start),
                        self.moment_text(end)
                    )
                };
                if let Some(progress) = progress {
                    text.push_str(&format!(" ({}%)", format_float(progress * 100.0)));
                }
                text
            });
            let mut classes: Vec<&str> = vec![];
            if self.animation.is_some() {
                classes.push("gantt-anim");
            }
            if tooltip_text.is_some() {
                classes.push("ct-trigger");
            }
            let class = (!classes.is_empty()).then(|| classes.join(" "));
            let style = self
                .animation
                .as_ref()
                .map(|a| format!("animation-delay:{}ms", item.row as u32 * a.delay));
            let mut dataset = vec![
                ("name".to_string(), task.name.clone()),
                ("row".to_string(), task.row_name().to_string()),
            ];
            if let Some(category) = &task.category {
                dataset.push(("category".to_string(), category.clone()));
            }
            dataset.push(("start".to_string(), self.moment_text(start)));
            if !is_milestone {
                dataset.push(("end".to_string(), self.moment_text(end)));
            }
            if let Some(progress) = progress {
                dataset.push(("progress".to_string(), format_number(progress as f64, 2)));
            }

            // Where a label beside the shape may start, and end.
            let (shape_left, shape_right) = extent(item);
            if is_milestone {
                let half = diamond;
                plot.polygon(Polygon {
                    fill: Some(item.color),
                    points: vec![
                        (item.left, middle - half).into(),
                        (item.left + half, middle).into(),
                        (item.left, middle + half).into(),
                        (item.left - half, middle).into(),
                    ],
                    class,
                    style,
                    title: tooltip_text.clone(),
                    dataset,
                    ..Default::default()
                });
            } else {
                let width = shape_right - shape_left;
                // What is done in the full color; over it the whole bar,
                // lighter, which leaves the done part as it is.
                let fill = match progress {
                    Some(progress) => {
                        if progress > 0.0 {
                            plot.rect(Rect {
                                fill: Some(item.color.into()),
                                left: item.left,
                                top,
                                width: width * progress,
                                height: bar_height,
                                rx: corner,
                                ry: corner,
                                class: self.animation.as_ref().map(|_| "gantt-anim".to_string()),
                                style: style.clone(),
                                ..Default::default()
                            });
                        }
                        item.color.with_alpha(REMAINING_ALPHA)
                    }
                    None => item.color,
                };
                plot.rect(Rect {
                    fill: Some(fill.into()),
                    left: item.left,
                    top,
                    width,
                    height: bar_height,
                    rx: corner,
                    ry: corner,
                    class,
                    style,
                    title: tooltip_text.clone(),
                    dataset,
                    ..Default::default()
                });
            }
            if let Some(text) = tooltip_text {
                plot.text_unmeasured(Text {
                    text,
                    class: Some("ct-tip".to_string()),
                    font_weight: self.tooltip.font.weight.clone(),
                    font_family: Some(self.font_family.clone()),
                    font_color: Some(self.tooltip_font_color(self.series.label.font.color)),
                    font_size: Some(self.tooltip_font_size(font_size)),
                    x: Some(((shape_left + shape_right) / 2.0).clamp(0.0, plot_width)),
                    y: Some(top),
                    dy: Some(-4.0),
                    text_anchor: Some("middle".to_string()),
                    ..Default::default()
                });
            }

            // The name of the task, unless its row tells it already: on the
            // bar when it fits, else beside it where nothing is in the way.
            if !self.label_show || task.name.is_empty() || task.name == task.row_name() {
                continue;
            }
            let width = measure(&task.name, font_size);
            let fits_inside = !is_milestone && width + LABEL_GAP <= shape_right - shape_left;
            let free = |from: f32, to: f32| -> bool {
                from >= 0.0
                    && to <= plot_width
                    && !placed.iter().any(|other| {
                        let (left, right) = extent(other);
                        !std::ptr::eq(other, item)
                            && other.row == item.row
                            && left - 4.0 < to
                            && from < right + 4.0
                    })
            };
            let (x, anchor, color) = if fits_inside {
                // Dark on a light bar, light on a dark one.
                let color = if item.color.is_light() {
                    Color::black().with_alpha(200)
                } else {
                    Color::white()
                };
                ((shape_left + shape_right) / 2.0, "middle", color)
            } else if free(shape_right + 4.0, shape_right + 4.0 + width) {
                (shape_right + 4.0, "start", self.series.label.font.color)
            } else if free(shape_left - 4.0 - width, shape_left - 4.0) {
                (shape_left - 4.0, "end", self.series.label.font.color)
            } else {
                continue;
            };
            plot.text_unmeasured(Text {
                text: task.name.clone(),
                font_family: Some(self.font_family.clone()),
                font_color: Some(color),
                font_size: Some(font_size),
                font_weight: self.series.label.font.weight.clone(),
                x: Some(x),
                y: Some(middle),
                text_anchor: Some(anchor.to_string()),
                dominant_baseline: Some("central".to_string()),
                class: self.animation.as_ref().map(|_| "gantt-fade".to_string()),
                ..Default::default()
            });
        }

        // The marked moment, over the bars.
        if let Some(now) = self.now.filter(|v| v.is_finite() && *v >= min && *v <= max) {
            let x = scale.px(now);
            plot.line(Line {
                color: Some(self.title.font.color),
                stroke_width: 1.0,
                left: x,
                top: 0.0,
                right: x,
                bottom: plot_height,
                stroke_dash_array: Some("4 3".to_string()),
            });
        }

        let mut css = String::new();
        if let Some(ref anim) = self.animation {
            css.push_str(&format!(
                "@keyframes gantt-grow{{from{{transform:scaleX(0)}}to{{transform:scaleX(1)}}}} \
                 @keyframes gantt-fade{{from{{opacity:0}}to{{opacity:1}}}} \
                 .gantt-anim{{transform-box:fill-box;transform-origin:left center;\
                 animation:gantt-grow {}ms {} both}} \
                 .gantt-fade{{animation:gantt-fade {}ms {} both}} ",
                anim.duration,
                anim.safe_easing(),
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
    use super::{GanttChart, GanttTask};
    use pretty_assertions::assert_eq;

    const DAY: f64 = 86400.0;
    /// 2024-03-04 00:00 UTC, a Monday.
    const T0: f64 = 1_709_510_400.0;

    #[test]
    fn task_spans() {
        let task: GanttTask = ("a", 5.0, 9.0).into();
        assert_eq!(Some((5.0, 9.0)), task.span());
        assert_eq!("a", task.row_name());
        // Given the wrong way round, a task is still its span.
        assert_eq!(Some((5.0, 9.0)), GanttTask::from(("a", 9.0, 5.0)).span());
        // Without an end it is a moment; without a start it is nowhere.
        assert_eq!(
            Some((5.0, 5.0)),
            GanttTask::from(("a", 5.0, f64::NAN)).span()
        );
        assert_eq!(None, GanttTask::from(("a", f64::NAN, 5.0)).span());
        let mut task = task;
        task.row = Some("room".to_string());
        assert_eq!("room", task.row_name());
    }

    #[test]
    fn gantt_chart_basic() {
        let task = |name: &str, from: f64, to: f64, progress: f32| GanttTask {
            name: name.to_string(),
            start: T0 + from * DAY,
            end: T0 + to * DAY,
            progress: Some(progress),
            ..Default::default()
        };
        let mut chart = GanttChart::new(vec![
            task("Research", 0.0, 4.0, 1.0),
            task("Design", 3.0, 9.0, 0.8),
            task("Build", 8.0, 18.0, 0.3),
            task("Test", 15.0, 21.0, 0.0),
            ("Launch", T0 + 22.0 * DAY, T0 + 22.0 * DAY).into(),
        ]);
        chart.title.text = "Project plan".to_string();
        chart.now = Some(T0 + 11.0 * DAY);
        assert_snapshot!("gantt_chart/basic.svg", chart.svg().unwrap());
    }
}
