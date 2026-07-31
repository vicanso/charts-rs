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

use super::color::Color;
use super::common::Align;
use super::font::DEFAULT_FONT_FAMILY;
use super::util::Box;
use arc_swap::ArcSwap;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::LazyLock;

/// Default canvas width.
pub static DEFAULT_WIDTH: f32 = 600.0;
/// Default canvas height.
pub static DEFAULT_HEIGHT: f32 = 400.0;

/// Default height reserved for the title row.
pub static DEFAULT_TITLE_HEIGHT: f32 = 30.0;
/// Default height reserved for the sub-title row.
pub static DEFAULT_SUB_TITLE_HEIGHT: f32 = 20.0;

/// Default height reserved for the x axis block.
pub static DEFAULT_X_AXIS_HEIGHT: f32 = 30.0;
/// Default gap between the x axis line and its labels.
pub static DEFAULT_X_AXIS_NAME_GAP: f32 = 5.0;

/// Default width reserved for a y axis block.
pub static DEFAULT_Y_AXIS_WIDTH: f32 = 40.0;
/// Default gap between a y axis line and its labels.
pub static DEFAULT_Y_AXIS_NAME_GAP: f32 = 8.0;
/// Default number of intervals a y axis splits into.
pub static DEFAULT_Y_AXIS_SPLIT_NUMBER: usize = 6;
/// Default font size.
pub static DEFAULT_FONT_SIZE: f32 = 14.0;

/// Default stroke width of series lines.
pub static DEFAULT_SERIES_STROKE_WIDTH: f32 = 2.0;

/// The "light" theme name (the default theme).
pub static THEME_LIGHT: &str = "light";
/// The "dark" theme name.
pub static THEME_DARK: &str = "dark";
/// The "ant" theme name.
pub static THEME_ANT: &str = "ant";
/// The "vintage" theme name.
pub static THEME_VINTAGE: &str = "vintage";
/// The "shine" theme name.
pub static THEME_SHINE: &str = "shine";
/// The "walden" theme name.
pub static THEME_WALDEN: &str = "walden";
/// The "westeros" theme name.
pub static THEME_WESTEROS: &str = "westeros";
/// The "chalk" theme name.
pub static THEME_CHALK: &str = "chalk";
/// The "grafana" theme name.
pub static THEME_GRAFANA: &str = "grafana";
/// The "shadcn" theme name.
pub static THEME_SHADCN: &str = "shadcn";

// Internal alias for the default theme name.
static LIGHT_THEME_NAME: &str = THEME_LIGHT;

/// A named set of chart defaults (sizes, fonts, colors, palette). Charts copy
/// these values in `fill_theme` before user options are applied; custom themes
/// are registered with [`add_theme`] and referenced by name.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]

pub struct Theme {
    /// Whether this is a light theme.
    pub is_light: bool,
    /// Default font family.
    pub font_family: String,
    /// Default chart margin.
    pub margin: Box,
    /// Default canvas width.
    pub width: f32,
    /// Default canvas height.
    pub height: f32,
    /// Default background color.
    pub background_color: Color,

    // title
    /// Default title font size.
    pub title_font_size: f32,
    /// Default title font color.
    pub title_font_color: Color,
    /// Default title font weight.
    pub title_font_weight: Option<String>,
    /// Default title margin.
    pub title_margin: Option<Box>,
    /// Default title alignment.
    pub title_align: Align,
    /// Default title row height.
    pub title_height: f32,

    // sub title
    /// Default sub-title font size.
    pub sub_title_font_size: f32,
    /// Default sub-title font color.
    pub sub_title_font_color: Color,
    /// Default sub-title margin.
    pub sub_title_margin: Option<Box>,
    /// Default sub-title alignment.
    pub sub_title_align: Align,
    /// Default sub-title row height.
    pub sub_title_height: f32,

    // legend
    /// Default legend font size.
    pub legend_font_size: f32,
    /// Default legend font color.
    pub legend_font_color: Color,
    /// Default legend alignment.
    pub legend_align: Align,
    /// Default legend margin.
    pub legend_margin: Option<Box>,

