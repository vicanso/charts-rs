use charts_rs::{measure_text_width_family, DEFAULT_FONT_FAMILY};
use criterion::{criterion_group, criterion_main, Criterion};

fn measure_text_benchmark(c: &mut Criterion) {
    c.bench_function("measure test", |b| {
        b.iter(|| measure_text_width_family(DEFAULT_FONT_FAMILY, 14.0, "Hello World!").unwrap())
    });
}

criterion_group!(benches, measure_text_benchmark);
criterion_main!(benches);
