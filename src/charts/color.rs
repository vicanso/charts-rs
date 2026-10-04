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

/// An RGBA color. Parses from hex strings (`"#345"`, `"#3456"`, `"#ffcc00"`,
/// `"#ffcc0080"`), CSS `rgb()` / `rgba()` functions and `"transparent"` via
/// [`Color::parse`], and converts from `(r, g, b)` / `(r, g, b, a)` tuples.
///
/// Serializes as a hex string (`"#RRGGBB"`, or `"#RRGGBBAA"` when not fully
/// opaque) and deserializes from any string [`Color::parse`] accepts, or from
/// the `{r, g, b, a}` object of earlier versions.
#[derive(Clone, Copy, PartialEq, Debug, Default)]
pub struct Color {
    /// Red channel.
    pub r: u8,
    /// Green channel.
    pub g: u8,
    /// Blue channel.
    pub b: u8,
    /// Alpha channel; 0 is transparent, 255 is opaque.
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
    /// Returns a fully transparent color.
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

fn parse_hex(hex: &str) -> Option<u8> {
    u8::from_str_radix(hex, 16).ok()
}

/// Parses a CSS `rgb(r, g, b)` / `rgba(r, g, b, a)` body. Channels are
/// `0..=255`; alpha is `0..=1` (or a percentage) as in CSS.
fn parse_rgb_function(body: &str, with_alpha: bool) -> Option<Color> {
    let body = body.strip_suffix(')')?;
    let parts: Vec<&str> = body.split(',').map(str::trim).collect();
    if parts.len() != if with_alpha { 4 } else { 3 } {
        return None;
    }
    let channel = |s: &str| -> Option<u8> {
        let v: f32 = s.parse().ok()?;
        if (0.0..=255.0).contains(&v) {
            Some(v.round() as u8)
        } else {
            None
        }
    };
    let a = if with_alpha {
        let s = parts[3];
        let v: f32 = if let Some(pct) = s.strip_suffix('%') {
            pct.trim().parse::<f32>().ok()? / 100.0
        } else {
            s.parse().ok()?
        };
        if !(0.0..=1.0).contains(&v) {
            return None;
        }
        (v * 255.0).round() as u8
    } else {
        255
    };
    Some(Color {
        r: channel(parts[0])?,
        g: channel(parts[1])?,
        b: channel(parts[2])?,
        a,
    })
}

impl Color {
    /// Parses a color string: `#rgb`, `#rgba`, `#rrggbb`, `#rrggbbaa`,
    /// `rgb(r, g, b)`, `rgba(r, g, b, a)` or `transparent`. Returns `None`
    /// for anything else, so callers can report the bad value instead of
    /// silently drawing with a wrong color.
    pub fn parse(value: &str) -> Option<Color> {
        let value = value.trim();
        if value.eq_ignore_ascii_case("transparent") {
            return Some(Color::transparent());
        }
        if let Some(body) = value.strip_prefix("rgba(") {
            return parse_rgb_function(body, true);
        }
        if let Some(body) = value.strip_prefix("rgb(") {
            return parse_rgb_function(body, false);
        }
        let hex = value.strip_prefix('#')?;
        if !hex.is_ascii() {
            return None;
        }
        match hex.len() {
            // Shorthand "#abc" / "#abcd": each digit doubles ("a" → 0xaa).
            3 | 4 => {
                let digit = |i: usize| parse_hex(&hex[i..i + 1]).map(|d| d * 17);
                Some(Color {
                    r: digit(0)?,
                    g: digit(1)?,
                    b: digit(2)?,
                    a: if hex.len() == 4 { digit(3)? } else { 255 },
                })
            }
            6 | 8 => Some(Color {
                r: parse_hex(&hex[0..2])?,
                g: parse_hex(&hex[2..4])?,
                b: parse_hex(&hex[4..6])?,
                a: if hex.len() == 8 {
                    parse_hex(&hex[6..8])?
                } else {
                    255
                },
            }),
            _ => None,
        }
    }
}

impl From<&str> for Color {
    /// Infallible conversion for the builder API; an unparsable string yields
    /// the transparent default. Use [`Color::parse`] to detect bad input.
    fn from(value: &str) -> Self {
        Color::parse(value).unwrap_or_default()
    }
}

impl Serialize for Color {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        if self.a == 255 {
            serializer.serialize_str(&self.hex())
        } else {
            serializer.serialize_str(&format!("{}{:02X}", self.hex(), self.a))
        }
    }
}

