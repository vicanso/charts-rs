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

//! Pure-Rust Mermaid diagram parser and SVG renderer.

mod adapters;
mod class;
mod draw;
mod er;
mod flowchart;
mod layout;
mod mindmap;
mod preprocess;
mod quadrant;
mod sequence;
mod state;
mod style;
mod text;

use std::sync::Arc;

use super::base::{ChartBase, check_theme_name, overlay_texts};
use super::canvas::Canvas;
use super::common::TextConfig;
use super::component::Text;
use super::error::{Error, Result};
use super::font::{DEFAULT_FONT_FAMILY, measure_text_width_family, text_ellipsis};
use super::params::{
    get_align_from_value, get_bool_from_value, get_color_from_value, get_f32_from_value,
    get_margin_from_value, get_string_from_value, get_texts_from_value,
};
use super::theme::{Theme, get_default_theme_name, get_theme};
use preprocess::{DiagramKind, preprocess};

/// A Mermaid diagram rendered into pure SVG (and optionally raster images).
///
/// Supports `flowchart` / `graph`, `sequenceDiagram`, `stateDiagram` /
/// `stateDiagram-v2`, `classDiagram` / `classDiagram-v2`, `erDiagram`,
/// `mindmap`, `quadrantChart`, `pie`, `gantt`, `sankey-beta`, `xychart-beta`,
/// `radar-beta` / `radar`, and `treemap-beta` / `treemap`, inheriting styling
/// from the active `charts-rs` [`Theme`](crate::Theme).
///
/// By default (`width == 0.0` and `height == 0.0`), graph/diagram engines
/// (`flowchart`, `sequenceDiagram`, `stateDiagram`, `classDiagram`,
/// `erDiagram`, `mindmap`) size their SVG canvas automatically to fit the
/// laid-out diagram; setting `width` and/or `height` scales the diagram
/// proportionally via `viewBox`.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct MermaidChart {
    /// Shared chart options (`margin`, `font_family`, `title`, `sub_title`,
    /// `background_color`, `compact`, `texts`); exposed directly via `Deref`.
    pub base: ChartBase,
    /// Mermaid diagram source text.
    pub source: String,
    theme_name: Option<String>,
    explicit_background: bool,
}

impl std::ops::Deref for MermaidChart {
    type Target = ChartBase;
    fn deref(&self) -> &ChartBase {
        &self.base
    }
}

impl std::ops::DerefMut for MermaidChart {
    fn deref_mut(&mut self) -> &mut ChartBase {
        &mut self.base
    }
}

impl MermaidChart {
    /// Creates a Mermaid chart with the default theme and automatic sizing.
    pub fn new(source: impl Into<String>) -> Self {
        Self::new_with_theme(source, &get_default_theme_name())
    }

    /// Creates a Mermaid chart with a specific theme and automatic sizing.
    pub fn new_with_theme(source: impl Into<String>, theme: &str) -> Self {
        let mut chart = Self {
            base: ChartBase::default(),
            source: source.into(),
            theme_name: (!theme.is_empty()).then(|| theme.to_string()),
            explicit_background: false,
        };
        let mut y_axis_configs = vec![];
        chart.base.fill_theme(get_theme(theme), &mut y_axis_configs);
        // `0.0` signals auto-size to the Mermaid layout engines.
        chart.base.width = 0.0;
        chart.base.height = 0.0;
        chart
    }

