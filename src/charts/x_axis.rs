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

//! Continuous x axes: numbers and time.
//!
//! A chart's x axis is a category axis by default — point `i` of every
//! series sits in slot `i`, evenly spaced. As soon as x values are given
//! (`x_axis_values` for the chart, or `x_values` on a series) the axis
//! becomes continuous: every point is placed at its x value, so unevenly
//! sampled data keeps its real spacing. `x_axis_type` says whether the
//! values are plain numbers or timestamps (unix seconds), which decides how
//! the ticks are chosen and labelled.

use std::borrow::Cow;

use super::base::ChartBase;
use super::canvas::Canvas;
use super::common::{Align, AxisType, Series};
use super::component::Axis;
use super::measure_text_width_family;
use super::util::format_string;

const MINUTE: f64 = 60.0;
const HOUR: f64 = 3600.0;
const DAY: f64 = 86400.0;

/// A linear mapping of x values onto the width of the plot.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct ContinuousX {
    /// Value at the left edge of the plot.
    pub min: f64,
    /// Value at the right edge of the plot.
    pub max: f64,
    /// Width of the plot in pixels.
    pub width: f32,
    /// Width in pixels a bar group may take around its x: the smallest gap
    /// between two x values.
    pub band_width: f32,
    /// The values are timestamps.
    pub time: bool,
    /// Ticks only at `min` plus multiples of this step (the bin edges of a
    /// histogram) instead of at any round value.
    pub tick_step: Option<f64>,
}

impl ContinuousX {
    /// Pixel offset of `x` from the left edge of the plot.
    pub fn px(&self, x: f64) -> f32 {
        ((x - self.min) / (self.max - self.min) * self.width as f64) as f32
    }
}

