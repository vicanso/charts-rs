extern crate proc_macro;
use proc_macro::TokenStream;
use quote::quote;
use syn::DeriveInput;

#[proc_macro_derive(Chart)]
pub fn my_default(input: TokenStream) -> TokenStream {
    let ast: DeriveInput = syn::parse(input).unwrap();
    let id = ast.ident;

    let gen = quote! {
        impl Chart for #id {
            fn fill_theme(&mut self, t: Theme) {
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
                self.title_height = t.title_height;

                self.sub_title_font_color = t.sub_title_font_color;
                self.sub_title_font_size = t.sub_title_font_size;
                self.sub_title_margin = t.sub_title_margin;
                self.sub_title_align = t.sub_title_align;
                self.sub_title_height = t.sub_title_height;

                self.legend_font_color = t.legend_font_color;
                self.legend_font_size = t.legend_font_size;
                self.legend_align = t.legend_align;
                self.legend_margin = t.legend_margin;

                self.x_axis_font_size = t.x_axis_font_size;
                self.x_axis_font_color = t.x_axis_font_color;
                self.x_axis_stroke_color = t.x_axis_stroke_color;
                self.x_axis_name_gap = t.x_axis_name_gap;
                self.x_axis_height = t.x_axis_height;

                self.y_axis_configs = vec![
                    YAxisConfig{
                        axis_font_size: t.y_axis_font_size,
                        axis_font_color: t.y_axis_font_color,
                        axis_stroke_color: t.y_axis_stroke_color,
                        axis_split_number: t.y_axis_split_number,
                        axis_name_gap: t.y_axis_name_gap,
                        ..Default::default()
                    }
                ];

                self.grid_stroke_color = t.grid_stroke_color;
                self.grid_stroke_width = t.grid_stroke_width;

                self.series_colors = t.series_colors;
                self.series_label_font_color = t.series_label_font_color;
                self.series_label_font_size = t.series_label_font_size;
                self.series_stroke_width = t.series_stroke_width;

                self.series_symbol = Some(Symbol::Circle(
                    self.series_stroke_width,
                    Some(self.background_color),
                ));
            }
            fn get_y_axis_config(&self, index: usize) -> YAxisConfig {
                let size = self.y_axis_configs.len();
                if size == 0 {
                    YAxisConfig::default()
                } else if index < size {
                    self.y_axis_configs[index].clone()
                } else {
                    self.y_axis_configs[0].clone()
                }
            }
            fn get_y_axis_values(&self, y_axis_index: usize) -> (AxisValues, f32) {
                let y_axis_config = self.get_y_axis_config(y_axis_index);
                let mut data_list = vec![];
                for series in self.series_list.iter() {
                    if series.y_axis_index == y_axis_index {
                        data_list.append(series.data.clone().as_mut());
                    }
                }
                if data_list.is_empty() {
                   return (AxisValues::default(), 0.0);
                }
                let y_axis_values = get_axis_values(AxisValueParams {
                    data_list,
                    split_number: y_axis_config.axis_split_number,
                    reverse: Some(true),
                    ..Default::default()
                });
                let y_axis_width = if let Some(value) = y_axis_config.axis_width {
                    value
                } else {
                    let y_axis_formatter = &y_axis_config.axis_formatter.clone().unwrap_or_default();
                    let str = format_string(&y_axis_values.data[0], y_axis_formatter);
                    if let Ok(b) = measure_text_width_family(&self.font_family, y_axis_config.axis_font_size, &str)
                    {
                        b.width() + 5.0
                    } else {
                        DEFAULT_Y_AXIS_WIDTH
                    }
                };
                (y_axis_values, y_axis_width)
            }
            fn render_background(&self, c: Canvas) {
                if self.background_color.is_transparent() {
                    return;
                }
                let mut c1 = c;
                c1.rect(Rect {
                    fill: Some(self.background_color),
                    left: 0.0,
                    top: 0.0,
                    width: self.width,
                    height: self.height,
                    ..Default::default()
                });
            }
            fn render_title(&self, c: Canvas) -> f32 {
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
                            _ => 0.0,
                        }
                    }
                    let title_margin_bottom = title_margin.bottom;
                    let b = c.child(title_margin).text(Text {
                        text: self.title_text.clone(),
                        font_family: Some(self.font_family.clone()),
                        font_size: Some(self.title_font_size),
                        font_weight: self.title_font_weight.clone(),
                        font_color: Some(self.title_font_color),
                        line_height: Some(self.title_height),
                        x: Some(x),
                        ..Default::default()
                    });
                    title_height = b.outer_height() + title_margin_bottom;
                }
                if !self.sub_title_text.is_empty() {
                    let mut sub_title_margin = self.sub_title_margin.clone().unwrap_or_default();
                    let mut x = 0.0;
                    if let Ok(sub_title_box) = measure_text_width_family(
                        &self.font_family,
                        self.sub_title_font_size,
                        &self.sub_title_text,
                    ) {
                        x = match self.title_align {
                            Align::Center => (c.width() - sub_title_box.width()) / 2.0,
                            Align::Right => c.width() - sub_title_box.width(),
                            _ => 0.0,
                        }
                    }
                    let sub_title_margin_bottom = sub_title_margin.bottom;
                    sub_title_margin.top += self.title_height;
                    let b = c.child(sub_title_margin).text(Text {
                        text: self.sub_title_text.clone(),
                        font_family: Some(self.font_family.clone()),
                        font_size: Some(self.sub_title_font_size),
                        font_color: Some(self.sub_title_font_color),
                        line_height: Some(self.sub_title_height),
                        x: Some(x),
                        ..Default::default()
                    });
                    title_height = b.outer_height() + sub_title_margin_bottom;
                }
                title_height
            }
            fn render_legend(&self, c: Canvas) -> f32 {
                if !self.legend_show.unwrap_or(true) {
                    return 0.0
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
                        .get(series.index.unwrap_or(index))
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
                        category: self.legend_category.clone(),
                    });
                    legend_left += b.width() + LEGEND_MARGIN;
                }
        
                legend_unit_height + legend_top + legend_margin_value
            }
            fn render_grid(&self, c: Canvas, axis_width: f32, axis_height: f32) {
                let mut c1 = c;
                let y_axis_config = self.get_y_axis_config(0);
                let axis_split_number = y_axis_config.axis_split_number;
                c1.grid(Grid {
                    right: axis_width,
                    bottom: axis_height,
                    color: Some(self.grid_stroke_color),
                    stroke_width: self.grid_stroke_width,
                    horizontals: axis_split_number,
                    hidden_horizontals: vec![axis_split_number],
                    ..Default::default()
                });
            }
            fn render_y_axis(&self, c: Canvas, data: Vec<String>, axis_height: f32, axis_width: f32, axis_index: usize) {
                let mut c1 = c; 
                let y_axis_config = &self.get_y_axis_config(axis_index);
                let mut position = Position::Left;
                if axis_index > 0 {
                    position = Position::Right;
                }
                c1.axis(Axis {
                    position,
                    height: axis_height,
                    width: axis_width,
                    split_number: y_axis_config.axis_split_number,
                    font_family: self.font_family.clone(),
                    stroke_color: Some(y_axis_config.axis_stroke_color),
                    name_align: Align::Left,
                    name_gap: y_axis_config.axis_name_gap,
                    font_color: Some(y_axis_config.axis_font_color),
                    font_size: y_axis_config.axis_font_size,
                    data,
                    formatter: y_axis_config.axis_formatter.clone(),
                    ..Default::default()
                });
            }
            fn render_x_axis(&self, c: Canvas, data: Vec<String>, axis_width: f32) {
                let mut c1 = c; 
                
                let mut split_number = data.len();
                let name_align = if self.x_boundary_gap.unwrap_or(true) {
                    Align::Center
                } else {
                    split_number -= 1;
                    Align::Left
                };

                c1.axis(Axis {
                    height: self.x_axis_height,
                    width: axis_width,
                    split_number,
                    font_family: self.font_family.clone(),
                    data,
                    font_color: Some(self.x_axis_font_color),
                    stroke_color: Some(self.x_axis_stroke_color),
                    font_size: self.x_axis_font_size,
                    name_gap: self.x_axis_name_gap,
                    name_rotate: self.x_axis_name_rotate,
                    name_align,
                    ..Default::default()
                });
            }
            fn render_series_label(&self, c:Canvas, series_labels_list: Vec<Vec<SeriesLabel>>) {
                if series_labels_list.is_empty() {
                    return;
                }
                let mut c1 = c;
                for series_labels in series_labels_list.iter() {
                    for series_label in series_labels.iter() {
                        let mut dx = None;
                        if let Ok(value) = measure_text_width_family(
                            &self.font_family,
                            self.series_label_font_size,
                            &series_label.text,
                        ) {
                            dx = Some(-value.width() / 2.0);
                        }
                        c1.text(Text {
                            text: series_label.text.clone(),
                            dy: Some(-8.0),
                            dx,
                            font_family: Some(self.font_family.clone()),
                            font_color: Some(self.series_label_font_color),
                            font_size: Some(self.series_label_font_size),
                            x: Some(series_label.point.x),
                            y: Some(series_label.point.y),
                            ..Default::default()
                        });
                    }
                }
            }
            fn render_bar(
                &self,
                c: Canvas,
                series_list: &[&Series],
                y_axis_values_list: &[&AxisValues],
                max_height: f32,
            ) -> Vec<Vec<SeriesLabel>> {
                if series_list.is_empty() {
                    return vec![];
                }
                let mut c1 = c;
                let unit_width = c1.width() / series_list[0].data.len() as f32;
                let bar_chart_margin = 5.0_f32;
                let bar_chart_gap = 3.0_f32;
        
                let bar_chart_margin_width = bar_chart_margin * 2.0;
                let bar_chart_gap_width = bar_chart_gap * (series_list.len() - 1) as f32;
                let bar_width = (unit_width - bar_chart_margin_width - bar_chart_gap_width) / series_list.len() as f32;
                let half_bar_width = bar_width / 2.0;
        
                let mut series_labels_list = vec![];
                for (index, series) in series_list.iter().enumerate() {
                    let y_axis_values = if index >= y_axis_values_list.len() {
                        y_axis_values_list[0]
                    } else {
                        y_axis_values_list[series.y_axis_index]
                    };
                    let color = *self
                        .series_colors
                        .get(series.index.unwrap_or(index))
                        .unwrap_or_else(|| &self.series_colors[0]);
                    let mut series_labels = vec![];
                    for (i, p) in series.data.iter().enumerate() {
                        let mut left = unit_width * i as f32 + bar_chart_margin;
                        left += (bar_width + bar_chart_gap) * index as f32;
        
                        let y = y_axis_values.get_offset_height(p.to_owned(), max_height);
                        c1.rect(Rect {
                            fill: Some(color),
                            left,
                            top: y,
                            width: bar_width,
                            height: max_height - y,
                            ..Default::default()
                        });
                        series_labels.push(SeriesLabel{
                            point: (left + half_bar_width, y).into(),
                            text: format_float(p.to_owned()),
                        })
                    }
                    if series.label_show {
                        series_labels_list.push(series_labels);
                    }
                }
                series_labels_list
            }
            fn render_line(
                &self,
                c: Canvas,
                series_list: &[&Series],
                y_axis_values_list: &[&AxisValues],
                max_height: f32,
                axis_height: f32,
            ) -> Vec<Vec<SeriesLabel>> {
                if series_list.is_empty() {
                    return vec![];
                }
                let mut c1 = c;
                let x_boundary_gap = self.x_boundary_gap.unwrap_or(true);
                let mut split_unit_offset = 0.0;
                if !x_boundary_gap {
                    split_unit_offset = 1.0;
                }
                let mut series_labels_list = vec![];
                for (index, series) in series_list.iter().enumerate() {
                    let y_axis_values = if series.y_axis_index >= y_axis_values_list.len() {
                        y_axis_values_list[0]
                    } else {
                        y_axis_values_list[series.y_axis_index]
                    };
                    let split_unit_count = series.data.len() as f32 - split_unit_offset;
                    let unit_width = c1.width() / split_unit_count;
                    let mut points: Vec<Point> = vec![];
                    let mut series_labels = vec![];
                    for (i, p) in series.data.iter().enumerate() {
                        // 居中
                        let mut x = unit_width * i as f32;
                        if x_boundary_gap {
                            x += unit_width / 2.0;
                        }
                        let y = y_axis_values.get_offset_height(p.to_owned(), max_height);
                        points.push((x, y).into());
                        series_labels.push(SeriesLabel{
                            point: (x, y).into(),
                            text: format_float(p.to_owned()),
                        })
                    }
                    if series.label_show {
                        series_labels_list.push(series_labels);
                    }
        
                    let color = *self
                        .series_colors
                        .get(series.index.unwrap_or(index))
                        .unwrap_or_else(|| &self.series_colors[0]);
        
                    let fill = color.with_alpha(100);
                    let series_fill = self.series_fill;
                    if self.series_smooth {
                        if series_fill {
                            c1.smooth_line_fill(SmoothLineFill {
                                fill,
                                points: points.clone(),
                                bottom: axis_height,
                            });
                        }
                        c1.smooth_line(SmoothLine {
                            points,
                            color: Some(color),
                            stroke_width: self.series_stroke_width,
                            symbol: self.series_symbol.clone(),
                        });
                    } else {
                        if series_fill {
                            c1.straight_line_fill(StraightLineFill {
                                fill,
                                points: points.clone(),
                                bottom: axis_height,
                                ..Default::default()
                            });
                        }
                        c1.straight_line(StraightLine {
                            points,
                            color: Some(color),
                            stroke_width: self.series_stroke_width,
                            symbol: self.series_symbol.clone(),
                            ..Default::default()
                        });
                    }
                }

                series_labels_list
            }
        }
    };
    gen.into()
}
