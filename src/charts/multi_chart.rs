use super::params::get_f32_from_value;
use super::{
    canvas, component::generate_svg, BarChart, CandlestickChart, CanvasResult, HorizontalBarChart,
    LineChart, PieChart, RadarChart, ScatterChart, TableChart,
};
use substring::Substring;

pub enum ChildChart {
    Bar(BarChart, Option<(f32, f32)>),
    Candlestick(CandlestickChart, Option<(f32, f32)>),
    HorizontalBar(HorizontalBarChart, Option<(f32, f32)>),
    Line(LineChart, Option<(f32, f32)>),
    Pie(PieChart, Option<(f32, f32)>),
    Radar(RadarChart, Option<(f32, f32)>),
    Scatter(ScatterChart, Option<(f32, f32)>),
    Table(TableChart, Option<(f32, f32)>),
}
#[derive(Default)]
pub struct MultiChart {
    pub charts: Vec<ChildChart>,
    pub gap: f32,
}
struct ChildChartResult {
    svg: String,
    right: f32,
    bottom: f32,
}

impl MultiChart {
    /// Creates a multi chart from json.
    pub fn from_json(data: &str) -> canvas::Result<MultiChart> {
        let value: serde_json::Value = serde_json::from_str(data)?;
        let mut theme = "".to_string();
        if let Some(value) = value.get("theme") {
            theme = value.to_string();
        }
        let mut multi_chart = MultiChart::new();
        if let Some(child_charts) = value.get("child_charts") {
            if let Some(values) = child_charts.as_array() {
                for item in values.iter() {
                    let chart_type = if let Some(value) = item.get("type") {
                        value.as_str().unwrap_or_default()
                    } else {
                        ""
                    };
                    let mut x = 0.0;
                    let mut y = 0.0;
                    let mut exists_position = false;
                    if let Some(v) = get_f32_from_value(item, "x") {
                        x = v;
                        exists_position = true;
                    }
                    if let Some(v) = get_f32_from_value(item, "y") {
                        y = v;
                        exists_position = true;
                    }
                    let mut position = None;
                    if exists_position {
                        position = Some((x, y));
                    }

                    // 由json转换，因此不会出错
                    let mut str = serde_json::to_string(item).unwrap();
                    if item.get("theme").is_none() {
                        str = format!(
                            r###"{},"theme":{theme}}}"###,
                            str.substring(0, str.len() - 1)
                        );
                    }
                    match chart_type {
                        "line" => {
                            let chart = LineChart::from_json(&str)?;
                            multi_chart.add(ChildChart::Line(chart, position));
                        }
                        "horizontal_bar" => {
                            let chart = HorizontalBarChart::from_json(&str)?;
                            multi_chart.add(ChildChart::HorizontalBar(chart, position));
                        }
                        "pie" => {
                            let chart = PieChart::from_json(&str)?;
                            multi_chart.add(ChildChart::Pie(chart, position));
                        }
                        "radar" => {
                            let chart = RadarChart::from_json(&str)?;
                            multi_chart.add(ChildChart::Radar(chart, position));
                        }
                        "table" => {
                            let chart = TableChart::from_json(&str)?;
                            multi_chart.add(ChildChart::Table(chart, position));
                        }
                        "scatter" => {
                            let chart = ScatterChart::from_json(&str)?;
                            multi_chart.add(ChildChart::Scatter(chart, position));
                        }
                        "candlestick" => {
                            let chart = CandlestickChart::from_json(&str)?;
                            multi_chart.add(ChildChart::Candlestick(chart, position));
                        }
                        _ => {
                            let chart = BarChart::from_json(&str)?;
                            multi_chart.add(ChildChart::Bar(chart, position));
                        }
                    };
                }
            }
        }
        Ok(multi_chart)
    }
    /// Creates a multi chart.
    pub fn new() -> MultiChart {
        MultiChart {
            charts: vec![],
            gap: 10.0,
        }
    }
    /// Adds a child chart to multi chart.
    pub fn add(&mut self, c: ChildChart) {
        self.charts.push(c);
    }
    /// Converts the chart to svg.
    pub fn svg(&mut self) -> CanvasResult<String> {
        let mut arr = vec![];
        let mut y = 0.0;
        let mut x = 0.0;
        for item in self.charts.iter_mut() {
            let result = match item {
                ChildChart::Bar(c, postion) => {
                    c.y = y;
                    // 指定定位的不增加gap
                    if let Some((x, y)) = postion {
                        c.y = y.to_owned();
                        c.x = x.to_owned();
                    } else if y > 0.0 {
                        // 非首个图，而且未设置定位
                        y += self.gap;
                        c.y = y;
                    }

                    ChildChartResult {
                        svg: c.svg()?,
                        right: c.x + c.width,
                        bottom: c.y + c.height,
                    }
                }
                ChildChart::Candlestick(c, postion) => {
                    c.y = y;
                    // 指定定位的不增加gap
                    if let Some((x, y)) = postion {
                        c.y = y.to_owned();
                        c.x = x.to_owned();
                    } else if y > 0.0 {
                        // 非首个图，而且未设置定位
                        y += self.gap;
                        c.y = y;
                    }

                    ChildChartResult {
                        svg: c.svg()?,
                        right: c.x + c.width,
                        bottom: c.y + c.height,
                    }
                }
                ChildChart::HorizontalBar(c, postion) => {
                    c.y = y;
                    // 指定定位的不增加gap
                    if let Some((x, y)) = postion {
                        c.y = y.to_owned();
                        c.x = x.to_owned();
                    } else if y > 0.0 {
                        // 非首个图，而且未设置定位
                        y += self.gap;
                        c.y = y;
                    }

                    ChildChartResult {
                        svg: c.svg()?,
                        right: c.x + c.width,
                        bottom: c.y + c.height,
                    }
                }
                ChildChart::Line(c, postion) => {
                    c.y = y;
                    // 指定定位的不增加gap
                    if let Some((x, y)) = postion {
                        c.y = y.to_owned();
                        c.x = x.to_owned();
                    } else if y > 0.0 {
                        // 非首个图，而且未设置定位
                        y += self.gap;
                        c.y = y;
                    }

                    ChildChartResult {
                        svg: c.svg()?,
                        right: c.x + c.width,
                        bottom: c.y + c.height,
                    }
                }
                ChildChart::Pie(c, postion) => {
                    c.y = y;
                    // 指定定位的不增加gap
                    if let Some((x, y)) = postion {
                        c.y = y.to_owned();
                        c.x = x.to_owned();
                    } else if y > 0.0 {
                        // 非首个图，而且未设置定位
                        y += self.gap;
                        c.y = y;
                    }

                    ChildChartResult {
                        svg: c.svg()?,
                        right: c.x + c.width,
                        bottom: c.y + c.height,
                    }
                }
                ChildChart::Radar(c, postion) => {
                    c.y = y;
                    // 指定定位的不增加gap
                    if let Some((x, y)) = postion {
                        c.y = y.to_owned();
                        c.x = x.to_owned();
                    } else if y > 0.0 {
                        // 非首个图，而且未设置定位
                        y += self.gap;
                        c.y = y;
                    }

                    ChildChartResult {
                        svg: c.svg()?,
                        right: c.x + c.width,
                        bottom: c.y + c.height,
                    }
                }
                ChildChart::Scatter(c, postion) => {
                    c.y = y;
                    // 指定定位的不增加gap
                    if let Some((x, y)) = postion {
                        c.y = y.to_owned();
                        c.x = x.to_owned();
                    } else if y > 0.0 {
                        // 非首个图，而且未设置定位
                        y += self.gap;
                        c.y = y;
                    }

                    ChildChartResult {
                        svg: c.svg()?,
                        right: c.x + c.width,
                        bottom: c.y + c.height,
                    }
                }
                ChildChart::Table(c, postion) => {
                    c.y = y;
                    // 指定定位的不增加gap
                    if let Some((x, y)) = postion {
                        c.y = y.to_owned();
                        c.x = x.to_owned();
                    } else if y > 0.0 {
                        // 非首个图，而且未设置定位
                        y += self.gap;
                        c.y = y;
                    }
                    // svg中会重新计算c.height
                    let svg = c.svg()?;
                    ChildChartResult {
                        svg,
                        right: c.x + c.width,
                        bottom: c.y + c.height,
                    }
                }
            };
            if result.bottom > y {
                y = result.bottom;
            }
            if result.right > x {
                x = result.right;
            }
            arr.push(result.svg);
        }

        Ok(generate_svg(x, y, 0.0, 0.0, arr.join("\n")))
    }
}

