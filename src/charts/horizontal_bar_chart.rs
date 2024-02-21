use super::canvas;
use super::color::*;
use super::common::*;
use super::component::*;
use super::params::*;
use super::theme::{get_default_theme, get_theme, Theme, DEFAULT_Y_AXIS_WIDTH};
use super::util::*;
use super::Canvas;
use crate::charts::measure_text_width_family;
use charts_rs_derive::Chart;

#[derive(Clone, Debug, Default, Chart)]
pub struct HorizontalBarChart {
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
    pub series_label_position: Option<Position>,
    pub series_colors: Vec<Color>,
    pub series_symbol: Option<Symbol>,
    pub series_smooth: bool,
    pub series_fill: bool,
}

impl HorizontalBarChart {
    /// Creates a horizontal bar from json.
    pub fn from_json(data: &str) -> canvas::Result<HorizontalBarChart> {
        let mut h = HorizontalBarChart {
            ..Default::default()
        };
        let value = h.fill_option(data)?;
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
            series_list,
            x_axis_data,
            ..Default::default()
        };
        let theme = get_theme(theme);
        h.fill_theme(theme);
        h
    }
    /// Creates a horizontal bar with default theme.
    pub fn new(series_list: Vec<Series>, x_axis_data: Vec<String>) -> HorizontalBarChart {
        HorizontalBarChart::new_with_theme(series_list, x_axis_data, &get_default_theme())
    }
    /// Converts horizontal bar chart to svg.
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

        let x_axis_height = 25.0_f32;
        let axis_height = c.height() - axis_top - x_axis_height;
        // minus the height of top text area
        if axis_top > 0.0 {
            c = c.child(Box {
                top: axis_top,
                ..Default::default()
            });
        }

        let mut data = self.x_axis_data.clone();
        data.reverse();
        let mut max_width = 0.0;
        for text in data.iter() {
            if let Ok(b) = measure_text_width_family(&self.font_family, self.x_axis_font_size, text)
            {
                if b.width() > max_width {
                    max_width = b.width();
                }
            }
        }

        let y_axis_width = max_width + 5.0;

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
            data,
            ..Default::default()
        });

        let mut data_list = vec![];
        for series in self.series_list.iter() {
            data_list.append(series.data.clone().as_mut());
        }
        let x_axis_config = self.get_y_axis_config(0);
        let x_axis_values = get_axis_values(AxisValueParams {
            data_list,
            split_number: x_axis_config.axis_split_number,
            ..Default::default()
        });

        let x_axis_width = c.width() - y_axis_width;
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
            data: x_axis_values.data.clone(),
            ..Default::default()
        });

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
        if !self.series_list.is_empty() {
            let mut c1 = c.child(Box {
                left: y_axis_width,
                bottom: x_axis_height,
                ..Default::default()
            });
            let max_width = c1.width();
            let unit_height = c1.height() / self.series_list[0].data.len() as f32;
            let bar_chart_margin = 5.0_f32;
            let bar_chart_gap = 3.0_f32;

            let bar_chart_margin_height = bar_chart_margin * 2.0;
            let bar_chart_gap_height = bar_chart_gap * (self.series_list.len() - 1) as f32;
            let bar_height = (unit_height - bar_chart_margin_height - bar_chart_gap_height)
                / self.series_list.len() as f32;
            let half_bar_height = bar_height / 2.0;

            let mut series_labels_list = vec![];
            for (index, series) in self.series_list.iter().enumerate() {
                let color = get_color(&self.series_colors, series.index.unwrap_or(index));

                let mut series_labels = vec![];
                let series_data_count = series.data.len();
                for (i, p) in series.data.iter().enumerate() {
                    let value = p.to_owned();
                    if value == NIL_VALUE {
                        continue;
                    }
                    let mut top =
                        unit_height * (series_data_count - i - 1) as f32 + bar_chart_margin;
                    top += (bar_height + bar_chart_gap) * index as f32;

                    let x = max_width - x_axis_values.get_offset_height(value, max_width);
                    c1.rect(Rect {
                        fill: Some(color),
                        top,
                        width: x,
                        height: bar_height,
                        ..Default::default()
                    });
                    series_labels.push(SeriesLabel {
                        point: (x, top + half_bar_height).into(),
                        text: format_series_value(value, &self.series_label_formatter),
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
                        x,
                        y: Some(series_label.point.y),
                        ..Default::default()
                    });
                }
            }
        }

        c.svg()
    }
}

#[cfg(test)]
mod tests {
    use super::HorizontalBarChart;
    use crate::{Align, Position, NIL_VALUE};
    use pretty_assertions::assert_eq;
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
        assert_eq!(
            include_str!("../../asset/horizontal_bar_chart/basic.svg"),
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
        assert_eq!(
            include_str!("../../asset/horizontal_bar_chart/basic_label_inside.svg"),
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
        assert_eq!(
            include_str!("../../asset/horizontal_bar_chart/nil_value.svg"),
            horizontal_bar_chart.svg().unwrap()
        );
    }
}
