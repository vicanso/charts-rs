#[cfg(feature = "png")]
use charts_rs::svg_to_png;
use charts_rs::{
    BarChart, Box, DEFAULT_FONT_FAMILY, LegendCategory, LineChart, PieChart, SankeyChart,
    SeriesCategory, TableChart, measure_text_width_family,
};
use criterion::{Criterion, criterion_group, criterion_main};
use std::hint::black_box;

fn measure_text_benchmark(c: &mut Criterion) {
    c.bench_function("measure text", |b| {
        b.iter(|| measure_text_width_family(DEFAULT_FONT_FAMILY, 14.0, "Hello World!").unwrap())
    });
}

fn make_bar_chart() -> BarChart {
    let mut bar_chart = BarChart::new(
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
    bar_chart.series_list[0].category = Some(SeriesCategory::Line);
    bar_chart.y_axis_configs[0].axis_width = Some(55.0);
    bar_chart.title_text = "Bar Chart".to_string();
    bar_chart.legend_margin = Some(Box {
        top: 35.0,
        bottom: 10.0,
        ..Default::default()
    });
    bar_chart.legend_category = LegendCategory::Rect;
    bar_chart.y_axis_configs[0].axis_formatter = Some("{c} ml".to_string());
    bar_chart.series_list[0].label_show = true;
    bar_chart.series_list[3].label_show = true;
    bar_chart
}

/// A line chart with `points` data points per series: the path is the
/// largest piece of output, so this is where allocation costs show.
fn make_line_chart(points: usize, tooltip: bool) -> LineChart {
    let data: Vec<f32> = (0..points)
        .map(|i| ((i as f32) * 0.1).sin() * 50.0 + 60.0)
        .collect();
    let categories: Vec<String> = (0..points).map(|i| format!("t{i}")).collect();
    let mut chart = LineChart::new(
        vec![("A", data.clone()).into(), ("B", data).into()],
        categories,
    );
    chart.series_smooth = true;
    chart.series_fill = true;
    chart.tooltip_show = tooltip;
    chart.title_text = "Line".to_string();
    chart
}

fn make_table_chart() -> TableChart {
    let mut rows = vec![vec![
        "Name".to_string(),
        "Description".to_string(),
        "Price".to_string(),
    ]];
    for i in 0..40 {
        rows.push(vec![
            format!("Item {i}"),
            "A fairly long description that has to be wrapped over several lines to fit"
                .to_string(),
            format!("{}.99", i * 7),
        ]);
    }
    let mut chart = TableChart::new(rows);
    chart.title_text = "Table".to_string();
    chart
}

fn make_sankey_chart() -> SankeyChart {
    SankeyChart::from_json(
        r##"{
            "nodes": [{"name": "Coal"}, {"name": "Gas"}, {"name": "Solar"}, {"name": "Electricity"},
                      {"name": "Heat"}, {"name": "Residential"}, {"name": "Industrial"}, {"name": "Commercial"}],
            "links": [
                {"source": "Coal", "target": "Electricity", "value": 25},
                {"source": "Coal", "target": "Heat", "value": 10},
                {"source": "Gas", "target": "Electricity", "value": 15},
                {"source": "Gas", "target": "Heat", "value": 20},
                {"source": "Solar", "target": "Electricity", "value": 10},
                {"source": "Electricity", "target": "Residential", "value": 18},
                {"source": "Electricity", "target": "Industrial", "value": 22},
                {"source": "Electricity", "target": "Commercial", "value": 10},
                {"source": "Heat", "target": "Residential", "value": 12},
                {"source": "Heat", "target": "Industrial", "value": 18}
            ]
        }"##,
    )
    .unwrap()
}

fn make_pie_chart() -> PieChart {
    let mut chart = PieChart::new(
        (0..12)
            .map(|i| (format!("Slice {i}").as_str(), vec![10.0 + i as f32 * 3.0]).into())
            .collect(),
    );
    chart.title_text = "Pie".to_string();
    chart
}

const BAR_JSON: &str = r##"{
    "title_text": "Bar Chart",
    "legend_category": "rect",
    "series_list": [
        {"name": "Email", "label_show": true, "data": [120, 132, 101, 134, 90, 230, 210]},
        {"name": "Union Ads", "data": [220, 182, 191, 234, 290, 330, 310]},
        {"name": "Direct", "data": [320, 332, 301, 334, 390, 330, 320]},
        {"name": "Search Engine", "data": [820, 932, 901, 934, 1290, 1330, 1320]}
    ],
    "x_axis_data": ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"],
    "y_axis_configs": [{"axis_width": 55, "axis_formatter": "{c} ml"}]
}"##;

fn svg_benchmarks(c: &mut Criterion) {
    let bar = make_bar_chart();
    c.bench_function("bar chart svg", |b| {
        b.iter(|| black_box(&bar).svg().unwrap())
    });
    let line_1k = make_line_chart(1_000, true);
    c.bench_function("line chart svg (1k points, smooth+fill+tooltip)", |b| {
        b.iter(|| black_box(&line_1k).svg().unwrap())
    });
    let line_10k = make_line_chart(10_000, false);
    c.bench_function("line chart svg (10k points, smooth+fill)", |b| {
        b.iter(|| black_box(&line_10k).svg().unwrap())
    });
    let table = make_table_chart();
    c.bench_function("table chart svg (text wrapping)", |b| {
        b.iter(|| black_box(&table).svg().unwrap())
    });
    let sankey = make_sankey_chart();
    c.bench_function("sankey chart svg (layout)", |b| {
        b.iter(|| black_box(&sankey).svg().unwrap())
    });
    let pie = make_pie_chart();
    c.bench_function("pie chart svg (paths)", |b| {
        b.iter(|| black_box(&pie).svg().unwrap())
    });
    c.bench_function("bar chart from_json", |b| {
        b.iter(|| BarChart::from_json(black_box(BAR_JSON)).unwrap())
    });
}

#[cfg(feature = "png")]
fn png_benchmark(c: &mut Criterion) {
    let svg = make_bar_chart().svg().unwrap();
    c.bench_function("bar chart png", |b| {
        b.iter(|| svg_to_png(black_box(&svg)).unwrap())
    });
}

#[cfg(feature = "png")]
criterion_group!(
    benches,
    measure_text_benchmark,
    svg_benchmarks,
    png_benchmark
);
#[cfg(not(feature = "png"))]
criterion_group!(benches, measure_text_benchmark, svg_benchmarks);
criterion_main!(benches);
