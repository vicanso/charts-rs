#![cfg(feature = "mermaid")]
mod common;

use charts_rs::{
    BarChart, Chart, ChildChart, MermaidChart, MultiChart, THEME_DARK, THEME_GRAFANA,
    THEME_SHADCN, mermaid_to_svg, mermaid_to_svg_with_theme,
};

fn assert_valid_svg(svg: &str) {
    assert!(svg.starts_with("<svg"), "expected <svg root: {svg}");
    assert!(svg.ends_with("</svg>"), "expected </svg> end: {svg}");
    assert!(!svg.contains("NaN"), "SVG contains NaN: {svg}");
    assert!(!svg.contains("inf"), "SVG contains inf: {svg}");
    assert!(
        !svg.contains("<foreignObject"),
        "pure SVG must not emit foreignObject"
    );
}

#[test]
fn flowchart_directions_shapes_and_edges() {
    for dir in ["TD", "TB", "BT", "LR", "RL"] {
        let src = format!(
            "flowchart {dir}\n\
             A[Rect] --> B(Rounded)\n\
             B --> C([Stadium])\n\
             C --> D[[Subroutine]]\n\
             D --> E[(Database)]\n\
             E --> F((Circle))\n\
             F --> G(((Double)))\n\
             G --> H{{Diamond}}\n\
             H -->|Yes| I{{{{Hexagon}}}}\n\
             H -- No --> J[/Parallelogram/]\n\
             J -.-> K[\\Parallelogram Alt\\]\n\
             K ==> L[/Trapezoid\\]\n\
             L <--> M[\\Trapezoid Alt/]\n\
             M o--o N>Asymmetric]"
        );
        let svg = mermaid_to_svg(&src).unwrap();
        assert_valid_svg(&svg);
        assert!(svg.contains("Database"));
        assert!(svg.contains("Yes"));
        assert!(svg.contains("No"));
    }
}

#[test]
fn flowchart_cycles_self_loops_chains_and_subgraphs() {
    let src = r#"
---
title: "Deployment Pipeline"
---
flowchart TD
    Dev[Developer] -->|push| CI
    subgraph CI [CI Pipeline]
        Build[Build] --> Test[Test] --> Lint[Lint]
        Test -. retry .-> Build
        Lint --> Lint
    end
    CI --> Prod[(Production)]
    style Prod fill:#22c55e,stroke:#14532d,stroke-width:2px,color:#ffffff
    classDef warn fill:#fef3c7,stroke:#d97706
    class Build,Test warn
    click Dev "https://example.com" "Ignored"
    accTitle: Ignored title
"#;
    let svg = mermaid_to_svg(src).unwrap();
    assert_valid_svg(&svg);
    assert!(svg.contains("Deployment Pipeline"));
    assert!(svg.contains("CI Pipeline"));
    assert!(svg.contains("#22C55E"));
    assert!(svg.contains("#FEF3C7"));
}

#[test]
fn flowchart_inline_styles_entities_multiline_cjk_and_semicolons() {
    let src = r#"
%%{init: {'theme': 'dark'}}%%
flowchart LR;
    A["<b>网关</b><br/>a &lt; b &amp; #quot;ok#quot;"]:::hot & B[服务 B] --> C[存储] ~~~ D[占位];
    classDef hot fill:coral,stroke:navy,stroke-width:3px;
    linkStyle 0 stroke:#ff0000,stroke-width:4px;
"#;
    let svg = mermaid_to_svg(src).unwrap();
    assert_valid_svg(&svg);
    assert!(svg.contains("网关"));
    assert!(svg.contains("a &lt; b &amp; \"ok\""));
    assert!(svg.contains("#FF7F50")); // coral
    assert!(svg.contains("#FF0000")); // linkStyle
    assert!(svg.contains("font-weight=\"bold\""));
}

#[test]
fn sequence_diagram_full_features() {
    let src = r#"
sequenceDiagram
    autonumber
    title Checkout Sequence
    box rgba(100, 149, 237, 0.15) Frontend
        actor U as User
        participant Web
    end
    participant API
    participant Pay as Payment

    U->>+Web: Click Pay
    Web->>+API: POST /checkout
    API->>API: Validate cart
    Note right of API: Idempotency<br/>Check
    alt Stock available
        API->>+Pay: Charge card
        Pay-->>-API: Success
        Note over API,Pay: Transaction committed
    else Out of stock
        API--xWeb: 409 Conflict
    end
    opt Receipt enabled
        API--)Web: Async email
    end
    loop Retry x2
        Web->API: Poll status
    end
    par Audit
        API->>Pay: Log
    and Metrics
        API->>Web: Emit metric
    end
    rect rgb(240, 240, 240)
        Web-->>-U: Show result
    end
    destroy Pay
