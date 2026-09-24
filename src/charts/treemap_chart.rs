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
use super::base::ChartBase;
use super::canvas;
use super::color::*;
use super::common::*;
use super::component::*;
use super::params::*;
use super::sunburst_chart::{SunburstData, lighten, parse_node};
use super::theme::{get_default_theme_name, get_theme};
use super::util::*;
use crate::charts::measure_text_width_family;

// ── Squarify algorithm ───────────────────────────────────────────────────────

struct TmItem {
    /// Index of the node this item came from, to find its children.
    index: usize,
    name: String,
    value_str: String, // pre-formatted original value for label
    color: Color,
    area: f32, // normalised pixel area
}

struct TmRect {
    index: usize,
    name: String,
    value_str: String,
    color: Color,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
}

/// A node of the (possibly nested) treemap data, with the value of a
/// branch being the sum of its leaves.
struct TmNode {
    name: String,
    value: f32,
    color: Color,
    children: Vec<TmNode>,
}

impl TmNode {
    /// Builds the tree from nested data; a branch takes the sum of its
    /// children as value, colors fade with depth like the sunburst rings.
    fn from_data(data: &SunburstData, color: Color, depth: usize) -> Option<TmNode> {
        let color = data.color.unwrap_or(color);
        let children: Vec<TmNode> = data
            .children
            .iter()
            .filter_map(|child| {
                TmNode::from_data(child, lighten(color, 0.15 * (depth + 1) as f32), depth + 1)
            })
            .collect();
        let value = if children.is_empty() {
            data.value
        } else {
            children.iter().map(|c| c.value).sum()
        };
        if value <= 0.0 {
            return None;
        }
        Some(TmNode {
            name: data.name.clone(),
            value,
            color,
            children,
        })
    }
}

/// Worst aspect ratio of a row of normalised areas given the available short side.
fn worst_ratio(areas: &[f32], short: f32) -> f32 {
    if short <= 0.0 || areas.is_empty() {
        return f32::MAX;
    }
    let s: f32 = areas.iter().sum();
    if s <= 0.0 {
        return f32::MAX;
    }
    let max = areas.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
    let min = areas.iter().cloned().fold(f32::INFINITY, f32::min);
    let s2 = s * s;
    let w2 = short * short;
    (max * w2 / s2).max(s2 / (min * w2))
}

fn squarify(items: &[TmItem], x: f32, y: f32, w: f32, h: f32, out: &mut Vec<TmRect>) {
    if items.is_empty() || w <= 0.0 || h <= 0.0 {
        return;
    }
    if items.len() == 1 {
        out.push(TmRect {
            index: items[0].index,
            name: items[0].name.clone(),
            value_str: items[0].value_str.clone(),
            color: items[0].color,
            x,
            y,
            w,
            h,
        });
        return;
    }

    let short = w.min(h);
    let areas: Vec<f32> = items.iter().map(|it| it.area).collect();
    let mut prev_worst = worst_ratio(&areas[..1], short);
    let mut split = 1usize;
    for i in 2..=items.len() {
        let wr = worst_ratio(&areas[..i], short);
        if wr > prev_worst {
            break;
        }
        prev_worst = wr;
        split = i;
    }

    let row = &items[..split];
    let rest = &items[split..];
    let row_sum: f32 = row.iter().map(|it| it.area).sum();

    if w >= h {
        // portrait row on the left: items stacked top-to-bottom
        let row_w = row_sum / h;
        let mut cy = y;
        for item in row {
            let ih = item.area / row_w;
            out.push(TmRect {
                index: item.index,
                name: item.name.clone(),
                value_str: item.value_str.clone(),
                color: item.color,
                x,
                y: cy,
                w: row_w,
                h: ih,
            });
            cy += ih;
        }
        squarify(rest, x + row_w, y, w - row_w, h, out);
    } else {
        // landscape row on top: items placed left-to-right
        let row_h = row_sum / w;
        let mut cx = x;
        for item in row {
            let iw = item.area / row_h;
            out.push(TmRect {
                index: item.index,
                name: item.name.clone(),
                value_str: item.value_str.clone(),
                color: item.color,
                x: cx,
                y,
                w: iw,
                h: row_h,
            });
            cx += iw;
        }
        squarify(rest, x, y + row_h, w, h - row_h, out);
    }
}

