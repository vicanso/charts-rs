use super::color::Color;
use super::common::Align;
use super::font::DEFAULT_FONT_FAMILY;
use super::util::Box;
use ahash::AHashMap;
use arc_swap::ArcSwap;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

pub static DEFAULT_WIDTH: f32 = 600.0;
pub static DEFAULT_HEIGHT: f32 = 400.0;

pub static DEFAULT_TITLE_HEIGHT: f32 = 30.0;
pub static DEFAULT_SUB_TITLE_HEIGHT: f32 = 20.0;

pub static DEFAULT_X_AXIS_HEIGHT: f32 = 30.0;
pub static DEFAULT_X_AXIS_NAME_GAP: f32 = 5.0;

pub static DEFAULT_Y_AXIS_WIDTH: f32 = 40.0;
pub static DEFAULT_Y_AXIS_NAME_GAP: f32 = 8.0;
pub static DEFAULT_Y_AXIS_SPLIT_NUMBER: usize = 6;
pub static DEFAULT_FONT_SIZE: f32 = 14.0;

pub static DEFAULT_SERIES_STROKE_WIDTH: f32 = 2.0;

pub static THEME_DARK: &str = "dark";
pub static THEME_ANT: &str = "ant";
pub static THEME_GRAFANA: &str = "grafana";

static LIGHT_THEME_NAME: &str = "light";

#[derive(Clone, Debug, Default, Serialize, Deserialize)]

pub struct Theme {
    pub is_light: bool,
    pub font_family: String,
    pub margin: Box,
    pub width: f32,
    pub height: f32,
    pub background_color: Color,

    // title
    pub title_font_size: f32,
    pub title_font_color: Color,
    pub title_font_weight: Option<String>,
    pub title_margin: Option<Box>,
    pub title_align: Align,
    pub title_height: f32,

    // sub title
    pub sub_title_font_size: f32,
    pub sub_title_font_color: Color,
    pub sub_title_margin: Option<Box>,
    pub sub_title_align: Align,
    pub sub_title_height: f32,

    // legend
    pub legend_font_size: f32,
    pub legend_font_color: Color,
    pub legend_align: Align,
    pub legend_margin: Option<Box>,

    // x axis
    pub x_axis_font_size: f32,
    pub x_axis_stroke_color: Color,
    pub x_axis_font_color: Color,
    pub x_axis_name_gap: f32,
    pub x_axis_height: f32,

    // y axis
    pub y_axis_font_size: f32,
    pub y_axis_font_color: Color,
    pub y_axis_stroke_color: Color,
    pub y_axis_split_number: usize,
    pub y_axis_name_gap: f32,

    // grid
    pub grid_stroke_color: Color,
    pub grid_stroke_width: f32,

    // series
    pub series_stroke_width: f32,
    pub series_label_font_size: f32,
    pub series_label_font_color: Color,
    pub series_colors: Vec<Color>,

    // table
    pub table_header_color: Color,
    pub table_body_colors: Vec<Color>,
    pub table_border_color: Color,
}

static LIGHT_THEME: Lazy<Theme> = Lazy::new(|| {
    let x_axis_color = (110, 112, 121).into();
    let font_color: Color = (70, 70, 70).into();
    Theme {
        is_light: true,
        font_family: DEFAULT_FONT_FAMILY.to_string(),
        margin: (5.0).into(),
        width: DEFAULT_WIDTH,
        height: DEFAULT_HEIGHT,
        background_color: Color::white(),

        title_font_color: font_color,
        title_font_size: 18.0,
        title_font_weight: Some("bold".to_string()),
        title_margin: None,
        title_align: Align::Center,
        title_height: DEFAULT_TITLE_HEIGHT,

        sub_title_font_color: font_color,
        sub_title_font_size: DEFAULT_FONT_SIZE,
        sub_title_margin: None,
        sub_title_align: Align::Center,
        sub_title_height: DEFAULT_SUB_TITLE_HEIGHT,

        legend_font_size: DEFAULT_FONT_SIZE,
        legend_font_color: font_color,
        legend_align: Align::Center,
        legend_margin: None,

        x_axis_font_size: DEFAULT_FONT_SIZE,
        x_axis_stroke_color: x_axis_color,
        x_axis_font_color: x_axis_color,
        x_axis_name_gap: DEFAULT_X_AXIS_NAME_GAP,
        x_axis_height: DEFAULT_X_AXIS_HEIGHT,

        y_axis_font_size: DEFAULT_FONT_SIZE,
        y_axis_font_color: x_axis_color,
        y_axis_stroke_color: Color::transparent(),
        y_axis_split_number: DEFAULT_Y_AXIS_SPLIT_NUMBER,
        y_axis_name_gap: DEFAULT_Y_AXIS_NAME_GAP,

        grid_stroke_color: (224, 230, 242).into(),
        grid_stroke_width: 1.0,

        series_stroke_width: DEFAULT_SERIES_STROKE_WIDTH,
        series_label_font_size: DEFAULT_FONT_SIZE,
        series_label_font_color: font_color,
        series_colors: vec![
            "#5470c6".into(),
            "#91cc75".into(),
            "#fac858".into(),
            "#ee6666".into(),
            "#73c0de".into(),
            "#3ba272".into(),
            "#fc8452".into(),
            "#9a60b4".into(),
            "#ea7ccc".into(),
        ],

        table_header_color: (242, 243, 245).into(),
        table_body_colors: vec![(255, 255, 255).into()],
        table_border_color: (229, 230, 235).into(),
    }
});

