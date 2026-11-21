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

//! Mermaid `xychart-beta` diagram adapter -> [`BarChart`](crate::BarChart) /
//! [`HorizontalBarChart`](crate::HorizontalBarChart).

use super::super::super::bar_chart::BarChart;
use super::super::super::common::{Series, SeriesCategory};
use super::super::super::error::{Error, Result};
use super::super::super::horizontal_bar_chart::HorizontalBarChart;
use super::super::MermaidChart;
use super::super::preprocess::SourceLine;
use super::apply_base_options;

fn parse_optional_quoted_title(rest: &str) -> (Option<String>, &str) {
    let trimmed = rest.trim();
    if let Some(after_q) = trimmed.strip_prefix('"')
        && let Some(end) = after_q.find('"')
    {
        return (Some(after_q[..end].to_string()), after_q[end + 1..].trim());
    }
    (None, trimmed)
}

fn parse_bracket_items(raw: &str, line_no: usize) -> Result<Vec<String>> {
    let inner = raw
        .trim()
        .strip_prefix('[')
        .and_then(|s| s.strip_suffix(']'))
        .ok_or_else(|| Error::Mermaid {
            line: Some(line_no),
            message: format!("expected `[...]` list, got `{raw}`"),
        })?;

    let mut out = Vec::new();
    let mut cur = String::new();
    let mut in_quotes = false;
    for ch in inner.chars() {
        if in_quotes {
            if ch == '"' {
                in_quotes = false;
            } else {
                cur.push(ch);
            }
            continue;
        }
        match ch {
            '"' => in_quotes = true,
            ',' => {
                let s = cur.trim();
                if !s.is_empty() {
                    out.push(s.to_string());
                }
                cur.clear();
            }
            _ => cur.push(ch),
        }
    }
    if in_quotes {
        return Err(Error::Mermaid {
            line: Some(line_no),
            message: "unclosed quote in xychart array".to_string(),
        });
    }
    let tail = cur.trim();
    if !tail.is_empty() {
        out.push(tail.to_string());
    }
    Ok(out)
}

fn parse_number_array(raw: &str, line_no: usize) -> Result<Vec<Option<f32>>> {
    let items = parse_bracket_items(raw, line_no)?;
    let mut out = Vec::with_capacity(items.len());
    for item in items {
        if item.eq_ignore_ascii_case("null") {
            out.push(None);
        } else {
            let v: f32 = item.parse().map_err(|_| Error::Mermaid {
                line: Some(line_no),
                message: format!("invalid number `{item}` in xychart series"),
            })?;
            if !v.is_finite() {
                return Err(Error::Mermaid {
                    line: Some(line_no),
                    message: format!("non-finite number `{item}` in xychart series"),
                });
            }
            out.push(Some(v));
        }
    }
    Ok(out)
}

fn parse_range(raw: &str, line_no: usize) -> Result<(f32, f32)> {
    let Some((l, r)) = raw.split_once("-->") else {
        return Err(Error::Mermaid {
            line: Some(line_no),
            message: format!("expected `min --> max` range, got `{raw}`"),
        });
    };
    let min: f32 = l.trim().parse().map_err(|_| Error::Mermaid {
        line: Some(line_no),
        message: format!("invalid range minimum `{}`", l.trim()),
    })?;
    let max: f32 = r.trim().parse().map_err(|_| Error::Mermaid {
        line: Some(line_no),
        message: format!("invalid range maximum `{}`", r.trim()),
    })?;
    Ok((min, max))
}

