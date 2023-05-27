use super::canvas;
use super::color::*;
use super::common::*;
use super::component::*;
use super::theme::{get_default_theme, get_theme, Theme};
use super::util::*;
use super::Canvas;
use super::Chart;
use crate::charts::measure_text_width_family;
use charts_rs_derive::Chart;

#[derive(Clone, Debug, Default, Chart)]
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
        let theme = get_theme(get_default_theme());
        l.fill_theme(theme);
        l
    }
    pub fn svg(&self) -> canvas::Result<String> {
        let mut c = Canvas::new(self.width, self.height);

        self.render_background(c.child(Box::default()));
        c.margin = self.margin.clone();

        let title_height = self.render_title(c.child(Box::default()));

        let legend_height = self.render_legend(c.child(Box::default()));
        let axis_top = if legend_height > title_height {
            legend_height
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

        self.render_grid(c.child(Box::default()), axis_width, axis_height);

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
        self.render_y_axis(
            c.child(Box::default()),
            y_axis_values.data.clone(),
            axis_height,
        );

        // x axis
        self.render_x_axis(
            c.child(Box {
                top: c.height() - self.x_axis_height,
                left: self.y_axis_width,
                ..Default::default()
            }),
            self.x_axis_data.clone(),
            axis_width,
        );

        // line point
        let max_height = c.height() - self.x_axis_height;
        self.render_lines(
            c.child(Box {
                left: self.y_axis_width,
                ..Default::default()
            }),
            &self.series_list,
            &y_axis_values,
            max_height,
            axis_height,
        );

        c.svg()
    }
}

