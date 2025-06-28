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

use super::canvas;
use super::color::*;
use super::common::*;
use super::component::*;
use super::params::*;
use super::theme::{get_default_theme_name, get_theme, Theme, DEFAULT_Y_AXIS_WIDTH};
use super::util::*;
use super::Canvas;
use crate::charts::measure_text_width_family;
use charts_rs_derive::Chart;
use core::f32;
use std::sync::Arc;

#[derive(Clone, Debug, Default, Chart)]
pub struct PieChart {
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

    pub radius: f32,
    pub inner_radius: f32,
    pub rose_type: Option<bool>,
    pub border_radius: Option<f32>,

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
}

impl PieChart {
    fn fill_default(&mut self) {
        self.radius = 150.0;
        self.inner_radius = 40.0;
        self.legend_show = Some(false);
        self.rose_type = Some(true);
    }
    /// Creates a pie chart from json.
    pub fn from_json(data: &str) -> canvas::Result<PieChart> {
        let mut p = PieChart {
            ..Default::default()
        };
        p.fill_default();
        let value = p.fill_option(data)?;
        if let Some(radius) = get_f32_from_value(&value, "radius") {
            p.radius = radius;
        }
        if let Some(inner_radius) = get_f32_from_value(&value, "inner_radius") {
            p.inner_radius = inner_radius;
        }
        if let Some(rose_type) = get_bool_from_value(&value, "rose_type") {
            p.rose_type = Some(rose_type);
        }
        if let Some(border_radius) = get_f32_from_value(&value, "border_radius") {
            p.border_radius = Some(border_radius);
        }
        Ok(p)
    }
    /// Creates a pie chart with custom theme.
    pub fn new_with_theme(series_list: Vec<Series>, theme: &str) -> PieChart {
        let mut p = PieChart {
            series_list,
            ..Default::default()
        };
        p.fill_default();
        p.fill_theme(get_theme(theme));
        p
    }
    /// Creates a pie chart with default theme.
    pub fn new(series_list: Vec<Series>) -> PieChart {
        PieChart::new_with_theme(series_list, &get_default_theme_name())
    }
    /// Converts pie chart to svg.
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
        if axis_top > 0.0 {
            c = c.child(Box {
                top: axis_top,
                ..Default::default()
            });
        }

        let values: Vec<f32> = self
            .series_list
            .iter()
            .map(|item| item.data.iter().sum())
            .collect();
        let mut max = 0.0;
        let mut sum = 0.0;
        for item in values.iter() {
            sum += *item;
            if *item > max {
                max = *item;
            }
        }
        let mut delta = 360.0 / values.len() as f32;
        let mut half_delta = delta / 2.0;
        let mut start_angle = 0.0_f32;
        let mut radius_double = c.height();

        if c.width() < radius_double {
            radius_double = c.width();
        }
        radius_double *= 0.8;
        let mut r = radius_double / 2.0;
        if r > self.radius {
            r = self.radius;
        }

        let cx = (c.width() - radius_double) / 2.0 + r;
        let cy = (c.height() - radius_double) / 2.0 + r;
        let label_offset = 20.0;
        let mut series_label_formatter = self.series_label_formatter.clone();
        if series_label_formatter.is_empty() {
            series_label_formatter = "{a}: {d}".to_string();
        }
        let rose_type = self.rose_type.unwrap_or_default();

        let mut prev_quadrant = u8::MAX;
        let mut prev_end_y = f32::MAX;
        for (index, series) in self.series_list.iter().enumerate() {
            let value = values[index];
            let mut cr = value / max * (r - self.inner_radius) + self.inner_radius;
            let color = get_color(&self.series_colors, series.index.unwrap_or(index));
            // normal pie
            if !rose_type {
                cr = r;
                delta = value / sum * 360.0;
                half_delta = delta / 2.0;
            }
            if cr - self.inner_radius < 1.0 {
                cr = self.inner_radius + 1.0;
            }
            let mut pie = Pie {
                fill: color,
                cx,
                cy,
                r: cr,
                ir: self.inner_radius,
                start_angle,
                delta,
                ..Default::default()
            };
            if let Some(border_radius) = self.border_radius {
                pie.border_radius = border_radius;
            }

            c.pie(pie);

            let angle = start_angle + half_delta;
            let mut points = vec![];
            points.push(get_pie_point(cx, cy, cr, angle));
            let mut end = get_pie_point(cx, cy, r + label_offset, angle);

            let quadrant = get_quadrant(cx, cy, &end);
            // quadrant change
            if quadrant != prev_quadrant {
                prev_end_y = f32::MAX;
                prev_quadrant = quadrant;
            }
            // label overlap
            if (end.y - prev_end_y).abs() < self.series_label_font_size {
                if quadrant == 1 || quadrant == 4 {
                    end.y = prev_end_y + self.series_label_font_size;
                } else {
                    end.y = prev_end_y - self.series_label_font_size;
                }
            }
            prev_end_y = end.y;

            points.push(end);

            let is_left = angle > 180.0;
            if is_left {
                end.x -= label_offset;
            } else {
                end.x += label_offset;
            }
            let mut label_margin = Box {
                left: end.x,
                top: end.y + 5.0,
                ..Default::default()
            };
            let label_option = LabelOption {
                series_name: series.name.clone(),
                value,
                percentage: value / sum,
                formatter: series_label_formatter.clone(),
                ..Default::default()
            };
            let label_text = label_option.format();

            if is_left {
                if let Ok(b) = measure_text_width_family(
                    &self.font_family,
                    self.series_label_font_size,
                    &label_text,
                ) {
                    label_margin.left -= b.width();
                }
            } else {
                label_margin.left += 3.0;
            }

            points.push(end);
            c.smooth_line(SmoothLine {
                color: Some(color),
                points,
                symbol: None,
                ..Default::default()
            });

            c.child(label_margin).text(Text {
                text: label_text,
                font_family: Some(self.font_family.clone()),
                font_size: Some(self.series_label_font_size),
                font_color: Some(self.series_label_font_color),
                ..Default::default()
            });

            start_angle += delta;
        }

