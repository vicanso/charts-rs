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

use serde::{Deserialize, Serialize};
use substring::Substring;

#[derive(Serialize, Deserialize, Clone, Copy, PartialEq, Debug, Default)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Color {
    /// Converts color to hex format.
    pub fn hex(&self) -> String {
        format!("#{:02X}{:02X}{:02X}", self.r, self.g, self.b)
    }
    /// Converts color to rgba format.
    pub fn rgba(&self) -> String {
        let fa = (self.a as f32) / 255.0;
        format!("rgba({},{},{},{:.1})", self.r, self.g, self.b, fa)
    }
    /// Gets opacity value of color.
    pub fn opacity(&self) -> f32 {
        let a = self.a as f32;
        a / 255.0
    }
    /// Returns true if color is zero.
    pub fn is_zero(&self) -> bool {
        self.r == 0 && self.g == 0 && self.b == 0 && self.a == 0
    }
    /// Returns true if color is transparent.
    pub fn is_transparent(&self) -> bool {
        self.a == 0
    }
    /// Returns true if color is not transparent.
    pub fn is_nontransparent(&self) -> bool {
        self.a == 255
    }
    /// Returns white color.
    pub fn white() -> Color {
        (255, 255, 255).into()
    }
    /// Returns black color.
    pub fn black() -> Color {
        (0, 0, 0).into()
    }
    pub fn transparent() -> Color {
        (0, 0, 0, 0).into()
    }
    /// Sets color with new alpha.
    pub fn with_alpha(&self, a: u8) -> Color {
        let mut c = *self;
        c.a = a;
        c
    }
    /// Returns ture if the color is light.
    pub fn is_light(&self) -> bool {
        let mut r = self.r as f64;
        let mut g = self.g as f64;
        let mut b = self.b as f64;
        r = r * r * 0.299;
        g = g * g * 0.587;
        b = b * b * 0.114;
        (r + g + b).sqrt() > 127.5
    }
}

impl From<(u8, u8, u8)> for Color {
    fn from(values: (u8, u8, u8)) -> Self {
        Color {
            r: values.0,
            g: values.1,
            b: values.2,
            a: 255,
        }
    }
}
impl From<(u8, u8, u8, u8)> for Color {
    fn from(values: (u8, u8, u8, u8)) -> Self {
        Color {
            r: values.0,
            g: values.1,
            b: values.2,
            a: values.3,
        }
    }
}

fn parse_hex(hex: &str) -> u8 {
    u8::from_str_radix(hex, 16).unwrap_or_default()
}

impl From<&str> for Color {
    fn from(value: &str) -> Self {
        let mut c = Color::default();
        if !value.starts_with('#') {
            return c;
        }
        let hex = value.substring(1, value.len());
        if hex.len() == 3 {
            c.r = parse_hex(&hex.substring(0, 1).repeat(2));
            c.g = parse_hex(&hex.substring(1, 2).repeat(2));
            c.b = parse_hex(&hex.substring(2, 3).repeat(2));
        } else {
            c.r = parse_hex(hex.substring(0, 2));
            c.g = parse_hex(hex.substring(2, 4));
            c.b = parse_hex(hex.substring(4, 6));
        }
        c.a = 255;
        c
    }
}

pub(crate) fn get_color(colors: &[Color], index: usize) -> Color {
    let i = index % colors.len();
    *colors.get(i).unwrap_or_else(|| &colors[0])
}

#[cfg(test)]
mod tests {
    use super::Color;
    use pretty_assertions::assert_eq;
    #[test]
    fn color_hex() {
        let mut c: Color = (200, 200, 200).into();
        assert_eq!("#C8C8C8", c.hex());

        c = (51, 51, 51).into();
        assert_eq!("#333333", c.hex());
    }
    #[test]
    fn color_rgba() {
        let mut c: Color = (200, 200, 200).into();
        assert_eq!("rgba(200,200,200,1.0)", c.rgba());
        c = (51, 51, 51, 51).into();
        assert_eq!("rgba(51,51,51,0.2)", c.rgba());
    }
    #[test]
    fn color_opacity() {
        let mut c: Color = (200, 200, 200).into();
        assert_eq!(1.0, c.opacity());
        c = (51, 51, 51, 51).into();
        assert_eq!(0.2, c.opacity());
    }
    #[test]
    fn color_is_zero() {
        let mut c: Color = (200, 200, 200).into();
        assert!(!c.is_zero());
        c = (0, 0, 0, 0).into();
        assert!(c.is_zero());
    }
    #[test]
    fn color_is_transparent() {
        let mut c: Color = (200, 200, 200).into();
        assert!(!c.is_transparent());
        assert!(c.is_nontransparent());
        c = (200, 200, 200, 0).into();
        assert!(c.is_transparent());
        c = (200, 200, 200, 100).into();
        assert!(!c.is_nontransparent());
    }
    #[test]
    fn color_static() {
        assert_eq!("rgba(255,255,255,1.0)", Color::white().rgba());
        assert_eq!("rgba(0,0,0,1.0)", Color::black().rgba());
    }

    #[test]
    fn color_with_alpha() {
        let mut c = Color::white();
        assert_eq!("rgba(255,255,255,1.0)", c.rgba());
        c = c.with_alpha(51);
        assert_eq!("rgba(255,255,255,0.2)", c.rgba());
    }
}
