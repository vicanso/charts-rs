// Polar bar charts, written to `polar_bar.svg` and `polar_bar_radial.svg`.
//
// Run with:
//   cargo run --example polar_bar

use charts_rs::{PolarAxis, PolarBarChart, Series};

fn main() {
    let months = [
        "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
    ];

    // Categories around the circle: the bars grow outwards, and the series
    // that share a stack name pile up.
    let mut north: Series = (
        "North",
        vec![
            42.0, 38.0, 51.0, 64.0, 88.0, 120.0, 148.0, 136.0, 94.0, 70.0, 52.0, 45.0,
        ],
    )
        .into();
    let mut south: Series = (
        "South",
        vec![
            30.0, 34.0, 40.0, 58.0, 76.0, 98.0, 110.0, 104.0, 82.0, 60.0, 41.0, 33.0,
        ],
    )
        .into();
    for series in [&mut north, &mut south] {
        series.stack = Some("total".to_string());
    }
    let mut chart = PolarBarChart::new(
        vec![north, south],
        months.iter().map(|m| m.to_string()).collect(),
    );
    chart.title.text = "Rainfall by month".to_string();
    chart.inner_radius = Some(30.0);
    chart.y_axis_configs[0].formatter = Some("{c} mm".to_string());
    std::fs::write("polar_bar.svg", chart.svg().unwrap()).unwrap();
    println!("wrote polar_bar.svg");

    // Categories from the center outwards: one ring each, and the bars run
    // around the circle.
    let mut done: Series = ("Done", vec![92.0, 74.0, 61.0, 45.0, 28.0]).into();
    done.label_show = true;
    let mut chart = PolarBarChart::new(
        vec![done],
        ["Sleep", "Steps", "Water", "Reading", "Workout"]
            .iter()
            .map(|c| c.to_string())
            .collect(),
    );
    chart.title.text = "Goals reached".to_string();
    chart.legend.show = Some(false);
    chart.category_axis = PolarAxis::Radius;
    chart.round_cap = true;
    chart.y_axis_configs[0].max = Some(100.0);
    chart.y_axis_configs[0].split_number = 4;
    chart.y_axis_configs[0].formatter = Some("{c}%".to_string());
    std::fs::write("polar_bar_radial.svg", chart.svg().unwrap()).unwrap();
    println!("wrote polar_bar_radial.svg");
}
