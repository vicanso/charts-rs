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

//! Mermaid `treemap-beta` / `treemap` diagram adapter -> [`TreemapChart`](crate::TreemapChart).

use std::collections::HashMap;

use super::super::super::error::{Error, Result};
use super::super::super::sunburst_chart::SunburstData;
use super::super::super::treemap_chart::TreemapChart;
use super::super::MermaidChart;
use super::super::preprocess::SourceLine;
use super::super::style::{
    StyleDirective, StyleProps, resolve_style, try_parse_style_directive, unquote_text,
};
use super::apply_base_options;

#[derive(Debug)]
struct TreemapNode {
    name: String,
    value: Option<f32>,
    children: Vec<usize>,
    classes: Vec<String>,
    inline_style: StyleProps,
}

#[derive(Debug)]
struct ParsedTreemap {
    title: Option<String>,
    nodes: Vec<TreemapNode>,
    roots: Vec<usize>,
    class_defs: HashMap<String, StyleProps>,
}

fn find_value_colon(line: &str) -> Option<usize> {
    let bytes = line.as_bytes();
    let mut in_quotes = false;
    let mut found = None;
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'"' {
            in_quotes = !in_quotes;
        } else if !in_quotes && bytes[i] == b':' {
            let prev_colon = i > 0 && bytes[i - 1] == b':';
            let next_colon = i + 1 < bytes.len() && bytes[i + 1] == b':';
            if !prev_colon && !next_colon {
                found = Some(i);
            }
        }
        i += 1;
    }
    found
}

fn split_inline_classes_tail(raw: &str) -> (&str, Vec<String>) {
    let bytes = raw.as_bytes();
    let mut in_quotes = false;
    let mut found = None;
    let mut i = 0;
    while i + 2 < bytes.len() {
        if bytes[i] == b'"' {
            in_quotes = !in_quotes;
        } else if !in_quotes && bytes[i..].starts_with(b":::") {
            found = Some(i);
            break;
        }
        i += 1;
    }
    let Some(pos) = found else {
        return (raw.trim(), Vec::new());
    };
    let head = raw[..pos].trim();
    let tail = raw[pos + 3..].trim();
    let classes = tail
        .split([',', ' ', '\t'])
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string)
        .collect();
    (head, classes)
}

fn parse_treemap(lines: &[SourceLine]) -> Result<ParsedTreemap> {
    let mut title = None;
    let mut nodes: Vec<TreemapNode> = Vec::new();
    let mut roots: Vec<usize> = Vec::new();
    let mut stack: Vec<(usize, usize)> = Vec::new();
    let mut class_defs: HashMap<String, StyleProps> = HashMap::new();
    let mut pending_classes: Vec<(String, Vec<String>)> = Vec::new();
    let mut pending_styles: Vec<(String, StyleProps)> = Vec::new();
    let mut in_acc_descr = false;

    for sl in lines {
        let trimmed = sl.text.trim();
        if trimmed.is_empty() {
            continue;
        }
        if in_acc_descr {
            if trimmed.contains('}') {
                in_acc_descr = false;
            }
            continue;
        }
        if trimmed.starts_with("accTitle") {
            continue;
        }
        if trimmed.starts_with("accDescr") {
            if trimmed.contains('{') && !trimmed.contains('}') {
                in_acc_descr = true;
            }
            continue;
        }
        if let Some(rest) = trimmed.strip_prefix("title")
            && (rest.is_empty() || rest.starts_with(char::is_whitespace))
        {
            let t = unquote_text(rest.trim());
            if !t.is_empty() {
                title = Some(t);
            }
            continue;
        }
        if let Some(directive) = try_parse_style_directive(trimmed, sl.line_no)? {
            match directive {
                StyleDirective::ClassDef(name, props) => {
                    class_defs.entry(name).or_default().merge_from(&props);
                }
                StyleDirective::ClassAssign { ids, classes } => {
                    for id in ids {
                        pending_classes.push((id, classes.clone()));
                    }
                }
                StyleDirective::NodeStyle { ids, props } => {
                    for id in ids {
                        pending_styles.push((id, props.clone()));
                    }
                }
            }
            continue;
        }

        let (label_raw, classes, value) = if let Some(colon_idx) = find_value_colon(trimmed) {
            let lhs = trimmed[..colon_idx].trim();
            let rhs_raw = trimmed[colon_idx + 1..].trim();
            let (num_str, mut rhs_classes) = split_inline_classes_tail(rhs_raw);
            let v: f32 = num_str.parse().map_err(|_| Error::Mermaid {
                line: Some(sl.line_no),
                message: format!("invalid numeric treemap value `{num_str}`"),
            })?;
            if !v.is_finite() || v < 0.0 {
                return Err(Error::Mermaid {
                    line: Some(sl.line_no),
                    message: format!("treemap value must be non-negative finite, got `{num_str}`"),
                });
            }
            let (head, mut lhs_classes) = split_inline_classes_tail(lhs);
            lhs_classes.append(&mut rhs_classes);
            (head, lhs_classes, Some(v))
        } else {
            let (head, cls) = split_inline_classes_tail(trimmed);
            (head, cls, None)
        };

        let name = unquote_text(label_raw);
        if name.is_empty() {
            return Err(Error::Mermaid {
                line: Some(sl.line_no),
                message: "empty treemap node label".to_string(),
            });
        }

        let idx = nodes.len();
        while stack.last().is_some_and(|&(pi, _)| pi >= sl.indent) {
            stack.pop();
        }
        if let Some(&(_, parent_idx)) = stack.last() {
            if nodes[parent_idx].value.is_some() {
                return Err(Error::Mermaid {
                    line: Some(sl.line_no),
                    message: format!(
                        "treemap leaf `{}` cannot contain child `{name}`",
                        nodes[parent_idx].name
                    ),
                });
            }
            nodes[parent_idx].children.push(idx);
        } else {
            roots.push(idx);
        }
        stack.push((sl.indent, idx));

        nodes.push(TreemapNode {
            name,
            value,
            children: Vec::new(),
            classes,
            inline_style: StyleProps::default(),
        });
    }

    if roots.is_empty() {
        return Err(Error::Mermaid {
            line: lines.first().map(|l| l.line_no),
            message: "treemap has no entries".to_string(),
        });
    }

    for (target, cls) in pending_classes {
        for n in &mut nodes {
            if n.name == target {
                for c in &cls {
                    if !n.classes.contains(c) {
                        n.classes.push(c.clone());
                    }
                }
            }
        }
    }
    for (target, props) in pending_styles {
        for n in &mut nodes {
            if n.name == target {
                n.inline_style.merge_from(&props);
            }
        }
    }

    Ok(ParsedTreemap {
        title,
        nodes,
        roots,
        class_defs,
    })
}

