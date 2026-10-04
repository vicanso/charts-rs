mod common;

use charts_rs::SankeyChart;

#[test]
fn sankey_chart_json() {
    let chart =
        SankeyChart::from_json(include_str!("../asset/sankey_chart/integration.json")).unwrap();
    common::assert_snapshot!("sankey_chart/integration_json.svg", chart.svg().unwrap());
}

#[test]
fn sankey_from_top_to_bottom() {
    let json = |extra: &str| {
        format!(
            r##"{{"width": 400, "height": 400{extra}, "links": [
                {{"source": "a", "target": "c", "value": 6}},
                {{"source": "b", "target": "c", "value": 4}},
                {{"source": "c", "target": "d", "value": 7}},
                {{"source": "c", "target": "e", "value": 3}}
            ]}}"##
        )
    };
    let number = |tag: &str, name: &str| -> f32 {
        let key = format!(" {name}=\"");
        let start = tag.find(&key).unwrap() + key.len();
        tag[start..start + tag[start..].find('"').unwrap()]
            .parse()
            .unwrap()
    };
    // `(x, y, width, height)` of the nodes, by name.
    let nodes = |svg: &str| -> Vec<(String, f32, f32, f32, f32)> {
        svg.split("<rect")
            .skip(1)
            .filter(|t| t.contains("data-name="))
            .map(|t| {
                let start = t.find("data-name=\"").unwrap() + 11;
                (
                    t[start..start + t[start..].find('"').unwrap()].to_string(),
                    number(t, "x"),
                    number(t, "y"),
                    number(t, "width"),
                    number(t, "height"),
                )
            })
            .collect()
    };
    let across = SankeyChart::from_json(&json("")).unwrap().svg().unwrap();
    let explicit = SankeyChart::from_json(&json(r#", "orient": "horizontal""#)).unwrap();
    assert_eq!(across, explicit.svg().unwrap());
    let down = SankeyChart::from_json(&json(r#", "orient": "Vertical""#))
        .unwrap()
        .svg()
        .unwrap();

    // On a square plot the diagram is the same one, turned: what was x is
    // y, and a node is as wide as it was high.
    let (h, v) = (nodes(&across), nodes(&down));
    assert_eq!(5, h.len());
    for (a, b) in h.iter().zip(v.iter()) {
        assert_eq!(a.0, b.0);
        assert_eq!((a.1, a.2, a.3, a.4), (b.2, b.1, b.4, b.3), "{}", a.0);
    }
    // The sources at the top, the flows down to the last row.
    let row = |name: &str| v.iter().find(|n| n.0 == name).unwrap().2;
    assert_eq!(row("a"), row("b"));
    assert!(row("a") < row("c") && row("c") < row("d"));
    assert_eq!(row("d"), row("e"));

    // The names stand under the nodes of the upper half, and over the
    // others, centered on them.
    let label = |name: &str| {
        down.split("<text")
            .find(|t| t.contains(&format!("\n{name}\n")))
            .unwrap()
            .to_string()
    };
    let a = v.iter().find(|n| n.0 == "a").unwrap();
    let d = v.iter().find(|n| n.0 == "d").unwrap();
    assert!(label("a").contains(r#"text-anchor="middle""#));
    assert!((number(&label("a"), "x") - (a.1 + a.3 / 2.0)).abs() < 0.2);
    assert!(number(&label("a"), "y") > a.2 + a.4);
    assert!(number(&label("d"), "y") < d.2);

    // The flows grow downwards too.
    let animated = SankeyChart::from_json(&json(r#", "orient": "vertical", "animation": {}"#))
        .unwrap()
        .svg()
        .unwrap();
    assert!(
        animated.contains("transform:scaleY(0)")
            && animated.contains("transform-origin:center top")
    );

    let message = match SankeyChart::from_json(&json(r#", "orient": "diagonal""#)) {
        Ok(_) => panic!("accepted"),
        Err(e) => e.to_string(),
    };
    assert!(message.contains("horizontal, vertical"), "{message}");
}