#[cfg(test)]
mod tests {
    use super::{ChildChart, MultiChart};
    use crate::{
        BarChart, CandlestickChart, HorizontalBarChart, LineChart, PieChart, RadarChart,
        ScatterChart, TableChart,
    };
    use pretty_assertions::assert_eq;
    #[test]
    fn multi_chart() {
        let mut charts = MultiChart::new();
        let bar_chart = BarChart::new(
            vec![
                (
                    "Email",
                    vec![120.0, 132.0, 101.0, 134.0, 90.0, 230.0, 210.0],
                )
                    .into(),
                (
                    "Union Ads",
                    vec![220.0, 182.0, 191.0, 234.0, 290.0, 330.0, 310.0],
                )
                    .into(),
                (
                    "Direct",
                    vec![320.0, 332.0, 301.0, 334.0, 390.0, 330.0, 320.0],
                )
                    .into(),
                (
                    "Search Engine",
                    vec![820.0, 932.0, 901.0, 934.0, 1290.0, 1330.0, 1320.0],
                )
                    .into(),
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
        charts.add(ChildChart::Bar(bar_chart, None));

        let candlestick_chart = CandlestickChart::new(
            vec![(
                "",
                vec![
                    20.0, 34.0, 10.0, 38.0, 40.0, 35.0, 30.0, 50.0, 31.0, 38.0, 33.0, 44.0, 38.0,
                    15.0, 5.0, 42.0,
                ],
            )
                .into()],
            vec![
                "2017-10-24".to_string(),
                "2017-10-25".to_string(),
                "2017-10-26".to_string(),
                "2017-10-27".to_string(),
            ],
        );
        charts.add(ChildChart::Candlestick(candlestick_chart, None));

        let horizontal_bar_chart = HorizontalBarChart::new(
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
        charts.add(ChildChart::HorizontalBar(horizontal_bar_chart, None));

        let line_chart = LineChart::new(
            vec![
                (
                    "Email",
                    vec![120.0, 132.0, 101.0, 134.0, 90.0, 230.0, 210.0],
                )
                    .into(),
                (
                    "Union Ads",
                    vec![220.0, 182.0, 191.0, 234.0, 290.0, 330.0, 310.0],
                )
                    .into(),
                (
                    "Direct",
                    vec![320.0, 332.0, 301.0, 334.0, 390.0, 330.0, 320.0],
                )
                    .into(),
                (
                    "Search Engine",
                    vec![820.0, 932.0, 901.0, 934.0, 1290.0, 1330.0, 1320.0],
                )
                    .into(),
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
        charts.add(ChildChart::Line(line_chart, None));

        let pie_chart = PieChart::new(vec![
            ("rose 1", vec![40.0]).into(),
            ("rose 2", vec![38.0]).into(),
            ("rose 3", vec![32.0]).into(),
            ("rose 4", vec![30.0]).into(),
            ("rose 5", vec![28.0]).into(),
            ("rose 6", vec![26.0]).into(),
            ("rose 7", vec![22.0]).into(),
            ("rose 8", vec![18.0]).into(),
        ]);

        charts.add(ChildChart::Pie(pie_chart, None));

        let radar_chart = RadarChart::new(
            vec![
                (
                    "Allocated Budget",
                    vec![4200.0, 3000.0, 20000.0, 35000.0, 50000.0, 18000.0],
                )
                    .into(),
                (
                    "Actual Spending",
                    vec![5000.0, 14000.0, 28000.0, 26000.0, 42000.0, 21000.0],
                )
                    .into(),
            ],
            vec![
                ("Sales", 6500.0).into(),
                ("Administration", 16000.0).into(),
                ("Information Technology", 30000.0).into(),
                ("Customer Support", 38000.0).into(),
                ("Development", 52000.0).into(),
                ("Marketing", 25000.0).into(),
            ],
        );
        charts.add(ChildChart::Radar(radar_chart, None));

        let scatter_chart = ScatterChart::new(vec![
            (
                "Female",
                vec![
                    161.2, 51.6, 167.5, 59.0, 159.5, 49.2, 157.0, 63.0, 155.8, 53.6, 170.0, 59.0,
                    159.1, 47.6, 166.0, 69.8, 176.2, 66.8, 160.2, 75.2, 172.5, 55.2, 170.9, 54.2,
                    172.9, 62.5, 153.4, 42.0, 160.0, 50.0, 147.2, 49.8, 168.2, 49.2, 175.0, 73.2,
                    157.0, 47.8, 167.6, 68.8, 159.5, 50.6, 175.0, 82.5, 166.8, 57.2, 176.5, 87.8,
                    170.2, 72.8,
                ],
            )
                .into(),
            (
                "Male",
                vec![
                    174.0, 65.6, 175.3, 71.8, 193.5, 80.7, 186.5, 72.6, 187.2, 78.8, 181.5, 74.8,
                    184.0, 86.4, 184.5, 78.4, 175.0, 62.0, 184.0, 81.6, 180.0, 76.6, 177.8, 83.6,
                    192.0, 90.0, 176.0, 74.6, 174.0, 71.0, 184.0, 79.6, 192.7, 93.8, 171.5, 70.0,
                    173.0, 72.4, 176.0, 85.9, 176.0, 78.8, 180.5, 77.8, 172.7, 66.2, 176.0, 86.4,
                    173.5, 81.8,
                ],
            )
                .into(),
        ]);
        charts.add(ChildChart::Scatter(scatter_chart, None));

        let table_chart = TableChart::new(vec![
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
        charts.add(ChildChart::Table(table_chart, None));

        assert_eq!(
            include_str!("../../asset/multi_chart/basic.svg"),
            charts.svg().unwrap()
        );
    }

    #[test]
    fn multi_chart_override() {
        let mut charts = MultiChart::new();
        let bar_chart = BarChart::new(
            vec![
                (
                    "Email",
                    vec![120.0, 132.0, 101.0, 134.0, 90.0, 230.0, 210.0],
                )
                    .into(),
                (
                    "Union Ads",
                    vec![220.0, 182.0, 191.0, 234.0, 290.0, 330.0, 310.0],
                )
                    .into(),
                (
                    "Direct",
                    vec![320.0, 332.0, 301.0, 334.0, 390.0, 330.0, 320.0],
                )
                    .into(),
                (
                    "Search Engine",
                    vec![820.0, 932.0, 901.0, 934.0, 1290.0, 1330.0, 1320.0],
                )
                    .into(),
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
        charts.add(ChildChart::Bar(bar_chart, None));

        let mut pie_chart = PieChart::new(vec![
            ("rose 1", vec![40.0]).into(),
            ("rose 2", vec![38.0]).into(),
            ("rose 3", vec![32.0]).into(),
            ("rose 4", vec![30.0]).into(),
            ("rose 5", vec![28.0]).into(),
            ("rose 6", vec![26.0]).into(),
            ("rose 7", vec![22.0]).into(),
            ("rose 8", vec![18.0]).into(),
        ]);
        pie_chart.width = 400.0;
        pie_chart.height = 200.0;
        pie_chart.background_color = (0, 0, 0, 0).into();

        charts.add(ChildChart::Pie(pie_chart, Some((200.0, 0.0))));

        assert_eq!(
            include_str!("../../asset/multi_chart/override.svg"),
            charts.svg().unwrap()
        );
    }
}
