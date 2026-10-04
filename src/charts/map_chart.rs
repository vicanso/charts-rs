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

use super::base::{ChartBase, LabelBoxes};
use super::canvas;
use super::color::*;
use super::common::*;
use super::component::*;
use super::params::*;
use super::theme::{get_default_theme_name, get_theme};
use super::util::*;
use crate::charts::measure_text_width_family;
use serde::{Deserialize, Serialize};

/// Latitude beyond which a Mercator map is cut off: the poles are
/// infinitely far away on it.
const MERCATOR_LIMIT: f64 = 85.0;
/// Corners of an outline closer to each other than this, in pixels, are
/// drawn as one: a map does not show more than its size allows.
const MIN_STEP: f32 = 0.4;
/// Width and height of the bar of a continuous scale.
const SCALE_BAR: (f32, f32) = (12.0, 110.0);
/// Number of pieces the bar of a continuous scale is drawn in.
const SCALE_PIECES: usize = 22;
/// Side of the swatch of a class of the scale.
const SWATCH: f32 = 14.0;
/// Gap between the scale and the map.
const SCALE_GAP: f32 = 12.0;

// ── Public data model ──────────────────────────────────────────────────────────

/// One region of a map: its name, and its outline.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct MapRegion {
    /// Name of the region; the values of the chart refer to it.
    pub name: String,
    /// The parts of the region (a mainland and its islands). Each part is a
    /// list of rings, the first one its outline and the others holes in it;
    /// a ring is a list of `(longitude, latitude)` corners in degrees.
    pub polygons: Vec<Vec<Vec<(f64, f64)>>>,
}

impl MapRegion {
    /// Reads the regions of a GeoJSON document: a `FeatureCollection`, or a
    /// single `Feature`, of `Polygon` and `MultiPolygon` geometries. The
    /// name of a region is its property `name_property` (usually `"name"`).
    pub fn from_geo_json(geo_json: &str, name_property: &str) -> canvas::Result<Vec<MapRegion>> {
        let value: serde_json::Value = serde_json::from_str(geo_json)?;
        Ok(regions_from_value(&value, name_property))
    }
}

/// How the longitudes and latitudes of a map are laid on the plane.
#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Eq, Debug, Default)]
pub enum MapProjection {
    /// Mercator: shapes are kept, areas grow towards the poles.
    #[default]
    Mercator,
    /// Longitude and latitude as they are, as x and y.
    Equirectangular,
}

/// The corners of one closed outline.
type Ring = Vec<(f64, f64)>;

/// A ring of GeoJSON coordinates: `[[lon, lat], …]`.
fn ring_from_value(value: &serde_json::Value) -> Ring {
    value
        .as_array()
        .map(|points| {
            points
                .iter()
                .filter_map(|point| {
                    let point = point.as_array()?;
                    let (lon, lat) = (point.first()?.as_f64()?, point.get(1)?.as_f64()?);
                    (lon.is_finite() && lat.is_finite()).then_some((lon, lat))
                })
                .collect()
        })
        .unwrap_or_default()
}

/// The regions of a GeoJSON document that is already parsed.
fn regions_from_value(value: &serde_json::Value, name_property: &str) -> Vec<MapRegion> {
    let features: Vec<&serde_json::Value> = match value.get("features").and_then(|f| f.as_array()) {
        Some(features) => features.iter().collect(),
        // A single feature is a map of one region.
        None => vec![value],
    };
    features
        .into_iter()
        .filter_map(|feature| {
            let geometry = feature.get("geometry")?;
            let coordinates = geometry.get("coordinates")?.as_array()?;
            let polygon = |rings: &serde_json::Value| -> Vec<Vec<(f64, f64)>> {
                rings
                    .as_array()
                    .map(|rings| rings.iter().map(ring_from_value).collect())
                    .unwrap_or_default()
            };
            let polygons: Vec<Vec<Vec<(f64, f64)>>> =
                match geometry.get("type").and_then(|t| t.as_str())? {
                    "Polygon" => vec![polygon(&serde_json::Value::Array(coordinates.clone()))],
                    "MultiPolygon" => coordinates.iter().map(polygon).collect(),
                    _ => return None,
                };
            let name = feature
                .get("properties")
                .and_then(|p| p.get(name_property))
                .or_else(|| feature.get("id"))
                .map(|name| match name.as_str() {
                    Some(text) => text.to_string(),
                    None => name.to_string(),
                })
                .unwrap_or_default();
            Some(MapRegion { name, polygons })
        })
        .collect()
}

