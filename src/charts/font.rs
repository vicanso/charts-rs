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
use std::cell::RefCell;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, OnceLock};
use ttf_parser::{Face, GlyphId, OutlineBuilder};

// Crate-level error/result (see `error.rs`); re-exported to keep `font::Error`.
pub use super::error::{Error, Result};

/// The default font family (the embedded Roboto).
pub static DEFAULT_FONT_FAMILY: &str = "Roboto";
/// Raw TTF data of the embedded default font.
pub static DEFAULT_FONT_DATA: &[u8] = include_bytes!("../Roboto.ttf");

/// Raw font bytes shared between the registry and the raster `fontdb`
/// (image-encoder), so each font is held in memory once.
pub(crate) type FontData = Arc<dyn AsRef<[u8]> + Send + Sync>;

/// A registered font: its bytes, and the numbers every measurement starts
/// from. The glyphs are read from the bytes as they are first measured, so a
/// font costs no more memory than its file.
struct FontFace {
    // Tells the fonts apart in the per-thread glyph cache.
    id: u64,
    data: FontData,
    units_per_em: f32,
    // Of a line, in font units: from its top down to the baseline, and down
    // to the top of the next line.
    ascent: f32,
    new_line: f32,
}

impl FontFace {
    /// Reads a font: its family (empty when it has no name) and what it is
    /// measured with.
    fn new(data: FontData) -> Result<(String, FontFace)> {
        static NEXT_ID: AtomicU64 = AtomicU64::new(0);
        let (family, units_per_em, ascent, new_line) = {
            let face = Face::parse((*data).as_ref(), 0).map_err(|e| Error::ParseFont {
                message: e.to_string(),
            })?;
            let ascent = i32::from(face.ascender());
            let new_line = ascent - i32::from(face.descender()) + i32::from(face.line_gap());
            (
                get_family_from_face(&face),
                f32::from(face.units_per_em()),
                ascent as f32,
                new_line as f32,
            )
        };
        Ok((
            family,
            FontFace {
                id: NEXT_ID.fetch_add(1, Ordering::Relaxed),
                data,
                units_per_em,
                ascent,
                new_line,
            },
        ))
    }

    /// The glyph of a character, read from the font the first time a thread
    /// asks for it.
    fn glyph(&self, c: char) -> Glyph {
        let generation = FONT_GENERATION.load(Ordering::Relaxed);
        GLYPHS.with(|cell| {
            let mut cache = cell.borrow_mut();
            if cache.0 != generation {
                cache.1.clear();
                cache.0 = generation;
            }
            if let Some(glyph) = cache.1.get(&(self.id, c)) {
                return *glyph;
            }
            let glyph = self.read_glyph(c);
            if cache.1.len() >= GLYPH_CACHE_LIMIT {
                cache.1.clear();
            }
            cache.1.insert((self.id, c), glyph);
            glyph
        })
    }

    fn read_glyph(&self, c: char) -> Glyph {
        // The bytes were parsed when the font was registered.
        let Ok(face) = Face::parse((*self.data).as_ref(), 0) else {
            return Glyph::default();
        };
        // A character the font does not have is measured as its glyph for
        // those, the first one.
        let id = face.glyph_index(c).unwrap_or(GlyphId(0));
        if id.0 >= face.number_of_glyphs() {
            return Glyph::default();
        }
        let mut outline = OutlineBox::new(self.units_per_em);
        face.outline_glyph(id, &mut outline);
        let (xmin, ymin, width, height) = outline.finish();
        Glyph {
            advance: face.glyph_hor_advance(id).map(f32::from).unwrap_or(0.0),
            xmin,
            ymin,
            width,
            height,
        }
    }
}

/// What is measured of a glyph, in font units: how far on the next one
/// starts, and the box around its outline.
#[derive(Clone, Copy, Default)]
struct Glyph {
    advance: f32,
    xmin: f32,
    ymin: f32,
    width: f32,
    height: f32,
}

// Upper bound of the per-thread glyph cache below: far more than the
// characters of any chart, and cleared when full so that it stays bounded
// whatever is measured.
const GLYPH_CACHE_LIMIT: usize = 16384;

