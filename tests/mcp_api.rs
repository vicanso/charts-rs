// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use charts_rs::{
    Error, THEME_ANT, THEME_CHALK, THEME_DARK, THEME_GRAFANA, THEME_LIGHT, THEME_SHADCN,
    THEME_SHINE, THEME_VINTAGE, THEME_WALDEN, THEME_WESTEROS, chart_json_schema, get_theme,
    list_chart_types, list_themes, render_chart_from_json,
};
use std::collections::HashSet;

#[test]
fn list_chart_types_is_complete_sorted_and_unique() {
    let types = list_chart_types();
    let expected_len = if cfg!(feature = "mermaid") { 30 } else { 29 };
    assert_eq!(types.len(), expected_len, "chart types: {types:?}");

    let unique: HashSet<&str> = types.iter().copied().collect();
    assert_eq!(unique.len(), types.len(), "duplicate chart type in list");

    let mut sorted = types.to_vec();
    sorted.sort_unstable();
    assert_eq!(types, sorted.as_slice(), "chart types should be sorted");

    assert!(types.contains(&"bar"));
    assert!(types.contains(&"multi"));
    assert_eq!(types.contains(&"mermaid"), cfg!(feature = "mermaid"));
}

#[test]
fn list_themes_matches_all_builtin_constants() {
    let themes = list_themes();
    let expected = [
        THEME_LIGHT,
        THEME_DARK,
        THEME_ANT,
        THEME_GRAFANA,
        THEME_VINTAGE,
        THEME_WALDEN,
        THEME_WESTEROS,
        THEME_CHALK,
        THEME_SHINE,
        THEME_SHADCN,
    ];
    assert_eq!(themes.len(), expected.len());
    for name in expected {
        assert!(
            themes.contains(&name),
            "missing built-in theme `{name}` in list_themes()"
        );
        // Ensure get_theme resolves without falling back unexpectedly.
        let _ = get_theme(name);
    }
}

#[test]
fn chart_json_schema_is_valid_draft_2020_12_for_every_chart_type() {
    let themes: Vec<serde_json::Value> = list_themes()
        .iter()
        .map(|t| serde_json::Value::String((*t).to_string()))
        .collect();

    for &chart_type in list_chart_types() {
        let schema = chart_json_schema(chart_type)
            .unwrap_or_else(|e| panic!("chart_json_schema({chart_type}) failed: {e}"));

        assert_eq!(
            schema["$schema"], "https://json-schema.org/draft/2020-12/schema",
            "{chart_type}: missing Draft 2020-12 $schema"
        );
        assert_eq!(schema["type"], "object", "{chart_type}: root type");
        assert_eq!(
            schema["additionalProperties"], false,
            "{chart_type}: additionalProperties"
        );
        assert!(
            schema["description"]
                .as_str()
                .is_some_and(|s| !s.is_empty()),
            "{chart_type}: missing description"
        );
        let props = schema["properties"]
            .as_object()
            .unwrap_or_else(|| panic!("{chart_type}: properties must be an object"));
        assert!(!props.is_empty(), "{chart_type}: empty properties");

        // Every chart accepts `theme`, with enum matching list_themes().
        let theme_prop = props
            .get("theme")
            .unwrap_or_else(|| panic!("{chart_type}: missing `theme` property"));
        assert_eq!(
            theme_prop["enum"].as_array(),
            Some(&themes),
            "{chart_type}: theme enum should match list_themes()"
        );
    }

    // Recursive tree schemas expose `$defs.tree_node`.
    for tree_type in ["sunburst", "tree", "treemap"] {
        let schema = chart_json_schema(tree_type).unwrap();
        assert!(
            schema["$defs"]["tree_node"]["properties"]["children"]["items"]["$ref"]
                == "#/$defs/tree_node",
            "{tree_type}: missing recursive tree_node $ref"
        );
    }

    // MultiChart child_charts carries enum of child chart types.
    let multi = chart_json_schema("multi").unwrap();
    let child_types = multi["properties"]["child_charts"]["items"]["properties"]["type"]["enum"]
        .as_array()
        .expect("multi.child_charts.items.properties.type.enum");
    assert!(child_types.iter().any(|v| v == "bar"));
    assert!(!child_types.iter().any(|v| v == "multi"));

    #[cfg(feature = "mermaid")]
    {
        let mermaid = chart_json_schema("mermaid").unwrap();
        assert_eq!(mermaid["required"], serde_json::json!(["source"]));
        assert_eq!(mermaid["properties"]["allow_links"]["type"], "boolean");
    }
}

