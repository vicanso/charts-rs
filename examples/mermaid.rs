use charts_rs::{MermaidChart, THEME_DARK, mermaid_to_svg, mermaid_validate};

fn main() {
    let flow = r#"
---
title: Service Request Flow
config:
  flowchart:
    curve: linear
    nodeSpacing: 55
---
flowchart LR
    accTitle: Service Request Flow
    accDescr: Routes authenticated requests to PostgreSQL or Redis
    A["`**Client** _app_`"] --> B@{ shape: diam, label: "Gateway" }
    B -->|Auth OK| C[(Primary DB)]
    B -->|Cache Hit| D([Redis])
    subgraph Backend
        C
        D
    end
    click A href "https://example.com/client" "Client docs" _blank
"#;
    mermaid_validate(flow).expect("validate flowchart");
    let flow_svg = mermaid_to_svg(flow).expect("render flowchart");
    println!("flowchart svg bytes: {}", flow_svg.len());

    // Untrusted input: disable emitted <a> hyperlinks via `allow_links = false`.
    let mut safe_chart = MermaidChart::new(flow);
    safe_chart.allow_links = false;
    let safe_svg = safe_chart.svg().expect("render flowchart without links");
    assert!(!safe_svg.contains("<a href="));
    println!("flowchart (allow_links=false) svg bytes: {}", safe_svg.len());

    let seq = r#"
sequenceDiagram
    autonumber
    actor User
    participant API
    participant DB
    User->>+API: GET /orders
    API->>+DB: SELECT * FROM orders
    DB-->>-API: rows
    API-->>-User: 200 OK
"#;
    let mut chart = MermaidChart::new_with_theme(seq, THEME_DARK);
    chart.title.text = "Order Query Sequence".to_string();
    let seq_svg = chart.svg().expect("render sequenceDiagram");
    println!("sequenceDiagram svg bytes: {}", seq_svg.len());
}
