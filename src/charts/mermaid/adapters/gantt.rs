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

//! Mermaid `gantt` diagram adapter -> [`GanttChart`](crate::GanttChart).

use super::super::super::color::Color;
use super::super::super::error::{Error, Result};
use super::super::super::gantt_chart::{GanttChart, GanttTask};
use super::super::super::x_axis::{days_from_civil, parse_time};
use super::super::MermaidChart;
use super::super::preprocess::SourceLine;
use super::apply_base_options;
use std::collections::HashMap;

const DEFAULT_DAY_SECONDS: f64 = 86_400.0;

#[derive(Clone, Debug)]
enum StartSpec {
    Timestamp(f64),
    After(Vec<String>),
    Sequential,
}

#[derive(Clone, Debug)]
enum EndSpec {
    Timestamp(f64),
    Duration(f64),
}

#[derive(Clone, Debug)]
struct ParsedTask {
    line_no: usize,
    name: String,
    id: Option<String>,
    section: Option<String>,
    is_done: bool,
    is_active: bool,
    is_crit: bool,
    is_milestone: bool,
    start: StartSpec,
    end: EndSpec,
}

/// Parses a date/time string according to a Mermaid `dateFormat` pattern
/// (`YYYY`, `YY`, `MM`, `DD`, `HH`, `mm`, `ss`), falling back to `parse_time`.
fn parse_date_with_format(input: &str, fmt: &str) -> Option<f64> {
    let input = input.trim();
    if input.is_empty() {
        return None;
    }
    if (fmt.is_empty() || fmt == "YYYY-MM-DD")
        && let Some(ts) = parse_time(input)
    {
        return Some(ts);
    }

    let fb = fmt.as_bytes();
    let ib = input.as_bytes();
    let mut fi = 0;
    let mut ii = 0;
    let mut year: Option<i64> = None;
    let mut month: Option<i64> = None;
    let mut day: Option<i64> = None;
    let mut hour: i64 = 0;
    let mut minute: i64 = 0;
    let mut second: i64 = 0;
    let mut matched_token = false;

    let take_digits = |bytes: &[u8], pos: &mut usize, max_len: usize| -> Option<i64> {
        let start = *pos;
        while *pos < bytes.len() && (*pos - start) < max_len && bytes[*pos].is_ascii_digit() {
            *pos += 1;
        }
        if *pos == start {
            return None;
        }
        std::str::from_utf8(&bytes[start..*pos]).ok()?.parse().ok()
    };

    while fi < fb.len() {
        if fmt[fi..].starts_with("YYYY") {
            year = Some(take_digits(ib, &mut ii, 4)?);
            fi += 4;
            matched_token = true;
        } else if fmt[fi..].starts_with("YY") {
            let yy = take_digits(ib, &mut ii, 2)?;
            year = Some(2000 + yy);
            fi += 2;
            matched_token = true;
        } else if fmt[fi..].starts_with("MM") {
            month = Some(take_digits(ib, &mut ii, 2)?);
            fi += 2;
            matched_token = true;
        } else if fmt[fi..].starts_with("DD") {
            day = Some(take_digits(ib, &mut ii, 2)?);
            fi += 2;
            matched_token = true;
        } else if fmt[fi..].starts_with("HH") {
            hour = take_digits(ib, &mut ii, 2)?;
            fi += 2;
            matched_token = true;
        } else if fmt[fi..].starts_with("mm") {
            minute = take_digits(ib, &mut ii, 2)?;
            fi += 2;
            matched_token = true;
        } else if fmt[fi..].starts_with("ss") {
            second = take_digits(ib, &mut ii, 2)?;
            fi += 2;
            matched_token = true;
        } else {
            if ii >= ib.len() || ib[ii] != fb[fi] {
                return parse_time(input);
            }
            fi += 1;
            ii += 1;
        }
    }

    if matched_token && ii == ib.len() {
        let y = year?;
        let m = month.unwrap_or(1);
        let d = day.unwrap_or(1);
        if (1..=12).contains(&m)
            && (1..=31).contains(&d)
            && (0..24).contains(&hour)
            && (0..60).contains(&minute)
            && (0..60).contains(&second)
        {
            let secs = days_from_civil(y, m, d) as f64 * DEFAULT_DAY_SECONDS
                + hour as f64 * 3600.0
                + minute as f64 * 60.0
                + second as f64;
            return Some(secs);
        }
        return None;
    }

    parse_time(input)
}

