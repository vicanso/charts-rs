use super::util::*;
use fontdue::layout::{CoordinateSystem, Layout, TextStyle};
use fontdue::Font;
use once_cell::sync::OnceCell;
use snafu::Snafu;
use std::{collections::HashMap, sync::MutexGuard};

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("Error unable get lock: {source}"))]
    UnableGetLock {
        source: std::sync::PoisonError<MutexGuard<'static, HashMap<String, Font>>>,
    },
    #[snafu(display("Error font: {name} not found"))]
    FontNotFound { name: String },
    #[snafu(display("Error parse font: {message}"))]
    ParseFont { message: String },
}

impl From<&str> for Error {
    fn from(value: &str) -> Self {
        Error::ParseFont {
            message: value.to_string(),
        }
    }
}

pub type Result<T, E = Error> = std::result::Result<T, E>;

pub static DEFAULT_FONT_FAMILY: &str = "Arial";
pub static DEFAULT_FONT_DATA: &[u8] = include_bytes!("../Arial.ttf");

pub fn get_or_try_init(
    fonts: Option<Vec<(String, &[u8])>>,
) -> Result<&'static HashMap<String, Font>> {
    static GLOBAL_FONTS: OnceCell<HashMap<String, Font>> = OnceCell::new();
    GLOBAL_FONTS.get_or_try_init(|| {
        let mut m = HashMap::new();
        // 初始化字体
        // 失败时直接出错
        let font = fontdue::Font::from_bytes(DEFAULT_FONT_DATA, fontdue::FontSettings::default())?;
        m.insert(DEFAULT_FONT_FAMILY.to_string(), font);
        let mut font_datas = vec![DEFAULT_FONT_DATA];
        if let Some(value) = fonts {
            for (name, data) in value.iter() {
                let font = fontdue::Font::from_bytes(*data, fontdue::FontSettings::default())?;
                m.insert(name.to_owned(), font);
                font_datas.push(*data);
            }
        }
        #[cfg(feature = "image")]
        crate::get_or_init_fontdb(Some(font_datas));
        Ok(m)
    })
}
pub fn get_font(name: &str) -> Result<&Font> {
    let fonts = get_or_try_init(None)?;
    if let Some(font) = fonts.get(name).or_else(|| fonts.get(DEFAULT_FONT_FAMILY)) {
        Ok(font)
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
    Ok(measure_text(font, font_size, text))
}

#[cfg(test)]
mod tests {
    use super::{get_font, measure_text_width_family};
    use pretty_assertions::assert_eq;
    #[test]
    fn measure_text() {
        let name = "Arial";
        get_font(name).unwrap();

        let str = "Hello World!";
        let b = measure_text_width_family(name, 14.0, str).unwrap();

        assert_eq!(81.0, b.width().ceil());
        assert_eq!(14.0, b.height());
    }
}
