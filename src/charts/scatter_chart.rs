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
/// Most bins a marginal histogram draws. More than this are hairlines
/// beside the plot, not a distribution.
const MAX_MARGINAL_BINS: usize = 40;
/// Default height of the top histogram and width of the right one.
const DEFAULT_MARGINAL_SIZE: f32 = 64.0;
/// Pixels kept between a marginal bar and the plot, so the bar does not
/// cover the axis it grows from.
const MARGINAL_PLOT_GAP: f32 = 1.0;
/// A series with fewer points than this has no density worth a line.
const MIN_CONTOUR_POINTS: usize = 8;
/// Lines drawn when `levels` is 0: at 20%, 40%, 60% and 80% of the peak.
const DEFAULT_CONTOUR_LEVELS: usize = 4;
/// More lines than this crowd the same peak.
const MAX_CONTOUR_LEVELS: usize = 8;
/// Cells along each axis of the density grid when `grid_size` is 0.
const DEFAULT_CONTOUR_GRID: usize = 64;
/// The grid stays inside this range: coarser than 16 misses the shape,
/// finer than 128 only lengthens the same lines.
const MIN_CONTOUR_GRID: usize = 16;
const MAX_CONTOUR_GRID: usize = 128;
/// Width of a contour line.
const CONTOUR_STROKE: f32 = 1.5;
/// A kernel sample beyond this many bandwidths is left out of a node.
const KERNEL_REACH: f64 = 4.0;

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

/// Histograms of a scatter chart's own points: the distribution of `x`
/// above the plot and the distribution of `y` to its right. The bins span
/// the axes, so a bar sits over the points it counts.
#[derive(Clone, Debug, PartialEq)]
pub struct MarginalHistogram {
    /// Height of the top histogram and width of the right one, in pixels.
    /// Default: 64. The plot keeps at least 40 pixels; a chart too small
    /// for that draws neither histogram.
    pub size: f32,
    /// Number of bins along each axis. `0` (the default) derives it from
    /// the number of points. Never more than 40.
    pub bin_count: usize,
    /// Gap between adjacent bars, in pixels. Default: 1.
    pub bar_gap: f32,
}

impl Default for MarginalHistogram {
    fn default() -> Self {
        Self {
            size: DEFAULT_MARGINAL_SIZE,
            bin_count: 0,
            bar_gap: 1.0,
        }
    }
}

/// Equal-width bins from an axis minimum to its maximum.
#[derive(Clone, Copy, Debug, PartialEq)]
struct MarginalBins {
    start: f64,
    width: f64,
    count: usize,
}

impl MarginalBins {
    fn edge(&self, i: usize) -> f64 {
        self.start + self.width * i as f64
    }
    /// The bin `value` falls in. Every bin holds its left edge, and the last
    /// one its right edge too. A value outside the axis belongs to no bin.
    fn index(&self, value: f64) -> Option<usize> {
        let slack = self.width * 1e-9;
        if value < self.start - slack || value > self.edge(self.count) + slack {
            return None;
        }
        let index = ((value - self.start) / self.width).floor().max(0.0) as usize;
        Some(index.min(self.count - 1))
    }
}

/// Bins spanning `min..=max`. `bin_count` of 0 derives the number from a
/// sample of `n` points. `None` when the axis has no range to divide.
fn marginal_bins(min: f32, max: f32, n: usize, bin_count: usize) -> Option<MarginalBins> {
    let (min, max) = (f64::from(min), f64::from(max));
    if !min.is_finite() || !max.is_finite() || max <= min {
        return None;
    }
    let count = if bin_count > 0 {
        bin_count
    } else {
        (n.max(1) as f64).log2().ceil() as usize + 1
    };
    let count = count.clamp(1, MAX_MARGINAL_BINS);
    Some(MarginalBins {
        start: min,
        width: (max - min) / count as f64,
        count,
    })
}

fn bin_counts(bins: MarginalBins, values: impl Iterator<Item = f32>) -> Vec<u32> {
    let mut counts = vec![0; bins.count];
    for value in values {
        if let Some(index) = bins.index(f64::from(value)) {
            counts[index] += 1;
        }
    }
    counts
}

/// Complete `(x, y)` points of a series. A bubble chart stores them as
/// `x, y, size` triples; the size is not a coordinate.
fn plotted_point_count(series: &Series, bubble: bool) -> usize {
    let mut coords = series.iter_values();
    let mut count = 0;
    while let (Some(x), Some(y)) = (coords.next(), coords.next()) {
        if bubble {
            coords.next();
        }
        if x != NIL_VALUE && y != NIL_VALUE {
            count += 1;
        }
    }
    count
}

/// Which side of the plot a marginal histogram belongs to.
#[derive(Clone, Copy, PartialEq, Eq)]
enum MarginalSide {
    /// The distribution of x, above the plot. Bars grow upwards.
    X,
    /// The distribution of y, to the right of the plot. Bars grow rightwards.
    Y,
}

/// The counts of one series, in the shared bins of each axis.
struct MarginalSeries {
    index: usize,
    name: String,
    x: Vec<u32>,
    y: Vec<u32>,
}

impl MarginalSeries {
    fn counts(&self, side: MarginalSide) -> &[u32] {
        match side {
            MarginalSide::X => &self.x,
            MarginalSide::Y => &self.y,
        }
    }
}

