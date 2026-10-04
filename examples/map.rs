// Map of regions colored by their values, written to `map.svg`.
//
// Run with:
//   cargo run --example map

use charts_rs::{MapChart, MapRegion};

fn main() {
    // The regions are GeoJSON: the library has no map data of its own. This
    // one is a made-up country of eleven regions.
    let geo_json = include_str!("../asset/map_chart/regions.geo.json");
    let regions = MapRegion::from_geo_json(geo_json, "name").unwrap();

    // The value of a region, by its name; `Kelp` has none.
    let data = [
        ("Alder", 412.0),
        ("Birch", 268.0),
        ("Cedar", 530.0),
        ("Dune", 145.0),
        ("Elm", 96.0),
        ("Fir", 325.0),
        ("Glen", 610.0),
        ("Heath", 204.0),
        ("Ivy", 78.0),
        ("Juniper", 356.0),
    ]
    .into_iter()
    .map(|(name, value)| (name.to_string(), value))
    .collect();

    let mut map = MapChart::new(regions, data);
    map.title_text = "Population by region".to_string();
    map.label_show = true;
    map.tooltip_show = true;
    // Five classes, each in a color of its own.
    map.thresholds = vec![100.0, 250.0, 400.0, 550.0];
    map.colors = ["#eff3ff", "#bdd7e7", "#6baed6", "#3182bd", "#08519c"]
        .iter()
        .map(|c| (*c).into())
        .collect();

    std::fs::write("map.svg", map.svg().unwrap()).unwrap();
    println!("wrote map.svg");
}
