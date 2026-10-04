mod common;

use charts_rs::TreeChart;

#[test]
fn tree_chart_json() {
    let chart = TreeChart::from_json(include_str!("../asset/tree_chart/integration.json")).unwrap();
    common::assert_snapshot!("tree_chart/integration_json.svg", chart.svg().unwrap());
}

/// `(cx, cy)` of the node circles, in drawing order, keyed by name.
fn node_positions(svg: &str) -> Vec<(String, f32, f32)> {
    let attr = |tag: &str, name: &str| -> String {
        let key = format!(" {name}=\"");
        let start = tag.find(&key).unwrap() + key.len();
        tag[start..start + tag[start..].find('"').unwrap()].to_string()
    };
    svg.split("<circle")
        .skip(1)
        .filter(|t| t.contains("data-name="))
        .map(|t| {
            (
                attr(t, "data-name"),
                attr(t, "cx").parse().unwrap(),
                attr(t, "cy").parse().unwrap(),
            )
        })
        .collect()
}

/// The points of every link.
fn link_points(svg: &str) -> Vec<Vec<(f32, f32)>> {
    svg.split("<polyline")
        .skip(1)
        .map(|t| {
            let start = t.find("points=\"").unwrap() + 8;
            t[start..start + t[start..].find('"').unwrap()]
                .split(' ')
                .map(|p| {
                    let (x, y) = p.split_once(',').unwrap();
                    (x.parse().unwrap(), y.parse().unwrap())
                })
                .collect()
        })
        .collect()
}

fn small_tree(options: &str) -> String {
    let json = format!(
        r#"{{"width": 400, "height": 400{options}, "series_data": [{{"name": "root", "children": [
            {{"name": "a", "children": [{{"name": "a1"}}, {{"name": "a2"}}]}},
            {{"name": "b"}},
            {{"name": "c"}}
        ]}}]}}"#
    );
    TreeChart::from_json(&json).unwrap().svg().unwrap()
}

#[test]
fn tree_grows_to_every_side() {
    let by_name = |svg: &str, name: &str| -> (f32, f32) {
        let nodes = node_positions(svg);
        let node = nodes.iter().find(|n| n.0 == name).unwrap();
        (node.1, node.2)
    };
    let (lr, rl) = (small_tree(""), small_tree(r#", "orient": "RL""#));
    // The same tree, mirrored: the root on the right, the leaves on the left.
    assert!(by_name(&lr, "root").0 < by_name(&lr, "a1").0);
    assert!(by_name(&rl, "root").0 > by_name(&rl, "a1").0);
    for name in ["root", "a", "a1", "b"] {
        let (a, b) = (by_name(&lr, name), by_name(&rl, name));
        assert_eq!(a.1, b.1, "{name}");
        // Mirrored in the 390 pixels between the margins.
        assert!((a.0 + b.0 - 400.0).abs() < 0.2, "{name}: {a:?} {b:?}");
    }
    // The names change sides too: a leaf is named on the far side.
    let anchor = |svg: &str, name: &str| -> String {
        let label = svg
            .split("<text")
            .find(|t| t.contains(&format!("\n{name}\n")))
            .unwrap();
        let start = label.find("text-anchor=\"").unwrap() + 13;
        label[start..start + label[start..].find('"').unwrap()].to_string()
    };
    assert_eq!(
        ("start", "end"),
        (anchor(&lr, "a1").as_str(), anchor(&lr, "root").as_str())
    );
    assert_eq!(
        ("end", "start"),
        (anchor(&rl, "a1").as_str(), anchor(&rl, "root").as_str())
    );

    // Top to bottom, and bottom to top; the names are taken whatever their
    // case, like every enum.
    let (tb, bt) = (
        small_tree(r#", "orient": "tb""#),
        small_tree(r#", "orient": "Bt""#),
    );
    assert!(by_name(&tb, "root").1 < by_name(&tb, "a1").1);
    assert!(by_name(&bt, "root").1 > by_name(&bt, "a1").1);
    assert_eq!(by_name(&tb, "b").0, by_name(&bt, "b").0);
    assert_eq!(small_tree(r#", "orient": "TB""#), tb);
}

#[test]
fn tree_links_with_corners() {
    let curved = small_tree("");
    let cornered = small_tree(r#", "edge_shape": "polyline""#);
    assert_eq!(node_positions(&curved), node_positions(&cornered));
    let links = link_points(&cornered);
    assert_eq!(5, links.len());
    for link in links.iter() {
        // Across, over, across: every piece level or upright.
        assert_eq!(4, link.len());
        assert_eq!(link[0].1, link[1].1);
        assert_eq!(link[1].0, link[2].0);
        assert_eq!(link[2].1, link[3].1);
        assert!((link[1].0 - (link[0].0 + link[3].0) / 2.0).abs() < 0.1);
    }
    assert!(link_points(&curved)[0].len() > 4);
    // In a tree from the top the corners are the other way round.
    let links = link_points(&small_tree(r#", "orient": "TB", "edge_shape": "polyline""#));
    assert_eq!(links[0][0].0, links[0][1].0);
    assert_eq!(links[0][1].1, links[0][2].1);
}

#[test]
fn tree_around_its_root() {
    let chart = TreeChart::from_json(include_str!("../asset/tree_chart/radial.json")).unwrap();
    common::assert_snapshot!("tree_chart/radial_json.svg", chart.svg().unwrap());

    let svg = small_tree(r#", "layout": "Radial""#);
    let nodes = node_positions(&svg);
    let (root, others) = (&nodes[0], &nodes[1..]);
    // The root in the middle of the plot, every level on a ring around it.
    assert!(
        (root.1 - 200.0).abs() < 0.5 && (root.2 - 200.0).abs() < 0.5,
        "{root:?}"
    );
    let distance = |name: &str| -> f32 {
        let node = others.iter().find(|n| n.0 == name).unwrap();
        ((node.1 - root.1).powi(2) + (node.2 - root.2).powi(2)).sqrt()
    };
    for name in ["b", "c"] {
        assert!((distance(name) - distance("a")).abs() < 0.2);
    }
    assert!((distance("a1") - 2.0 * distance("a")).abs() < 0.3);
    assert!((distance("a1") - distance("a2")).abs() < 0.2);
    // The four leaves share the turn: a1 at the top, the others a quarter on.
    let leaf = |name: &str| {
        let node = others.iter().find(|n| n.0 == name).unwrap();
        (node.1 - root.1, node.2 - root.2)
    };
    assert!(leaf("a1").0.abs() < 0.5 && leaf("a1").1 < 0.0);
    assert!(leaf("a2").0 > 0.0 && leaf("a2").1.abs() < 0.5);
    assert!(leaf("b").0.abs() < 0.5 && leaf("b").1 > 0.0);
    assert!(leaf("c").0 < 0.0 && leaf("c").1.abs() < 0.5);
    // Their names run along their spokes.
    assert_eq!(4, svg.matches("transform=\"rotate(").count() - 1);
    // Curved links by default, straight ones with corners asked for.
    assert!(link_points(&svg)[0].len() > 4);
    let straight = small_tree(r#", "layout": "radial", "edge_shape": "polyline""#);
    assert!(link_points(&straight).iter().all(|l| l.len() == 2));

    for (key, value) in [
        ("layout", "circular"),
        ("edge_shape", "step"),
        ("orient", "up"),
    ] {
        let json = format!(r#"{{"{key}": "{value}"}}"#);
        let message = match TreeChart::from_json(&json) {
            Ok(_) => panic!("{key}: {value} is accepted"),
            Err(e) => e.to_string(),
        };
        assert!(message.contains(key), "{message}");
    }
}
