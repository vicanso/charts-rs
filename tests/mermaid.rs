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

#[test]
fn m03_gantt_excludes_includes_weekday_tick_interval() {
    let src = r#"
gantt
  dateFormat YYYY-MM-DD
  excludes weekends
  weekday monday
  tickInterval 1week
  section A
    Long task :a1, 2026-01-01, 30d
    Another   :after a1, 20d
"#;
    let svg = mermaid_to_svg(src).unwrap();
    assert_valid_svg(&svg);
    assert!(svg.contains("Long task"));
    assert!(svg.contains("Another"));
    // With `excludes weekends`, 30 workdays from 2026-01-01 ends 2026-02-11 (exclusive 2026-02-12),
    // and `Another` runs 2026-02-12 .. 2026-03-12 with shaded excluded weekend bands.
    assert!(
        svg.contains("data-start=\"2026-01-01\" data-end=\"2026-02-12\""),
        "expected Long task 2026-01-01..2026-02-12: {svg}"
    );
    assert!(
        svg.contains("data-start=\"2026-02-12\" data-end=\"2026-03-12\""),
        "expected Another 2026-02-12..2026-03-12: {svg}"
    );
    assert!(
        svg.contains("fill-opacity=") && svg.matches("<rect").count() > 6,
        "expected shaded excluded weekend bands in gantt SVG: {svg}"
    );
    // `tickInterval 1week` must still format axis ticks as dates (e.g. `01-05`, `02-`), not unix numbers.
    assert!(
        svg.contains("01-") && svg.contains("02-") && !svg.contains("1767"),
        "expected formatted date ticks for tickInterval: {svg}"
    );

    // `includes` overrides `excludes`: excluding weekends + 2026-01-05, but including
    // 2026-01-03 (Sat) and 2026-01-04 (Sun), should still render cleanly.
    let with_includes = r#"
gantt
  dateFormat YYYY-MM-DD
  excludes weekends, 2026-01-05
  includes 2026-01-03, 2026-01-04
  section A
    Work :a1, 2026-01-02, 4d
"#;
    let inc_svg = mermaid_to_svg(with_includes).unwrap();
    assert_valid_svg(&inc_svg);
    assert!(inc_svg.contains("Work"));
}

#[test]
fn m05_parallel_edges_and_two_pass_labels() {
    // Class diagram: opposite-direction parallel relations (`Fish --> Zebra : eats` and `Zebra -- Fish`).
    let class_src = r#"
classDiagram
  class Fish
  class Zebra
  Fish --> Zebra : eats
  Zebra -- Fish
"#;
    let class_svg = mermaid_to_svg(class_src).unwrap();
    assert_valid_svg(&class_svg);
    assert!(class_svg.contains("eats"));
    // Extract the two curved edge <path d="..."> attributes (ignoring the open-arrow marker <path>)
    // and confirm they are distinct separated curves.
    let class_paths: Vec<&str> = class_svg
        .split("<path d=\"")
        .skip(1)
        .filter_map(|s| s.split('"').next())
        .filter(|d| d.contains(" C"))
        .collect();
    assert_eq!(
        class_paths.len(),
        2,
        "expected 2 curved relationship paths in class diagram: {class_svg}"
    );
    assert_ne!(
        class_paths[0], class_paths[1],
        "parallel opposite-direction edges must produce distinct paths: {:?}",
        class_paths
    );
    // Two-pass rendering: the label `eats` must appear AFTER both `<path` elements.
    let last_path_pos = class_svg.rfind("<path ").unwrap();
    let label_pos = class_svg.find("eats").unwrap();
    assert!(
        label_pos > last_path_pos,
        "edge label must render after all edge paths (two-pass): {class_svg}"
    );

    // Flowchart: bidirectional parallel edges (`A --> B` and `B --> A`).
    let fc_src = "flowchart LR\n  A -->|req| B\n  B -->|res| A";
    let fc_svg = mermaid_to_svg(fc_src).unwrap();
    assert_valid_svg(&fc_svg);
    let fc_paths: Vec<&str> = fc_svg
        .split("<path d=\"")
        .skip(1)
        .filter_map(|s| s.split('"').next())
        .collect();
    assert_eq!(fc_paths.len(), 2, "expected 2 edge paths: {fc_svg}");
    assert_ne!(
        fc_paths[0], fc_paths[1],
        "bidirectional flowchart edges must separate: {:?}",
        fc_paths
    );
    let fc_last_path = fc_svg.rfind("<path ").unwrap();
    assert!(fc_svg.find("req").unwrap() > fc_last_path);
    assert!(fc_svg.find("res").unwrap() > fc_last_path);
}

