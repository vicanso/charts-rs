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

use super::Canvas;
use super::base::{ChartBase, LabelBoxes, axis_value_params, get_y_axis_config, render_error_bar};
use super::canvas;
use super::color::*;
use super::common::*;
use super::component::*;
use super::params::*;
use super::theme::{DEFAULT_Y_AXIS_WIDTH, get_default_theme_name, get_theme};
use super::util::*;
use crate::charts::measure_text_width_family;
use serde::{Deserialize, Serialize};

/// The curve fitted through the points of a scatter series.
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
pub enum Regression {
    /// A straight line: `y = a + b·x`.
    Linear,
    /// `y = a·e^(b·x)`; fitted to the points above 0.
    Exponential,
    /// `y = a + b·ln(x)`; fitted to the points right of 0.
    Logarithmic,
    /// A polynomial of the order given by `regression_order`.
    Polynomial,
}

/// Highest order of a fitted polynomial: beyond it the curve only chases
/// the noise of the points.
const MAX_REGRESSION_ORDER: usize = 6;
/// Number of straight pieces a fitted curve is drawn with.
const REGRESSION_SEGMENTS: usize = 64;

/// A curve fitted to points by least squares.
#[derive(Clone, Debug, PartialEq)]
struct Fit {
    kind: Regression,
    /// The coefficients, lowest order first: of the polynomial in `x` (a
    /// line is one of order 1), or `[a, b]` of the other curves.
    coefficients: Vec<f64>,
}

/// Solves the linear system `matrix · x = rhs` by elimination; `None` when
/// the equations do not determine `x`.
fn solve(mut matrix: Vec<Vec<f64>>, mut rhs: Vec<f64>) -> Option<Vec<f64>> {
    let n = rhs.len();
    for col in 0..n {
        // The row with the largest entry in this column goes on top.
        let pivot =
            (col..n).max_by(|a, b| matrix[*a][col].abs().total_cmp(&matrix[*b][col].abs()))?;
        if matrix[pivot][col].abs() < 1e-12 {
            return None;
        }
        matrix.swap(col, pivot);
        rhs.swap(col, pivot);
        // The rows below lose their entry in this column.
        let (above, below) = matrix.split_at_mut(col + 1);
        let top = &above[col];
        for (offset, row) in below.iter_mut().enumerate() {
            let factor = row[col] / top[col];
            for (entry, of_top) in row[col..].iter_mut().zip(&top[col..]) {
                *entry -= factor * of_top;
            }
            rhs[col + 1 + offset] -= factor * rhs[col];
        }
    }
    let mut x = vec![0.0; n];
    for row in (0..n).rev() {
        let known: f64 = (row + 1..n).map(|k| matrix[row][k] * x[k]).sum();
        x[row] = (rhs[row] - known) / matrix[row][row];
    }
    x.iter().all(|v| v.is_finite()).then_some(x)
}

/// The polynomial of `order` closest to the points, lowest order first.
fn fit_polynomial(points: &[(f64, f64)], order: usize) -> Option<Vec<f64>> {
    let order = order.min(points.len().saturating_sub(1));
    if order == 0 {
        return None;
    }
    // Fitted in a variable that runs from about -1 to 1 over the points:
    // high powers of large x values would drown the small ones.
    let n = points.len() as f64;
    let mean = points.iter().map(|p| p.0).sum::<f64>() / n;
    let scale = points
        .iter()
        .map(|p| (p.0 - mean).abs())
        .fold(0.0, f64::max);
    if scale <= 0.0 {
        return None;
    }
    let size = order + 1;
    let mut matrix = vec![vec![0.0; size]; size];
    let mut rhs = vec![0.0; size];
    for (x, y) in points {
        let t = (x - mean) / scale;
        let powers: Vec<f64> = (0..2 * size).map(|k| t.powi(k as i32)).collect();
        for row in 0..size {
            for col in 0..size {
                matrix[row][col] += powers[row + col];
            }
            rhs[row] += powers[row] * y;
        }
    }
    let scaled = solve(matrix, rhs)?;
    // Back to a polynomial in x: every power of `(x - mean) / scale` is
    // the one before it times that line.
    let mut coefficients = vec![0.0; size];
    let mut power = vec![1.0];
    for c in scaled {
        for (k, p) in power.iter().enumerate() {
            coefficients[k] += c * p;
        }
        let mut next = vec![0.0; power.len() + 1];
        for (k, p) in power.iter().enumerate() {
            next[k] -= p * mean / scale;
            next[k + 1] += p / scale;
        }
        power = next;
    }
    Some(coefficients)
}

