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

use super::canvas;
use super::color::*;
use super::common::*;
use super::component::*;
use super::params::*;
use super::theme::{get_default_theme_name, get_theme, Theme, DEFAULT_Y_AXIS_WIDTH};
use super::util::*;
use super::Canvas;
use crate::charts::measure_text_width_family;
use charts_rs_derive::Chart;
use std::sync::Arc;

// ── Simple date helpers (no external crate) ──────────────────────────────────

fn is_leap_year(year: i32) -> bool {
    (year % 4 == 0 && year % 100 != 0) || year % 400 == 0
}

fn days_in_month(year: i32, month: u32) -> u32 {
    match month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 => {
            if is_leap_year(year) {
                29
            } else {
                28
            }
        }
        _ => 0,
    }
}

/// Returns day-of-week: 0 = Sunday, 1 = Monday, …, 6 = Saturday.
/// Uses Tomohiko Sakamoto's algorithm.
fn day_of_week(year: i32, month: u32, day: u32) -> u32 {
    let t: [i32; 12] = [0, 3, 2, 5, 0, 3, 5, 1, 4, 6, 2, 4];
    let y = if month < 3 { year - 1 } else { year };
    ((y + y / 4 - y / 100 + y / 400 + t[(month - 1) as usize] + day as i32).rem_euclid(7)) as u32
}

/// Parses "YYYY-MM-DD" → (year, month, day).  Returns None on malformed input.
fn parse_date(s: &str) -> Option<(i32, u32, u32)> {
    let mut parts = s.splitn(3, '-');
    let year: i32 = parts.next()?.parse().ok()?;
    let month: u32 = parts.next()?.parse().ok()?;
    let day: u32 = parts.next()?.parse().ok()?;
    if !(1..=12).contains(&month) || !(1..=31).contains(&day) {
        return None;
    }
    Some((year, month, day))
}

/// Julian Day Number – used only to compute day differences.
fn jdn(year: i32, month: u32, day: u32) -> i64 {
    let a = (14_i64 - month as i64) / 12;
    let y = year as i64 + 4800 - a;
    let m = month as i64 + 12 * a - 3;
    day as i64 + (153 * m + 2) / 5 + 365 * y + y / 4 - y / 100 + y / 400 - 32045
}

/// Number of days from `(y1,m1,d1)` to `(y2,m2,d2)` (negative if before).
fn days_diff(y1: i32, m1: u32, d1: u32, y2: i32, m2: u32, d2: u32) -> i64 {
    jdn(y2, m2, d2) - jdn(y1, m1, d1)
}

/// Advance a date by `n` days (n ≥ 0).
fn add_days(mut year: i32, mut month: u32, mut day: u32, mut n: u32) -> (i32, u32, u32) {
    while n > 0 {
        let dim = days_in_month(year, month);
        let remaining = dim - day;
        if n <= remaining {
            day += n;
            n = 0;
        } else {
            n -= remaining + 1;
            day = 1;
            month += 1;
            if month > 12 {
                month = 1;
                year += 1;
            }
        }
    }
    (year, month, day)
}

static MONTH_ABBR: [&str; 12] = [
    "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
];
static DOW_ABBR: [&str; 7] = ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"];

// ── CalendarChart ─────────────────────────────────────────────────────────────

#[derive(Clone, Debug, Default, Chart)]
pub struct CalendarChart {
    pub width: f32,
    pub height: f32,
    pub x: f32,
    pub y: f32,
    pub margin: Box,
    // dummy – required by #[derive(Chart)]
    series_list: Vec<Series>,
    pub font_family: String,
    pub background_color: Color,
    pub is_light: bool,

    // title
    pub title_text: String,
    pub title_font_size: f32,
    pub title_font_color: Color,
    pub title_font_weight: Option<String>,
    pub title_margin: Option<Box>,
    pub title_align: Align,
    pub title_height: f32,