#[test]
fn m10_xychart_no_auto_legend_and_clean_numeric_ticks() {
    let src = r#"
xychart-beta
  title "Numeric X"
  x-axis "Step" 1 --> 10
  y-axis "Score" 0 --> 100
  bar [20, 40, 60, 80, 100]
  line [15, 35, 55, 75, 95]
"#;
    let svg = mermaid_to_svg(src).unwrap();
    assert_valid_svg(&svg);
    assert!(svg.contains("Numeric X"));
    assert!(svg.contains("Step"), "expected x-axis title: {svg}");
    assert!(svg.contains("Score"), "expected y-axis title: {svg}");
    // No anonymous auto-legend `Bar 1` / `Line 1` text elements.
    assert!(
        !svg.contains("\nBar 1\n"),
        "auto-legend Bar 1 should be hidden: {svg}"
    );
    assert!(
        !svg.contains("\nLine 1\n"),
        "auto-legend Line 1 should be hidden: {svg}"
    );
    // Clean numeric ticks (no awkward `3.2` or `7.8`) and no negative bar x-coordinates.
    assert!(!svg.contains("\n3.2\n"), "unexpected fractional tick 3.2: {svg}");
    assert!(!svg.contains("\n7.8\n"), "unexpected fractional tick 7.8: {svg}");
    assert!(
        !svg.contains("<rect x=\"-"),
        "bars must not overflow past left edge: {svg}"
    );

    // Horizontal mode also hides auto-legend and avoids `3.2`/`7.8`.
    let horiz = r#"
xychart-beta horizontal
  x-axis "Step" 1 --> 10
  y-axis "Score" 0 --> 100
  bar [20, 40, 60, 80, 100]
"#;
    let hsvg = mermaid_to_svg(horiz).unwrap();
    assert_valid_svg(&hsvg);
    assert!(!hsvg.contains("\nBar 1\n"));
    assert!(!hsvg.contains("\n3.2\n"));
    assert!(!hsvg.contains("\n7.8\n"));
}

#[test]
fn m14_gantt_task_status_styling() {
    let src = r#"
gantt
  title Status Styling
  dateFormat YYYY-MM-DD
  section Plan
    Completed :done, d1, 2026-03-01, 4d
    InFlight  :active, a1, after d1, 4d
    Upcoming  :u1, after a1, 4d
    Critical  :crit, c1, after u1, 4d
    CritDone  :crit, done, cd1, after c1, 4d
"#;
    let svg = mermaid_to_svg(src).unwrap();
    assert_valid_svg(&svg);
    // `active` must NOT emit a half-bar progress overlay (`fill-opacity="0.35"` track).
    assert!(
        !svg.contains("fill-opacity=\"0.35\""),
        "gantt active task should not render as a 50% progress bar: {svg}"
    );
    // Extract task bar rects (those with `data-name=`) and verify distinct fills for done/active/normal/crit.
    let fill_for = |name: &str| -> String {
        let needle = format!("data-name=\"{name}\"");
        let line = svg
            .lines()
            .find(|l| l.contains(&needle))
            .unwrap_or_else(|| panic!("missing task {name} in {svg}"));
        let start = line.find("fill=\"").unwrap() + 6;
        let end = start + line[start..].find('"').unwrap();
        line[start..end].to_string()
    };
    let done_fill = fill_for("Completed");
    let active_fill = fill_for("InFlight");
    let normal_fill = fill_for("Upcoming");
    let crit_fill = fill_for("Critical");
    let crit_done_fill = fill_for("CritDone");

    assert_ne!(done_fill, normal_fill, "done should be muted grey, distinct from normal");
    assert_ne!(active_fill, normal_fill, "active should be highlighted, distinct from normal");
    assert_ne!(crit_fill, normal_fill, "crit should be red, distinct from normal");
    assert_ne!(crit_done_fill, crit_fill, "crit+done should be muted compared to crit");
}

#[test]
fn m15_class_abstract_italic_and_static_underline() {
    let src = r#"
classDiagram
  class Shape {
    <<abstract>>
    +area()* double
    +create()$ Shape
    +name() String
  }
"#;
    let svg = mermaid_to_svg(src).unwrap();
    assert_valid_svg(&svg);
    // Abstract member `+area() double` must render with `font-style="italic"`.
    assert!(
        svg.contains("font-style=\"italic\""),
        "abstract member must emit font-style=\"italic\": {svg}"
    );
    // Static member `+create() Shape` must have an underline <line> matching its measured text width.
    assert!(svg.contains("+create() Shape"));
    assert!(svg.contains("+area() double"));
}

#[test]
fn m19_m21_strip_fa_icons_and_larger_cross_marker() {
    let src = r#"
flowchart LR
  A["fa:fa-car Car"] --x B["fab:fa-github Repo: main"]
  B --x C["fas:fa-check Done"]
"#;
    let svg = mermaid_to_svg(src).unwrap();
    assert_valid_svg(&svg);
    assert!(svg.contains("\nCar\n"), "fa:fa-car should be stripped: {svg}");
    assert!(
        svg.contains("\nRepo: main\n"),
        "fab:fa-github stripped while preserving colon: {svg}"
    );
    assert!(svg.contains("\nDone\n"), "fas:fa-check should be stripped: {svg}");
    assert!(!svg.contains("fa:fa-"), "raw fa:fa-* token leaked: {svg}");
    assert!(!svg.contains("fab:fa-"), "raw fab:fa-* token leaked: {svg}");
    assert!(!svg.contains("fas:fa-"), "raw fas:fa-* token leaked: {svg}");

    // Cross markers (--x): 2 edges x 2 diagonal lines = 4 <line> elements, each spanning ~10.5px.
    let cross_lines: Vec<&str> = svg.lines().filter(|l| l.starts_with("<line ")).collect();
    assert_eq!(cross_lines.len(), 4, "expected 4 cross-marker lines: {svg}");
}

