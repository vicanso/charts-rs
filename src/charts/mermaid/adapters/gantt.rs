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

use super::super::super::color::{Color, get_color};
use super::super::super::error::{Error, Result};
use super::super::super::gantt_chart::{GanttChart, GanttMarker, GanttTask};
use super::super::super::theme::get_theme;
use super::super::super::x_axis::{days_from_civil, parse_time};
use super::super::MermaidChart;
use super::super::preprocess::SourceLine;
use super::super::style::mix_color;
use super::apply_base_options;
use std::collections::{HashMap, HashSet};

const DEFAULT_DAY_SECONDS: f64 = 86_400.0;

#[derive(Clone, Debug, Default)]
struct GanttCalendar {
    excluded_weekdays: [bool; 7],
    included_weekdays: [bool; 7],
    excluded_dates: HashSet<i64>,
    included_dates: HashSet<i64>,
    week_start: u8,
}

impl GanttCalendar {
    fn has_excludes(&self) -> bool {
        self.excluded_weekdays.iter().any(|&b| b) || !self.excluded_dates.is_empty()
    }

    fn is_excluded(&self, day: i64) -> bool {
        if self.included_dates.contains(&day) {
            return false;
        }
        if self.excluded_dates.contains(&day) {
            return true;
        }
        // 1970-01-01 (day 0) was Thursday (4); 0 = Sunday .. 6 = Saturday.
        let wd = (day + 4).rem_euclid(7) as usize;
        if self.included_weekdays[wd] {
            return false;
        }
        self.excluded_weekdays[wd]
    }

    fn snap_start(&self, ts: f64) -> f64 {
        if !self.has_excludes() {
            return ts;
        }
        let mut day = (ts / DEFAULT_DAY_SECONDS).floor() as i64;
        if !self.is_excluded(day) {
            return ts;
        }
        for _ in 0..3660 {
            day += 1;
            if !self.is_excluded(day) {
                return day as f64 * DEFAULT_DAY_SECONDS;
            }
        }
        ts
    }

    fn advance_duration(&self, start: f64, dur: f64) -> f64 {
        if !self.has_excludes() || dur <= 0.0 {
            return start + dur;
        }
        let whole_days = (dur / DEFAULT_DAY_SECONDS).round();
        if whole_days >= 1.0 && (dur - whole_days * DEFAULT_DAY_SECONDS).abs() < 1e-3 {
            let mut remaining = whole_days as i64;
            let mut cur_day = (start / DEFAULT_DAY_SECONDS).floor() as i64;
            let time_of_day = start - cur_day as f64 * DEFAULT_DAY_SECONDS;
            let mut guard = 0;
            while remaining > 0 && guard < 36_600 {
                if !self.is_excluded(cur_day) {
                    remaining -= 1;
                }
                cur_day += 1;
                guard += 1;
            }
            return cur_day as f64 * DEFAULT_DAY_SECONDS + time_of_day;
        }
        let end = start + dur;
        self.snap_start(end)
    }

    fn excluded_ranges(&self, min_ts: f64, max_ts: f64) -> Vec<(f64, f64)> {
        if !self.has_excludes() || !min_ts.is_finite() || !max_ts.is_finite() || max_ts <= min_ts {
            return Vec::new();
        }
        let first_day = (min_ts / DEFAULT_DAY_SECONDS).floor() as i64;
        let last_day = (max_ts / DEFAULT_DAY_SECONDS).ceil() as i64;
        if last_day - first_day > 3660 {
            return Vec::new();
        }
        let mut ranges = Vec::new();
        let mut run_start: Option<i64> = None;
        for day in first_day..=last_day {
            if self.is_excluded(day) {
                if run_start.is_none() {
                    run_start = Some(day);
                }
            } else if let Some(start_day) = run_start.take() {
                ranges.push((
                    start_day as f64 * DEFAULT_DAY_SECONDS,
                    day as f64 * DEFAULT_DAY_SECONDS,
                ));
            }
        }
        if let Some(start_day) = run_start {
            ranges.push((
                start_day as f64 * DEFAULT_DAY_SECONDS,
                (last_day + 1) as f64 * DEFAULT_DAY_SECONDS,
            ));
        }
        ranges
    }
}

fn parse_weekday(token: &str) -> Option<usize> {
    match token.trim().to_ascii_lowercase().as_str() {
        "sunday" | "sun" => Some(0),
        "monday" | "mon" => Some(1),
        "tuesday" | "tue" => Some(2),
        "wednesday" | "wed" => Some(3),
        "thursday" | "thu" => Some(4),
        "friday" | "fri" => Some(5),
        "saturday" | "sat" => Some(6),
        _ => None,
    }
}

