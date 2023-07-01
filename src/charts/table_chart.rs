use super::canvas;
use super::color::*;
use super::common::*;
use super::component::*;
use super::theme::{get_default_theme, get_theme, Theme};
use super::util::*;
use super::Canvas;
use crate::charts::measure_text_width_family;

#[derive(Clone, Debug, Default)]
pub struct TableChart {
    pub width: f32,
    pub height: f32,
    pub font_family: String,
    pub background_color: Color,

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
    pub sub_title_margin: Option<Box>,
    pub sub_title_align: Align,
    pub sub_title_height: f32,

    pub data: Vec<Vec<String>>,
    pub spans: Vec<f32>,

    pub header_row_padding: Box,
    pub header_row_height: f32,
    pub header_font_size: f32,
    pub header_font_color: Color,
    pub header_background_color: Color,

    pub body_row_padding: Box,
    pub body_row_height: f32,
    pub body_font_size: f32,
    pub body_font_color: Color,
    pub body_background_colors: Vec<Color>,
}

impl TableChart {
    pub fn new_with_theme(data: Vec<Vec<String>>, theme: &str) -> TableChart {
        let mut table = TableChart {
            data,
            header_row_padding: (10.0, 8.0).into(),
            header_row_height: 30.0,
            body_row_padding: (10.0, 5.0).into(),
            body_row_height: 30.0,
            ..Default::default()
        };
        table.fill_theme(get_theme(theme));
        table
    }
    fn fill_theme(&mut self, t: Theme) {
        self.font_family = t.font_family;
        self.width = t.width;
        self.background_color = t.background_color;

        self.title_font_color = t.title_font_color;
        self.title_font_size = t.title_font_size;
        self.title_font_weight = t.title_font_weight;
        self.title_margin = t.title_margin;
        self.title_align = t.title_align;
        self.title_height = t.title_height * 1.5;

        self.sub_title_font_color = t.sub_title_font_color;
        self.sub_title_font_size = t.sub_title_font_size;
        self.sub_title_margin = t.sub_title_margin;
        self.sub_title_align = t.sub_title_align;
        self.sub_title_height = t.sub_title_height;

        self.header_font_size = t.sub_title_font_size;
        self.header_font_color = t.sub_title_font_color;
        self.header_background_color = t.table_header_color;

        self.body_font_size = t.sub_title_font_size;
        self.body_font_color = t.sub_title_font_color;
        self.body_background_colors = t.table_body_colors;
    }
    pub fn new(data: Vec<Vec<String>>) -> TableChart {
        TableChart::new_with_theme(data, &get_default_theme())
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
    pub fn svg(&self) -> canvas::Result<String> {
        if self.data.is_empty() {
            return Err(canvas::Error::Params {
                message: "data is empty".to_string(),
            });
        }
        let column_count = self.data[0].len();
        if column_count == 0 {
            return Err(canvas::Error::Params {
                message: "table header column is empty".to_string(),
            });
        }
        for item in self.data.iter() {
            if item.len() != column_count {
                return Err(canvas::Error::Params {
                    message: "data len is invalid".to_string(),
                });
            }
        }

        let mut c = Canvas::new(self.width, self.height);

        if !self.title_text.is_empty() {
            let mut title_height = self.title_height;
            if let Some(value) = self.title_margin.clone() {
                title_height += value.top + value.bottom;
            }
            c.rect(Rect {
                fill: Some(self.background_color),
                left: 0.0,
                top: 0.0,
                width: self.width,
                height: title_height,
                ..Default::default()
            });
        }

        let title_height = self.render_title(c.child(Box::default()));

        c = c.child(Box {
            top: title_height,
            ..Default::default()
        });
        let width = c.width();
        let spans = if self.spans.len() != column_count {
            let mut spans = vec![];
            let span = 1.0 / column_count as f32;
            for _ in 0..column_count {
                spans.push(span * width);
            }
            spans
        } else {
            self.spans.iter().map(|value| value * width).collect()
        };
        let mut top = 0.0;
        let body_background_color_count = self.body_background_colors.len();
        for (i, items) in self.data.iter().enumerate() {
            let mut left = 0.0;
            let mut right = 0.0;
            let mut cell_height = self.body_row_height;
            let mut padding = self.body_row_padding.top + self.body_row_padding.bottom;
            let mut font_size = self.body_font_size;
            let mut font_color = self.body_font_color;

            let is_header = i == 0;
            // self.body_row_padding.clone()
            // title
            let bg_color = if is_header {
                cell_height = self.header_row_height;
                padding = self.header_row_padding.top + self.header_row_padding.bottom;
                font_size = self.header_font_size;
                font_color = self.header_font_color;
                self.header_background_color
            } else {
                self.body_background_colors[(i - 1) % body_background_color_count]
            };

            let row_padding = if is_header {
                self.header_row_padding.clone()
            } else {
                self.body_row_padding.clone()
            };
            let row_height = cell_height + padding;

            c.rect(Rect {
                fill: Some(bg_color),
                top,
                width: c.width(),
                height: row_height,
                ..Default::default()
            });
            for (j, item) in items.iter().enumerate() {
                // 已保证肯定有数据
                let span_width = spans[j];
                let mut y_offset = 0.0;
                // if let Ok(value) =
                //     measure_text_vertical_center(&self.font_family, font_size, item, cell_height)
                // {
                //     y_offset = value;
                // }
                right += span_width;
                c.child(row_padding.clone()).text(Text {
                    text: item.to_string(),
                    font_family: Some(self.font_family.clone()),
                    font_size: Some(font_size),
                    font_color: Some(font_color),
                    x: Some(left),
                    y: Some(top + y_offset),
                    ..Default::default()
                });

                left = right
            }
            top += row_height;
        }

        c.height = c.margin.top + top;

        c.svg()
    }
}

#[cfg(test)]
mod tests {
    use super::TableChart;
    use crate::THEME_DARK;