impl<'de> Deserialize<'de> for Color {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(untagged)]
        enum Repr {
            Text(String),
            Channels { r: u8, g: u8, b: u8, a: u8 },
        }
        match Repr::deserialize(deserializer)? {
            Repr::Text(s) => Color::parse(&s)
                .ok_or_else(|| serde::de::Error::custom(format!("invalid color {s:?}"))),
            Repr::Channels { r, g, b, a } => Ok(Color { r, g, b, a }),
        }
    }
}

/// Where `value` lies on a color scale from `min` to `max`, from 0 to 1.
///
/// Without classes the scale is continuous. With `thresholds` (the values
/// where one class ends and the next begins) or `steps` (that many classes
/// of the same width) the values of a class all lie at the same place: the
/// first class at 0, the last one at 1, the others evenly between them.
pub(crate) fn scale_position(
    value: f32,
    (min, max): (f32, f32),
    steps: usize,
    thresholds: &[f32],
) -> f32 {
    let mut limits: Vec<f32> = thresholds
        .iter()
        .copied()
        .filter(|t| t.is_finite())
        .collect();
    if !limits.is_empty() {
        limits.sort_by(f32::total_cmp);
        let class = limits.iter().filter(|limit| value >= **limit).count();
        return class as f32 / limits.len() as f32;
    }
    let range = max - min;
    let share = if range > 0.0 {
        ((value - min) / range).clamp(0.0, 1.0)
    } else {
        1.0
    };
    if steps < 2 || !share.is_finite() {
        return if share.is_finite() { share } else { 0.0 };
    }
    let class = ((share * steps as f32) as usize).min(steps - 1);
    class as f32 / (steps - 1) as f32
}

/// The color at `position` (from 0 to 1) of a gradient through `colors`,
/// which are spread evenly over it.
pub(crate) fn gradient_color(colors: &[Color], position: f32) -> Color {
    let (Some(first), Some(last)) = (colors.first(), colors.last()) else {
        return Color::default();
    };
    // The start of the scale, also when it is a single color.
    if !position.is_finite() || position <= 0.0 || colors.len() < 2 {
        return *first;
    }
    if position >= 1.0 {
        return *last;
    }
    let place = position * (colors.len() - 1) as f32;
    let index = (place as usize).min(colors.len() - 2);
    let share = place - index as f32;
    let (from, to) = (colors[index], colors[index + 1]);
    let mix = |a: u8, b: u8| (a as f32 + (b as f32 - a as f32) * share).round() as u8;
    Color {
        r: mix(from.r, to.r),
        g: mix(from.g, to.g),
        b: mix(from.b, to.b),
        a: mix(from.a, to.a),
    }
}

pub(crate) fn get_color(colors: &[Color], index: usize) -> Color {
    // Guard against an empty palette (e.g. `"series_colors": []` from JSON),
    // which would otherwise panic on `index % 0` / out-of-bounds indexing.
    if colors.is_empty() {
        return Color::default();
    }
    let i = index % colors.len();
    *colors.get(i).unwrap_or(&colors[0])
}