// The generation of the registry the glyphs were read at, and the glyphs by
// font and character.
type GlyphCache = (u64, HashMap<(u64, char), Glyph>);

thread_local! {
    // The glyphs a thread has read so far.
    static GLYPHS: RefCell<GlyphCache> = RefCell::new((0, HashMap::new()));
}

/// The box around the outline of a glyph. The text of every chart was placed
/// with the boxes of `fontdue`, which are kept to the bit: a curve is cut
/// into the lines it was cut into there, and a level line is left out as it
/// was (the algorithm is `fontdue`'s, MIT / Apache-2.0 / Zlib).
struct OutlineBox {
    // Twice the area of the triangle a curve may bulge by before it is cut.
    max_area: f32,
    start: (f32, f32),
    previous: (f32, f32),
    xmin: f32,
    xmax: f32,
    ymin: f32,
    ymax: f32,
    // Whether any line counted.
    lines: bool,
    // The pieces of the curve that is being cut: both ends, as a point and
    // as how far along the curve it is.
    pieces: Vec<[(f32, f32, f32); 2]>,
}

impl OutlineBox {
    fn new(units_per_em: f32) -> Self {
        // 3 pixels at 40 pixels to the em.
        OutlineBox {
            max_area: 3.0 * 2.0 * (units_per_em / 40.0),
            start: (0.0, 0.0),
            previous: (0.0, 0.0),
            xmin: f32::MAX,
            xmax: f32::MIN,
            ymin: f32::MAX,
            ymax: f32::MIN,
            lines: false,
            pieces: Vec::new(),
        }
    }

    fn line(&mut self, from: (f32, f32), to: (f32, f32)) {
        // Only whether they are exactly the same matters.
        if from.1.to_bits() == to.1.to_bits() {
            return;
        }
        self.lines = true;
        for (x, y) in [from, to] {
            if x < self.xmin {
                self.xmin = x;
            }
            if x > self.xmax {
                self.xmax = x;
            }
            if y < self.ymin {
                self.ymin = y;
            }
            if y > self.ymax {
                self.ymax = y;
            }
        }
    }

    /// Cuts a curve from the previous point to `end` into lines; `point`
    /// gives the point at a share of the way along it.
    fn curve(&mut self, end: (f32, f32), point: impl Fn(f32) -> (f32, f32)) {
        let start = self.previous;
        self.pieces.clear();
        self.pieces
            .push([(start.0, start.1, 0.0), (end.0, end.1, 1.0)]);
        while let Some([a, c]) = self.pieces.pop() {
            let t = (a.2 + c.2) * 0.5;
            let b = point(t);
            let area = (b.0 - a.0) * (c.1 - a.1) - (c.0 - a.0) * (b.1 - a.1);
            if area.abs() > self.max_area {
                self.pieces.push([a, (b.0, b.1, t)]);
                self.pieces.push([(b.0, b.1, t), c]);
            } else {
                self.line((a.0, a.1), (c.0, c.1));
            }
        }
        self.previous = end;
    }

    /// `(xmin, ymin, width, height)`.
    fn finish(self) -> (f32, f32, f32, f32) {
        if !self.lines {
            return (0.0, 0.0, 0.0, 0.0);
        }
        (
            self.xmin,
            self.ymin,
            self.xmax - self.xmin,
            self.ymax - self.ymin,
        )
    }
}

impl OutlineBuilder for OutlineBox {
    fn move_to(&mut self, x: f32, y: f32) {
        self.start = (x, y);
        self.previous = (x, y);
    }

    fn line_to(&mut self, x: f32, y: f32) {
        self.line(self.previous, (x, y));
        self.previous = (x, y);
    }

    fn quad_to(&mut self, x1: f32, y1: f32, x: f32, y: f32) {
        let (a, b, c) = (self.previous, (x1, y1), (x, y));
        self.curve(c, |t| {
            let tm = 1.0 - t;
            let (ka, kb, kc) = (tm * tm, 2.0 * tm * t, t * t);
            (
                ka * a.0 + kb * b.0 + kc * c.0,
                ka * a.1 + kb * b.1 + kc * c.1,
            )
        });
    }

