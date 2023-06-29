use super::util::*;
use fontdue::layout::{CoordinateSystem, Layout, TextStyle};
use fontdue::Font;
use once_cell::sync::Lazy;
use snafu::{ResultExt, Snafu};
use std::{collections::HashMap, sync::Mutex, sync::MutexGuard};

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("Error unable get lock: {source}"))]
    UnableGetLock {
        source: std::sync::PoisonError<MutexGuard<'static, HashMap<String, Font>>>,
    },
    #[snafu(display("Error font:{name} not found"))]
    FontNotFound { name: String },
    #[snafu(display("Error parse font: {message}"))]
    ParseFont { message: String },
}

pub type Result<T, E = Error> = std::result::Result<T, E>;

pub static DEFAULT_FONT_FAMILY: &str = "Arial";

static GLOBAL_FONTS: Lazy<Mutex<HashMap<String, Font>>> = Lazy::new(|| {
    let mut m = HashMap::new();
    // 初始化字体
    // 失败时直接出错
    let font = include_bytes!("../Arial.ttf") as &[u8];
    let font = fontdue::Font::from_bytes(font, fontdue::FontSettings::default()).unwrap();
    m.insert(DEFAULT_FONT_FAMILY.to_string(), font);

    Mutex::new(m)
});

pub fn add_font(name: &str, data: &[u8]) -> Result<()> {
    let font =
        fontdue::Font::from_bytes(data, fontdue::FontSettings::default()).map_err(|str| {
            Error::ParseFont {
                message: str.to_string(),
            }
        })?;
    let mut m = GLOBAL_FONTS.lock().context(UnableGetLockSnafu)?;
    m.insert(name.to_string(), font);
    Ok(())
}

pub fn get_font(name: &str) -> Result<Font> {
    let m = GLOBAL_FONTS.lock().context(UnableGetLockSnafu)?;
    if let Some(font) = m.get(name) {
        Ok(font.clone())
    } else {
        FontNotFoundSnafu {
            name: name.to_string(),
        }
        .fail()
    }
}

pub fn measure_text(font: &Font, font_size: f32, text: &str) -> Box {
    let mut layout = Layout::new(CoordinateSystem::PositiveYDown);
    layout.append(&[font], &TextStyle::new(text, font_size, 0));

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

pub fn measure_text_width_family(font_family: &str, font_size: f32, text: &str) -> Result<Box> {
    let font = get_font(font_family)?;
    Ok(measure_text(&font, font_size, text))
}

pub fn measure_text_vertical_center(
    font_family: &str,
    font_size: f32,
    text: &str,
    line_height: f32,
) -> Result<f32> {
    let b = measure_text_width_family(font_family, font_size, text)?;
    let height = b.height();
    Ok((line_height - height) / 2.0 + height)
}

#[cfg(test)]
mod tests {
    use super::{add_font, get_font, measure_text_width_family};
    use pretty_assertions::assert_eq;
    #[test]
    fn measure_text() {
        let data = include_bytes!("../../src/Arial.ttf") as &[u8];
        let name = "custom";
        add_font(name, data).unwrap();

        get_font(name).unwrap();

        let str = "Hello World!";
        let b = measure_text_width_family(name, 14.0, str).unwrap();

        assert_eq!(81.0, b.width().ceil());
        assert_eq!(14.0, b.height());
    }
}