// ── TreemapChart ─────────────────────────────────────────────────────────────

/// A treemap laying values out as rectangles sized proportionally.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct TreemapChart {
    /// The shared chart options (size, series, title/legend, axes); exposed
    /// directly on the chart through `Deref`, e.g. `chart.title_text`.
    pub base: ChartBase,
    y_axis_configs: Vec<YAxisConfig>,

    // treemap-specific
    /// Pixel gap between adjacent cells. Default: 2.0.
    pub item_gap: f32,
    /// Nested data (name / value / children, as for the sunburst and tree
    /// charts); when set it takes precedence over `series_list`, and every
    /// branch is subdivided into its children.
    pub series_data: Vec<SunburstData>,
}

impl std::ops::Deref for TreemapChart {
    type Target = ChartBase;
    fn deref(&self) -> &ChartBase {
        &self.base
    }
}
impl std::ops::DerefMut for TreemapChart {
    fn deref_mut(&mut self) -> &mut ChartBase {
        &mut self.base
    }
}

impl TreemapChart {
    fn fill_default(&mut self) {
        if self.item_gap < 0.0 {
            self.item_gap = 0.0;
        }
        if self.item_gap == 0.0 {
            self.item_gap = 2.0;
        }
    }

    /// Creates a treemap chart with the given theme.
    pub fn new_with_theme(series_list: Vec<Series>, theme: &str) -> TreemapChart {
        let mut c = TreemapChart {
            ..Default::default()
        };
        c.series_list = series_list;
        c.base.fill_theme(get_theme(theme), &mut c.y_axis_configs);
        c.fill_default();
        c
    }

    /// Creates a treemap chart with the default theme.
    pub fn new(series_list: Vec<Series>) -> TreemapChart {
        TreemapChart::new_with_theme(series_list, &get_default_theme_name())
    }

    /// Creates a treemap chart from JSON options.
    pub fn from_json(json: &str) -> canvas::Result<TreemapChart> {
        let mut c = TreemapChart {
            ..Default::default()
        };
        let value =
            c.base
                .fill_option(json, &mut c.y_axis_configs, super::schema::TREEMAP_FIELDS)?;
        if let Some(v) = get_f32_from_value(&value, "item_gap") {
            c.item_gap = v;
        }
        if let Some(arr) = value.get("series_data").and_then(|v| v.as_array()) {
            c.series_data = arr.iter().filter_map(parse_node).collect();
        }
        c.fill_default();
        Ok(c)
    }

