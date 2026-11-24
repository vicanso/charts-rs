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

//! Mermaid `pie` diagram adapter -> [`PieChart`](crate::PieChart).

use super::super::super::color::Color;
use super::super::super::common::Series;
use super::super::super::error::{Error, Result};
use super::super::super::font::measure_text_width_family;
use super::super::super::pie_chart::PieChart;
use super::super::super::util::format_float;
use super::super::MermaidChart;
use super::super::preprocess::SourceLine;
use super::super::style::mix_color;
use super::apply_base_options;

/// Extends `colors` to at least `n` entries by mixing wrapped base colors
/// alternately toward white (odd cycles) and black (even cycles), preserving
/// alpha.
fn extend_palette(colors: &[Color], n: usize) -> Vec<Color> {
    if colors.is_empty() || n <= colors.len() {
        return colors.to_vec();
    }
    let mut out = Vec::with_capacity(n);
    out.extend_from_slice(colors);
    for i in colors.len()..n {
        let k = i / colors.len();
        let c = colors[i % colors.len()];
        let derived = if k % 2 == 1 {
            mix_color(c, (255, 255, 255, c.a).into(), 0.45)
        } else {
            mix_color(c, (0, 0, 0, c.a).into(), 0.30)
        };
        out.push(derived);
    }
    out
}

fn parse_pie_entry(line: &str, line_no: usize) -> Result<Series> {
    let trimmed = line.trim();
    let (label, value_str) = if let Some(rest) = trimmed.strip_prefix('"') {
        let Some(end_quote) = rest.find('"') else {
            return Err(Error::Mermaid {
                line: Some(line_no),
                message: "unclosed quote in pie entry".to_string(),
            });
        };
        let label = rest[..end_quote].trim().to_string();
        let after = rest[end_quote + 1..].trim();
        let Some(val) = after.strip_prefix(':') else {
            return Err(Error::Mermaid {
                line: Some(line_no),
                message: format!("expected `:` after pie label `{label}`"),
            });
        };
        (label, val.trim())
    } else if let Some((left, right)) = trimmed.split_once(':') {
        (left.trim().to_string(), right.trim())
    } else {
        return Err(Error::Mermaid {
            line: Some(line_no),
            message: format!("invalid pie statement `{trimmed}`"),
        });
    };

    if label.is_empty() {
        return Err(Error::Mermaid {
            line: Some(line_no),
            message: "empty label in pie entry".to_string(),
        });
    }
    let val: f32 = value_str.parse().map_err(|_| Error::Mermaid {
        line: Some(line_no),
        message: format!("invalid numeric value `{value_str}` in pie entry"),
    })?;
    if !val.is_finite() || val < 0.0 {
        return Err(Error::Mermaid {
            line: Some(line_no),
            message: format!("pie value must be a non-negative finite number, got `{value_str}`"),
        });
    }
    Ok(Series::new(label, vec![val]))
}

