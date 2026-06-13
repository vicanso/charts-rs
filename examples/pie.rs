// Nightingale (rose) pie chart, written to `pie.svg`.
//
// Run from the repository root:
//   cargo run --example pie

use charts_rs::PieChart;

fn main() {
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
    pie_chart.title_text = "Nightingale Chart".to_string();
    pie_chart.sub_title_text = "Fake Data".to_string();
    // "{a}" series name, "{d}" percentage — see the README for all placeholders
    pie_chart.series_label_formatter = "{a}: {d}".to_string();

    std::fs::write("pie.svg", pie_chart.svg().unwrap()).unwrap();
    println!("wrote pie.svg");
}
