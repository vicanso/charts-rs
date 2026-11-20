use charts_rs::{MermaidChart, THEME_DARK, mermaid_to_svg};

fn main() {
    let flow = r#"
flowchart LR
    A[Client] --> B{Gateway}
    B -->|Auth OK| C[(Primary DB)]
    B -->|Cache Hit| D([Redis])
    subgraph Backend
        C
        D
    end
"#;
    let flow_svg = mermaid_to_svg(flow).expect("render flowchart");
    println!("flowchart svg bytes: {}", flow_svg.len());

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