#[test]
fn unknown_chart_type_returns_params_error_listing_valid_types() {
    for err in [
        chart_json_schema("not_a_chart").unwrap_err(),
        render_chart_from_json("not_a_chart", "{}").unwrap_err(),
    ] {
        match err {
            Error::Params { message } => {
                assert!(message.contains("not_a_chart"), "{message}");
                assert!(message.contains("bar"), "{message}");
                assert!(message.contains("waterfall"), "{message}");
            }
            other => panic!("expected Error::Params, got {other:?}"),
        }
    }
}

#[test]
fn render_chart_from_json_works_for_every_chart_type() {
    let manifest_dir = env!("CARGO_MANIFEST_DIR");

    let fixture_for = |chart_type: &str| -> String {
        let asset_path = match chart_type {
            "bar" => Some("asset/bar_chart/stack_percent.json"),
            "bullet" => Some("asset/bullet_chart/basic.json"),
            "chord" => Some("asset/chord_chart/basic.json"),
            "gantt" => Some("asset/gantt_chart/basic.json"),
            "gauge" => Some("asset/gauge_chart/segments.json"),
            "graph" => Some("asset/graph_chart/integration.json"),
            "heatmap" => Some("asset/heatmap_chart/scale.json"),
            "histogram" => Some("asset/histogram_chart/basic.json"),
            "line" => Some("asset/line_chart/step.json"),
            "lines" => Some("asset/lines_chart/flows.json"),
            "map" => Some("asset/map_chart/basic.json"),
            #[cfg(feature = "mermaid")]
            "mermaid" => Some("asset/mermaid_chart/flowchart.json"),
            "multi" => Some("asset/multi_chart/aligned.json"),
            "parallel" => Some("asset/parallel_chart/integration.json"),
            "pie" => Some("asset/pie_chart/half.json"),
            "polar_bar" => Some("asset/polar_bar_chart/basic.json"),
            "sankey" => Some("asset/sankey_chart/integration.json"),
            "scatter" => Some("asset/scatter_chart/bubble.json"),
            "table" => Some("asset/table_chart/sparkline.json"),
            "theme_river" => Some("asset/theme_river_chart/integration.json"),
            "tree" => Some("asset/tree_chart/integration.json"),
            _ => None,
        };
        if let Some(rel) = asset_path {
            return std::fs::read_to_string(format!("{manifest_dir}/{rel}"))
                .unwrap_or_else(|e| panic!("failed to read {rel}: {e}"));
        }
        match chart_type {
            "box_plot" => r#"{"x_axis_data":["A","B"],"box_series":[{"name":"S","data":[[1,2,3,4,5],[2,4,6,8,10]]}]}"#.to_string(),
            "calendar" => r#"{"start_date":"2024-01-01","end_date":"2024-03-31","data":[["2024-01-10",5],["2024-02-14",12]]}"#.to_string(),
            "candlestick" => r#"{"x_axis_data":["D1","D2"],"series_list":[{"name":"K","data":[10,15,8,18,15,12,11,17]}]}"#.to_string(),
            "funnel" => r#"{"series_list":[{"name":"Visit","data":[100]},{"name":"Buy","data":[30]}]}"#.to_string(),
            "horizontal_bar" => r#"{"x_axis_data":["A","B"],"series_list":[{"name":"S","data":[10,25]}]}"#.to_string(),
            "radar" => r#"{"indicators":[{"name":"A","max":100},{"name":"B","max":100},{"name":"C","max":100}],"series_list":[{"name":"S","data":[80,60,90]}]}"#.to_string(),
            "sunburst" => r#"{"series_data":[{"name":"Root","children":[{"name":"A","value":3},{"name":"B","value":2}]}]}"#.to_string(),
            "treemap" => r#"{"series_data":[{"name":"A","value":60},{"name":"B","value":40}]}"#.to_string(),
            "waterfall" => r#"{"x_axis_data":["Start","Up","Down","End"],"data":[[100,false],[40,false],[-20,false],[120,true]]}"#.to_string(),
            other => panic!("missing fixture for chart type `{other}`"),
        }
    };

    for &chart_type in list_chart_types() {
        let json = fixture_for(chart_type);
        let svg = render_chart_from_json(chart_type, &json)
            .unwrap_or_else(|e| panic!("render_chart_from_json({chart_type}) failed: {e}"));
        assert!(
            svg.starts_with("<svg"),
            "{chart_type}: output did not start with <svg"
        );
        assert!(
            svg.ends_with("</svg>"),
            "{chart_type}: output did not end with </svg>"
        );
    }
}