pub(crate) fn render(
    chart: &MermaidChart,
    theme_name: &str,
    fm_title: Option<&str>,
    horizontal: bool,
    lines: &[SourceLine],
) -> Result<(String, f32, f32)> {
    let mut title = fm_title.map(str::to_string);
    let mut x_categories: Vec<String> = Vec::new();
    let mut x_title: Option<String> = None;
    let mut x_range: Option<(f32, f32)> = None;
    let mut y_title: Option<String> = None;
    let mut y_range: Option<(f32, f32)> = None;
    let mut series_list: Vec<Series> = Vec::new();
    let mut bar_count = 0usize;
    let mut line_count = 0usize;

    for sl in lines {
        let text = sl.text.trim();
        let lower = text.to_ascii_lowercase();
        if lower.starts_with("acctitle") || lower.starts_with("accdescr") {
            continue;
        }
        if let Some(rest) = text.strip_prefix("title")
            && rest.starts_with(char::is_whitespace)
        {
            let (quoted, raw) = parse_optional_quoted_title(rest);
            let t = quoted.unwrap_or_else(|| raw.to_string());
            if !t.is_empty() {
                title = Some(t);
            }
            continue;
        }
        if let Some(rest) = text.strip_prefix("x-axis")
            && (rest.is_empty() || rest.starts_with(char::is_whitespace))
        {
            let (maybe_title, after) = parse_optional_quoted_title(rest);
            if let Some(t) = maybe_title {
                x_title = Some(t);
            }
            if after.starts_with('[') {
                x_categories = parse_bracket_items(after, sl.line_no)?;
            } else if after.contains("-->") {
                x_range = Some(parse_range(after, sl.line_no)?);
            } else if !after.is_empty() && x_title.is_none() {
                x_title = Some(after.to_string());
            }
            continue;
        }
        if let Some(rest) = text.strip_prefix("y-axis")
            && (rest.is_empty() || rest.starts_with(char::is_whitespace))
        {
            let (maybe_title, after) = parse_optional_quoted_title(rest);
            if let Some(t) = maybe_title {
                y_title = Some(t);
            }
            if after.contains("-->") {
                y_range = Some(parse_range(after, sl.line_no)?);
            } else if !after.is_empty() && y_title.is_none() {
                y_title = Some(after.to_string());
            }
            continue;
        }
        if let Some(rest) = text.strip_prefix("bar")
            && rest.starts_with(char::is_whitespace)
        {
            bar_count += 1;
            let data = parse_number_array(rest, sl.line_no)?;
            series_list.push(Series {
                name: format!("Bar {bar_count}"),
                data,
                category: Some(SeriesCategory::Bar),
                ..Default::default()
            });
            continue;
        }
        if let Some(rest) = text.strip_prefix("line")
            && rest.starts_with(char::is_whitespace)
        {
            line_count += 1;
            let data = parse_number_array(rest, sl.line_no)?;
            series_list.push(Series {
                name: format!("Line {line_count}"),
                data,
                category: Some(SeriesCategory::Line),
                ..Default::default()
            });
            continue;
        }
        return Err(Error::Mermaid {
            line: Some(sl.line_no),
            message: format!("invalid xychart statement `{text}`"),
        });
    }

    if series_list.is_empty() {
        return Err(Error::Mermaid {
            line: lines.first().map(|l| l.line_no),
            message: "xychart diagram has no `bar` or `line` series".to_string(),
        });
    }

    if x_categories.is_empty() {
        let len = series_list.iter().map(|s| s.data.len()).max().unwrap_or(0);
        if let Some((min, max)) = x_range
            && len > 1
        {
            let step = (max - min) / (len - 1) as f32;
            x_categories = (0..len)
                .map(|i| {
                    let v = min + step * i as f32;
                    if (v - v.round()).abs() < 1e-4 {
                        format!("{}", v.round() as i64)
                    } else {
                        format!("{v:.1}")
                    }
                })
                .collect();
        } else {
            x_categories = (1..=len).map(|i| i.to_string()).collect();
        }
    }

    if horizontal {
        let mut hbar = HorizontalBarChart::new_with_theme(series_list, x_categories, theme_name);
        if let Some(xt) = x_title {
            hbar.y_axis_configs[0].title = Some(xt);
        }
        if let Some(yt) = y_title {
            hbar.x_axis.title = yt;
        }
        if let Some((min, max)) = y_range {
            hbar.y_axis_configs[0].min = Some(min);
            hbar.y_axis_configs[0].max = Some(max);
        }
        apply_base_options(
            chart,
            &mut hbar.base,
            title.as_deref(),
            640.0,
            400.0,
        );
        let (w, h) = (hbar.width, hbar.height);
        return Ok((hbar.svg()?, w, h));
    }

    let mut bar = BarChart::new_with_theme(series_list, x_categories, theme_name);
    if let Some(xt) = x_title {
        bar.x_axis.title = xt;
    }
    if let Some(yt) = y_title {
        bar.y_axis_configs[0].title = Some(yt);
    }
    if let Some((min, max)) = y_range {
        bar.y_axis_configs[0].min = Some(min);
        bar.y_axis_configs[0].max = Some(max);
    }
    apply_base_options(
        chart,
        &mut bar.base,
        title.as_deref(),
        640.0,
        400.0,
    );
    let (w, h) = (bar.width, bar.height);
    Ok((bar.svg()?, w, h))
}