"#;
    let svg = mermaid_to_svg(src).unwrap();
    assert_valid_svg(&svg);
    assert!(svg.contains("Checkout Sequence"));
    assert!(svg.contains("1. Click Pay"));
    assert!(svg.contains("Transaction committed"));
    assert!(svg.contains("alt"));
    assert!(svg.contains("Out of stock"));
    assert!(svg.contains("Frontend"));
    assert!(
        svg.contains("fill=\"none\""),
        "control-flow frames must not omit fill"
    );
}

#[test]
fn adapters_pie_gantt_sankey_xychart() {
    let pie_svg = mermaid_to_svg(
        "pie showData\n  title Languages\n  \"Rust\" : 65\n  \"Go\" : 25\n  \"Python\" : 10",
    )
    .unwrap();
    assert_valid_svg(&pie_svg);
    assert!(pie_svg.contains("Languages"));
    assert!(pie_svg.contains("Rust"));
    assert!(pie_svg.contains("Rust: 65 (65%)"), "{pie_svg}");

    let gantt_svg = mermaid_to_svg(
        r#"
gantt
    title Release Plan
    dateFormat YYYY-MM-DD
    todayMarker 2026-03-10
    section Design
    Spec       :done, s1, 2026-03-01, 5d
    Review     :active, r1, after s1, 3d
    section Build
    Implement  :crit, impl1, after r1, 2026-03-18
    Launch     :milestone, m1, after impl1, 0d
"#,
    )
    .unwrap();
    assert_valid_svg(&gantt_svg);
    assert!(gantt_svg.contains("Release Plan"));
    assert!(gantt_svg.contains("Design"));
    assert!(gantt_svg.contains("Build"));

    let sankey_svg = mermaid_to_svg(
        "sankey-beta\nAgricultural 'waste',Bio-conversion,124.7\nBio-conversion,Liquid,0.597\nBio-conversion,Losses,26.862\n\"Bio, Gas\",Electricity,45.0",
    )
    .unwrap();
    assert_valid_svg(&sankey_svg);
    assert!(sankey_svg.contains("Bio-conversion"));
    assert!(sankey_svg.contains("Bio, Gas"));

    let xy_svg = mermaid_to_svg(
        r#"
xychart-beta
    title "Sales Revenue"
    x-axis [jan, feb, mar, apr]
    y-axis "Revenue" 0 --> 12000
    bar [5000, 6000, 7500, 8200]
    line [4500, 6500, 7000, 9000]
"#,
    )
    .unwrap();
    assert_valid_svg(&xy_svg);
    assert!(xy_svg.contains("Sales Revenue"));

    let xy_horiz_svg =
        mermaid_to_svg("xychart-beta horizontal\n  x-axis [A, B, C]\n  bar [10, 20, 15]").unwrap();
    assert_valid_svg(&xy_horiz_svg);
}

