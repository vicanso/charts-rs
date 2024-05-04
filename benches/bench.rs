use charts_rs::{
    measure_text_width_family, svg_to_png, BarChart, Box, LegendCategory, SeriesCategory,
    DEFAULT_FONT_FAMILY,
};
use criterion::{criterion_group, criterion_main, Criterion};

fn measure_text_benchmark(c: &mut Criterion) {
    c.bench_function("measure test", |b| {
        b.iter(|| measure_text_width_family(DEFAULT_FONT_FAMILY, 14.0, "Hello World!").unwrap())
    });
}

fn bar_chart_line_mixin() {
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
    bar_chart.svg().unwrap();
}

fn bar_chart_line_mixin_png() {
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

    svg_to_png(&bar_chart.svg().unwrap()).unwrap();
}

fn bar_chart_benchmark(c: &mut Criterion) {
    c.bench_function("bar chart test", |b| b.iter(bar_chart_line_mixin));
}

fn bar_chart_png_benchmark(c: &mut Criterion) {
    c.bench_function("bar chart png test", |b| b.iter(bar_chart_line_mixin_png));
}

criterion_group!(
    benches,
    measure_text_benchmark,
    bar_chart_benchmark,
    bar_chart_png_benchmark,
);
criterion_main!(benches);