    fn curve_to(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, x: f32, y: f32) {
        let (a, b, c, d) = (self.previous, (x1, y1), (x2, y2), (x, y));
        self.curve(d, |t| {
            let tm = 1.0 - t;
            let (ka, kb, kc, kd) = (
                tm * tm * tm,
                3.0 * (tm * tm) * t,
                3.0 * tm * (t * t),
                t * t * t,
            );
            (
                ka * a.0 + kb * b.0 + kc * c.0 + kd * d.0,
                ka * a.1 + kb * b.1 + kc * c.1 + kd * d.1,
            )
        });
    }

    fn close(&mut self) {
        if self.start != self.previous {
            self.line(self.previous, self.start);
        }
        self.previous = self.start;
    }
}

struct FontRegistry {
    fonts: HashMap<String, Arc<FontFace>>,
    // Raw bytes of every registered font; the raster fontdb (image-encoder)
    // needs the original data to rebuild itself when fonts change.
    datas: Vec<FontData>,
}

// Bumped whenever the registry changes so the per-thread measurement caches
// drop entries computed against the previous fonts.
static FONT_GENERATION: AtomicU64 = AtomicU64::new(0);

fn get_family_from_face(face: &Face) -> String {
    // The full name of the font, where it is given in Unicode.
    let Some(name) = face
        .names()
        .into_iter()
        .find(|name| name.name_id == ttf_parser::name_id::FULL_NAME && name.is_unicode())
        .and_then(|name| name.to_string())
    else {
        return String::new();
    };
    // Strip font-weight words so e.g. "Roboto Bold" registers as "Roboto".
    // https://developer.mozilla.org/en-US/docs/Web/CSS/font-weight
    let mut family = name;
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
    // The embedded font is referenced in place, not copied.
    let data: FontData = Arc::new(DEFAULT_FONT_DATA);
    let (_, font) = FontFace::new(data.clone())?;
    let mut fonts = HashMap::new();
    fonts.insert(DEFAULT_FONT_FAMILY.to_string(), Arc::new(font));
    let registry = FontRegistry {
        fonts,
        datas: vec![data],
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
        let data: FontData = Arc::new(data.to_vec());
        let (family, font) = FontFace::new(data.clone())?;
        if !family.is_empty() {
            parsed.push((family, Arc::new(font), data));
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

#[cfg(feature = "raster")]
pub(crate) fn registered_font_datas() -> Vec<FontData> {
    global_fonts()
        .map(|cell| cell.load().datas.clone())
        .unwrap_or_default()
}

/// Gets the font of a family, or the default one when it is not registered.
fn get_font(name: &str) -> Result<Arc<FontFace>> {
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

// What ends a line after the character before.
#[derive(Clone, Copy, PartialEq)]
enum LineEnd {
    None,
    // A line feed and the like: the next character starts a line.
    Always,
    // A carriage return: so does the next one, unless it is the line feed
    // of the pair.
    CarriageReturn,
}

/// The extent of text as it is laid out, one character after another. The
/// lines and the rounding to whole pixels are those of the layout of
/// `fontdue`, which the charts were measured with before: a glyph starts on
/// a whole pixel and its box is as many whole pixels as cover its outline.
struct Extent<'a> {
    font: &'a FontFace,
    scale: f32,
    // Of a line, in pixels.
    ascent: f32,
    new_line: f32,
    // Where the next glyph starts, and where its line did.
    position: f32,
    line_start: f32,
    // Of the line: its baseline, and how far below it the next line starts.
    baseline: f32,
    below: f32,
    line_end: LineEnd,
    count: usize,
    right: f32,
    bottom: f32,
}

impl<'a> Extent<'a> {
    fn new(font: &'a FontFace, font_size: f32) -> Self {
        let scale = font_size / font.units_per_em;
        let ascent = (font.ascent * scale).ceil();
        let new_line = (font.new_line * scale).ceil();
        // The first line is never less than nothing.
        let first_ascent = if ascent > 0.0 { ascent } else { 0.0 };
        let first_new_line = if new_line > 0.0 { new_line } else { 0.0 };
        Extent {
            font,
            scale,
            ascent,
            new_line,
            position: 0.0,
            line_start: 0.0,
            baseline: first_ascent,
            below: first_new_line - first_ascent,
            line_end: LineEnd::None,
            count: 0,
            right: 0.0,
            bottom: 0.0,
        }
    }

    fn push(&mut self, c: char) {
        let starts_line = match self.line_end {
            LineEnd::None => false,
            LineEnd::Always => true,
            LineEnd::CarriageReturn => c != '\n',
        };
        self.line_end = match c {
            '\n' | '\u{0B}' | '\u{0C}' | '\u{85}' | '\u{2028}' | '\u{2029}' => LineEnd::Always,
            '\r' => LineEnd::CarriageReturn,
            _ => LineEnd::None,
        };
        if starts_line && self.count > 0 {
            self.baseline += self.below;
            self.baseline += self.ascent;
            self.below = self.new_line - self.ascent;
            self.line_start = self.position;
        }
        self.count += 1;

        // A control character takes no room.
        let glyph = if matches!(c, '\0'..='\x1F' | '\x7F') {
            Glyph::default()
        } else {
            self.font.glyph(c)
        };
        let (xmin, ymin) = (glyph.xmin * self.scale, glyph.ymin * self.scale);
        let (width, height) = (glyph.width * self.scale, glyph.height * self.scale);
        // The box in whole pixels: what the outline starts into its first
        // pixel is added to its size before that is rounded up.
        let mut offset_x = (xmin + 0.0).fract();
        if offset_x.is_sign_negative() {
            offset_x += 1.0;
        }
        let mut offset_y = (1.0 - height.fract() - ymin.fract()).fract();
        if offset_y.is_sign_negative() {
            offset_y += 1.0;
        }
        let pixels = |size: f32| (size.ceil() as i32) as usize as f32;
        let x = (self.position + xmin).floor() + (0.0 - self.line_start);
        let y = (-height - ymin).floor() + self.baseline;
        let right = x + pixels(width + offset_x);
        let bottom = y + pixels(height + offset_y);
        if right > self.right {
            self.right = right;
        }
        if bottom > self.bottom {
            self.bottom = bottom;
        }
        self.position += (self.scale * glyph.advance).ceil();
    }

    fn width(&self) -> f32 {
        self.right
    }

    fn to_box(&self) -> Box {
        Box {
            right: self.right,
            bottom: self.bottom,
            ..Default::default()
        }
    }
}

/// Measures the display area of text of a specified font size.
fn measure_text(font: &FontFace, font_size: f32, text: &str) -> Box {
    let mut extent = Extent::new(font, font_size);
    for c in text.chars() {
        extent.push(c);
    }
    extent.to_box()
}

// Upper bound for the measurement memo below; charts re-measure the same
// labels constantly (legend layout measures twice, axis ticks repeat), so a
// small per-thread cache removes most glyph-layout work. Cleared when full to
// stay bounded for long-running processes with ever-changing texts.
const MEASURE_CACHE_LIMIT: usize = 4096;

struct CachedMeasure {
    family: String,
    size_bits: u32,
    text: String,
    right: f32,
    bottom: f32,
}

// generation, entry count, hash → entries (compared on hit, so a lookup
// borrows the caller's strings and allocates only on a miss).
type MeasureCache = (u64, usize, HashMap<u64, Vec<CachedMeasure>>);

fn hash_measure(family: &str, size_bits: u32, text: &str) -> u64 {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    family.hash(&mut hasher);
    size_bits.hash(&mut hasher);
    text.hash(&mut hasher);
    hasher.finish()
}

/// Measures the display area of text of a specified font size and font family.
pub fn measure_text_width_family(font_family: &str, font_size: f32, text: &str) -> Result<Box> {
    thread_local! {
        // The generation detects font registry changes that would invalidate
        // cached widths.
        static MEASURE_CACHE: std::cell::RefCell<MeasureCache> =
            std::cell::RefCell::new((0, 0, HashMap::new()));
    }
    let generation = FONT_GENERATION.load(Ordering::Relaxed);
    let size_bits = font_size.to_bits();
    let hash = hash_measure(font_family, size_bits, text);
    if let Some(b) = MEASURE_CACHE.with(|c| {
        let mut cache = c.borrow_mut();
        if cache.0 != generation {
            cache.1 = 0;
            cache.2.clear();
            cache.0 = generation;
        }
        cache.2.get(&hash).and_then(|entries| {
            entries
                .iter()
                .find(|entry| {
                    entry.size_bits == size_bits
                        && entry.family == font_family
                        && entry.text == text
                })
                .map(|entry| (entry.right, entry.bottom))
        })
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
            if cache.1 >= MEASURE_CACHE_LIMIT {
                cache.1 = 0;
                cache.2.clear();
            }
            cache.2.entry(hash).or_default().push(CachedMeasure {
                family: font_family.to_string(),
                size_bits,
                text: text.to_string(),
                right: b.right,
                bottom: b.bottom,
            });
            cache.1 += 1;
        }
    });
    Ok(b)
}

/// One cached width of a joined axis-label row.
struct LabelRowHit {
    check: u64,
    count: usize,
    bytes: usize,
    width: f32,
}

/// Width of `labels` joined by single spaces, after `formatter` (empty leaves
/// them unchanged). Memoized per thread as the width alone, so an axis with
/// thousands of categories does not rebuild or remeasure that string on every
/// render. The width matches measuring the joined string directly.
pub(crate) fn measure_label_row_width(
    font_family: &str,
    font_size: f32,
    formatter: &str,
    labels: &[String],
) -> Result<f32> {
    if labels.is_empty() {
        return Ok(0.0);
    }
    thread_local! {
        static ROW_CACHE: std::cell::RefCell<(u64, HashMap<u64, Vec<LabelRowHit>>)> =
            std::cell::RefCell::new((0, HashMap::new()));
    }
    let generation = FONT_GENERATION.load(Ordering::Relaxed);
    let (primary, check, count, bytes) =
        label_row_fingerprint(font_family, font_size.to_bits(), formatter, labels);
    if let Some(width) = ROW_CACHE.with(|c| {
        let mut cache = c.borrow_mut();
        if cache.0 != generation {
            cache.1.clear();
            cache.0 = generation;
        }
        cache.1.get(&primary).and_then(|hits| {
            hits.iter()
                .find(|hit| hit.check == check && hit.count == count && hit.bytes == bytes)
                .map(|hit| hit.width)
        })
    }) {
        return Ok(width);
    }
    let mut joined = String::with_capacity(bytes + count);
    for (i, label) in labels.iter().enumerate() {
        if i > 0 {
            joined.push(' ');
        }
        if formatter.is_empty() {
            joined.push_str(label);
        } else {
            joined.push_str(&format_string(label, formatter));
        }
    }
    let font = get_font(font_family)?;
    let width = measure_text(&font, font_size, &joined).width();
    ROW_CACHE.with(|c| {
        let mut cache = c.borrow_mut();
        if cache.0 != generation {
            return;
        }
        if cache.1.len() >= MEASURE_CACHE_LIMIT {
            cache.1.clear();
        }
        cache.1.entry(primary).or_default().push(LabelRowHit {
            check,
            count,
            bytes,
            width,
        });
    });
    Ok(width)
}

fn label_row_fingerprint(
    family: &str,
    size_bits: u32,
    formatter: &str,
    labels: &[String],
) -> (u64, u64, usize, usize) {
    let mut primary = std::collections::hash_map::DefaultHasher::new();
    family.hash(&mut primary);
    size_bits.hash(&mut primary);
    formatter.hash(&mut primary);
    let mut bytes = 0usize;
    for (i, label) in labels.iter().enumerate() {
        i.hash(&mut primary);
        label.hash(&mut primary);
        bytes += label.len();
    }
    let mut check = std::collections::hash_map::DefaultHasher::new();
    0xA5A5_u64.hash(&mut check);
    bytes.hash(&mut check);
    labels.len().hash(&mut check);
    formatter.hash(&mut check);
    for label in labels.iter().rev() {
        label.hash(&mut check);
    }
    (primary.finish(), check.finish(), labels.len(), bytes)
}

/// Gets the max width of multi text.
pub fn measure_max_text_width_family(
    font_family: &str,
    font_size: f32,
    texts: Vec<&str>,
) -> Result<Box> {
    let mut result = Box::default();
    for item in texts.iter() {
        let b = measure_text_width_family(font_family, font_size, item)?;
        if b.width() > result.width() {
            result = b;
        }
    }
    Ok(result)
}

/// Shortens `text` with an ellipsis so it is at most `max_width` wide at
/// `font_size`; text that already fits comes back unchanged.
pub(crate) fn text_ellipsis(
    font_family: &str,
    font_size: f32,
    text: &str,
    max_width: f32,
) -> String {
    let Ok(font) = get_font(font_family) else {
        return text.to_string();
    };
    if measure_text(&font, font_size, text).width() <= max_width {
        return text.to_string();
    }
    const ELLIPSIS: &str = "…";
    let chars: Vec<char> = text.chars().collect();
    // Binary search the longest prefix that fits together with the ellipsis.
    let (mut lo, mut hi) = (0_usize, chars.len());
    while lo < hi {
        let mid = (lo + hi).div_ceil(2);
        let candidate: String = chars[..mid]
            .iter()
            .copied()
            .chain(ELLIPSIS.chars())
            .collect();
        if measure_text(&font, font_size, &candidate).width() <= max_width {
            lo = mid;
        } else {
            hi = mid - 1;
        }
    }
    chars[..lo]
        .iter()
        .copied()
        .chain(ELLIPSIS.chars())
        .collect()
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

    // Append char by char into one running extent instead of re-measuring
    // every growing prefix (O(n²) glyph layouts → O(n)). No kerning is
    // applied, so the running extent matches a fresh one of the same prefix.
    let mut extent = Extent::new(&font, font_size);
    let mut current = String::new();
    let mut result = vec![];
    for item in text.chars() {
        extent.push(item);
        if extent.width() > width {
            result.push(current);
            current = String::from(item);
            // Start the next line's measurement from scratch.
            extent = Extent::new(&font, font_size);
            extent.push(item);
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
    use super::{
        GLYPH_CACHE_LIMIT, GLYPHS, get_font, get_font_families, measure_label_row_width,
        measure_text_width_family, text_wrap_fit,
    };
    use crate::format_string;
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
    fn glyphs_are_read_as_they_are_measured() {
        let font = get_font("Roboto").unwrap();
        // A space has no outline, and so no box; it still takes room.
        let space = font.glyph(' ');
        assert_eq!((0.0, 0.0), (space.width, space.height));
        assert!(space.advance > 0.0);
        // A letter stands on the baseline.
        let letter = font.glyph('H');
        assert!(letter.width > 0.0 && letter.height > 0.0 && letter.ymin == 0.0);
        assert!(letter.advance > letter.width);

        // The glyphs a thread has read are kept, up to a limit: more
        // characters than that (none of them in the font) do not grow it.
        let first = 0x4E00_u32;
        for code in first..first + GLYPH_CACHE_LIMIT as u32 + 100 {
            font.glyph(char::from_u32(code).unwrap());
        }
        let kept = GLYPHS.with(|cell| cell.borrow().1.len());
        assert!(kept > 0 && kept <= GLYPH_CACHE_LIMIT, "{kept}");
        // What was dropped is read again, the same.
        let again = font.glyph('H');
        assert_eq!(
            (letter.advance, letter.width, letter.height),
            (again.advance, again.width, again.height)
        );
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
    #[test]
    fn label_row_width_matches_joined_string() {
        let name = "Roboto";
        let labels: Vec<String> = ["Mon", "Tue", "Wednesday", "t10000"]
            .into_iter()
            .map(str::to_string)
            .collect();
        let joined = labels.join(" ");
        let expected = measure_text_width_family(name, 14.0, &joined)
            .unwrap()
            .width();
        let width = measure_label_row_width(name, 14.0, "", &labels).unwrap();
        assert_eq!(expected, width);
        assert_eq!(
            width,
            measure_label_row_width(name, 14.0, "", &labels).unwrap()
        );

        let formatter = "[{c}]";
        let formatted = labels
            .iter()
            .map(|label| format_string(label, formatter))
            .collect::<Vec<_>>()
            .join(" ");
        let expected = measure_text_width_family(name, 14.0, &formatted)
            .unwrap()
            .width();
        assert_eq!(
            expected,
            measure_label_row_width(name, 14.0, formatter, &labels).unwrap()
        );
    }
}