fn build_sunburst_node(
    idx: usize,
    nodes: &[TreemapNode],
    class_defs: &HashMap<String, StyleProps>,
) -> SunburstData {
    let n = &nodes[idx];
    let eff = resolve_style(&n.classes, &n.inline_style, class_defs);
    let children: Vec<SunburstData> = n
        .children
        .iter()
        .map(|&c| build_sunburst_node(c, nodes, class_defs))
        .collect();
    let value = if children.is_empty() {
        n.value.unwrap_or(0.0)
    } else {
        children.iter().map(|c| c.value).sum()
    };
    SunburstData {
        name: n.name.clone(),
        value,
        children,
        color: eff.fill,
    }
}

pub(crate) fn render(
    chart: &MermaidChart,
    theme_name: &str,
    fm_title: Option<&str>,
    lines: &[SourceLine],
) -> Result<(String, f32, f32)> {
    let parsed = parse_treemap(lines)?;
    let series_data: Vec<SunburstData> = parsed
        .roots
        .iter()
        .map(|&r| build_sunburst_node(r, &parsed.nodes, &parsed.class_defs))
        .collect();
    let total: f32 = series_data.iter().map(|r| r.value).sum();
    if !total.is_finite() || total <= 0.0 {
        return Err(Error::Mermaid {
            line: lines.first().map(|l| l.line_no),
            message: "treemap has no positive leaf values".to_string(),
        });
    }

    let mut treemap = TreemapChart::new_with_theme(vec![], theme_name);
    treemap.series_data = series_data;

    let effective_title = parsed.title.as_deref().or(fm_title);
    apply_base_options(
        chart,
        &mut treemap.base,
        theme_name,
        effective_title,
        700.0,
        480.0,
    );
    let (w, h) = (treemap.width, treemap.height);
    Ok((treemap.svg()?, w, h))
}

#[cfg(test)]
mod tests {
    use super::super::super::preprocess::preprocess;
    use super::*;

    #[test]
    fn renders_hierarchical_treemap_with_classes() {
        let src = r#"
treemap-beta
    title Budget Allocation 2025
    "Engineering"
        "Core Infra": 420
        "Platform API": 280
    "Product"
        "Design": 160:::highlight
        "Research": 140
    classDef highlight fill:#0ea5e9
"#;
        let prep = preprocess(src).unwrap();
        let chart = MermaidChart::new(src);
        let (svg, w, h) = render(&chart, "light", prep.title.as_deref(), &prep.lines).unwrap();
        assert!(svg.starts_with("<svg") && w >= 600.0 && h >= 400.0);
        assert!(svg.contains("Budget Allocation 2025"));
        assert!(svg.contains("Core Infra"));
        assert!(svg.contains("Platform API"));
        assert!(svg.contains("Design"));
        assert!(
            svg.contains("#0ea5e9")
                || svg.contains("#0EA5E9")
                || svg.contains("14, 165, 233")
                || svg.contains("14,165,233"),
            "{svg}"
        );
    }

    #[test]
    fn reports_line_number_on_bad_treemap_value() {
        let src = "treemap-beta\n  \"Section\"\n    \"Leaf\": nope";
        let prep = preprocess(src).unwrap();
        let err = parse_treemap(&prep.lines).unwrap_err().to_string();
        assert!(err.contains("line 3"), "{err}");
    }
}