impl ChartBase {
    /// True when x values were given, which makes the x axis continuous.
    pub(crate) fn has_x_values(&self) -> bool {
        !self.x_axis_values.is_empty()
            || self
                .series_list
                .iter()
                .any(|s| s.x_values.as_ref().is_some_and(|x| !x.is_empty()))
    }
    /// The slot of data point `i` of `series`: its index into the series'
    /// own `x_values` when it has them, else into the chart's categories or
    /// `x_axis_values` (which `start_index` shifts).
    pub(crate) fn x_slot(&self, series: &Series, i: usize) -> usize {
        if series.x_values.is_some() {
            i
        } else {
            i.saturating_add(series.start_index)
        }
    }
    /// The x value of a series' slot on a continuous axis.
    pub(crate) fn x_value(&self, series: &Series, slot: usize) -> Option<f64> {
        let values = series.x_values.as_ref().unwrap_or(&self.x_axis_values);
        values.get(slot).copied().filter(|v| v.is_finite())
    }
    /// How many slots the series span: the categories, or on a continuous
    /// axis the longest list of x values.
    pub(crate) fn x_count(&self) -> usize {
        if !self.has_x_values() {
            return self.x_axis_data.len();
        }
        self.series_list
            .iter()
            .filter_map(|s| s.x_values.as_ref().map(|x| x.len().min(s.slot_len())))
            .chain(std::iter::once(self.x_axis_values.len()))
            .max()
            .unwrap_or(0)
    }
    /// What a data point's x is called in labels and `data-*` attributes:
    /// its category, or its formatted x value on a continuous axis.
    pub(crate) fn x_label(&self, series: &Series, slot: usize) -> Option<Cow<'_, str>> {
        if series.x_values.is_none() && self.x_axis_values.is_empty() {
            return self
                .x_axis_data
                .get(slot)
                .map(|s| Cow::Borrowed(s.as_str()));
        }
        let x = self.x_value(series, slot)?;
        Some(Cow::Owned(if self.x_axis_type == AxisType::Time {
            format_time_full(x + self.time_offset())
        } else {
            format_number(x, 6)
        }))
    }
    fn time_offset(&self) -> f64 {
        self.x_axis_time_offset as f64 * MINUTE
    }
    /// The scale of the continuous x axis over a plot `width` pixels wide,
    /// or `None` on a category axis. With `bands` the range is widened by
    /// half a band on both sides so bars at the first and last x fit.
    pub(crate) fn continuous_x(&self, width: f32, bands: bool) -> Option<ContinuousX> {
        let mut xs: Vec<f64> = self
            .series_list
            .iter()
            .filter_map(|s| s.x_values.as_ref())
            .flatten()
            .chain(self.x_axis_values.iter())
            .copied()
            .filter(|v| v.is_finite())
            .collect();
        if xs.is_empty() {
            return None;
        }
        xs.sort_by(f64::total_cmp);
        xs.dedup();
        let (mut min, mut max) = (xs[0], xs[xs.len() - 1]);
        // The smallest gap between two x values is the room one bar group
        // has; a lone x gets a band as wide as the default range below.
        let gap = xs
            .windows(2)
            .map(|w| w[1] - w[0])
            .fold(f64::INFINITY, f64::min);
        let gap = if gap.is_finite() { gap } else { 1.0 };
        if bands {
            // Half a band at both ends; a lone x sits in a band half the
            // plot wide rather than filling it.
            let pad = if xs.len() == 1 { gap } else { gap / 2.0 };
            min -= pad;
            max += pad;
        } else if min == max {
            min -= 0.5;
            max += 0.5;
        }
        if let Some(value) = self.x_axis_min.filter(|v| v.is_finite()) {
            min = value;
        }
        if let Some(value) = self.x_axis_max.filter(|v| v.is_finite()) {
            max = value;
        }
        if max <= min {
            max = min + 1.0;
        }
        let band_width = (gap / (max - min) * width as f64) as f32;
        Some(ContinuousX {
            min,
            max,
            width,
            band_width,
            time: self.x_axis_type == AxisType::Time,
            tick_step: None,
        })
    }
    /// The ticks of a continuous x axis: pixel offsets and labels. As many
    /// ticks as fit side by side, at round values (or round times).
    pub(crate) fn x_ticks(&self, scale: &ContinuousX) -> (Vec<f32>, Vec<String>) {
        let formatter = self.x_axis_formatter.as_deref().unwrap_or("");
        let fits = |ticks: &[(f64, String)]| -> bool {
            let needed: f32 = ticks
                .iter()
                .map(|(_, label)| {
                    measure_text_width_family(&self.font_family, self.x_axis_font_size, label)
                        .map(|b| b.width())
                        .unwrap_or_default()
                        + 12.0
                })
                .sum();
            needed <= scale.width
        };
        let place = |ticks: Vec<(f64, String)>| -> (Vec<f32>, Vec<String>) {
            ticks
                .into_iter()
                .map(|(x, label)| (scale.px(x), label))
                .unzip()
        };
        // Ticks tied to a step: every edge when the labels fit, else every
        // 2nd, 5th, 10th… so a tick is always on an edge.
        if let Some(step) = scale.tick_step.filter(|s| s.is_finite() && *s > 0.0) {
            let mut multiple = 1.0;
            loop {
                let ticks = stepped_ticks(scale.min, scale.max, step * multiple, formatter);
                if fits(&ticks) || ticks.len() <= 2 {
                    return place(ticks);
                }
                multiple = next_multiple(multiple);
            }
        }
        let mut target = ((scale.width / 60.0) as usize).clamp(2, 10);
        loop {
            let ticks = if scale.time {
                time_ticks(scale.min, scale.max, target, self.time_offset(), formatter)
            } else {
                value_ticks(scale.min, scale.max, target, formatter)
            };
            if fits(&ticks) || target <= 2 {
                return place(ticks);
            }
            target -= 1;
        }
    }
    /// Draws the continuous x axis: the line, a tick and a label at every
    /// round value.
    pub(crate) fn render_continuous_x_axis(
        &self,
        c: Canvas,
        scale: &ContinuousX,
        axis_width: f32,
        x_axis_height: f32,
    ) {
        let (positions, labels) = self.x_ticks(scale);
        let margin = self.x_axis_margin.unwrap_or_default();
        c.child(margin).axis(Axis {
            height: x_axis_height,
            width: axis_width,
            split_number: 1,
            font_family: self.font_family.clone(),
            data: labels,
            tick_positions: Some(positions),
            font_color: Some(self.x_axis_font_color),
            font_weight: self.x_axis_font_weight.clone(),
            stroke_color: Some(self.x_axis_stroke_color),
            font_size: self.x_axis_font_size,
            name_gap: self.x_axis_name_gap,
            name_rotate: self.x_axis_name_rotate,
            name_align: Align::Center,
            ..Default::default()
        });
    }
}

