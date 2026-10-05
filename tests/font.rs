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
