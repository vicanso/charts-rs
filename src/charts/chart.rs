use super::canvas::Canvas;
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
    pub legend_labels: Vec<String>,
    pub legend_families: Option<String>,
    pub legend_font_size: Option<f64>,
    pub legend_color: Option<Color>,
    pub legend_icon: Option<LegendIcon>,
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
            legend_labels: vec![],
            legend_families: None,
            legend_font_size: None,
            legend_color: None,
            legend_icon: Some(LegendIcon::LineDot),
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

        let families = opt
            .legend_families
            .clone()
            .unwrap_or_else(|| opt.families.clone());
        let font_size = opt.legend_font_size.unwrap_or(opt.font_size);
        let color = opt
            .legend_color
            .clone()
            .unwrap_or_else(|| opt.color.clone());
        let mut left = 0.0;
        let unit_offset = 10.0;
        let icon_offset = 3.0;

        let render_text = |text: String, margin_left: f64| -> Result<Box> {
            let font_option = new_font_option(families.clone(), font_size, color)?;
            let b = self
                .canvas
                .child(Box {
                    left: margin_left,
                    ..Default::default()
                })
                .text(text.clone(), font_option)?;
            Ok(b)
        };

        let render_legend_icon = |index: usize, margin_left: f64| -> Result<Box> {
            let color = self.get_series_color(index);
            let child = self.canvas.child(Box {
                left: margin_left,
                top: font_size / 2.0,
                ..Default::default()
            });
            // 已保证不为空
            let b = match opt.legend_icon.clone().unwrap() {
                LegendIcon::Rect => child.legend_rect(color)?,
                _ => child.legend_dot_line(color)?,
            };
            Ok(b)
        };

        let should_render_icon = opt.legend_icon.is_some();

        let mut b = Box::new_neg_infinity();

        for (index, text) in opt.legend_labels.iter().enumerate() {
            if should_render_icon {
                let area = render_legend_icon(index, left)?;
                left += area.width() + icon_offset;
                b.merge(Box {
                    right: left,
                    bottom: area.bottom,
                    ..Default::default()
                });
            }
            let area = render_text(text.clone(), left)?;
            left += area.width() + unit_offset;
            b.merge(Box {
                right: left,
                bottom: area.bottom,
                ..Default::default()
            });
        }

        Ok(b)
    }
    pub fn render(&self) -> Result<()> {
        let legend_box = self.render_legend();
        println!("{legend_box:?}");
        Ok(())
    }
    pub fn to_svg(&self) -> String {
        self.canvas.to_svg(self.opt.background_color)
    }
    pub fn to_png(&self) -> Result<Vec<u8>> {
        self.canvas.to_png(self.opt.background_color)
    }
}
