use charts_rs::SunburstChart;
use pretty_assertions::assert_eq;

const DATA: &str = r##"
    "inner_radius": 20,
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
"##;

#[test]
fn sunburst_chart_basic_json() {
    let chart =
        SunburstChart::from_json(&format!(r##"{{"title_text": "Sunburst",{DATA}}}"##)).unwrap();
    assert_eq!(
        include_str!("../asset/sunburst_chart/basic_json.svg"),
        chart.svg().unwrap()
    );
}

#[test]
fn sunburst_chart_grafana_json() {
    let chart = SunburstChart::from_json(&format!(
        r##"{{"theme": "grafana", "title_text": "Sunburst",{DATA}}}"##
    ))
    .unwrap();
    assert_eq!(
        include_str!("../asset/sunburst_chart/grafana_json.svg"),
        chart.svg().unwrap()
    );
}