fn parse_duration(token: &str) -> Option<f64> {
    let t = token.trim();
    if let Some(num) = t.strip_suffix("ms") {
        let v: f64 = num.trim().parse().ok()?;
        return (v.is_finite() && v >= 0.0).then_some(v / 1000.0);
    }
    let split = t.len().checked_sub(1)?;
    let (num, unit) = t.split_at(split);
    let v: f64 = num.trim().parse().ok()?;
    if !v.is_finite() || v < 0.0 {
        return None;
    }
    let scale = match unit {
        "s" => 1.0,
        "m" => 60.0,
        "h" => 3600.0,
        "d" => DEFAULT_DAY_SECONDS,
        "w" => DEFAULT_DAY_SECONDS * 7.0,
        _ => return None,
    };
    Some(v * scale)
}

fn parse_after(token: &str) -> Option<Vec<String>> {
    let trimmed = token.trim();
    let rest = if trimmed.len() >= 5 && trimmed[..5].eq_ignore_ascii_case("after") {
        &trimmed[5..]
    } else {
        return None;
    };
    if !rest.starts_with(char::is_whitespace) {
        return None;
    }
    let ids: Vec<String> = rest.split_whitespace().map(str::to_string).collect();
    (!ids.is_empty()).then_some(ids)
}

fn parse_task_line(
    line: &str,
    line_no: usize,
    date_format: &str,
    section: Option<String>,
) -> Result<ParsedTask> {
    let Some((raw_name, raw_meta)) = line.split_once(':') else {
        return Err(Error::Mermaid {
            line: Some(line_no),
            message: format!("invalid gantt statement `{line}`"),
        });
    };
    let name = raw_name.trim().to_string();
    if name.is_empty() {
        return Err(Error::Mermaid {
            line: Some(line_no),
            message: "empty task name in gantt diagram".to_string(),
        });
    }

    let parts: Vec<&str> = raw_meta.split(',').map(str::trim).collect();
    let mut is_done = false;
    let mut is_active = false;
    let mut is_crit = false;
    let mut is_milestone = false;
    let mut idx = 0;

    while idx < parts.len() {
        match parts[idx].to_ascii_lowercase().as_str() {
            "done" => is_done = true,
            "active" => is_active = true,
            "crit" => is_crit = true,
            "milestone" => is_milestone = true,
            _ => break,
        }
        idx += 1;
    }

    let rem = &parts[idx..];
    if rem.is_empty() {
        return Err(Error::Mermaid {
            line: Some(line_no),
            message: format!("missing schedule specification for gantt task `{name}`"),
        });
    }

    let parse_start = |tok: &str| -> Result<StartSpec> {
        if let Some(ids) = parse_after(tok) {
            Ok(StartSpec::After(ids))
        } else if let Some(ts) = parse_date_with_format(tok, date_format) {
            Ok(StartSpec::Timestamp(ts))
        } else {
            Err(Error::Mermaid {
                line: Some(line_no),
                message: format!("invalid gantt start date or dependency `{tok}`"),
            })
        }
    };

    let parse_end = |tok: &str| -> Result<EndSpec> {
        if let Some(dur) = parse_duration(tok) {
            Ok(EndSpec::Duration(dur))
        } else if let Some(ts) = parse_date_with_format(tok, date_format) {
            Ok(EndSpec::Timestamp(ts))
        } else {
            Err(Error::Mermaid {
                line: Some(line_no),
                message: format!("invalid gantt end date or duration `{tok}`"),
            })
        }
    };

    let (id, start, end) = match rem.len() {
        1 => {
            if let Some(dur) = parse_duration(rem[0]) {
                (None, StartSpec::Sequential, EndSpec::Duration(dur))
            } else {
                let s = parse_start(rem[0])?;
                let default_dur = if is_milestone {
                    0.0
                } else {
                    DEFAULT_DAY_SECONDS
                };
                (None, s, EndSpec::Duration(default_dur))
            }
        }
        2 => {
            if parse_after(rem[0]).is_some()
                || parse_date_with_format(rem[0], date_format).is_some()
            {
                (None, parse_start(rem[0])?, parse_end(rem[1])?)
            } else if let Some(dur) = parse_duration(rem[1]) {
                (
                    (!rem[0].is_empty()).then(|| rem[0].to_string()),
                    StartSpec::Sequential,
                    EndSpec::Duration(dur),
                )
            } else {
                let s = parse_start(rem[1])?;
                let default_dur = if is_milestone {
                    0.0
                } else {
                    DEFAULT_DAY_SECONDS
                };
                (
                    (!rem[0].is_empty()).then(|| rem[0].to_string()),
                    s,
                    EndSpec::Duration(default_dur),
                )
            }
        }
        3 => (
            (!rem[0].is_empty()).then(|| rem[0].to_string()),
            parse_start(rem[1])?,
            parse_end(rem[2])?,
        ),
        _ => {
            return Err(Error::Mermaid {
                line: Some(line_no),
                message: format!("too many fields in gantt task `{name}`"),
            });
        }
    };

    Ok(ParsedTask {
        line_no,
        name,
        id,
        section,
        is_done,
        is_active,
        is_crit,
        is_milestone,
        start,
        end,
    })
}