static DARK_THEME: Lazy<Theme> = Lazy::new(|| {
    let x_axis_color = (185, 184, 206).into();
    let bg_color = (16, 12, 42).into();

    let font_color: Color = (238, 238, 238).into();
    Theme {
        is_light: false,
        font_family: DEFAULT_FONT_FAMILY.to_string(),
        margin: (5.0).into(),
        width: DEFAULT_WIDTH,
        height: DEFAULT_HEIGHT,
        background_color: bg_color,

        title_font_color: font_color,
        title_font_size: 18.0,
        title_font_weight: Some("bold".to_string()),
        title_margin: None,
        title_align: Align::Center,
        title_height: DEFAULT_TITLE_HEIGHT,

        sub_title_font_color: font_color,
        sub_title_font_size: DEFAULT_FONT_SIZE,
        sub_title_margin: None,
        sub_title_align: Align::Center,
        sub_title_height: DEFAULT_SUB_TITLE_HEIGHT,

        legend_font_size: DEFAULT_FONT_SIZE,
        legend_font_color: font_color,
        legend_align: Align::Center,
        legend_margin: None,

        x_axis_font_size: DEFAULT_FONT_SIZE,
        x_axis_stroke_color: x_axis_color,
        x_axis_font_color: x_axis_color,
        x_axis_name_gap: DEFAULT_X_AXIS_NAME_GAP,
        x_axis_height: DEFAULT_X_AXIS_HEIGHT,

        y_axis_font_size: DEFAULT_FONT_SIZE,
        y_axis_font_color: x_axis_color,
        y_axis_stroke_color: Color::transparent(),
        y_axis_split_number: DEFAULT_Y_AXIS_SPLIT_NUMBER,
        y_axis_name_gap: DEFAULT_Y_AXIS_NAME_GAP,

        grid_stroke_color: (71, 71, 83).into(),
        grid_stroke_width: 1.0,

        series_stroke_width: DEFAULT_SERIES_STROKE_WIDTH,
        series_label_font_size: DEFAULT_FONT_SIZE,
        series_label_font_color: font_color,
        series_colors: vec![
            "#5470c6".into(),
            "#91cc75".into(),
            "#fac858".into(),
            "#ee6666".into(),
            "#73c0de".into(),
            "#3ba272".into(),
            "#fc8452".into(),
            "#9a60b4".into(),
            "#ea7ccc".into(),
        ],

        table_header_color: bg_color,
        table_body_colors: vec![bg_color.with_alpha(230)],
        table_border_color: (100, 100, 100).into(),
    }
});