#[test]
fn sizing_themes_compact_trait_and_multichart() {
    let src = "flowchart TD\n  A[Start] --> B[End]";

    // Auto-size vs explicit width/height scaling via viewBox.
    let auto_svg = mermaid_to_svg(src).unwrap();
    assert_valid_svg(&auto_svg);

    let mut scaled = MermaidChart::new(src);
    scaled.width = 900.0;
    let w_svg = scaled.svg().unwrap();
    assert_valid_svg(&w_svg);
    assert!(w_svg.contains("width=\"900\""), "{w_svg}");

    scaled.height = 500.0;
    let wh_svg = scaled.svg().unwrap();
    assert!(wh_svg.contains("width=\"900\" height=\"500\""), "{wh_svg}");

    // Built-in themes & compact output.
    for theme in [THEME_DARK, THEME_GRAFANA, THEME_SHADCN] {
        let t_svg = mermaid_to_svg_with_theme(src, theme).unwrap();
        assert_valid_svg(&t_svg);
    }

    let mut compact = MermaidChart::new(src);
    compact.compact = true;
    let compact_svg = compact.svg().unwrap();
    assert_valid_svg(&compact_svg);
    assert!(compact_svg.len() < auto_svg.len());

    // Chart trait object compatibility.
    let dyn_chart: Box<dyn Chart> = Box::new(
        MermaidChart::from_json(r#"{"source": "flowchart LR\n  X --> Y", "theme": "dark"}"#)
            .unwrap(),
    );
    assert_valid_svg(&dyn_chart.svg().unwrap());

    // MultiChart integration (Rust + JSON).
    let mut mc = MultiChart::new();
    mc.add(ChildChart::Bar(
        BarChart::new(
            vec![("S", vec![1.0, 2.0]).into()],
            vec!["a".into(), "b".into()],
        ),
        None,
    ));
    mc.add(ChildChart::Mermaid(MermaidChart::new(src), None));
    let mc_svg = mc.svg().unwrap();
    assert_valid_svg(&mc_svg);
    assert!(mc_svg.matches("<svg").count() >= 3);

    let mc_json = MultiChart::from_json(
        r#"{
            "theme": "grafana",
            "child_charts": [
                {"type": "mermaid", "source": "pie\n  \"A\": 30\n  \"B\": 70"},
                {"type": "mermaid", "source": "sequenceDiagram\n  A->>B: ping"}
            ]
        }"#,
    )
    .unwrap();
    assert_valid_svg(&mc_json.svg().unwrap());
}

