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
use super::common::{
    Align, FontConfig, GridConfig, LegendConfig, SeriesConfig, SeriesLabelConfig, TitleConfig,
    XAxisConfig, YAxisConfig,
};
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

    /// Defaults of the title.
    pub title: TitleConfig,
    /// Defaults of the sub-title.
    pub sub_title: TitleConfig,
    /// Defaults of the legend.
    pub legend: LegendConfig,
    /// Defaults of the x axis. Its `data` are the labels of a chart: what a
    /// theme has there is not used.
    pub x_axis: XAxisConfig,
    /// Defaults of every y axis.
    pub y_axis: YAxisConfig,
    /// Defaults of the grid lines.
    pub grid: GridConfig,
    /// Defaults of the series. Without a `symbol`, the points of a line are
    /// circles of the stroke width, filled with the background color.
    pub series: SeriesConfig,

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
        title: TitleConfig {
            font: FontConfig {
                size: 18.0,
                color: font_color,
                weight: Some("bold".to_string()),
            },
            margin: None,
            align: Align::Center,
            height: DEFAULT_TITLE_HEIGHT,
            ..Default::default()
        },
        sub_title: TitleConfig {
            font: FontConfig {
                size: DEFAULT_FONT_SIZE,
                color: font_color,
                ..Default::default()
            },
            margin: None,
            align: Align::Center,
            height: DEFAULT_SUB_TITLE_HEIGHT,
            ..Default::default()
        },
        legend: LegendConfig {
            font: FontConfig {
                size: DEFAULT_FONT_SIZE,
                color: font_color,
                ..Default::default()
            },
            align: Align::Center,
            margin: None,
            ..Default::default()
        },
        x_axis: XAxisConfig {
            font: FontConfig {
                size: DEFAULT_FONT_SIZE,
                color: x_axis_color,
                ..Default::default()
            },
            stroke_color: x_axis_color,
            name_gap: DEFAULT_X_AXIS_NAME_GAP,
            height: DEFAULT_X_AXIS_HEIGHT,
            ..Default::default()
        },
        y_axis: YAxisConfig {
            font: FontConfig {
                size: DEFAULT_FONT_SIZE,
                color: x_axis_color,
                ..Default::default()
            },
            stroke_color: Color::transparent(),
            split_number: DEFAULT_Y_AXIS_SPLIT_NUMBER,
            name_gap: DEFAULT_Y_AXIS_NAME_GAP,
            ..Default::default()
        },
        grid: GridConfig {
            stroke_color: (224, 230, 242).into(),
            stroke_width: 1.0,
            ..Default::default()
        },
        series: SeriesConfig {
            label: SeriesLabelConfig {
                font: FontConfig {
                    size: DEFAULT_FONT_SIZE,
                    color: font_color,
                    ..Default::default()
                },
                ..Default::default()
            },
            stroke_width: DEFAULT_SERIES_STROKE_WIDTH,
            colors: vec![
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
            ..Default::default()
        },
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
        title: TitleConfig {
            font: FontConfig {
                size: 18.0,
                color: font_color,
                weight: Some("bold".to_string()),
            },
            margin: None,
            align: Align::Center,
            height: DEFAULT_TITLE_HEIGHT,
            ..Default::default()
        },
        sub_title: TitleConfig {
            font: FontConfig {
                size: DEFAULT_FONT_SIZE,
                color: font_color,
                ..Default::default()
            },
            margin: None,
            align: Align::Center,
            height: DEFAULT_SUB_TITLE_HEIGHT,
            ..Default::default()
        },
        legend: LegendConfig {
            font: FontConfig {
                size: DEFAULT_FONT_SIZE,
                color: font_color,
                ..Default::default()
            },
            align: Align::Center,
            margin: None,
            ..Default::default()
        },
        x_axis: XAxisConfig {
            font: FontConfig {
                size: DEFAULT_FONT_SIZE,
                color: x_axis_color,
                ..Default::default()
            },
            stroke_color: x_axis_color,
            name_gap: DEFAULT_X_AXIS_NAME_GAP,
            height: DEFAULT_X_AXIS_HEIGHT,
            ..Default::default()
        },
        y_axis: YAxisConfig {
            font: FontConfig {
                size: DEFAULT_FONT_SIZE,
                color: x_axis_color,
                ..Default::default()
            },
            stroke_color: Color::transparent(),
            split_number: DEFAULT_Y_AXIS_SPLIT_NUMBER,
            name_gap: DEFAULT_Y_AXIS_NAME_GAP,
            ..Default::default()
        },
        grid: GridConfig {
            stroke_color: (71, 71, 83).into(),
            stroke_width: 1.0,
            ..Default::default()
        },
        series: SeriesConfig {
            label: SeriesLabelConfig {
                font: FontConfig {
                    size: DEFAULT_FONT_SIZE,
                    color: font_color,
                    ..Default::default()
                },
                ..Default::default()
            },
            stroke_width: DEFAULT_SERIES_STROKE_WIDTH,
            colors: vec![
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
            ..Default::default()
        },
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
        title: TitleConfig {
            font: FontConfig {
                size: 18.0,
                color: font_color,
                weight: Some("bold".to_string()),
            },
            margin: None,
            align: Align::Center,
            height: DEFAULT_TITLE_HEIGHT,
            ..Default::default()
        },
        sub_title: TitleConfig {
            font: FontConfig {
                size: DEFAULT_FONT_SIZE,
                color: font_color,
                ..Default::default()
            },
            margin: None,
            align: Align::Center,
            height: DEFAULT_SUB_TITLE_HEIGHT,
            ..Default::default()
        },
        legend: LegendConfig {
            font: FontConfig {
                size: DEFAULT_FONT_SIZE,
                color: font_color,
                ..Default::default()
            },
            align: Align::Center,
            margin: None,
            ..Default::default()
        },
        x_axis: XAxisConfig {
            font: FontConfig {
                size: DEFAULT_FONT_SIZE,
                color: x_axis_color,
                ..Default::default()
            },
            stroke_color: x_axis_color,
            name_gap: DEFAULT_X_AXIS_NAME_GAP,
            height: DEFAULT_X_AXIS_HEIGHT,
            ..Default::default()
        },
        y_axis: YAxisConfig {
            font: FontConfig {
                size: DEFAULT_FONT_SIZE,
                color: x_axis_color,
                ..Default::default()
            },
            stroke_color: Color::transparent(),
            split_number: DEFAULT_Y_AXIS_SPLIT_NUMBER,
            name_gap: DEFAULT_Y_AXIS_NAME_GAP,
            ..Default::default()
        },
        grid: GridConfig {
            stroke_color: (224, 230, 242).into(),
            stroke_width: 1.0,
            ..Default::default()
        },
        series: SeriesConfig {
            label: SeriesLabelConfig {
                font: FontConfig {
                    size: DEFAULT_FONT_SIZE,
                    color: font_color,
                    ..Default::default()
                },
                ..Default::default()
            },
            stroke_width: DEFAULT_SERIES_STROKE_WIDTH,
            colors: vec![
                "#5b8ff9".into(),
                "#5ad8a6".into(),
                "#5d7092".into(),
                "#f6bd16".into(),
                "#6f5ef9".into(),
                "#6dc8ec".into(),
                "#945fb9".into(),
                "#ff9845".into(),
            ],
            ..Default::default()
        },
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
        title: TitleConfig {
            font: FontConfig {
                size: 18.0,
                color: font_color,
                weight: Some("bold".to_string()),
            },
            margin: None,
            align: Align::Center,
            height: DEFAULT_TITLE_HEIGHT,
            ..Default::default()
        },
        sub_title: TitleConfig {
            font: FontConfig {
                size: DEFAULT_FONT_SIZE,
                color: font_color,
                ..Default::default()
            },
            margin: None,
            align: Align::Center,
            height: DEFAULT_SUB_TITLE_HEIGHT,
            ..Default::default()
        },
        legend: LegendConfig {
            font: FontConfig {
                size: DEFAULT_FONT_SIZE,
                color: font_color,
                ..Default::default()
            },
            align: Align::Center,
            margin: None,
            ..Default::default()
        },
        x_axis: XAxisConfig {
            font: FontConfig {
                size: DEFAULT_FONT_SIZE,
                color: x_axis_color,
                ..Default::default()
            },
            stroke_color: x_axis_color,
            name_gap: DEFAULT_X_AXIS_NAME_GAP,
            height: DEFAULT_X_AXIS_HEIGHT,
            ..Default::default()
        },
        y_axis: YAxisConfig {
            font: FontConfig {
                size: DEFAULT_FONT_SIZE,
                color: x_axis_color,
                ..Default::default()
            },
            stroke_color: Color::transparent(),
            split_number: DEFAULT_Y_AXIS_SPLIT_NUMBER,
            name_gap: DEFAULT_Y_AXIS_NAME_GAP,
            ..Default::default()
        },
        grid: GridConfig {
            stroke_color: (224, 230, 242).into(),
            stroke_width: 1.0,
            ..Default::default()
        },
        series: SeriesConfig {
            label: SeriesLabelConfig {
                font: FontConfig {
                    size: DEFAULT_FONT_SIZE,
                    color: font_color,
                    ..Default::default()
                },
                ..Default::default()
            },
            stroke_width: DEFAULT_SERIES_STROKE_WIDTH,
            colors: vec![
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
            ..Default::default()
        },
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
        title: TitleConfig {
            font: FontConfig {
                size: 18.0,
                color: font_color,
                weight: Some("bold".to_string()),
            },
            margin: None,
            align: Align::Center,
            height: DEFAULT_TITLE_HEIGHT,
            ..Default::default()
        },
        sub_title: TitleConfig {
            font: FontConfig {
                size: DEFAULT_FONT_SIZE,
                color: font_color,
                ..Default::default()
            },
            margin: None,
            align: Align::Center,
            height: DEFAULT_SUB_TITLE_HEIGHT,
            ..Default::default()
        },
        legend: LegendConfig {
            font: FontConfig {
                size: DEFAULT_FONT_SIZE,
                color: font_color,
                ..Default::default()
            },
            align: Align::Center,
            margin: None,
            ..Default::default()
        },
        x_axis: XAxisConfig {
            font: FontConfig {
                size: DEFAULT_FONT_SIZE,
                color: x_axis_color,
                ..Default::default()
            },
            stroke_color: x_axis_color,
            name_gap: DEFAULT_X_AXIS_NAME_GAP,
            height: DEFAULT_X_AXIS_HEIGHT,
            ..Default::default()
        },
        y_axis: YAxisConfig {
            font: FontConfig {
                size: DEFAULT_FONT_SIZE,
                color: x_axis_color,
                ..Default::default()
            },
            stroke_color: Color::transparent(),
            split_number: DEFAULT_Y_AXIS_SPLIT_NUMBER,
            name_gap: DEFAULT_Y_AXIS_NAME_GAP,
            ..Default::default()
        },
        grid: GridConfig {
            stroke_color: (224, 230, 242).into(),
            stroke_width: 1.0,
            ..Default::default()
        },
        series: SeriesConfig {
            label: SeriesLabelConfig {
                font: FontConfig {
                    size: DEFAULT_FONT_SIZE,
                    color: font_color,
                    ..Default::default()
                },
                ..Default::default()
            },
            stroke_width: DEFAULT_SERIES_STROKE_WIDTH,
            colors: vec![
                "#c12e34".into(),
                "#e6b600".into(),
                "#0098d9".into(),
                "#2b821d".into(),
                "#005eaa".into(),
                "#339ca8".into(),
                "#cda819".into(),
                "#32a487".into(),
            ],
            ..Default::default()
        },
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
        title: TitleConfig {
            font: FontConfig {
                size: 18.0,
                color: font_color,
                weight: Some("bold".to_string()),
            },
            margin: None,
            align: Align::Center,
            height: DEFAULT_TITLE_HEIGHT,
            ..Default::default()
        },
        sub_title: TitleConfig {
            font: FontConfig {
                size: DEFAULT_FONT_SIZE,
                color: font_color,
                ..Default::default()
            },
            margin: None,
            align: Align::Center,
            height: DEFAULT_SUB_TITLE_HEIGHT,
            ..Default::default()
        },
        legend: LegendConfig {
            font: FontConfig {
                size: DEFAULT_FONT_SIZE,
                color: font_color,
                ..Default::default()
            },
            align: Align::Center,
            margin: None,
            ..Default::default()
        },
        x_axis: XAxisConfig {
            font: FontConfig {
                size: DEFAULT_FONT_SIZE,
                color: x_axis_color,
                ..Default::default()
            },
            stroke_color: x_axis_color,
            name_gap: DEFAULT_X_AXIS_NAME_GAP,
            height: DEFAULT_X_AXIS_HEIGHT,
            ..Default::default()
        },
        y_axis: YAxisConfig {
            font: FontConfig {
                size: DEFAULT_FONT_SIZE,
                color: x_axis_color,
                ..Default::default()
            },
            stroke_color: Color::transparent(),
            split_number: DEFAULT_Y_AXIS_SPLIT_NUMBER,
            name_gap: DEFAULT_Y_AXIS_NAME_GAP,
            ..Default::default()
        },
        grid: GridConfig {
            stroke_color: (224, 230, 242).into(),
            stroke_width: 1.0,
            ..Default::default()
        },
        series: SeriesConfig {
            label: SeriesLabelConfig {
                font: FontConfig {
                    size: DEFAULT_FONT_SIZE,
                    color: font_color,
                    ..Default::default()
                },
                ..Default::default()
            },
            stroke_width: DEFAULT_SERIES_STROKE_WIDTH,
            colors: vec![
                "#3fb1e3".into(),
                "#6be6c1".into(),
                "#626c91".into(),
                "#a0a7e6".into(),
                "#c4ebad".into(),
                "#96dee8".into(),
            ],
            ..Default::default()
        },
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
        title: TitleConfig {
            font: FontConfig {
                size: 18.0,
                color: font_color,
                weight: Some("bold".to_string()),
            },
            margin: None,
            align: Align::Center,
            height: DEFAULT_TITLE_HEIGHT,
            ..Default::default()
        },
        sub_title: TitleConfig {
            font: FontConfig {
                size: DEFAULT_FONT_SIZE,
                color: font_color,
                ..Default::default()
            },
            margin: None,
            align: Align::Center,
            height: DEFAULT_SUB_TITLE_HEIGHT,
            ..Default::default()
        },
        legend: LegendConfig {
            font: FontConfig {
                size: DEFAULT_FONT_SIZE,
                color: font_color,
                ..Default::default()
            },
            align: Align::Center,
            margin: None,
            ..Default::default()
        },
        x_axis: XAxisConfig {
            font: FontConfig {
                size: DEFAULT_FONT_SIZE,
                color: x_axis_color,
                ..Default::default()
            },
            stroke_color: x_axis_color,
            name_gap: DEFAULT_X_AXIS_NAME_GAP,
            height: DEFAULT_X_AXIS_HEIGHT,
            ..Default::default()
        },
        y_axis: YAxisConfig {
            font: FontConfig {
                size: DEFAULT_FONT_SIZE,
                color: x_axis_color,
                ..Default::default()
            },
            stroke_color: Color::transparent(),
            split_number: DEFAULT_Y_AXIS_SPLIT_NUMBER,
            name_gap: DEFAULT_Y_AXIS_NAME_GAP,
            ..Default::default()
        },
        grid: GridConfig {
            stroke_color: (224, 230, 242).into(),
            stroke_width: 1.0,
            ..Default::default()
        },
        series: SeriesConfig {
            label: SeriesLabelConfig {
                font: FontConfig {
                    size: DEFAULT_FONT_SIZE,
                    color: font_color,
                    ..Default::default()
                },
                ..Default::default()
            },
            stroke_width: DEFAULT_SERIES_STROKE_WIDTH,
            colors: vec![
                "#516b91".into(),
                "#59c4e6".into(),
                "#edafda".into(),
                "#93b7e3".into(),
                "#a5e7f0".into(),
                "#cbb0e3".into(),
            ],
            ..Default::default()
        },
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
        title: TitleConfig {
            font: FontConfig {
                size: 18.0,
                color: font_color,
                weight: Some("bold".to_string()),
            },
            margin: None,
            align: Align::Center,
            height: DEFAULT_TITLE_HEIGHT,
            ..Default::default()
        },
        sub_title: TitleConfig {
            font: FontConfig {
                size: DEFAULT_FONT_SIZE,
                color: font_color,
                ..Default::default()
            },
            margin: None,
            align: Align::Center,
            height: DEFAULT_SUB_TITLE_HEIGHT,
            ..Default::default()
        },
        legend: LegendConfig {
            font: FontConfig {
                size: DEFAULT_FONT_SIZE,
                color: font_color,
                ..Default::default()
            },
            align: Align::Center,
            margin: None,
            ..Default::default()
        },
        x_axis: XAxisConfig {
            font: FontConfig {
                size: DEFAULT_FONT_SIZE,
                color: x_axis_color,
                ..Default::default()
            },
            stroke_color: x_axis_color,
            name_gap: DEFAULT_X_AXIS_NAME_GAP,
            height: DEFAULT_X_AXIS_HEIGHT,
            ..Default::default()
        },
        y_axis: YAxisConfig {
            font: FontConfig {
                size: DEFAULT_FONT_SIZE,
                color: x_axis_color,
                ..Default::default()
            },
            stroke_color: Color::transparent(),
            split_number: DEFAULT_Y_AXIS_SPLIT_NUMBER,
            name_gap: DEFAULT_Y_AXIS_NAME_GAP,
            ..Default::default()
        },
        grid: GridConfig {
            stroke_color: (41, 52, 65, 0).into(),
            stroke_width: 1.0,
            ..Default::default()
        },
        series: SeriesConfig {
            label: SeriesLabelConfig {
                font: FontConfig {
                    size: DEFAULT_FONT_SIZE,
                    color: font_color,
                    ..Default::default()
                },
                ..Default::default()
            },
            stroke_width: DEFAULT_SERIES_STROKE_WIDTH,
            colors: vec![
                "#fc97af".into(),
                "#87f7cf".into(),
                "#f7f494".into(),
                "#72ccff".into(),
                "#f7c5a0".into(),
                "#d4a4eb".into(),
                "#d2f5a6".into(),
                "#76f2f2".into(),
            ],
            ..Default::default()
        },
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
        title: TitleConfig {
            font: FontConfig {
                size: 18.0,
                color: font_color,
                weight: Some("bold".to_string()),
            },
            margin: None,
            align: Align::Center,
            height: DEFAULT_TITLE_HEIGHT,
            ..Default::default()
        },
        sub_title: TitleConfig {
            font: FontConfig {
                size: DEFAULT_FONT_SIZE,
                color: font_color,
                ..Default::default()
            },
            margin: None,
            align: Align::Center,
            height: DEFAULT_SUB_TITLE_HEIGHT,
            ..Default::default()
        },
        legend: LegendConfig {
            font: FontConfig {
                size: DEFAULT_FONT_SIZE,
                color: font_color,
                ..Default::default()
            },
            align: Align::Center,
            margin: None,
            ..Default::default()
        },
        x_axis: XAxisConfig {
            font: FontConfig {
                size: DEFAULT_FONT_SIZE,
                color: x_axis_color,
                ..Default::default()
            },
            stroke_color: x_axis_color,
            name_gap: DEFAULT_X_AXIS_NAME_GAP,
            height: DEFAULT_X_AXIS_HEIGHT,
            ..Default::default()
        },
        y_axis: YAxisConfig {
            font: FontConfig {
                size: DEFAULT_FONT_SIZE,
                color: x_axis_color,
                ..Default::default()
            },
            stroke_color: Color::transparent(),
            split_number: DEFAULT_Y_AXIS_SPLIT_NUMBER,
            name_gap: DEFAULT_Y_AXIS_NAME_GAP,
            ..Default::default()
        },
        grid: GridConfig {
            stroke_color: (68, 67, 67).into(),
            stroke_width: 1.0,
            ..Default::default()
        },
        series: SeriesConfig {
            label: SeriesLabelConfig {
                font: FontConfig {
                    size: DEFAULT_FONT_SIZE,
                    color: font_color,
                    ..Default::default()
                },
                ..Default::default()
            },
            stroke_width: DEFAULT_SERIES_STROKE_WIDTH,
            colors: vec![
                "#7EB26D".into(),
                "#EAB839".into(),
                "#6ED0E0".into(),
                "#EF843C".into(),
                "#E24D42".into(),
                "#1F78C1".into(),
                "#705DA0".into(),
                "#508642".into(),
            ],
            ..Default::default()
        },
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
        title: TitleConfig {
            font: FontConfig {
                size: 18.0,
                color: font_color,
                weight: Some("bold".to_string()),
            },
            margin: None,
            align: Align::Center,
            height: DEFAULT_TITLE_HEIGHT,
            ..Default::default()
        },
        sub_title: TitleConfig {
            font: FontConfig {
                size: DEFAULT_FONT_SIZE,
                color: font_color,
                ..Default::default()
            },
            margin: None,
            align: Align::Center,
            height: DEFAULT_SUB_TITLE_HEIGHT,
            ..Default::default()
        },
        legend: LegendConfig {
            font: FontConfig {
                size: DEFAULT_FONT_SIZE,
                color: font_color,
                ..Default::default()
            },
            align: Align::Center,
            margin: None,
            ..Default::default()
        },
        x_axis: XAxisConfig {
            font: FontConfig {
                size: DEFAULT_FONT_SIZE,
                color: font_color,
                ..Default::default()
            },
            stroke_color: x_axis_color,
            name_gap: DEFAULT_X_AXIS_NAME_GAP,
            height: DEFAULT_X_AXIS_HEIGHT,
            ..Default::default()
        },
        y_axis: YAxisConfig {
            font: FontConfig {
                size: DEFAULT_FONT_SIZE,
                color: font_color,
                ..Default::default()
            },
            stroke_color: Color::transparent(),
            split_number: DEFAULT_Y_AXIS_SPLIT_NUMBER,
            name_gap: DEFAULT_Y_AXIS_NAME_GAP,
            ..Default::default()
        },
        grid: GridConfig {
            stroke_color: (39, 39, 42).into(),
            stroke_width: 1.0,
            ..Default::default()
        },
        series: SeriesConfig {
            label: SeriesLabelConfig {
                font: FontConfig {
                    size: DEFAULT_FONT_SIZE,
                    color: font_color,
                    ..Default::default()
                },
                ..Default::default()
            },
            stroke_width: DEFAULT_SERIES_STROKE_WIDTH,
            colors: vec![
                "#2662d9".into(),
                "#e23670".into(),
                "#2eb88a".into(),
                "#e88c30".into(),
                "#af57db".into(),
                "#0e2014".into(),
                "#3b86f7".into(),
                "#f17e92".into(),
            ],
            ..Default::default()
        },
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