    /// Creates a Mermaid chart from a JSON options document.
    pub fn from_json(data: &str) -> Result<Self> {
        let value: serde_json::Value = serde_json::from_str(data)?;
        super::schema::validate(&value, &[super::schema::MERMAID_FIELDS])?;

        let theme = get_string_from_value(&value, "theme").unwrap_or_default();
        check_theme_name(&theme)?;
        let source = get_string_from_value(&value, "source").unwrap_or_default();
        let mut chart = Self::new_with_theme(source, &theme);

        if let Some(width) = get_f32_from_value(&value, "width") {
            chart.width = width;
        }
        if let Some(height) = get_f32_from_value(&value, "height") {
            chart.height = height;
        }
        if let Some(x) = get_f32_from_value(&value, "x") {
            chart.x = x;
        }
        if let Some(y) = get_f32_from_value(&value, "y") {
            chart.y = y;
        }
        if let Some(margin) = get_margin_from_value(&value, "margin") {
            chart.margin = margin;
        }
        if let Some(font_family) = get_string_from_value(&value, "font_family") {
            chart.font_family = font_family;
        }
        if let Some(bg) = get_color_from_value(&value, "background_color") {
            chart.background_color = bg;
            chart.explicit_background = true;
        }
        if let Some(compact) = get_bool_from_value(&value, "compact") {
            chart.compact = compact;
        }

        if let Some(t) = get_string_from_value(&value, "title_text") {
            chart.title.text = t;
        }
        if let Some(v) = get_f32_from_value(&value, "title_font_size") {
            chart.title.font.size = v;
        }
        if let Some(v) = get_color_from_value(&value, "title_font_color") {
            chart.title.font.color = v;
        }
        if let Some(v) = get_string_from_value(&value, "title_font_weight") {
            chart.title.font.weight = Some(v);
        }
        if let Some(v) = get_margin_from_value(&value, "title_margin") {
            chart.title.margin = Some(v);
        }
        if let Some(v) = get_align_from_value(&value, "title_align") {
            chart.title.align = v;
        }
        if let Some(v) = get_f32_from_value(&value, "title_height") {
            chart.title.height = v;
        }

        if let Some(t) = get_string_from_value(&value, "sub_title_text") {
            chart.sub_title.text = t;
        }
        if let Some(v) = get_f32_from_value(&value, "sub_title_font_size") {
            chart.sub_title.font.size = v;
        }
        if let Some(v) = get_color_from_value(&value, "sub_title_font_color") {
            chart.sub_title.font.color = v;
        }
        if let Some(v) = get_string_from_value(&value, "sub_title_font_weight") {
            chart.sub_title.font.weight = Some(v);
        }
        if let Some(v) = get_margin_from_value(&value, "sub_title_margin") {
            chart.sub_title.margin = Some(v);
        }
        if let Some(v) = get_align_from_value(&value, "sub_title_align") {
            chart.sub_title.align = v;
        }
        if let Some(v) = get_f32_from_value(&value, "sub_title_height") {
            chart.sub_title.height = v;
        }

        if let Some(texts) = get_texts_from_value(&value, "texts") {
            chart.texts = texts;
        }

        Ok(chart)
    }

    /// Renders the Mermaid diagram to an SVG string.
    pub fn svg(&self) -> Result<String> {
        let (svg, _, _) = self.render()?;
        Ok(svg)
    }

    /// Renders the diagram and returns `(svg, actual_width, actual_height)`.
    pub(crate) fn render(&self) -> Result<(String, f32, f32)> {
        if self.source.trim().is_empty() {
            return Err(Error::Mermaid {
                line: None,
                message: "empty Mermaid diagram".to_string(),
            });
        }
        let prep = preprocess(&self.source)?;
        let effective_theme = match &self.theme_name {
            Some(name) if !name.is_empty() => name.clone(),
            _ => prep
                .init_theme
                .clone()
                .unwrap_or_else(get_default_theme_name),
        };
        check_theme_name(&effective_theme)?;

        match prep.kind {
            DiagramKind::Flowchart(dir) => flowchart::render(
                self,
                &effective_theme,
                prep.title.as_deref(),
                dir,
                &prep.lines,
            ),
            DiagramKind::Sequence => {
                sequence::render(self, &effective_theme, prep.title.as_deref(), &prep.lines)
            }
            DiagramKind::State => {
                state::render(self, &effective_theme, prep.title.as_deref(), &prep.lines)
            }
            DiagramKind::Class => {
                class::render(self, &effective_theme, prep.title.as_deref(), &prep.lines)
            }
            DiagramKind::Er => {
                er::render(self, &effective_theme, prep.title.as_deref(), &prep.lines)
            }
            DiagramKind::Mindmap => {
                mindmap::render(self, &effective_theme, prep.title.as_deref(), &prep.lines)
            }
            DiagramKind::Quadrant => {
                quadrant::render(self, &effective_theme, prep.title.as_deref(), &prep.lines)
            }
            DiagramKind::Radar => {
                adapters::radar::render(self, &effective_theme, prep.title.as_deref(), &prep.lines)
            }
            DiagramKind::Treemap => adapters::treemap::render(
                self,
                &effective_theme,
                prep.title.as_deref(),
                &prep.lines,
            ),
            DiagramKind::Pie {
                show_data,
                inline_title,
            } => adapters::pie::render(
                self,
                &effective_theme,
                prep.title.as_deref(),
                show_data,
                inline_title.as_deref(),
                &prep.lines,
            ),
            DiagramKind::Gantt => {
                adapters::gantt::render(self, &effective_theme, prep.title.as_deref(), &prep.lines)
            }
            DiagramKind::Sankey => {
                adapters::sankey::render(self, &effective_theme, prep.title.as_deref(), &prep.lines)
            }
            DiagramKind::XyChart { horizontal } => adapters::xychart::render(
                self,
                &effective_theme,
                prep.title.as_deref(),
                horizontal,
                &prep.lines,
            ),
        }
    }

