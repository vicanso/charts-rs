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

use super::base::ChartBase;
use super::canvas;
use super::color::*;
use super::common::*;
use super::component::*;
use super::map_chart::{MapProjection, MapRegion};
use super::params::*;
use super::theme::{get_default_theme_name, get_theme};
use super::util::*;

/// One flow, from one point to another. On a plane the points are `(x, y)`,
/// with y growing upwards. On a map they are `(longitude, latitude)` in
/// degrees. `value` is the flow: it sets the width of the line.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct FlowLine {
    /// Where the flow starts.
    pub from: (f64, f64),
    /// Where the flow ends. The arrow, when drawn, points here.
    pub to: (f64, f64),
    /// The size of the flow. It is mapped from the smallest value to
    /// `min_width` and from the largest to `max_width`.
    pub value: f32,
    /// Name of the flow, written as the tooltip when tooltips are on.
    pub name: String,
}

impl From<((f64, f64), (f64, f64), f32)> for FlowLine {
    fn from((from, to, value): ((f64, f64), (f64, f64), f32)) -> Self {
        FlowLine {
            from,
            to,
            value,
            name: String::new(),
        }
    }
}

/// Flows drawn as lines from one point to another: straight, or bent into
/// an arc. The width of a line is its value.
///
/// Without regions the points are on a plane. With `regions` (usually read
/// from GeoJSON) the points are longitudes and latitudes, projected the same
/// way as a [`MapChart`](super::MapChart), and the regions are drawn under
/// the lines.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct LinesChart {
    /// The shared chart options (size, series, title/legend, axes); exposed
    /// directly on the chart through `Deref`, e.g. `chart.title.text`.
    pub base: ChartBase,
    y_axis_configs: Vec<YAxisConfig>,

    /// The flows.
    pub lines: Vec<FlowLine>,
    /// How far a line bends, as a fraction of its length, to the left of
    /// the direction it travels. `0` is straight. The default.
    pub curveness: f32,
    /// Width of the smallest value. `0` means 1.
    pub min_width: f32,
    /// Width of the largest value. `0` means 6.
    pub max_width: f32,
    /// Draws an arrow at the end of each line. The default, set by
    /// [`new`](LinesChart::new) and [`from_json`](LinesChart::from_json).
    pub arrow: bool,
    /// The regions under the lines, when the points are on a map.
    pub regions: Vec<MapRegion>,
    /// How longitudes and latitudes are laid on the plane. Ignored when
    /// there are no regions: the points are already on a plane.
    pub projection: MapProjection,
}

impl LinesChart {
    /// Creates a lines chart on the plane.
    pub fn new(lines: Vec<FlowLine>) -> LinesChart {
        LinesChart::new_with_theme(lines, &get_default_theme_name())
    }
    /// Creates a lines chart with custom theme.
    pub fn new_with_theme(lines: Vec<FlowLine>, theme: &str) -> LinesChart {
        let mut c = LinesChart {
            lines,
            arrow: true,
            ..Default::default()
        };
        let theme = get_theme(theme);
        c.base.fill_theme(theme, &mut c.y_axis_configs);
        c
    }
    /// Creates a lines chart from JSON.
    pub fn from_json(data: &str) -> canvas::Result<LinesChart> {
        let mut c = LinesChart {
            arrow: true,
            ..Default::default()
        };
        let data = c
            .base
            .fill_option(data, &mut c.y_axis_configs, super::schema::LINES_FIELDS)?;
        if let Some(lines) = data.get("lines").and_then(|v| v.as_array()) {
            c.lines = lines.iter().filter_map(flow_line_from_value).collect();
        }
        if let Some(curveness) = get_f32_from_value(&data, "curveness") {
            c.curveness = curveness;
        }
        if let Some(width) = get_f32_from_value(&data, "min_width") {
            c.min_width = width;
        }
        if let Some(width) = get_f32_from_value(&data, "max_width") {
            c.max_width = width;
        }
        if let Some(arrow) = get_bool_from_value(&data, "arrow") {
            c.arrow = arrow;
        }
        if get_string_from_value(&data, "projection")
            .is_some_and(|projection| projection.eq_ignore_ascii_case("equirectangular"))
        {
            c.projection = MapProjection::Equirectangular;
        }
        if let Some(geo_json) = data.get("geo_json") {
            let name =
                get_string_from_value(&data, "name_property").unwrap_or_else(|| "name".into());
            c.regions = super::map_chart::regions_from_value(geo_json, &name);
        }
        Ok(c)
    }