    // sub title
    pub sub_title_text: String,
    pub sub_title_font_size: f32,
    pub sub_title_font_color: Color,
    pub sub_title_font_weight: Option<String>,
    pub sub_title_margin: Option<Box>,
    pub sub_title_align: Align,
    pub sub_title_height: f32,

    // legend (required by derive, not rendered)
    pub legend_font_size: f32,
    pub legend_font_color: Color,
    pub legend_font_weight: Option<String>,
    pub legend_align: Align,
    pub legend_margin: Option<Box>,
    pub legend_category: LegendCategory,
    pub legend_show: Option<bool>,

    // x/y axis fields required by derive (not used in rendering)
    pub x_axis_data: Vec<String>,
    pub x_axis_height: f32,
    pub x_axis_stroke_color: Color,
    pub x_axis_font_size: f32,
    pub x_axis_font_color: Color,
    pub x_axis_font_weight: Option<String>,
    pub x_axis_name_gap: f32,
    pub x_axis_name_rotate: f32,
    pub x_axis_margin: Option<Box>,
    pub x_axis_hidden: bool,
    pub x_boundary_gap: Option<bool>,

    pub y_axis_hidden: bool,
    y_axis_configs: Vec<YAxisConfig>,

    grid_stroke_color: Color,
    grid_stroke_width: f32,

    // series fields required by derive
    pub series_stroke_width: f32,
    pub series_label_font_color: Color,
    pub series_label_font_size: f32,
    pub series_label_font_weight: Option<String>,
    pub series_label_formatter: String,
    pub series_colors: Vec<Color>,
    pub series_symbol: Option<Symbol>,
    pub series_smooth: bool,
    pub series_fill: bool,

    // ── Calendar-specific fields ──────────────────────────────────────────────
    /// Data points: each entry is `("YYYY-MM-DD", value)`.
    pub data: Vec<(String, f32)>,

    /// First day shown, inclusive.  Format: `"YYYY-MM-DD"`.
    pub start_date: String,

    /// Last day shown, inclusive.  Format: `"YYYY-MM-DD"`.
    pub end_date: String,

    /// Value that maps to `min_color`.  Auto-computed from data when 0.
    pub min: f32,

    /// Value that maps to `max_color`.  Auto-computed from data when 0.
    pub max: f32,

    /// Color for cells with the minimum value (default: light gray).
    pub min_color: Color,

    /// Color for cells with the maximum value (default: green).
    pub max_color: Color,

    /// Color for cells that have no data entry.
    pub empty_color: Color,

    /// Side length of each day square in pixels (default: 13).
    pub cell_size: f32,

    /// Gap between adjacent squares in pixels (default: 3).
    pub cell_gap: f32,

    /// Height of the month-label row at the top of the grid (default: 20).
    pub month_label_height: f32,

    /// Width of the day-of-week label column at the left of the grid (default: 30).
    pub week_label_width: f32,

    /// Rows of the day-of-week labels to display.
    /// Each entry is a day-of-week index (0 = Sun … 6 = Sat).
    /// Defaults to `[1, 3, 5]` (Mon, Wed, Fri), matching the GitHub style.
    pub show_dow_labels: Vec<usize>,
}