    // x axis
    /// Default x axis label font size.
    pub x_axis_font_size: f32,
    /// Default x axis stroke color.
    pub x_axis_stroke_color: Color,
    /// Default x axis label font color.
    pub x_axis_font_color: Color,
    /// Default gap between the x axis line and its labels.
    pub x_axis_name_gap: f32,
    /// Default x axis block height.
    pub x_axis_height: f32,

    // y axis
    /// Default y axis label font size.
    pub y_axis_font_size: f32,
    /// Default y axis label font color.
    pub y_axis_font_color: Color,
    /// Default y axis stroke color.
    pub y_axis_stroke_color: Color,
    /// Default number of y axis intervals.
    pub y_axis_split_number: usize,
    /// Default gap between the y axis line and its labels.
    pub y_axis_name_gap: f32,

    // grid
    /// Default grid stroke color.
    pub grid_stroke_color: Color,
    /// Default grid stroke width.
    pub grid_stroke_width: f32,

    // series
    /// Default stroke width of series lines.
    pub series_stroke_width: f32,
    /// Default series label font size.
    pub series_label_font_size: f32,
    /// Default series label font color.
    pub series_label_font_color: Color,
    /// Default series color palette.
    pub series_colors: Vec<Color>,

    // table
    /// Default table header background color.
    pub table_header_color: Color,
    /// Default table body background colors.
    pub table_body_colors: Vec<Color>,
    /// Default table border color.
    pub table_border_color: Color,
}

static LIGHT_THEME: LazyLock<Theme> = LazyLock::new(|| {
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

static DARK_THEME: LazyLock<Theme> = LazyLock::new(|| {
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

static ANT_THEME: LazyLock<Theme> = LazyLock::new(|| {
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

static VINTAGE_THEME: LazyLock<Theme> = LazyLock::new(|| {
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

static SHINE_THEME: LazyLock<Theme> = LazyLock::new(|| {
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

static WALDEN_THEME: LazyLock<Theme> = LazyLock::new(|| {
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

static WESTEROS_THEME: LazyLock<Theme> = LazyLock::new(|| {
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

static CHALK_THEME: LazyLock<Theme> = LazyLock::new(|| {
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

static GRAFANA_THEME: LazyLock<Theme> = LazyLock::new(|| {
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

static SHADCN_THEME: LazyLock<Theme> = LazyLock::new(|| {
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

type Themes = HashMap<String, Arc<Theme>>;
static LIGHT_THEME_ARC: LazyLock<Arc<Theme>> = LazyLock::new(|| Arc::new(LIGHT_THEME.clone()));
static THEME_MAP: LazyLock<ArcSwap<Themes>> = LazyLock::new(|| {
    let mut m = HashMap::new();
    m.insert("dark".to_string(), Arc::new(DARK_THEME.clone()));
    m.insert("ant".to_string(), Arc::new(ANT_THEME.clone()));
    m.insert("grafana".to_string(), Arc::new(GRAFANA_THEME.clone()));
    m.insert("vintage".to_string(), Arc::new(VINTAGE_THEME.clone()));
    m.insert("shine".to_string(), Arc::new(SHINE_THEME.clone()));
    m.insert("walden".to_string(), Arc::new(WALDEN_THEME.clone()));
    m.insert("westeros".to_string(), Arc::new(WESTEROS_THEME.clone()));
    m.insert("chalk".to_string(), Arc::new(CHALK_THEME.clone()));
    m.insert("shadcn".to_string(), Arc::new(SHADCN_THEME.clone()));
    m.insert("light".to_string(), Arc::clone(&LIGHT_THEME_ARC));
    ArcSwap::from_pointee(m)
});

/// Add theme of charts
pub fn add_theme(name: &str, data: Theme) {
    let mut m: Themes = (**THEME_MAP.load()).clone();
    m.insert(name.to_string(), Arc::new(data));
    THEME_MAP.store(Arc::new(m));
}

/// Get the theme of charts
pub fn get_theme(theme: &str) -> Arc<Theme> {
    if let Some(theme) = THEME_MAP.load().get(theme) {
        Arc::clone(theme)
    } else {
        Arc::clone(&LIGHT_THEME_ARC)
    }
}

/// List the theme name
pub fn list_theme_name() -> Vec<String> {
    THEME_MAP.load().keys().cloned().collect()
}

/// Get default theme
pub fn get_default_theme_name() -> String {
    LIGHT_THEME_NAME.to_string()
}