#[test]
fn errors_report_line_numbers() {
    let empty_err = mermaid_to_svg("   \n  ").unwrap_err().to_string();
    assert!(empty_err.contains("empty"), "{empty_err}");

    let unsupported = mermaid_to_svg("\n\ngitGraph\n  commit")
        .unwrap_err()
        .to_string();
    assert!(unsupported.contains("line 3"), "{unsupported}");

    let unclosed_sg = mermaid_to_svg("flowchart TD\n  subgraph G1\n    A --> B")
        .unwrap_err()
        .to_string();
    assert!(unclosed_sg.contains("line 2"), "{unclosed_sg}");

    let unclosed_alt = mermaid_to_svg("sequenceDiagram\n  alt ok\n    A->>B: hi")
        .unwrap_err()
        .to_string();
    assert!(unclosed_alt.contains("line 2"), "{unclosed_alt}");

    let neg_pie = mermaid_to_svg("pie\n  \"Bad\" : -5")
        .unwrap_err()
        .to_string();
    assert!(neg_pie.contains("line 2"), "{neg_pie}");

    let bad_after = mermaid_to_svg("gantt\n  dateFormat YYYY-MM-DD\n  A :after missing, 3d")
        .unwrap_err()
        .to_string();
    assert!(bad_after.contains("line 3"), "{bad_after}");

    let bad_theme = MermaidChart::from_json(r#"{"theme": "unknown_theme"}"#)
        .unwrap_err()
        .to_string();
    assert!(bad_theme.contains("unknown theme"), "{bad_theme}");

    let typo_json = MermaidChart::from_json(r#"{"sorce": "pie"}"#)
        .unwrap_err()
        .to_string();
    assert!(typo_json.contains("did you mean `source`"), "{typo_json}");

    let unclosed_state = mermaid_to_svg("stateDiagram-v2\n  state Active {\n    [*] --> Run")
        .unwrap_err()
        .to_string();
    assert!(unclosed_state.contains("line 2"), "{unclosed_state}");

    let unclosed_class = mermaid_to_svg("classDiagram\n  class Foo {\n    +int x")
        .unwrap_err()
        .to_string();
    assert!(unclosed_class.contains("line 2"), "{unclosed_class}");

    let unclosed_er = mermaid_to_svg("erDiagram\n  USER {\n    int id PK")
        .unwrap_err()
        .to_string();
    assert!(unclosed_er.contains("line 2"), "{unclosed_er}");

    let multi_root_mm = mermaid_to_svg("mindmap\n  RootA\n  RootB")
        .unwrap_err()
        .to_string();
    assert!(multi_root_mm.contains("line 3"), "{multi_root_mm}");

    let bad_quadrant = mermaid_to_svg("quadrantChart\n  Bad: [1.5, 0.2]")
        .unwrap_err()
        .to_string();
    assert!(bad_quadrant.contains("line 2"), "{bad_quadrant}");

    let bad_radar = mermaid_to_svg("radar-beta\n  axis a, b, c\n  curve x{1, 2}")
        .unwrap_err()
        .to_string();
    assert!(bad_radar.contains("expected 3"), "{bad_radar}");

    let bad_treemap = mermaid_to_svg("treemap-beta\n  \"A\"\n    \"B\": abc")
        .unwrap_err()
        .to_string();
    assert!(bad_treemap.contains("line 3"), "{bad_treemap}");
}

#[test]
fn state_diagram_full_features() {
    let src = r#"
stateDiagram-v2
    direction LR
    [*] --> Idle
    Idle --> Active : start
    state Decision <<choice>>
    Active --> Decision
    Decision --> Stopped : stop
    note left of Idle : Initial state
    Stopped --> [*]
"#;
    let out = mermaid_to_svg(src).unwrap();
    assert_valid_svg(&out);
    assert!(out.contains("Active"));
    assert!(out.contains("Initial state"));
}

#[test]
fn class_diagram_full_features() {
    let src = r#"
classDiagram-v2
    class Base~T~ {
        <<abstract>>
        #T id
        +validate()* bool
        +version()$ String
    }
    class Impl {
        +int id
        +validate() bool
    }
    Base <|-- Impl : extends
"#;
    let out = mermaid_to_svg(src).unwrap();
    assert_valid_svg(&out);
    assert!(out.contains("Base&lt;T&gt;"));
    assert!(out.contains("&lt;&lt;abstract&gt;&gt;"));
    assert!(out.contains("+validate() bool"));
    assert!(out.contains("+version() String"));
}

#[test]
fn er_diagram_full_features() {
    let src = r#"
erDiagram
    ACCOUNT["Bank Account"] {
        uuid id PK
        varchar code UK "account code"
    }
    LEDGER {
        uuid id PK
        uuid account_id PK,FK
    }
    ACCOUNT only one to zero or more LEDGER : records
"#;
    let out = mermaid_to_svg(src).unwrap();
    assert_valid_svg(&out);
    assert!(out.contains("Bank Account"));
    assert!(out.contains("account code"));
    assert!(out.contains("PK, FK"));
}

#[test]
fn mindmap_full_features() {
    let src = r#"
mindmap
  root((Center))
    )CloudSide(
      [Square]
      ((Circle))
    {{HexSide}}
      ))BangAlert((
      (Rounded)
"#;
    let out = mermaid_to_svg(src).unwrap();
    assert_valid_svg(&out);
    assert!(out.contains("Center"));
    assert!(out.contains("CloudSide"));
    assert!(out.contains("BangAlert"));
}

#[test]
fn quadrant_chart_full_features() {
    let src = r#"
quadrantChart
  title Priority
  x-axis Low --> High
  y-axis Cold --> Hot
  quadrant-1 Focus
  quadrant-2 Quick
  classDef hot color:#ef4444,radius:9
  FeatureA:::hot: [0.8, 0.85]
  FeatureB: [0.25, 0.7]
"#;
    let out = mermaid_to_svg(src).unwrap();
    assert_valid_svg(&out);
    assert!(out.contains("Priority"));
    assert!(out.contains("FeatureA"));
    assert!(out.contains("rotate(-90"));
}

#[test]
fn radar_diagram_full_features() {
    let src = r#"
radar-beta
  title Metrics
  axis a["Speed"], b["Memory"], c["DX"], d["Safety"]
  curve s1["Rust"]{95, 92, 82, 98}
  curve s2["Go"]{d: 85, a: 84, b: 86, c: 90}
  graticule circle
  min 0
  max 100
  ticks 5
  showLegend true
"#;
    let out = mermaid_to_svg(src).unwrap();
    assert_valid_svg(&out);
    assert!(out.contains("Metrics"));
    assert!(out.contains("Rust"));
    assert!(out.contains("Safety"));
}

#[test]
fn treemap_diagram_full_features() {
    let src = r#"
treemap-beta
  title Storage
  "Primary"
    "NVMe": 500
    "SSD": 300
  "Archive"
    "Cold": 200:::blue
  classDef blue fill:#0ea5e9
"#;
    let out = mermaid_to_svg(src).unwrap();
    assert_valid_svg(&out);
    assert!(out.contains("Storage"));
    assert!(out.contains("NVMe"));
    assert!(out.contains("Cold"));
}

#[cfg(feature = "png")]
#[test]
fn rasterizes_mermaid_diagrams_to_png() {
    use charts_rs::svg_to_png;

    let samples = [
        "flowchart TD\n  A[Start] --> B{Check}\n  B -->|Yes| C[(DB)]\n  B -.-> D([End])",
        "sequenceDiagram\n  actor U\n  participant S\n  U->>+S: Request\n  Note over S: Process\n  S-->>-U: Response",
        "pie showData\n  \"Alpha\" : 40\n  \"Beta\" : 60",
        "stateDiagram-v2\n  [*] --> Ready\n  Ready --> Done : finish\n  Done --> [*]",
        "classDiagram\n  class A {\n    +int x\n  }\n  class B\n  A <|-- B",
        "erDiagram\n  A ||--o{ B : has",
        "mindmap\n  root((Hub))\n    RightBranch\n    LeftBranch",
        "quadrantChart\n  x-axis Low --> High\n  y-axis Low --> High\n  P: [0.4, 0.7]",
        "radar-beta\n  axis a, b, c\n  curve s{10, 20, 30}",
        "treemap-beta\n  \"A\"\n    \"B\": 10",
    ];
    for src in samples {
        let svg = mermaid_to_svg(src).unwrap();
        let png = svg_to_png(&svg).expect("resvg rasterization must succeed");
        assert!(png.starts_with(b"\x89PNG\r\n\x1a\n"));
        assert!(png.len() > 200);
    }
    let _ = Error::Params {
        message: String::new(),
    };
}

#[test]
fn mermaid_snapshots() {
    let flowchart =
        MermaidChart::from_json(include_str!("../asset/mermaid_chart/flowchart.json")).unwrap();
    common::assert_snapshot!("mermaid_chart/flowchart_json.svg", flowchart.svg().unwrap());

    let sequence =
        MermaidChart::from_json(include_str!("../asset/mermaid_chart/sequence.json")).unwrap();
    common::assert_snapshot!("mermaid_chart/sequence_json.svg", sequence.svg().unwrap());

    let pie = MermaidChart::from_json(include_str!("../asset/mermaid_chart/pie.json")).unwrap();
    common::assert_snapshot!("mermaid_chart/pie_json.svg", pie.svg().unwrap());

    let gantt = MermaidChart::from_json(include_str!("../asset/mermaid_chart/gantt.json")).unwrap();
    common::assert_snapshot!("mermaid_chart/gantt_json.svg", gantt.svg().unwrap());

    let sankey =
        MermaidChart::from_json(include_str!("../asset/mermaid_chart/sankey.json")).unwrap();
    common::assert_snapshot!("mermaid_chart/sankey_json.svg", sankey.svg().unwrap());

    let xychart =
        MermaidChart::from_json(include_str!("../asset/mermaid_chart/xychart.json")).unwrap();
    common::assert_snapshot!("mermaid_chart/xychart_json.svg", xychart.svg().unwrap());

    let state = MermaidChart::from_json(include_str!("../asset/mermaid_chart/state.json")).unwrap();
    common::assert_snapshot!("mermaid_chart/state_json.svg", state.svg().unwrap());

    let class = MermaidChart::from_json(include_str!("../asset/mermaid_chart/class.json")).unwrap();
    common::assert_snapshot!("mermaid_chart/class_json.svg", class.svg().unwrap());

    let er = MermaidChart::from_json(include_str!("../asset/mermaid_chart/er.json")).unwrap();
    common::assert_snapshot!("mermaid_chart/er_json.svg", er.svg().unwrap());

    let mindmap =
        MermaidChart::from_json(include_str!("../asset/mermaid_chart/mindmap.json")).unwrap();
    common::assert_snapshot!("mermaid_chart/mindmap_json.svg", mindmap.svg().unwrap());

    let quadrant =
        MermaidChart::from_json(include_str!("../asset/mermaid_chart/quadrant.json")).unwrap();
    common::assert_snapshot!("mermaid_chart/quadrant_json.svg", quadrant.svg().unwrap());

    let radar = MermaidChart::from_json(include_str!("../asset/mermaid_chart/radar.json")).unwrap();
    common::assert_snapshot!("mermaid_chart/radar_json.svg", radar.svg().unwrap());

    let treemap =
        MermaidChart::from_json(include_str!("../asset/mermaid_chart/treemap.json")).unwrap();
    common::assert_snapshot!("mermaid_chart/treemap_json.svg", treemap.svg().unwrap());
}

#[test]
fn m02_sankey_cycle_and_self_loop_rejected() {
    let cycle = "sankey-beta\nA,B,10\nB,C,5\nC,A,2\nB,D,5";
    let err = mermaid_to_svg(cycle).unwrap_err().to_string();
    assert!(err.contains("line 4"), "expected line 4, got: {err}");
    assert!(
        err.contains("circular link") && err.contains("C -> A"),
        "expected circular link C -> A, got: {err}"
    );

    let self_loop = "sankey-beta\nA,B,10\nB,B,5";
    let err_self = mermaid_to_svg(self_loop).unwrap_err().to_string();
    assert!(
        err_self.contains("line 3"),
        "expected line 3, got: {err_self}"
    );
    assert!(
        err_self.contains("circular link") && err_self.contains("B -> B"),
        "expected circular link B -> B, got: {err_self}"
    );

    // Defensive guard in core SankeyChart: a cyclic graph must not produce collapsed ribbons.
    let chart = charts_rs::SankeyChart::new(
        vec![],
        vec![("A", "B", 10.0).into(), ("B", "C", 5.0).into(), ("C", "A", 2.0).into()],
    );
    let svg = chart.svg().unwrap();
    assert_valid_svg(&svg);
    assert!(
        !svg.contains("<polygon"),
        "cyclic SankeyChart should not draw collapsed ribbons: {svg}"
    );
}

#[test]
fn m01_xychart_horizontal_line_series() {
    let src = r#"
xychart-beta horizontal
  title "Sales"
  x-axis "Month" [jan, feb, mar, apr]
  y-axis "Units" 0 --> 100
  bar [20, 50, 75, 90]
  line [10, 40, 60, 80]
"#;
    let svg = mermaid_to_svg(src).unwrap();
    assert_valid_svg(&svg);
    assert!(svg.contains("Sales"));
    assert!(svg.contains("Month"));
    assert!(svg.contains("Units"));
    // Bar series draws 4 horizontal bars (plus background rect = 5 rects total,
    // NOT 9 rects which would happen if the line series were drawn as bars).
    let bar_rect_count = svg.matches("<rect").count();
    assert_eq!(
        bar_rect_count, 5,
        "expected 1 background + 4 bar rects (line must not render as bars), got {bar_rect_count}"
    );
    // Line series draws a <path> polyline across the 4 category rows.
    assert!(
        svg.contains("<path d=\"M"),
        "expected line series <path> in horizontal xychart: {svg}"
    );

    // Line-only horizontal xychart should also render cleanly without bars.
    let line_only = "xychart-beta horizontal\n  x-axis [A, B, C]\n  y-axis 0 --> 50\n  line [10, 35, 25]";
    let line_svg = mermaid_to_svg(line_only).unwrap();
    assert_valid_svg(&line_svg);
    assert!(line_svg.contains("<path d=\"M"));
}

#[test]
fn m04_sequence_create_and_destroy_semantics() {
    let src = r#"
sequenceDiagram
  participant C
  C->>C: start
  create participant D
  C->>D: create
  destroy D
  D->>C: bye
"#;
    let svg = mermaid_to_svg(src).unwrap();
    assert_valid_svg(&svg);
    // C has both a top header and a bottom footer (2 occurrences of `\nC\n`).
    assert_eq!(
        svg.matches("\nC\n").count(),
        2,
        "participant C should have top header and bottom footer: {svg}"
    );
    // D is created inline and destroyed (no top header, no bottom footer) -> 1 occurrence.
    assert_eq!(
        svg.matches("\nD\n").count(),
        1,
        "created+destroyed participant D should render exactly one box (no footer): {svg}"
    );
    // Destroy cross (stroke-width="2.2") must be present (2 diagonal lines).
    assert_eq!(
        svg.matches("stroke-width=\"2.2\"").count(),
        2,
        "expected destroy cross (2 diagonal lines): {svg}"
    );
    // C's header box is at y="29" and footer at y="261"; D's box is at y="152" (inline, not top).
    assert!(
        svg.contains("x=\"153\" y=\"152\""),
        "expected D's box inline at creation message y: {svg}"
    );
    // Creation arrow terminates at D's left edge (x2="153"), not D's center (195).
    assert!(
        svg.contains("x1=\"71\" y1=\"171\" x2=\"153\" y2=\"171\""),
        "expected creation arrow to terminate at D's box edge: {svg}"
    );
    // D's lifeline runs only from bottom of D's box (190) to the destroy message y (233).
    assert!(
        svg.contains("x1=\"195\" y1=\"190\" x2=\"195\" y2=\"233\""),
        "expected D's lifeline from created box bottom to destroy message y: {svg}"
    );
}