/// Unfilled density contours of each scatter series. The density is a
/// Gaussian kernel estimate of that series' own points, and the lines sit
/// at even fractions of its highest density. `None` on the chart draws the
/// points alone.
#[derive(Clone, Debug, PartialEq)]
pub struct DensityContour {
    /// How many lines to draw. They fall at `k / (levels + 1)` of the
    /// highest density, so 4 (the default) is 20%, 40%, 60% and 80%.
    /// `0` is those four. Never more than 8.
    pub levels: usize,
    /// Cells along each axis of the grid the density is estimated on.
    /// `0` is 64, the default. Clamped to 16..=128.
    pub grid_size: usize,
}

impl Default for DensityContour {
    fn default() -> Self {
        Self {
            levels: DEFAULT_CONTOUR_LEVELS,
            grid_size: DEFAULT_CONTOUR_GRID,
        }
    }
}

impl DensityContour {
    /// The line count and the grid the chart actually draws with.
    fn resolved(&self) -> (usize, usize) {
        let levels = if self.levels == 0 {
            DEFAULT_CONTOUR_LEVELS
        } else {
            self.levels.min(MAX_CONTOUR_LEVELS)
        };
        let grid_size = if self.grid_size == 0 {
            DEFAULT_CONTOUR_GRID
        } else {
            self.grid_size.clamp(MIN_CONTOUR_GRID, MAX_CONTOUR_GRID)
        };
        (levels, grid_size)
    }
}

/// One contour line, in the scale coordinates of the axes: a linear value,
/// or the logarithm of a value on a log axis.
#[derive(Debug)]
struct ContourLine {
    points: Vec<(f64, f64)>,
    closed: bool,
    /// 1-based. `1` is the lowest line, the widest ring around the peak.
    level: usize,
}

/// The axis, as the coordinate the kernel is estimated in. `None` when the
/// axis has no span.
fn scale_span(axis: &AxisValues) -> Option<(f64, f64)> {
    let (min, max) = match axis.scale {
        AxisScale::Linear => (f64::from(axis.min), f64::from(axis.max)),
        AxisScale::Log(base) => {
            let base = f64::from(base);
            if base <= 0.0 || (base - 1.0).abs() < 1e-9 {
                return None;
            }
            let min = f64::from(axis.min.max(f32::MIN_POSITIVE)).log(base);
            let max = f64::from(axis.max.max(f32::MIN_POSITIVE)).log(base);
            (min, max)
        }
    };
    (min.is_finite() && max.is_finite() && max > min).then_some((min, max))
}

/// `value` in the same coordinate as [`scale_span`]. A missing value, a
/// non-positive value on a log axis, and a value outside the axis are left
/// out.
fn to_scale(axis: &AxisValues, value: f32) -> Option<f64> {
    let (min, max) = scale_span(axis)?;
    let scaled = match axis.scale {
        AxisScale::Linear => f64::from(value),
        AxisScale::Log(base) => {
            if value <= 0.0 {
                return None;
            }
            f64::from(value).log(f64::from(base))
        }
    };
    if !scaled.is_finite() {
        return None;
    }
    let slack = (max - min) * 1e-9;
    (scaled >= min - slack && scaled <= max + slack).then_some(scaled.clamp(min, max))
}

/// The value whose scale coordinate is `scaled`, ready for the axis.
fn from_scale(axis: &AxisValues, scaled: f64) -> f32 {
    match axis.scale {
        AxisScale::Linear => scaled as f32,
        AxisScale::Log(base) => f64::from(base).powf(scaled) as f32,
    }
}

/// Points of `series` that land on both axes. A bubble's size is skipped:
/// it is not a coordinate.
fn contour_samples(
    series: &Series,
    bubble: bool,
    x_axis: &AxisValues,
    y_axis: &AxisValues,
) -> Vec<(f64, f64)> {
    let mut coords = series.iter_values();
    let mut samples = Vec::new();
    while let (Some(x), Some(y)) = (coords.next(), coords.next()) {
        if bubble {
            coords.next();
        }
        if x == NIL_VALUE || y == NIL_VALUE {
            continue;
        }
        let (Some(x), Some(y)) = (to_scale(x_axis, x), to_scale(y_axis, y)) else {
            continue;
        };
        samples.push((x, y));
    }
    samples
}

/// Scott's rule for two dimensions: `σ · n^(-1/6)`, with the sample standard
/// deviation. A line of points has `σ` of 0 on one axis; the bandwidth is
/// then at least one cell of the grid, so the line still grows a thin contour.
fn bandwidth(values: &[f64], span: f64, cells: usize) -> f64 {
    let n = values.len();
    let mean = values.iter().sum::<f64>() / n as f64;
    let variance = values.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / (n - 1) as f64;
    let scott = variance.sqrt() * (n as f64).powf(-1.0 / 6.0);
    scott.max(span / cells as f64)
}