#[cfg(feature = "mermaid")]
#[test]
fn mermaid_capabilities_matches_renderer_and_serializes_to_json() {
    use charts_rs::{mermaid_capabilities, mermaid_validate};

    let caps = mermaid_capabilities();
    let json = serde_json::to_value(&caps).expect("MermaidCapabilities must serialize to JSON");
    assert_eq!(caps.supported_diagrams.len(), 13);
    assert_eq!(caps.diagram_aliases.len(), 7);
    assert!(
        json["supported_diagrams"]
            .as_array()
            .is_some_and(|a| a.len() == 13)
    );
    assert!(
        json["diagram_aliases"]
            .as_array()
            .is_some_and(|a| a.len() == 7)
    );
    assert!(
        json["unsupported_diagrams"]
            .as_array()
            .is_some_and(|a| !a.is_empty())
    );
    assert!(json["v11_shapes"].as_array().is_some_and(|a| !a.is_empty()));
    assert!(
        json["init_config_keys"]
            .as_array()
            .is_some_and(|a| !a.is_empty())
    );
    assert!(json["features"].as_array().is_some_and(|a| !a.is_empty()));
    assert!(
        json["limitations"]
            .as_array()
            .is_some_and(|a| !a.is_empty())
    );

    for &(_, canonical) in caps.diagram_aliases {
        assert!(
            caps.supported_diagrams.contains(&canonical),
            "alias canonical target `{canonical}` not in supported_diagrams"
        );
    }

    // Every advertised canonical diagram keyword and alias must validate on a minimal diagram.
    for &kw in caps
        .supported_diagrams
        .iter()
        .chain(caps.diagram_aliases.iter().map(|(alias, _)| alias))
    {
        let src = match kw {
            "flowchart" | "graph" => format!("{kw} LR\n  A --> B"),
            "sequenceDiagram" => "sequenceDiagram\n  Alice->>Bob: Hi".to_string(),
            "stateDiagram" | "stateDiagram-v2" => format!("{kw}\n  [*] --> Active"),
            "classDiagram" | "classDiagram-v2" => format!("{kw}\n  class Foo"),
            "erDiagram" => "erDiagram\n  A ||--o{ B : has".to_string(),
            "mindmap" => "mindmap\n  root((R))\n    Child".to_string(),
            "quadrantChart" => {
                "quadrantChart\n  x-axis Low --> High\n  y-axis Low --> High\n  A: [0.3, 0.7]"
                    .to_string()
            }
            "pie" => "pie\n  \"A\" : 60\n  \"B\" : 40".to_string(),
            "gantt" => {
                "gantt\n  dateFormat YYYY-MM-DD\n  section S\n  T1 :a1, 2024-01-01, 5d".to_string()
            }
            "sankey-beta" | "sankey" => format!("{kw}\nA,B,10"),
            "xychart-beta" | "xychart" => {
                format!("{kw}\n  x-axis [a, b]\n  bar [1, 2]")
            }
            "radar-beta" | "radar" => {
                format!("{kw}\n  axis a, b, c\n  curve s{{1, 2, 3}}")
            }
            "treemap-beta" | "treemap" => format!("{kw}\n\"Root\"\n  \"A\": 10"),
            other => panic!("missing sample for supported Mermaid keyword `{other}`"),
        };
        mermaid_validate(&src)
            .unwrap_or_else(|e| panic!("supported Mermaid keyword `{kw}` failed validation: {e}"));
    }

    // Every advertised unsupported diagram keyword must be rejected with Error::Mermaid.
    for &kw in caps.unsupported_diagrams {
        let err = mermaid_validate(&format!("{kw}\n  foo")).unwrap_err();
        assert!(
            matches!(err, Error::Mermaid { .. }),
            "unsupported keyword `{kw}` should return Error::Mermaid, got {err:?}"
        );
    }

    // Every advertised v11 shape name/alias must validate in a flowchart.
    for &shape in caps.v11_shapes {
        let src = format!("flowchart LR\n  A@{{ shape: {shape}, label: \"X\" }}");
        mermaid_validate(&src)
            .unwrap_or_else(|e| panic!("v11 shape `{shape}` failed validation: {e}"));
    }
}

