extern crate proc_macro;
use proc_macro::TokenStream;
use quote::quote;
use syn::DeriveInput;

#[proc_macro_derive(ChartBasic)]
pub fn my_default(input: TokenStream) -> TokenStream {
    let ast: DeriveInput = syn::parse(input).unwrap();
    let id = ast.ident;

    let gen = quote! {
        impl ChartBasic for #id {
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
        }
    };
    gen.into()
}
