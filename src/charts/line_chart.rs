use crate::charts::measure_text_width_family;

use super::color::*;
use super::common::*;
use super::component::*;
use super::theme::get_default_theme;
use super::theme::get_theme;
use super::util::*;
use super::Canvas;

#[derive(Clone, Debug, Default)]
pub struct LineChart {
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

    // x axis
    pub x_axis_data: Vec<String>,
    pub x_axis_height: f32,
    pub x_axis_stroke_color: Color,
    pub x_axis_font_size: f32,
    pub x_axis_font_color: Color,
    pub x_axis_name_gap: f32,
    pub x_axis_name_rotate: f32,
    // y axis
    pub y_axis_font_size: f32,
    pub y_axis_font_color: Color,
    pub y_axis_width: f32,
    pub y_axis_split_number: usize,
    pub y_axis_name_gap: f32,

    // grid
    pub grid_stroke_color: Color,
    pub grid_stroke_width: f32,

    // series
    pub series_stroke_width: f32,
    pub series_colors: Vec<Color>,
    pub series_symbol: Option<Symbol>,
    pub series_smooth: bool,
    pub series_fill: bool,
}

impl LineChart {
    pub fn new(series_list: Vec<Series>, x_axis_data: Vec<String>) -> LineChart {
        let mut l = LineChart {
            series_list,
            x_axis_data,
            ..Default::default()
        };
        l.fill_theme(get_default_theme());
        l
    }
    pub fn fill_theme(&mut self, theme: String) {
        let t = get_theme(theme);

        self.font_family = t.font_family;
        self.margin = t.margin;
        self.width = t.width;
        self.height = t.height;
        self.background_color = t.background_color;
        self.is_light = t.is_light;

        self.title_font_color = t.title_font_color;
        self.title_font_size = t.title_font_size;
        self.title_font_weight = t.title_font_weight;
        self.title_margin = t.title_margin;
        self.title_align = t.title_align;

        self.sub_title_font_color = t.sub_title_font_color;
        self.sub_title_font_size = t.sub_title_font_size;
        self.sub_title_margin = t.sub_title_margin;
        self.sub_title_align = t.sub_title_align;

        self.legend_font_color = t.legend_font_color;
        self.legend_font_size = t.legend_font_size;
        self.legend_align = t.legend_align;
        self.legend_margin = t.legend_margin;

        self.x_axis_font_size = t.x_axis_font_size;
        self.x_axis_font_color = t.x_axis_font_color;
        self.x_axis_stroke_color = t.x_axis_stroke_color;
        self.x_axis_name_gap = t.x_axis_name_gap;
        self.x_axis_height = t.x_axis_height;

        self.y_axis_font_color = t.y_axis_font_color;
        self.y_axis_font_size = t.y_axis_font_size;
        self.y_axis_width = t.y_axis_width;
        self.y_axis_split_number = t.y_axis_split_number;
        self.y_axis_name_gap = t.y_axis_name_gap;

        self.grid_stroke_color = t.grid_stroke_color;
        self.grid_stroke_width = t.grid_stroke_width;

        self.series_colors = t.series_colors;
        self.series_stroke_width = t.series_stroke_width;

        self.series_symbol = Some(Symbol::Circle(
            self.series_stroke_width,
            Some(self.background_color),
        ));
    }
    pub fn svg(&self) {
        let mut c = Canvas::new(self.width, self.height);
        c.margin = self.margin.clone();

        let mut title_height = 0.0;

        if !self.title_text.is_empty() {
            let title_margin = self.title_margin.clone().unwrap_or_default();
            let mut x = 0.0;
            if let Ok(title_box) =
                measure_text_width_family(&self.font_family, self.title_font_size, &self.title_text)
            {
                x = match self.title_align {
                    Align::Center => (c.width() - title_box.width()) / 2.0,
                    Align::Right => c.width() - title_box.width(),
                    Align::Left => 0.0,
                }
            }
            let b = c.child(title_margin).text(Text {
                text: self.title_text.clone(),
                font_family: Some(self.font_family.clone()),
                font_size: Some(self.title_font_size),
                font_weight: self.title_font_weight.clone(),
                font_color: Some(self.title_font_color),
                y: Some(self.title_font_size),
                x: Some(x),
                ..Default::default()
            });
            title_height = b.outer_height();
        }

        if !self.sub_title_text.is_empty() {
            let mut sub_title_margin = self.sub_title_margin.clone().unwrap_or_default();
            let mut x = 0.0;
            if let Ok(title_box) = measure_text_width_family(
                &self.font_family,
                self.sub_title_font_size,
                &self.sub_title_text,
            ) {
                x = match self.title_align {
                    Align::Center => (c.width() - title_box.width()) / 2.0,
                    Align::Right => c.width() - title_box.width(),
                    Align::Left => 0.0,
                }
            }
            sub_title_margin.top += title_height;
            let b = c.child(sub_title_margin).text(Text {
                text: self.sub_title_text.clone(),
                font_family: Some(self.font_family.clone()),
                font_size: Some(self.sub_title_font_size),
                font_color: Some(self.sub_title_font_color),
                y: Some(self.sub_title_font_size),
                x: Some(x),
                ..Default::default()
            });
            title_height = b.outer_height();
        }

        let mut legend_left = 0.0;
        let legends: Vec<&str> = self
            .series_list
            .iter()
            .map(|item| item.name.as_str())
            .collect();
        let legend_margin = self.legend_margin.clone().unwrap_or_default();
        let legend_margin_value = legend_margin.top + legend_margin.bottom;
        let mut legend_canvas = c.child(legend_margin);
        let (legend_width, legend_width_list) =
            measure_legends(&self.font_family, self.legend_font_size, &legends);
        let legend_canvas_width = legend_canvas.width();
        if legend_width < legend_canvas_width {
            legend_left = match self.legend_align {
                Align::Right => legend_canvas_width - legend_width,
                Align::Left => 0.0,
                Align::Center => (legend_canvas_width - legend_width) / 2.0,
            };
            if legend_left < 0.0 {
                legend_left = 0.0;
            }
        }
        let legend_unit_height = self.legend_font_size + LEGEND_MARGIN;
        let mut legend_top = 0.0;
        for (index, series) in self.series_list.iter().enumerate() {
            let color = *self
                .series_colors
                .get(index)
                .unwrap_or_else(|| &self.series_colors[0]);
            let fill = if self.is_light {
                Some(self.background_color)
            } else {
                Some(color)
            };
            if legend_left + legend_width_list[index] > legend_canvas_width {
                legend_left = 0.0;
                legend_top += legend_unit_height;
            }
            let b = legend_canvas.legend(Legend {
                text: series.name.to_string(),
                font_size: self.legend_font_size,
                font_family: self.font_family.clone(),
                font_color: Some(self.legend_font_color),
                stroke_color: Some(color),
                fill,
                left: legend_left,
                top: legend_top,
            });
            legend_left += b.width() + LEGEND_MARGIN;
        }

        let legend_outer_height = legend_unit_height + legend_top + legend_margin_value;
        let axis_top = if legend_outer_height > title_height {
            legend_outer_height
        } else {
            title_height
        };

        let axis_height = c.height() - self.x_axis_height - axis_top;
        let axis_width = c.width() - self.y_axis_width;
        // 顶部文本区域
        if axis_top > 0.0 {
            c = c.child(Box {
                top: axis_top,
                ..Default::default()
            });
        }

        c.grid(Grid {
            left: self.y_axis_width,
            right: self.y_axis_width + axis_width,
            bottom: axis_height,
            color: Some(self.grid_stroke_color),
            stroke_width: self.grid_stroke_width,
            horizontals: self.y_axis_split_number,
            hidden_horizontals: vec![self.y_axis_split_number],
            ..Default::default()
        });

        let mut data_list = vec![];
        for series in self.series_list.iter() {
            data_list.append(series.data.clone().as_mut());
        }
        let y_axis_values = get_axis_values(AxisValueParams {
            data_list,
            split_number: self.y_axis_split_number,
            reverse: Some(true),
            ..Default::default()
        });
        // y axis
        c.axis(Axis {
            position: Position::Left,
            height: axis_height,
            width: self.y_axis_width,
            split_number: self.y_axis_split_number,
            font_family: self.font_family.clone(),
            stroke_color: Some((0, 0, 0, 0).into()),
            name_align: Align::Left,
            name_gap: self.y_axis_name_gap,
            font_color: Some(self.y_axis_font_color),
            font_size: self.y_axis_font_size,
            data: y_axis_values.data.clone(),
            ..Default::default()
        });

        // x axis
        c.child(Box {
            top: c.height() - self.x_axis_height,
            left: self.y_axis_width,
            ..Default::default()
        })
        .axis(Axis {
            height: self.x_axis_height,
            width: axis_width,
            split_number: self.x_axis_data.len(),
            font_family: self.font_family.clone(),
            data: self.x_axis_data.clone(),
            font_color: Some(self.x_axis_font_color),
            stroke_color: Some(self.x_axis_stroke_color),
            font_size: self.x_axis_font_size,
            name_gap: self.x_axis_name_gap,
            name_rotate: self.x_axis_name_rotate,
            ..Default::default()
        });

        // line point
        let max_height = c.height() - self.x_axis_height;

        let mut series_canvas = c.child(Box {
            left: self.y_axis_width,
            ..Default::default()
        });
        for (index, series) in self.series_list.iter().enumerate() {
            let unit_width = series_canvas.width() / series.data.len() as f32;
            let mut points: Vec<Point> = vec![];
            for (i, p) in series.data.iter().enumerate() {
                // 居中
                let x = unit_width * i as f32 + unit_width / 2.0;
                let y = y_axis_values.get_offset_height(p.to_owned(), max_height);
                points.push((x, y).into());
            }

            let color = *self
                .series_colors
                .get(index)
                .unwrap_or_else(|| &self.series_colors[0]);

            let fill = color.with_alpha(100);
            let series_fill = self.series_fill;
            if self.series_smooth {
                if series_fill {
                    series_canvas.smooth_line_fill(SmoothLineFill {
                        fill,
                        points: points.clone(),
                        bottom: axis_height,
                    });
                }
                series_canvas.smooth_line(SmoothLine {
                    points,
                    color: Some(color),
                    stroke_width: self.series_stroke_width,
                    symbol: self.series_symbol.clone(),
                });
            } else {
                if series_fill {
                    series_canvas.straight_line_fill(StraightLineFill {
                        fill,
                        points: points.clone(),
                        bottom: axis_height,
                    });
                }
                series_canvas.straight_line(StraightLine {
                    points,
                    color: Some(color),
                    stroke_width: self.series_stroke_width,
                    symbol: self.series_symbol.clone(),
                });
            }
        }

        println!("{}", c.svg().unwrap())
    }
}