fn walk_properties(
    schema: &serde_json::Value,
    path: &str,
    visit: &mut dyn FnMut(&str, &serde_json::Value),
) {
    if let Some(props) = schema.get("properties").and_then(|p| p.as_object()) {
        for (key, prop) in props {
            let path = if path.is_empty() {
                key.clone()
            } else {
                format!("{path}.{key}")
            };
            visit(&path, prop);
            walk_properties(prop, &path, visit);
        }
    }
    if let Some(items) = schema.get("items") {
        walk_properties(items, path, visit);
    }
    for key in ["anyOf", "oneOf", "prefixItems"] {
        if let Some(arr) = schema.get(key).and_then(|v| v.as_array()) {
            for item in arr {
                walk_properties(item, path, visit);
            }
        }
    }
    if let Some(defs) = schema.get("$defs").and_then(|d| d.as_object()) {
        for (name, def) in defs {
            walk_properties(def, &format!("$defs.{name}"), visit);
        }
    }
}

#[test]
fn full_schema_documents_every_property_from_the_reference() {
    for &chart_type in list_chart_types() {
        let schema = chart_json_schema(chart_type).unwrap();
        let mut missing = vec![];
        walk_properties(&schema, "", &mut |path, prop| {
            if !prop["description"].as_str().is_some_and(|d| !d.is_empty()) {
                missing.push(path.to_string());
            }
        });
        assert!(
            missing.is_empty(),
            "{chart_type}: properties without description: {missing:?}"
        );
        assert!(
            schema["examples"].as_array().is_some_and(|e| e.len() == 1),
            "{chart_type}: missing examples"
        );
    }
    // Defaults come from the reference, typed as JSON.
    let bar = chart_json_schema("bar").unwrap();
    assert_eq!(bar["properties"]["width"]["default"], 600);
    assert_eq!(bar["properties"]["title_align"]["default"], "center");
    assert!(
        bar["properties"]["title_font_color"]["description"]
            .as_str()
            .unwrap()
            .contains("Default: from the theme")
    );
}

#[test]
fn schema_examples_and_defaults_render() {
    use charts_rs::chart_example;
    for &chart_type in list_chart_types() {
        let example = chart_example(chart_type).unwrap().expect("example");
        let schema = chart_json_schema(chart_type).unwrap();
        assert_eq!(schema["examples"][0], example, "{chart_type}");
        // Every top-level default, set explicitly on the example, is accepted.
        let mut with_defaults = example.clone();
        for (key, prop) in schema["properties"].as_object().unwrap() {
            if let Some(default) = prop.get("default") {
                with_defaults[key] = default.clone();
            }
        }
        render_chart_from_json(chart_type, &with_defaults.to_string())
            .unwrap_or_else(|e| panic!("{chart_type}: defaults rejected: {e}"));
    }
    assert!(chart_example("nope").is_err());
}

#[test]
fn essential_schema_is_a_small_open_subset_of_the_full_one() {
    use charts_rs::{SchemaLevel, chart_json_schema_with};
    for &chart_type in list_chart_types() {
        let full = chart_json_schema_with(chart_type, SchemaLevel::Full).unwrap();
        let essential = chart_json_schema_with(chart_type, SchemaLevel::Essential).unwrap();
        assert_eq!(full, chart_json_schema(chart_type).unwrap());
        assert_eq!(essential["additionalProperties"], true, "{chart_type}");
        assert_eq!(essential["required"], full["required"], "{chart_type}");
        let props = essential["properties"].as_object().unwrap();
        assert!(props.len() >= 2, "{chart_type}: {:?}", props.keys());
        // Every essential property is a property of the full schema.
        let mut full_paths = vec![];
        walk_properties(&full, "", &mut |path, _| full_paths.push(path.to_string()));
        walk_properties(&essential, "", &mut |path, prop| {
            assert!(
                full_paths.iter().any(|p| p == path),
                "{chart_type}: {path} is not in the full schema"
            );
            assert!(prop["description"].is_string(), "{chart_type}: {path}");
        });
        let (e, f) = (essential.to_string().len(), full.to_string().len());
        assert!(e * 2 < f, "{chart_type}: essential {e} bytes vs full {f}");
        // The example is valid under the essential schema's keys + open objects
        // and renders.
        render_chart_from_json(chart_type, &essential["examples"][0].to_string())
            .unwrap_or_else(|err| panic!("{chart_type}: {err}"));
    }
    let pie = chart_json_schema_with("pie", SchemaLevel::Essential).unwrap();
    assert!(pie["properties"].get("y_axis_configs").is_none());
    assert!(pie["properties"].get("x_axis_data").is_none());
    let calendar = chart_json_schema_with("calendar", SchemaLevel::Essential).unwrap();
    assert!(calendar["properties"].get("width").is_none());
    let bar = chart_json_schema_with("bar", SchemaLevel::Essential).unwrap();
    let series = &bar["properties"]["series_list"]["items"];
    assert_eq!(series["additionalProperties"], true);
    assert!(series["properties"]["stack"].is_object());
    assert!(series["properties"].get("mark_lines").is_none());
}

