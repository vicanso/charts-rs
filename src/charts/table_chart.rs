use super::canvas;
use super::color::*;
use super::common::*;
use super::component::*;
use super::font::text_wrap_fit;
use super::params::*;
use super::theme::{get_default_theme, get_theme, Theme};
use super::util::*;
use super::Canvas;
use crate::charts::measure_text_width_family;

#[derive(Clone, Debug, Default)]
pub struct TableCellStyle {
    pub font_color: Option<Color>,
    pub font_weight: Option<String>,
    pub background_color: Option<Color>,
    pub indexes: Vec<usize>,
}

#[derive(Clone, Debug, Default)]
pub struct TableChart {
    pub width: f32,
    pub height: f32,
    pub x: f32,
    pub y: f32,
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
    pub sub_title_font_weight: Option<String>,
    pub sub_title_margin: Option<Box>,
    pub sub_title_align: Align,
    pub sub_title_height: f32,

    pub data: Vec<Vec<String>>,
    pub spans: Vec<f32>,
    pub text_aligns: Vec<Align>,
    pub border_color: Color,

    pub header_row_padding: Box,
    pub header_row_height: f32,
    pub header_font_size: f32,
    pub header_font_weight: Option<String>,
    pub header_font_color: Color,
    pub header_background_color: Color,

    pub body_row_padding: Box,
    pub body_row_height: f32,
    pub body_font_size: f32,
    pub body_font_color: Color,
    pub body_background_colors: Vec<Color>,

    pub cell_styles: Vec<TableCellStyle>,
}

