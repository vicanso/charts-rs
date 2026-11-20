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

//! Mermaid `sankey-beta` diagram adapter -> [`SankeyChart`](crate::SankeyChart).

use super::super::super::error::{Error, Result};
use super::super::super::sankey_chart::{SankeyChart, SankeyLink};
use super::super::MermaidChart;
use super::super::preprocess::SourceLine;
use super::apply_base_options;

fn parse_csv_row(line: &str, line_no: usize) -> Result<Vec<String>> {
    let mut fields = Vec::new();
    let mut cur = String::new();
    let chars: Vec<char> = line.chars().collect();
    let mut i = 0;
    let mut in_quotes = false;
    let mut at_field_start = true;

    while i < chars.len() {
        let ch = chars[i];
        if in_quotes {
            if ch == '"' {
                if i + 1 < chars.len() && chars[i + 1] == '"' {
                    cur.push('"');
                    i += 2;
                    continue;
                }
                in_quotes = false;
            } else {
                cur.push(ch);
            }
            i += 1;
            continue;
        }
        if at_field_start && ch == '"' {
            in_quotes = true;
            at_field_start = false;
            i += 1;
            continue;
        }
        if ch == ',' {
            fields.push(cur.trim().to_string());
            cur.clear();
            at_field_start = true;
        } else {
            cur.push(ch);
            if !ch.is_whitespace() {
                at_field_start = false;
            }
        }
        i += 1;
    }

    if in_quotes {
        return Err(Error::Mermaid {
            line: Some(line_no),
            message: "unclosed quote in sankey CSV row".to_string(),
        });
    }
    fields.push(cur.trim().to_string());
    Ok(fields)
}

pub(crate) fn render(
    chart: &MermaidChart,
    theme_name: &str,
    fm_title: Option<&str>,
    lines: &[SourceLine],
) -> Result<(String, f32, f32)> {
    let mut links = Vec::new();
    for sl in lines {
        let text = sl.text.trim();
        if text.is_empty() {
            continue;
        }
        let cols = parse_csv_row(text, sl.line_no)?;
        if cols.len() != 3 {
            return Err(Error::Mermaid {
                line: Some(sl.line_no),
                message: format!(
                    "sankey row must have 3 CSV columns (`source,target,value`), got {}",
                    cols.len()
                ),
            });
        }
        if cols[0].is_empty() || cols[1].is_empty() {
            return Err(Error::Mermaid {
                line: Some(sl.line_no),
                message: "sankey source and target cannot be empty".to_string(),
            });
        }
        let val: f32 = cols[2].parse().map_err(|_| Error::Mermaid {
            line: Some(sl.line_no),
            message: format!("invalid numeric value `{}` in sankey link", cols[2]),
        })?;
        if !val.is_finite() || val < 0.0 {
            return Err(Error::Mermaid {
                line: Some(sl.line_no),
                message: format!("sankey link value must be non-negative, got `{}`", cols[2]),
            });
        }
        links.push(SankeyLink {
            source: cols[0].clone(),
            target: cols[1].clone(),
            value: val,
        });
    }

    if links.is_empty() {
        return Err(Error::Mermaid {
            line: lines.first().map(|l| l.line_no),
            message: "sankey diagram has no links".to_string(),
        });
    }

    let mut sankey = SankeyChart::new_with_theme(vec![], links, theme_name);
    apply_base_options(chart, &mut sankey.base, theme_name, fm_title, 640.0, 420.0);
    let (w, h) = (sankey.width, sankey.height);
    Ok((sankey.svg()?, w, h))
}
