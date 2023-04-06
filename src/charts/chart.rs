use super::canvas::{AxisOption, Canvas, LEGEND_WIDTH};
use super::color::*;
use super::util::*;

#[derive(Clone, Debug, Default)]
pub struct ChartOption {
    pub width: f64,
    pub height: f64,
    pub margin: Option<Box>,
    pub families: String,
    pub font_size: f64,
    pub color: Color,
    pub background_color: Option<Color>,
    pub series_colors: Vec<Color>,

    // legend option
    pub legend_margin: Option<Box>,
    pub legend_labels: Vec<String>,
    pub legend_families: Option<String>,
    pub legend_font_size: Option<f64>,
    pub legend_color: Option<Color>,
    pub legend_orientation: Option<Orientation>,
    pub legend_icon: Option<LegendIcon>,

    // y axis option
    pub y_axis_margin: Option<Box>,
    pub y_axis_position: Position,
    pub y_axis_stroke_color: Color,
}

impl ChartOption {
    pub fn default() -> Self {
        ChartOption {
            width: 400.0,
            height: 300.0,
            margin: Some((20.0).into()),
            color: (216, 217, 218).into(),
            background_color: Some((31, 29, 29).into()),
            series_colors: get_grafana_series_colors(),
            families: "Times New Roman,Arial".to_string(),
            font_size: 14.0,

            legend_margin: None,
            legend_labels: vec![],
            legend_families: None,
            legend_font_size: None,
            legend_color: None,
            legend_orientation: None,
            legend_icon: Some(LegendIcon::LineDot),

            y_axis_margin: None,
            y_axis_position: Position::Left,
            y_axis_stroke_color: (185, 184, 206).into(),
        }
    }
}

#[derive(Clone)]
pub struct Chart {
    canvas: Canvas,
    opt: ChartOption,
}

impl Chart {
    pub fn new(opt: ChartOption) -> Result<Self> {
        let canvas = if let Some(ref margin) = opt.margin {
            Canvas::new_with_margin(opt.width, opt.height, margin.clone())?
        } else {
            Canvas::new(opt.width, opt.height)?
        };
        Ok(Chart { canvas, opt })
    }
    fn get_series_color(&self, index: usize) -> Color {
        let opt = &self.opt;
        let i = index % opt.series_colors.len();
        opt.series_colors[i]
    }
    fn render_legend(&self) -> Result<Box> {
        let opt = &self.opt;
        if opt.legend_labels.is_empty() {
            return Ok((0.0).into());
        }
        let canvas = self
            .canvas
            .child(opt.legend_margin.clone().unwrap_or_default());

        let families = opt
            .legend_families
            .clone()
            .unwrap_or_else(|| opt.families.clone());
        let font_size = opt.legend_font_size.unwrap_or(opt.font_size);
        let color = opt.legend_color.unwrap_or(opt.color);
        println!("{color:?}");

        let get_font_option =
            || -> Result<TextOption> { new_font_option(families.clone(), font_size, color) };
        let render_text = |text: String, margin: Box| -> Result<Box> {
            let font_option = get_font_option()?;
            let b = canvas.child(margin).text(text, font_option)?;
            Ok(b)
        };
        let measure_text =
            |text: String| -> Result<Box> { canvas.measure(text, get_font_option()?) };

        let render_legend_icon = |index: usize, margin: Box| -> Result<Box> {
            let color = self.get_series_color(index);
            let child = canvas.child(margin);
            // 已保证不为空
            let b = match opt.legend_icon.clone().unwrap() {
                LegendIcon::Rect => child.legend_rect(color)?,
                _ => child.legend_dot_line(color)?,
            };
            Ok(b)
        };

        let should_render_icon = opt.legend_icon.is_some();

        let mut b = Box::new_neg_infinity();

        let width = canvas.width();
        let mut left = 0.0;
        let mut top = 0.0;
        let mut bottom = 0.0;
        let unit_offset = 10.0;
        let icon_offset = 3.0;
        let unit_height_offset = 5.0;

        let orientation = opt.legend_orientation.clone().unwrap_or_default();

        for (index, text) in opt.legend_labels.iter().enumerate() {
            let text_box = measure_text(text.clone())?;
            match orientation {
                Orientation::Vertical => {
                    left = 0.0;
                    if index != 0 {
                        top = bottom + unit_height_offset;
                    }
                }
                _ => {
                    let mut legend_width = text_box.width();
                    if should_render_icon {
                        legend_width += LEGEND_WIDTH;
                    }
                    // 换下一行
                    if left + legend_width > width {
                        top += font_size + unit_height_offset;
                        left = 0.0;
                    }
                }
            }

            if should_render_icon {
                let area = render_legend_icon(
                    index,
                    Box {
                        left,
                        top: top + font_size / 2.0,
                        ..Default::default()
                    },
                )?;
                left += area.width() + icon_offset;
                b.merge(Box {
                    top,
                    right: left,
                    bottom: top + area.bottom,
                    ..Default::default()
                });
            }
            let area = render_text(
                text.clone(),
                Box {
                    top,
                    left,
                    ..Default::default()
                },
            )?;
            left += area.width() + unit_offset;
            bottom = top + area.height();
            b.merge(Box {
                top,
                right: left,
                bottom,
                ..Default::default()
            });
        }

        Ok(b)
    }
    fn render_y_axis(&self) -> Result<Box> {
        let opt = &self.opt;
        let canvas = self
            .canvas
            .child(opt.y_axis_margin.clone().unwrap_or_default());

        let mut axis_option: AxisOption = (Position::Left, 5).into();

        if opt.y_axis_position == Position::Right {
            axis_option.position = Position::Right;
        }
        print!("{:?}", opt.y_axis_stroke_color);
        canvas.axis(axis_option, opt.y_axis_stroke_color)

        // let position =
        // let mut axis_option:AxisOption = ();
        // Ok(Box::new_neg_infinity())
    }
    pub fn render(&self) -> Result<()> {
        self.render_legend()?;
        self.render_y_axis()?;
        Ok(())
    }
    pub fn to_svg(&self) -> String {
        self.canvas.to_svg(self.opt.background_color)
    }
    pub fn to_png(&self) -> Result<Vec<u8>> {
        self.canvas.to_png(self.opt.background_color)
    }
}