impl CalendarChart {
    fn fill_default(&mut self) {
        if self.cell_size <= 0.0 {
            self.cell_size = 13.0;
        }
        if self.cell_gap <= 0.0 {
            self.cell_gap = 3.0;
        }
        if self.month_label_height <= 0.0 {
            self.month_label_height = 20.0;
        }
        if self.week_label_width <= 0.0 {
            self.week_label_width = 30.0;
        }
        if self.show_dow_labels.is_empty() {
            self.show_dow_labels = vec![1, 3, 5]; // Mon, Wed, Fri
        }
        // default colors – GitHub contribution-graph palette
        if self.min_color.is_zero() {
            self.min_color = (235, 237, 240).into();
        }
        if self.max_color.is_zero() {
            self.max_color = (33, 110, 57).into();
        }
        if self.empty_color.is_zero() {
            let mut c: Color = if self.is_light {
                (235, 237, 240).into()
            } else {
                (40, 40, 45).into()
            };
            c = c.with_alpha(180);
            self.empty_color = c;
        }
        // auto min / max
        if self.max == 0.0 {
            for (_, v) in &self.data {
                if *v > self.max {
                    self.max = *v;
                }
            }
        }
        // default date range: current year (fall back to a hardcoded year if we
        // cannot determine it – 2024 is a safe arbitrary default)
        if self.start_date.is_empty() {
            self.start_date = "2024-01-01".to_string();
        }
        if self.end_date.is_empty() {
            self.end_date = "2024-12-31".to_string();
        }
    }

    /// Interpolate between min_color and max_color.
    fn cell_color(&self, value: f32) -> Color {
        let value = value.clamp(self.min, self.max);
        let range = self.max - self.min;
        if range <= 0.0 {
            return self.max_color;
        }
        let t = (value - self.min) / range;
        let lerp = |a: u8, b: u8| -> u8 {
            let diff = (b as f32 - a as f32) * t;
            (a as f32 + diff).round() as u8
        };
        Color {
            r: lerp(self.min_color.r, self.max_color.r),
            g: lerp(self.min_color.g, self.max_color.g),
            b: lerp(self.min_color.b, self.max_color.b),
            a: lerp(self.min_color.a, self.max_color.a),
        }
    }

    /// Creates a calendar chart for `year` (Jan 1 – Dec 31) with default theme.
    pub fn new(data: Vec<(String, f32)>, year: i32) -> CalendarChart {
        CalendarChart::new_with_theme(data, year, &get_default_theme_name())
    }

    /// Creates a calendar chart for `year` with a custom theme.
    pub fn new_with_theme(data: Vec<(String, f32)>, year: i32, theme: &str) -> CalendarChart {
        let mut c = CalendarChart {
            data,
            start_date: format!("{year:04}-01-01"),
            end_date: format!("{year:04}-12-31"),
            ..Default::default()
        };
        let t = get_theme(theme);
        c.fill_theme(t);
        c.fill_default();
        // Auto-size to fit the calendar
        c.width = c.auto_width();
        c.height = c.auto_height();
        c
    }

    /// Creates a calendar chart from a JSON string.
    pub fn from_json(json: &str) -> canvas::Result<CalendarChart> {
        let mut c = CalendarChart {
            ..Default::default()
        };
        let value = c.fill_option(json)?;
        if let Some(start) = get_string_from_value(&value, "start_date") {
            c.start_date = start;
        }
        if let Some(end) = get_string_from_value(&value, "end_date") {
            c.end_date = end;
        }
        if let Some(min) = get_f32_from_value(&value, "min") {
            c.min = min;
        }
        if let Some(max) = get_f32_from_value(&value, "max") {
            c.max = max;
        }
        if let Some(col) = get_color_from_value(&value, "min_color") {
            c.min_color = col;
        }
        if let Some(col) = get_color_from_value(&value, "max_color") {
            c.max_color = col;
        }
        if let Some(col) = get_color_from_value(&value, "empty_color") {
            c.empty_color = col;
        }
        if let Some(v) = get_f32_from_value(&value, "cell_size") {
            c.cell_size = v;
        }
        if let Some(v) = get_f32_from_value(&value, "cell_gap") {
            c.cell_gap = v;
        }
        // parse data: [[date_str, value], ...]
        if let Some(arr) = value.get("data").and_then(|v| v.as_array()) {
            let mut items = vec![];
            for item in arr {
                if let Some(pair) = item.as_array() {
                    if pair.len() == 2 {
                        if let (Some(date), Some(val)) = (pair[0].as_str(), pair[1].as_f64()) {
                            items.push((date.to_string(), val as f32));
                        }
                    }
                }
            }
            c.data = items;
        }
        c.fill_default();
        // Auto-size when the user did not explicitly set width/height
        if c.width <= 0.0 {
            c.width = c.auto_width();
        }
        if c.height <= 0.0 {
            c.height = c.auto_height();
        }
        Ok(c)
    }

