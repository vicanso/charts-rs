// Sankey (flow) chart built with the builder API, written to `sankey.svg`.
// Nodes are derived automatically from the link endpoints.
//
// Run from the repository root:
//   cargo run --example sankey

use charts_rs::{AnimationConfig, SankeyChart, SankeyLink};

fn main() {
    // Fuels fan out through two energy carriers and back into three sectors,
    // so the ribbons branch, cross and vary noticeably in width.
    let links: Vec<SankeyLink> = vec![
        ("Coal", "Electricity", 25.0).into(),
        ("Coal", "Heat", 10.0).into(),
        ("Gas", "Electricity", 15.0).into(),
        ("Gas", "Heat", 20.0).into(),
        ("Solar", "Electricity", 10.0).into(),
        ("Electricity", "Residential", 18.0).into(),
        ("Electricity", "Industrial", 22.0).into(),
        ("Electricity", "Commercial", 10.0).into(),
        ("Heat", "Residential", 12.0).into(),
        ("Heat", "Industrial", 18.0).into(),
    ];

    let mut sankey_chart = SankeyChart::new(vec![], links);
    sankey_chart.title_text = "Energy Flow".to_string();
    // "{b}" node name, "{c}" throughput — see the README for all placeholders
    sankey_chart.series_label_formatter = "{b} ({c})".to_string();
    // Fill each ribbon with a source -> target color gradient.
    sankey_chart.link_gradient = true;
    // Flow-grow: columns expand left to right, 200ms apart; labels fade in.
    sankey_chart.animation = Some(AnimationConfig {
        duration: 1000,
        easing: "ease".to_string(),
        delay: 200,
    });

    std::fs::write("sankey.svg", sankey_chart.svg().unwrap()).unwrap();
    println!("wrote sankey.svg");
}
