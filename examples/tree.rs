// Tree (node-link hierarchy) chart with curved links, written to `tree.svg`.
// Reuses the same {name, children, value} model as the sunburst chart.
//
// Run from the repository root:
//   cargo run --example tree

use charts_rs::TreeChart;

fn main() {
    let tree_chart = TreeChart::from_json(
        r##"{
            "title_text": "Org Tree",
            "orient": "LR",
            "series_data": [
                {
                    "name": "Company",
                    "children": [
                        {"name": "Engineering", "children": [
                            {"name": "Frontend"},
                            {"name": "Backend"},
                            {"name": "Mobile"}
                        ]},
                        {"name": "Design", "children": [
                            {"name": "UX"},
                            {"name": "Visual"}
                        ]},
                        {"name": "Operations", "children": [
                            {"name": "Infra"},
                            {"name": "Support"}
                        ]}
                    ]
                }
            ]
        }"##,
    )
    .unwrap();

    std::fs::write("tree.svg", tree_chart.svg().unwrap()).unwrap();
    println!("wrote tree.svg");
}