// ---------------------------------------------------------------------------
// Number ticks.

/// The smallest of 1, 2, 5 × 10ⁿ that is at least `raw`.
pub(crate) fn nice_step(raw: f64) -> f64 {
    if !raw.is_finite() || raw <= 0.0 {
        return 1.0;
    }
    let magnitude = 10_f64.powf(raw.log10().floor());
    let normalized = raw / magnitude;
    let nice = if normalized <= 1.0 {
        1.0
    } else if normalized <= 2.0 {
        2.0
    } else if normalized <= 5.0 {
        5.0
    } else {
        10.0
    };
    nice * magnitude
}

/// Decimals needed to write multiples of `step` exactly (at most 6).
pub(crate) fn decimals_of(step: f64) -> usize {
    (0..=6)
        .find(|d| {
            let scaled = step * 10_f64.powi(*d);
            (scaled - scaled.round()).abs() < 1e-6 * scaled.abs().max(1.0)
        })
        .unwrap_or(6) as usize
}

/// `value` with at most `decimals` decimals, trailing zeros trimmed.
pub(crate) fn format_number(value: f64, decimals: usize) -> String {
    let mut text = format!("{value:.decimals$}");
    if text.contains('.') {
        while text.ends_with('0') {
            text.pop();
        }
        if text.ends_with('.') {
            text.pop();
        }
    }
    if text == "-0" {
        text = "0".to_string();
    }
    text
}

/// Round values between `min` and `max`, about `target` of them, with
/// labels that carry a k / M / G / T suffix once the step is that large.
fn value_ticks(min: f64, max: f64, target: usize, formatter: &str) -> Vec<(f64, String)> {
    let step = nice_step((max - min) / target.max(1) as f64);
    let label = tick_labeler(min, max, step, 0.0, formatter);
    // Multiples of the step by an integer factor, so no error accumulates.
    let first = (min / step - 1e-9).ceil() as i64;
    let last = (max / step + 1e-9).floor() as i64;
    (first..=last)
        .take(1000)
        .map(|k| {
            let value = k as f64 * step;
            (value, label(value))
        })
        .collect()
}

/// Ticks at `min` plus every multiple of `step` up to `max`.
fn stepped_ticks(min: f64, max: f64, step: f64, formatter: &str) -> Vec<(f64, String)> {
    let label = tick_labeler(min, max, step, min, formatter);
    let count = ((max - min) / step + 1e-9).floor() as i64;
    (0..=count.clamp(0, 1000))
        .map(|k| {
            let value = min + k as f64 * step;
            (value, label(value))
        })
        .collect()
}

/// The next of 1, 2, 5, 10, 20, 50, … after `multiple`.
fn next_multiple(multiple: f64) -> f64 {
    let magnitude = 10_f64.powf(multiple.log10().floor());
    match (multiple / magnitude).round() as i64 {
        1 => 2.0 * magnitude,
        2 => 5.0 * magnitude,
        _ => 10.0 * magnitude,
    }
}

/// How the ticks of a number axis are written: with the largest unit suffix
/// (k / M / G / T) the values reach, as long as the step — and the `origin`
/// the ticks are counted from — still read with at most two decimals in it
/// (1.5M, not 0.0005k).
fn tick_labeler<'a>(
    min: f64,
    max: f64,
    step: f64,
    origin: f64,
    formatter: &'a str,
) -> impl Fn(f64) -> String + 'a {
    let decimals = |divisor: f64| decimals_of(step / divisor).max(decimals_of(origin / divisor));
    let largest = min.abs().max(max.abs());
    let (divisor, unit) = [(1e12, "T"), (1e9, "G"), (1e6, "M"), (1e3, "k")]
        .into_iter()
        .find(|(divisor, _)| largest >= *divisor && decimals(*divisor) <= 2)
        .unwrap_or((1.0, ""));
    let decimals = decimals(divisor);
    move |value| {
        let mut label = format_number(value / divisor, decimals);
        // Zero needs no unit.
        if label != "0" {
            label.push_str(unit);
        }
        format_string(&label, formatter)
    }
}

