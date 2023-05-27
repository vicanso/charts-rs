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
                    sub_title_margin.top += self.title_font_size;
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
                title_height
            }
            fn render_legend(&self, c: Canvas) -> f32 {
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
                        ..Default::default()
                    });
                    legend_left += b.width() + LEGEND_MARGIN;
                }
        
                legend_unit_height + legend_top + legend_margin_value
            }
            fn render_grid(&self, c: Canvas, axis_width: f32, axis_height: f32) {
                let mut c1 = c;
                c1.grid(Grid {
                    left: self.y_axis_width,
                    right: self.y_axis_width + axis_width,
                    bottom: axis_height,
                    color: Some(self.grid_stroke_color),
                    stroke_width: self.grid_stroke_width,
                    horizontals: self.y_axis_split_number,
                    hidden_horizontals: vec![self.y_axis_split_number],
                    ..Default::default()
                });
            }
            fn render_y_axis(&self, c: Canvas, data: Vec<String>, axis_height: f32) {
                let mut c1 = c; 
                c1.axis(Axis {
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
                    data,
                    ..Default::default()
                });
            }
            fn render_x_axis(&self, c: Canvas, data: Vec<String>, axis_width: f32) {
                let mut c1 = c; 
                c1.axis(Axis {
                    height: self.x_axis_height,
                    width: axis_width,
                    split_number:data.len(),
                    font_family: self.font_family.clone(),
                    data,
                    font_color: Some(self.x_axis_font_color),
                    stroke_color: Some(self.x_axis_stroke_color),
                    font_size: self.x_axis_font_size,
                    name_gap: self.x_axis_name_gap,
                    name_rotate: self.x_axis_name_rotate,
                    ..Default::default()
                });
            }
        }
    };
    gen.into()
}
