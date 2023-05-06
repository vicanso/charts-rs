use super::Color;

#[derive(Clone, PartialEq, Debug, Default)]
pub enum Position {
    #[default]
    Left,
    Top,
    Right,
    Bottom,
}

#[derive(Clone, PartialEq, Debug, Default)]
pub enum Align {
    Left,
    #[default]
    Center,
    Right,
}

#[derive(Clone, PartialEq, Debug)]
pub enum Symbol {
    Circle(f64, Option<Color>),
}

#[derive(Clone, PartialEq, Debug, Default)]
pub struct Series {
    pub name: String,
    pub data: Vec<f64>,
}