impl Fit {
    /// Fits a curve of `kind` to the points; `None` when they do not tell
    /// one (a single point, or all of them above each other).
    fn new(kind: Regression, order: usize, points: &[(f64, f64)]) -> Option<Fit> {
        let finite = |p: &&(f64, f64)| p.0.is_finite() && p.1.is_finite();
        let coefficients = match kind {
            Regression::Linear | Regression::Polynomial => {
                let order = if kind == Regression::Linear {
                    1
                } else {
                    order.clamp(1, MAX_REGRESSION_ORDER)
                };
                let points: Vec<(f64, f64)> = points.iter().filter(finite).copied().collect();
                fit_polynomial(&points, order)?
            }
            // A line through (x, ln y): ln y = ln a + b·x.
            Regression::Exponential => {
                let points: Vec<(f64, f64)> = points
                    .iter()
                    .filter(finite)
                    .filter(|p| p.1 > 0.0)
                    .map(|p| (p.0, p.1.ln()))
                    .collect();
                let line = fit_polynomial(&points, 1)?;
                vec![line[0].exp(), line[1]]
            }
            // A line through (ln x, y).
            Regression::Logarithmic => {
                let points: Vec<(f64, f64)> = points
                    .iter()
                    .filter(finite)
                    .filter(|p| p.0 > 0.0)
                    .map(|p| (p.0.ln(), p.1))
                    .collect();
                fit_polynomial(&points, 1)?
            }
        };
        coefficients
            .iter()
            .all(|c| c.is_finite())
            .then_some(Fit { kind, coefficients })
    }
    /// The value of the curve at `x`.
    fn value(&self, x: f64) -> f64 {
        let c = &self.coefficients;
        match self.kind {
            Regression::Linear | Regression::Polynomial => {
                c.iter().rev().fold(0.0, |sum, c| sum * x + c)
            }
            Regression::Exponential => c[0] * (c[1] * x).exp(),
            Regression::Logarithmic => c[0] + c[1] * x.ln(),
        }
    }
    /// The curve as it is written: `y = 1.5x + 2`.
    fn formula(&self) -> String {
        // Three digits tell a coefficient, whatever its size.
        let number = |value: f64| -> String {
            let value = value.abs();
            if value == 0.0 {
                return "0".to_string();
            }
            let decimals = (2 - value.log10().floor() as i32).clamp(0, 9) as usize;
            let text = format!("{value:.decimals$}");
            if text.contains('.') {
                text.trim_end_matches('0').trim_end_matches('.').to_string()
            } else {
                text
            }
        };
        let sign = |value: f64| if value < 0.0 { "-" } else { "+" };
        let lead = |value: f64| if value < 0.0 { "-" } else { "" };
        let c = &self.coefficients;
        match self.kind {
            Regression::Exponential => {
                format!(
                    "y = {}{}e^({}{}x)",
                    lead(c[0]),
                    number(c[0]),
                    lead(c[1]),
                    number(c[1])
                )
            }
            Regression::Logarithmic => format!(
                "y = {}{} {} {}ln(x)",
                lead(c[0]),
                number(c[0]),
                sign(c[1]),
                number(c[1])
            ),
            Regression::Linear | Regression::Polynomial => {
                let mut text = "y =".to_string();
                for (power, value) in c.iter().enumerate().rev() {
                    let first = power + 1 == c.len();
                    let x = match power {
                        0 => String::new(),
                        1 => "x".to_string(),
                        _ => format!("x^{power}"),
                    };
                    // `x`, not `1x`.
                    let mut factor = number(*value);
                    if power > 0 && factor == "1" {
                        factor.clear();
                    }
                    if first {
                        text.push_str(&format!(" {}{factor}{x}", lead(*value)));
                    } else {
                        text.push_str(&format!(" {} {factor}{x}", sign(*value)));
                    }
                }
                text
            }
        }
    }
}

/// A scatter chart of (x, y) point pairs.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ScatterChart {
    /// The shared chart options (size, series, title/legend, axes); exposed
    /// directly on the chart through `Deref`, e.g. `chart.title.text`.
    pub base: ChartBase,
    // x axis
    /// Configuration of the value x axis.
    pub x_axis_config: YAxisConfig,

    // y axis
    /// Y axis configurations; one per axis, up to two.
    pub y_axis_configs: Vec<YAxisConfig>,

    // grid

    // series

    // symbol
    /// Marker radius per series.
    pub series_symbol_sizes: Vec<f32>,
    /// Per-series symbol shapes. When empty the chart cycles through
    /// Circle → Triangle → Rect → Diamond by series index.
    /// `series.symbol` (if Some) overrides all per-series symbols.
    pub series_symbols: Vec<Symbol>,

    // bubble
    /// Bubble chart: the series data are `[x, y, size]` triples instead of
    /// `[x, y]` pairs, and each symbol's radius follows its size (by area),
    /// between `bubble_min_size` and `bubble_max_size`.
    pub bubble: bool,
    /// Radius of the smallest bubble. Default: 4.
    pub bubble_min_size: f32,
    /// Radius of the largest bubble. Default: 30.
    pub bubble_max_size: f32,

    // regression
    /// Draws the curve of this kind that fits the points of each series
    /// best (by least squares), in the color of the series.
    pub regression: Option<Regression>,
    /// Order of a `Polynomial` regression, from 1 to 6. Default: 2.
    pub regression_order: usize,
    /// Writes the formula of each fitted curve at its end.
    pub regression_label_show: bool,
}

