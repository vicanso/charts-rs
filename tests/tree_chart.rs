mod common;

use charts_rs::{TreeChart, measure_text_width_family};

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

/// Whether `written` is the name, in full or cut with an ellipsis.
fn is_name(written: &str, name: &str) -> bool {
    written == name
        || written
            .strip_suffix('…')
            .is_some_and(|cut| name.starts_with(cut))
}

/// Where the name of a node is written, along its spoke: in full or cut.
fn name_position(svg: &str, name: &str) -> (f32, f32) {
    let tag = svg
        .split("<text")
        .skip(1)
        .find(|t| {
            t.contains("transform=\"rotate(")
                && t.split('\n').nth(1).is_some_and(|text| is_name(text, name))
        })
        .unwrap();
    let attr = |name: &str| -> f32 {
        let key = format!(" {name}=\"");
        let start = tag.find(&key).unwrap() + key.len();
        tag[start..start + tag[start..].find('"').unwrap()]
            .parse()
            .unwrap()
    };
    (attr("x"), attr("y"))
}

#[test]
fn tree_names_around_the_root_keep_clear() {
    let position = |svg: &str, name: &str| -> (f32, f32) {
        let nodes = node_positions(svg);
        let node = nodes.iter().find(|n| n.0 == name).unwrap();
        (node.1, node.2)
    };
    // How far the name of a node is from its spoke, and from the middle
    // compared to the node itself.
    let placed = |svg: &str, name: &str| -> (f32, f32) {
        let (root, node, at) = (
            position(svg, "root"),
            position(svg, name),
            name_position(svg, name),
        );
        let spoke = (node.0 - root.0, node.1 - root.1);
        let to = (at.0 - root.0, at.1 - root.1);
        let length = spoke.0.hypot(spoke.1);
        (
            (spoke.0 * to.1 - spoke.1 * to.0).abs() / length,
            to.0.hypot(to.1) - length,
        )
    };
    let tree = |size: u32, children: &str| -> String {
        let json = format!(
            r#"{{"width": {size}, "height": {size}, "layout": "radial", "series_data": [
                {{"name": "root", "children": [{children}]}}
            ]}}"#
        );
        TreeChart::from_json(&json).unwrap().svg().unwrap()
    };
    let leaves = |names: &[&str]| -> String {
        names
            .iter()
            .map(|n| format!(r#"{{"name": "{n}"}}"#))
            .collect::<Vec<_>>()
            .join(",")
    };

    // The middle one of three children is on the spoke of its parent: with
    // no room before it, the name of the parent is beside the spoke.
    let branch = |name: &str, children: &[&str]| -> String {
        format!(
            r#"{{"name": "{name}", "children": [{}]}}"#,
            leaves(children)
        )
    };
    let three = format!(
        "{},{}",
        branch("a branch of three", &["x", "y", "z"]),
        leaves(&["p", "q", "r"])
    );
    let (beside, further) = placed(&tree(300, &three), "a branch of three");
    assert!(beside > 10.0 && further > 0.0, "{beside} {further}");
    // Between two children there is room: the name stays on the spoke.
    let two = format!(
        "{},{}",
        branch("a branch of two", &["x", "y"]),
        leaves(&["p", "q", "r"])
    );
    let (beside, further) = placed(&tree(300, &two), "a branch of two");
    assert!(beside < 0.5 && further > 0.0, "{beside} {further}");
    // With room for it before the child, it is on the spoke as well.
    let short = format!("{},{}", branch("b", &["x", "y", "z"]), leaves(&["p", "q"]));
    let (beside, _) = placed(&tree(600, &short), "b");
    assert!(beside < 0.5, "{beside}");

    // A name is written towards the middle when there is room before the
    // root. The root has its own name above it, so that a spoke that points
    // up needs more room than one that points down.
    let first_inwards = |children: &str| -> u32 {
        (200..=1200)
            .step_by(20)
            .find(|size| placed(&tree(*size, children), "branch").1 < 0.0)
            .unwrap()
    };
    let up = format!(
        "{},{}",
        branch("branch", &["x", "y"]),
        leaves(&["a", "b", "c", "d", "e", "f"])
    );
    let down = format!(
        "{},{},{}",
        leaves(&["a", "b", "c"]),
        branch("branch", &["x", "y"]),
        leaves(&["d", "e", "f"])
    );
    assert!(
        first_inwards(&up) > first_inwards(&down),
        "{} {}",
        first_inwards(&up),
        first_inwards(&down)
    );
}

/// A name of a tree around its root as it is written: its text, and the
/// line through the middle of it from its first letter to its last.
struct WrittenName {
    text: String,
    from: (f32, f32),
    to: (f32, f32),
}

/// The names along the spokes, and the one above the node in the middle.
fn written_names(svg: &str, font_size: f32) -> Vec<WrittenName> {
    let attr = |tag: &str, name: &str| -> Option<String> {
        let key = format!(" {name}=\"");
        let start = tag.find(&key)? + key.len();
        Some(tag[start..start + tag[start..].find('"')?].to_string())
    };
    svg.split("<text")
        .skip(1)
        .filter(|tag| !tag.contains("class=\"ct-tip\"") && !tag.contains("font-weight=\"bold\""))
        .map(|tag| {
            let text = tag.split('\n').nth(1).unwrap().to_string();
            let number = |name: &str| -> f32 { attr(tag, name).unwrap().parse().unwrap() };
            let (x, y) = (number("x"), number("y"));
            let width = measure_text_width_family("Roboto", font_size, &text)
                .unwrap()
                .width();
            // The way the text runs, and where it is from there.
            let turn = attr(tag, "transform").map_or(0.0, |transform| {
                let turn = transform.trim_start_matches("rotate(");
                turn[..turn.find(' ').unwrap()].parse::<f32>().unwrap()
            });
            let (sin, cos) = turn.to_radians().sin_cos();
            let (before, after) = match attr(tag, "text-anchor").as_deref() {
                Some("end") => (width, 0.0),
                Some("middle") => (width / 2.0, width / 2.0),
                _ => (0.0, width),
            };
            WrittenName {
                text,
                from: (x - cos * before, y - sin * before),
                to: (x + cos * after, y + sin * after),
            }
        })
        .collect()
}

/// How far a point is from a line.
fn point_distance(point: (f32, f32), (from, to): ((f32, f32), (f32, f32))) -> f32 {
    let (dx, dy) = (to.0 - from.0, to.1 - from.1);
    let length = dx * dx + dy * dy;
    let t = if length > 0.0 {
        (((point.0 - from.0) * dx + (point.1 - from.1) * dy) / length).clamp(0.0, 1.0)
    } else {
        0.0
    };
    (point.0 - from.0 - dx * t).hypot(point.1 - from.1 - dy * t)
}

/// How far two lines are apart: the least of what their points are, a
/// third of a pixel from one to the next.
fn lines_apart(a: ((f32, f32), (f32, f32)), b: ((f32, f32), (f32, f32))) -> f32 {
    let along = |(from, to): ((f32, f32), (f32, f32)), other| {
        let steps = ((to.0 - from.0).hypot(to.1 - from.1) * 3.0).ceil().max(1.0) as usize;
        (0..=steps)
            .map(|i| {
                let t = i as f32 / steps as f32;
                let point = (from.0 + (to.0 - from.0) * t, from.1 + (to.1 - from.1) * t);
                point_distance(point, other)
            })
            .fold(f32::MAX, f32::min)
    };
    along(a, b).min(along(b, a))
}

const FLARE: &str = include_str!("../asset/tree_chart/radial.json");

/// The tree of the gallery on a canvas of another size.
fn flare(width: u32, height: u32) -> String {
    let json = FLARE.replace(
        r#""height": 560,"#,
        &format!(r#""width": {width}, "height": {height},"#),
    );
    TreeChart::from_json(&json).unwrap().svg().unwrap()
}

#[test]
fn tree_names_around_the_root_are_cut_to_fit() {
    // The nodes are 6 pixels in radius and the names 14 pixels high, of
    // which their letters take about four fifths.
    let (r, height) = (6.0, 14.0 * 0.8);
    let long = r#"{"layout": "radial", "width": 420, "height": 420, "tooltip_show": true,
        "series_data": [{"name": "Organization", "children": [
        {"name": "Research and Development", "children": [{"name": "Machine Learning Platform"},
            {"name": "Infrastructure"}, {"name": "Developer Experience"}]},
        {"name": "Sales and Marketing", "children": [{"name": "Enterprise Accounts"},
            {"name": "Growth"}, {"name": "Partnerships and Alliances"}]},
        {"name": "Operations", "children": [{"name": "Finance"}, {"name": "People"},
            {"name": "Legal and Compliance"}, {"name": "Workplace"}]}
    ]}]}"#;
    let charts = [
        ("600x560", (600.0, 560.0), flare(600, 560)),
        ("400x400", (400.0, 400.0), flare(400, 400)),
        ("300x300", (300.0, 300.0), flare(300, 300)),
        ("240x240", (240.0, 240.0), flare(240, 240)),
        ("500x260", (500.0, 260.0), flare(500, 260)),
        (
            "long names",
            (420.0, 420.0),
            TreeChart::from_json(long).unwrap().svg().unwrap(),
        ),
    ];
    for (case, (width, high), svg) in charts.iter() {
        let nodes = node_positions(svg);
        let names = written_names(svg, 14.0);
        assert!(!names.is_empty() && names.len() <= nodes.len(), "{case}");
        for name in names.iter() {
            // The name of a node, in full or two letters of it at the least.
            let of: Vec<_> = nodes
                .iter()
                .filter(|node| is_name(&name.text, &node.0))
                .collect();
            assert!(!of.is_empty(), "{case}: {}", name.text);
            assert!(
                name.text.chars().count() > 2 || of[0].0 == name.text,
                "{case}"
            );
            // On the chart, all of it.
            for (x, y) in [name.from, name.to] {
                assert!(
                    (-0.5..=width + 0.5).contains(&x) && (-0.5..=high + 0.5).contains(&y),
                    "{case}: {} at {x},{y}",
                    name.text
                );
            }
            // Over no node, but for the one it is the name of.
            for node in nodes.iter().filter(|node| !is_name(&name.text, &node.0)) {
                let apart = point_distance((node.1, node.2), (name.from, name.to));
                assert!(
                    apart > r + height / 2.0 - 0.2,
                    "{case}: {} is {apart} from {}",
                    name.text,
                    node.0
                );
            }
        }
        // And over no other name.
        for (i, a) in names.iter().enumerate() {
            for b in names[i + 1..].iter() {
                let apart = lines_apart((a.from, a.to), (b.from, b.to));
                assert!(
                    apart > height - 0.4,
                    "{case}: {} is {apart} from {}",
                    a.text,
                    b.text
                );
            }
        }
        // A node keeps its name whatever is written of it: in what it tells
        // a script, and in its tooltip.
        for (name, _, _) in nodes.iter() {
            assert!(
                svg.contains(&format!("<title>{name}</title>")),
                "{case}: {name}"
            );
            assert!(
                svg.contains(&format!("class=\"ct-tip\">\n{name}\n")),
                "{case}: {name}"
            );
        }
    }

    let cut = |svg: &str| -> usize {
        written_names(svg, 14.0)
            .iter()
            .filter(|name| name.text.ends_with('…'))
            .count()
    };
    // With room for them, every name is written, and in full.
    let roomy = &charts[0].2;
    assert_eq!(31, node_positions(roomy).len());
    assert_eq!((31, 0), (written_names(roomy, 14.0).len(), cut(roomy)));
    // With less of it the names of the leaves are all there, and in full;
    // those of the nodes they hang from, which would be over them, are not.
    let less = &charts[1].2;
    let names = written_names(less, 14.0);
    assert_eq!((23, 0), (names.len(), cut(less)));
    for leaf in ["Agglomerative", "ShortestPaths", "Transition", "TextSprite"] {
        assert!(names.iter().any(|name| name.text == leaf), "{leaf}");
    }
    for inner in ["flare", "analytics", "animate", "data", "display"] {
        assert!(
            names.iter().all(|name| !is_name(&name.text, inner)),
            "{inner}"
        );
    }
    // With still less the long ones are cut, at the edge of the chart.
    let small = &charts[2].2;
    let names = written_names(small, 14.0);
    assert_eq!(23, names.len());
    assert!(cut(small) > 3, "{}", cut(small));
    assert!(names.iter().any(|name| name.text == "Easing"));
    assert!(
        names
            .iter()
            .any(|name| name.text != "Agglomerative" && is_name(&name.text, "Agglomerative"))
    );
    // Where the leaves are no line of text apart, every other name is left.
    let tiny = written_names(&charts[3].2, 14.0);
    assert!(tiny.len() > 8 && tiny.len() < 23, "{}", tiny.len());
    // On a chart wider than high the names to the sides have their room.
    let wide = written_names(&charts[4].2, 14.0);
    assert!(wide.iter().any(|name| name.text == "ShortestPaths"));
    assert!(wide.iter().any(|name| name.text == "DataSchema"));
    assert!(wide.iter().all(|name| name.text != "Agglomerative"));
}

#[test]
fn tree_around_its_root_is_no_smaller_than_it_is_read() {
    // How far out the leaves are, and how far the first ring.
    let rings = |svg: &str| -> (f32, f32) {
        let nodes = node_positions(svg);
        let root = nodes.iter().find(|node| node.0 == "flare").unwrap();
        let far = |name: &str| -> f32 {
            let node = nodes.iter().find(|node| node.0 == name).unwrap();
            (node.1 - root.1).hypot(node.2 - root.2)
        };
        (far("Agglomerative"), far("analytics"))
    };
    // With room for the names in full the tree is what they leave.
    let (outer, first) = rings(&flare(600, 560));
    assert!(
        outer > 140.0 && (first * 3.0 - outer).abs() < 0.5,
        "{outer}"
    );
    // On 400 pixels that is still enough for it.
    let (outer, _) = rings(&flare(400, 400));
    assert!(outer > 66.0 && outer < 70.0, "{outer}");
    // With less its three levels are a node and a gap apart each, 22
    // pixels, and it is the names that are shorter: the tree was one of 10
    // pixels across before, its nodes one over the other.
    for (width, height) in [(360, 360), (300, 300)] {
        let (outer, first) = rings(&flare(width, height));
        assert!((outer - 66.0).abs() < 0.5, "{width}: {outer}");
        assert!((first - 22.0).abs() < 0.5, "{width}: {first}");
    }
    // Until the names would have no room for a few letters: three times
    // their font size is what they keep, 42 pixels of the half of the
    // chart, which is less its header and as high as it is wide at the most.
    for ((width, height), half) in [((240, 240), 97.5), ((500, 260), 107.5)] {
        let (outer, _) = rings(&flare(width, height));
        assert!(
            (outer - (half - 16.0 - 42.0)).abs() < 0.5,
            "{width}: {outer}"
        );
        let names = written_names(&flare(width, height), 14.0);
        assert!(names.iter().any(|name| name.text.chars().count() > 4));
    }

    // Many leaves want a ring that is a line of text apiece around: their
    // names, which would leave less, are cut for it.
    let many = |size: u32| -> (f32, usize) {
        let leaves: Vec<String> = (0..80)
            .map(|i| format!(r#"{{"name": "a leaf of a long name {i}"}}"#))
            .collect();
        let svg = TreeChart::from_json(&format!(
            r#"{{"width": {size}, "height": {size}, "layout": "radial", "series_data": [
                {{"name": "root", "children": [{}]}}]}}"#,
            leaves.join(",")
        ))
        .unwrap()
        .svg()
        .unwrap();
        let nodes = node_positions(&svg);
        let cut = written_names(&svg, 14.0)
            .iter()
            .filter(|name| name.text.ends_with('…'))
            .count();
        (
            (nodes[1].1 - nodes[0].1).hypot(nodes[1].2 - nodes[0].2),
            cut,
        )
    };
    // 80 lines of 14 pixels around, less the gap before a name.
    let around = 80.0 * 14.0 / std::f32::consts::TAU - 10.0;
    let (outer, cut) = many(600);
    assert!((outer - around).abs() < 0.5, "{outer}");
    assert!(cut > 40, "{cut}");
    // With room for more than that, more, and the names in full.
    let (outer, cut) = many(1200);
    assert!(outer > around + 100.0 && cut == 0, "{outer} {cut}");
}