#[test]
fn typed_array_schemas_have_element_constraints() {
    let cal = chart_json_schema("calendar").unwrap();
    let prefix = cal["properties"]["data"]["items"]["prefixItems"]
        .as_array()
        .unwrap();
    assert_eq!(prefix.len(), 2);
    assert_eq!(prefix[0]["format"], "date");
    assert_eq!(prefix[1]["type"], "number");

    let map = chart_json_schema("map").unwrap();
    let map_prefix = map["properties"]["data"]["items"]["prefixItems"]
        .as_array()
        .unwrap();
    assert_eq!(map_prefix.len(), 2);
    assert_eq!(map_prefix[0]["type"], "string");
    assert_eq!(map_prefix[1]["type"], "number");

    let box_plot = chart_json_schema("box_plot").unwrap();
    let box_prefix =
        box_plot["properties"]["box_series"]["items"]["properties"]["data"]["items"]["prefixItems"]
            .as_array()
            .unwrap();
    assert_eq!(box_prefix.len(), 5);

    let table = chart_json_schema("table").unwrap();
    let idx_prefix =
        table["properties"]["cell_styles"]["items"]["properties"]["indexes"]["prefixItems"]
            .as_array()
            .unwrap();
    assert_eq!(idx_prefix.len(), 2);
}

#[test]
fn all_asset_json_fixtures_render_under_validation() {
    let asset_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("asset");
    let mut tested = 0usize;
    for entry in std::fs::read_dir(&asset_dir).unwrap() {
        let dir = entry.unwrap().path();
        if !dir.is_dir() {
            continue;
        }
        let Some(dir_name) = dir.file_name().and_then(|n| n.to_str()) else {
            continue;
        };
        let Some(chart_type) = dir_name.strip_suffix("_chart") else {
            continue;
        };
        #[cfg(not(feature = "mermaid"))]
        if chart_type == "mermaid" {
            continue;
        }
        for file_entry in std::fs::read_dir(&dir).unwrap() {
            let path = file_entry.unwrap().path();
            let Some(fname) = path.file_name().and_then(|n| n.to_str()) else {
                continue;
            };
            if !fname.ends_with(".json") || fname.ends_with(".geo.json") {
                continue;
            }
            let json = std::fs::read_to_string(&path).unwrap();
            let svg = render_chart_from_json(chart_type, &json)
                .unwrap_or_else(|e| panic!("{}: render failed: {e}", path.display()));
            assert!(svg.starts_with("<svg"), "{}", path.display());
            tested += 1;
        }
    }
    let min_expected = if cfg!(feature = "mermaid") { 60 } else { 40 };
    assert!(
        tested >= min_expected,
        "expected >={min_expected} fixtures, tested {tested}"
    );
}