// ── MapChart ───────────────────────────────────────────────────────────────────

/// A map of regions colored by their values (a choropleth).
///
/// The regions come from the caller — usually as GeoJSON — and the chart
/// has no map data of its own. A region takes the color of its value on
/// the scale of the chart; one without a value is drawn in `empty_color`.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct MapChart {
    /// The shared chart options (size, series, title/legend, axes); exposed
    /// directly on the chart through `Deref`, e.g. `chart.title_text`.
    pub base: ChartBase,
    y_axis_configs: Vec<YAxisConfig>,

    // map-specific
    /// The regions of the map.
    pub regions: Vec<MapRegion>,
    /// The values: the name of a region, and its value.
    pub data: Vec<(String, f32)>,
    /// How longitudes and latitudes are laid on the plane.
    pub projection: MapProjection,
    /// Value at the start of the scale. With `min` and `max` both 0 the
    /// scale goes from the smallest to the largest value.
    pub min: f32,
    /// Value at the end of the scale.
    pub max: f32,
    /// Color of the smallest value. Default: a light tint of the first
    /// color of the theme.
    pub min_color: Color,
    /// Color of the largest value. Default: the first color of the theme.
    pub max_color: Color,
    /// The colors of the scale, from the smallest value to the largest.
    /// Two or more take the place of `min_color` and `max_color`.
    pub colors: Vec<Color>,
    /// Number of classes of the same width the values are sorted into, each
    /// in one color, instead of a continuous scale.
    pub steps: usize,
    /// The values where one class ends and the next begins; takes the place
    /// of `steps`.
    pub thresholds: Vec<f32>,
    /// Color of the regions without a value.
    pub empty_color: Color,
    /// Color of the borders of the regions. Default: the background color.
    pub border_color: Color,
    /// Width of the borders of the regions. Default: 1.0.
    pub border_width: f32,
    /// Writes the name of every region on it; a name that would run into
    /// another one is left out.
    pub label_show: bool,
    /// Shows the scale of the colors beside the map. Default: true.
    pub visual_map_show: Option<bool>,
}

impl std::ops::Deref for MapChart {
    type Target = ChartBase;
    fn deref(&self) -> &ChartBase {
        &self.base
    }
}
impl std::ops::DerefMut for MapChart {
    fn deref_mut(&mut self) -> &mut ChartBase {
        &mut self.base
    }
}

/// The corners of a ring on the plot, and what it encloses.
struct Outline {
    points: Vec<Point>,
    /// Twice the signed area of the ring.
    area: f32,
    /// The point its area balances on.
    center: Point,
}

impl MapChart {
    fn fill_default(&mut self) {
        let first = get_color(&self.series_colors, 0);
        let background = self.background_color;
        let tint = |a: u8, b: u8| (b as f32 + (a as f32 - b as f32) * 0.15).round() as u8;
        self.max_color = first;
        self.min_color = Color {
            r: tint(first.r, background.r),
            g: tint(first.g, background.g),
            b: tint(first.b, background.b),
            a: 255,
        };
        self.empty_color = if self.is_light {
            (235, 237, 240).into()
        } else {
            (60, 63, 70).into()
        };
        self.border_color = background;
        self.border_width = 1.0;
    }

    /// Creates a map chart with the default theme.
    pub fn new(regions: Vec<MapRegion>, data: Vec<(String, f32)>) -> MapChart {
        MapChart::new_with_theme(regions, data, &get_default_theme_name())
    }

    /// Creates a map chart with a custom theme.
    pub fn new_with_theme(
        regions: Vec<MapRegion>,
        data: Vec<(String, f32)>,
        theme: &str,
    ) -> MapChart {
        let mut c = MapChart {
            regions,
            data,
            ..Default::default()
        };
        c.base.fill_theme(get_theme(theme), &mut c.y_axis_configs);
        c.fill_default();
        c
    }