    /// Renders the chart to an SVG string.
    pub fn svg(&self) -> canvas::Result<String> {
        let mut c = Canvas::new_width_xy(self.width, self.height, self.x, self.y);
        let top = self.render_header(&mut c);

        let mut content_c = c.child(Box {
            top,
            ..Default::default()
        });

        let cw = content_c.width();
        let ch = content_c.height();
        if cw <= 0.0 || ch <= 0.0 {
            return c.svg();
        }

        // The tree to lay out: nested data, or one leaf per series.
        let nodes: Vec<TmNode> = if !self.series_data.is_empty() {
            self.series_data
                .iter()
                .enumerate()
                .filter_map(|(i, data)| {
                    TmNode::from_data(data, get_color(&self.series_colors, i), 0)
                })
                .collect()
        } else {
            self.series_list
                .iter()
                .enumerate()
                .filter_map(|(i, s)| {
                    let v = *s.data_values().first()?;
                    if v <= 0.0 {
                        return None;
                    }
                    Some(TmNode {
                        name: s.name.clone(),
                        value: v,
                        color: get_color(&self.series_colors, s.index.unwrap_or(i)),
                        children: vec![],
                    })
                })
                .collect()
        };
        if nodes.is_empty() {
            return c.svg();
        }
        let grand_total: f32 = nodes.iter().map(|n| n.value).sum();
        let formatter = &self.series_label_formatter;
        let value_label = |name: &str, value: f32| -> String {
            if formatter.is_empty() {
                format_float(value)
            } else {
                LabelOption {
                    series_name: name.to_string(),
                    category_name: name.to_string(),
                    value,
                    percentage: value / grand_total,
                    formatter: formatter.clone(),
                }
                .format()
            }
        };

        // Lays out one level: the nodes fill `(x, y, w, h)` by value, and a
        // branch is subdivided into its children inside its own cell.
        #[allow(clippy::too_many_arguments)]
        fn layout(
            nodes: &[TmNode],
            x: f32,
            y: f32,
            w: f32,
            h: f32,
            gap: f32,
            label: &dyn Fn(&str, f32) -> String,
            out: &mut Vec<TmRect>,
        ) {
            let mut items: Vec<TmItem> = nodes
                .iter()
                .enumerate()
                .map(|(index, n)| TmItem {
                    index,
                    name: n.name.clone(),
                    value_str: label(&n.name, n.value),
                    color: n.color,
                    area: n.value,
                })
                .collect();
            items.sort_by(|a, b| {
                b.area
                    .partial_cmp(&a.area)
                    .unwrap_or(std::cmp::Ordering::Equal)
            });
            let total: f32 = items.iter().map(|it| it.area).sum();
            if total <= 0.0 {
                return;
            }
            let area = w * h;
            for it in &mut items {
                it.area = it.area / total * area;
            }
            let mut rects: Vec<TmRect> = vec![];
            squarify(&items, x, y, w, h, &mut rects);
            for r in rects {
                let node = &nodes[r.index];
                if node.children.is_empty() {
                    out.push(r);
                } else {
                    // Inset by the gap so sibling branches stay separated.
                    let inner_w = (r.w - gap).max(0.0);
                    let inner_h = (r.h - gap).max(0.0);
                    layout(
                        &node.children,
                        r.x + gap / 2.0,
                        r.y + gap / 2.0,
                        inner_w,
                        inner_h,
                        gap,
                        label,
                        out,
                    );
                }
            }
        }

        let mut rects: Vec<TmRect> = vec![];
        layout(
            &nodes,
            0.0,
            0.0,
            cw,
            ch,
            self.item_gap,
            &value_label,
            &mut rects,
        );

        let half_gap = self.item_gap / 2.0;
        let font_size = self.series_label_font_size.max(10.0);
        let font_color = self.series_label_font_color;
        let anim_class = self.animation.as_ref().map(|_| "treemap-anim".to_string());

        for r in &rects {
            let rx = r.x + half_gap;
            let ry = r.y + half_gap;
            let rw = (r.w - self.item_gap).max(0.0);
            let rh = (r.h - self.item_gap).max(0.0);
            if rw <= 0.0 || rh <= 0.0 {
                continue;
            }

            let tooltip_text = self
                .tooltip_show
                .then(|| format!("{}: {}", r.name, r.value_str));
            let mut class = anim_class.clone();
            if tooltip_text.is_some() {
                class = Some(match class {
                    Some(c) => format!("{c} ct-trigger"),
                    None => "ct-trigger".to_string(),
                });
            }
            content_c.rect(Rect {
                fill: Some(r.color.into()),
                left: rx,
                top: ry,
                width: rw,
                height: rh,
                class,
                title: tooltip_text.clone(),
                dataset: vec![
                    ("series".to_string(), r.name.clone()),
                    ("value".to_string(), r.value_str.clone()),
                ],
                ..Default::default()
            });
            if let Some(text) = tooltip_text {
                content_c.text(Text {
                    text,
                    class: Some("ct-tip".to_string()),
                    font_family: Some(self.font_family.clone()),
                    font_color: Some(font_color),
                    font_size: Some(font_size),
                    x: Some(rx + rw / 2.0),
                    y: Some(ry + rh / 2.0),
                    text_anchor: Some("middle".to_string()),
                    dominant_baseline: Some("central".to_string()),
                    ..Default::default()
                });
            }

            // Label: show name when cell is large enough
            if rw < font_size * 2.0 || rh < font_size + 4.0 {
                continue;
            }
            let name_w = measure_text_width_family(&self.font_family, font_size, &r.name)
                .map(|b| b.width())
                .unwrap_or(r.name.len() as f32 * font_size * 0.6);
            if name_w + 4.0 > rw {
                continue;
            }

            let show_value = rh >= font_size * 2.5;
            let label_y = if show_value {
                ry + rh / 2.0 - font_size * 0.6
            } else {
                ry + rh / 2.0
            };

            // Lighten text colour against dark background for readability
            let text_color = if r.color.is_light() {
                Color {
                    r: 30,
                    g: 30,
                    b: 30,
                    a: 255,
                }
            } else {
                Color {
                    r: 255,
                    g: 255,
                    b: 255,
                    a: 230,
                }
            };

            content_c.text(Text {
                text: r.name.clone(),
                font_family: Some(self.font_family.clone()),
                font_color: Some(text_color),
                font_size: Some(font_size),
                x: Some(rx + rw / 2.0),
                y: Some(label_y),
                text_anchor: Some("middle".to_string()),
                dominant_baseline: Some("central".to_string()),
                class: anim_class.clone(),
                ..Default::default()
            });

            if show_value {
                let val_font_size = (font_size * 0.85).max(9.0);
                content_c.text(Text {
                    text: r.value_str.clone(),
                    font_family: Some(self.font_family.clone()),
                    font_color: Some(text_color.with_alpha(180)),
                    font_size: Some(val_font_size),
                    x: Some(rx + rw / 2.0),
                    y: Some(label_y + font_size * 1.3),
                    text_anchor: Some("middle".to_string()),
                    dominant_baseline: Some("central".to_string()),
                    class: anim_class.clone(),
                    ..Default::default()
                });
            }
        }

        let mut css = String::new();
        if let Some(ref anim) = self.animation {
            css.push_str(&format!(
                "@keyframes treemap-fade{{from{{opacity:0}}to{{opacity:1}}}} \
                 .treemap-anim{{animation:treemap-fade {}ms {} both}} ",
                anim.duration,
                anim.safe_easing()
            ));
        }
        if self.tooltip_show {
            css.push_str(TOOLTIP_STYLE);
        }
        if css.is_empty() {
            c.svg()
        } else {
            c.svg_with_style(&css)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::TreemapChart;

    fn make_treemap() -> TreemapChart {
        TreemapChart::new(vec![
            ("nodeExcel", vec![600.0]).into(),
            ("nodePPT", vec![500.0]).into(),
            ("nodeDoc", vec![400.0]).into(),
            ("nodeWeb", vec![300.0]).into(),
            ("nodeWord", vec![200.0]).into(),
            ("nodeOther", vec![100.0]).into(),
        ])
    }

    #[test]
    fn treemap_chart_basic() {
        let chart = make_treemap();
        assert_snapshot!("treemap_chart/basic.svg", chart.svg().unwrap());
    }

    #[test]
    fn treemap_chart_basic_json() {
        let chart = TreemapChart::from_json(
            r##"{
                "title_text": "Disk Usage",
                "item_gap": 3,
                "series_list": [
                    {"name": "nodeExcel", "data": [600]},
                    {"name": "nodePPT",   "data": [500]},
                    {"name": "nodeDoc",   "data": [400]},
                    {"name": "nodeWeb",   "data": [300]},
                    {"name": "nodeWord",  "data": [200]},
                    {"name": "nodeOther", "data": [100]}
                ]
            }"##,
        )
        .unwrap();
        assert_snapshot!("treemap_chart/basic_json.svg", chart.svg().unwrap());
    }

    #[test]
    fn treemap_chart_animation() {
        let mut chart = make_treemap();
        chart.animation = Some(super::AnimationConfig {
            duration: 600,
            easing: "linear".to_string(),
            delay: 0,
        });
        let svg = chart.svg().unwrap();
        assert!(
            svg.contains("treemap-fade"),
            "missing @keyframes treemap-fade"
        );
        assert!(
            svg.contains(r#"class="treemap-anim""#),
            "missing class on cell"
        );
        assert!(svg.contains("600ms linear"), "missing duration/easing");
    }
}