        c.svg()
    }
}

#[cfg(test)]
mod tests {
    use super::PieChart;
    use pretty_assertions::assert_eq;

    #[test]
    fn pie_basic() {
        let mut pie_chart = PieChart::new(vec![
            ("rose 1", vec![40.0]).into(),
            ("rose 2", vec![38.0]).into(),
            ("rose 3", vec![32.0]).into(),
            ("rose 4", vec![30.0]).into(),
            ("rose 5", vec![28.0]).into(),
            ("rose 6", vec![26.0]).into(),
            ("rose 7", vec![22.0]).into(),
            ("rose 8", vec![18.0]).into(),
        ]);
        pie_chart.title_text = "Nightingale Chart".to_string();
        pie_chart.sub_title_text = "Fake Data".to_string();
        assert_eq!(
            include_str!("../../asset/pie_chart/basic.svg"),
            pie_chart.svg().unwrap()
        );
    }

    #[test]
    fn small_pie_basic() {
        let mut pie_chart = PieChart::new(vec![
            ("rose 1", vec![400.0]).into(),
            ("rose 2", vec![38.0]).into(),
            ("rose 3", vec![32.0]).into(),
            ("rose 4", vec![30.0]).into(),
            ("rose 5", vec![28.0]).into(),
            ("rose 6", vec![26.0]).into(),
            ("rose 7", vec![22.0]).into(),
            ("rose 8", vec![18.0]).into(),
        ]);
        pie_chart.width = 400.0;
        pie_chart.height = 300.0;
        pie_chart.title_text = "Nightingale Chart".to_string();
        pie_chart.sub_title_text = "Fake Data".to_string();
        assert_eq!(
            include_str!("../../asset/pie_chart/small_basic.svg"),
            pie_chart.svg().unwrap()
        );
    }

    #[test]
    fn not_rose_pie() {
        let mut pie_chart = PieChart::new(vec![
            ("rose 1", vec![400.0]).into(),
            ("rose 2", vec![38.0]).into(),
            ("rose 3", vec![32.0]).into(),
            ("rose 4", vec![30.0]).into(),
            ("rose 5", vec![28.0]).into(),
            ("rose 6", vec![26.0]).into(),
            ("rose 7", vec![22.0]).into(),
            ("rose 8", vec![18.0]).into(),
        ]);
        pie_chart.rose_type = Some(false);
        pie_chart.title_text = "Pie Chart".to_string();
        pie_chart.sub_title_text = "Fake Data".to_string();
        assert_eq!(
            include_str!("../../asset/pie_chart/not_rose.svg").trim(),
            pie_chart.svg().unwrap()
        );
    }

    #[test]
    fn not_rose_radius_pie() {
        let mut pie_chart = PieChart::new(vec![
            ("rose 1", vec![400.0]).into(),
            ("rose 2", vec![38.0]).into(),
            ("rose 3", vec![32.0]).into(),
            ("rose 4", vec![30.0]).into(),
            ("rose 5", vec![28.0]).into(),
            ("rose 6", vec![26.0]).into(),
            ("rose 7", vec![22.0]).into(),
            ("rose 8", vec![18.0]).into(),
        ]);
        pie_chart.rose_type = Some(false);
        pie_chart.inner_radius = 0.0;
        pie_chart.border_radius = Some(0.0);
        pie_chart.title_text = "Pie Chart".to_string();
        pie_chart.sub_title_text = "Fake Data".to_string();
        assert_eq!(
            include_str!("../../asset/pie_chart/not_rose_radius.svg").trim(),
            pie_chart.svg().unwrap()
        );
    }

    #[test]
    fn pie_rose_small_piece() {
        let mut pie_chart = PieChart::new(vec![
            ("rose 1", vec![40000.0]).into(),
            ("rose 2", vec![38.0]).into(),
            ("rose 3", vec![32.0]).into(),
            ("rose 4", vec![30.0]).into(),
            ("rose 5", vec![28.0]).into(),
            ("rose 6", vec![26.0]).into(),
            ("rose 7", vec![22.0]).into(),
            ("rose 8", vec![18.0]).into(),
        ]);
        pie_chart.title_text = "Nightingale Chart".to_string();
        pie_chart.sub_title_text = "Fake Data".to_string();
        assert_eq!(
            include_str!("../../asset/pie_chart/rose_small_piece.svg"),
            pie_chart.svg().unwrap()
        );
    }
}