    #[test]
    fn table_basic() {
        let mut table_chart = TableChart::new(vec![
            vec![
                "Name".to_string(),
                "Price".to_string(),
                "Change".to_string(),
            ],
            vec![
                "Datadog Inc".to_string(),
                "97.32".to_string(),
                "-7.49%".to_string(),
            ],
            vec![
                "Hashicorp Inc".to_string(),
                "28.66".to_string(),
                "-9.25%".to_string(),
            ],
            vec![
                "Gitlab Inc".to_string(),
                "51.63".to_string(),
                "+4.32%".to_string(),
            ],
        ]);
        table_chart.title_text = "NASDAQ".to_string();

        println!("{}", table_chart.svg().unwrap());

        // assert_eq!(
        //     include_str!("../../asset/table_chart/basic.svg"),
        //     table_chart.svg().unwrap()
        // );
    }

    #[test]
    fn table_basic_dark() {
        let mut table_chart = TableChart::new_with_theme(
            vec![
                vec![
                    "Name".to_string(),
                    "Price".to_string(),
                    "Change".to_string(),
                ],
                vec![
                    "Datadog Inc".to_string(),
                    "97.32".to_string(),
                    "-7.49%".to_string(),
                ],
                vec![
                    "Hashicorp Inc".to_string(),
                    "28.66".to_string(),
                    "-9.25%".to_string(),
                ],
                vec![
                    "Gitlab Inc".to_string(),
                    "51.63".to_string(),
                    "+4.32%".to_string(),
                ],
            ],
            THEME_DARK,
        );
        table_chart.title_text = "NASDAQ".to_string();
        table_chart.header_row_height = 50.0;

        println!("{}", table_chart.svg().unwrap());
        // assert_eq!(
        //     include_str!("../../asset/table_chart/basic_dark.svg"),
        //     table_chart.svg().unwrap()
        // );
    }
}