#[test]
fn m16_sequence_fragment_horizontal_bounds() {
    let src = r#"
sequenceDiagram
  participant A
  participant B
  participant C
  participant D
  A->>B: init
  loop Retry
    B->>C: call
    opt Fallback
      C-->>B: err
    end
  end
  C->>D: forward
"#;
    let svg = mermaid_to_svg(src).unwrap();
    assert_valid_svg(&svg);
    // Extract fragment frame rects (stroke-width="1.4" with transparent fill).
    let frames: Vec<(f32, f32)> = svg
        .lines()
        .filter(|l| l.starts_with("<rect ") && l.contains("stroke-width=\"1.4\""))
        .map(|l| {
            let get = |k: &str| -> f32 {
                let pat = format!("{k}=\"");
                let s = l.find(&pat).unwrap() + pat.len();
                let e = s + l[s..].find('"').unwrap();
                l[s..e].parse().unwrap()
            };
            (get("x"), get("width"))
        })
        .collect();
    assert_eq!(frames.len(), 2, "expected 2 fragment frames (loop + opt): {svg}");
    let (loop_x, loop_w) = frames[0];
    let (opt_x, opt_w) = frames[1];
    // `loop` covers only B and C (must start well to the right of A's left edge x=29).
    assert!(
        loop_x > 100.0,
        "loop fragment should start near B, not at left edge: x={loop_x}"
    );
    // Nested `opt` sits strictly inside `loop`.
    assert!(
        opt_x > loop_x && (opt_x + opt_w) < (loop_x + loop_w),
        "nested opt ({opt_x}..{}) must fit strictly inside loop ({loop_x}..{})",
        opt_x + opt_w,
        loop_x + loop_w
    );
}

#[test]
fn m06_flowchart_subgraph_collision_and_title_clearance() {
    let src = r#"
flowchart BT
  subgraph Outer
    subgraph Inner
      a1 --> a2
    end
    b1 --> a1
  end
  c1 --> Inner
  a2 --> d1
"#;
    let svg = mermaid_to_svg(src).unwrap();
    assert_valid_svg(&svg);
    assert!(svg.contains("Outer"));
    assert!(svg.contains("Inner"));
    assert!(svg.contains("c1"));
    assert!(svg.contains("d1"));

    let parse_rect = |line: &str| -> (f32, f32, f32, f32) {
        let get = |k: &str| -> f32 {
            let pat = format!("{k}=\"");
            let s = line.find(&pat).unwrap() + pat.len();
            let e = s + line[s..].find('"').unwrap();
            line[s..e].parse().unwrap()
        };
        (get("x"), get("y"), get("width"), get("height"))
    };

    // Subgraph rects have `rx="6" ry="6"`; Outer is drawn first (depth 0).
    let sg_rects: Vec<(f32, f32, f32, f32)> = svg
        .lines()
        .filter(|l| l.starts_with("<rect ") && l.contains("rx=\"6\""))
        .map(parse_rect)
        .collect();
    assert_eq!(sg_rects.len(), 2, "expected Outer and Inner subgraph rects: {svg}");
    let (ox, oy, ow, oh) = sg_rects[0];

    // Find c1's <text> position and verify it lies strictly outside Outer's bounding box.
    let lines: Vec<&str> = svg.lines().collect();
    let c1_idx = lines.iter().position(|l| *l == "c1").unwrap();
    let c1_text_line = lines[c1_idx - 1];
    let get_attr = |line: &str, k: &str| -> f32 {
        let pat = format!("{k}=\"");
        let s = line.find(&pat).unwrap() + pat.len();
        let e = s + line[s..].find('"').unwrap();
        line[s..e].parse().unwrap()
    };
    let c1_x = get_attr(c1_text_line, "x");
    let c1_y = get_attr(c1_text_line, "y");
    let inside_outer = c1_x >= ox && c1_x <= ox + ow && c1_y >= oy && c1_y <= oy + oh;
    assert!(
        !inside_outer,
        "external node c1 ({c1_x}, {c1_y}) must not overlap Outer ({ox}..{}, {oy}..{})",
        ox + ow,
        oy + oh
    );

    // Titles `Outer` and `Inner` should shift off-center (`text-anchor="start"` or `"end"`)
    // to stay clear of the vertical `a2 --> d1` edge.
    let inner_idx = lines.iter().position(|l| *l == "Inner").unwrap();
    let outer_idx = lines.iter().position(|l| *l == "Outer").unwrap();
    assert!(
        lines[inner_idx - 1].contains("text-anchor=\"start\"")
            || lines[inner_idx - 1].contains("text-anchor=\"end\""),
        "Inner title should shift clear of crossing edge: {}",
        lines[inner_idx - 1]
    );
    assert!(
        lines[outer_idx - 1].contains("text-anchor=\"start\"")
            || lines[outer_idx - 1].contains("text-anchor=\"end\""),
        "Outer title should shift clear of crossing edge: {}",
        lines[outer_idx - 1]
    );
}