#[test]
fn validate_chart_json_collects_all_issues_and_error_codes() {
    use charts_rs::validate_chart_json;

    // Unknown chart_type returns Err(Error::Params) with stable code.
    let err = validate_chart_json("nope", &serde_json::json!({})).unwrap_err();
    assert_eq!(err.code(), "params");
    assert_eq!(
        Error::Json {
            source: serde_json::from_str::<serde_json::Value>("{").unwrap_err()
        }
        .code(),
        "json"
    );
    assert_eq!(
        Error::FontNotFound { name: "x".into() }.code(),
        "font_not_found"
    );
    assert_eq!(
        Error::ParseFont {
            message: "x".into()
        }
        .code(),
        "parse_font"
    );
    #[cfg(feature = "raster")]
    {
        assert_eq!(
            Error::Size {
                width: 0,
                height: 0
            }
            .code(),
            "image_size"
        );
        assert_eq!(Error::Raw { size: 0 }.code(), "image_raw");
    }
    #[cfg(feature = "mermaid")]
    assert_eq!(
        Error::Mermaid {
            line: Some(1),
            message: "bad".into()
        }
        .code(),
        "mermaid"
    );

    // Every valid chart_example passes validation cleanly.
    for &chart_type in list_chart_types() {
        let ex = charts_rs::chart_example(chart_type).unwrap().unwrap();
        let issues = validate_chart_json(chart_type, &ex).unwrap();
        assert!(issues.is_empty(), "{chart_type}: {issues:?}");
    }

    // Multiple simultaneous issues are all reported with structured metadata.
    let doc = serde_json::json!({
        "theme": "nope",
        "title_font_color": "red",
        "series_list": [{"name": "a", "lable_show": true}]
    });
    let issues = validate_chart_json("bar", &doc).unwrap();
    assert_eq!(issues.len(), 3, "{issues:?}");
    // serde_json::Map iterates keys in sorted order (`series_list`, `theme`, `title_font_color`).
    assert_eq!(issues[0].code, "unknown_field");
    assert_eq!(issues[0].path, "series_list[0].lable_show");
    assert_eq!(
        issues[0].suggestion.as_deref(),
        Some("series_list[0].label_show")
    );
    assert_eq!(issues[1].code, "unknown_theme");
    assert_eq!(issues[1].path, "theme");
    assert!(
        issues[1]
            .expected
            .as_ref()
            .is_some_and(|e| e.contains("dark"))
    );
    assert_eq!(issues[2].code, "invalid_color");
    assert_eq!(issues[2].path, "title_font_color");
    let serialized = serde_json::to_value(&issues).unwrap();
    assert_eq!(serialized.as_array().unwrap().len(), 3);

    // Unknown theme message is deterministic across repeated calls.
    let m1 = charts_rs::BarChart::from_json(r#"{"theme":"nope"}"#)
        .unwrap_err()
        .to_string();
    let m2 = charts_rs::BarChart::from_json(r#"{"theme":"nope"}"#)
        .unwrap_err()
        .to_string();
    assert_eq!(m1, m2);

    // Multi child validation reports paths under child_charts[i].
    let multi_issues = validate_chart_json(
        "multi",
        &serde_json::json!({
            "child_charts": [
                {"type": "bar", "width": -5},
                {"type": "unknown_kind"}
            ]
        }),
    )
    .unwrap();
    assert_eq!(multi_issues.len(), 2, "{multi_issues:?}");
    assert_eq!(multi_issues[0].code, "out_of_range");
    assert_eq!(multi_issues[0].path, "child_charts[0].width");
    assert_eq!(multi_issues[1].code, "invalid_enum");
    assert_eq!(multi_issues[1].path, "child_charts[1].type");
}

#[cfg(feature = "mermaid")]
#[test]
fn validate_chart_json_checks_mermaid_required_and_syntax() {
    use charts_rs::validate_chart_json;

    let missing = validate_chart_json("mermaid", &serde_json::json!({})).unwrap();
    assert_eq!(missing.len(), 1);
    assert_eq!(missing[0].code, "missing_required");
    assert_eq!(missing[0].path, "source");

    let syntax = validate_chart_json(
        "mermaid",
        &serde_json::json!({"source": "gitGraph\n  commit"}),
    )
    .unwrap();
    assert_eq!(syntax.len(), 1);
    assert_eq!(syntax[0].code, "mermaid_syntax");
    assert_eq!(syntax[0].path, "source");
    assert!(syntax[0].line.is_some());
}

#[test]
fn chart_catalog_and_render_chart_from_value() {
    use charts_rs::{chart_catalog, render_chart_from_value};

    let catalog = chart_catalog();
    let types = list_chart_types();
    assert_eq!(catalog.len(), types.len());

    let serialized = serde_json::to_value(&catalog).unwrap();
    assert_eq!(serialized.as_array().unwrap().len(), types.len());

    for (info, &expected_name) in catalog.iter().zip(types.iter()) {
        assert_eq!(info.name, expected_name);
        assert!(!info.title.is_empty(), "{expected_name}");
        assert!(!info.description.is_empty(), "{expected_name}");
        let example = info
            .example
            .as_ref()
            .unwrap_or_else(|| panic!("{expected_name}: missing example"));
        let svg_from_value = render_chart_from_value(info.name, example).unwrap();
        let svg_from_json = render_chart_from_json(info.name, &example.to_string()).unwrap();
        assert_eq!(svg_from_value, svg_from_json, "{expected_name}");
        assert!(svg_from_value.starts_with("<svg"), "{expected_name}");
    }

    assert!(render_chart_from_value("nope", &serde_json::json!({})).is_err());
}

#[test]
fn render_limits_enforce_json_canvas_points_and_mermaid_bounds() {
    use charts_rs::{
        RenderLimits, render_chart_from_json_with_limits, render_chart_from_value_with_limits,
        validate_chart_json_with_limits,
    };

    let mut limits = RenderLimits::default();
    assert_eq!(limits.max_json_bytes, 1024 * 1024);
    assert_eq!(limits.max_canvas_side, 8192.0);
    assert_eq!(limits.max_data_points, 100_000);
    assert_eq!(limits.max_mermaid_source, 64 * 1024);

    // 1. max_json_bytes
    limits.max_json_bytes = 20;
    let bar = serde_json::json!({
        "x_axis_data": ["A", "B"],
        "series_list": [{"name": "S", "data": [1, 2]}]
    });
    let err = render_chart_from_json_with_limits("bar", &bar.to_string(), &limits).unwrap_err();
    assert!(matches!(err, Error::Params { .. }), "{err:?}");
    assert!(err.to_string().contains("max_json_bytes"), "{err}");
    let err_v = render_chart_from_value_with_limits("bar", &bar, &limits).unwrap_err();
    assert!(err_v.to_string().contains("max_json_bytes"), "{err_v}");

    // 2. max_canvas_side (root and multi child)
    let mut side_limits = RenderLimits::default();
    side_limits.max_canvas_side = 500.0;
    let wide = serde_json::json!({
        "width": 600,
        "x_axis_data": ["A"],
        "series_list": [{"name": "S", "data": [1]}]
    });
    let issues = validate_chart_json_with_limits("bar", &wide, &side_limits).unwrap();
    assert_eq!(issues.len(), 1);
    assert_eq!(issues[0].code, "limit_exceeded");
    assert_eq!(issues[0].path, "width");
    assert!(render_chart_from_value_with_limits("bar", &wide, &side_limits).is_err());

    let multi_wide = serde_json::json!({
        "child_charts": [
            {"type": "bar", "height": 900, "x_axis_data": ["A"], "series_list": [{"name": "S", "data": [1]}]}
        ]
    });
    let multi_issues = validate_chart_json_with_limits("multi", &multi_wide, &side_limits).unwrap();
    assert_eq!(multi_issues.len(), 1);
    assert_eq!(multi_issues[0].code, "limit_exceeded");
    assert_eq!(multi_issues[0].path, "child_charts[0].height");
    assert!(render_chart_from_value_with_limits("multi", &multi_wide, &side_limits).is_err());

    // 3. max_data_points
    let mut point_limits = RenderLimits::default();
    point_limits.max_data_points = 3;
    let many = serde_json::json!({
        "x_axis_data": ["A", "B", "C"],
        "series_list": [{"name": "S", "data": [1, 2, 3]}]
    });
    let point_issues = validate_chart_json_with_limits("bar", &many, &point_limits).unwrap();
    assert_eq!(point_issues.len(), 1);
    assert_eq!(point_issues[0].code, "limit_exceeded");
    assert!(render_chart_from_value_with_limits("bar", &many, &point_limits).is_err());

    // Within limits succeeds.
    assert!(render_chart_from_value_with_limits("bar", &bar, &RenderLimits::default()).is_ok());

    // 4. max_mermaid_source
    #[cfg(feature = "mermaid")]
    {
        let mut mermaid_limits = RenderLimits::default();
        mermaid_limits.max_mermaid_source = 16;
        let doc = serde_json::json!({"source": "flowchart LR\n  A --> B --> C"});
        let m_issues = validate_chart_json_with_limits("mermaid", &doc, &mermaid_limits).unwrap();
        assert_eq!(m_issues.len(), 1);
        assert_eq!(m_issues[0].code, "limit_exceeded");
        assert_eq!(m_issues[0].path, "source");
        assert!(render_chart_from_value_with_limits("mermaid", &doc, &mermaid_limits).is_err());

        let multi_doc = serde_json::json!({
            "child_charts": [{"type": "mermaid", "source": "flowchart LR\n  A --> B --> C"}]
        });
        let mm_issues =
            validate_chart_json_with_limits("multi", &multi_doc, &mermaid_limits).unwrap();
        assert_eq!(mm_issues.len(), 1);
        assert_eq!(mm_issues[0].code, "limit_exceeded");
        assert_eq!(mm_issues[0].path, "child_charts[0].source");
    }
}

#[test]
fn geo_json_registry_supports_named_maps_in_map_and_lines() {
    use charts_rs::{list_geo_json, register_geo_json, validate_chart_json};

    assert!(register_geo_json("   ", serde_json::json!({})).is_err());
    assert!(register_geo_json("bad_value", serde_json::json!([])).is_err());

    let geo = serde_json::json!({
        "type": "FeatureCollection",
        "features": [{
            "type": "Feature",
            "properties": {"name": "Island"},
            "geometry": {
                "type": "Polygon",
                "coordinates": [[[100, 20], [105, 20], [105, 25], [100, 25], [100, 20]]]
            }
        }]
    });
    register_geo_json("mcp_test_island", geo).unwrap();
    assert!(list_geo_json().contains(&"mcp_test_island".to_string()));

    let map_doc = serde_json::json!({
        "geo_json": "mcp_test_island",
        "data": [["Island", 42]]
    });
    assert!(validate_chart_json("map", &map_doc).unwrap().is_empty());
    let map_svg = render_chart_from_json("map", &map_doc.to_string()).unwrap();
    assert!(map_svg.contains("Island"));

    let lines_doc = serde_json::json!({
        "geo_json": "mcp_test_island",
        "lines": [{"from": [101, 21], "to": [104, 24], "value": 5}]
    });
    assert!(validate_chart_json("lines", &lines_doc).unwrap().is_empty());
    let lines_svg = render_chart_from_json("lines", &lines_doc.to_string()).unwrap();
    assert!(lines_svg.contains("<path"));

    let unknown_doc = serde_json::json!({
        "geo_json": "mcp_unregistered_map_xyz",
        "data": [["Island", 42]]
    });
    let issues = validate_chart_json("map", &unknown_doc).unwrap();
    assert_eq!(issues.len(), 1);
    assert_eq!(issues[0].code, "unknown_map");
    assert_eq!(issues[0].path, "geo_json");
    assert!(render_chart_from_json("map", &unknown_doc.to_string()).is_err());
}

#[test]
fn llms_full_txt_is_in_sync() {
    let expected = format!(
        "{}\n---\n\n{}\n---\n\n{}",
        include_str!("../llms.txt"),
        include_str!("../docs/guide.md"),
        include_str!("../docs/json.md"),
    );
    let actual = include_str!("../llms-full.txt");
    assert!(
        actual == expected,
        "llms-full.txt is out of date; run `make llms` to regenerate it"
    );
}

#[test]
fn llms_txt_minimal_examples_validate_and_render() {
    use charts_rs::{render_chart_from_value, validate_chart_json};

    let llms = include_str!("../llms.txt");
    let section = llms
        .split("## Minimal JSON Examples\n")
        .nth(1)
        .expect("missing Minimal JSON Examples section in llms.txt")
        .split("\n## ")
        .next()
        .unwrap();

    let mut parsed = Vec::new();
    for line in section.lines() {
        if let Some(rest) = line.strip_prefix("- `")
            && let Some((chart_type, after)) = rest.split_once("`: `")
            && let Some(json_str) = after.strip_suffix('`')
        {
            let value: serde_json::Value = serde_json::from_str(json_str)
                .unwrap_or_else(|e| panic!("{chart_type}: invalid JSON in llms.txt: {e}"));
            parsed.push(chart_type);
            #[cfg(not(feature = "mermaid"))]
            if chart_type == "mermaid" {
                continue;
            }
            let issues = validate_chart_json(chart_type, &value).unwrap();
            assert!(issues.is_empty(), "{chart_type}: {issues:?}");
            let svg = render_chart_from_value(chart_type, &value)
                .unwrap_or_else(|e| panic!("{chart_type}: render failed: {e}"));
            assert!(svg.starts_with("<svg"), "{chart_type}");
        }
    }

    assert_eq!(parsed.len(), 30, "expected 30 minimal examples in llms.txt");
    for &t in list_chart_types() {
        assert!(parsed.contains(&t), "missing minimal example for `{t}`");
    }
}
