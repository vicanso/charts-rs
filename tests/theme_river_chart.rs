mod common;

use charts_rs::ThemeRiverChart;

#[test]
fn theme_river_chart_json() {
    let chart =
        ThemeRiverChart::from_json(include_str!("../asset/theme_river_chart/integration.json"))
            .unwrap();
    common::assert_snapshot!(
        "theme_river_chart/integration_json.svg",
        chart.svg().unwrap()
    );
}