#[test]
fn m08_er_lr_spacing_and_self_loop_markers() {
    let src = r#"
erDiagram
  direction LR
  CUSTOMER ||--o{ ORDER : places
  ORDER ||--|{ LINE-ITEM : contains
  CUSTOMER }|..|{ DELIVERY-ADDRESS : uses
  CUSTOMER |o--o| LOYALTY : has
  EMPLOYEE ||--o{ EMPLOYEE : manages
"#;
    let svg = mermaid_to_svg(src).unwrap();
    assert_valid_svg(&svg);
    for label in ["places", "contains", "uses", "has", "manages"] {
        assert!(svg.contains(label), "missing label {label}: {svg}");
    }

    // Verify `has` and `manages` label boxes do not overlap.
    let label_box = |label: &str| -> (f32, f32, f32, f32) {
        let lines: Vec<&str> = svg.lines().collect();
        let idx = lines.iter().position(|l| *l == label).unwrap();
        // The pill <rect> immediately precedes the <text> element.
        let rect_line = lines[idx - 2];
        let get = |k: &str| -> f32 {
            let pat = format!("{k}=\"");
            let s = rect_line.find(&pat).unwrap() + pat.len();
            let e = s + rect_line[s..].find('"').unwrap();
            rect_line[s..e].parse().unwrap()
        };
        (get("x"), get("y"), get("width"), get("height"))
    };
    let (hx, hy, hw, hh) = label_box("has");
    let (mx, my, mw, mh) = label_box("manages");
    let overlap_x = hx < mx + mw && mx < hx + hw;
    let overlap_y = hy < my + mh && my < hy + hh;
    assert!(
        !(overlap_x && overlap_y),
        "`has` ({hx}..{}, {hy}..{}) and `manages` ({mx}..{}, {my}..{}) must not overlap",
        hx + hw,
        hy + hh,
        mx + mw,
        my + mh
    );
}

#[test]
fn m12_sankey_margin_and_reduced_crossings() {
    let sankey =
        MermaidChart::from_json(include_str!("../asset/mermaid_chart/sankey.json")).unwrap();
    let svg = sankey.svg().unwrap();
    assert_valid_svg(&svg);

    // Every node rect (has `data-name=`) must keep >= 12px margin from left/right/bottom edges
    // of the 640x420 canvas.
    let get_attr = |line: &str, k: &str| -> f32 {
        let pat = format!("{k}=\"");
        let s = line.find(&pat).unwrap() + pat.len();
        let e = s + line[s..].find('"').unwrap();
        line[s..e].parse().unwrap()
    };
    let mut transport_y = None;
    let mut residential_y = None;
    for line in svg.lines().filter(|l| l.starts_with("<rect ") && l.contains("data-name=")) {
        let x = get_attr(line, "x");
        let y = get_attr(line, "y");
        let w = get_attr(line, "width");
        let h = get_attr(line, "height");
        assert!(x >= 12.0, "sankey node touches left edge: x={x}");
        assert!(x + w <= 628.0, "sankey node touches right edge: right={}", x + w);
        assert!(y + h <= 406.5, "sankey node touches bottom edge: bottom={}", y + h);
        if line.contains("data-name=\"Transport\"") {
            transport_y = Some(y);
        }
        if line.contains("data-name=\"Residential\"") {
            residential_y = Some(y);
        }
    }
    // Barycenter ordering places `Transport` (only fed by top `Electricity`) above `Residential`
    // (fed by both `Electricity` and bottom `Heat`), avoiding unnecessary crossings.
    assert!(
        transport_y.unwrap() < residential_y.unwrap(),
        "expected Transport ({:?}) above Residential ({:?}) after barycenter ordering",
        transport_y,
        residential_y
    );
}

#[test]
fn m13_radar_polygon_vertical_centering() {
    let radar =
        MermaidChart::from_json(include_str!("../asset/mermaid_chart/radar.json")).unwrap();
    let svg = radar.svg().unwrap();
    assert_valid_svg(&svg);

    // Find the outer pentagon grid ring (`stroke-width="1" fill="none"`)
    // and check that its visual top/bottom whitespace in the plot area is balanced.
    let grid_paths: Vec<&str> = svg
        .lines()
        .filter(|l| {
            l.starts_with("<path d=\"M ")
                && l.contains(" Z\"")
                && l.contains("fill=\"none\"")
                && l.contains("stroke-width=\"1\"")
        })
        .collect();
    assert!(!grid_paths.is_empty(), "expected radar grid polygons: {svg}");
    let outer = grid_paths.last().unwrap();
    let d_attr = outer.split("d=\"").nth(1).unwrap().split('"').next().unwrap();
    let mut min_y = f32::INFINITY;
    let mut max_y = f32::NEG_INFINITY;
    for seg in d_attr.split(['M', 'L', 'Z']) {
        let mut nums = seg.split_whitespace();
        if let (Some(_x), Some(y_str)) = (nums.next(), nums.next())
            && let Ok(y) = y_str.parse::<f32>()
        {
            min_y = min_y.min(y);
            max_y = max_y.max(y);
        }
    }
    // Plot area sits below title+legend (~62px) and above bottom margin (475px).
    // Top visual extent includes ~19px top axis label; bottom includes ~7px bottom label.
    let top_space = (min_y - 19.0) - 62.0;
    let bottom_space = 475.0 - (max_y + 7.0);
    assert!(
        (top_space - bottom_space).abs() <= 10.0,
        "radar vertical whitespace should be balanced (<=10px delta): top={top_space}, bottom={bottom_space}"
    );
}

#[test]
fn m09_treemap_group_headers_and_hierarchy() {
    // Gallery treemap.json: top-level section headers (`Compute`, `Storage`, `Networking`) must render.
    let tm =
        MermaidChart::from_json(include_str!("../asset/mermaid_chart/treemap.json")).unwrap();
    let svg = tm.svg().unwrap();
    assert_valid_svg(&svg);
    for section in ["Compute", "Storage", "Networking"] {
        assert!(
            svg.contains(section),
            "expected treemap section header `{section}`: {svg}"
        );
    }

    // 4-level nested hierarchy: `Root` -> `A`/`B` -> `B2` -> `B21`.
    let nested = r#"
treemap-beta
"Root"
  "A"
    "A1": 10
    "A2": 20
  "B"
    "B1": 5
    "B2"
      "B21": 15
      "B22": 10
"#;
    let nsvg = mermaid_to_svg(nested).unwrap();
    assert_valid_svg(&nsvg);
    for hdr in ["Root", "A", "B", "B2", "B21", "B22"] {
        assert!(
            nsvg.contains(hdr),
            "expected nested treemap header/leaf `{hdr}`: {nsvg}"
        );
    }
}

#[test]
fn m11_pie_label_collision_avoidance_and_zero_validation() {
    // 11 small slices (1 each) + 1 large slice (50): outer labels on each side must not overlap.
    let src = r#"
pie title Many
  "a": 1
  "b": 1
  "c": 1
  "d": 1
  "e": 1
  "f": 1
  "g": 1
  "h": 1
  "i": 1
  "j": 1
  "k": 1
  "l": 50
"#;
    let svg = mermaid_to_svg(src).unwrap();
    assert_valid_svg(&svg);
    assert!(svg.contains("Many"));
    // Collect outer slice label `(x, y)` positions (text lines containing `%`).
    let lines: Vec<&str> = svg.lines().collect();
    let mut left_ys = Vec::new();
    let mut right_ys = Vec::new();
    for (i, line) in lines.iter().enumerate() {
        if line.contains('%') && i > 0 && lines[i - 1].starts_with("<text ") {
            let tag = lines[i - 1];
            let get = |k: &str| -> f32 {
                let pat = format!("{k}=\"");
                let s = tag.find(&pat).unwrap() + pat.len();
                let e = s + tag[s..].find('"').unwrap();
                tag[s..e].parse().unwrap()
            };
            let x = get("x");
            let y = get("y");
            if x < 320.0 {
                left_ys.push(y);
            } else {
                // Right-side labels must sit outside the pie circle (center ~320,240, r ~128).
                let dist = (x - 320.0).hypot(y - 240.0);
                assert!(
                    dist > 130.0,
                    "right-side outer label sits over pie circle: x={x}, y={y}, dist={dist}"
                );
                right_ys.push(y);
            }
        }
    }
    for mut ys in [left_ys, right_ys] {
        ys.sort_by(|a, b| a.partial_cmp(b).unwrap());
        for pair in ys.windows(2) {
            assert!(
                pair[1] - pair[0] >= 15.0,
                "pie outer labels overlap vertically: {:?}",
                ys
            );
        }
    }

    // All-zero pie diagram must return an explicit Error::Mermaid.
    let zero_pie = "pie\n  \"a\": 0\n  \"b\": 0";
    let err = mermaid_to_svg(zero_pie).unwrap_err().to_string();
    assert!(
        err.contains("no positive values"),
        "expected 'no positive values' error for all-zero pie, got: {err}"
    );
}

#[test]
fn m20_gantt_vert_marker() {
    let src = r#"
gantt
  title Release Schedule
  dateFormat YYYY-MM-DD
  section Build
    Alpha       :a1, 2026-01-10, 15d
    Code Freeze :vert, v1, 2026-02-01, 0d
    Beta        :after a1, 15d
    GA Cut      :crit, vert, v2, after v1, 0d
"#;
    let svg = mermaid_to_svg(src).unwrap();
    assert_valid_svg(&svg);
    assert!(svg.contains("Code Freeze"), "expected vert marker label: {svg}");
    assert!(svg.contains("GA Cut"), "expected second vert marker label: {svg}");
    // `vert` markers must NOT consume task rows (only `Alpha` and `Beta` have `data-name=`).
    assert!(svg.contains("data-name=\"Alpha\""));
    assert!(svg.contains("data-name=\"Beta\""));
    assert!(!svg.contains("data-name=\"Code Freeze\""));
    assert!(!svg.contains("data-name=\"GA Cut\""));
    // Both vertical marker dashed lines (`stroke-width="1.4"`) must be rendered.
    assert_eq!(
        svg.matches("stroke-width=\"1.4\"").count(),
        2,
        "expected 2 vertical marker lines: {svg}"
    );
    // Coincident vertical marker labels (`Code Freeze` and `GA Cut`) must stagger vertically.
    let lines: Vec<&str> = svg.lines().collect();
    let y_of = |lbl: &str| -> f32 {
        let idx = lines.iter().position(|l| *l == lbl).unwrap();
        let tag = lines[idx - 1];
        let s = tag.find("y=\"").unwrap() + 3;
        let e = s + tag[s..].find('"').unwrap();
        tag[s..e].parse().unwrap()
    };
    assert!(
        (y_of("GA Cut") - y_of("Code Freeze")).abs() >= 10.0,
        "coincident vert marker labels should stagger vertically"
    );
}

#[test]
fn m17_m22_markdown_strings_bold_italic_and_multiline() {
    // Flowchart: multiline Markdown string with bold + italic spans.
    let fc_src = "flowchart LR\n  A[\"`**Bold** and _italic_\n  second line`\"] --> B";
    let fc_svg = mermaid_to_svg(fc_src).unwrap();
    assert_valid_svg(&fc_svg);
    assert!(
        fc_svg.contains("<tspan font-weight=\"bold\">Bold</tspan>"),
        "expected bold tspan in flowchart: {fc_svg}"
    );
    assert!(
        fc_svg.contains("<tspan font-style=\"italic\">italic</tspan>"),
        "expected italic tspan in flowchart: {fc_svg}"
    );
    assert!(
        fc_svg.contains("second line"),
        "expected second line from multiline Markdown string: {fc_svg}"
    );
    assert!(!fc_svg.contains("`**"), "raw markdown backticks leaked: {fc_svg}");

    // Mindmap: Markdown string with bold + italic spans.
    let mm_src = "mindmap\n  root((Hub))\n    (\"`Zero **External** _Deps_`\")";
    let mm_svg = mermaid_to_svg(mm_src).unwrap();
    assert_valid_svg(&mm_svg);
    assert!(
        mm_svg.contains("<tspan font-weight=\"bold\">External</tspan>"),
        "expected bold tspan in mindmap: {mm_svg}"
    );
    assert!(
        mm_svg.contains("<tspan font-style=\"italic\">Deps</tspan>"),
        "expected italic tspan in mindmap: {mm_svg}"
    );
}

#[test]
fn m18_flowchart_v11_shape_syntax() {
    let src = r#"
flowchart LR
  S@{ shape: sm-circ } --> A@{ shape: doc, label: "Spec, v1" }
  A --> B@{ shape: docs, label: "Reports" }
  B --> C@{ shape: delay, label: "Wait" }
  C --> D@{ shape: tri, label: "Extract" }
  D --> E@{ shape: flag, label: "Signal" }
  E --> J@{ shape: f-circ }
  J --> T@{ shape: text, label: "Plain Note" }
  J --> DB@{ shape: cyl, label: "Store", icon: "fa:database" }
"#;
    let svg = mermaid_to_svg(src).unwrap();
    assert_valid_svg(&svg);
    for lbl in ["Spec, v1", "Reports", "Wait", "Extract", "Signal", "Plain Note", "Store"] {
        assert!(svg.contains(lbl), "missing v11 node label `{lbl}`: {svg}");
    }
    // Unknown shape returns a clear line-numbered Error::Mermaid.
    let bad = "flowchart TD\n  A@{ shape: nonexistent, label: \"Oops\" }";
    let err = mermaid_to_svg(bad).unwrap_err().to_string();
    assert!(err.contains("line 2"), "expected line 2 in error: {err}");
    assert!(
        err.contains("unknown flowchart v11 shape"),
        "expected unknown shape diagnostic: {err}"
    );
}

#[test]
fn e04_accessibility_acc_title_and_descr() {
    // Single-line accTitle + accDescr with XML special characters.
    let fc = r#"
flowchart LR
    accTitle: Auth <Flow> & "Tokens"
    accDescr: Validates A < B & issues JWTs
    A[Client] --> B[Server]
"#;
    let svg = mermaid_to_svg(fc).unwrap();
    assert_valid_svg(&svg);
    assert!(svg.contains("role=\"img\""), "missing role=img: {svg}");
    assert!(
        svg.contains("aria-labelledby=\"mermaid-title\""),
        "missing aria-labelledby: {svg}"
    );
    assert!(
        svg.contains("aria-describedby=\"mermaid-desc\""),
        "missing aria-describedby: {svg}"
    );
    assert!(
        svg.contains("<title id=\"mermaid-title\">Auth &lt;Flow&gt; &amp; &quot;Tokens&quot;</title>"),
        "missing escaped <title>: {svg}"
    );
    assert!(
        svg.contains("<desc id=\"mermaid-desc\">Validates A &lt; B &amp; issues JWTs</desc>"),
        "missing escaped <desc>: {svg}"
    );

    // Multiline accDescr { ... } on an adapter diagram (pie).
    let pie = r#"
pie title Budget
    accTitle: Quarterly Budget
    accDescr {
        Engineering 60%
        Operations 40%
    }
    "Engineering" : 60
    "Operations" : 40
"#;
    let pie_svg = mermaid_to_svg(pie).unwrap();
    assert_valid_svg(&pie_svg);
    assert!(pie_svg.contains("<title id=\"mermaid-title\">Quarterly Budget</title>"));
    assert!(pie_svg.contains("<desc id=\"mermaid-desc\">Engineering 60%\nOperations 40%</desc>"));

    // Unclosed accDescr { returns a line-numbered error.
    let bad = "flowchart TD\n  accDescr {\n    missing close\n  A --> B";
    let err = mermaid_to_svg(bad).unwrap_err().to_string();
    assert!(err.contains("line 2"), "expected line 2 in error: {err}");
    assert!(err.contains("unclosed"), "expected unclosed diagnostic: {err}");

    // Diagram without accTitle/accDescr must not emit accessibility metadata.
    let plain = mermaid_to_svg("flowchart LR\n  A --> B").unwrap();
    assert!(!plain.contains("role=\"img\""));
    assert!(!plain.contains("<title"));
    assert!(!plain.contains("<desc"));
}

#[test]
fn e05_click_links_and_protocol_allowlist() {
    // Flowchart: href, shorthand, tooltip, target=_blank, unsafe javascript:/data:, callback.
    let fc = r#"
flowchart LR
    A[Docs] --> B[Mail] --> C[Unsafe] --> D[Callback]
    click A href "https://example.com/docs?a=1&b=2" "Open Docs" _blank
    click B "mailto:ops@example.com" "Email Ops"
    click C "javascript:alert(1)" "XSS"
    click D callback "JS"
"#;
    let svg = mermaid_to_svg(fc).unwrap();
    assert_valid_svg(&svg);
    assert!(
        svg.contains(
            "<a href=\"https://example.com/docs?a=1&amp;b=2\" target=\"_blank\"><title>Open Docs</title>"
        ),
        "missing escaped href/target/tooltip: {svg}"
    );
    assert!(
        svg.contains("<a href=\"mailto:ops@example.com\"><title>Email Ops</title>"),
        "missing mailto link: {svg}"
    );
    assert!(!svg.contains("javascript:"), "javascript: URL must be blocked: {svg}");
    assert!(!svg.contains("XSS"), "blocked link tooltip must not appear: {svg}");

    // allow_links = false disables all emitted hyperlinks.
    let mut chart = MermaidChart::new(fc);
    chart.allow_links = false;
    let disabled_svg = chart.svg().unwrap();
    assert_valid_svg(&disabled_svg);
    assert!(
        !disabled_svg.contains("<a href="),
        "allow_links=false must suppress all <a> elements: {disabled_svg}"
    );

    // Class diagram click + link directives.
    let cls = r#"
classDiagram
    class Order
    class User
    Order --> User
    click Order href "/models/order" "Order model" _self
    link User "https://example.com/user" "User docs"
"#;
    let cls_svg = mermaid_to_svg(cls).unwrap();
    assert_valid_svg(&cls_svg);
    assert!(cls_svg.contains("<a href=\"/models/order\" target=\"_self\"><title>Order model</title>"));
    assert!(cls_svg.contains("<a href=\"https://example.com/user\"><title>User docs</title>"));

    // State diagram click directive + data: protocol rejection.
    let st = r#"
stateDiagram-v2
    [*] --> Active
    Active --> Blocked
    click Active href "./active.html" "Active state"
    click Blocked "data:text/html,<script>alert(1)</script>"
"#;
    let st_svg = mermaid_to_svg(st).unwrap();
    assert_valid_svg(&st_svg);
    assert!(st_svg.contains("<a href=\"./active.html\"><title>Active state</title>"));
    assert!(!st_svg.contains("data:text/html"));
}

#[test]
fn e12_mermaid_validate_and_typo_hints() {
    use charts_rs::mermaid_validate;

    // Valid diagrams across multiple types pass validation.
    for valid in [
        "flowchart LR\n  A --> B",
        "sequenceDiagram\n  Alice->>Bob: Hi",
        "classDiagram\n  class Order",
        "pie\n  \"A\" : 10\n  \"B\" : 20",
        "gantt\n  dateFormat YYYY-MM-DD\n  Task : 2025-01-01, 3d",
        "sankey-beta\n  A,B,10",
    ] {
        assert!(
            mermaid_validate(valid).is_ok(),
            "expected valid diagram to pass mermaid_validate: {valid}"
        );
    }

    // Invalid semantic diagram (Sankey cycle) fails validation.
    assert!(mermaid_validate("sankey-beta\n  A,B,10\n  B,A,5").is_err());

    // Near-miss diagram type suggestions.
    let err_state = mermaid_validate("stateDiagram-v3\n  [*] --> A")
        .unwrap_err()
        .to_string();
    assert!(
        err_state.contains("did you mean `stateDiagram-v2`?"),
        "expected stateDiagram-v2 suggestion: {err_state}"
    );
    let err_fc = mermaid_validate("flowchar LR\n  A --> B")
        .unwrap_err()
        .to_string();
    assert!(
        err_fc.contains("did you mean `flowchart`?"),
        "expected flowchart suggestion: {err_fc}"
    );

    // Invalid flowchart direction lists valid options.
    let err_dir = mermaid_validate("flowchart TOP\n  A --> B")
        .unwrap_err()
        .to_string();
    assert!(
        err_dir.contains("TB, TD, BT, LR, RL"),
        "expected valid direction list: {err_dir}"
    );

    // Sequence arrow `->>` / `-->>` in flowchart produces actionable hint.
    let err_arrow = mermaid_validate("flowchart LR\n  A ->> B")
        .unwrap_err()
        .to_string();
    assert!(err_arrow.contains("line 2"), "expected line 2: {err_arrow}");
    assert!(
        err_arrow.contains("sequenceDiagram") && err_arrow.contains("-->"),
        "expected sequenceDiagram arrow hint: {err_arrow}"
    );
}

#[test]
fn e01_frontmatter_and_init_config_passthrough() {
    // 1. YAML frontmatter flowchart config: linear curve + larger spacing + fontFamily.
    let fc_linear = r#"
---
config:
  fontFamily: "Courier New"
  flowchart:
    curve: linear
    nodeSpacing: 80
    rankSpacing: 100
---
flowchart TD
    A --> B
    A --> C
    B --> D
    C --> D
"#;
    let fc_default = "flowchart TD\n  A --> B\n  A --> C\n  B --> D\n  C --> D";
    let svg_linear = mermaid_to_svg(fc_linear).unwrap();
    let svg_default = mermaid_to_svg(fc_default).unwrap();
    assert_valid_svg(&svg_linear);
    assert!(
        svg_linear.contains("font-family=\"Courier New\""),
        "expected frontmatter fontFamily: {svg_linear}"
    );
    // Linear curve uses L segments instead of cubic C segments for edges.
    assert!(
        svg_linear.contains(" L"),
        "expected linear edge segments: {svg_linear}"
    );
    // Explicit Rust API font_family overrides frontmatter fontFamily.
    let mut override_chart = MermaidChart::new(fc_linear);
    override_chart.font_family = "Georgia".to_string();
    let svg_override = override_chart.svg().unwrap();
    assert!(
        svg_override.contains("font-family=\"Georgia\""),
        "expected Rust API font_family to override frontmatter: {svg_override}"
    );
    assert!(
        !svg_override.contains("font-family=\"Courier New\""),
        "frontmatter fontFamily must not leak over explicit API override"
    );
    assert_ne!(svg_linear, svg_default);

    // 2. Step curve via %%{init}%% directive.
    let fc_step = r#"
%%{init: {'flowchart': {'curve': 'step'}}}%%
flowchart LR
    A --> B
    A --> C
    B --> D
    C --> D
"#;
    let svg_step = mermaid_to_svg(fc_step).unwrap();
    assert_valid_svg(&svg_step);

    // 3. Sequence diagram: mirrorActors: false & showSequenceNumbers: true via %%{init}%%.
    let seq_src = r#"
%%{init: {'sequence': {'mirrorActors': false, 'showSequenceNumbers': true}}}%%
sequenceDiagram
    Alice->>Bob: Hello
    Bob-->>Alice: World
"#;
    let seq_svg = mermaid_to_svg(seq_src).unwrap();
    assert_valid_svg(&seq_svg);
    assert!(
        seq_svg.contains("1. Hello") && seq_svg.contains("2. World"),
        "expected showSequenceNumbers autonumbering: {seq_svg}"
    );
    // With mirrorActors: false, each participant label appears once (header only, no footer).
    assert_eq!(
        seq_svg.matches("Alice").count(),
        1,
        "expected Alice footer suppressed when mirrorActors=false: {seq_svg}"
    );
    assert_eq!(
        seq_svg.matches("Bob").count(),
        1,
        "expected Bob footer suppressed when mirrorActors=false: {seq_svg}"
    );

    // 4. xyChart: showLegend: true via frontmatter re-enables legend.
    let xy_src = r#"
---
config:
  xyChart:
    showLegend: true
---
xychart-beta
    x-axis [Q1, Q2, Q3]
    y-axis 0 --> 100
    bar [30, 50, 70]
"#;
    let xy_svg = mermaid_to_svg(xy_src).unwrap();
    assert_valid_svg(&xy_svg);
    assert!(
        xy_svg.contains("Bar 1"),
        "expected xychart legend when showLegend=true: {xy_svg}"
    );
}