#[cfg(test)]
mod tests {
    #[test]
    fn color_scales() {
        use super::{gradient_color, scale_position};
        // Continuous: where the value lies between the ends, kept on them.
        assert_eq!(0.25, scale_position(25.0, (0.0, 100.0), 0, &[]));
        assert_eq!(0.0, scale_position(-5.0, (0.0, 100.0), 1, &[]));
        assert_eq!(1.0, scale_position(500.0, (0.0, 100.0), 0, &[]));
        // Classes of the same width: the first at 0, the last at 1.
        let class = |value: f32| scale_position(value, (0.0, 100.0), 5, &[]);
        assert_eq!(
            vec![0.0, 0.0, 0.25, 0.5, 0.75, 1.0, 1.0],
            [0.0, 19.9, 20.0, 59.9, 60.0, 99.9, 100.0]
                .map(class)
                .to_vec()
        );
        // Classes of one's own, whatever the order they are given in.
        let class = |value: f32| scale_position(value, (0.0, 0.0), 9, &[50.0, 10.0, f32::NAN]);
        assert_eq!(
            vec![0.0, 0.5, 0.5, 1.0, 1.0],
            [9.9, 10.0, 49.9, 50.0, 1e9].map(class).to_vec()
        );
        // A scale without a range, and a value that is no number.
        assert_eq!(1.0, scale_position(3.0, (7.0, 7.0), 0, &[]));
        assert_eq!(0.0, scale_position(f32::NAN, (0.0, 100.0), 0, &[]));

        let (red, white, blue): (Color, Color, Color) =
            ("#ff0000".into(), "#ffffff".into(), "#0000ff".into());
        let colors = [red, white, blue];
        // The colors are spread evenly: the middle one in the middle.
        assert_eq!(red, gradient_color(&colors, 0.0));
        assert_eq!(white, gradient_color(&colors, 0.5));
        assert_eq!(blue, gradient_color(&colors, 1.0));
        assert_eq!("#FF8080", gradient_color(&colors, 0.25).hex());
        assert_eq!("#8080FF", gradient_color(&colors, 0.75).hex());
        // Off the scale is its end.
        assert_eq!(red, gradient_color(&colors, -3.0));
        assert_eq!(blue, gradient_color(&colors, 3.0));
        assert_eq!(red, gradient_color(&colors, f32::NAN));
        assert_eq!(red, gradient_color(&[red], 0.7));
        assert_eq!(Color::default(), gradient_color(&[], 0.5));
    }

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

    #[test]
    fn color_parse() {
        assert_eq!(Some((51, 68, 85, 255).into()), Color::parse("#345"));
        assert_eq!(Some((51, 68, 85, 102).into()), Color::parse("#3456"));
        assert_eq!(Some((255, 204, 0, 255).into()), Color::parse("#ffcc00"));
        assert_eq!(Some((255, 204, 0, 128).into()), Color::parse("#ffcc0080"));
        assert_eq!(Some((255, 204, 0, 255).into()), Color::parse(" #FFCC00 "));
        assert_eq!(
            Some((10, 20, 30, 255).into()),
            Color::parse("rgb(10, 20, 30)")
        );
        assert_eq!(
            Some((10, 20, 30, 128).into()),
            Color::parse("rgba(10,20,30,0.5)")
        );
        assert_eq!(
            Some((10, 20, 30, 64).into()),
            Color::parse("rgba(10,20,30,25%)")
        );
        assert_eq!(Some(Color::transparent()), Color::parse("transparent"));
        for bad in [
            "",
            "red",
            "#12345",
            "#GGG",
            "#ffcc00ff00",
            "rgb(1,2)",
            "rgba(1,2,3,2)",
        ] {
            assert_eq!(None, Color::parse(bad), "{bad:?} should not parse");
        }
        // The infallible conversion keeps its transparent fallback.
        let c: Color = "red".into();
        assert!(c.is_zero());
        let c: Color = "#345".into();
        assert_eq!("#334455", c.hex());
    }

    #[test]
    fn color_serde() {
        let c: Color = (255, 204, 0, 128).into();
        assert_eq!("\"#FFCC0080\"", serde_json::to_string(&c).unwrap());
        let opaque: Color = (255, 204, 0).into();
        assert_eq!("\"#FFCC00\"", serde_json::to_string(&opaque).unwrap());
        assert_eq!(c, serde_json::from_str("\"#ffcc0080\"").unwrap());
        assert_eq!(
            c,
            serde_json::from_str(r#"{"r": 255, "g": 204, "b": 0, "a": 128}"#).unwrap()
        );
        assert!(serde_json::from_str::<Color>("\"red\"").is_err());
    }
}
