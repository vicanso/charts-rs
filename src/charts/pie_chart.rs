use super::canvas;
use super::color::*;
use super::common::*;
use super::component::*;
use super::theme::{get_default_theme, get_theme, Theme, DEFAULT_Y_AXIS_WIDTH};
use super::util::*;
use super::Canvas;
use super::Chart;
use crate::charts::measure_text_width_family;
use charts_rs_derive::Chart;

#[derive(Clone, Debug, Default, Chart)]
pub struct PieChart {
    pub width: f32,
    pub height: f32,
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

    // sub title
    pub sub_title_text: String,
    pub sub_title_font_size: f32,
    pub sub_title_font_color: Color,
    pub sub_title_margin: Option<Box>,
    pub sub_title_align: Align,

    // legend
    pub legend_font_size: f32,
    pub legend_font_color: Color,
    pub legend_align: Align,
    pub legend_margin: Option<Box>,
    pub legend_category: LegendCategory,

    pub radius: f32,
    pub inner_radius: f32,

    // x axis
    pub x_axis_data: Vec<String>,
    pub x_axis_height: f32,
    pub x_axis_stroke_color: Color,
    pub x_axis_font_size: f32,
    pub x_axis_font_color: Color,
    pub x_axis_name_gap: f32,
    pub x_axis_name_rotate: f32,
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
    pub series_colors: Vec<Color>,
    pub series_symbol: Option<Symbol>,
    pub series_smooth: bool,
    pub series_fill: bool,
}

impl PieChart {
    pub fn new_with_theme(series_list: Vec<Series>, theme: String) -> PieChart {
        let mut p = PieChart {
            series_list,
            ..Default::default()
        };
        let theme = get_theme(theme);
        p.radius = 150.0;
        p.inner_radius = 40.0;
        p.fill_theme(theme);
        p
    }
    pub fn new(series_list: Vec<Series>) -> PieChart {
        PieChart::new_with_theme(series_list, get_default_theme())
    }

    pub fn svg(&self) -> canvas::Result<String> {
        let mut c = Canvas::new(self.width, self.height);

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

        let values: Vec<f32> = self
            .series_list
            .iter()
            .map(|item| item.data.iter().sum())
            .collect();
        let mut max = 0.0;
        for item in values.iter() {
            if *item > max {
                max = *item;
            }
        }
        let delta = 360.0 / values.len() as f32;
        let half_delta = delta / 2.0;
        let mut start_angle = 0.0_f32;
        let mut radius_double = c.height();

        if c.width() < radius_double {
            radius_double = c.width();
        }
        radius_double *= 0.8;
        let r = radius_double / 2.0;

        let cx = (c.width() - radius_double) / 2.0 + r;
        let cy = (c.height() - radius_double) / 2.0 + r;
        let label_offset = 20.0;

        for (index, series) in self.series_list.iter().enumerate() {
            let value = values[index] / max * r;
            let color = *self
                .series_colors
                .get(series.index.unwrap_or(index))
                .unwrap_or_else(|| &self.series_colors[0]);

            c.pie(Pie {
                fill: color,
                cx,
                cy,
                r: value,
                ir: self.inner_radius,
                start_angle,
                delta,
                ..Default::default()
            });
            let angle = start_angle + half_delta;
            let mut points = vec![];
            points.push(get_pie_point(cx, cy, value, angle));
            let mut end = get_pie_point(cx, cy, r + label_offset, angle);
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
            if is_left {
                if let Ok(b) = measure_text_width_family(
                    &self.font_family,
                    self.series_label_font_size,
                    &series.name,
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
                text: series.name.clone(),
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
    use crate::Series;

    use super::PieChart;

    #[test]
    fn pie_basic() {
        let pie_chart = PieChart::new(vec![
            Series::new("rose 1".to_string(), vec![40.0]),
            Series::new("rose 2".to_string(), vec![38.0]),
            Series::new("rose 3".to_string(), vec![32.0]),
            Series::new("rose 4".to_string(), vec![30.0]),
            Series::new("rose 5".to_string(), vec![28.0]),
            Series::new("rose 6".to_string(), vec![26.0]),
            Series::new("rose 7".to_string(), vec![22.0]),
            Series::new("rose 8".to_string(), vec![18.0]),
        ]);
        assert_eq!(
            include_str!("../../asset/pie_chart/basic.svg"),
            pie_chart.svg().unwrap()
        );
    }
}