static ANT_THEME: Lazy<Theme> = Lazy::new(|| {
    let x_axis_color = (110, 112, 121).into();

    let font_color: Color = (70, 70, 70).into();
    Theme {
        is_light: true,
        font_family: DEFAULT_FONT_FAMILY.to_string(),
        margin: (5.0).into(),
        width: DEFAULT_WIDTH,
        height: DEFAULT_HEIGHT,
        background_color: Color::white(),

        title_font_color: font_color,
        title_font_size: 18.0,
        title_font_weight: Some("bold".to_string()),
        title_margin: None,
        title_align: Align::Center,
        title_height: DEFAULT_TITLE_HEIGHT,

        sub_title_font_color: font_color,
        sub_title_font_size: DEFAULT_FONT_SIZE,
        sub_title_margin: None,
        sub_title_align: Align::Center,
        sub_title_height: DEFAULT_SUB_TITLE_HEIGHT,

        legend_font_size: DEFAULT_FONT_SIZE,
        legend_font_color: font_color,
        legend_align: Align::Center,
        legend_margin: None,

        x_axis_font_size: DEFAULT_FONT_SIZE,
        x_axis_stroke_color: x_axis_color,
        x_axis_font_color: x_axis_color,
        x_axis_name_gap: DEFAULT_X_AXIS_NAME_GAP,
        x_axis_height: DEFAULT_X_AXIS_HEIGHT,

        y_axis_font_size: DEFAULT_FONT_SIZE,
        y_axis_font_color: x_axis_color,
        y_axis_stroke_color: Color::transparent(),
        y_axis_split_number: DEFAULT_Y_AXIS_SPLIT_NUMBER,
        y_axis_name_gap: DEFAULT_Y_AXIS_NAME_GAP,

        grid_stroke_color: (224, 230, 242).into(),
        grid_stroke_width: 1.0,

        series_stroke_width: DEFAULT_SERIES_STROKE_WIDTH,
        series_label_font_size: DEFAULT_FONT_SIZE,
        series_label_font_color: font_color,

        series_colors: vec![
            "#5b8ff9".into(),
            "#5ad8a6".into(),
            "#5d7092".into(),
            "#f6bd16".into(),
            "#6f5ef9".into(),
            "#6dc8ec".into(),
            "#945fb9".into(),
            "#ff9845".into(),
        ],

        table_header_color: (250, 250, 250).into(),
        table_body_colors: vec![(255, 255, 255).into()],
        table_border_color: (239, 239, 244).into(),
    }
});

static VINTAGE_THEME: Lazy<Theme> = Lazy::new(|| {
    let x_axis_color = (0, 0, 0).into();

    let font_color: Color = (51, 51, 51).into();
    Theme {
        is_light: true,
        font_family: DEFAULT_FONT_FAMILY.to_string(),
        margin: (5.0).into(),
        width: DEFAULT_WIDTH,
        height: DEFAULT_HEIGHT,
        background_color: (254, 248, 239).into(),

        title_font_color: font_color,
        title_font_size: 18.0,
        title_font_weight: Some("bold".to_string()),
        title_margin: None,
        title_align: Align::Center,
        title_height: DEFAULT_TITLE_HEIGHT,

        sub_title_font_color: font_color,
        sub_title_font_size: DEFAULT_FONT_SIZE,
        sub_title_margin: None,
        sub_title_align: Align::Center,
        sub_title_height: DEFAULT_SUB_TITLE_HEIGHT,

        legend_font_size: DEFAULT_FONT_SIZE,
        legend_font_color: font_color,
        legend_align: Align::Center,
        legend_margin: None,

        x_axis_font_size: DEFAULT_FONT_SIZE,
        x_axis_stroke_color: x_axis_color,
        x_axis_font_color: x_axis_color,
        x_axis_name_gap: DEFAULT_X_AXIS_NAME_GAP,
        x_axis_height: DEFAULT_X_AXIS_HEIGHT,

        y_axis_font_size: DEFAULT_FONT_SIZE,
        y_axis_font_color: x_axis_color,
        y_axis_stroke_color: Color::transparent(),
        y_axis_split_number: DEFAULT_Y_AXIS_SPLIT_NUMBER,
        y_axis_name_gap: DEFAULT_Y_AXIS_NAME_GAP,

        grid_stroke_color: (224, 230, 242).into(),
        grid_stroke_width: 1.0,

        series_stroke_width: DEFAULT_SERIES_STROKE_WIDTH,
        series_label_font_size: DEFAULT_FONT_SIZE,
        series_label_font_color: font_color,

        series_colors: vec![
            "#d87c7c".into(),
            "#919e8b".into(),
            "#d7ab82".into(),
            "#6e7074".into(),
            "#61a0a8".into(),
            "#efa18d".into(),
            "#787464".into(),
            "#cc7e63".into(),
            "#724e58".into(),
            "#4b565b".into(),
        ],

        table_header_color: (250, 250, 250).into(),
        table_body_colors: vec![(255, 255, 255).into()],
        table_border_color: (239, 239, 244).into(),
    }
});