/// Density of `samples` on the `(cells + 1)²` nodes of the grid that spans
/// `x` and `y`. The constant in front of the kernel is left out: the lines
/// are fractions of whatever the highest node is.
fn density_grid(samples: &[(f64, f64)], x: (f64, f64), y: (f64, f64), cells: usize) -> Vec<f64> {
    let nodes = cells + 1;
    let mut grid = vec![0.0; nodes * nodes];
    let xs: Vec<f64> = samples.iter().map(|p| p.0).collect();
    let ys: Vec<f64> = samples.iter().map(|p| p.1).collect();
    let hx = bandwidth(&xs, x.1 - x.0, cells);
    let hy = bandwidth(&ys, y.1 - y.0, cells);
    let dx = (x.1 - x.0) / cells as f64;
    let dy = (y.1 - y.0) / cells as f64;
    let reach_x = KERNEL_REACH * hx;
    let reach_y = KERNEL_REACH * hy;
    for &(px, py) in samples {
        let i0 = node_index(px - reach_x, x.0, dx, cells, true);
        let i1 = node_index(px + reach_x, x.0, dx, cells, false);
        let j0 = node_index(py - reach_y, y.0, dy, cells, true);
        let j1 = node_index(py + reach_y, y.0, dy, cells, false);
        for j in j0..=j1 {
            let gy = y.0 + dy * j as f64;
            let dy2 = ((gy - py) / hy).powi(2);
            if dy2 > KERNEL_REACH * KERNEL_REACH {
                continue;
            }
            for i in i0..=i1 {
                let gx = x.0 + dx * i as f64;
                let dx2 = ((gx - px) / hx).powi(2);
                if dx2 + dy2 > KERNEL_REACH * KERNEL_REACH {
                    continue;
                }
                grid[j * nodes + i] += (-0.5 * (dx2 + dy2)).exp();
            }
        }
    }
    grid
}

/// The node index of `position` along an axis. `low` rounds down, so a
/// point's window includes the nodes on either side of it.
fn node_index(position: f64, origin: f64, step: f64, cells: usize, low: bool) -> usize {
    let t = (position - origin) / step;
    let t = if low { t.floor() } else { t.ceil() };
    if t <= 0.0 {
        0
    } else if t >= cells as f64 {
        cells
    } else {
        t as usize
    }
}

/// Where a line at `threshold` crosses an edge, as a fraction from the
/// first corner to the second. An edge whose ends are equal is crossed
/// in the middle.
fn edge_fraction(start: f64, end: f64, threshold: f64) -> f64 {
    let span = end - start;
    if span.abs() < 1e-15 {
        0.5
    } else {
        ((threshold - start) / span).clamp(0.0, 1.0)
    }
}

/// The two edges a cell of this mask connects. Bits, from the low corner:
/// `1` at `(i, j)`, `2` at `(i+1, j)`, `4` at `(i+1, j+1)`, `8` at `(i, j+1)`.
/// Sides of the pair are `0` south, `1` east, `2` north, `3` west.
/// A saddle (`5` and `10`) joins its two high corners when their average
/// with the low ones is itself over the line.
fn cell_segments(mask: u8, joined: bool) -> &'static [(u8, u8)] {
    match mask {
        0 | 15 => &[],
        1 | 14 => &[(0, 3)],
        2 | 13 => &[(0, 1)],
        3 | 12 => &[(3, 1)],
        4 | 11 => &[(2, 1)],
        5 => {
            if joined {
                &[(0, 1), (3, 2)]
            } else {
                &[(0, 3), (2, 1)]
            }
        }
        6 | 9 => &[(0, 2)],
        7 | 8 => &[(3, 2)],
        10 => {
            if joined {
                &[(0, 3), (2, 1)]
            } else {
                &[(0, 1), (3, 2)]
            }
        }
        _ => &[],
    }
}

/// Index of one side of the cell `(i, j)` on a grid of `cells` cells.
/// Horizontal edges come first, then vertical ones, so an edge shared by
/// two cells has one index.
fn edge_index(cells: usize, i: usize, j: usize, side: u8) -> usize {
    let horizontal = cells * (cells + 1);
    match side {
        0 => j * cells + i,
        1 => horizontal + j * (cells + 1) + i + 1,
        2 => (j + 1) * cells + i,
        _ => horizontal + j * (cells + 1) + i,
    }
}

/// The two nodes of `edge`, each `(column, row)`.
fn edge_nodes(cells: usize, edge: usize) -> ((usize, usize), (usize, usize)) {
    let horizontal = cells * (cells + 1);
    if edge < horizontal {
        let row = edge / cells;
        let column = edge % cells;
        ((column, row), (column + 1, row))
    } else {
        let vertical = edge - horizontal;
        let row = vertical / (cells + 1);
        let column = vertical % (cells + 1);
        ((column, row), (column, row + 1))
    }
}

