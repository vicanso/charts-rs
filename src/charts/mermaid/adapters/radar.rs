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

//! Mermaid `radar-beta` / `radar` diagram adapter -> [`RadarChart`](crate::RadarChart).

use std::collections::HashMap;

use super::super::super::common::Series;
use super::super::super::error::{Error, Result};
use super::super::super::radar_chart::{RadarChart, RadarIndicator, RadarShape};
use super::super::MermaidChart;
use super::super::preprocess::SourceLine;
use super::super::style::unquote_text;
use super::apply_base_options;

#[derive(Clone, Debug)]
struct RadarAxisSpec {
    id: String,
    label: String,
}

#[derive(Clone, Debug)]
enum CurveValues {
    Positional(Vec<f32>),
    Keyed(Vec<(String, f32)>),
}

#[derive(Clone, Debug)]
struct RadarCurveSpec {
    id: String,
    label: String,
    values: CurveValues,
}

#[derive(Debug, Default)]
struct ParsedRadar {
    title: Option<String>,
    axes: Vec<RadarAxisSpec>,
    curves: Vec<RadarCurveSpec>,
    shape: RadarShape,
    min: Option<f32>,
    max: Option<f32>,
    ticks: Option<usize>,
    show_legend: Option<bool>,
}

fn split_top_level_commas(raw: &str) -> Vec<&str> {
    let mut parts = Vec::new();
    let mut start = 0usize;
    let mut in_quotes = false;
    let mut bracket_depth = 0usize;
    let mut brace_depth = 0usize;

    for (i, ch) in raw.char_indices() {
        if in_quotes {
            if ch == '"' {
                in_quotes = false;
            }
            continue;
        }
        match ch {
            '"' => in_quotes = true,
            '[' => bracket_depth += 1,
            ']' => bracket_depth = bracket_depth.saturating_sub(1),
            '{' => brace_depth += 1,
            '}' => brace_depth = brace_depth.saturating_sub(1),
            ',' if bracket_depth == 0 && brace_depth == 0 => {
                let s = raw[start..i].trim();
                if !s.is_empty() {
                    parts.push(s);
                }
                start = i + 1;
            }
            _ => {}
        }
    }
    let tail = raw[start..].trim();
    if !tail.is_empty() {
        parts.push(tail);
    }
    parts
}

fn parse_id_and_opt_label(raw: &str, line_no: usize) -> Result<(String, String)> {
    let trimmed = raw.trim();
    if let Some(open_idx) = trimmed.find('[') {
        let id = trimmed[..open_idx].trim();
        let rest = &trimmed[open_idx..];
        let inner = rest
            .strip_prefix('[')
            .and_then(|s| s.strip_suffix(']'))
            .ok_or_else(|| Error::Mermaid {
                line: Some(line_no),
                message: format!("unclosed `[label]` in radar item `{trimmed}`"),
            })?;
        if id.is_empty() {
            return Err(Error::Mermaid {
                line: Some(line_no),
                message: format!("missing identifier before `[...]` in `{trimmed}`"),
            });
        }
        let label = unquote_text(inner.trim());
        Ok((
            id.to_string(),
            if label.is_empty() {
                id.to_string()
            } else {
                label
            },
        ))
    } else if !trimmed.is_empty() {
        let id = unquote_text(trimmed);
        Ok((id.clone(), id))
    } else {
        Err(Error::Mermaid {
            line: Some(line_no),
            message: "empty radar axis or curve identifier".to_string(),
        })
    }
}

fn parse_curve_item(raw: &str, line_no: usize) -> Result<RadarCurveSpec> {
    let trimmed = raw.trim();
    let brace_open = trimmed.find('{').ok_or_else(|| Error::Mermaid {
        line: Some(line_no),
        message: format!("expected `{{...}}` values in radar curve `{trimmed}`"),
    })?;
    let after_open = &trimmed[brace_open + 1..];
    let inner = after_open.strip_suffix('}').ok_or_else(|| Error::Mermaid {
        line: Some(line_no),
        message: format!("unclosed `{{` in radar curve `{trimmed}`"),
    })?;

    let head = trimmed[..brace_open].trim();
    let (id, label) = parse_id_and_opt_label(head, line_no)?;

    let entries = split_top_level_commas(inner);
    if entries.is_empty() {
        return Err(Error::Mermaid {
            line: Some(line_no),
            message: format!("radar curve `{id}` has no values"),
        });
    }

    let is_keyed = entries.iter().any(|e| e.contains(':'));
    let values = if is_keyed {
        let mut keyed = Vec::with_capacity(entries.len());
        for entry in entries {
            let Some((k, v)) = entry.split_once(':') else {
                return Err(Error::Mermaid {
                    line: Some(line_no),
                    message: format!("mixed positional and keyed entries in radar curve `{id}`"),
                });
            };
            let key = unquote_text(k.trim());
            let val: f32 = v.trim().parse().map_err(|_| Error::Mermaid {
                line: Some(line_no),
                message: format!("invalid number `{}` in radar curve `{id}`", v.trim()),
            })?;
            if !val.is_finite() {
                return Err(Error::Mermaid {
                    line: Some(line_no),
                    message: format!("non-finite number in radar curve `{id}`"),
                });
            }
            keyed.push((key, val));
        }
        CurveValues::Keyed(keyed)
    } else {
        let mut vals = Vec::with_capacity(entries.len());
        for entry in entries {
            let val: f32 = entry.trim().parse().map_err(|_| Error::Mermaid {
                line: Some(line_no),
                message: format!("invalid number `{}` in radar curve `{id}`", entry.trim()),
            })?;
            if !val.is_finite() {
                return Err(Error::Mermaid {
                    line: Some(line_no),
                    message: format!("non-finite number in radar curve `{id}`"),
                });
            }
            vals.push(val);
        }
        CurveValues::Positional(vals)
    };

    Ok(RadarCurveSpec { id, label, values })
}

