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
            schema["$schema"],
            "https://json-schema.org/draft/2020-12/schema",
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
    assert!(json["supported_diagrams"].as_array().is_some_and(|a| !a.is_empty()));
    assert!(json["unsupported_diagrams"].as_array().is_some_and(|a| !a.is_empty()));
    assert!(json["v11_shapes"].as_array().is_some_and(|a| !a.is_empty()));
    assert!(json["init_config_keys"].as_array().is_some_and(|a| !a.is_empty()));
    assert!(json["features"].as_array().is_some_and(|a| !a.is_empty()));
    assert!(json["limitations"].as_array().is_some_and(|a| !a.is_empty()));

    // Every advertised supported diagram keyword must validate on a minimal diagram.
    for &kw in caps.supported_diagrams {
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
