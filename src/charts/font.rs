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

use arc_swap::ArcSwap;

use super::util::*;
use fontdue::Font;
use fontdue::layout::{CoordinateSystem, Layout, TextStyle};
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, OnceLock};

// Crate-level error/result (see `error.rs`); re-exported to keep `font::Error`.
pub use super::error::{Error, Result};

pub static DEFAULT_FONT_FAMILY: &str = "Roboto";
pub static DEFAULT_FONT_DATA: &[u8] = include_bytes!("../Roboto.ttf");

struct FontRegistry {
    fonts: HashMap<String, Arc<Font>>,
    // Raw bytes of every registered font; the raster fontdb (image-encoder)
    // needs the original data to rebuild itself when fonts change.
    datas: Vec<Vec<u8>>,
}

// Bumped whenever the registry changes so the per-thread measurement caches
// drop entries computed against the previous fonts.
static FONT_GENERATION: AtomicU64 = AtomicU64::new(0);

// fontdue reports parse failures as plain `&str` messages.
fn parse_font_error(message: &str) -> Error {
    Error::ParseFont {
        message: message.to_string(),
    }
}

fn get_family_from_font(font: &Font) -> String {
    let Some(name) = font.name() else {
        return String::new();
    };
    // Strip font-weight words so e.g. "Roboto Bold" registers as "Roboto".
    // https://developer.mozilla.org/en-US/docs/Web/CSS/font-weight
    let mut family = name.to_string();
    for weight in ["Thin", "Light", "Regular", "Medium", "Bold"] {
        if family.contains(weight) {
            family = family.replace(weight, "");
        }
    }
    if let Some(value) = family.strip_suffix("Black") {
        family = value.to_string();
    }
    family.trim().to_string()
}

fn global_fonts() -> Result<&'static ArcSwap<FontRegistry>> {
    static GLOBAL_FONTS: OnceLock<ArcSwap<FontRegistry>> = OnceLock::new();
    if let Some(cell) = GLOBAL_FONTS.get() {
        return Ok(cell);
    }
    // Build outside the cell: std's `OnceLock` has no stable `get_or_try_init`,
    // so a font-parse failure is propagated here before anything is stored.
    let font = Font::from_bytes(DEFAULT_FONT_DATA, fontdue::FontSettings::default())
        .map_err(parse_font_error)?;
    let mut fonts = HashMap::new();
    fonts.insert(DEFAULT_FONT_FAMILY.to_string(), Arc::new(font));
    let registry = FontRegistry {
        fonts,
        datas: vec![DEFAULT_FONT_DATA.to_vec()],
    };
    // A concurrent caller may have initialized first; keep whichever won.
    Ok(GLOBAL_FONTS.get_or_init(|| ArcSwap::from_pointee(registry)))
}

/// Registers fonts (TTF/OTF data); the family name is read from the font
/// itself. Fonts can be added at any time — new fonts take effect for
/// subsequent renders, replacing any font already registered under the same
/// family.
pub fn add_fonts(fonts: &[&[u8]]) -> Result<()> {
    // Parse up front so errors surface before the registry is touched.
    let mut parsed = Vec::with_capacity(fonts.len());
    for data in fonts.iter() {
        let font =
            Font::from_bytes(*data, fontdue::FontSettings::default()).map_err(parse_font_error)?;
        let family = get_family_from_font(&font);
        if !family.is_empty() {
            parsed.push((family, Arc::new(font), data.to_vec()));
        }
    }
    let cell = global_fonts()?;
    if parsed.is_empty() {
        return Ok(());
    }
    cell.rcu(|current| {
        let mut fonts = current.fonts.clone();
        let mut datas = current.datas.clone();
        for (family, font, data) in parsed.iter() {
            fonts.insert(family.clone(), font.clone());
            datas.push(data.clone());
        }
        FontRegistry { fonts, datas }
    });
    FONT_GENERATION.fetch_add(1, Ordering::Relaxed);
    #[cfg(feature = "raster")]
    super::encoder::rebuild_fontdb(&cell.load().datas);
    Ok(())
}

/// Registers fonts once (legacy API).
#[deprecated(note = "use `add_fonts`, which can register fonts at any time")]
pub fn get_or_try_init_fonts(fonts: Option<Vec<&[u8]>>) -> Result<()> {
    if let Some(value) = fonts {
        add_fonts(&value)
    } else {
        global_fonts().map(|_| ())
    }
}

#[cfg(feature = "raster")]
pub(crate) fn registered_font_datas() -> Vec<Vec<u8>> {
    global_fonts()
        .map(|cell| cell.load().datas.clone())
        .unwrap_or_default()
}

/// Gets font by font family.
pub fn get_font(name: &str) -> Result<Arc<Font>> {
    let registry = global_fonts()?.load();
    if let Some(font) = registry
        .fonts
        .get(name)
        .or_else(|| registry.fonts.get(DEFAULT_FONT_FAMILY))
    {
        Ok(font.clone())
    } else {
        Err(Error::FontNotFound {
            name: name.to_string(),
        })
    }
}
/// Gets all supported font family
pub fn get_font_families() -> Result<Vec<String>> {
    let registry = global_fonts()?.load();
    let mut families: Vec<String> = registry.fonts.keys().cloned().collect();
    families.sort_unstable();
    Ok(families)
}

thread_local! {
    // One reusable layout per thread: `clear()` keeps the internal buffers, so
    // the per-measurement `Layout` allocations disappear after warm-up.
    static MEASURE_LAYOUT: std::cell::RefCell<Layout> =
        std::cell::RefCell::new(Layout::new(CoordinateSystem::PositiveYDown));
}