    /// Creates a map chart from a JSON string. The regions are the GeoJSON
    /// document under `geo_json`.
    pub fn from_json(json: &str) -> canvas::Result<MapChart> {
        let mut c = MapChart {
            ..Default::default()
        };
        let value = c
            .base
            .fill_option(json, &mut c.y_axis_configs, super::schema::MAP_FIELDS)?;
        c.fill_default();
        let name_property =
            get_string_from_value(&value, "name_property").unwrap_or_else(|| "name".to_string());
        if let Some(geo_json) = value.get("geo_json") {
            c.regions = regions_from_value(geo_json, &name_property);
        }
        // [[name, value], ...]
        if let Some(arr) = value.get("data").and_then(|v| v.as_array()) {
            c.data = arr
                .iter()
                .filter_map(|item| {
                    let pair = item.as_array()?;
                    let name = pair.first()?.as_str()?;
                    Some((name.to_string(), pair.get(1)?.as_f64()? as f32))
                })
                .collect();
        }
        if let Some(projection) = get_string_from_value(&value, "projection") {
            c.projection = if projection.eq_ignore_ascii_case("equirectangular") {
                MapProjection::Equirectangular
            } else {
                MapProjection::Mercator
            };
        }
        if let Some(v) = get_f32_from_value(&value, "min") {
            c.min = v;
        }
        if let Some(v) = get_f32_from_value(&value, "max") {
            c.max = v;
        }
        if let Some(v) = get_color_from_value(&value, "min_color") {
            c.min_color = v;
        }
        if let Some(v) = get_color_from_value(&value, "max_color") {
            c.max_color = v;
        }
        if let Some(v) = get_color_slice_from_value(&value, "colors") {
            c.colors = v;
        }
        if let Some(v) = get_usize_from_value(&value, "steps") {
            c.steps = v;
        }
        if let Some(v) = get_f32_slice_from_value(&value, "thresholds") {
            c.thresholds = v;
        }
        if let Some(v) = get_color_from_value(&value, "empty_color") {
            c.empty_color = v;
        }
        if let Some(v) = get_color_from_value(&value, "border_color") {
            c.border_color = v;
        }
        if let Some(v) = get_f32_from_value(&value, "border_width") {
            c.border_width = v;
        }
        if let Some(v) = get_bool_from_value(&value, "label_show") {
            c.label_show = v;
        }
        if let Some(v) = get_bool_from_value(&value, "visual_map_show") {
            c.visual_map_show = Some(v);
        }
        Ok(c)
    }

    /// A longitude and a latitude on the plane, y growing downwards.
    fn project(&self, (lon, lat): (f64, f64)) -> (f64, f64) {
        match self.projection {
            MapProjection::Equirectangular => (lon, -lat),
            MapProjection::Mercator => {
                let lat = lat.clamp(-MERCATOR_LIMIT, MERCATOR_LIMIT).to_radians();
                let y = (std::f64::consts::FRAC_PI_4 + lat / 2.0).tan().ln();
                (lon, -y.to_degrees())
            }
        }
    }

    /// The ends of the scale: as set, or the smallest and the largest value.
    fn range(&self) -> (f32, f32) {
        if self.max > self.min {
            return (self.min, self.max);
        }
        let values = self.data.iter().map(|d| d.1).filter(|v| v.is_finite());
        let (min, max) = values.fold((f32::MAX, f32::MIN), |(lo, hi), v| (lo.min(v), hi.max(v)));
        if min > max { (0.0, 1.0) } else { (min, max) }
    }

    /// The colors the scale goes through.
    fn scale_colors(&self) -> Vec<Color> {
        if self.colors.len() >= 2 {
            self.colors.clone()
        } else {
            vec![self.min_color, self.max_color]
        }
    }

    /// The limits between the classes of the scale, when it has classes.
    fn class_limits(&self, (min, max): (f32, f32)) -> Vec<f32> {
        let mut limits: Vec<f32> = self
            .thresholds
            .iter()
            .copied()
            .filter(|t| t.is_finite())
            .collect();
        if !limits.is_empty() {
            limits.sort_by(f32::total_cmp);
            limits.dedup();
            return limits;
        }
        if self.steps < 2 {
            return vec![];
        }
        (1..self.steps)
            .map(|i| min + (max - min) * i as f32 / self.steps as f32)
            .collect()
    }

