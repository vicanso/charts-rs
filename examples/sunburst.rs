// Sunburst chart built from JSON, showcasing the label formatter, per-level
// ring thickness and the expand animation. Written to `sunburst.svg`.
//
// Run from the repository root:
//   cargo run --example sunburst

use charts_rs::SunburstChart;

fn main() {
    let sunburst_chart = SunburstChart::from_json(
        r##"{
            "title_text": "Sunburst",
            "inner_radius": 20,
            "level_thickness": [1.5, 1, 1],
            "series_label_formatter": "{b}: {d}",
            "animation": {"duration": 1000, "easing": "ease", "delay": 200},
            "series_data": [
                {
                    "name": "Grandpa",
                    "children": [
                        {"name": "Uncle Leo", "children": [
                            {"name": "Cousin Jack", "value": 18},
                            {"name": "Cousin Mary", "value": 12}
                        ]},
                        {"name": "Father", "children": [
                            {"name": "Me", "value": 40},
                            {"name": "Brother Peter", "value": 20}
                        ]}
                    ]
                },
                {
                    "name": "Nancy",
                    "children": [
                        {"name": "Uncle Nike", "children": [
                            {"name": "Cousin Betty", "value": 10},
                            {"name": "Cousin Jenny", "value": 30}
                        ]}
                    ]
                }
            ]
        }"##,
    )
    .unwrap();

    std::fs::write("sunburst.svg", sunburst_chart.svg().unwrap()).unwrap();
    println!("wrote sunburst.svg");
}
