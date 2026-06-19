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
use super::canvas;
use super::color::*;
use super::common::*;
use super::component::*;
use super::params::*;
use super::theme::{DEFAULT_Y_AXIS_WIDTH, Theme, get_default_theme_name, get_theme};
use super::util::*;
use crate::charts::measure_text_width_family;
use charts_rs_derive::Chart;
use std::sync::Arc;

// ── Squarify algorithm ───────────────────────────────────────────────────────

struct TmItem {
    name: String,
    value_str: String, // pre-formatted original value for label
    color: Color,
    area: f32, // normalised pixel area
}

struct TmRect {
    name: String,
    value_str: String,
    color: Color,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
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

#[charts_rs_derive::chart_common_fields]
#[derive(Clone, Debug, Default, Chart)]
pub struct TreemapChart {
    // x/y axis (required by #[derive(Chart)], unused in rendering)
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

    // series (required by #[derive(Chart)])
    pub series_stroke_width: f32,
    pub series_label_font_color: Color,
    pub series_label_font_size: f32,
    pub series_label_font_weight: Option<String>,
    pub series_label_formatter: String,
    pub series_colors: Vec<Color>,
    pub series_symbol: Option<Symbol>,
    pub series_smooth: bool,
    pub series_fill: bool,

    // treemap-specific
    /// Pixel gap between adjacent cells. Default: 2.0.
    pub item_gap: f32,
    /// Optional fade-in animation for the cells and their labels. The
    /// `delay` field is not used (all cells fade in together).
    pub animation: Option<AnimationConfig>,
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

    pub fn new_with_theme(series_list: Vec<Series>, theme: &str) -> TreemapChart {
        let mut c = TreemapChart {
            series_list,
            ..Default::default()
        };
        c.fill_theme(get_theme(theme));
        c.fill_default();
        c
    }

    pub fn new(series_list: Vec<Series>) -> TreemapChart {
        TreemapChart::new_with_theme(series_list, &get_default_theme_name())
    }

    pub fn from_json(json: &str) -> canvas::Result<TreemapChart> {
        let mut c = TreemapChart {
            ..Default::default()
        };
        let value = c.fill_option(json)?;
        if let Some(v) = get_f32_from_value(&value, "item_gap") {
            c.item_gap = v;
        }
        if let Some(anim) = value.get("animation")
            && !anim.is_null()
        {
            let mut config = AnimationConfig::default();
            if let Some(d) = get_usize_from_value(anim, "duration") {
                config.duration = d as u32;
            }
            if let Some(e) = get_string_from_value(anim, "easing") {
                config.easing = e;
            }
            if let Some(d) = get_usize_from_value(anim, "delay") {
                config.delay = d as u32;
            }
            c.animation = Some(config);
        }
        c.fill_default();
        Ok(c)
    }

    pub fn svg(&self) -> canvas::Result<String> {
        let mut c = Canvas::new_width_xy(self.width, self.height, self.x, self.y);
        self.render_background(c.child(Box::default()));
        c.margin = self.margin.clone();

        let title_height = self.render_title(c.child(Box::default()));
        let legend_height = self.render_legend(c.child(Box::default()));
        let top = title_height.max(legend_height);

        let mut content_c = c.child(Box {
            top,
            ..Default::default()
        });

        let cw = content_c.width();
        let ch = content_c.height();
        if cw <= 0.0 || ch <= 0.0 {
            return c.svg();
        }

        // Collect items with positive values, sort descending
        let mut items: Vec<TmItem> = self
            .series_list
            .iter()
            .enumerate()
            .filter_map(|(i, s)| {
                let v = *s.data_values().first()?;
                if v <= 0.0 {
                    return None;
                }
                let color = get_color(&self.series_colors, s.index.unwrap_or(i));
                let value_str = format_float(v);
                Some(TmItem {
                    name: s.name.clone(),
                    value_str,
                    color,
                    area: v,
                })
            })
            .collect();

        if items.is_empty() {
            return c.svg();
        }

        items.sort_by(|a, b| {
            b.area
                .partial_cmp(&a.area)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        // Normalise to canvas area
        let total: f32 = items.iter().map(|it| it.area).sum();
        let canvas_area = cw * ch;
        for it in &mut items {
            it.area = it.area / total * canvas_area;
        }

        let mut rects: Vec<TmRect> = vec![];
        squarify(&items, 0.0, 0.0, cw, ch, &mut rects);

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

            content_c.rect(Rect {
                fill: Some(r.color.into()),
                left: rx,
                top: ry,
                width: rw,
                height: rh,
                class: anim_class.clone(),
                ..Default::default()
            });

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
            let _ = font_color; // use auto-contrast instead

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

        if let Some(ref anim) = self.animation {
            let css = format!(
                "@keyframes treemap-fade{{from{{opacity:0}}to{{opacity:1}}}} \
                 .treemap-anim{{animation:treemap-fade {}ms {} both}}",
                anim.duration, anim.easing
            );
            c.svg_with_style(&css)
        } else {
            c.svg()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::TreemapChart;
    use pretty_assertions::assert_eq;

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
        assert_eq!(
            include_str!("../../asset/treemap_chart/basic.svg"),
            chart.svg().unwrap()
        );
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
        assert_eq!(
            include_str!("../../asset/treemap_chart/basic_json.svg"),
            chart.svg().unwrap()
        );
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