impl std::ops::Deref for ScatterChart {
    type Target = ChartBase;
    fn deref(&self) -> &ChartBase {
        &self.base
    }
}
impl std::ops::DerefMut for ScatterChart {
    fn deref_mut(&mut self) -> &mut ChartBase {
        &mut self.base
    }
}

#[allow(clippy::too_many_arguments)]
fn render_scatter_symbol(
    canvas: &mut Canvas,
    symbol: &Symbol,
    cx: f32,
    cy: f32,
    r: f32,
    color: Color,
    title: Option<String>,
    tip_font: &FontConfig,
    dataset: Vec<(String, String)>,
) {
    // When a tooltip is present the symbol is the hover trigger; a hidden
    // `.ct-tip` label is drawn right after it (adjacent-sibling reveal).
    let class = title.as_ref().map(|_| "ct-trigger".to_string());
    match symbol {
        Symbol::Circle(_, fill_override) => {
            canvas.circle(Circle {
                fill: Some(fill_override.unwrap_or(color)),
                cx,
                cy,
                r,
                title: title.clone(),
                dataset: dataset.clone(),
                class,
                ..Default::default()
            });
        }
        Symbol::Rect(_, fill_override) => {
            canvas.rect(Rect {
                fill: Some(fill_override.unwrap_or(color).into()),
                left: cx - r,
                top: cy - r,
                width: r * 2.0,
                height: r * 2.0,
                title: title.clone(),
                dataset: dataset.clone(),
                class,
                ..Default::default()
            });
        }
        Symbol::Triangle(_, fill_override) => {
            canvas.polygon(Polygon {
                fill: Some(fill_override.unwrap_or(color)),
                points: vec![
                    (cx, cy - r).into(),
                    (cx + r * 0.866, cy + r * 0.5).into(),
                    (cx - r * 0.866, cy + r * 0.5).into(),
                ],
                title: title.clone(),
                dataset: dataset.clone(),
                class,
                ..Default::default()
            });
        }
        Symbol::Diamond(_, fill_override) => {
            canvas.polygon(Polygon {
                fill: Some(fill_override.unwrap_or(color)),
                points: vec![
                    (cx, cy - r).into(),
                    (cx + r, cy).into(),
                    (cx, cy + r).into(),
                    (cx - r, cy).into(),
                ],
                title: title.clone(),
                dataset: dataset.clone(),
                class,
                ..Default::default()
            });
        }
        Symbol::None => return,
    }
    if let Some(text) = title {
        // What the font of the tooltips leaves unset is left to the viewer.
        canvas.text_unmeasured(Text {
            text,
            class: Some("ct-tip".to_string()),
            font_size: (tip_font.size > 0.0).then_some(tip_font.size),
            font_color: (!tip_font.color.is_zero()).then_some(tip_font.color),
            font_weight: tip_font.weight.clone(),
            x: Some(cx),
            y: Some(cy),
            dy: Some(-8.0),
            text_anchor: Some("middle".to_string()),
            ..Default::default()
        });
    }
}

