// Basic bar chart, written to `bar.svg`.
//
// Run from the repository root:
//   cargo run --example bar

use charts_rs::BarChart;

fn main() {
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
    bar_chart.title_text = "Bar Chart".to_string();
    bar_chart.sub_title_text = "Weekly traffic".to_string();
    // show the value on top of every bar of the first series
    bar_chart.series_list[0].label_show = true;

    std::fs::write("bar.svg", bar_chart.svg().unwrap()).unwrap();
    println!("wrote bar.svg");
}