    /// Renders the chart to an SVG string.
    pub fn svg(&self) -> canvas::Result<String> {
        let mut c = self.new_canvas();
        let axis_top = self.render_header(&mut c);
        let mut content = c.child(Box {
            top: axis_top,
            ..Default::default()
        });
        if content.width() <= 0.0 || content.height() <= 0.0 {
            return c.svg();
        }
        let on_map = !self.regions.is_empty();
        let project = |point: (f64, f64)| -> (f64, f64) {
            if on_map {
                project_geo(self.projection, point)
            } else {
                (point.0, -point.1)
            }
        };
        let mut bounds: Vec<(f64, f64)> = self
            .lines
            .iter()
            .flat_map(|line| [line.from, line.to])
            .filter(|p| p.0.is_finite() && p.1.is_finite())
            .map(project)
            .collect();
        for region in &self.regions {
            for polygon in &region.polygons {
                for ring in polygon {
                    bounds.extend(ring.iter().copied().map(project));
                }
            }
        }
        if bounds.is_empty() {
            self.render_empty_text(content);
            return c.svg();
        }
        let (mut left, mut top, mut right, mut bottom) = (
            f64::INFINITY,
            f64::INFINITY,
            f64::NEG_INFINITY,
            f64::NEG_INFINITY,
        );
        for (x, y) in &bounds {
            left = left.min(*x);
            right = right.max(*x);
            top = top.min(*y);
            bottom = bottom.max(*y);
        }
        let pad_x = (right - left).max(1.0) * 0.08;
        let pad_y = (bottom - top).max(1.0) * 0.08;
        left -= pad_x;
        right += pad_x;
        top -= pad_y;
        bottom += pad_y;
        let (plot_w, plot_h) = (content.width() as f64, content.height() as f64);
        let scale = (plot_w / (right - left).max(1e-9)).min(plot_h / (bottom - top).max(1e-9));
        let offset_x = (plot_w - (right - left) * scale) / 2.0;
        let offset_y = (plot_h - (bottom - top) * scale) / 2.0;
        let place = |point: (f64, f64)| -> Point {
            let (x, y) = project(point);
            Point {
                x: (offset_x + (x - left) * scale) as f32,
                y: (offset_y + (y - top) * scale) as f32,
            }
        };

        let region_fill = self.grid.stroke_color.with_alpha(36);
        for region in &self.regions {
            for polygon in &region.polygons {
                let rings: Vec<Vec<Point>> = polygon
                    .iter()
                    .map(|ring| ring.iter().copied().map(place).collect())
                    .filter(|ring: &Vec<Point>| ring.len() >= 3)
                    .collect();
                if rings.is_empty() {
                    continue;
                }
                content.shape(Shape {
                    color: Some(self.grid.stroke_color),
                    stroke_width: 1.0,
                    fill: Some(region_fill),
                    rings,
                    ..Default::default()
                });
            }
        }

        let (min_w, max_w) = widths(self.min_width, self.max_width);
        let values: Vec<f32> = self
            .lines
            .iter()
            .map(|line| line.value)
            .filter(|v| v.is_finite())
            .collect();
        let (lo, hi) = values
            .iter()
            .fold((f32::MAX, f32::MIN), |(a, b), v| (a.min(*v), b.max(*v)));
        let span = if lo <= hi { hi - lo } else { 0.0 };
        let width_of = |value: f32| -> f32 {
            // One value, or none that differ: there is no range to scale, so
            // the line takes the full width.
            if !value.is_finite() || span <= f32::EPSILON {
                return max_w;
            }
            let t = ((value - lo) / span).clamp(0.0, 1.0);
            min_w + (max_w - min_w) * t
        };
        let curveness = if self.curveness.is_finite() {
            self.curveness.clamp(-1.0, 1.0)
        } else {
            0.0
        };
        let color = get_color(&self.series.colors, 0).with_alpha(176);
        for line in &self.lines {
            if !line.from.0.is_finite()
                || !line.from.1.is_finite()
                || !line.to.0.is_finite()
                || !line.to.1.is_finite()
            {
                continue;
            }
            let from = place(line.from);
            let to = place(line.to);
            let mut points = bend(from, to, curveness);
            if points.len() < 2 {
                continue;
            }
            let width = width_of(line.value);
            let tip = *points.last().unwrap();
            let before = points[points.len() - 2];
            let direction = (tip.x - before.x, tip.y - before.y);
            let title = (!line.name.is_empty())
                .then(|| format!("{}: {}", line.name, format_float(line.value)));
            if self.arrow {
                // Stop the stroke short of the tip, under the arrow.
                let size = (width * 2.4).max(7.0);
                shorten(&mut points, size * 0.55);
                content.polygon(Polygon {
                    fill: Some(color),
                    points: arrowhead(tip, direction, size),
                    title: title.clone(),
                    ..Default::default()
                });
            }
            content.polyline(Polyline {
                color: Some(color),
                stroke_width: width,
                points,
            });
        }
        c.svg()
    }
}

