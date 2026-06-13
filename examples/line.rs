// Line chart showcasing smooth curves, area fill and a mark line / mark point,
// written to `line.svg`.
//
// Run from the repository root:
//   cargo run --example line

use charts_rs::{LineChart, MarkLine, MarkLineCategory, MarkPoint, MarkPointCategory};

fn main() {
    let mut line_chart = LineChart::new(
        vec![
            (
                "Email",
                vec![120.0, 132.0, 101.0, 134.0, 90.0, 230.0, 210.0],
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
    line_chart.title_text = "Line Chart".to_string();
    // smooth curves with a translucent area fill under each line
    line_chart.series_smooth = true;
    line_chart.series_fill = true;
    // an average mark line and min/max mark points on the second series
    line_chart.series_list[1].mark_lines = vec![MarkLine {
        category: MarkLineCategory::Average,
    }];
    line_chart.series_list[1].mark_points = vec![
        MarkPoint {
            category: MarkPointCategory::Max,
        },
        MarkPoint {
            category: MarkPointCategory::Min,
        },
    ];

    std::fs::write("line.svg", line_chart.svg().unwrap()).unwrap();
    println!("wrote line.svg");
}