fn resolve_task(
    idx: usize,
    parsed: &[ParsedTask],
    id_map: &HashMap<String, usize>,
    state: &mut [u8],
    resolved: &mut [Option<(f64, f64)>],
) -> Result<(f64, f64)> {
    if let Some(span) = resolved[idx] {
        return Ok(span);
    }
    if state[idx] == 1 {
        return Err(Error::Mermaid {
            line: Some(parsed[idx].line_no),
            message: format!("cyclic `after` dependency on task `{}`", parsed[idx].name),
        });
    }
    state[idx] = 1;
    let task = &parsed[idx];

    let start = match &task.start {
        StartSpec::Timestamp(ts) => *ts,
        StartSpec::Sequential => {
            if idx == 0 {
                return Err(Error::Mermaid {
                    line: Some(task.line_no),
                    message: format!("task `{}` has no start date or predecessor", task.name),
                });
            }
            resolve_task(idx - 1, parsed, id_map, state, resolved)?.1
        }
        StartSpec::After(deps) => {
            let mut max_end = f64::NEG_INFINITY;
            for dep in deps {
                let Some(&dep_idx) = id_map.get(dep) else {
                    return Err(Error::Mermaid {
                        line: Some(task.line_no),
                        message: format!("unknown task id `{dep}` in `after` clause"),
                    });
                };
                let (_, dep_end) = resolve_task(dep_idx, parsed, id_map, state, resolved)?;
                max_end = max_end.max(dep_end);
            }
            max_end
        }
    };

    let end = if task.is_milestone {
        start
    } else {
        match task.end {
            EndSpec::Timestamp(ts) => {
                if ts < start {
                    return Err(Error::Mermaid {
                        line: Some(task.line_no),
                        message: format!("gantt task `{}` ends before it starts", task.name),
                    });
                }
                ts
            }
            EndSpec::Duration(dur) => start + dur,
        }
    };

    state[idx] = 2;
    resolved[idx] = Some((start, end));
    Ok((start, end))
}