fn parse_radar(lines: &[SourceLine]) -> Result<ParsedRadar> {
    let mut parsed = ParsedRadar::default();
    let mut in_acc_descr = false;

    for sl in lines {
        let text = sl.text.trim();
        if text.is_empty() {
            continue;
        }
        if in_acc_descr {
            if text.contains('}') {
                in_acc_descr = false;
            }
            continue;
        }
        if text.starts_with("accTitle") {
            continue;
        }
        if text.starts_with("accDescr") {
            if text.contains('{') && !text.contains('}') {
                in_acc_descr = true;
            }
            continue;
        }

        if let Some(rest) = text.strip_prefix("title")
            && (rest.is_empty() || rest.starts_with(char::is_whitespace))
        {
            let t = unquote_text(rest.trim());
            if !t.is_empty() {
                parsed.title = Some(t);
            }
            continue;
        }

        if let Some(rest) = text.strip_prefix("axis")
            && rest.starts_with(char::is_whitespace)
        {
            for item in split_top_level_commas(rest) {
                let (id, label) = parse_id_and_opt_label(item, sl.line_no)?;
                if parsed.axes.iter().any(|a| a.id == id) {
                    return Err(Error::Mermaid {
                        line: Some(sl.line_no),
                        message: format!("duplicate radar axis `{id}`"),
                    });
                }
                parsed.axes.push(RadarAxisSpec { id, label });
            }
            continue;
        }

        if let Some(rest) = text.strip_prefix("curve")
            && rest.starts_with(char::is_whitespace)
        {
            for item in split_top_level_commas(rest) {
                let c = parse_curve_item(item, sl.line_no)?;
                if parsed.curves.iter().any(|existing| existing.id == c.id) {
                    return Err(Error::Mermaid {
                        line: Some(sl.line_no),
                        message: format!("duplicate radar curve `{}`", c.id),
                    });
                }
                parsed.curves.push(c);
            }
            continue;
        }

        if let Some(rest) = text.strip_prefix("graticule")
            && rest.starts_with(char::is_whitespace)
        {
            let kind = rest.trim().to_ascii_lowercase();
            parsed.shape = match kind.as_str() {
                "polygon" => RadarShape::Polygon,
                "circle" => RadarShape::Circle,
                _ => {
                    return Err(Error::Mermaid {
                        line: Some(sl.line_no),
                        message: format!("unknown radar graticule `{}`", rest.trim()),
                    });
                }
            };
            continue;
        }

        if let Some(rest) = text.strip_prefix("min")
            && rest.starts_with(char::is_whitespace)
        {
            let v: f32 = rest.trim().parse().map_err(|_| Error::Mermaid {
                line: Some(sl.line_no),
                message: format!("invalid radar min `{}`", rest.trim()),
            })?;
            if !v.is_finite() {
                return Err(Error::Mermaid {
                    line: Some(sl.line_no),
                    message: "radar min must be finite".to_string(),
                });
            }
            parsed.min = Some(v);
            continue;
        }

        if let Some(rest) = text.strip_prefix("max")
            && rest.starts_with(char::is_whitespace)
        {
            let v: f32 = rest.trim().parse().map_err(|_| Error::Mermaid {
                line: Some(sl.line_no),
                message: format!("invalid radar max `{}`", rest.trim()),
            })?;
            if !v.is_finite() {
                return Err(Error::Mermaid {
                    line: Some(sl.line_no),
                    message: "radar max must be finite".to_string(),
                });
            }
            parsed.max = Some(v);
            continue;
        }

        if let Some(rest) = text.strip_prefix("ticks")
            && rest.starts_with(char::is_whitespace)
        {
            let t: usize = rest.trim().parse().map_err(|_| Error::Mermaid {
                line: Some(sl.line_no),
                message: format!("invalid radar ticks `{}`", rest.trim()),
            })?;
            if t == 0 {
                return Err(Error::Mermaid {
                    line: Some(sl.line_no),
                    message: "radar ticks must be >= 1".to_string(),
                });
            }
            parsed.ticks = Some(t);
            continue;
        }

        if let Some(rest) = text.strip_prefix("showLegend")
            && rest.starts_with(char::is_whitespace)
        {
            match rest.trim().to_ascii_lowercase().as_str() {
                "true" => parsed.show_legend = Some(true),
                "false" => parsed.show_legend = Some(false),
                _ => {
                    return Err(Error::Mermaid {
                        line: Some(sl.line_no),
                        message: format!("invalid showLegend boolean `{}`", rest.trim()),
                    });
                }
            }
            continue;
        }

        return Err(Error::Mermaid {
            line: Some(sl.line_no),
            message: format!("unknown radar statement `{text}`"),
        });
    }

    if parsed.axes.len() < 3 {
        return Err(Error::Mermaid {
            line: lines.first().map(|l| l.line_no),
            message: "radar diagram requires at least 3 axes".to_string(),
        });
    }
    if parsed.curves.is_empty() {
        return Err(Error::Mermaid {
            line: lines.first().map(|l| l.line_no),
            message: "radar diagram requires at least 1 curve".to_string(),
        });
    }

    Ok(parsed)
}