pub(crate) fn render(
    chart: &MermaidChart,
    theme_name: &str,
    fm_title: Option<&str>,
    show_data: bool,
    inline_title: Option<&str>,
    lines: &[SourceLine],
) -> Result<(String, f32, f32)> {
    let mut title = inline_title
        .map(str::to_string)
        .or_else(|| fm_title.map(str::to_string));
    let mut series_list = Vec::new();
    let mut in_acc_descr_block = false;

    for sl in lines {
        let text = sl.text.trim();
        if in_acc_descr_block {
            if text.contains('}') {
                in_acc_descr_block = false;
            }
            continue;
        }
        if text.starts_with("accTitle") {
            continue;
        }
        if text.starts_with("accDescr") {
            if text.contains('{') && !text.contains('}') {
                in_acc_descr_block = true;
            }
            continue;
        }
        if let Some(rest) = text.strip_prefix("title")
            && (rest.is_empty() || rest.starts_with(char::is_whitespace))
        {
            let t = rest.trim();
            if !t.is_empty() {
                title = Some(t.to_string());
            }
            continue;
        }
        series_list.push(parse_pie_entry(text, sl.line_no)?);
    }

    if series_list.is_empty() {
        return Err(Error::Mermaid {
            line: lines.first().map(|l| l.line_no),
            message: "pie diagram has no data entries".to_string(),
        });
    }

    let total: f32 = series_list
        .iter()
        .map(|s| s.data.iter().flatten().sum::<f32>())
        .sum();
    if !total.is_finite() || total <= 0.0 {
        return Err(Error::Mermaid {
            line: lines.first().map(|l| l.line_no),
            message: "pie diagram has no positive values".to_string(),
        });
    }

    let mut pie = PieChart::new_with_theme(series_list, theme_name);
    let n = pie.series_list.len();
    if !pie.series.colors.is_empty() && n > pie.series.colors.len() {
        pie.series.colors = extend_palette(&pie.series.colors, n);
    }
    pie.rose_type = Some(false);
    pie.inner_radius = 0.0;
    pie.legend.show = Some(true);
    pie.avoid_label_overlap = true;
    pie.min_show_label_angle = 3.0;
    if show_data {
        pie.series.label.formatter = "{a}: {c} ({d})".to_string();
    }
    let label_font = if !chart.base.font_family.is_empty() {
        chart.base.font_family.as_str()
    } else {
        pie.font_family.as_str()
    };
    let label_size = pie.series.label.font.size;
    let mut max_label_w = 0.0_f32;
    for s in &pie.series_list {
        let v: f32 = s.data.iter().flatten().sum();
        let pct = (v / total * 100.0).round() as i32;
        let sample = if show_data {
            format!("{}: {} ({pct}%)", s.name, format_float(v))
        } else {
            format!("{}: {pct}%", s.name)
        };
        if let Ok(b) = measure_text_width_family(label_font, label_size, &sample) {
            max_label_w = max_label_w.max(b.width());
        }
    }
    let default_width = (2.0 * (135.0 + 48.0 + max_label_w + 16.0)).clamp(640.0, 960.0);
    apply_base_options(chart, &mut pie.base, title.as_deref(), default_width, 400.0);
    let (w, h) = (pie.width, pie.height);
    Ok((pie.svg()?, w, h))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn extend_palette_derives_distinct_tint_and_shade_cycles() {
        let base = vec![
            Color::from((84, 112, 198, 200)),
            Color::from((145, 204, 117, 200)),
        ];
        // Short targets return the base palette untouched.
        assert_eq!(extend_palette(&base, 2), base);
        assert!(extend_palette(&[], 4).is_empty());

        let extended = extend_palette(&base, 6);
        assert_eq!(extended.len(), 6);
        // Cycle 1 (odd) tints toward white, preserving alpha.
        assert_eq!(extended[2].a, 200);
        assert!(extended[2].r > base[0].r);
        // Cycle 2 (even) shades toward black, preserving alpha.
        assert_eq!(extended[4].a, 200);
        assert!(extended[4].r < base[0].r);

        let unique: HashSet<(u8, u8, u8, u8)> =
            extended.iter().map(|c| (c.r, c.g, c.b, c.a)).collect();
        assert_eq!(unique.len(), 6);
    }

    #[test]
    fn many_slice_pie_assigns_distinct_slice_colors() {
        let chart = MermaidChart::from_json(include_str!(
            "../../../../asset/mermaid_chart/pie_many_slices.json"
        ))
        .unwrap();
        let svg = chart.svg().unwrap();
        let fills: Vec<&str> = svg
            .lines()
            .filter(|l| l.starts_with("<path ") && l.contains("data-series=\""))
            .map(|l| {
                let start = l.find("fill=\"").unwrap() + 6;
                let end = start + l[start..].find('"').unwrap();
                &l[start..end]
            })
            .collect();
        assert_eq!(fills.len(), 12, "{svg}");
        let unique: HashSet<&str> = fills.iter().copied().collect();
        assert_eq!(unique.len(), 12, "fills={fills:?}");
    }
}