pub(crate) fn render(
    chart: &MermaidChart,
    theme_name: &str,
    fm_title: Option<&str>,
    lines: &[SourceLine],
) -> Result<(String, f32, f32)> {
    let mut title = fm_title.map(str::to_string);
    let mut date_format = "YYYY-MM-DD".to_string();
    let mut axis_format: Option<String> = None;
    let mut today_marker: Option<String> = None;
    let mut current_section: Option<String> = None;
    let mut parsed_tasks = Vec::new();
    let mut in_acc_descr_block = false;

    for sl in lines {
        let text = sl.text.trim();
        if in_acc_descr_block {
            if text.contains('}') {
                in_acc_descr_block = false;
            }
            continue;
        }
        let lower = text.to_ascii_lowercase();
        if lower.starts_with("acctitle")
            || lower.starts_with("excludes")
            || lower.starts_with("includes")
            || lower.starts_with("tickinterval")
            || lower.starts_with("weekday")
            || lower.starts_with("click ")
        {
            continue;
        }
        if lower.starts_with("accdescr") {
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
        if let Some(rest) = text.strip_prefix("dateFormat")
            && rest.starts_with(char::is_whitespace)
        {
            date_format = rest.trim().to_string();
            continue;
        }
        if let Some(rest) = text.strip_prefix("axisFormat")
            && rest.starts_with(char::is_whitespace)
        {
            axis_format = Some(rest.trim().to_string());
            continue;
        }
        if let Some(rest) = text.strip_prefix("todayMarker")
            && rest.starts_with(char::is_whitespace)
        {
            let v = rest.trim();
            if !v.eq_ignore_ascii_case("off") {
                today_marker = Some(v.to_string());
            } else {
                today_marker = None;
            }
            continue;
        }
        if let Some(rest) = text.strip_prefix("section")
            && (rest.is_empty() || rest.starts_with(char::is_whitespace))
        {
            let sec = rest.trim();
            current_section = (!sec.is_empty()).then(|| sec.to_string());
            continue;
        }

        parsed_tasks.push(parse_task_line(
            text,
            sl.line_no,
            &date_format,
            current_section.clone(),
        )?);
    }

    if parsed_tasks.is_empty() {
        return Err(Error::Mermaid {
            line: lines.first().map(|l| l.line_no),
            message: "gantt diagram has no tasks".to_string(),
        });
    }

    let mut id_map = HashMap::new();
    for (i, t) in parsed_tasks.iter().enumerate() {
        if let Some(ref id) = t.id {
            id_map.insert(id.clone(), i);
        }
    }

    let mut state = vec![0u8; parsed_tasks.len()];
    let mut resolved = vec![None; parsed_tasks.len()];
    for i in 0..parsed_tasks.len() {
        resolve_task(i, &parsed_tasks, &id_map, &mut state, &mut resolved)?;
    }

    let crit_color: Color = (220, 68, 68).into();
    let tasks: Vec<GanttTask> = parsed_tasks
        .into_iter()
        .zip(resolved)
        .map(|(t, span)| {
            let (start, end) = span.unwrap_or((0.0, 0.0));
            let progress = if t.is_done {
                Some(1.0)
            } else if t.is_active {
                Some(0.5)
            } else {
                None
            };
            GanttTask {
                name: t.name,
                start,
                end,
                row: None,
                category: t.section,
                progress,
                color: t.is_crit.then_some(crit_color),
            }
        })
        .collect();

    let default_height = (tasks.len() as f32 * 36.0 + 130.0).clamp(240.0, 1200.0);
    let mut gantt = GanttChart::new_with_theme(tasks, theme_name);
    if let Some(fmt) = axis_format {
        gantt.x_axis.formatter = Some(fmt);
    }
    if let Some(marker) = today_marker
        && let Some(ts) = parse_date_with_format(&marker, &date_format)
    {
        gantt.now = Some(ts);
    }

    apply_base_options(
        chart,
        &mut gantt.base,
        title.as_deref(),
        680.0,
        default_height,
    );
    let (w, h) = (gantt.width, gantt.height);
    Ok((gantt.svg()?, w, h))
}
