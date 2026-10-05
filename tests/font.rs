mod common;

use charts_rs::{
    DEFAULT_FONT_DATA, DEFAULT_FONT_FAMILY, add_fonts, get_font_families, measure_text_width_family,
};

/// The box of a text at the sizes the charts write most of theirs in.
fn boxes(text: &str) -> String {
    [10.0, 12.0, 14.0, 18.0]
        .iter()
        .map(|size| {
            let b = measure_text_width_family(DEFAULT_FONT_FAMILY, *size, text).unwrap();
            format!("{}x{}", b.width(), b.height())
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// Every text of every chart is placed by these numbers: they are those
/// `fontdue` measured up to 1.x, and the snapshot holds them as they are.
#[test]
fn roboto_measurements() {
    let face = ttf_parser::Face::parse(DEFAULT_FONT_DATA, 0).unwrap();
    let mut codes = vec![];
    for subtable in face.tables().cmap.unwrap().subtables {
        if subtable.is_unicode() {
            subtable.codepoints(|code| codes.push(code));
        }
    }
    codes.sort_unstable();
    codes.dedup();
    assert!(codes.len() > 800, "{}", codes.len());

    let mut lines = vec!["# width x height at 10, 12, 14 and 18 pixels".to_string()];
    for c in codes.into_iter().filter_map(char::from_u32) {
        lines.push(format!("U+{:04X} {}", u32::from(c), boxes(&c.to_string())));
    }
    // Text, with characters the font has no glyph for, with characters that
    // take no room, and on several lines.
    for text in [
        "",
        "Hello World!",
        "2013/4/26",
        "1,234,567.89",
        "Search Engine: 33.3%",
        "AVAWA To. fi fl ffi",
        "邮件营销",
        "😀",
        "a\tb",
        "\u{0}\u{7F}",
        "a\nb",
        "a\n",
        "\na",
        "a\n\nbcd\nef",
        "wide line here\nx",
        "a\r\nb",
        "a\rb",
        "a\u{2028}b\u{85}c\u{0C}d",
    ] {
        lines.push(format!("{text:?} {}", boxes(text)));
    }
    common::assert_snapshot!("font/roboto_measure.txt", lines.join("\n") + "\n");
}

#[test]
fn text_on_lines_and_without_glyphs() {
    let measure = |text: &str| {
        let b = measure_text_width_family(DEFAULT_FONT_FAMILY, 14.0, text).unwrap();
        (b.width(), b.height())
    };
    assert_eq!((0.0, 0.0), measure(""));
    // A line break starts a line: as wide as the widest, as high as all.
    let (wide, narrow, both) = (measure("wide line"), measure("x"), measure("wide line\nx"));
    assert!((both.0 - wide.0).abs() <= 1.0, "{wide:?} {both:?}");
    assert!(
        narrow.0 < wide.0 && both.1 > wide.1 + 10.0,
        "{wide:?} {both:?}"
    );
    assert_eq!(both, measure("wide line\r\nx"));
    assert_eq!(measure("x\ny").1, measure("x\ry").1);
    // A break at the end starts no line.
    assert_eq!(measure("x").1, measure("x\n").1);
    // Control characters take no room.
    assert_eq!(measure("ab").0, measure("a\u{1}\u{7F}b").0);
    // A character without a glyph is measured as the glyph for those: every
    // one the same, and none nothing.
    assert_eq!(measure("中"), measure("文"));
    assert!(measure("中").0 > 0.0);
    assert_eq!(2.0 * measure("中中").0, 2.0 * measure("中文").0);
    // No size, no extent; whatever the size, no panic.
    assert_eq!(
        0.0,
        measure_text_width_family(DEFAULT_FONT_FAMILY, 0.0, "abc")
            .unwrap()
            .width()
    );
    for size in [-14.0, f32::NAN, f32::INFINITY, f32::MAX, 1e-30] {
        measure_text_width_family(DEFAULT_FONT_FAMILY, size, "abc\ndef").unwrap();
    }
    // A family that is not registered is measured with the default font.
    assert_eq!(measure("Hello World!"), {
        let b = measure_text_width_family("No Such Family", 14.0, "Hello World!").unwrap();
        (b.width(), b.height())
    });
}

#[test]
fn fonts_are_registered_by_their_name() {
    assert!(
        get_font_families()
            .unwrap()
            .contains(&DEFAULT_FONT_FAMILY.to_string())
    );
    let before = measure_text_width_family(DEFAULT_FONT_FAMILY, 14.0, "Hello World!").unwrap();
    // The embedded font again: the same family, measured the same.
    add_fonts(&[DEFAULT_FONT_DATA]).unwrap();
    assert_eq!(
        1,
        get_font_families()
            .unwrap()
            .iter()
            .filter(|family| family.as_str() == DEFAULT_FONT_FAMILY)
            .count()
    );
    let after = measure_text_width_family(DEFAULT_FONT_FAMILY, 14.0, "Hello World!").unwrap();
    assert_eq!(
        (before.width(), before.height()),
        (after.width(), after.height())
    );

    // What is not a font is refused.
    for data in [&b"not a font"[..], &[], &DEFAULT_FONT_DATA[..100]] {
        let message = match add_fonts(&[DEFAULT_FONT_DATA, data]) {
            Ok(()) => panic!("accepted"),
            Err(e) => e.to_string(),
        };
        assert!(message.starts_with("Error parse font: "), "{message}");
    }
}

// ── Fonts made of the embedded one ──────────────────────────────────────────

/// Where a table of a font is: its offset and its length.
fn table(font: &[u8], tag: &[u8; 4]) -> (usize, usize) {
    let u32_at = |at: usize| u32::from_be_bytes(font[at..at + 4].try_into().unwrap()) as usize;
    let count = u16::from_be_bytes([font[4], font[5]]) as usize;
    (0..count)
        .map(|i| 12 + i * 16)
        .find(|record| &font[*record..*record + 4] == tag)
        .map(|record| (u32_at(record + 8), u32_at(record + 12)))
        .unwrap()
}

/// The embedded font under another name of six letters.
fn renamed(name: &str) -> Vec<u8> {
    assert_eq!(6, name.len());
    let utf16 = |text: &str| -> Vec<u8> { text.bytes().flat_map(|b| [0, b]).collect() };
    let (from, to) = (utf16("Roboto"), utf16(name));
    let mut font = DEFAULT_FONT_DATA.to_vec();
    let (offset, length) = table(&font, b"name");
    let mut at = offset;
    while at + from.len() <= offset + length {
        if font[at..at + from.len()] == from[..] {
            font[at..at + to.len()].copy_from_slice(&to);
            at += from.len();
        } else {
            at += 1;
        }
    }
    font
}

fn width(family: &str, text: &str) -> f32 {
    measure_text_width_family(family, 14.0, text)
        .unwrap()
        .width()
}

#[test]
fn a_font_is_registered_by_its_family() {
    add_fonts(&[&renamed("Tester")]).unwrap();
    let families = get_font_families().unwrap();
    assert!(families.contains(&"Tester".to_string()), "{families:?}");
    assert_eq!(
        width("Roboto", "Hello World!"),
        width("Tester", "Hello World!")
    );
}

#[test]
fn a_font_named_in_mac_roman_only_is_registered() {
    // The name of the family, moved from the Unicode names of the font to
    // one in Mac Roman, written over its copyright notice.
    let mut font = renamed("Unused");
    let (name, _) = table(&font, b"name");
    let u16_at = |font: &[u8], at: usize| u16::from_be_bytes([font[at], font[at + 1]]) as usize;
    let (count, storage) = (u16_at(&font, name + 2), u16_at(&font, name + 4));
    let record = |id: usize| -> usize {
        (0..count)
            .map(|i| name + 6 + i * 12)
            .find(|record| u16_at(&font, *record + 6) == id)
            .unwrap()
    };
    let (family, copyright) = (record(1), record(0));
    let text = name + storage + u16_at(&font, copyright + 10);
    font[text..text + 6].copy_from_slice(b"Macish");
    // Platform 1 (Macintosh), encoding 0 (Roman), language 0 (English),
    // six bytes at the offset of the notice.
    let offset = [font[copyright + 10], font[copyright + 11]];
    font[family..family + 6].copy_from_slice(&[0, 1, 0, 0, 0, 0]);
    font[family + 8..family + 10].copy_from_slice(&[0, 6]);
    font[family + 10..family + 12].copy_from_slice(&offset);

    add_fonts(&[&font]).unwrap();
    let families = get_font_families().unwrap();
    assert!(families.contains(&"Macish".to_string()), "{families:?}");
    assert_eq!(
        width("Roboto", "Hello World!"),
        width("Macish", "Hello World!")
    );
}

#[test]
fn a_symbol_font_is_measured_by_its_own_table() {
    // Both tables of characters said to be those of a symbol font: none of
    // them is one of Unicode any more.
    let mut font = renamed("Symbls");
    let (cmap, _) = table(&font, b"cmap");
    let count = u16::from_be_bytes([font[cmap + 2], font[cmap + 3]]) as usize;
    for record in (0..count).map(|i| cmap + 4 + i * 8) {
        font[record..record + 4].copy_from_slice(&[0, 3, 0, 0]);
    }
    add_fonts(&[&font]).unwrap();
    assert_eq!(
        width("Roboto", "Hello World!"),
        width("Symbls", "Hello World!")
    );
    // Not the one box that every character without a glyph is measured as.
    assert!(width("Symbls", "WWWWW") > 2.0 * width("Symbls", "iiiii"));
    assert_eq!(width("Symbls", "中中中中中"), width("Symbls", "文文文文文"));
}

#[test]
fn the_regular_face_of_a_family_is_measured_with() {
    // A bold face of the family, twice as large so that it shows.
    let bold = |name: &str| -> Vec<u8> {
        let mut font = renamed(name);
        let (os2, _) = table(&font, b"OS/2");
        font[os2 + 4..os2 + 6].copy_from_slice(&700_u16.to_be_bytes());
        let (head, _) = table(&font, b"head");
        font[head + 18..head + 20].copy_from_slice(&1024_u16.to_be_bytes());
        font
    };
    let regular = width("Roboto", "Hello World!");

    // Whichever comes first in a call.
    add_fonts(&[&bold("Weight"), &renamed("Weight")]).unwrap();
    assert_eq!(regular, width("Weight", "Hello World!"));
    add_fonts(&[&renamed("Heavyw"), &bold("Heavyw")]).unwrap();
    assert_eq!(regular, width("Heavyw", "Hello World!"));

    // A later call replaces the family, as it always did.
    add_fonts(&[&bold("Weight")]).unwrap();
    assert!(width("Weight", "Hello World!") > regular * 1.9);
    // The only face of a family is the one, whatever its weight.
    add_fonts(&[&bold("Onlybd")]).unwrap();
    assert!(width("Onlybd", "Hello World!") > regular * 1.9);
}
