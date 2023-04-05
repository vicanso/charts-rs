use usvg::{Fill, Paint};

#[derive(Clone, Copy, PartialEq, Debug, Default)]
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
    pub fn black() -> Self {
        (0, 0, 0).into()
    }
    pub fn white() -> Self {
        (255, 255, 255).into()
    }
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

pub fn get_echart_series_colors() -> Vec<Color> {
    vec![
        (84, 112, 198).into(),
        (145, 204, 117).into(),
        (250, 200, 88).into(),
        (238, 102, 102).into(),
        (115, 192, 222).into(),
        (59, 162, 114).into(),
        (252, 132, 82).into(),
        (154, 96, 180).into(),
        (234, 124, 204).into(),
    ]
}

pub fn get_grafana_series_colors() -> Vec<Color> {
    vec![
        (126, 178, 109).into(),
        (234, 184, 57).into(),
        (110, 208, 224).into(),
        (239, 132, 60).into(),
        (226, 77, 66).into(),
        (31, 120, 193).into(),
        (112, 93, 160).into(),
        (80, 134, 66).into(),
    ]
}

pub fn get_ant_series_colors() -> Vec<Color> {
    vec![
        (91, 143, 249).into(),
        (90, 216, 166).into(),
        (93, 112, 146).into(),
        (246, 189, 22).into(),
        (111, 94, 249).into(),
        (109, 200, 236).into(),
        (148, 95, 185).into(),
        (255, 152, 69).into(),
    ]
}