impl std::ops::Deref for LinesChart {
    type Target = ChartBase;
    fn deref(&self) -> &ChartBase {
        &self.base
    }
}
impl std::ops::DerefMut for LinesChart {
    fn deref_mut(&mut self) -> &mut ChartBase {
        &mut self.base
    }
}

fn flow_line_from_value(value: &serde_json::Value) -> Option<FlowLine> {
    let pair = |value: &serde_json::Value| -> Option<(f64, f64)> {
        let point = value.as_array()?;
        let x = point.first()?.as_f64()?;
        let y = point.get(1)?.as_f64()?;
        (x.is_finite() && y.is_finite()).then_some((x, y))
    };
    let (from, to) = if let Some(coords) = value.get("coords").and_then(|c| c.as_array()) {
        (pair(coords.first()?)?, pair(coords.get(1)?)?)
    } else {
        (pair(value.get("from")?)?, pair(value.get("to")?)?)
    };
    Some(FlowLine {
        from,
        to,
        value: get_f32_from_value(value, "value").unwrap_or(1.0),
        name: get_string_from_value(value, "name").unwrap_or_default(),
    })
}

fn widths(min_width: f32, max_width: f32) -> (f32, f32) {
    let mut min_w = if min_width > 0.0 { min_width } else { 1.0 };
    let mut max_w = if max_width > 0.0 { max_width } else { 6.0 };
    if max_w < min_w {
        std::mem::swap(&mut min_w, &mut max_w);
    }
    (min_w, max_w)
}

/// Longitude and latitude on the plane, y growing downwards. The same
/// projection as [`MapChart`](super::MapChart).
fn project_geo(projection: MapProjection, (lon, lat): (f64, f64)) -> (f64, f64) {
    match projection {
        MapProjection::Equirectangular => (lon, -lat),
        MapProjection::Mercator => {
            let lat = lat.clamp(-85.0, 85.0).to_radians();
            let y = (std::f64::consts::FRAC_PI_4 + lat / 2.0).tan().ln();
            (lon, -y.to_degrees())
        }
    }
}

/// The line from `from` to `to`, bent by `curveness`. Straight when that
/// is 0. Twenty-four steps are enough that the arc reads as a curve.
fn bend(from: Point, to: Point, curveness: f32) -> Vec<Point> {
    let dx = to.x - from.x;
    let dy = to.y - from.y;
    let len = (dx * dx + dy * dy).sqrt();
    if len < 0.5 {
        return vec![];
    }
    if curveness.abs() < 1e-3 {
        return vec![from, to];
    }
    let (nx, ny) = (-dy / len, dx / len);
    let (cx, cy) = (
        (from.x + to.x) / 2.0 + nx * curveness * len,
        (from.y + to.y) / 2.0 + ny * curveness * len,
    );
    const STEPS: usize = 24;
    (0..=STEPS)
        .map(|i| {
            let t = i as f32 / STEPS as f32;
            let u = 1.0 - t;
            Point {
                x: u * u * from.x + 2.0 * u * t * cx + t * t * to.x,
                y: u * u * from.y + 2.0 * u * t * cy + t * t * to.y,
            }
        })
        .collect()
}

