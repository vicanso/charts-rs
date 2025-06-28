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

use super::util::*;
use fontdue::layout::{CoordinateSystem, Layout, TextStyle};
use fontdue::Font;
use once_cell::sync::OnceCell;
use snafu::Snafu;
use std::collections::HashMap;

#[derive(Debug, Snafu)]
pub enum Error {
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

pub static DEFAULT_FONT_FAMILY: &str = "Roboto";
pub static DEFAULT_FONT_DATA: &[u8] = include_bytes!("../Roboto.ttf");

fn get_family_from_font(font: &fontdue::Font) -> String {
    if let Ok(re) = regex::Regex::new(r#"name:( ?)Some\("(?P<family>[\S ]+)"\)"#) {
        let desc = format!("{:?}", font);
        if let Some(caps) = re.captures(&desc) {
            let mut family = caps["family"].to_string();
            // https://developer.mozilla.org/en-US/docs/Web/CSS/font-weight
            // replace some font weight
            if let Ok(weight) = regex::Regex::new(r#"Thin|Light|Regular|Medium|Bold|Black$"#) {
                family = weight.replace_all(&family, "").to_string();
            }
            return family.trim().to_string();
        }
    }
    "".to_string()
}

pub fn get_or_try_init_fonts(fonts: Option<Vec<&[u8]>>) -> Result<&'static HashMap<String, Font>> {
    static GLOBAL_FONTS: OnceCell<HashMap<String, Font>> = OnceCell::new();
    GLOBAL_FONTS.get_or_try_init(|| {
        let mut m = HashMap::new();
        // init fonts, will returns an error if load font fails.
        let font = fontdue::Font::from_bytes(DEFAULT_FONT_DATA, fontdue::FontSettings::default())?;
        m.insert(DEFAULT_FONT_FAMILY.to_string(), font);
        let mut font_datas = vec![DEFAULT_FONT_DATA];
        if let Some(value) = fonts {
            for data in value.iter() {
                let font = fontdue::Font::from_bytes(*data, fontdue::FontSettings::default())?;
                let family = get_family_from_font(&font);
                if !family.is_empty() {
                    m.insert(family, font);
                    font_datas.push(*data);
                }
            }
        }
        #[cfg(feature = "image-encoder")]
        crate::get_or_init_fontdb(Some(font_datas));
        Ok(m)
    })
}
/// Gets font by font family.
pub fn get_font(name: &str) -> Result<&Font> {
    let fonts = get_or_try_init_fonts(None)?;
    if let Some(font) = fonts.get(name).or_else(|| fonts.get(DEFAULT_FONT_FAMILY)) {
        Ok(font)
    } else {
        FontNotFoundSnafu {
            name: name.to_string(),
        }
        .fail()
    }
}
/// Gets all supported font family
pub fn get_font_families() -> Result<Vec<String>> {
    let fonts = get_or_try_init_fonts(None)?;
    let mut families = vec![];
    for (name, _) in fonts.iter() {
        families.push(name.to_string());
    }
    Ok(families)
}

/// Measures the display area of text of a specified font size.
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

/// Measures the display area of text of a specified font size and font family.
pub fn measure_text_width_family(font_family: &str, font_size: f32, text: &str) -> Result<Box> {
    let font = get_font(font_family)?;
    Ok(measure_text(font, font_size, text))
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
        let b = measure_text(font, font_size, item);
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
    let b = measure_text(font, font_size, text);
    if b.width() <= width {
        return Ok(vec![text.to_string()]);
    }

    let mut current = "".to_string();
    let mut result = vec![];
    for item in text.chars() {
        let new_str = current.clone() + &item.to_string();
        let b = measure_text(font, font_size, &new_str);
        if b.width() > width {
            result.push(current);
            current = item.to_string();
            continue;
        }
        current = new_str;
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
