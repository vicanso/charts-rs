// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use super::base::{overlay_texts, reserved_margin, with_plot_edges};
use super::canvas;
use super::common::TextConfig;
use super::component::Rect;
use super::component::generate_svg;
use super::font::DEFAULT_FONT_FAMILY;
use super::params::{
    get_bool_from_value, get_color_from_value, get_f32_from_value, get_margin_from_value,
    get_texts_from_value,
};
use super::theme::{get_default_theme_name, get_theme};
use super::{
    BarChart, BoxPlotChart, BulletChart, CalendarChart, CandlestickChart, ChordChart, FunnelChart,
    GanttChart, GaugeChart, GraphChart, HeatmapChart, HistogramChart, HorizontalBarChart,
    LineChart, LinesChart, MapChart, ParallelChart, PieChart, PolarBarChart, RadarChart,
    SankeyChart, ScatterChart, SunburstChart, TableChart, ThemeRiverChart, TreeChart, TreemapChart,
    WaterfallChart,
};
use super::{Box, Color};

/// A chart embedded in a [`MultiChart`], with an optional explicit `(x, y)`
/// position; `None` stacks it below the previous chart.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub enum ChildChart {
    /// A bar chart child.
    Bar(BarChart, Option<(f32, f32)>),
    /// A candlestick chart child.
    Candlestick(CandlestickChart, Option<(f32, f32)>),
    /// A horizontal bar chart child.
    HorizontalBar(HorizontalBarChart, Option<(f32, f32)>),
    /// A line chart child.
    Line(LineChart, Option<(f32, f32)>),
    /// A pie chart child.
    Pie(PieChart, Option<(f32, f32)>),
    /// A radar chart child.
    Radar(RadarChart, Option<(f32, f32)>),
    /// A scatter chart child.
    Scatter(ScatterChart, Option<(f32, f32)>),
    /// A table chart child.
    Table(TableChart, Option<(f32, f32)>),
    /// A heatmap chart child.
    Heatmap(HeatmapChart, Option<(f32, f32)>),
    /// A funnel chart child.
    Funnel(FunnelChart, Option<(f32, f32)>),
    /// A waterfall chart child.
    Waterfall(WaterfallChart, Option<(f32, f32)>),
    /// A calendar chart child.
    Calendar(CalendarChart, Option<(f32, f32)>),
    /// A gauge chart child.
    Gauge(GaugeChart, Option<(f32, f32)>),
    /// A treemap chart child.
    Treemap(TreemapChart, Option<(f32, f32)>),
    /// A box plot chart child.
    BoxPlot(BoxPlotChart, Option<(f32, f32)>),
    /// A sunburst chart child.
    Sunburst(SunburstChart, Option<(f32, f32)>),
    /// A sankey chart child.
    Sankey(SankeyChart, Option<(f32, f32)>),
    /// A tree chart child.
    Tree(TreeChart, Option<(f32, f32)>),
    /// A graph chart child.
    Graph(GraphChart, Option<(f32, f32)>),
    /// A parallel coordinates chart child.
    Parallel(ParallelChart, Option<(f32, f32)>),
    /// A theme river chart child.
    ThemeRiver(ThemeRiverChart, Option<(f32, f32)>),
    /// A histogram chart child.
    Histogram(HistogramChart, Option<(f32, f32)>),
    /// A polar bar chart child.
    PolarBar(PolarBarChart, Option<(f32, f32)>),
    /// A chord chart child.
    Chord(ChordChart, Option<(f32, f32)>),
    /// A gantt chart child.
    Gantt(GanttChart, Option<(f32, f32)>),
    /// A map chart child.
    Map(MapChart, Option<(f32, f32)>),
    /// A lines chart child.
    Lines(LinesChart, Option<(f32, f32)>),
    /// A bullet chart child.
    Bullet(BulletChart, Option<(f32, f32)>),
}
/// Several charts composed into one SVG: children are stacked vertically
/// with `gap` between them, or placed at an explicit position.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct MultiChart {
    /// The child charts, rendered in order.
    pub charts: Vec<ChildChart>,
    /// Vertical gap between auto-positioned children.
    pub gap: f32,
    /// Margin around the composed chart.
    pub margin: Box,
    /// Background color of the composed chart.
    pub background_color: Option<Color>,
    /// Emit compact SVG for the composition and every child; see
    /// [`compact_svg`](crate::compact_svg).
    pub compact: bool,
    /// Texts written over the composed chart: a copyright line, a
    /// watermark. A child chart has `texts` of its own, over itself.
    /// The room a text reserves is around the margin: the composition is
    /// larger by it, and the children start inside of it — but for those
    /// placed by hand, which stay where they are put.
    pub texts: Vec<TextConfig>,
    /// Font family of the texts; the default font where it is empty.
    pub font_family: String,
    /// Lines up the plots of the children one above the other: they start
    /// where the one that starts furthest from the left starts, and end
    /// where the one that ends furthest from the right ends, whatever their
    /// axes take — so that a candlestick chart and the bars of its volume
    /// below it share their x positions. It is done by `grid.left` and
    /// `grid.right` of the children: those of bar, line, candlestick,
    /// histogram, scatter, box plot and waterfall charts, which are not
    /// placed by hand.
    pub align_plots: bool,
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
        super::schema::validate(&value, &[super::schema::MULTI_FIELDS])?;
        // The top-level theme is inherited by every child that does not set
        // its own.
        let theme = value.get("theme").cloned();
        let mut multi_chart = MultiChart::new();
        if let Some(margin) = get_margin_from_value(&value, "margin") {
            multi_chart.margin = margin;
        }
        if let Some(gap) = get_f32_from_value(&value, "gap") {
            multi_chart.gap = gap;
        }
        if let Some(background_color) = get_color_from_value(&value, "background_color") {
            multi_chart.background_color = Some(background_color);
        }
        if let Some(compact) = get_bool_from_value(&value, "compact") {
            multi_chart.compact = compact;
        }
        if let Some(align) = get_bool_from_value(&value, "align_plots") {
            multi_chart.align_plots = align;
        }
        // The texts are written in the font of the theme, as those of a
        // chart are, and in the color of its sub-title.
        if let Some(mut texts) = get_texts_from_value(&value, "texts") {
            let name = theme.as_ref().and_then(|t| t.as_str()).unwrap_or_default();
            let theme = get_theme(name);
            for text in texts.iter_mut() {
                if text.font.color.is_zero() {
                    text.font.color = theme.sub_title.font.color;
                }
            }
            multi_chart.font_family = theme.font_family.clone();
            multi_chart.texts = texts;
        }
        if let Some(child_charts) = value.get("child_charts")
            && let Some(values) = child_charts.as_array()
        {
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

                // Inject the inherited theme on the parsed value rather than
                // by string splicing, which produced invalid JSON when the
                // top level had no theme either.
                let mut item = item.clone();
                if let Some(obj) = item.as_object_mut() {
                    // The placement keys belong to the multi chart, not to
                    // the child, whose own validation would reject them.
                    for field in super::schema::CHILD_CHART_FIELDS {
                        obj.remove(field.name);
                    }
                    if let Some(theme) = &theme
                        && !obj.contains_key("theme")
                    {
                        obj.insert("theme".to_string(), theme.clone());
                    }
                }
                // item is a value parsed from json, so serialization will
                // not fail in practice; propagate as an error just in case.
                let str = serde_json::to_string(&item)?;
                macro_rules! child {
                    ($chart:ty, $variant:ident) => {{
                        let chart = <$chart>::from_json(&str)?;
                        multi_chart.add(ChildChart::$variant(chart, position));
                    }};
                }
                match chart_type {
                    "" | "bar" => child!(BarChart, Bar),
                    "line" => child!(LineChart, Line),
                    "horizontal_bar" => child!(HorizontalBarChart, HorizontalBar),
                    "pie" => child!(PieChart, Pie),
                    "radar" => child!(RadarChart, Radar),
                    "table" => child!(TableChart, Table),
                    "scatter" => child!(ScatterChart, Scatter),
                    "candlestick" => child!(CandlestickChart, Candlestick),
                    "heatmap" => child!(HeatmapChart, Heatmap),
                    "funnel" => child!(FunnelChart, Funnel),
                    "waterfall" => child!(WaterfallChart, Waterfall),
                    "calendar" => child!(CalendarChart, Calendar),
                    "gauge" => child!(GaugeChart, Gauge),
                    "treemap" => child!(TreemapChart, Treemap),
                    "box_plot" => child!(BoxPlotChart, BoxPlot),
                    "sunburst" => child!(SunburstChart, Sunburst),
                    "sankey" => child!(SankeyChart, Sankey),
                    "tree" => child!(TreeChart, Tree),
                    "graph" => child!(GraphChart, Graph),
                    "parallel" => child!(ParallelChart, Parallel),
                    "theme_river" => child!(ThemeRiverChart, ThemeRiver),
                    "histogram" => child!(HistogramChart, Histogram),
                    "polar_bar" => child!(PolarBarChart, PolarBar),
                    "chord" => child!(ChordChart, Chord),
                    "gantt" => child!(GanttChart, Gantt),
                    "map" => child!(MapChart, Map),
                    "lines" => child!(LinesChart, Lines),
                    "bullet" => child!(BulletChart, Bullet),
                    other => {
                        return Err(canvas::Error::Params {
                            message: format!("unsupported child chart type: {other}"),
                        });
                    }
                };
            }
        }
        Ok(multi_chart)
    }
    /// Creates a multi chart.
    pub fn new() -> MultiChart {
        MultiChart {
            charts: vec![],
            gap: 10.0,
            margin: (10.0).into(),
            ..Default::default()
        }
    }
    /// Adds a child chart to multi chart.
    pub fn add(&mut self, c: ChildChart) {
        self.charts.push(c);
    }
    /// Converts the chart to svg.
    pub fn svg(&self) -> canvas::Result<String> {
        let mut arr = vec![];
        let mut y = 0.0;
        let mut x = 0.0;
        let font_family = if self.font_family.is_empty() {
            DEFAULT_FONT_FAMILY
        } else {
            &self.font_family
        };
        // The room the texts reserve is around the margin: the children
        // that are not placed by hand start inside of it.
        let reserved = reserved_margin(&self.texts, font_family);
        let margin_top = self.margin.top + reserved.top;
        let margin_left = self.margin.left + reserved.left;
        // Where the plots of the children are to be when they line up: from
        // the left edge that is furthest in to the right edge that is.
        macro_rules! measure {
            ($chart:expr) => {{
                let x = $chart.x.max(margin_left);
                let (_, edges) = with_plot_edges(|| $chart.svg());
                // A child that wants its plot further in still has it there.
                edges.map(|(left, right)| {
                    let left = left.max($chart.grid.left.unwrap_or_default());
                    let right = right.max($chart.grid.right.unwrap_or_default());
                    (x + left, x + $chart.width - right)
                })
            }};
        }
        let frame = if self.align_plots {
            let mut frame: Option<(f32, f32)> = None;
            for item in self.charts.iter() {
                let edges = match item {
                    ChildChart::Bar(c, None) => measure!(c),
                    ChildChart::Line(c, None) => measure!(c),
                    ChildChart::Candlestick(c, None) => measure!(c),
                    ChildChart::Histogram(c, None) => measure!(c),
                    ChildChart::Scatter(c, None) => measure!(c),
                    ChildChart::BoxPlot(c, None) => measure!(c),
                    ChildChart::Waterfall(c, None) => measure!(c),
                    _ => None,
                };
                if let Some((left, right)) = edges {
                    frame = Some(frame.map_or((left, right), |(l, r)| (l.max(left), r.min(right))));
                }
            }
            frame
        } else {
            None
        };
        // Places a child chart on a local clone: an explicit position wins,
        // otherwise charts stack vertically with `gap` between them.
        macro_rules! place_child {
            ($chart:expr, $position:expr) => {{
                let mut c = $chart.clone();
                c.y = y;
                // fixed position, no need for gap
                if let Some((px, py)) = $position {
                    c.y = *py;
                    c.x = *px;
                } else if y == 0.0 {
                    c.y = margin_top;
                } else {
                    // not the first chart and not set position
                    y += self.gap;
                    c.y = y;
                }
                if $position.is_none() {
                    c.x = c.x.max(margin_left);
                    if let Some((left, right)) = frame {
                        c.grid.left = Some(left - c.x);
                        c.grid.right = Some(c.x + c.width - right);
                    }
                }
                c
            }};
        }
        macro_rules! render {
            ($chart:expr, $position:expr) => {{
                let c = place_child!($chart, $position);
                ChildChartResult {
                    svg: c.svg()?,
                    right: c.x + c.width,
                    bottom: c.y + c.height,
                }
            }};
        }
        for item in self.charts.iter() {
            let result = match item {
                ChildChart::Bar(c, position) => render!(c, position),
                ChildChart::Candlestick(c, position) => render!(c, position),
                ChildChart::HorizontalBar(c, position) => render!(c, position),
                ChildChart::Line(c, position) => render!(c, position),
                ChildChart::Pie(c, position) => render!(c, position),
                ChildChart::Radar(c, position) => render!(c, position),
                ChildChart::Scatter(c, position) => render!(c, position),
                ChildChart::Heatmap(c, position) => render!(c, position),
                ChildChart::Funnel(c, position) => render!(c, position),
                ChildChart::Waterfall(c, position) => render!(c, position),
                ChildChart::Calendar(c, position) => render!(c, position),
                ChildChart::Gauge(c, position) => render!(c, position),
                ChildChart::Treemap(c, position) => render!(c, position),
                ChildChart::BoxPlot(c, position) => render!(c, position),
                ChildChart::Sunburst(c, position) => render!(c, position),
                ChildChart::Sankey(c, position) => render!(c, position),
                ChildChart::Tree(c, position) => render!(c, position),
                ChildChart::Graph(c, position) => render!(c, position),
                ChildChart::Parallel(c, position) => render!(c, position),
                ChildChart::ThemeRiver(c, position) => render!(c, position),
                ChildChart::Histogram(c, position) => render!(c, position),
                ChildChart::PolarBar(c, position) => render!(c, position),
                ChildChart::Chord(c, position) => render!(c, position),
                ChildChart::Gantt(c, position) => render!(c, position),
                ChildChart::Map(c, position) => render!(c, position),
                ChildChart::Lines(c, position) => render!(c, position),
                ChildChart::Bullet(c, position) => render!(c, position),
                ChildChart::Table(c, position) => {
                    let c = place_child!(c, position);
                    // the height is recomputed by the table itself
                    let (svg, height) = c.render()?;
                    ChildChartResult {
                        svg,
                        right: c.x + c.width,
                        bottom: c.y + height,
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
        x += self.margin.right + reserved.right;
        y += self.margin.bottom + reserved.bottom;

        if let Some(background_color) = self.background_color {
            arr.insert(
                0,
                Rect {
                    fill: Some(background_color.into()),
                    left: 0.0,
                    top: 0.0,
                    width: x,
                    height: y,
                    ..Default::default()
                }
                .svg(),
            );
        }

        // The texts go over the children, placed on the whole picture.
        let color = get_theme(&get_default_theme_name()).sub_title.font.color;
        for text in overlay_texts(&self.texts, font_family, color) {
            if let Some(text) = text.text(x, y) {
                arr.push(text.svg());
            }
        }

        let svg = generate_svg(x, y, 0.0, 0.0, arr.join("\n"));
        if self.compact {
            return Ok(super::compact::compact_svg(&svg));
        }
        Ok(svg)
    }
}

#[cfg(test)]
mod tests {
    use super::{ChildChart, MultiChart};
    use crate::{
        BarChart, CandlestickChart, HorizontalBarChart, LineChart, PieChart, RadarChart,
        ScatterChart, TableChart,
    };
    #[test]
    fn multi_chart() {
        let mut charts = MultiChart::new();
        charts.margin = (10.0).into();
        charts.background_color = Some((31, 29, 29, 150).into());

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
            vec![
                (
                    "",
                    vec![
                        20.0, 34.0, 10.0, 38.0, 40.0, 35.0, 30.0, 50.0, 31.0, 38.0, 33.0, 44.0,
                        38.0, 15.0, 5.0, 42.0,
                    ],
                )
                    .into(),
            ],
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

        assert_snapshot!("multi_chart/basic.svg", charts.svg().unwrap());
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

        assert_snapshot!("multi_chart/override.svg", charts.svg().unwrap());
    }
}