// ---------------------------------------------------------------------------
// Time: civil calendar arithmetic, parsing, formatting and ticks. Times are
// seconds since the unix epoch; "local" seconds have the display offset added.

/// Days since 1970-01-01 of a proleptic Gregorian date.
fn days_from_civil(year: i64, month: i64, day: i64) -> i64 {
    let y = if month <= 2 { year - 1 } else { year };
    let era = if y >= 0 { y } else { y - 399 } / 400;
    let yoe = y - era * 400;
    let mp = if month > 2 { month - 3 } else { month + 9 };
    let doy = (153 * mp + 2) / 5 + day - 1;
    let doe = yoe * 365 + yoe / 4 - yoe / 100 + doy;
    era * 146097 + doe - 719468
}

/// `(year, month, day)` of a day count since 1970-01-01.
fn civil_from_days(days: i64) -> (i64, i64, i64) {
    let z = days + 719468;
    let era = if z >= 0 { z } else { z - 146096 } / 146097;
    let doe = z - era * 146097;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = yoe + era * 400 + i64::from(month <= 2);
    (year, month, day)
}

struct Civil {
    year: i64,
    month: i64,
    day: i64,
    hour: i64,
    minute: i64,
    second: i64,
}

fn civil(local_seconds: f64) -> Civil {
    let total = local_seconds.floor() as i64;
    let days = total.div_euclid(86400);
    let rest = total.rem_euclid(86400);
    let (year, month, day) = civil_from_days(days);
    Civil {
        year,
        month,
        day,
        hour: rest / 3600,
        minute: rest % 3600 / 60,
        second: rest % 60,
    }
}

const MONTHS: [&str; 12] = [
    "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
];

/// Formats local seconds with a `strftime`-like pattern: `%Y %y %m %d %H %M
/// %S %b` and `%%`.
pub(crate) fn format_time(local_seconds: f64, pattern: &str) -> String {
    let t = civil(local_seconds);
    let mut out = String::with_capacity(pattern.len() + 8);
    let mut chars = pattern.chars();
    while let Some(ch) = chars.next() {
        if ch != '%' {
            out.push(ch);
            continue;
        }
        match chars.next() {
            Some('Y') => out.push_str(&t.year.to_string()),
            Some('y') => out.push_str(&format!("{:02}", t.year.rem_euclid(100))),
            Some('m') => out.push_str(&format!("{:02}", t.month)),
            Some('d') => out.push_str(&format!("{:02}", t.day)),
            Some('H') => out.push_str(&format!("{:02}", t.hour)),
            Some('M') => out.push_str(&format!("{:02}", t.minute)),
            Some('S') => out.push_str(&format!("{:02}", t.second)),
            Some('b') => out.push_str(MONTHS[(t.month - 1) as usize]),
            Some('%') => out.push('%'),
            Some(other) => {
                out.push('%');
                out.push(other);
            }
            None => out.push('%'),
        }
    }
    out
}

/// The date, with the time of day only as far as it is not zero.
pub(crate) fn format_time_full(local_seconds: f64) -> String {
    let t = civil(local_seconds);
    let pattern = if t.second != 0 {
        "%Y-%m-%d %H:%M:%S"
    } else if t.hour != 0 || t.minute != 0 {
        "%Y-%m-%d %H:%M"
    } else {
        "%Y-%m-%d"
    };
    format_time(local_seconds, pattern)
}