static SHINE_THEME: Lazy<Theme> = Lazy::new(|| {
    let x_axis_color = (0, 0, 0).into();

    let font_color: Color = (51, 51, 51).into();
    Theme {
        is_light: true,
        font_family: DEFAULT_FONT_FAMILY.to_string(),
        margin: (5.0).into(),
        width: DEFAULT_WIDTH,
        height: DEFAULT_HEIGHT,
        background_color: (255, 255, 255).into(),

        title_font_color: font_color,
        title_font_size: 18.0,
        title_font_weight: Some("bold".to_string()),
        title_margin: None,
        title_align: Align::Center,
        title_height: DEFAULT_TITLE_HEIGHT,

        sub_title_font_color: font_color,
        sub_title_font_size: DEFAULT_FONT_SIZE,
        sub_title_margin: None,
        sub_title_align: Align::Center,
        sub_title_height: DEFAULT_SUB_TITLE_HEIGHT,

        legend_font_size: DEFAULT_FONT_SIZE,
        legend_font_color: font_color,
        legend_align: Align::Center,
        legend_margin: None,

        x_axis_font_size: DEFAULT_FONT_SIZE,
        x_axis_stroke_color: x_axis_color,
        x_axis_font_color: x_axis_color,
        x_axis_name_gap: DEFAULT_X_AXIS_NAME_GAP,
        x_axis_height: DEFAULT_X_AXIS_HEIGHT,

        y_axis_font_size: DEFAULT_FONT_SIZE,
        y_axis_font_color: x_axis_color,
        y_axis_stroke_color: Color::transparent(),
        y_axis_split_number: DEFAULT_Y_AXIS_SPLIT_NUMBER,
        y_axis_name_gap: DEFAULT_Y_AXIS_NAME_GAP,

        grid_stroke_color: (224, 230, 242).into(),
        grid_stroke_width: 1.0,

        series_stroke_width: DEFAULT_SERIES_STROKE_WIDTH,
        series_label_font_size: DEFAULT_FONT_SIZE,
        series_label_font_color: font_color,

        series_colors: vec![
            "#c12e34".into(),
            "#e6b600".into(),
            "#0098d9".into(),
            "#2b821d".into(),
            "#005eaa".into(),
            "#339ca8".into(),
            "#cda819".into(),
            "#32a487".into(),
        ],

        table_header_color: (250, 250, 250).into(),
        table_body_colors: vec![(255, 255, 255).into()],
        table_border_color: (239, 239, 244).into(),
    }
});

static WALDEN_THEME: Lazy<Theme> = Lazy::new(|| {
    let x_axis_color = (110, 112, 121).into();

    let font_color: Color = (70, 70, 70).into();
    Theme {
        is_light: true,
        font_family: DEFAULT_FONT_FAMILY.to_string(),
        margin: (5.0).into(),
        width: DEFAULT_WIDTH,
        height: DEFAULT_HEIGHT,
        background_color: Color::white(),

        title_font_color: font_color,
        title_font_size: 18.0,
        title_font_weight: Some("bold".to_string()),
        title_margin: None,
        title_align: Align::Center,
        title_height: DEFAULT_TITLE_HEIGHT,

        sub_title_font_color: font_color,
        sub_title_font_size: DEFAULT_FONT_SIZE,
        sub_title_margin: None,
        sub_title_align: Align::Center,
        sub_title_height: DEFAULT_SUB_TITLE_HEIGHT,

        legend_font_size: DEFAULT_FONT_SIZE,
        legend_font_color: font_color,
        legend_align: Align::Center,
        legend_margin: None,

        x_axis_font_size: DEFAULT_FONT_SIZE,
        x_axis_stroke_color: x_axis_color,
        x_axis_font_color: x_axis_color,
        x_axis_name_gap: DEFAULT_X_AXIS_NAME_GAP,
        x_axis_height: DEFAULT_X_AXIS_HEIGHT,

        y_axis_font_size: DEFAULT_FONT_SIZE,
        y_axis_font_color: x_axis_color,
        y_axis_stroke_color: Color::transparent(),
        y_axis_split_number: DEFAULT_Y_AXIS_SPLIT_NUMBER,
        y_axis_name_gap: DEFAULT_Y_AXIS_NAME_GAP,

        grid_stroke_color: (224, 230, 242).into(),
        grid_stroke_width: 1.0,

        series_stroke_width: DEFAULT_SERIES_STROKE_WIDTH,
        series_label_font_size: DEFAULT_FONT_SIZE,
        series_label_font_color: font_color,

        series_colors: vec![
            "#3fb1e3".into(),
            "#6be6c1".into(),
            "#626c91".into(),
            "#a0a7e6".into(),
            "#c4ebad".into(),
            "#96dee8".into(),
        ],

        table_header_color: (250, 250, 250).into(),
        table_body_colors: vec![(255, 255, 255).into()],
        table_border_color: (239, 239, 244).into(),
    }
});

