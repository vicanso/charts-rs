// Chord diagram of the flows between a few nodes, written to `chord.svg`.
//
// Run with:
//   cargo run --example chord

use charts_rs::{ChordChart, ChordLink};

fn main() {
    // The nodes are derived from the names in the links, in first-seen order;
    // pass explicit `ChordNode`s to fix their order or their colors.
    let links: Vec<ChordLink> = vec![
        ("Asia", "Europe", 60.0).into(),
        ("Asia", "Americas", 45.0).into(),
        ("Europe", "Americas", 50.0).into(),
        ("Europe", "Africa", 25.0).into(),
        ("Americas", "Africa", 15.0).into(),
        ("Asia", "Oceania", 20.0).into(),
        ("Oceania", "Americas", 10.0).into(),
    ];
    let mut chord = ChordChart::new(vec![], links);
    chord.title_text = "Trade between regions".to_string();
    // Name every node with its share of all the flows.
    chord.series_label_formatter = "{b} ({d})".to_string();
    // Fade every ribbon from the color of its source to that of its target.
    chord.link_gradient = true;
    chord.tooltip_show = true;

    std::fs::write("chord.svg", chord.svg().unwrap()).unwrap();
    println!("wrote chord.svg");
}