/// Parses a date or date-time into unix seconds: `2024-01-05`,
/// `2024-01-05 08:30`, `2024-01-05T08:30:15.5Z`, `2024-01-05T08:30+08:00`
/// (`/` is accepted in the date too). Without a zone the time is UTC.
pub(crate) fn parse_time(text: &str) -> Option<f64> {
    let text = text.trim();
    let (date, rest) = match text.find(['T', ' ']) {
        Some(pos) => (&text[..pos], text[pos + 1..].trim()),
        None => (text, ""),
    };
    let mut parts = date.split(['-', '/']);
    let year: i64 = parts.next()?.parse().ok()?;
    let month: i64 = parts.next()?.parse().ok()?;
    let day: i64 = match parts.next() {
        Some(d) => d.parse().ok()?,
        None => 1,
    };
    if parts.next().is_some() || !(1..=12).contains(&month) || !(1..=31).contains(&day) {
        return None;
    }
    let mut seconds = days_from_civil(year, month, day) as f64 * DAY;
    if rest.is_empty() {
        return Some(seconds);
    }
    // Split off the zone: `Z`, or a sign after the time.
    let (time, zone_seconds) = if let Some(time) = rest.strip_suffix(['Z', 'z']) {
        (time, 0.0)
    } else if let Some(pos) = rest.rfind(['+', '-']) {
        let zone = &rest[pos + 1..];
        let (h, m) = match zone.split_once(':') {
            Some((h, m)) => (h, m),
            None if zone.len() == 4 => zone.split_at(2),
            None => (zone, "0"),
        };
        let offset = h.parse::<f64>().ok()? * HOUR + m.parse::<f64>().ok()? * MINUTE;
        let sign = if rest.as_bytes()[pos] == b'-' {
            -1.0
        } else {
            1.0
        };
        (&rest[..pos], sign * offset)
    } else {
        (rest, 0.0)
    };
    let mut fields = time.trim().split(':');
    let hour: f64 = fields.next()?.parse().ok()?;
    let minute: f64 = fields.next()?.parse().ok()?;
    let second: f64 = match fields.next() {
        Some(s) => s.parse().ok()?,
        None => 0.0,
    };
    if fields.next().is_some()
        || !(0.0..24.0).contains(&hour)
        || !(0.0..60.0).contains(&minute)
        || !(0.0..61.0).contains(&second)
    {
        return None;
    }
    seconds += hour * HOUR + minute * MINUTE + second - zone_seconds;
    Some(seconds)
}

/// The spacing of time ticks.
#[derive(Clone, Copy, Debug, PartialEq)]
enum TimeStep {
    Seconds(f64),
    Months(i64),
    Years(i64),
}

const SECOND_STEPS: [f64; 22] = [
    1.0,
    2.0,
    5.0,
    10.0,
    15.0,
    30.0,
    MINUTE,
    2.0 * MINUTE,
    5.0 * MINUTE,
    10.0 * MINUTE,
    15.0 * MINUTE,
    30.0 * MINUTE,
    HOUR,
    2.0 * HOUR,
    3.0 * HOUR,
    6.0 * HOUR,
    12.0 * HOUR,
    DAY,
    2.0 * DAY,
    7.0 * DAY,
    14.0 * DAY,
    // Sentinel: beyond two weeks the steps follow the calendar.
    f64::INFINITY,
];

/// The finest step that gives at most `target` ticks over `span` seconds.
fn time_step(span: f64, target: usize) -> TimeStep {
    let target = target.max(1) as f64;
    for step in SECOND_STEPS {
        if step.is_finite() && span / step <= target {
            return TimeStep::Seconds(step);
        }
    }
    for months in [1, 2, 3, 6] {
        if span / (months as f64 * 30.44 * DAY) <= target {
            return TimeStep::Months(months);
        }
    }
    let years = nice_step(span / (365.25 * DAY) / target).max(1.0);
    TimeStep::Years(years as i64)
}