static WESTEROS_THEME: Lazy<Theme> = Lazy::new(|| {
    let x_axis_color = (110, 112, 121).into();

    let font_color: Color = (70, 70, 70).into();
    Theme {
        is_light: true,
        font_family: DEFAULT_FONT_FAMILY.to_string(),
        margin: (5.0).into(),
        width: DEFAULT_WIDTH,
        height: DEFAULT_HEIGHT,
        background_color: Color::white(),

        title_font_color: font_color,
        title_font_size: 18.0,
        title_font_weight: Some("bold".to_string()),
        title_margin: None,
        title_align: Align::Center,
        title_height: DEFAULT_TITLE_HEIGHT,

        sub_title_font_color: font_color,
        sub_title_font_size: DEFAULT_FONT_SIZE,
        sub_title_margin: None,
        sub_title_align: Align::Center,
        sub_title_height: DEFAULT_SUB_TITLE_HEIGHT,

        legend_font_size: DEFAULT_FONT_SIZE,
        legend_font_color: font_color,
        legend_align: Align::Center,
        legend_margin: None,

        x_axis_font_size: DEFAULT_FONT_SIZE,
        x_axis_stroke_color: x_axis_color,
        x_axis_font_color: x_axis_color,
        x_axis_name_gap: DEFAULT_X_AXIS_NAME_GAP,
        x_axis_height: DEFAULT_X_AXIS_HEIGHT,

        y_axis_font_size: DEFAULT_FONT_SIZE,
        y_axis_font_color: x_axis_color,
        y_axis_stroke_color: Color::transparent(),
        y_axis_split_number: DEFAULT_Y_AXIS_SPLIT_NUMBER,
        y_axis_name_gap: DEFAULT_Y_AXIS_NAME_GAP,

        grid_stroke_color: (224, 230, 242).into(),
        grid_stroke_width: 1.0,

        series_stroke_width: DEFAULT_SERIES_STROKE_WIDTH,
        series_label_font_size: DEFAULT_FONT_SIZE,
        series_label_font_color: font_color,

        series_colors: vec![
            "#516b91".into(),
            "#59c4e6".into(),
            "#edafda".into(),
            "#93b7e3".into(),
            "#a5e7f0".into(),
            "#cbb0e3".into(),
        ],

        table_header_color: (250, 250, 250).into(),
        table_body_colors: vec![(255, 255, 255).into()],
        table_border_color: (239, 239, 244).into(),
    }
});

