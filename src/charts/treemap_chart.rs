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

#[derive(Clone, Debug, Default, Chart)]
pub struct TreemapChart {
    pub width: f32,
    pub height: f32,
    pub x: f32,
    pub y: f32,
    pub margin: Box,
    pub series_list: Vec<Series>,
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

    // legend
    pub legend_font_size: f32,
    pub legend_font_color: Color,
    pub legend_font_weight: Option<String>,
    pub legend_align: Align,
    pub legend_margin: Option<Box>,
    pub legend_category: LegendCategory,
    pub legend_show: Option<bool>,

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
        let mut c = TreemapChart { ..Default::default() };
        let value = c.fill_option(json)?;
        if let Some(v) = get_f32_from_value(&value, "item_gap") {
            c.item_gap = v;
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
                let v = *s.data.first()?;
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

        items.sort_by(|a, b| b.area.partial_cmp(&a.area).unwrap_or(std::cmp::Ordering::Equal));

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
                Color { r: 30, g: 30, b: 30, a: 255 }
            } else {
                Color { r: 255, g: 255, b: 255, a: 230 }
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
                    ..Default::default()
                });
            }
        }

        c.svg()
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
}