    /// Computes the number of week-columns needed for the current date range.
    fn num_weeks(&self) -> usize {
        let (sy, sm, sd) = match parse_date(&self.start_date) {
            Some(d) => d,
            None => return 53,
        };
        let (ey, em, ed) = match parse_date(&self.end_date) {
            Some(d) => d,
            None => return 53,
        };
        let total_days = days_diff(sy, sm, sd, ey, em, ed) + 1;
        if total_days <= 0 {
            return 1;
        }
        let start_dow = day_of_week(sy, sm, sd) as i64; // 0=Sun
        ((start_dow + total_days + 6) / 7) as usize
    }

    fn auto_width(&self) -> f32 {
        let step = self.cell_size + self.cell_gap;
        self.margin.left
            + self.margin.right
            + self.week_label_width
            + self.num_weeks() as f32 * step
    }

    fn auto_height(&self) -> f32 {
        let step = self.cell_size + self.cell_gap;
        let title_h = if !self.title_text.is_empty() {
            self.title_height
                + if !self.sub_title_text.is_empty() {
                    self.sub_title_height
                } else {
                    0.0
                }
        } else {
            0.0
        };
        self.margin.top + self.margin.bottom + title_h + self.month_label_height + 7.0 * step
    }

    /// Renders the calendar heatmap to an SVG string.
    pub fn svg(&self) -> canvas::Result<String> {
        let (sy, sm, sd) = parse_date(&self.start_date).ok_or_else(|| canvas::Error::Params {
            message: format!("invalid start_date: {}", self.start_date),
        })?;
        let (ey, em, ed) = parse_date(&self.end_date).ok_or_else(|| canvas::Error::Params {
            message: format!("invalid end_date: {}", self.end_date),
        })?;

        let total_days = days_diff(sy, sm, sd, ey, em, ed) + 1;
        if total_days <= 0 {
            return Err(canvas::Error::Params {
                message: "end_date must be >= start_date".to_string(),
            });
        }

        let start_dow = day_of_week(sy, sm, sd) as i64; // 0=Sun

        // Build lookup: date-string → value
        let mut lookup = std::collections::HashMap::new();
        for (date_str, val) in &self.data {
            lookup.insert(date_str.as_str(), *val);
        }

        let mut c = Canvas::new_width_xy(self.width, self.height, self.x, self.y);
        self.render_background(c.child(Box::default()));
        c.margin = self.margin.clone();

        let top_offset = self.render_title(c.child(Box::default()));

        // Canvas region below the title
        let mut grid_c = c.child(Box {
            top: top_offset,
            ..Default::default()
        });

        let step = self.cell_size + self.cell_gap;
        let wlw = self.week_label_width; // left column width
        let mlh = self.month_label_height; // top row height

        // ── Day-of-week labels ────────────────────────────────────────────────
        let dow_font_size = self.x_axis_font_size.max(10.0);
        let dow_color = self.x_axis_font_color;
        for &row in &self.show_dow_labels {
            let label = DOW_ABBR[row % 7];
            let y = mlh + row as f32 * step + self.cell_size / 2.0;
            grid_c.text(Text {
                text: label.to_string(),
                font_family: Some(self.font_family.clone()),
                font_color: Some(dow_color),
                font_size: Some(dow_font_size),
                dominant_baseline: Some("central".to_string()),
                x: Some(0.0),
                y: Some(y),
                ..Default::default()
            });
        }

        // ── Month labels ──────────────────────────────────────────────────────
        // We track which week each month starts in.
        let month_font_size = self.x_axis_font_size.max(10.0);
        let month_color = self.x_axis_font_color;
        let mut cur_y = sy;
        let mut cur_m = sm;
        let mut cur_d = sd;
        let mut last_month_col: Option<u32> = None;
        for day_idx in 0..total_days {
            let col = ((start_dow + day_idx) / 7) as u32;
            // Is this the first day of a new month within the visible range?
            if cur_d == 1 && last_month_col != Some(col) {
                let label = MONTH_ABBR[(cur_m - 1) as usize];
                let x = wlw + col as f32 * step;
                grid_c.text(Text {
                    text: label.to_string(),
                    font_family: Some(self.font_family.clone()),
                    font_color: Some(month_color),
                    font_size: Some(month_font_size),
                    dominant_baseline: Some("auto".to_string()),
                    x: Some(x),
                    y: Some(mlh - 2.0),
                    ..Default::default()
                });
                last_month_col = Some(col);
            }
            // Advance date by one day
            let next = add_days(cur_y, cur_m, cur_d, 1);
            cur_y = next.0;
            cur_m = next.1;
            cur_d = next.2;
        }

        // ── Day cells ─────────────────────────────────────────────────────────
        let mut cy = sy;
        let mut cm = sm;
        let mut cd = sd;
        for day_idx in 0..total_days {
            let col = ((start_dow + day_idx) / 7) as usize;
            let row = ((start_dow + day_idx) % 7) as usize;

            let date_str = format!("{cy:04}-{cm:02}-{cd:02}");
            let color = if let Some(&val) = lookup.get(date_str.as_str()) {
                self.cell_color(val)
            } else {
                self.empty_color
            };

            let x = wlw + col as f32 * step;
            let y = mlh + row as f32 * step;

            grid_c.rect(Rect {
                color: Some(color),
                fill: Some(color),
                left: x,
                top: y,
                width: self.cell_size,
                height: self.cell_size,
                rx: Some(2.0),
                ry: Some(2.0),
                ..Default::default()
            });

            let next = add_days(cy, cm, cd, 1);
            cy = next.0;
            cm = next.1;
            cd = next.2;
        }

        c.svg()
    }
}