    /// Draws the scale of the colors at the bottom left of `c`, and returns
    /// the width it takes.
    fn render_visual_map(&self, c: &mut canvas::Canvas, range: (f32, f32)) -> f32 {
        let font_size = self.series_label_font_size;
        let colors = self.scale_colors();
        let limits = self.class_limits(range);
        let label = |c: &mut canvas::Canvas, text: String, x: f32, y: f32| -> f32 {
            let width = measure_text_width_family(&self.font_family, font_size, &text)
                .map(|b| b.width())
                .unwrap_or_default();
            c.text_unmeasured(Text {
                text,
                font_family: Some(self.font_family.clone()),
                font_color: Some(self.series_label_font_color),
                font_size: Some(font_size),
                x: Some(x),
                y: Some(y),
                dominant_baseline: Some("central".to_string()),
                ..Default::default()
            });
            width
        };
        let bottom = c.height();
        if limits.is_empty() {
            // A bar from the color of the smallest value, at the bottom, to
            // that of the largest, at the top.
            let (width, height) = SCALE_BAR;
            let piece = height / SCALE_PIECES as f32;
            for i in 0..SCALE_PIECES {
                let position = (i as f32 + 0.5) / SCALE_PIECES as f32;
                c.rect(Rect {
                    fill: Some(gradient_color(&colors, position).into()),
                    left: 0.0,
                    top: bottom - piece * (i + 1) as f32,
                    width,
                    // A hair more, so no line shows between two pieces.
                    height: piece + 0.5,
                    ..Default::default()
                });
            }
            let x = width + 4.0;
            let high = label(
                c,
                format_float(range.1),
                x,
                bottom - height + font_size / 2.0,
            );
            let low = label(c, format_float(range.0), x, bottom - font_size / 2.0);
            return x + high.max(low);
        }
        // A swatch for every class, the highest one on top.
        let classes = limits.len() + 1;
        let row = SWATCH + 4.0;
        let mut widest = 0.0_f32;
        for class in 0..classes {
            let position = class as f32 / (classes - 1) as f32;
            let top = bottom - row * (class + 1) as f32 + 4.0;
            c.rect(Rect {
                fill: Some(gradient_color(&colors, position).into()),
                left: 0.0,
                top,
                width: SWATCH,
                height: SWATCH,
                rx: Some(2.0),
                ry: Some(2.0),
                ..Default::default()
            });
            let text = if class == 0 {
                format!("< {}", format_float(limits[0]))
            } else if class == classes - 1 {
                format!("≥ {}", format_float(limits[class - 1]))
            } else {
                format!(
                    "{} – {}",
                    format_float(limits[class - 1]),
                    format_float(limits[class])
                )
            };
            widest = widest.max(label(c, text, SWATCH + 6.0, top + SWATCH / 2.0));
        }
        SWATCH + 6.0 + widest
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

        // The regions on the plane, and the box around all of them.
        let projected: Vec<Vec<Vec<Ring>>> = self
            .regions
            .iter()
            .map(|region| {
                region
                    .polygons
                    .iter()
                    .map(|rings| {
                        rings
                            .iter()
                            .map(|ring| ring.iter().map(|p| self.project(*p)).collect())
                            .collect()
                    })
                    .collect()
            })
            .collect();
        let (mut left, mut top, mut right, mut bottom) = (
            f64::INFINITY,
            f64::INFINITY,
            f64::NEG_INFINITY,
            f64::NEG_INFINITY,
        );
        for (x, y) in projected.iter().flatten().flatten().flatten() {
            left = left.min(*x);
            right = right.max(*x);
            top = top.min(*y);
            bottom = bottom.max(*y);
        }
        if left > right {
            // No region, no map.
            self.render_empty_text(content);
            return c.svg();
        }

        // The scale, left of the map.
        let range = self.range();
        let has_values = self.data.iter().any(|d| d.1.is_finite());
        let scale_width = if self.visual_map_show.unwrap_or(true) && has_values {
            self.render_visual_map(&mut content, range) + SCALE_GAP
        } else {
            0.0
        };

        // As large as fits, in the middle of what is left of the plot.
        let (plot_width, plot_height) = (content.width() - scale_width, content.height());
        if plot_width <= 0.0 {
            return c.svg();
        }
        let (map_width, map_height) = ((right - left).max(1e-9), (bottom - top).max(1e-9));
        let scale = (plot_width as f64 / map_width).min(plot_height as f64 / map_height);
        let offset_x = scale_width as f64 + (plot_width as f64 - map_width * scale) / 2.0;
        let offset_y = (plot_height as f64 - map_height * scale) / 2.0;
        let place = |(x, y): (f64, f64)| -> Point {
            Point {
                x: (offset_x + (x - left) * scale) as f32,
                y: (offset_y + (y - top) * scale) as f32,
            }
        };

        let colors = self.scale_colors();
        let mut labels: Vec<(Point, &str, Color)> = vec![];
        for (region, polygons) in self.regions.iter().zip(projected.iter()) {
            // The outlines of the region on the plot, without the corners
            // that are too close to the one before to show.
            let mut outlines: Vec<Outline> = vec![];
            for ring in polygons.iter().flatten() {
                let mut points: Vec<Point> = Vec::with_capacity(ring.len());
                for point in ring.iter().map(|p| place(*p)) {
                    let far = points.last().is_none_or(|last| {
                        (point.x - last.x).abs() >= MIN_STEP || (point.y - last.y).abs() >= MIN_STEP
                    });
                    if far {
                        points.push(point);
                    }
                }
                // A closed ring ends where it starts: once is enough.
                if points.len() > 1
                    && let (Some(first), Some(last)) = (points.first(), points.last())
                    && (first.x - last.x).abs() < MIN_STEP
                    && (first.y - last.y).abs() < MIN_STEP
                {
                    points.pop();
                }
                if points.len() < 3 {
                    continue;
                }
                let (mut area, mut cx, mut cy) = (0.0_f32, 0.0_f32, 0.0_f32);
                for (i, a) in points.iter().enumerate() {
                    let b = points[(i + 1) % points.len()];
                    let cross = a.x * b.y - b.x * a.y;
                    area += cross;
                    cx += (a.x + b.x) * cross;
                    cy += (a.y + b.y) * cross;
                }
                let center = if area.abs() > f32::EPSILON {
                    Point {
                        x: cx / (3.0 * area),
                        y: cy / (3.0 * area),
                    }
                } else {
                    points[0]
                };
                outlines.push(Outline {
                    points,
                    area,
                    center,
                });
            }
            if outlines.is_empty() {
                continue;
            }
            let value = self
                .data
                .iter()
                .find(|d| d.0 == region.name)
                .map(|d| d.1)
                .filter(|v| v.is_finite());
            let fill = match value {
                Some(value) => gradient_color(
                    &colors,
                    scale_position(value, range, self.steps, &self.thresholds),
                ),
                None => self.empty_color,
            };
            let text = match value {
                Some(value) => format!("{}: {}", region.name, format_float(value)),
                None => region.name.clone(),
            };
            let tooltip_text = (self.tooltip_show && !text.is_empty()).then_some(text);
            // The name stands on the largest part of the region.
            let center = outlines
                .iter()
                .max_by(|a, b| a.area.abs().total_cmp(&b.area.abs()))
                .map(|outline| outline.center)
                .unwrap_or_default();
            let mut dataset = vec![("name".to_string(), region.name.clone())];
            if let Some(value) = value {
                dataset.push(("value".to_string(), format_float(value)));
            }
            content.shape(Shape {
                color: Some(self.border_color),
                stroke_width: self.border_width.max(0.0),
                fill: Some(fill),
                rings: outlines.into_iter().map(|o| o.points).collect(),
                class: tooltip_text.as_ref().map(|_| "ct-trigger".to_string()),
                title: tooltip_text.clone(),
                dataset,
                ..Default::default()
            });
            if let Some(text) = tooltip_text {
                content.text_unmeasured(Text {
                    text,
                    class: Some("ct-tip".to_string()),
                    font_family: Some(self.font_family.clone()),
                    font_color: Some(self.series_label_font_color),
                    font_size: Some(self.series_label_font_size),
                    x: Some(center.x),
                    y: Some(center.y),
                    text_anchor: Some("middle".to_string()),
                    dominant_baseline: Some("central".to_string()),
                    ..Default::default()
                });
            }
            if self.label_show && !region.name.is_empty() {
                // Dark on a light region, light on a dark one.
                let color = if fill.is_light() {
                    Color::black().with_alpha(200)
                } else {
                    Color::white()
                };
                labels.push((center, &region.name, color));
            }
        }

        // The names, over every region.
        let font_size = self.series_label_font_size;
        let mut boxes = LabelBoxes::new(true);
        for (center, name, color) in labels {
            let width = measure_text_width_family(&self.font_family, font_size, name)
                .map(|b| b.width())
                .unwrap_or_default();
            if !boxes.try_place(
                center.x - width / 2.0,
                center.y - font_size / 2.0,
                width,
                font_size,
            ) {
                continue;
            }
            content.text_unmeasured(Text {
                text: name.to_string(),
                font_family: Some(self.font_family.clone()),
                font_color: Some(color),
                font_size: Some(font_size),
                font_weight: self.series_label_font_weight.clone(),
                x: Some(center.x),
                y: Some(center.y),
                text_anchor: Some("middle".to_string()),
                dominant_baseline: Some("central".to_string()),
                ..Default::default()
            });
        }

        if self.tooltip_show {
            c.svg_with_style(TOOLTIP_STYLE)
        } else {
            c.svg()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{MapChart, MapProjection, MapRegion};
    use pretty_assertions::assert_eq;

    const GEO_JSON: &str = r#"{
        "type": "FeatureCollection",
        "features": [
            {"type": "Feature", "properties": {"name": "Square", "code": 7},
             "geometry": {"type": "Polygon", "coordinates": [
                [[0, 0], [10, 0], [10, 10], [0, 10], [0, 0]],
                [[4, 4], [6, 4], [6, 6], [4, 6], [4, 4]]
             ]}},
            {"type": "Feature", "id": "isles", "properties": {},
             "geometry": {"type": "MultiPolygon", "coordinates": [
                [[[12, 0], [14, 0], [14, 2], [12, 0]]],
                [[[12, 5], [15, 5], [15, 9], [12, 5]]]
             ]}},
            {"type": "Feature", "properties": {"name": "Road"},
             "geometry": {"type": "LineString", "coordinates": [[0, 0], [1, 1]]}}
        ]
    }"#;

    #[test]
    fn regions_from_geo_json() {
        let regions = MapRegion::from_geo_json(GEO_JSON, "name").unwrap();
        // The areas, with their holes and islands; a line is no region.
        assert_eq!(2, regions.len());
        assert_eq!("Square", regions[0].name);
        assert_eq!(1, regions[0].polygons.len());
        assert_eq!(2, regions[0].polygons[0].len());
        assert_eq!((10.0, 10.0), regions[0].polygons[0][0][2]);
        // Without the property the id is the name.
        assert_eq!("isles", regions[1].name);
        assert_eq!(2, regions[1].polygons.len());
        // Another property, also one that is a number.
        let regions = MapRegion::from_geo_json(GEO_JSON, "code").unwrap();
        assert_eq!("7", regions[0].name);

        // A single feature is a map of one region.
        let single = r#"{"type": "Feature", "properties": {"name": "One"},
            "geometry": {"type": "Polygon", "coordinates": [[[0, 0], [1, 0], [1, 1]]]}}"#;
        assert_eq!(
            "One",
            MapRegion::from_geo_json(single, "name").unwrap()[0].name
        );
        assert!(MapRegion::from_geo_json("{}", "name").unwrap().is_empty());
        assert!(MapRegion::from_geo_json("not json", "name").is_err());
    }

    #[test]
    fn projections() {
        let mut chart = MapChart::new(vec![], vec![]);
        assert_eq!(MapProjection::Mercator, chart.projection);
        // The equator is where it is; north is up, and further up than its
        // latitude tells.
        let (x, y) = chart.project((30.0, 0.0));
        assert!(x == 30.0 && y.abs() < 1e-9, "{x} {y}");
        let (_, y) = chart.project((0.0, 60.0));
        assert!((y + 75.456).abs() < 0.01, "{y}");
        // The poles are cut off.
        assert_eq!(chart.project((0.0, 85.0)), chart.project((0.0, 90.0)));
        chart.projection = MapProjection::Equirectangular;
        assert_eq!((30.0, -60.0), chart.project((30.0, 60.0)));
    }
}