/// Pulls the last point back along the line by `by` pixels, so the stroke
/// ends under the arrow instead of past its tip.
fn shorten(points: &mut [Point], by: f32) {
    let Some(tip) = points.last().copied() else {
        return;
    };
    let Some(prev) = points.iter().rev().nth(1).copied() else {
        return;
    };
    let (dx, dy) = (tip.x - prev.x, tip.y - prev.y);
    let len = (dx * dx + dy * dy).sqrt();
    if len < 0.5 {
        return;
    }
    let step = by.min(len * 0.8);
    if let Some(last) = points.last_mut() {
        last.x -= dx / len * step;
        last.y -= dy / len * step;
    }
}

fn arrowhead(tip: Point, direction: (f32, f32), size: f32) -> Vec<Point> {
    let len = (direction.0 * direction.0 + direction.1 * direction.1)
        .sqrt()
        .max(1e-3);
    let (dx, dy) = (direction.0 / len, direction.1 / len);
    let (px, py) = (-dy, dx);
    let wing = size * 0.42;
    vec![
        tip,
        Point {
            x: tip.x - dx * size + px * wing,
            y: tip.y - dy * size + py * wing,
        },
        Point {
            x: tip.x - dx * size - px * wing,
            y: tip.y - dy * size - py * wing,
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::LinesChart;

    #[test]
    fn plane_width_follows_value_and_a_curve_bends() {
        let straight = LinesChart::from_json(
            r#"{
                "arrow": false,
                "lines": [
                    {"from": [0, 0], "to": [10, 0], "value": 1},
                    {"from": [0, 2], "to": [10, 2], "value": 5}
                ]
            }"#,
        )
        .unwrap();
        let svg = straight.svg().unwrap();
        let widths: Vec<f32> = svg
            .split("<polyline ")
            .skip(1)
            .map(|tag| {
                let key = "stroke-width=\"";
                let start = tag.find(key).unwrap() + key.len();
                let end = start + tag[start..].find('"').unwrap();
                tag[start..end].parse().unwrap()
            })
            .collect();
        assert_eq!(vec![1.0, 6.0], widths, "{svg}");
        assert_eq!(2, svg.matches("<polyline ").count());
        let points = |svg: &str| {
            svg.split("<polyline ")
                .nth(1)
                .unwrap()
                .split("points=\"")
                .nth(1)
                .unwrap()
                .split('"')
                .next()
                .unwrap()
                .to_string()
        };
        let first = points(&svg);
        assert_eq!(1, first.matches(' ').count(), "{first}");

        let curved = LinesChart::from_json(
            r#"{"curveness": 0.4, "lines": [{"from": [0, 0], "to": [10, 0], "value": 3}]}"#,
        )
        .unwrap();
        let curved = curved.svg().unwrap();
        let points = points(&curved);
        assert!(points.matches(' ').count() > 8, "{points}");
        assert!(curved.contains("<polygon "), "{curved}");
    }

    #[test]
    fn map_lines_sit_on_the_regions() {
        let chart = LinesChart::from_json(
            r#"{
                "projection": "equirectangular",
                "curveness": 0,
                "lines": [{"coords": [[0.2, 0.2], [0.8, 0.8]], "value": 4, "name": "across"}],
                "geo_json": {
                    "type": "Feature",
                    "properties": {"name": "square"},
                    "geometry": {"type": "Polygon", "coordinates": [[[0, 0], [1, 0], [1, 1], [0, 1], [0, 0]]]}
                }
            }"#,
        )
        .unwrap();
        assert_eq!(1, chart.regions.len());
        let svg = chart.svg().unwrap();
        assert!(svg.contains("<path "), "{svg}");
        assert!(svg.contains("<polyline "), "{svg}");
        assert!(svg.contains("across: 4"), "{svg}");
        assert_snapshot!("lines_chart/map.svg", svg);
    }

    #[test]
    fn migration_flows_over_the_sample_country() {
        let chart =
            LinesChart::from_json(include_str!("../../asset/lines_chart/flows.json")).unwrap();
        assert_eq!(11, chart.regions.len());
        assert_eq!(16, chart.lines.len());
        let svg = chart.svg().unwrap();
        assert_eq!(16, svg.matches("<polyline ").count(), "{svg}");
        assert!(svg.contains("Migration between regions"), "{svg}");
        assert!(svg.contains("Cedar → Glen: 220"), "{svg}");
        assert_snapshot!("lines_chart/flows.svg", svg);
        #[cfg(feature = "raster")]
        super::super::save_test_asset("image/lines.png", &crate::svg_to_png(&svg).unwrap());
    }
}