static CHALK_THEME: Lazy<Theme> = Lazy::new(|| {
    let x_axis_color = (170, 170, 170).into();

    let font_color: Color = (255, 255, 255).into();
    let bg_color: Color = (41, 52, 65).into();
    Theme {
        is_light: true,
        font_family: DEFAULT_FONT_FAMILY.to_string(),
        margin: (5.0).into(),
        width: DEFAULT_WIDTH,
        height: DEFAULT_HEIGHT,
        background_color: bg_color,

        title_font_color: font_color,
        title_font_size: 18.0,
        title_font_weight: Some("bold".to_string()),
        title_margin: None,
        title_align: Align::Center,
        title_height: DEFAULT_TITLE_HEIGHT,

        sub_title_font_color: font_color,
        sub_title_font_size: DEFAULT_FONT_SIZE,
        sub_title_margin: None,
        sub_title_align: Align::Center,
        sub_title_height: DEFAULT_SUB_TITLE_HEIGHT,

        legend_font_size: DEFAULT_FONT_SIZE,
        legend_font_color: font_color,
        legend_align: Align::Center,
        legend_margin: None,

        x_axis_font_size: DEFAULT_FONT_SIZE,
        x_axis_stroke_color: x_axis_color,
        x_axis_font_color: x_axis_color,
        x_axis_name_gap: DEFAULT_X_AXIS_NAME_GAP,
        x_axis_height: DEFAULT_X_AXIS_HEIGHT,

        y_axis_font_size: DEFAULT_FONT_SIZE,
        y_axis_font_color: x_axis_color,
        y_axis_stroke_color: Color::transparent(),
        y_axis_split_number: DEFAULT_Y_AXIS_SPLIT_NUMBER,
        y_axis_name_gap: DEFAULT_Y_AXIS_NAME_GAP,

        grid_stroke_color: (41, 52, 65, 0).into(),
        grid_stroke_width: 1.0,

        series_stroke_width: DEFAULT_SERIES_STROKE_WIDTH,
        series_label_font_size: DEFAULT_FONT_SIZE,
        series_label_font_color: font_color,

        series_colors: vec![
            "#fc97af".into(),
            "#87f7cf".into(),
            "#f7f494".into(),
            "#72ccff".into(),
            "#f7c5a0".into(),
            "#d4a4eb".into(),
            "#d2f5a6".into(),
            "#76f2f2".into(),
        ],

        table_header_color: bg_color,
        table_body_colors: vec![bg_color.with_alpha(230)],
        table_border_color: (100, 100, 100).into(),
    }
});

static GRAFANA_THEME: Lazy<Theme> = Lazy::new(|| {
    let x_axis_color = (185, 184, 206).into();

    let font_color: Color = (216, 217, 218).into();
    let bg_color = (31, 29, 29).into();
    Theme {
        is_light: false,
        font_family: DEFAULT_FONT_FAMILY.to_string(),
        margin: (5.0).into(),
        width: DEFAULT_WIDTH,
        height: DEFAULT_HEIGHT,
        background_color: bg_color,

        title_font_color: font_color,
        title_font_size: 18.0,
        title_font_weight: Some("bold".to_string()),
        title_margin: None,
        title_align: Align::Center,
        title_height: DEFAULT_TITLE_HEIGHT,

        sub_title_font_color: font_color,
        sub_title_font_size: DEFAULT_FONT_SIZE,
        sub_title_margin: None,
        sub_title_align: Align::Center,
        sub_title_height: DEFAULT_SUB_TITLE_HEIGHT,

        legend_font_size: DEFAULT_FONT_SIZE,
        legend_font_color: font_color,
        legend_align: Align::Center,
        legend_margin: None,

        x_axis_font_size: DEFAULT_FONT_SIZE,
        x_axis_stroke_color: x_axis_color,
        x_axis_font_color: x_axis_color,
        x_axis_name_gap: DEFAULT_X_AXIS_NAME_GAP,
        x_axis_height: DEFAULT_X_AXIS_HEIGHT,

        y_axis_font_size: DEFAULT_FONT_SIZE,
        y_axis_font_color: x_axis_color,
        y_axis_stroke_color: Color::transparent(),
        y_axis_split_number: DEFAULT_Y_AXIS_SPLIT_NUMBER,
        y_axis_name_gap: DEFAULT_Y_AXIS_NAME_GAP,

        grid_stroke_color: (68, 67, 67).into(),
        grid_stroke_width: 1.0,

        series_stroke_width: DEFAULT_SERIES_STROKE_WIDTH,
        series_label_font_size: DEFAULT_FONT_SIZE,
        series_label_font_color: font_color,

        series_colors: vec![
            "#7EB26D".into(),
            "#EAB839".into(),
            "#6ED0E0".into(),
            "#EF843C".into(),
            "#E24D42".into(),
            "#1F78C1".into(),
            "#705DA0".into(),
            "#508642".into(),
        ],

        table_header_color: bg_color,
        table_body_colors: vec![bg_color.with_alpha(230)],
        table_border_color: (239, 239, 244).into(),
    }
});