impl TableChart {
    fn fill_option(&mut self, data: &str) -> canvas::Result<serde_json::Value> {
        let data: serde_json::Value = serde_json::from_str(data)?;
        let theme = get_string_from_value(&data, "theme").unwrap_or_default();
        self.fill_theme(get_theme(&theme));

        if let Some(width) = get_f32_from_value(&data, "width") {
            self.width = width;
        }
        if let Some(height) = get_f32_from_value(&data, "height") {
            self.height = height;
        }
        if let Some(x) = get_f32_from_value(&data, "x") {
            self.x = x;
        }
        if let Some(y) = get_f32_from_value(&data, "y") {
            self.y = y;
        }
        if let Some(font_family) = get_string_from_value(&data, "font_family") {
            self.font_family = font_family;
        }
        if let Some(title_text) = get_string_from_value(&data, "title_text") {
            self.title_text = title_text;
        }
        if let Some(title_font_size) = get_f32_from_value(&data, "title_font_size") {
            self.title_font_size = title_font_size;
        }
        if let Some(title_font_color) = get_color_from_value(&data, "title_font_color") {
            self.title_font_color = title_font_color;
        }
        if let Some(title_font_weight) = get_string_from_value(&data, "title_font_weight") {
            self.title_font_weight = Some(title_font_weight);
        }
        if let Some(title_margin) = get_margin_from_value(&data, "title_margin") {
            self.title_margin = Some(title_margin);
        }
        if let Some(title_align) = get_align_from_value(&data, "title_align") {
            self.title_align = title_align;
        }
        if let Some(title_height) = get_f32_from_value(&data, "title_height") {
            self.title_height = title_height;
        }

        if let Some(sub_title_text) = get_string_from_value(&data, "sub_title_text") {
            self.sub_title_text = sub_title_text;
        }
        if let Some(sub_title_font_size) = get_f32_from_value(&data, "sub_title_font_size") {
            self.sub_title_font_size = sub_title_font_size;
        }
        if let Some(sub_title_font_color) = get_color_from_value(&data, "sub_title_font_color") {
            self.sub_title_font_color = sub_title_font_color;
        }
        if let Some(sub_title_font_weight) = get_string_from_value(&data, "sub_title_font_weight") {
            self.sub_title_font_weight = Some(sub_title_font_weight);
        }
        if let Some(sub_title_margin) = get_margin_from_value(&data, "sub_title_margin") {
            self.sub_title_margin = Some(sub_title_margin);
        }
        if let Some(sub_title_align) = get_align_from_value(&data, "sub_title_align") {
            self.sub_title_align = sub_title_align;
        }
        if let Some(sub_title_height) = get_f32_from_value(&data, "sub_title_height") {
            self.sub_title_height = sub_title_height;
        }
        if let Some(data) = data.get("cell_styles") {
            if let Some(arr) = data.as_array() {
                let mut cell_styles = vec![];
                for item in arr.iter() {
                    let mut style = TableCellStyle {
                        ..Default::default()
                    };
                    if let Some(font_color) = get_color_from_value(item, "font_color") {
                        style.font_color = Some(font_color);
                    }
                    if let Some(background_color) = get_color_from_value(item, "background_color") {
                        style.background_color = Some(background_color);
                    }
                    if let Some(font_weight) = get_string_from_value(item, "font_weight") {
                        style.font_weight = Some(font_weight);
                    }
                    if let Some(indexes) = get_usize_slice_from_value(item, "indexes") {
                        style.indexes = indexes;
                    }
                    cell_styles.push(style);
                }
                self.cell_styles = cell_styles;
            }
        }

        if let Some(data) = data.get("data") {
            if let Some(arr) = data.as_array() {
                let mut data_list = vec![];
                for items in arr.iter() {
                    let mut list = vec![];
                    if let Some(sub_arr) = items.as_array() {
                        for item in sub_arr.iter() {
                            if let Some(str) = item.as_str() {
                                list.push(str.to_string());
                            }
                        }
                    }
                    data_list.push(list);
                }
                self.data = data_list;
            }
        }
        if let Some(spans) = get_f32_slice_from_value(&data, "spans") {
            self.spans = spans;
        }
        if let Some(text_aligns) = get_align_slice_from_value(&data, "text_aligns") {
            self.text_aligns = text_aligns;
        }
        if let Some(header_row_padding) = get_margin_from_value(&data, "header_row_padding") {
            self.header_row_padding = header_row_padding;
        }
        if let Some(header_row_height) = get_f32_from_value(&data, "header_row_height") {
            self.header_row_height = header_row_height;
        }
        if let Some(header_font_size) = get_f32_from_value(&data, "header_font_size") {
            self.header_font_size = header_font_size;
        }
        if let Some(header_font_weight) = get_string_from_value(&data, "header_font_weight") {
            self.header_font_weight = Some(header_font_weight);
        }
        if let Some(header_font_color) = get_color_from_value(&data, "header_font_color") {
            self.header_font_color = header_font_color;
        }
        if let Some(header_background_color) =
            get_color_from_value(&data, "header_background_color")
        {
            self.header_background_color = header_background_color;
        }
        if let Some(body_row_padding) = get_margin_from_value(&data, "body_row_padding") {
            self.body_row_padding = body_row_padding;
        }
        if let Some(body_row_height) = get_f32_from_value(&data, "body_row_height") {
            self.body_row_height = body_row_height;
        }
        if let Some(body_font_size) = get_f32_from_value(&data, "body_font_size") {
            self.body_font_size = body_font_size;
        }
        if let Some(body_font_color) = get_color_from_value(&data, "body_font_color") {
            self.body_font_color = body_font_color;
        }
        if let Some(body_background_colors) =
            get_color_slice_from_value(&data, "body_background_colors")
        {
            self.body_background_colors = body_background_colors;
        }
        if let Some(border_color) = get_color_from_value(&data, "border_color") {
            self.border_color = border_color;
        }
        Ok(data)
    }
    /// Creates a table chart from json.
    pub fn from_json(data: &str) -> canvas::Result<TableChart> {
        let mut t = TableChart::new_with_theme(vec![], "");
        t.fill_option(data)?;
        Ok(t)
    }
    /// Creates a table chart with custom theme.
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
    fn fill_theme(&mut self, t: &Theme) {
        self.font_family = t.font_family.clone();
        self.width = t.width;
        self.background_color = t.background_color;

        self.title_font_color = t.title_font_color;
        self.title_font_size = t.title_font_size;
        self.title_font_weight = t.title_font_weight.clone();
        self.title_margin = t.title_margin.clone();
        self.title_align = t.title_align.clone();
        self.title_height = t.title_height * 1.5;

        self.sub_title_font_color = t.sub_title_font_color;
        self.sub_title_font_size = t.sub_title_font_size;
        self.sub_title_margin = t.sub_title_margin.clone();
        self.sub_title_align = t.sub_title_align.clone();
        self.sub_title_height = t.sub_title_height;

        self.header_font_size = t.sub_title_font_size;
        self.header_font_color = t.sub_title_font_color;
        self.header_background_color = t.table_header_color;

        self.body_font_size = t.sub_title_font_size;
        self.body_font_color = t.sub_title_font_color;
        self.body_background_colors = t.table_body_colors.clone();
        self.border_color = t.table_border_color;
    }
    /// Creates a table chart with default theme.
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
            title_height = b.height() + title_margin_bottom;
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
                font_weight: self.sub_title_font_weight.clone(),
                line_height: Some(self.sub_title_height),
                x: Some(x),
                ..Default::default()
            });
            title_height = b.outer_height() + sub_title_margin_bottom;
        }
        title_height
    }
    /// Converts bar chart to svg.
    pub fn svg(&mut self) -> canvas::Result<String> {
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

        let mut c = Canvas::new_width_xy(self.width, self.height, self.x, self.y);

        if !self.title_text.is_empty() {
            let mut title_height = self.title_height;
            if let Some(value) = self.title_margin.clone() {
                title_height += value.top + value.bottom;
            }
            if !self.sub_title_text.is_empty() {
                title_height += self.sub_title_height;
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

        let find_cell_style = |row: usize, column: usize| -> Option<&TableCellStyle> {
            for cell_style in self.cell_styles.iter() {
                if cell_style.indexes.len() != 2 {
                    continue;
                }
                if cell_style.indexes[0] != row || cell_style.indexes[1] != column {
                    continue;
                }
                return Some(cell_style);
            }
            None
        };

        let mut table_content_list = vec![];
        for (i, items) in self.data.iter().enumerate() {
            let mut font_size = self.body_font_size;
            let is_header = i == 0;
            if is_header {
                font_size = self.header_font_size;
            }

            let mut row_content_list = vec![];
            for (j, item) in items.iter().enumerate() {
                // 已保证肯定有数据
                let span_width = spans[j];
                if let Ok(result) = text_wrap_fit(&self.font_family, font_size, item, span_width) {
                    row_content_list.push(result);
                } else {
                    row_content_list.push(vec![item.clone()]);
                }
            }
            table_content_list.push(row_content_list);
        }
        let mut top = 0.0;
        let body_background_color_count = self.body_background_colors.len();

        for (i, items) in table_content_list.iter().enumerate() {
            let mut left = 0.0;
            let mut right = 0.0;
            let mut line_height = self.body_row_height;
            let mut padding = self.body_row_padding.top + self.body_row_padding.bottom;
            let mut font_size = self.body_font_size;
            let mut font_color = self.body_font_color;

            let is_header = i == 0;
            let mut font_weight = None;
            let bg_color = if is_header {
                line_height = self.header_row_height;
                padding = self.header_row_padding.top + self.header_row_padding.bottom;
                font_size = self.header_font_size;
                font_color = self.header_font_color;
                font_weight = self.header_font_weight.clone();
                self.header_background_color
            } else {
                self.body_background_colors[(i - 1) % body_background_color_count]
            };

            let row_padding = if is_header {
                self.header_row_padding.clone()
            } else {
                self.body_row_padding.clone()
            };
            let mut count = 0;
            for content_list in items.iter() {
                if count < content_list.len() {
                    count = content_list.len();
                }
            }
            let row_height = line_height * count as f32 + padding;

            c.rect(Rect {
                fill: Some(bg_color),
                top,
                width: c.width(),
                height: row_height,
                ..Default::default()
            });
            if !self.border_color.is_transparent() {
                c.line(Line {
                    color: Some(self.border_color),
                    stroke_width: 1.0,
                    top,
                    right: c.width(),
                    bottom: top,
                    ..Default::default()
                });
            }
            for (j, content_list) in items.iter().enumerate() {
                // 已保证肯定有数据
                let span_width = spans[j];

                let mut cell_font_color = font_color;
                let mut cell_font_weight = font_weight.clone();

                // 每个table cell的背景色
                if let Some(cell_style) = find_cell_style(i, j) {
                    // 有配置则设置
                    if let Some(value) = cell_style.font_color {
                        cell_font_color = value;
                    }
                    if let Some(ref value) = cell_style.font_weight {
                        cell_font_weight = Some(value.clone());
                    }
                    if let Some(value) = cell_style.background_color {
                        c.rect(Rect {
                            fill: Some(value),
                            left,
                            top: top + 1.0,
                            width: span_width,
                            height: row_height - 1.0,
                            ..Default::default()
                        });
                    }
                }

                for (index, item) in content_list.iter().enumerate() {
                    let mut dx = None;
                    if let Ok(measurement) =
                        measure_text_width_family(&self.font_family, font_size, item)
                    {
                        let mut align = Align::Left;
                        if let Some(value) = self.text_aligns.get(j) {
                            align = value.to_owned();
                        }
                        let text_width = measurement.width();
                        let text_max_width = span_width - row_padding.left - row_padding.right;
                        dx = match align {
                            Align::Center => Some((text_max_width - text_width) / 2.0),
                            Align::Right => Some(text_max_width - text_width),
                            Align::Left => None,
                        };
                    }
                    c.child(row_padding.clone()).text(Text {
                        text: item.to_string(),
                        font_weight: cell_font_weight.clone(),
                        font_family: Some(self.font_family.clone()),
                        font_size: Some(font_size),
                        font_color: Some(cell_font_color),
                        line_height: Some(line_height),
                        dx,
                        x: Some(left),
                        y: Some(top + line_height * index as f32),
                        ..Default::default()
                    });
                }

                right += span_width;
                left = right
            }
            top += row_height;
        }

        c.height = c.margin.top + top;
        self.height = c.height;

        c.svg()
    }
}

#[cfg(test)]
mod tests {
    use super::{TableCellStyle, TableChart};
    use crate::{Align, THEME_ANT, THEME_DARK, THEME_GRAFANA};
    use pretty_assertions::assert_eq;

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
        table_chart.cell_styles = vec![TableCellStyle {
            indexes: vec![1, 2],
            font_weight: Some("bold".to_string()),
            background_color: Some("#3bb357".into()),
            font_color: Some(("#fff").into()),
        }];
        assert_eq!(
            include_str!("../../asset/table_chart/basic.svg"),
            table_chart.svg().unwrap()
        );
    }

    #[test]
    fn table_multi_lines() {
        let mut table_chart = TableChart::new(vec![
            vec![
                "Name".to_string(),
                "Price".to_string(),
                "ChangeChangeChangeChangeChangeChange".to_string(),
            ],
            vec![
                "Datadog Inc".to_string(),
                "97.32".to_string(),
                "-7.49%".to_string(),
            ],
            vec![
                "Hashicorp Inc".to_string(),
                "28.66".to_string(),
                "Hashicorp Inc Hashicorp Inc -9.25%".to_string(),
            ],
            vec![
                "Gitlab Inc".to_string(),
                "51.63".to_string(),
                "+4.32%".to_string(),
            ],
        ]);
        table_chart.title_text = "NASDAQ".to_string();
        table_chart.cell_styles = vec![TableCellStyle {
            indexes: vec![1, 2],
            font_weight: Some("bold".to_string()),
            background_color: Some("#3bb357".into()),
            font_color: Some(("#fff").into()),
        }];
        assert_eq!(
            include_str!("../../asset/table_chart/multi_lines.svg"),
            table_chart.svg().unwrap()
        );
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
        table_chart.text_aligns = vec![Align::Left, Align::Center, Align::Right];
        assert_eq!(
            include_str!("../../asset/table_chart/basic_dark.svg"),
            table_chart.svg().unwrap()
        );
    }

    #[test]
    fn table_basic_ant() {
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
            THEME_ANT,
        );
        table_chart.title_text = "NASDAQ".to_string();
        table_chart.text_aligns = vec![Align::Left, Align::Center, Align::Right];
        assert_eq!(
            include_str!("../../asset/table_chart/basic_ant.svg"),
            table_chart.svg().unwrap()
        );
    }

    #[test]
    fn table_basic_grafana() {
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
            THEME_GRAFANA,
        );
        table_chart.title_text = "NASDAQ".to_string();
        table_chart.sub_title_text = "stock".to_string();
        table_chart.spans = vec![0.5, 0.3, 0.2];
        let green = "#2d7c2b".into();
        let red = "#a93b01".into();
        table_chart.cell_styles = vec![
            TableCellStyle {
                indexes: vec![1, 2],
                font_weight: Some("bold".to_string()),
                background_color: Some(green),
                ..Default::default()
            },
            TableCellStyle {
                indexes: vec![2, 2],
                font_weight: Some("bold".to_string()),
                background_color: Some(green),
                ..Default::default()
            },
            TableCellStyle {
                indexes: vec![3, 2],
                font_weight: Some("bold".to_string()),
                background_color: Some(red),
                ..Default::default()
            },
        ];
        table_chart.text_aligns = vec![Align::Left, Align::Center, Align::Center];
        assert_eq!(
            include_str!("../../asset/table_chart/basic_grafana.svg"),
            table_chart.svg().unwrap()
        );
    }
}