#[cfg(test)]
mod tests {
    use super::CalendarChart;
    use pretty_assertions::assert_eq;

    fn make_data() -> Vec<(String, f32)> {
        vec![
            ("2024-01-05".to_string(), 2.0),
            ("2024-01-10".to_string(), 5.0),
            ("2024-01-15".to_string(), 3.0),
            ("2024-02-14".to_string(), 8.0),
            ("2024-03-20".to_string(), 6.0),
            ("2024-04-01".to_string(), 1.0),
            ("2024-06-15".to_string(), 9.0),
            ("2024-09-01".to_string(), 4.0),
            ("2024-12-25".to_string(), 10.0),
        ]
    }

    #[test]
    fn calendar_chart_basic() {
        let chart = CalendarChart::new(make_data(), 2024);
        assert_eq!(
            include_str!("../../asset/calendar_chart/basic.svg"),
            chart.svg().unwrap()
        );
    }

    #[test]
    fn calendar_chart_basic_json() {
        let chart = CalendarChart::from_json(
            r##"{
                "start_date": "2024-01-01",
                "end_date": "2024-12-31",
                "title_text": "2024 Contributions",
                "cell_size": 11,
                "cell_gap": 2,
                "max_color": "#216e39",
                "min_color": "#ebedf0",
                "data": [
                    ["2024-01-05", 2],
                    ["2024-02-14", 8],
                    ["2024-06-15", 9],
                    ["2024-09-01", 4],
                    ["2024-12-25", 10]
                ]
            }"##,
        )
        .unwrap();
        assert_eq!(
            include_str!("../../asset/calendar_chart/basic_json.svg"),
            chart.svg().unwrap()
        );
    }
}