static SHADCN_THEME: Lazy<Theme> = Lazy::new(|| {
    let x_axis_color = (39, 39, 42).into();

    let font_color: Color = (161, 161, 170).into();
    let bg_color = (9, 9, 11).into();
    Theme {
        is_light: false,
        font_family: DEFAULT_FONT_FAMILY.to_string(),
        margin: (5.0).into(),
        width: DEFAULT_WIDTH,
        height: DEFAULT_HEIGHT,
        background_color: bg_color,

        title_font_color: font_color,
        title_font_size: 18.0,
        title_font_weight: Some("bold".to_string()),
        title_margin: None,
        title_align: Align::Center,
        title_height: DEFAULT_TITLE_HEIGHT,

        sub_title_font_color: font_color,
        sub_title_font_size: DEFAULT_FONT_SIZE,
        sub_title_margin: None,
        sub_title_align: Align::Center,
        sub_title_height: DEFAULT_SUB_TITLE_HEIGHT,

        legend_font_size: DEFAULT_FONT_SIZE,
        legend_font_color: font_color,
        legend_align: Align::Center,
        legend_margin: None,

        x_axis_font_size: DEFAULT_FONT_SIZE,
        x_axis_stroke_color: x_axis_color,
        x_axis_font_color: font_color,
        x_axis_name_gap: DEFAULT_X_AXIS_NAME_GAP,
        x_axis_height: DEFAULT_X_AXIS_HEIGHT,

        y_axis_font_size: DEFAULT_FONT_SIZE,
        y_axis_font_color: font_color,
        y_axis_stroke_color: Color::transparent(),
        y_axis_split_number: DEFAULT_Y_AXIS_SPLIT_NUMBER,
        y_axis_name_gap: DEFAULT_Y_AXIS_NAME_GAP,

        grid_stroke_color: (39, 39, 42).into(),
        grid_stroke_width: 1.0,

        series_stroke_width: DEFAULT_SERIES_STROKE_WIDTH,
        series_label_font_size: DEFAULT_FONT_SIZE,
        series_label_font_color: font_color,

        series_colors: vec![
            "#2662d9".into(),
            "#e23670".into(),
            "#2eb88a".into(),
            "#e88c30".into(),
            "#af57db".into(),
            "#0e2014".into(),
            "#3b86f7".into(),
            "#f17e92".into(),
        ],

        table_header_color: bg_color.with_alpha(230),
        table_body_colors: vec![bg_color],
        table_border_color: (39, 39, 42).into(),
    }
});

type Themes = AHashMap<String, Arc<Theme>>;
static THEME_MAP: Lazy<ArcSwap<Themes>> = Lazy::new(|| {
    let mut m = AHashMap::new();
    m.insert("dark".to_string(), Arc::new(DARK_THEME.clone()));
    m.insert("ant".to_string(), Arc::new(ANT_THEME.clone()));
    m.insert("grafana".to_string(), Arc::new(GRAFANA_THEME.clone()));
    m.insert("vintage".to_string(), Arc::new(VINTAGE_THEME.clone()));
    m.insert("shine".to_string(), Arc::new(SHINE_THEME.clone()));
    m.insert("walden".to_string(), Arc::new(WALDEN_THEME.clone()));
    m.insert("westeros".to_string(), Arc::new(WESTEROS_THEME.clone()));
    m.insert("chalk".to_string(), Arc::new(CHALK_THEME.clone()));
    m.insert("shadcn".to_string(), Arc::new(SHADCN_THEME.clone()));
    m.insert("light".to_string(), Arc::new(LIGHT_THEME.clone()));
    ArcSwap::from_pointee(m)
});

/// Add theme of charts
pub fn add_theme(name: &str, data: Theme) {
    let mut m: Themes = AHashMap::new();
    for (name, data) in THEME_MAP.load().iter() {
        m.insert(name.to_string(), data.clone());
    }
    m.insert(name.to_string(), Arc::new(data));
    THEME_MAP.store(Arc::new(m))
}

/// Get the theme of charts
pub fn get_theme(theme: &str) -> Arc<Theme> {
    if let Some(theme) = THEME_MAP.load().get(theme) {
        theme.clone()
    } else {
        Arc::new(LIGHT_THEME.clone())
    }
}

/// List the theme name
pub fn list_theme_name() -> Vec<String> {
    let mut themes = vec![];
    for name in THEME_MAP.load().keys() {
        themes.push(name.to_string());
    }
    themes
}

/// Get default theme
pub fn get_default_theme_name() -> String {
    LIGHT_THEME_NAME.to_string()
}
