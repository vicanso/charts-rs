// Histogram of a sample, written to `histogram.svg`.
//
// Run from the repository root:
//   cargo run --example histogram

use charts_rs::HistogramChart;

fn main() {
    // The series data is the sample itself: one value per observation.
    let heights: Vec<f32> = (0..200)
        .map(|i| {
            let a = ((i * 37) % 101) as f32 / 100.0;
            let b = ((i * 61) % 89) as f32 / 88.0;
            let c = ((i * 17) % 53) as f32 / 52.0;
            150.0 + (a + b + c) / 3.0 * 40.0
        })
        .collect();
    let mut histogram = HistogramChart::new(vec![("Adults", heights).into()]);
    histogram.title_text = "Height distribution".to_string();
    histogram.x_axis_title = "Height (cm)".to_string();
    histogram.y_axis_configs[0].axis_title = Some("People".to_string());
    histogram.series_list[0].label_show = true;
    // Bins are chosen from the sample by default; fix their width instead.
    histogram.bin_width = Some(5.0);

    std::fs::write("histogram.svg", histogram.svg().unwrap()).unwrap();
    println!("wrote histogram.svg");
}