fn glyphs_extent(layout: &Layout) -> Box {
    let mut right = 0.0_f32;
    let mut bottom = 0.0_f32;
    for g in layout.glyphs().iter() {
        let x = g.x + g.width as f32;
        let y = g.y + g.height as f32;
        if x > right {
            right = x;
        }
        if y > bottom {
            bottom = y;
        }
    }
    Box {
        right,
        bottom,
        ..Default::default()
    }
}

/// Measures the display area of text of a specified font size.
pub fn measure_text(font: &Font, font_size: f32, text: &str) -> Box {
    MEASURE_LAYOUT.with(|l| {
        let mut layout = l.borrow_mut();
        layout.clear();
        layout.append(&[font], &TextStyle::new(text, font_size, 0));
        glyphs_extent(&layout)
    })
}

// Upper bound for the measurement memo below; charts re-measure the same
// labels constantly (legend layout measures twice, axis ticks repeat), so a
// small per-thread cache removes most glyph-layout work. Cleared when full to
// stay bounded for long-running processes with ever-changing texts.
const MEASURE_CACHE_LIMIT: usize = 4096;

// (family, font-size bits, text) → (right, bottom)
type MeasureCacheMap = HashMap<(String, u32, String), (f32, f32)>;

/// Measures the display area of text of a specified font size and font family.
pub fn measure_text_width_family(font_family: &str, font_size: f32, text: &str) -> Result<Box> {
    thread_local! {
        // (font generation, measurements) — the generation detects font
        // registry changes that would invalidate cached widths.
        static MEASURE_CACHE: std::cell::RefCell<(u64, MeasureCacheMap)> =
            std::cell::RefCell::new((0, HashMap::new()));
    }
    let generation = FONT_GENERATION.load(Ordering::Relaxed);
    let key = (
        font_family.to_string(),
        font_size.to_bits(),
        text.to_string(),
    );
    if let Some(b) = MEASURE_CACHE.with(|c| {
        let mut cache = c.borrow_mut();
        if cache.0 != generation {
            cache.1.clear();
            cache.0 = generation;
        }
        cache.1.get(&key).copied()
    }) {
        return Ok(Box {
            right: b.0,
            bottom: b.1,
            ..Default::default()
        });
    }
    let font = get_font(font_family)?;
    let b = measure_text(&font, font_size, text);
    MEASURE_CACHE.with(|c| {
        let mut cache = c.borrow_mut();
        if cache.0 == generation {
            if cache.1.len() >= MEASURE_CACHE_LIMIT {
                cache.1.clear();
            }
            cache.1.insert(key, (b.right, b.bottom));
        }
    });
    Ok(b)
}

/// Gets the max width of multi text.
pub fn measure_max_text_width_family(
    font_family: &str,
    font_size: f32,
    texts: Vec<&str>,
) -> Result<Box> {
    let font = get_font(font_family)?;
    let mut result = Box::default();
    for item in texts.iter() {
        let b = measure_text(&font, font_size, item);
        if b.width() > result.width() {
            result = b;
        }
    }
    Ok(result)
}

/// Cuts the text wrap fix size to muli text list.
pub fn text_wrap_fit(
    font_family: &str,
    font_size: f32,
    text: &str,
    width: f32,
) -> Result<Vec<String>> {
    let font = get_font(font_family)?;
    let b = measure_text(&font, font_size, text);
    if b.width() <= width {
        return Ok(vec![text.to_string()]);
    }

    // Append char by char into one persistent layout instead of re-laying out
    // every growing prefix (O(n²) glyph layouts → O(n)). The layout applies no
    // kerning, so incremental appends match a fresh layout of the same prefix.
    let mut layout = Layout::new(CoordinateSystem::PositiveYDown);
    let fonts = std::slice::from_ref(&font);
    let mut buf = [0u8; 4];
    let mut current = String::new();
    let mut result = vec![];
    for item in text.chars() {
        layout.append(
            fonts,
            &TextStyle::new(item.encode_utf8(&mut buf), font_size, 0),
        );
        if glyphs_extent(&layout).width() > width {
            result.push(current);
            current = String::from(item);
            // Start the next line's measurement from scratch.
            layout.clear();
            layout.append(
                fonts,
                &TextStyle::new(item.encode_utf8(&mut buf), font_size, 0),
            );
            continue;
        }
        current.push(item);
    }
    if !current.is_empty() {
        result.push(current);
    }
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::{get_font, get_font_families, measure_text_width_family, text_wrap_fit};
    use pretty_assertions::assert_eq;
    #[test]
    fn measure_text() {
        let name = "Roboto";
        get_font(name).unwrap();

        let str = "Hello World!";
        let b = measure_text_width_family(name, 14.0, str).unwrap();

        assert_eq!(79.0, b.width().ceil());
        assert_eq!(14.0, b.height());

        assert_eq!("Roboto", get_font_families().unwrap().join(","));
    }
    #[test]
    fn wrap_fit() {
        let name = "Roboto";
        let result = text_wrap_fit(name, 14.0, "An event-driven, non-blocking I/O platform for writing asynchronous I/O backed applications", 100.0).unwrap();
        assert_eq!(
            vec![
                "An event-drive",
                "n, non-blocking ",
                "I/O platform fo",
                "r writing async",
                "hronous I/O ba",
                "cked applicati",
                "ons",
            ],
            result
        );
    }
}