/// Round times between `min` and `max` (unix seconds), about `target` of
/// them, labelled by how fine the step is — or with `formatter` when it is
/// a `%` pattern.
fn time_ticks(
    min: f64,
    max: f64,
    target: usize,
    offset: f64,
    formatter: &str,
) -> Vec<(f64, String)> {
    let (local_min, local_max) = (min + offset, max + offset);
    let step = time_step(max - min, target);
    let mut locals: Vec<f64> = vec![];
    match step {
        TimeStep::Seconds(step) => {
            // Weeks start on a Monday; 1970-01-05 was the first one.
            let anchor = if step >= 7.0 * DAY { 4.0 * DAY } else { 0.0 };
            let mut k = ((local_min - anchor) / step - 1e-9).ceil();
            while anchor + k * step <= local_max + 1e-6 && locals.len() < 1000 {
                locals.push(anchor + k * step);
                k += 1.0;
            }
        }
        TimeStep::Months(months) => {
            let first = civil(local_min);
            let mut index = first.year * 12 + first.month - 1;
            // Start on the step boundary at or before `local_min`; ticks
            // before the range are dropped below.
            index = index.div_euclid(months) * months;
            loop {
                let local =
                    days_from_civil(index.div_euclid(12), index.rem_euclid(12) + 1, 1) as f64 * DAY;
                if local > local_max + 1e-6 || locals.len() >= 1000 {
                    break;
                }
                if local >= local_min - 1e-6 {
                    locals.push(local);
                }
                index += months;
            }
        }
        TimeStep::Years(years) => {
            let mut year = civil(local_min).year.div_euclid(years) * years;
            loop {
                let local = days_from_civil(year, 1, 1) as f64 * DAY;
                if local > local_max + 1e-6 || locals.len() >= 1000 {
                    break;
                }
                if local >= local_min - 1e-6 {
                    locals.push(local);
                }
                year += years;
            }
        }
    }
    locals
        .into_iter()
        .map(|local| {
            let label = if formatter.contains('%') {
                format_time(local, formatter)
            } else {
                let pattern = match step {
                    TimeStep::Seconds(s) if s < MINUTE => "%H:%M:%S",
                    // A tick at midnight names the day it starts.
                    TimeStep::Seconds(s) if s < DAY => {
                        if local.rem_euclid(DAY) == 0.0 {
                            "%m-%d"
                        } else {
                            "%H:%M"
                        }
                    }
                    TimeStep::Seconds(_) => "%m-%d",
                    TimeStep::Months(_) => "%Y-%m",
                    TimeStep::Years(_) => "%Y",
                };
                format_string(&format_time(local, pattern), formatter)
            };
            (local - offset, label)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    fn labels(ticks: Vec<(f64, String)>) -> Vec<String> {
        ticks.into_iter().map(|(_, label)| label).collect()
    }

    #[test]
    fn civil_round_trip() {
        assert_eq!(0, days_from_civil(1970, 1, 1));
        assert_eq!(19_723, days_from_civil(2024, 1, 1));
        assert_eq!((2024, 2, 29), civil_from_days(days_from_civil(2024, 2, 29)));
        assert_eq!((1969, 12, 31), civil_from_days(-1));
        assert_eq!((1900, 3, 1), civil_from_days(days_from_civil(1900, 3, 1)));
        for days in (-800_000..800_000).step_by(997) {
            let (y, m, d) = civil_from_days(days);
            assert_eq!(days, days_from_civil(y, m, d));
        }
    }

    #[test]
    fn parses_dates_and_times() {
        assert_eq!(Some(1_704_067_200.0), parse_time("2024-01-01"));
        assert_eq!(Some(1_704_067_200.0), parse_time("2024/01/01"));
        assert_eq!(Some(1_704_067_200.0), parse_time("2024-01"));
        assert_eq!(Some(1_704_097_800.0), parse_time("2024-01-01 08:30"));
        assert_eq!(Some(1_704_097_815.5), parse_time("2024-01-01T08:30:15.5Z"));
        // 08:30 in UTC+8 is 00:30 UTC.
        assert_eq!(Some(1_704_069_000.0), parse_time("2024-01-01T08:30+08:00"));
        assert_eq!(
            Some(1_704_069_000.0),
            parse_time("2024-01-01T08:30:00+0800")
        );
        assert_eq!(Some(1_704_115_800.0), parse_time("2024-01-01T08:30-05:00"));
        for bad in [
            "",
            "2024",
            "2024-13-01",
            "2024-01-01 25:00",
            "tomorrow",
            "2024-01-01 8",
        ] {
            assert_eq!(None, parse_time(bad), "{bad:?}");
        }
    }

    #[test]
    fn formats_times() {
        let t = parse_time("2024-03-05 08:07:09").unwrap();
        assert_eq!(
            "2024-03-05 08:07:09 24 Mar 100%",
            format_time(t, "%Y-%m-%d %H:%M:%S %y %b 100%%")
        );
        assert_eq!("2024-03-05 08:07:09", format_time_full(t));
        assert_eq!("2024-03-05 08:07", format_time_full(t - 9.0));
        assert_eq!(
            "2024-03-05",
            format_time_full(parse_time("2024-03-05").unwrap())
        );
        // Before the epoch.
        assert_eq!("1969-12-31 23:59", format_time_full(-60.0));
    }

    #[test]
    fn number_ticks() {
        assert_eq!(
            vec!["0", "20", "40", "60", "80", "100"],
            labels(value_ticks(0.0, 100.0, 5, ""))
        );
        assert_eq!(
            vec!["0.5", "1", "1.5", "2"],
            labels(value_ticks(0.3, 2.2, 4, ""))
        );
        assert_eq!(vec!["-5", "0", "5"], labels(value_ticks(-7.0, 9.0, 4, "")));
        assert_eq!(
            vec!["2k", "4k", "6k"],
            labels(value_ticks(1500.0, 7000.0, 3, ""))
        );
        assert_eq!(
            vec!["1.5M", "2M", "2.5M"],
            labels(value_ticks(1_400_000.0, 2_600_000.0, 3, ""))
        );
        assert_eq!(
            vec!["0 km", "50 km"],
            labels(value_ticks(0.0, 60.0, 2, "{c} km"))
        );
        assert_eq!(
            vec!["0.02", "0.04"],
            labels(value_ticks(0.012, 0.05, 2, ""))
        );
        // The unit follows the values only while the step stays readable.
        assert_eq!(
            vec!["999.5", "1000", "1000.5"],
            labels(value_ticks(999.4, 1000.6, 3, ""))
        );
    }

    #[test]
    fn ticks_on_a_step() {
        assert_eq!(
            vec!["150", "155", "160"],
            labels(stepped_ticks(150.0, 160.0, 5.0, ""))
        );
        // Counted from the start of the range, not from zero.
        assert_eq!(
            vec!["1.5", "4", "6.5"],
            labels(stepped_ticks(1.5, 7.0, 2.5, ""))
        );
        assert_eq!(
            vec!["0", "2.5k", "5k"],
            labels(stepped_ticks(0.0, 5000.0, 2500.0, ""))
        );
        let multiples: Vec<f64> = std::iter::successors(Some(1.0), |m| Some(next_multiple(*m)))
            .take(7)
            .collect();
        assert_eq!(vec![1.0, 2.0, 5.0, 10.0, 20.0, 50.0, 100.0], multiples);
    }

    #[test]
    fn time_tick_steps() {
        let day = |text: &str| parse_time(text).unwrap();
        // A few hours: hour ticks, the midnight one names the day.
        assert_eq!(
            vec!["21:00", "01-02", "03:00", "06:00"],
            labels(time_ticks(
                day("2024-01-01 20:10"),
                day("2024-01-02 06:30"),
                4,
                0.0,
                ""
            ))
        );
        // Days.
        assert_eq!(
            vec!["01-30", "02-01", "02-03"],
            labels(time_ticks(
                day("2024-01-28 12:00"),
                day("2024-02-04"),
                4,
                0.0,
                ""
            ))
        );
        // Weeks start on Mondays.
        assert_eq!(
            vec!["01-01", "01-08", "01-15", "01-22"],
            labels(time_ticks(day("2023-12-30"), day("2024-01-25"), 4, 0.0, ""))
        );
        // Months and quarters fall on real month starts.
        assert_eq!(
            vec!["2024-01", "2024-04", "2024-07", "2024-10"],
            labels(time_ticks(day("2023-12-15"), day("2024-11-20"), 4, 0.0, ""))
        );
        let ticks = time_ticks(day("2023-12-15"), day("2024-11-20"), 4, 0.0, "");
        assert_eq!(day("2024-04-01"), ticks[1].0);
        // Years.
        assert_eq!(
            vec!["2015", "2020", "2025"],
            labels(time_ticks(day("2012-06-01"), day("2026-01-01"), 3, 0.0, ""))
        );
        // A custom pattern, and a display offset (UTC+8).
        assert_eq!(
            vec!["Jan 02", "Jan 03"],
            labels(time_ticks(
                day("2024-01-01 12:00"),
                day("2024-01-03"),
                2,
                0.0,
                "%b %d"
            ))
        );
        let ticks = time_ticks(
            day("2024-01-01 15:00"),
            day("2024-01-01 20:00"),
            3,
            8.0 * HOUR,
            "",
        );
        assert_eq!(vec!["01-02", "02:00", "04:00"], labels(ticks.clone()));
        assert_eq!(day("2024-01-01 16:00"), ticks[0].0);
    }
}