    pub(crate) fn has_custom_background(&self) -> bool {
        if self.explicit_background {
            return true;
        }
        // Detect builder overrides (`chart.background_color = ...`) against the
        // theme filled at construction time.
        let orig_theme = self
            .theme_name
            .as_deref()
            .map(get_theme)
            .unwrap_or_else(|| get_theme(&get_default_theme_name()));
        self.base.background_color != orig_theme.background_color
    }

    pub(crate) fn resolved_font_family(&self, theme: &Theme) -> String {
        if !self.base.font_family.is_empty() {
            self.base.font_family.clone()
        } else if !theme.font_family.is_empty() {
            theme.font_family.clone()
        } else {
            DEFAULT_FONT_FAMILY.to_string()
        }
    }
}

/// Renders a Mermaid diagram source string directly to SVG using the default theme.
#[cfg_attr(docsrs, doc(cfg(feature = "mermaid")))]
pub fn mermaid_to_svg(source: &str) -> Result<String> {
    MermaidChart::new(source).svg()
}

/// Renders a Mermaid diagram source string directly to SVG using the specified theme.
#[cfg_attr(docsrs, doc(cfg(feature = "mermaid")))]
pub fn mermaid_to_svg_with_theme(source: &str, theme: &str) -> Result<String> {
    check_theme_name(theme)?;
    MermaidChart::new_with_theme(source, theme).svg()
}

pub(crate) fn compute_header_height(
    chart: &MermaidChart,
    title_text: Option<&str>,
    theme: &Arc<Theme>,
) -> f32 {
    let mut h = 0.0_f32;
    if let Some(t) = title_text
        && !t.is_empty()
    {
        let m = chart.base.title.margin.unwrap_or_default();
        let row_h = if chart.base.title.height > 0.0 {
            chart.base.title.height
        } else {
            theme.title.height.max(30.0)
        };
        h += m.top + row_h + m.bottom;
    }
    if !chart.base.sub_title.text.is_empty() {
        let m = chart.base.sub_title.margin.unwrap_or_default();
        let row_h = if chart.base.sub_title.height > 0.0 {
            chart.base.sub_title.height
        } else {
            theme.sub_title.height.max(20.0)
        };
        h += m.top + row_h + m.bottom;
    }
    h
}