fn parse_tick_interval(spec: &str) -> Option<f64> {
    let s = spec.trim();
    if s.is_empty() {
        return None;
    }
    let split = s
        .find(|c: char| !c.is_ascii_digit() && c != '.')
        .unwrap_or(s.len());
    let num_str = s[..split].trim();
    let unit = s[split..].trim().to_ascii_lowercase();
    let count: f64 = if num_str.is_empty() {
        1.0
    } else {
        num_str.parse().ok()?
    };
    if !count.is_finite() || count <= 0.0 {
        return None;
    }
    let unit_secs = match unit.as_str() {
        "s" | "sec" | "second" | "seconds" => 1.0,
        "m" | "min" | "minute" | "minutes" => 60.0,
        "h" | "hr" | "hour" | "hours" => 3600.0,
        "d" | "day" | "days" => DEFAULT_DAY_SECONDS,
        "w" | "wk" | "week" | "weeks" => DEFAULT_DAY_SECONDS * 7.0,
        "month" | "months" => DEFAULT_DAY_SECONDS * 30.0,
        _ => return None,
    };
    Some(count * unit_secs)
}

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
    is_vert: bool,
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
    let mut is_vert = false;
    let mut idx = 0;

    while idx < parts.len() {
        match parts[idx].to_ascii_lowercase().as_str() {
            "done" => is_done = true,
            "active" => is_active = true,
            "crit" => is_crit = true,
            "milestone" => is_milestone = true,
            "vert" => is_vert = true,
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
                let default_dur = if is_milestone || is_vert {
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
                let default_dur = if is_milestone || is_vert {
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
        is_vert,
        start,
        end,
    })
}

fn resolve_task(
    idx: usize,
    parsed: &[ParsedTask],
    id_map: &HashMap<String, usize>,
    calendar: &GanttCalendar,
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

    let raw_start = match &task.start {
        StartSpec::Timestamp(ts) => *ts,
        StartSpec::Sequential => {
            if idx == 0 {
                return Err(Error::Mermaid {
                    line: Some(task.line_no),
                    message: format!("task `{}` has no start date or predecessor", task.name),
                });
            }
            resolve_task(idx - 1, parsed, id_map, calendar, state, resolved)?.1
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
                let (_, dep_end) =
                    resolve_task(dep_idx, parsed, id_map, calendar, state, resolved)?;
                max_end = max_end.max(dep_end);
            }
            max_end
        }
    };
    let start = calendar.snap_start(raw_start);

    let end = if task.is_milestone || task.is_vert {
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
            EndSpec::Duration(dur) => calendar.advance_duration(start, dur),
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
    let mut tick_interval: Option<f64> = None;
    let mut calendar = GanttCalendar::default();
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
        if lower.starts_with("acctitle") || lower.starts_with("click ") {
            continue;
        }
        if lower.starts_with("accdescr") {
            if text.contains('{') && !text.contains('}') {
                in_acc_descr_block = true;
            }
            continue;
        }
        if lower.starts_with("excludes")
            && text.len() > 8
            && text[8..].starts_with(char::is_whitespace)
        {
            for tok in text[8..].split(',').flat_map(|s| s.split_whitespace()) {
                let t = tok.trim();
                if t.is_empty() {
                    continue;
                }
                if t.eq_ignore_ascii_case("weekends") {
                    calendar.excluded_weekdays[0] = true;
                    calendar.excluded_weekdays[6] = true;
                } else if let Some(wd) = parse_weekday(t) {
                    calendar.excluded_weekdays[wd] = true;
                } else if let Some(ts) = parse_date_with_format(t, &date_format) {
                    calendar
                        .excluded_dates
                        .insert((ts / DEFAULT_DAY_SECONDS).floor() as i64);
                }
            }
            continue;
        }
        if lower.starts_with("includes")
            && text.len() > 8
            && text[8..].starts_with(char::is_whitespace)
        {
            for tok in text[8..].split(',').flat_map(|s| s.split_whitespace()) {
                let t = tok.trim();
                if t.is_empty() {
                    continue;
                }
                if let Some(wd) = parse_weekday(t) {
                    calendar.included_weekdays[wd] = true;
                } else if let Some(ts) = parse_date_with_format(t, &date_format) {
                    calendar
                        .included_dates
                        .insert((ts / DEFAULT_DAY_SECONDS).floor() as i64);
                }
            }
            continue;
        }
        if lower.starts_with("weekday")
            && text.len() > 7
            && text[7..].starts_with(char::is_whitespace)
        {
            if let Some(wd) = parse_weekday(&text[7..]) {
                calendar.week_start = wd as u8;
            }
            continue;
        }
        if lower.starts_with("tickinterval")
            && text.len() > 12
            && text[12..].starts_with(char::is_whitespace)
        {
            tick_interval = parse_tick_interval(&text[12..]);
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
        resolve_task(
            i,
            &parsed_tasks,
            &id_map,
            &calendar,
            &mut state,
            &mut resolved,
        )?;
    }

    let (min_ts, max_ts) = resolved
        .iter()
        .flatten()
        .fold((f64::INFINITY, f64::NEG_INFINITY), |(lo, hi), &(s, e)| {
            (lo.min(s), hi.max(e))
        });

    let theme = get_theme(theme_name);
    let bg = if theme.background_color.is_transparent() {
        Color::from((255, 255, 255))
    } else {
        theme.background_color
    };
    let done_fill = mix_color(theme.y_axis.font.color, bg, 0.58);
    let crit_fill: Color = (220, 68, 68).into();
    let crit_done_fill = mix_color(crit_fill, done_fill, 0.65);
    let crit_active_fill = mix_color(crit_fill, Color::from((255, 255, 255)), 0.18);

    // Match GanttChart's first-seen category order so `active` highlights derive
    // from the task's section color.
    let mut sections: Vec<String> = Vec::new();
    for t in &parsed_tasks {
        if let Some(ref sec) = t.section
            && !sections.contains(sec)
        {
            sections.push(sec.clone());
        }
    }

    let mut tasks: Vec<GanttTask> = Vec::new();
    let mut vertical_markers: Vec<GanttMarker> = Vec::new();
    for (t, span) in parsed_tasks.into_iter().zip(resolved) {
        let (start, end) = span.unwrap_or((0.0, 0.0));
        if t.is_vert {
            vertical_markers.push(GanttMarker {
                x: start,
                label: t.name,
                color: t.is_crit.then_some(crit_fill),
            });
            continue;
        }
        let color = match (t.is_crit, t.is_done, t.is_active) {
            (true, true, _) => Some(crit_done_fill),
            (true, false, true) => Some(crit_active_fill),
            (true, false, false) => Some(crit_fill),
            (false, true, _) => Some(done_fill),
            (false, false, true) => {
                let cat_idx = t
                    .section
                    .as_deref()
                    .and_then(|s| sections.iter().position(|c| c == s))
                    .unwrap_or(sections.len());
                let base = get_color(&theme.series.colors, cat_idx);
                Some(mix_color(base, Color::from((255, 255, 255)), 0.28))
            }
            (false, false, false) => None,
        };
        tasks.push(GanttTask {
            name: t.name,
            start,
            end,
            row: None,
            category: t.section,
            progress: None,
            color,
        });
    }

    let default_height = (tasks.len().max(1) as f32 * 36.0 + 130.0).clamp(240.0, 1200.0);
    let mut gantt = GanttChart::new_with_theme(tasks, theme_name);
    gantt.vertical_markers = vertical_markers;
    if let Some(fmt) = axis_format {
        gantt.x_axis.formatter = Some(fmt);
    }
    if let Some(interval) = tick_interval {
        gantt.tick_interval = Some(interval);
    }
    if calendar.has_excludes() {
        gantt.excluded_ranges = calendar.excluded_ranges(min_ts, max_ts);
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn calendar_excludes_weekends_and_includes_override() {
        let mut cal = GanttCalendar::default();
        cal.excluded_weekdays[0] = true; // Sun
        cal.excluded_weekdays[6] = true; // Sat

        // 2026-01-01 (Thu) + 30 workdays -> 30th workday is 2026-02-11 (Wed),
        // so exclusive end = 2026-02-12 00:00.
        let jan1 = parse_date_with_format("2026-01-01", "YYYY-MM-DD").unwrap();
        let feb12 = parse_date_with_format("2026-02-12", "YYYY-MM-DD").unwrap();
        let mar12 = parse_date_with_format("2026-03-12", "YYYY-MM-DD").unwrap();

        let end1 = cal.advance_duration(jan1, 30.0 * DEFAULT_DAY_SECONDS);
        assert_eq!(end1, feb12);

        // `after a1` starts 2026-02-12 (Thu) + 20 workdays -> ends 2026-03-12 00:00.
        let start2 = cal.snap_start(end1);
        assert_eq!(start2, feb12);
        let end2 = cal.advance_duration(start2, 20.0 * DEFAULT_DAY_SECONDS);
        assert_eq!(end2, mar12);

        // Starting on Saturday 2026-01-03 snaps to Monday 2026-01-05.
        let sat = parse_date_with_format("2026-01-03", "YYYY-MM-DD").unwrap();
        let mon = parse_date_with_format("2026-01-05", "YYYY-MM-DD").unwrap();
        assert_eq!(cal.snap_start(sat), mon);

        // `includes` overrides `excludes`.
        cal.included_dates
            .insert((sat / DEFAULT_DAY_SECONDS).floor() as i64);
        assert_eq!(cal.snap_start(sat), sat);

        // Tick interval parsing.
        assert_eq!(
            parse_tick_interval("1week"),
            Some(7.0 * DEFAULT_DAY_SECONDS)
        );
        assert_eq!(parse_tick_interval("2day"), Some(2.0 * DEFAULT_DAY_SECONDS));
        assert_eq!(
            parse_tick_interval("1month"),
            Some(30.0 * DEFAULT_DAY_SECONDS)
        );
    }
}
