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

//! The JSON reference (`docs/json.md`) read as data.
//!
//! The reference is the single source of the descriptions, defaults and
//! examples of the JSON options; the tests in `schema.rs` keep its tables in
//! step with the validation tables. [`chart_json_schema`](super::chart_json_schema)
//! takes what the reference says from here, so a key documented there is
//! documented in the JSON Schema as well.
//!
//! The page is laid out for this:
//!
//! - `<!-- keys: path -->` precedes the table of the keys of `path`
//!   (`base`, `bar`, `base.series_list`, `radar.indicators`, …);
//!   `<!-- keys: path = other -->` says `path` has the keys of `other`.
//! - A table row is `` | `key` | type | default | description | ``.
//! - `<!-- example: chart -->` precedes a fenced JSON example of `chart`.
//! - The paragraph that introduces a chart starts with
//!   `` `XxxChart::from_json` — ``.

use std::collections::HashMap;
use std::sync::OnceLock;

static PAGE: &str = include_str!("../../docs/json.md");

/// One documented key.
pub(crate) struct Row {
    /// The default column, as written (backticks included).
    pub default: &'static str,
    /// The description column, as written.
    pub description: &'static str,
}

#[derive(Default)]
pub(crate) struct Reference {
    sections: HashMap<&'static str, Vec<(&'static str, Row)>>,
    aliases: HashMap<&'static str, &'static str>,
    intros: HashMap<&'static str, &'static str>,
    examples: HashMap<&'static str, &'static str>,
}

/// The parsed JSON reference.
pub(crate) fn reference() -> &'static Reference {
    static REFERENCE: OnceLock<Reference> = OnceLock::new();
    REFERENCE.get_or_init(|| parse(PAGE))
}

fn parse(page: &'static str) -> Reference {
    let mut reference = Reference::default();
    let mut section: Option<&'static str> = None;
    let mut pending_intro: Option<&'static str> = None;
    let mut lines = page.lines();
    while let Some(line) = lines.next() {
        if let Some(marker) = line
            .strip_prefix("<!-- keys: ")
            .and_then(|l| l.strip_suffix(" -->"))
        {
            match marker.split_once(" = ") {
                Some((path, other)) => {
                    // Usually right below the marker of `other`, whose
                    // table follows: the section stays.
                    reference.aliases.insert(path, other);
                }
                None => {
                    reference.sections.entry(marker).or_default();
                    section = Some(marker);
                }
            }
            if let Some(intro) = pending_intro.take() {
                reference.intros.insert(chart_of(marker), intro);
            }
        } else if let Some(chart) = line
            .strip_prefix("<!-- example: ")
            .and_then(|l| l.strip_suffix(" -->"))
        {
            section = None;
            if let Some(intro) = pending_intro.take() {
                reference.intros.insert(chart, intro);
            }
            // The fenced block that follows, as a slice of the page.
            let (mut start, mut end) = (None, None);
            for line in lines.by_ref() {
                let offset = line.as_ptr() as usize - page.as_ptr() as usize;
                if start.is_none() {
                    if line.starts_with("```json") {
                        start = Some(offset + line.len());
                    }
                } else if line.starts_with("```") {
                    end = Some(offset);
                    break;
                }
            }
            if let (Some(start), Some(end)) = (start, end) {
                reference.examples.insert(chart, page[start..end].trim());
            }
        } else if line.starts_with('`')
            && line.contains("::from_json`")
            && let Some((_, intro)) = line.split_once(" — ")
        {
            pending_intro = Some(intro);
        } else if let Some(section) = section
            && let Some(row) = line.strip_prefix("| `")
            && let Some((key, rest)) = row.split_once('`')
        {
            let cells: Vec<&'static str> = rest
                .trim()
                .trim_start_matches('|')
                .trim_end_matches('|')
                .splitn(3, '|')
                .map(str::trim)
                .collect();
            if let [_, default, description] = cells[..] {
                reference.sections.entry(section).or_default().push((
                    key,
                    Row {
                        default,
                        description,
                    },
                ));
            }
        }
    }
    reference
}

/// `bar` for `bar`, `bar.series_list`, …
fn chart_of(marker: &str) -> &str {
    marker.split('.').next().unwrap_or(marker)
}

impl Reference {
    fn section(&self, path: &str) -> Option<&[(&'static str, Row)]> {
        let path = self.aliases.get(path).copied().unwrap_or(path);
        self.sections.get(path).map(Vec::as_slice)
    }

    /// The row of `key` inside the object at `parent` (a dotted path such as
    /// `series_list.mark_lines`, empty for the top level) of `chart`: the
    /// chart's own table first, the common options after.
    pub fn row(&self, chart: &str, parent: &str, key: &str) -> Option<&Row> {
        let candidates = if parent.is_empty() {
            [chart.to_string(), "base".to_string()]
        } else {
            [format!("{chart}.{parent}"), format!("base.{parent}")]
        };
        candidates.iter().find_map(|path| {
            self.section(path)?
                .iter()
                .find(|(k, _)| *k == key)
                .map(|(_, row)| row)
        })
    }

    /// The sentence that introduces `chart`.
    pub fn intro(&self, chart: &str) -> Option<&'static str> {
        self.intros.get(chart).copied()
    }

    /// The JSON example of `chart`.
    pub fn example(&self, chart: &str) -> Option<&'static str> {
        self.examples.get(chart).copied()
    }
}

/// Markdown links `[text](target)` become `text`.
pub(crate) fn plain(markdown: &str) -> String {
    let mut out = String::with_capacity(markdown.len());
    let mut rest = markdown;
    while let Some(open) = rest.find('[') {
        let after = &rest[open + 1..];
        if let Some(close) = after.find("](")
            && let Some(end) = after[close + 2..].find(')')
        {
            out.push_str(&rest[..open]);
            out.push_str(&after[..close]);
            rest = &after[close + 2 + end + 1..];
        } else {
            out.push_str(&rest[..=open]);
            rest = after;
        }
    }
    out.push_str(rest);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_rows_aliases_intros_and_examples() {
        let r = reference();
        let row = r.row("bar", "", "width").unwrap();
        assert_eq!(row.default, "`600`");
        assert!(r.row("bar", "", "radius").is_some());
        assert!(r.row("bar", "series_list", "stack").is_some());
        // Through `<!-- keys: scatter.x_axis_config = base.y_axis_configs -->`.
        assert!(r.row("scatter", "x_axis_config", "axis_min").is_some());
        assert!(r.row("bar", "series_symbol", "type").is_some());
        assert!(r.row("bar", "", "no_such_key").is_none());
        assert!(r.intro("line").unwrap().starts_with("Lines over"));
        assert!(r.intro("mermaid").is_some());
        let example: serde_json::Value =
            serde_json::from_str(r.example("calendar").unwrap()).unwrap();
        assert!(example["data"].is_array());
    }

    #[test]
    fn strips_markdown_links() {
        assert_eq!(plain("See [`texts[]`](#texts)."), "See `texts[]`.");
        assert_eq!(plain("a [b] c"), "a [b] c");
    }
}