/// Lines of one threshold. An edge is the pair of grid nodes it joins, so
/// the two cells that share it meet on the same point. A line that reaches
/// the boundary of the grid stays open; one that comes back to its start
/// is closed.
fn trace_threshold(
    grid: &[f64],
    cells: usize,
    threshold: f64,
    x: (f64, f64),
    y: (f64, f64),
) -> Vec<(Vec<(f64, f64)>, bool)> {
    let stride = cells + 1;
    let at = |column: usize, row: usize| grid[row * stride + column];
    let edge_count = 2 * cells * (cells + 1);
    let mut segments: Vec<(usize, usize)> = Vec::new();
    let mut crossing = vec![None; edge_count];
    let dx = (x.1 - x.0) / cells as f64;
    let dy = (y.1 - y.0) / cells as f64;
    let point_on = |edge: usize, fraction: f64| -> (f64, f64) {
        let (a, b) = edge_nodes(cells, edge);
        let column = a.0 as f64 + fraction * (b.0 as f64 - a.0 as f64);
        let row = a.1 as f64 + fraction * (b.1 as f64 - a.1 as f64);
        (x.0 + dx * column, y.0 + dy * row)
    };
    for row in 0..cells {
        for column in 0..cells {
            let (sw, se, ne, nw) = (
                at(column, row),
                at(column + 1, row),
                at(column + 1, row + 1),
                at(column, row + 1),
            );
            let mut mask = 0_u8;
            if sw >= threshold {
                mask |= 1;
            }
            if se >= threshold {
                mask |= 2;
            }
            if ne >= threshold {
                mask |= 4;
            }
            if nw >= threshold {
                mask |= 8;
            }
            let joined = (sw + se + ne + nw) / 4.0 >= threshold;
            for &(side_a, side_b) in cell_segments(mask, joined) {
                let a = edge_index(cells, column, row, side_a);
                let b = edge_index(cells, column, row, side_b);
                if crossing[a].is_none() {
                    let (start, end) = edge_nodes(cells, a);
                    let fraction = edge_fraction(at(start.0, start.1), at(end.0, end.1), threshold);
                    crossing[a] = Some(point_on(a, fraction));
                }
                if crossing[b].is_none() {
                    let (start, end) = edge_nodes(cells, b);
                    let fraction = edge_fraction(at(start.0, start.1), at(end.0, end.1), threshold);
                    crossing[b] = Some(point_on(b, fraction));
                }
                segments.push((a, b));
            }
        }
    }
    chain_edges(&segments, &crossing)
}

/// Walks the segments into polylines. Each edge records the segments that
/// end on it; an edge of the grid boundary has one, an edge between two
/// cells has two.
fn chain_edges(
    segments: &[(usize, usize)],
    crossing: &[Option<(f64, f64)>],
) -> Vec<(Vec<(f64, f64)>, bool)> {
    let mut adjacent: Vec<Vec<usize>> = vec![Vec::new(); crossing.len()];
    for (index, &(a, b)) in segments.iter().enumerate() {
        adjacent[a].push(index);
        adjacent[b].push(index);
    }
    let degrees: Vec<usize> = adjacent.iter().map(Vec::len).collect();
    let mut used = vec![false; segments.len()];
    let step = |current: usize, used: &mut [bool]| -> Option<usize> {
        for &segment in &adjacent[current] {
            if used[segment] {
                continue;
            }
            used[segment] = true;
            let (a, b) = segments[segment];
            return Some(if a == current { b } else { a });
        }
        None
    };
    let walk = |start: usize, closed: bool, used: &mut [bool]| -> Option<Vec<(f64, f64)>> {
        if adjacent[start].iter().all(|&segment| used[segment]) {
            return None;
        }
        let mut edges = vec![start];
        let mut current = start;
        for _ in 0..=segments.len() {
            let Some(next) = step(current, used) else {
                break;
            };
            if next == start {
                break;
            }
            edges.push(next);
            current = next;
        }
        let mut points = Vec::with_capacity(edges.len());
        for edge in edges {
            let Some(point) = crossing[edge] else {
                continue;
            };
            if points.last() == Some(&point) {
                continue;
            }
            points.push(point);
        }
        if closed && points.len() > 1 && points.first() == points.last() {
            points.pop();
        }
        let enough = if closed { 3 } else { 2 };
        (points.len() >= enough).then_some(points)
    };
    let mut lines = Vec::new();
    for (start, degree) in degrees.iter().enumerate() {
        if *degree != 1 {
            continue;
        }
        if let Some(points) = walk(start, false, &mut used) {
            lines.push((points, false));
        }
    }
    for (start, degree) in degrees.iter().enumerate() {
        if *degree != 2 {
            continue;
        }
        if let Some(points) = walk(start, true, &mut used) {
            lines.push((points, true));
        }
    }
    lines
}

