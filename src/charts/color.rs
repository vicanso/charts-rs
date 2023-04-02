use usvg::{Fill, Paint};

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl From<usvg::Color> for Color {
    fn from(color: usvg::Color) -> Self {
        Color {
            r: color.red,
            g: color.green,
            b: color.blue,
            a: 255,
        }
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

impl Color {
    pub fn divide(&self) -> (usvg::Color, usvg::Opacity) {
        let c = usvg::Color::new_rgb(self.r, self.g, self.b);
        let a = usvg::Opacity::new_u8(self.a);
        (c, a)
    }
    pub fn is_zero(&self) -> bool {
        self.r == 0 && self.g == 0 && self.b == 0 && self.a == 0
    }
    pub fn is_transparent(&self) -> bool {
        self.a == 0
    }
    pub fn string(&self) -> String {
        let fa = (self.a as f64) / 255.0;
        format!("rgba({},{},{},{:.1})", self.r, self.g, self.b, fa)
    }
}

impl From<Color> for Fill {
    fn from(val: Color) -> Self {
        let mut fill = Fill::default();
        let (c, opacity) = val.divide();
        fill.paint = Paint::Color(c);
        fill.opacity = opacity;
        fill
    }
}

// // IsZero returns if the color has been set or not.
// func (c Color) IsZero() bool {
// 	return c.R == 0 && c.G == 0 && c.B == 0 && c.A == 0
// }

// // IsTransparent returns if the colors alpha channel is zero.
// func (c Color) IsTransparent() bool {
// 	return c.A == 0
// }