#[cfg(test)]
mod tests {
    use super::LineChart;
    use crate::{Box, Series};
    #[test]
    fn line_chart() {
        let mut line_chart = LineChart::new(
            vec![
                Series::new(
                    "Email".to_string(),
                    vec![120.0, 132.0, 101.0, 134.0, 90.0, 230.0, 210.0],
                ),
                Series::new(
                    "Union Ads".to_string(),
                    vec![220.0, 182.0, 191.0, 234.0, 290.0, 330.0, 310.0],
                ),
                Series::new(
                    "Direct".to_string(),
                    vec![320.0, 332.0, 301.0, 334.0, 390.0, 330.0, 320.0],
                ),
                Series::new(
                    "Search Engine".to_string(),
                    vec![820.0, 932.0, 901.0, 934.0, 1290.0, 1330.0, 1320.0],
                ),
            ],
            vec![
                "Mon".to_string(),
                "Tue".to_string(),
                "Wed".to_string(),
                "Thu".to_string(),
                "Fri".to_string(),
                "Sat".to_string(),
                "Sun".to_string(),
            ],
        );
        line_chart.title_text = "Stacked Area Chart".to_string();
        line_chart.sub_title_text = "Hello World".to_string();
        line_chart.legend_margin = Some(Box {
            top: 40.0,
            bottom: 10.0,
            ..Default::default()
        });

        assert_eq!(
            r###"<svg width="600" height="400" viewBox="0 0 600 400" xmlns="http://www.w3.org/2000/svg">
<rect x="0" y="0" width="600" height="400" fill="#FFFFFF"/>
<text font-size="18" x="215.5" y="23" font-weight="bold" font-family="Arial" fill="#464646">
Stacked Area Chart
</text>
<text font-size="14" x="261.5" y="37" font-family="Arial" fill="#464646">
Hello World
</text>
<g>
<line stroke-width="2" x1="112" y1="55" x2="137" y2="55" stroke="#5470C6"/>
<circle cx="124.5" cy="55" r="5.5" stroke-width="2" stroke="#5470C6" fill="#FFFFFF"/>
<text font-size="14" x="140" y="59" font-family="Arial" fill="#464646">
Email
</text>
</g>
<g>
<line stroke-width="2" x1="182" y1="55" x2="207" y2="55" stroke="#91CC75"/>
<circle cx="194.5" cy="55" r="5.5" stroke-width="2" stroke="#91CC75" fill="#FFFFFF"/>
<text font-size="14" x="210" y="59" font-family="Arial" fill="#464646">
Union Ads
</text>
</g>
<g>
<line stroke-width="2" x1="283" y1="55" x2="308" y2="55" stroke="#FAC858"/>
<circle cx="295.5" cy="55" r="5.5" stroke-width="2" stroke="#FAC858" fill="#FFFFFF"/>
<text font-size="14" x="311" y="59" font-family="Arial" fill="#464646">
Direct
</text>
</g>
<g>
<line stroke-width="2" x1="355" y1="55" x2="380" y2="55" stroke="#EE6666"/>
<circle cx="367.5" cy="55" r="5.5" stroke-width="2" stroke="#EE6666" fill="#FFFFFF"/>
<text font-size="14" x="383" y="59" font-family="Arial" fill="#464646">
Search Engine
</text>
</g>
<g stroke="#E0E6F2">
<line stroke-width="1" x1="45" y1="77" x2="595" y2="77"/><line stroke-width="1" x1="45" y1="125" x2="595" y2="125"/><line stroke-width="1" x1="45" y1="173" x2="595" y2="173"/><line stroke-width="1" x1="45" y1="221" x2="595" y2="221"/><line stroke-width="1" x1="45" y1="269" x2="595" y2="269"/><line stroke-width="1" x1="45" y1="317" x2="595" y2="317"/>
</g>
<g>

<text font-size="14" x="5" y="84" font-family="Arial" fill="#6E7079">
1800
</text>
<text font-size="14" x="5" y="132" font-family="Arial" fill="#6E7079">
1500
</text>
<text font-size="14" x="5" y="180" font-family="Arial" fill="#6E7079">
1200
</text>
<text font-size="14" x="13" y="228" font-family="Arial" fill="#6E7079">
900
</text>
<text font-size="14" x="13" y="276" font-family="Arial" fill="#6E7079">
600
</text>
<text font-size="14" x="13" y="324" font-family="Arial" fill="#6E7079">
300
</text>
<text font-size="14" x="29" y="372" font-family="Arial" fill="#6E7079">
0
</text>
</g>
<g>
<g stroke="#6E7079">
<line stroke-width="1" x1="45" y1="365" x2="595" y2="365"/>
<line stroke-width="1" x1="45" y1="365" x2="45" y2="370"/>
<line stroke-width="1" x1="123.6" y1="365" x2="123.6" y2="370"/>
<line stroke-width="1" x1="202.1" y1="365" x2="202.1" y2="370"/>
<line stroke-width="1" x1="280.7" y1="365" x2="280.7" y2="370"/>
<line stroke-width="1" x1="359.3" y1="365" x2="359.3" y2="370"/>
<line stroke-width="1" x1="437.9" y1="365" x2="437.9" y2="370"/>
<line stroke-width="1" x1="516.4" y1="365" x2="516.4" y2="370"/>
<line stroke-width="1" x1="595" y1="365" x2="595" y2="370"/>
</g>
<text font-size="14" x="70.8" y="384" font-family="Arial" fill="#6E7079">
Mon
</text>
<text font-size="14" x="150.4" y="384" font-family="Arial" fill="#6E7079">
Tue
</text>
<text font-size="14" x="226.9" y="384" font-family="Arial" fill="#6E7079">
Wed
</text>
<text font-size="14" x="308" y="384" font-family="Arial" fill="#6E7079">
Thu
</text>
<text font-size="14" x="390.1" y="384" font-family="Arial" fill="#6E7079">
Fri
</text>
<text font-size="14" x="466.1" y="384" font-family="Arial" fill="#6E7079">
Sat
</text>
<text font-size="14" x="543.2" y="384" font-family="Arial" fill="#6E7079">
Sun
</text>
</g>
<g>
<path fill="none" d="M 84.3 345.8 L 162.9 343.9 L 241.4 348.8 L 320 343.6 L 398.6 350.6 L 477.1 328.2 L 555.7 331.4" stroke-width="2" stroke="#5470C6"/>
<circle cx="84.3" cy="345.8" r="2" stroke-width="2" stroke="#5470C6" fill="#FFFFFF"/>
<circle cx="162.9" cy="343.9" r="2" stroke-width="2" stroke="#5470C6" fill="#FFFFFF"/>
<circle cx="241.4" cy="348.8" r="2" stroke-width="2" stroke="#5470C6" fill="#FFFFFF"/>
<circle cx="320" cy="343.6" r="2" stroke-width="2" stroke="#5470C6" fill="#FFFFFF"/>
<circle cx="398.6" cy="350.6" r="2" stroke-width="2" stroke="#5470C6" fill="#FFFFFF"/>
<circle cx="477.1" cy="328.2" r="2" stroke-width="2" stroke="#5470C6" fill="#FFFFFF"/>
<circle cx="555.7" cy="331.4" r="2" stroke-width="2" stroke="#5470C6" fill="#FFFFFF"/>
</g>
<g>
<path fill="none" d="M 84.3 329.8 L 162.9 335.9 L 241.4 334.4 L 320 327.6 L 398.6 318.6 L 477.1 312.2 L 555.7 315.4" stroke-width="2" stroke="#91CC75"/>
<circle cx="84.3" cy="329.8" r="2" stroke-width="2" stroke="#91CC75" fill="#FFFFFF"/>
<circle cx="162.9" cy="335.9" r="2" stroke-width="2" stroke="#91CC75" fill="#FFFFFF"/>
<circle cx="241.4" cy="334.4" r="2" stroke-width="2" stroke="#91CC75" fill="#FFFFFF"/>
<circle cx="320" cy="327.6" r="2" stroke-width="2" stroke="#91CC75" fill="#FFFFFF"/>
<circle cx="398.6" cy="318.6" r="2" stroke-width="2" stroke="#91CC75" fill="#FFFFFF"/>
<circle cx="477.1" cy="312.2" r="2" stroke-width="2" stroke="#91CC75" fill="#FFFFFF"/>
<circle cx="555.7" cy="315.4" r="2" stroke-width="2" stroke="#91CC75" fill="#FFFFFF"/>
</g>
<g>
<path fill="none" d="M 84.3 313.8 L 162.9 311.9 L 241.4 316.8 L 320 311.6 L 398.6 302.6 L 477.1 312.2 L 555.7 313.8" stroke-width="2" stroke="#FAC858"/>
<circle cx="84.3" cy="313.8" r="2" stroke-width="2" stroke="#FAC858" fill="#FFFFFF"/>
<circle cx="162.9" cy="311.9" r="2" stroke-width="2" stroke="#FAC858" fill="#FFFFFF"/>
<circle cx="241.4" cy="316.8" r="2" stroke-width="2" stroke="#FAC858" fill="#FFFFFF"/>
<circle cx="320" cy="311.6" r="2" stroke-width="2" stroke="#FAC858" fill="#FFFFFF"/>
<circle cx="398.6" cy="302.6" r="2" stroke-width="2" stroke="#FAC858" fill="#FFFFFF"/>
<circle cx="477.1" cy="312.2" r="2" stroke-width="2" stroke="#FAC858" fill="#FFFFFF"/>
<circle cx="555.7" cy="313.8" r="2" stroke-width="2" stroke="#FAC858" fill="#FFFFFF"/>
</g>
<g>
<path fill="none" d="M 84.3 233.8 L 162.9 215.9 L 241.4 220.8 L 320 215.6 L 398.6 158.6 L 477.1 152.2 L 555.7 153.8" stroke-width="2" stroke="#EE6666"/>
<circle cx="84.3" cy="233.8" r="2" stroke-width="2" stroke="#EE6666" fill="#FFFFFF"/>
<circle cx="162.9" cy="215.9" r="2" stroke-width="2" stroke="#EE6666" fill="#FFFFFF"/>
<circle cx="241.4" cy="220.8" r="2" stroke-width="2" stroke="#EE6666" fill="#FFFFFF"/>
<circle cx="320" cy="215.6" r="2" stroke-width="2" stroke="#EE6666" fill="#FFFFFF"/>
<circle cx="398.6" cy="158.6" r="2" stroke-width="2" stroke="#EE6666" fill="#FFFFFF"/>
<circle cx="477.1" cy="152.2" r="2" stroke-width="2" stroke="#EE6666" fill="#FFFFFF"/>
<circle cx="555.7" cy="153.8" r="2" stroke-width="2" stroke="#EE6666" fill="#FFFFFF"/>
</g>
</svg>"###,
            line_chart.svg().unwrap()
        );
    }
}