pub(crate) fn draw_header(
    canvas: &mut Canvas,
    chart: &MermaidChart,
    title_text: Option<&str>,
    theme: &Arc<Theme>,
    content_w: f32,
) {
    let font_family = chart.resolved_font_family(theme);
    let inner_w = (content_w - chart.base.margin.left - chart.base.margin.right).max(40.0);
    let mut cursor_y = chart.base.margin.top;

    let place_x = |align: &super::common::Align, text: &str, size: f32| -> f32 {
        let w = measure_text_width_family(&font_family, size, text)
            .map(|b| b.width())
            .unwrap_or_default();
        match align {
            super::common::Align::Center => (inner_w - w) * 0.5,
            super::common::Align::Right => (inner_w - w).max(0.0),
            super::common::Align::Left => 0.0,
        }
    };

    if let Some(t) = title_text
        && !t.is_empty()
    {
        let m = chart.base.title.margin.unwrap_or_default();
        let size = if chart.base.title.font.size > 0.0 {
            chart.base.title.font.size
        } else {
            theme.title.font.size
        };
        let color = if !chart.base.title.font.color.is_zero() {
            chart.base.title.font.color
        } else {
            theme.title.font.color
        };
        let weight = chart
            .base
            .title
            .font
            .weight
            .clone()
            .or_else(|| theme.title.font.weight.clone());
        let row_h = if chart.base.title.height > 0.0 {
            chart.base.title.height
        } else {
            theme.title.height.max(30.0)
        };
        let clipped = text_ellipsis(
            &font_family,
            size,
            t,
            (inner_w - m.left - m.right).max(20.0),
        );
        let x = chart.base.margin.left + m.left + place_x(&chart.base.title.align, &clipped, size);
        canvas.text_unmeasured(Text {
            text: clipped,
            font_family: Some(font_family.clone()),
            font_size: Some(size),
            font_color: Some(color),
            font_weight: weight,
            x: Some(x),
            y: Some(cursor_y + m.top + row_h * 0.5),
            dominant_baseline: Some("central".to_string()),
            ..Default::default()
        });
        cursor_y += m.top + row_h + m.bottom;
    }

    if !chart.base.sub_title.text.is_empty() {
        let m = chart.base.sub_title.margin.unwrap_or_default();
        let size = if chart.base.sub_title.font.size > 0.0 {
            chart.base.sub_title.font.size
        } else {
            theme.sub_title.font.size
        };
        let color = if !chart.base.sub_title.font.color.is_zero() {
            chart.base.sub_title.font.color
        } else {
            theme.sub_title.font.color
        };
        let weight = chart
            .base
            .sub_title
            .font
            .weight
            .clone()
            .or_else(|| theme.sub_title.font.weight.clone());
        let row_h = if chart.base.sub_title.height > 0.0 {
            chart.base.sub_title.height
        } else {
            theme.sub_title.height.max(20.0)
        };
        let clipped = text_ellipsis(
            &font_family,
            size,
            &chart.base.sub_title.text,
            (inner_w - m.left - m.right).max(20.0),
        );
        let x =
            chart.base.margin.left + m.left + place_x(&chart.base.sub_title.align, &clipped, size);
        canvas.text_unmeasured(Text {
            text: clipped,
            font_family: Some(font_family),
            font_size: Some(size),
            font_color: Some(color),
            font_weight: weight,
            x: Some(x),
            y: Some(cursor_y + m.top + row_h * 0.5),
            dominant_baseline: Some("central".to_string()),
            ..Default::default()
        });
    }
}

pub(crate) fn create_scaled_canvas(
    chart: &MermaidChart,
    content_w: f32,
    content_h: f32,
) -> (Canvas, f32, f32) {
    let cw = content_w.max(1.0);
    let ch = content_h.max(1.0);
    let req_w = chart.base.width;
    let req_h = chart.base.height;

    let (out_w, out_h, view_box) = match (req_w > 0.0, req_h > 0.0) {
        (false, false) => (cw, ch, None),
        (true, false) => (req_w, (ch * (req_w / cw)).max(1.0), Some((cw, ch))),
        (false, true) => ((cw * (req_h / ch)).max(1.0), req_h, Some((cw, ch))),
        (true, true) => (req_w, req_h, Some((cw, ch))),
    };

    let mut canvas = Canvas::new_width_xy(out_w, out_h, chart.base.x, chart.base.y);
    canvas.compact = chart.base.compact;
    canvas.view_box = view_box;
    let texts: &[TextConfig] = &chart.base.texts;
    if !texts.is_empty() {
        canvas.set_texts(overlay_texts(
            texts,
            &chart.base.font_family,
            chart.base.sub_title.font.color,
        ));
    }
    (canvas, out_w, out_h)
}
