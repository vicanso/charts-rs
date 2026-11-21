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

//! Adapters that translate data-oriented Mermaid diagrams (`pie`, `gantt`,
//! `sankey-beta`, `xychart-beta`) into existing `charts-rs` chart types.

pub(crate) mod gantt;
pub(crate) mod pie;
pub(crate) mod radar;
pub(crate) mod sankey;
pub(crate) mod treemap;
pub(crate) mod xychart;

use super::super::base::ChartBase;
use super::MermaidChart;

/// Copies `MermaidChart`'s shared options (size, position, margins, font,
/// background override, title, sub-title, overlay texts and `compact`) onto a
/// target chart's [`ChartBase`].
pub(crate) fn apply_base_options(
    src: &MermaidChart,
    dst: &mut ChartBase,
    parsed_title: Option<&str>,
    default_width: f32,
    default_height: f32,
) {
    dst.width = if src.base.width > 0.0 {
        src.base.width
    } else {
        default_width
    };
    dst.height = if src.base.height > 0.0 {
        src.base.height
    } else {
        default_height
    };
    dst.x = src.base.x;
    dst.y = src.base.y;
    dst.margin = src.base.margin;
    dst.compact = src.base.compact;

    if !src.base.font_family.is_empty() {
        dst.font_family = src.base.font_family.clone();
    }
    if src.has_custom_background() {
        dst.background_color = src.base.background_color;
    }

    if !src.base.title.text.is_empty() {
        dst.title = src.base.title.clone();
    } else if let Some(t) = parsed_title
        && !t.is_empty()
    {
        dst.title.text = t.to_string();
        if src.base.title.font.size > 0.0 {
            dst.title.font = src.base.title.font.clone();
        }
    }

    if !src.base.sub_title.text.is_empty() {
        dst.sub_title = src.base.sub_title.clone();
    }
    if !src.base.texts.is_empty() {
        dst.texts = src.base.texts.clone();
    }
}