/// The contour lines of `samples`, over the rectangle `x` × `y`, in that
/// same coordinate. `levels` lines, the first the lowest.
fn density_lines(
    samples: &[(f64, f64)],
    x: (f64, f64),
    y: (f64, f64),
    cells: usize,
    levels: usize,
) -> Vec<ContourLine> {
    if samples.len() < MIN_CONTOUR_POINTS || cells < 2 || levels == 0 {
        return Vec::new();
    }
    if !(x.1 > x.0 && y.1 > y.0) {
        return Vec::new();
    }
    let grid = density_grid(samples, x, y, cells);
    let peak = grid.iter().copied().fold(0.0_f64, f64::max);
    if peak <= 0.0 {
        return Vec::new();
    }
    let mut lines = Vec::new();
    for level in 1..=levels {
        let threshold = peak * level as f64 / (levels as f64 + 1.0);
        for (points, closed) in trace_threshold(&grid, cells, threshold, x, y) {
            lines.push(ContourLine {
                points,
                closed,
                level,
            });
        }
    }
    lines
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

    /// Histograms of the points, above the plot and to its right. `None`
    /// (the default) draws the scatter chart alone.
    pub marginal: Option<MarginalHistogram>,
    /// Density contours of the points, drawn under them in the color of
    /// each series. `None` (the default) draws the points alone.
    pub contour: Option<DensityContour>,
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
    tip: Option<Text>,
    dataset: Vec<(String, String)>,
) {
    // When a tooltip is present the symbol is the hover trigger; a hidden
    // `.ct-tip` label is drawn right after it (adjacent-sibling reveal).
    let title = tip.as_ref().map(|tip| tip.text.clone());
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
    if let Some(tip) = tip {
        canvas.text_unmeasured(Text {
            class: Some("ct-tip".to_string()),
            x: Some(cx),
            y: Some(cy),
            dy: Some(-8.0),
            text_anchor: Some("middle".to_string()),
            ..tip
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
        if let Some(marginal) = value.get("marginal") {
            let mut histogram = MarginalHistogram::default();
            if let Some(size) = get_f32_from_value(marginal, "size") {
                histogram.size = size;
            }
            if let Some(count) = get_usize_from_value(marginal, "bin_count") {
                histogram.bin_count = count;
            }
            if let Some(gap) = get_f32_from_value(marginal, "bar_gap") {
                histogram.bar_gap = gap;
            }
            s.marginal = Some(histogram);
        }
        if let Some(contour) = value.get("contour") {
            let mut density = DensityContour::default();
            if let Some(levels) = get_usize_from_value(contour, "levels") {
                density.levels = levels;
            }
            if let Some(grid_size) = get_usize_from_value(contour, "grid_size") {
                density.grid_size = grid_size;
            }
            s.contour = Some(density);
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
        if let Some(marginal) = &mut self.marginal {
            if marginal.size <= 0.0 {
                marginal.size = DEFAULT_MARGINAL_SIZE;
            }
            if marginal.bar_gap < 0.0 {
                marginal.bar_gap = 0.0;
            }
        }
        if let Some(contour) = &mut self.contour {
            let (levels, grid_size) = contour.resolved();
            contour.levels = levels;
            contour.grid_size = grid_size;
        }
    }
    /// The band the marginal histograms take off the plot, or 0 when they
    /// are off or the plot would have nothing left to be read in.
    fn marginal_band(&self, plot_width: f32, plot_height: f32) -> f32 {
        let Some(marginal) = &self.marginal else {
            return 0.0;
        };
        // The smaller side of the plot keeps 40 pixels; a band under 8
        // pixels is not a histogram.
        let room = plot_width.min(plot_height) - 40.0;
        if room < 8.0 {
            0.0
        } else {
            marginal.size.min(room)
        }
    }
    /// Draws one marginal histogram into `canvas`, whose origin is the
    /// corner of the band nearest the top-left of the chart. `place` maps a
    /// value to a pixel along the axis the bars are counted on, in that
    /// canvas.
    fn draw_marginal(
        &self,
        canvas: &mut Canvas,
        bins: MarginalBins,
        series: &[MarginalSeries],
        side: MarginalSide,
        panel: f32,
        place: impl Fn(f64) -> f32,
    ) {
        let max = series
            .iter()
            .flat_map(|s| s.counts(side))
            .copied()
            .max()
            .unwrap_or(0);
        if max == 0 {
            return;
        }
        let overlaid = series
            .iter()
            .filter(|s| s.counts(side).iter().any(|n| *n > 0))
            .count()
            > 1;
        let gap = self
            .marginal
            .as_ref()
            .map(|m| m.bar_gap)
            .unwrap_or(1.0)
            .max(0.0);
        // The bars fill the band up to a pixel short of the plot.
        let room = (panel - MARGINAL_PLOT_GAP).max(0.0);
        let horizontal = side == MarginalSide::Y;
        let axis_name = if horizontal { "y" } else { "x" };
        for series in series {
            if series.counts(side).iter().all(|n| *n == 0) {
                continue;
            }
            let mut color = get_color(&self.series.colors, series.index);
            color = color.with_alpha(if overlaid { 140 } else { 210 });
            for (i, &count) in series.counts(side).iter().enumerate() {
                if count == 0 {
                    continue;
                }
                let (from, to) = (bins.edge(i), bins.edge(i + 1));
                let (a, b) = (place(from), place(to));
                let start = a.min(b);
                let span = (a - b).abs();
                let bar_gap = gap.clamp(0.0, span / 4.0);
                let along = start + bar_gap / 2.0;
                let thickness = span - bar_gap;
                if thickness < 0.5 || room <= 0.0 {
                    continue;
                }
                let length = count as f32 / max as f32 * room;
                let (left, top, width, height) = if horizontal {
                    (MARGINAL_PLOT_GAP, along, length, thickness)
                } else {
                    (along, panel - MARGINAL_PLOT_GAP - length, thickness, length)
                };
                let from_text = format_float(from as f32);
                let to_text = format_float(to as f32);
                let tooltip_text = self
                    .tooltip
                    .show
                    .then(|| format!("{}: {from_text} – {to_text}: {count}", series.name));
                let mut classes = vec!["ct-marginal"];
                if tooltip_text.is_some() {
                    classes.push("ct-trigger");
                }
                canvas.rect(Rect {
                    fill: Some(color.into()),
                    left,
                    top,
                    width,
                    height,
                    class: Some(classes.join(" ")),
                    title: tooltip_text.clone(),
                    dataset: vec![
                        ("series".to_string(), series.name.clone()),
                        ("axis".to_string(), axis_name.to_string()),
                        ("from".to_string(), from_text),
                        ("to".to_string(), to_text),
                        ("value".to_string(), count.to_string()),
                    ],
                    ..Default::default()
                });
                if let Some(text) = tooltip_text {
                    canvas.text_unmeasured(Text {
                        text,
                        class: Some("ct-tip".to_string()),
                        font_weight: self.tooltip.font.weight.clone(),
                        font_family: Some(self.font_family.clone()),
                        font_color: Some(self.tooltip_font_color(self.series.label.font.color)),
                        font_size: Some(self.tooltip_font_size(self.series.label.font.size)),
                        x: Some(left + width / 2.0),
                        y: Some(top),
                        dy: Some(-6.0),
                        text_anchor: Some("middle".to_string()),
                        ..Default::default()
                    });
                }
            }
        }
    }
    /// Draws the density contours of every series that has enough points on
    /// the axes. `canvas` is the plot, and the lines use the same placement
    /// as the points.
    fn draw_contours(
        &self,
        canvas: &mut Canvas,
        x_axis: &AxisValues,
        y_axis: &AxisValues,
        width: f32,
        height: f32,
    ) {
        let Some(contour) = &self.contour else {
            return;
        };
        let (Some(x_span), Some(y_span)) = (scale_span(x_axis), scale_span(y_axis)) else {
            return;
        };
        let (levels, cells) = contour.resolved();
        // (series position, series, points on the axes)
        type Prepared<'a> = (usize, &'a Series, Vec<(f64, f64)>);
        let prepared: Vec<Prepared> = self
            .series_list
            .iter()
            .enumerate()
            .filter_map(|(index, series)| {
                let samples = contour_samples(series, self.bubble, x_axis, y_axis);
                (samples.len() >= MIN_CONTOUR_POINTS).then_some((index, series, samples))
            })
            .collect();
        if prepared.is_empty() {
            return;
        }
        // Several series share the plot, so each is drawn lighter.
        let alpha = if prepared.len() > 1 { 140 } else { 210 };
        for (index, series, samples) in prepared {
            let series_idx = series.index.unwrap_or(index);
            let color = get_color(&self.series.colors, series_idx).with_alpha(alpha);
            for line in density_lines(&samples, x_span, y_span, cells, levels) {
                let points = line
                    .points
                    .iter()
                    .map(|(x, y)| {
                        let px = width - x_axis.get_offset_height(from_scale(x_axis, *x), width);
                        let py = y_axis.get_offset_height(from_scale(y_axis, *y), height);
                        (px, py).into()
                    })
                    .collect();
                canvas.straight_line(StraightLine {
                    color: Some(color),
                    points,
                    stroke_width: CONTOUR_STROKE,
                    symbol: None,
                    close: line.closed,
                    class: Some(format!("ct-contour l{}", line.level)),
                    ..Default::default()
                });
            }
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
            // A log axis of no values has no labels to be as wide as.
            let y_axis_formatter = &y_axis_config.formatter.clone().unwrap_or_default();
            let width = y_axis_values.data.first().and_then(|label| {
                let str = format_string(label, y_axis_formatter);
                measure_text_width_family(&self.font_family, y_axis_config.font.size, &str).ok()
            });
            match width {
                Some(b) => b.width() + 5.0,
                None => DEFAULT_Y_AXIS_WIDTH,
            }
        };

        // The histograms take a band above the plot and one to its right.
        // Without them the plot is the size it was.
        let marginal_size = self.marginal_band(
            c.width() - y_axis_width,
            c.height() - x_axis_height - axis_top,
        );
        // Further in, where the plot is to line up with others: the plot
        // without the histogram to the right of it, which stays against it.
        let (y_axis_width, right) = self.fit_plot(&c, y_axis_width, marginal_size);
        let axis_height = c.height() - x_axis_height - axis_top - marginal_size;
        let axis_width = c.width() - y_axis_width - right;
        self.render_axis_titles(
            &titles,
            &self.y_axis_configs,
            &self.x_axis.title,
            y_axis_width,
            axis_top + marginal_size,
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
                top: marginal_size,
                ..Default::default()
            }),
            &self.y_axis_configs,
            axis_width,
            axis_height,
        );
        c.child(Box {
            left: y_axis_width,
            top: marginal_size,
            ..Default::default()
        })
        .grid(Grid {
            right: axis_width,
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
                c.child(Box {
                    top: marginal_size,
                    ..Default::default()
                }),
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
        let content_width = axis_width;
        let content_height = axis_height;
        let bin_count = self.marginal.as_ref().map(|m| m.bin_count).unwrap_or(0);
        let (x_bins, y_bins) = if marginal_size > 0.0 {
            let largest = self
                .series_list
                .iter()
                .map(|series| plotted_point_count(series, self.bubble))
                .max()
                .unwrap_or(0);
            (
                marginal_bins(x_axis_values.min, x_axis_values.max, largest, bin_count),
                marginal_bins(y_axis_values.min, y_axis_values.max, largest, bin_count),
            )
        } else {
            (None, None)
        };
        let mut marginal_series = Vec::new();
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
            top: marginal_size,
            ..Default::default()
        });
        // The contours go down first, so the points are drawn over them.
        self.draw_contours(
            &mut content_canvas,
            &x_axis_values,
            &y_axis_values,
            content_width,
            content_height,
        );
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
            if marginal_size > 0.0 {
                marginal_series.push(MarginalSeries {
                    index: series_idx,
                    name: series.name.clone(),
                    x: x_bins
                        .map(|bins| bin_counts(bins, points.iter().map(|p| p.0)))
                        .unwrap_or_default(),
                    y: y_bins
                        .map(|bins| bin_counts(bins, points.iter().map(|p| p.1)))
                        .unwrap_or_default(),
                });
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
                // Written as the tooltips of the other charts are: in the
                // font of the chart, as large as the labels and in their
                // color where the tooltips have neither of their own.
                let tip = self.tooltip.show.then(|| Text {
                    text: match bubble_size {
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
                    },
                    font_family: Some(self.font_family.clone()),
                    font_size: Some(self.tooltip_font_size(self.series.label.font.size)),
                    font_color: Some(self.tooltip_font_color(self.series.label.font.color)),
                    font_weight: self.tooltip.font.weight.clone(),
                    ..Default::default()
                });
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
                    tip,
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

        if marginal_size > 0.0 {
            if let Some(bins) = x_bins {
                let mut panel = c.child(Box {
                    left: y_axis_width,
                    ..Default::default()
                });
                self.draw_marginal(
                    &mut panel,
                    bins,
                    &marginal_series,
                    MarginalSide::X,
                    marginal_size,
                    |value| {
                        content_width - x_axis_values.get_offset_height(value as f32, content_width)
                    },
                );
            }
            if let Some(bins) = y_bins {
                let mut panel = c.child(Box {
                    left: y_axis_width + content_width,
                    top: marginal_size,
                    ..Default::default()
                });
                self.draw_marginal(
                    &mut panel,
                    bins,
                    &marginal_series,
                    MarginalSide::Y,
                    marginal_size,
                    |value| y_axis_values.get_offset_height(value as f32, content_height),
                );
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

    #[test]
    fn marginal_bins_span_the_axis() {
        // Four equal bins from the axis minimum to its maximum, the last
        // edge included and a value outside the axis left out.
        let bins = super::marginal_bins(0.0, 4.0, 100, 4).unwrap();
        assert_eq!((0.0, 1.0, 4), (bins.start, bins.width, bins.count));
        assert_eq!(Some(0), bins.index(0.0));
        assert_eq!(Some(0), bins.index(0.999));
        assert_eq!(Some(1), bins.index(1.0));
        assert_eq!(Some(3), bins.index(4.0));
        assert_eq!(None, bins.index(-0.1));
        assert_eq!(None, bins.index(4.1));
        // No range to divide: the points all sit on one value.
        assert_eq!(None, super::marginal_bins(2.0, 2.0, 10, 4));
        // Asked for too many, and asked for none: capped, or derived.
        assert_eq!(40, super::marginal_bins(0.0, 10.0, 5, 1000).unwrap().count);
        // 20 points → Sturges asks for 6.
        assert_eq!(6, super::marginal_bins(0.0, 12.0, 20, 0).unwrap().count);
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

    fn near(found: (f64, f64), wanted: (f64, f64)) -> bool {
        (found.0 - wanted.0).abs() < 1e-6 && (found.1 - wanted.1).abs() < 1e-6
    }

    fn has_point(points: &[(f64, f64)], wanted: (f64, f64)) -> bool {
        points.iter().any(|point| near(*point, wanted))
    }

    /// Even-odd ray cast. A point on the boundary is not what the tests ask.
    fn encloses(points: &[(f64, f64)], x: f64, y: f64) -> bool {
        let mut crossings = 0;
        for index in 0..points.len() {
            let (x1, y1) = points[index];
            let (x2, y2) = points[(index + 1) % points.len()];
            if (y1 > y) != (y2 > y) {
                let at = x1 + (y - y1) * (x2 - x1) / (y2 - y1);
                if x < at {
                    crossings += 1;
                }
            }
        }
        crossings % 2 == 1
    }

    #[test]
    fn marching_squares_rings_a_peak_and_skips_a_flat_cell() {
        // A single high node in the middle of a 3×3 grid. The four cells
        // around it share their edges and come back to one diamond.
        let mut grid = vec![0.0; 9];
        grid[4] = 1.0;
        let lines = super::trace_threshold(&grid, 2, 0.5, (0.0, 2.0), (0.0, 2.0));
        assert_eq!(1, lines.len(), "{lines:?}");
        assert!(lines[0].1);
        for point in [(1.0, 0.5), (0.5, 1.0), (1.0, 1.5), (1.5, 1.0)] {
            assert!(
                has_point(&lines[0].0, point),
                "{point:?} missing from {lines:?}"
            );
        }
        assert!(encloses(&lines[0].0, 1.0, 1.0));
        assert!(!encloses(&lines[0].0, 0.0, 0.0));

        // Over the peak there is nothing to cross, and a cell that is high
        // in every corner has no edge to cross either.
        assert!(super::trace_threshold(&grid, 2, 2.0, (0.0, 2.0), (0.0, 2.0)).is_empty());
        let flat = vec![1.0; 9];
        assert!(super::trace_threshold(&flat, 2, 0.5, (0.0, 2.0), (0.0, 2.0)).is_empty());
    }

    #[test]
    fn marching_squares_splits_a_saddle_by_its_middle() {
        // The two high corners are diagonal. The middle of the cell is the
        // average of the four corners.
        //   0 -- 1
        //   |    |
        //   1 -- 0
        let grid = vec![1.0, 0.0, 0.0, 1.0];
        let joined = super::trace_threshold(&grid, 1, 0.5, (0.0, 1.0), (0.0, 1.0));
        assert_eq!(2, joined.len(), "{joined:?}");
        assert!(joined.iter().all(|line| !line.1));
        assert!(
            joined
                .iter()
                .any(|line| has_point(&line.0, (0.5, 0.0)) && has_point(&line.0, (1.0, 0.5)))
        );
        assert!(
            joined
                .iter()
                .any(|line| has_point(&line.0, (0.0, 0.5)) && has_point(&line.0, (0.5, 1.0)))
        );

        // A higher line leaves the two high corners separate.
        let separate = super::trace_threshold(&grid, 1, 0.6, (0.0, 1.0), (0.0, 1.0));
        assert_eq!(2, separate.len(), "{separate:?}");
        assert!(
            separate
                .iter()
                .any(|line| has_point(&line.0, (0.4, 0.0)) && has_point(&line.0, (0.0, 0.4)))
        );
        assert!(
            separate
                .iter()
                .any(|line| has_point(&line.0, (0.6, 1.0)) && has_point(&line.0, (1.0, 0.6)))
        );
    }

    fn bounds_of(lines: &[super::ContourLine], level: usize) -> (f64, f64, f64, f64) {
        let mut bounds = (f64::MAX, f64::MAX, f64::MIN, f64::MIN);
        for line in lines
            .iter()
            .filter(|line| line.level == level && line.closed)
        {
            for (x, y) in &line.points {
                bounds.0 = bounds.0.min(*x);
                bounds.1 = bounds.1.min(*y);
                bounds.2 = bounds.2.max(*x);
                bounds.3 = bounds.3.max(*y);
            }
        }
        bounds
    }

    #[test]
    fn density_lines_ring_a_cluster_and_nest() {
        let samples: Vec<(f64, f64)> = (0..24)
            .map(|i| {
                let t = i as f64;
                (0.12 * (t * 1.3).sin(), 0.12 * (t * 0.7).cos())
            })
            .collect();
        let lines = super::density_lines(&samples, (-8.0, 8.0), (-8.0, 8.0), 32, 4);
        let outer: Vec<_> = lines
            .iter()
            .filter(|line| line.level == 1 && line.closed)
            .collect();
        assert_eq!(
            1,
            outer.len(),
            "levels: {:?}",
            lines.iter().map(|l| l.level).collect::<Vec<_>>()
        );
        for (x, y) in &samples {
            assert!(
                encloses(&outer[0].points, *x, *y),
                "({x}, {y}) is outside the outer line"
            );
        }
        assert!(!encloses(&outer[0].points, -8.0, -8.0));
        let (outer_box, inner_box) = (bounds_of(&lines, 1), bounds_of(&lines, 4));
        assert!(outer_box.0 < inner_box.0 && inner_box.2 < outer_box.2);
        assert!(outer_box.1 < inner_box.1 && inner_box.3 < outer_box.3);
    }

    #[test]
    fn density_lines_split_two_clusters_at_the_high_line() {
        let mut samples = Vec::new();
        for i in 0..20 {
            let t = i as f64 * 0.2;
            samples.push((-6.0 + 0.05 * t.sin(), 0.05 * t.cos()));
            samples.push((6.0 + 0.05 * t.sin(), 0.05 * t.cos()));
        }
        let lines = super::density_lines(&samples, (-12.0, 12.0), (-8.0, 8.0), 48, 4);
        let high: Vec<_> = lines
            .iter()
            .filter(|line| line.level == 4 && line.closed)
            .collect();
        assert_eq!(2, high.len(), "{high:?}");
        assert!(high.iter().any(|line| encloses(&line.points, -6.0, 0.0)));
        assert!(high.iter().any(|line| encloses(&line.points, 6.0, 0.0)));
        assert!(high.iter().all(|line| !encloses(&line.points, 0.0, 0.0)));
    }

    #[test]
    fn density_lines_of_a_straight_row_stay_finite() {
        // A short row in the middle of the axes: the contour stays off the
        // boundary, where it would be an open line.
        let samples: Vec<(f64, f64)> = (0..16).map(|i| (i as f64 * 0.2 - 1.5, 0.0)).collect();
        let lines = super::density_lines(&samples, (-8.0, 8.0), (-8.0, 8.0), 16, 4);
        assert!(!lines.is_empty());
        for line in &lines {
            assert!(
                line.points
                    .iter()
                    .all(|p| p.0.is_finite() && p.1.is_finite())
            );
        }
        let outer = lines
            .iter()
            .find(|line| line.level == 1 && line.closed)
            .unwrap();
        assert!(encloses(&outer.points, 0.0, 0.0));
    }
}