pub(crate) fn render(
    chart: &MermaidChart,
    theme_name: &str,
    fm_title: Option<&str>,
    lines: &[SourceLine],
) -> Result<(String, f32, f32)> {
    let parsed = parse_radar(lines)?;
    let axis_index: HashMap<&str, usize> = parsed
        .axes
        .iter()
        .enumerate()
        .map(|(i, a)| (a.id.as_str(), i))
        .collect();

    let min_val = parsed.min.unwrap_or(0.0);
    let mut series_list = Vec::with_capacity(parsed.curves.len());
    let mut observed_max = min_val;

    for curve in &parsed.curves {
        let data = match &curve.values {
            CurveValues::Positional(vals) => {
                if vals.len() != parsed.axes.len() {
                    return Err(Error::Mermaid {
                        line: None,
                        message: format!(
                            "radar curve `{}` has {} values, expected {}",
                            curve.id,
                            vals.len(),
                            parsed.axes.len()
                        ),
                    });
                }
                vals.clone()
            }
            CurveValues::Keyed(pairs) => {
                let mut row = vec![min_val; parsed.axes.len()];
                for (k, v) in pairs {
                    let &idx = axis_index.get(k.as_str()).ok_or_else(|| Error::Mermaid {
                        line: None,
                        message: format!("unknown axis `{k}` in radar curve `{}`", curve.id),
                    })?;
                    row[idx] = *v;
                }
                row
            }
        };
        for &v in &data {
            if v > observed_max {
                observed_max = v;
            }
        }
        series_list.push(Series::new(curve.label.clone(), data));
    }

    let max_val = match parsed.max {
        Some(m) => {
            if m <= min_val {
                return Err(Error::Mermaid {
                    line: None,
                    message: format!("radar max ({m}) must be greater than min ({min_val})"),
                });
            }
            m
        }
        None => {
            if observed_max <= 100.0 && min_val >= 0.0 {
                100.0
            } else {
                (observed_max * 1.1).max(min_val + 1.0)
            }
        }
    };

    let indicators: Vec<RadarIndicator> = parsed
        .axes
        .iter()
        .map(|a| RadarIndicator {
            name: a.label.clone(),
            min: min_val,
            max: max_val,
        })
        .collect();

    let mut radar = RadarChart::new_with_theme(series_list, indicators, theme_name);
    radar.shape = parsed.shape;
    radar.visual_center = true;
    if let Some(t) = parsed.ticks {
        radar.split_number = t;
    }
    radar.legend.show = parsed.show_legend.or(Some(true));

    let effective_title = parsed.title.as_deref().or(fm_title);
    apply_base_options(
        chart,
        &mut radar.base,
        effective_title,
        700.0,
        480.0,
    );
    let (w, h) = (radar.width, radar.height);
    Ok((radar.svg()?, w, h))
}

#[cfg(test)]
mod tests {
    use super::super::super::preprocess::preprocess;
    use super::*;

    #[test]
    fn renders_positional_and_keyed_radar_curves() {
        let src = r#"
radar-beta
    title Service Quality Benchmark
    axis p["Latency"], t["Throughput"], r["Reliability"], d["DX"], c["Cost"]
    curve rust["Rust"]{94, 96, 95, 84, 90}
    curve go["Go"]{c: 88, p: 84, r: 91, t: 86, d: 90}
    graticule polygon
    min 0
    max 100
    ticks 5
    showLegend true
"#;
        let prep = preprocess(src).unwrap();
        let chart = MermaidChart::new(src);
        let (svg, w, h) = render(&chart, "light", prep.title.as_deref(), &prep.lines).unwrap();
        assert!(svg.starts_with("<svg") && w >= 600.0 && h >= 400.0);
        assert!(svg.contains("Service Quality Benchmark"));
        assert!(svg.contains("Throughput"));
        assert!(svg.contains("Rust"));
        assert!(svg.contains("Go"));
    }

    #[test]
    fn reports_line_number_on_bad_graticule() {
        let src = "radar-beta\n  axis a, b, c\n  graticule triangle\n  curve x{1,2,3}";
        let prep = preprocess(src).unwrap();
        let err = parse_radar(&prep.lines).unwrap_err().to_string();
        assert!(err.contains("line 3"), "{err}");
    }
}