impl ScatterChart {
    /// Creates a scatter chart from json.
    pub fn from_json(data: &str) -> canvas::Result<ScatterChart> {
        let mut s = ScatterChart {
            ..Default::default()
        };
        let value =
            s.base
                .fill_option(data, &mut s.y_axis_configs, super::schema::SCATTER_FIELDS)?;
        s.fill_default();

        if let Some(series_symbol_sizes) = get_f32_slice_from_value(&value, "series_symbol_sizes") {
            s.series_symbol_sizes = series_symbol_sizes;
        }
        if let Some(arr) = value.get("series_symbols").and_then(|v| v.as_array()) {
            s.series_symbols = arr
                .iter()
                .filter_map(|item| match item {
                    // Just the type: "circle", "triangle", etc.
                    serde_json::Value::String(kind) => {
                        Some(get_symbol_from_object(&serde_json::json!({ "type": kind })))
                    }
                    serde_json::Value::Object(_) => Some(get_symbol_from_object(item)),
                    _ => None,
                })
                .collect();
        }
        if let Some(bubble) = get_bool_from_value(&value, "bubble") {
            s.bubble = bubble;
        }
        if let Some(size) = get_f32_from_value(&value, "bubble_min_size") {
            s.bubble_min_size = size;
        }
        if let Some(size) = get_f32_from_value(&value, "bubble_max_size") {
            s.bubble_max_size = size;
        }
        if let Some(kind) = get_string_from_value(&value, "regression") {
            s.regression = match kind.to_lowercase().as_str() {
                "linear" => Some(Regression::Linear),
                "exponential" => Some(Regression::Exponential),
                "logarithmic" => Some(Regression::Logarithmic),
                "polynomial" => Some(Regression::Polynomial),
                _ => None,
            };
        }
        if let Some(order) = get_usize_from_value(&value, "regression_order") {
            s.regression_order = order;
        }
        if let Some(show) = get_bool_from_value(&value, "regression_label_show") {
            s.regression_label_show = show;
        }
        s.fill_default();
        let theme = get_string_from_value(&value, "theme").unwrap_or_default();
        if let Some(x_axis_config) = value.get("x_axis_config") {
            s.x_axis_config = get_y_axis_config_from_value(get_theme(&theme), x_axis_config);
        }
        Ok(s)
    }
    /// Creates a scatter chart with  theme.
    pub fn new_with_theme(series_list: Vec<Series>, theme: &str) -> ScatterChart {
        let mut s = ScatterChart {
            ..Default::default()
        };
        s.series_list = series_list;
        let theme = get_theme(theme);
        s.base.fill_theme(theme, &mut s.y_axis_configs);
        s.fill_default();

        s
    }
    fn fill_default(&mut self) {
        if self.y_axis_configs[0].stroke_color.is_zero() {
            self.y_axis_configs[0].stroke_color = self.x_axis.stroke_color;
        }
        if self.x_axis_config.split_number == 0 {
            self.x_axis_config = self.y_axis_configs[0].clone();
            // The y axis' title is not the x axis' (that is `x_axis.title`).
            self.x_axis_config.title = None;
        }
        self.x_axis.boundary_gap = Some(false);
        if self.bubble_min_size <= 0.0 {
            self.bubble_min_size = 4.0;
        }
        if self.bubble_max_size < self.bubble_min_size {
            self.bubble_max_size = self.bubble_min_size.max(30.0);
        }
        if self.regression_order == 0 {
            self.regression_order = 2;
        }
    }
    /// Creates a scatter chart with default theme.
    pub fn new(series_list: Vec<Series>) -> ScatterChart {
        ScatterChart::new_with_theme(series_list, &get_default_theme_name())
    }
    /// Converts scatter chart to svg.
    pub fn svg(&self) -> canvas::Result<String> {
        let mut c = self.new_canvas();

        let mut x_axis_height = self.x_axis.height;
        if self.x_axis.hidden {
            x_axis_height = 0.0;
        }
        let axis_top = self.render_header(&mut c);
        let titles =
            self.reserve_axis_titles(&mut c, &self.y_axis_configs, &self.x_axis.title, false);

        let y_axis_config = get_y_axis_config(&self.y_axis_configs, 0);

        // Points are `[x, y]` pairs, or `[x, y, size]` triples for bubbles.
        let dimension = if self.bubble { 3 } else { 2 };
        let mut y_axis_data_list = vec![];
        let mut x_axis_data_list = vec![];
        let (mut size_min, mut size_max) = (f32::MAX, f32::MIN);
        for series in self.series_list.iter() {
            // The error bars have to fit on the y axis as well.
            if let Some(error) = &series.error_bar {
                y_axis_data_list.extend(error.values());
            }
            for (index, data) in series.iter_values().enumerate() {
                match index % dimension {
                    0 => x_axis_data_list.push(data),
                    1 => y_axis_data_list.push(data),
                    _ if data != NIL_VALUE => {
                        size_min = size_min.min(data);
                        size_max = size_max.max(data);
                    }
                    _ => {}
                }
            }
        }
        // A bubble's area follows its size, so the radius follows the root.
        let bubble_radius = |size: f32| -> f32 {
            if size_max > size_min {
                let ratio = ((size - size_min) / (size_max - size_min)).clamp(0.0, 1.0);
                self.bubble_min_size + (self.bubble_max_size - self.bubble_min_size) * ratio.sqrt()
            } else {
                (self.bubble_min_size + self.bubble_max_size) / 2.0
            }
        };
        let y_axis_values =
            get_axis_values(axis_value_params(&y_axis_config, y_axis_data_list, true));
        let y_axis_width = if self.y_axis_hidden {
            0.0
        } else if let Some(value) = y_axis_config.width {
            value
        } else {
            let y_axis_formatter = &y_axis_config.formatter.clone().unwrap_or_default();
            let str = format_string(&y_axis_values.data[0], y_axis_formatter);
            if let Ok(b) =
                measure_text_width_family(&self.font_family, y_axis_config.font.size, &str)
            {
                b.width() + 5.0
            } else {
                DEFAULT_Y_AXIS_WIDTH
            }
        };

        let axis_height = c.height() - x_axis_height - axis_top;
        let axis_width = c.width() - y_axis_width;
        self.render_axis_titles(
            &titles,
            &self.y_axis_configs,
            &self.x_axis.title,
            y_axis_width,
            axis_top,
            axis_width,
            axis_height,
        );
        // minus the height of top text area
        if axis_top > 0.0 {
            c = c.child(Box {
                top: axis_top,
                ..Default::default()
            });
        }

        // grid
        self.render_grid(
            c.child(Box {
                left: y_axis_width,
                ..Default::default()
            }),
            &self.y_axis_configs,
            axis_width,
            axis_height,
        );
        let x_axis_width = c.width() - y_axis_width;
        c.child(Box {
            left: y_axis_width,
            ..Default::default()
        })
        .grid(Grid {
            right: x_axis_width,
            bottom: axis_height,
            color: Some(self.grid.stroke_color),
            stroke_width: self.grid.stroke_width,
            stroke_dash_array: self.grid.stroke_dash_array.clone(),
            verticals: y_axis_config.split_number,
            hidden_verticals: vec![0],
            ..Default::default()
        });

        // y axis
        if !self.y_axis_hidden {
            self.render_y_axis(
                c.child(Box::default()),
                &self.y_axis_configs,
                y_axis_values.data.clone(),
                axis_height,
                y_axis_width,
                0,
            );
        }

        // x axis
        let x_axis_values = get_axis_values(axis_value_params(
            &self.x_axis_config,
            x_axis_data_list,
            false,
        ));
        let x_axis_formatter = &self.x_axis_config.formatter.clone().unwrap_or_default();
        let content_width = c.width() - y_axis_width;
        let content_height = axis_height;
        if !self.x_axis.hidden {
            self.render_x_axis(
                c.child(Box {
                    top: c.height() - x_axis_height,
                    left: y_axis_width,
                    ..Default::default()
                }),
                x_axis_values
                    .data
                    .iter()
                    .map(|item| format_string(item, x_axis_formatter))
                    .collect(),
                axis_width,
            );
        }

        // Default cycling order when no per-series symbol is configured.
        const DEFAULT_SYMBOLS: [Symbol; 4] = [
            Symbol::Circle(0.0, None),
            Symbol::Triangle(0.0, None),
            Symbol::Rect(0.0, None),
            Symbol::Diamond(0.0, None),
        ];

        // render dot
        let mut content_canvas = c.child(Box {
            left: y_axis_width,
            ..Default::default()
        });
        let default_symbol_size = 10.0_f32;
        // Where the formulas of the fitted curves are written.
        let mut formula_boxes = LabelBoxes::new(true);
        for (index, series) in self.series_list.iter().enumerate() {
            let series_idx = series.index.unwrap_or(index);
            let mut color = get_color(&self.series.colors, series_idx);
            let size = *self
                .series_symbol_sizes
                .get(series_idx)
                .unwrap_or(&default_symbol_size);
            color = color.with_alpha(210);

            // Resolve which symbol to use for this series.
            // series_symbols takes precedence; otherwise cycle through defaults.
            // (`series.symbol` is intentionally ignored here — it's set by fill_theme
            //  for line-chart node colors and is not meaningful for scatter dots.)
            let symbol = if let Some(s) = self.series_symbols.get(series_idx) {
                s.clone()
            } else if self.bubble {
                // A bubble's size reads as the area of a circle.
                Symbol::Circle(0.0, None)
            } else {
                DEFAULT_SYMBOLS[index % DEFAULT_SYMBOLS.len()].clone()
            };

            // (x, y, bubble size, error bounds) of every complete point.
            type Bounds = Option<(f32, f32)>;
            let mut points: Vec<(f32, f32, Option<f32>, Bounds)> = vec![];
            let mut coords = series.iter_values();
            let mut point_index = 0;
            while let (Some(x_value), Some(y_value)) = (coords.next(), coords.next()) {
                let bubble_size = if self.bubble {
                    coords.next().filter(|v| *v != NIL_VALUE)
                } else {
                    None
                };
                let error = series
                    .error_bar
                    .as_ref()
                    .and_then(|e| e.bounds(point_index));
                point_index += 1;
                if x_value != NIL_VALUE && y_value != NIL_VALUE {
                    points.push((x_value, y_value, bubble_size, error));
                }
            }
            // Large bubbles first, so small ones are not buried under them.
            if self.bubble {
                points.sort_by(|a, b| b.2.unwrap_or(0.0).total_cmp(&a.2.unwrap_or(0.0)));
            }
            // The curve fitted to the points, kept for after they are drawn.
            let fit = self.regression.and_then(|kind| {
                let pairs: Vec<(f64, f64)> =
                    points.iter().map(|p| (p.0 as f64, p.1 as f64)).collect();
                // From the leftmost to the rightmost point; a logarithm
                // only knows the ones right of 0.
                let xs = pairs
                    .iter()
                    .map(|p| p.0)
                    .filter(|x| kind != Regression::Logarithmic || *x > 0.0);
                let (from, to) = xs.fold((f64::INFINITY, f64::NEG_INFINITY), |(lo, hi), x| {
                    (lo.min(x), hi.max(x))
                });
                let fit = Fit::new(kind, self.regression_order, &pairs)?;
                (from < to).then_some((fit, from, to))
            });
            for (x_value, y_value, bubble_size, error) in points {
                let cx = content_width - x_axis_values.get_offset_height(x_value, content_width);
                let cy = y_axis_values.get_offset_height(y_value, content_height);
                let radius = bubble_size.map(bubble_radius).unwrap_or(size);
                // The error bar of the point, under its symbol.
                if let Some((lower, upper)) = error {
                    let ends = (
                        y_axis_values.get_offset_height(lower, content_height),
                        y_axis_values.get_offset_height(upper, content_height),
                    );
                    let stroke = (
                        color,
                        SeriesBand::error_bar_width(series.error_bar.as_ref()),
                    );
                    render_error_bar(&mut content_canvas, cx, ends, 4.0, stroke);
                }
                let title = if self.tooltip.show {
                    Some(match bubble_size {
                        Some(v) => format!(
                            "{}: ({}, {}, {})",
                            series.name,
                            format_float(x_value),
                            format_float(y_value),
                            format_float(v)
                        ),
                        None => format!(
                            "{}: ({}, {})",
                            series.name,
                            format_float(x_value),
                            format_float(y_value)
                        ),
                    })
                } else {
                    None
                };
                let mut dataset = vec![
                    ("series".to_string(), series.name.clone()),
                    ("x".to_string(), format_float(x_value)),
                    ("y".to_string(), format_float(y_value)),
                ];
                if let Some(v) = bubble_size {
                    dataset.push(("size".to_string(), format_float(v)));
                }
                if let Some((lower, upper)) = error {
                    dataset.push(("lower".to_string(), format_float(lower)));
                    dataset.push(("upper".to_string(), format_float(upper)));
                }
                render_scatter_symbol(
                    &mut content_canvas,
                    &symbol,
                    cx,
                    cy,
                    radius,
                    color,
                    title,
                    &self.tooltip.font,
                    dataset,
                );
            }

            let Some((fit, mut from, mut to)) = fit else {
                continue;
            };
            let (low, high) = (y_axis_values.min as f64, y_axis_values.max as f64);
            // A straight line is cut where it leaves the plot; a curve is
            // drawn in short pieces, those outside of the plot left out.
            let straight = fit.coefficients.len() == 2
                && fit.kind != Regression::Exponential
                && fit.kind != Regression::Logarithmic;
            if straight {
                let (a, b) = (fit.coefficients[0], fit.coefficients[1]);
                if b.abs() > 1e-12 {
                    let (x_low, x_high) = ((low - a) / b, (high - a) / b);
                    from = from.max(x_low.min(x_high));
                    to = to.min(x_low.max(x_high));
                } else if a < low || a > high {
                    continue;
                }
                if from >= to {
                    continue;
                }
            }
            let steps = if straight { 1 } else { REGRESSION_SEGMENTS };
            let mut runs: Vec<Vec<Point>> = vec![vec![]];
            for i in 0..=steps {
                let x = from + (to - from) * i as f64 / steps as f64;
                let y = fit.value(x);
                // A hair of slack for the ends of a line cut at the edge.
                let slack = (high - low) * 1e-6;
                if !y.is_finite() || y < low - slack || y > high + slack {
                    if runs.last().is_some_and(|run| !run.is_empty()) {
                        runs.push(vec![]);
                    }
                    continue;
                }
                let px = content_width - x_axis_values.get_offset_height(x as f32, content_width);
                let py = y_axis_values.get_offset_height(y as f32, content_height);
                if let Some(run) = runs.last_mut() {
                    run.push((px, py).into());
                }
            }
            let line_color = get_color(&self.series.colors, series_idx);
            let mut end = None;
            for run in runs.into_iter().filter(|run| run.len() > 1) {
                end = run.last().copied();
                content_canvas.straight_line(StraightLine {
                    color: Some(line_color),
                    points: run,
                    stroke_width: self.series.stroke_width,
                    symbol: None,
                    class: Some("ct-regression".to_string()),
                    ..Default::default()
                });
            }
            // The formula, at the end of the curve: above it, or below it
            // where that is off the plot or taken by another formula.
            if self.regression_label_show
                && let Some(end) = end
            {
                let text = fit.formula();
                let font_size = self.series.label.font.size;
                let width = measure_text_width_family(&self.font_family, font_size, &text)
                    .map(|b| b.width())
                    .unwrap_or_default();
                let left = (end.x - width).max(0.0);
                let baseline = [end.y - 8.0, end.y + font_size + 6.0]
                    .into_iter()
                    .filter(|y| *y - font_size >= 0.0 && *y <= content_height)
                    .find(|y| formula_boxes.try_place(left, y - font_size, width, font_size));
                if let Some(baseline) = baseline {
                    content_canvas.text_unmeasured(Text {
                        text,
                        font_family: Some(self.font_family.clone()),
                        font_color: Some(self.series.label.font.color),
                        font_size: Some(font_size),
                        font_weight: self.series.label.font.weight.clone(),
                        x: Some(left),
                        y: Some(baseline),
                        ..Default::default()
                    });
                }
            }
        }

        if self.tooltip.show {
            c.svg_with_style(TOOLTIP_STYLE)
        } else {
            c.svg()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Fit, Regression, ScatterChart};
    use crate::Align;

    fn close(found: &[f64], wanted: &[f64]) {
        assert_eq!(wanted.len(), found.len(), "{found:?}");
        for (a, b) in found.iter().zip(wanted) {
            assert!(
                (a - b).abs() < 1e-6 * b.abs().max(1.0),
                "{found:?} is not {wanted:?}"
            );
        }
    }

    #[test]
    fn fitted_curves() {
        // Points on a line give the line back.
        let line = Fit::new(Regression::Linear, 2, &[(0.0, 1.0), (1.0, 3.0), (2.0, 5.0)]).unwrap();
        close(&line.coefficients, &[1.0, 2.0]);
        assert_eq!("y = 2x + 1", line.formula());
        assert!((line.value(10.0) - 21.0).abs() < 1e-9);
        // Otherwise the line closest to them, by least squares.
        let line = Fit::new(Regression::Linear, 2, &[(0.0, 0.0), (1.0, 1.0), (2.0, 1.0)]).unwrap();
        close(&line.coefficients, &[1.0 / 6.0, 0.5]);
        assert_eq!("y = 0.5x + 0.167", line.formula());
        let down = Fit::new(Regression::Linear, 2, &[(0.0, -1.0), (1.0, -3.0)]).unwrap();
        assert_eq!("y = -2x - 1", down.formula());

        // A parabola: y = x² - 2x + 3.
        let points: Vec<(f64, f64)> = (-2..=3)
            .map(|x| (x as f64, (x * x - 2 * x + 3) as f64))
            .collect();
        let parabola = Fit::new(Regression::Polynomial, 2, &points).unwrap();
        close(&parabola.coefficients, &[3.0, -2.0, 1.0]);
        assert_eq!("y = x^2 - 2x + 3", parabola.formula());
        // A higher order than the points can tell is cut down to it, and
        // to 6 at most.
        let cut = Fit::new(Regression::Polynomial, 5, &points[..3]).unwrap();
        assert_eq!(3, cut.coefficients.len());
        let many: Vec<(f64, f64)> = (0..20).map(|x| (x as f64, (x % 3) as f64)).collect();
        assert_eq!(
            7,
            Fit::new(Regression::Polynomial, 50, &many)
                .unwrap()
                .coefficients
                .len()
        );
        // Far from 0 the fit holds as well: y = x²/2 around x = 1000.
        let far: Vec<(f64, f64)> = (1000..=1010)
            .map(|x| (x as f64, (x * x) as f64 / 2.0))
            .collect();
        let fit = Fit::new(Regression::Polynomial, 2, &far).unwrap();
        assert!(
            (fit.value(1005.0) - 505_012.5).abs() < 1e-3,
            "{}",
            fit.value(1005.0)
        );

        // y = 2·e^(x/2), and y = 1 + 3·ln(x).
        let points: Vec<(f64, f64)> = (0..5)
            .map(|x| (x as f64, 2.0 * (x as f64 / 2.0).exp()))
            .collect();
        let exponential = Fit::new(Regression::Exponential, 2, &points).unwrap();
        close(&exponential.coefficients, &[2.0, 0.5]);
        assert_eq!("y = 2e^(0.5x)", exponential.formula());
        let points: Vec<(f64, f64)> = (1..6)
            .map(|x| (x as f64, 1.0 + 3.0 * (x as f64).ln()))
            .collect();
        let logarithmic = Fit::new(Regression::Logarithmic, 2, &points).unwrap();
        close(&logarithmic.coefficients, &[1.0, 3.0]);
        assert_eq!("y = 1 + 3ln(x)", logarithmic.formula());
        // Points that have no logarithm are left out of the fit.
        let mixed = [
            (-1.0, 5.0),
            (0.0, 9.0),
            (1.0, 1.0),
            (std::f64::consts::E, 4.0),
        ];
        close(
            &Fit::new(Regression::Logarithmic, 2, &mixed)
                .unwrap()
                .coefficients,
            &[1.0, 3.0],
        );

        // Nothing to fit: one point, points above each other, no numbers.
        for points in [
            vec![(1.0, 2.0)],
            vec![(1.0, 2.0), (1.0, 5.0)],
            vec![(f64::NAN, 1.0), (2.0, f64::INFINITY)],
            vec![],
        ] {
            for kind in [
                Regression::Linear,
                Regression::Polynomial,
                Regression::Exponential,
                Regression::Logarithmic,
            ] {
                assert_eq!(None, Fit::new(kind, 2, &points), "{kind:?} {points:?}");
            }
        }
    }

    fn make_scatter() -> ScatterChart {
        let mut scatter_chart = ScatterChart::new(vec![
            (
                "Female",
                vec![
                    161.2, 51.6, 167.5, 59.0, 159.5, 49.2, 157.0, 63.0, 155.8, 53.6, 170.0, 59.0,
                    159.1, 47.6, 166.0, 69.8, 176.2, 66.8, 160.2, 75.2, 172.5, 55.2, 170.9, 54.2,
                    172.9, 62.5, 153.4, 42.0, 160.0, 50.0, 147.2, 49.8, 168.2, 49.2, 175.0, 73.2,
                    157.0, 47.8, 167.6, 68.8, 159.5, 50.6, 175.0, 82.5, 166.8, 57.2, 176.5, 87.8,
                    170.2, 72.8,
                ],
            )
                .into(),
            (
                "Male",
                vec![
                    174.0, 65.6, 175.3, 71.8, 193.5, 80.7, 186.5, 72.6, 187.2, 78.8, 181.5, 74.8,
                    184.0, 86.4, 184.5, 78.4, 175.0, 62.0, 184.0, 81.6, 180.0, 76.6, 177.8, 83.6,
                    192.0, 90.0, 176.0, 74.6, 174.0, 71.0, 184.0, 79.6, 192.7, 93.8, 171.5, 70.0,
                    173.0, 72.4, 176.0, 85.9, 176.0, 78.8, 180.5, 77.8, 172.7, 66.2, 176.0, 86.4,
                    173.5, 81.8,
                ],
            )
                .into(),
        ]);
        scatter_chart.title.text = "Male and female height and weight distribution".to_string();
        scatter_chart.margin.right = 20.0;
        scatter_chart.title.align = Align::Left;
        scatter_chart.sub_title.text = "Data from: Heinz 2003".to_string();
        scatter_chart.sub_title.align = Align::Left;
        scatter_chart.legend.align = Align::Right;
        scatter_chart.y_axis_configs[0].min = Some(40.0);
        scatter_chart.y_axis_configs[0].max = Some(130.0);
        scatter_chart.y_axis_configs[0].formatter = Some("{c} kg".to_string());
        scatter_chart.x_axis_config.min = Some(140.0);
        scatter_chart.x_axis_config.max = Some(230.0);
        scatter_chart.x_axis_config.formatter = Some("{c} cm".to_string());
        scatter_chart.series_symbol_sizes = vec![6.0, 6.0];
        scatter_chart
    }

    #[test]
    fn scatter_chart_basic() {
        assert_snapshot!("scatter_chart/basic.svg", make_scatter().svg().unwrap());
    }

    #[test]
    fn scatter_chart_no_axis() {
        let mut scatter_chart = make_scatter();
        scatter_chart.x_axis.hidden = true;
        scatter_chart.y_axis_hidden = true;
        assert_snapshot!("scatter_chart/no_axis.svg", scatter_chart.svg().unwrap());
    }

    #[test]
    fn scatter_chart_tooltip() {
        let chart = ScatterChart::from_json(
            r#"{"tooltip_show": true, "series_list": [{"name": "a", "data": [1, 2, 3, 4]}]}"#,
        )
        .unwrap();
        let svg = chart.svg().unwrap();
        assert!(svg.contains("<title>"), "missing scatter title");
        assert!(svg.contains(r#"class="ct-tip""#), "missing hover label");
        assert!(
            svg.contains(".ct-trigger:hover+.ct-tip"),
            "missing hover css"
        );
        let off =
            ScatterChart::from_json(r#"{"series_list": [{"name": "a", "data": [1, 2, 3, 4]}]}"#)
                .unwrap();
        let off_svg = off.svg().unwrap();
        assert!(!off_svg.contains("<title>"));
        assert!(!off_svg.contains("ct-tip"));
    }
}
