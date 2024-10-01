// This Source Code Form is subject to the terms of the Mozilla Public
// License, v. 2.0. If a copy of the MPL was not distributed with this
// file, You can obtain one at http://mozilla.org/MPL/2.0/.

//! Charts support multi charts: bar chart, horizontal bar chart,
//! line chart, pie chart, radar chart and table.
//!
//! It supports four themes and very easy to use.
//! Each attribute can be customized, it can be save as svg or png.
//! The chart can be new from json, which sets default value if the field is undefined.
//!
//! # New a bar chart from json, the other charts also support the function.
//! ```rust
//! use charts_rs::{BarChart};
//! let bar_chart = BarChart::from_json(
//!     r###"{
//!         "width": 630,
//!         "height": 410,
//!         "margin": {
//!             "left": 10,
//!             "top": 5,
//!             "right": 10
//!         },
//!         "title_text": "Bar Chart",
//!         "title_font_color": "#345",
//!         "title_align": "right",
//!         "sub_title_text": "demo",
//!         "legend_align": "left",
//!         "series_list": [
//!             {
//!                 "name": "Email",
//!                 "label_show": true,
//!                 "data": [120.0, 132.0, 101.0, 134.0, 90.0, 230.0, 210.0]
//!             },
//!             {
//!                 "name": "Union Ads",
//!                 "data": [220.0, 182.0, 191.0, 234.0, 290.0, 330.0, 310.0]
//!             }
//!         ],
//!         "x_axis_data": [
//!             "Mon",
//!             "Tue",
//!             "Wed",
//!             "Thu",
//!             "Fri",
//!             "Sat",
//!             "Sun"
//!         ]
//!     }"###,
//! ).unwrap();
//! println!("{}", bar_chart.svg().unwrap());
//! ```
//!
//! # New bar chart with theme
//!
//! There are four themes: echart, dark, ant and grafana.
//! The echart is default theme.
//! ```rust
//! use charts_rs::{BarChart, Series, THEME_GRAFANA};
//! let bar_chart = BarChart::new_with_theme(
//!     vec![
//!         ("Email", vec![120.0, 132.0, 101.0, 134.0, 90.0, 230.0, 210.0]).into(),
//!     ],
//!     vec![
//!         "Mon".to_string(),
//!         "Tue".to_string(),
//!         "Wed".to_string(),
//!         "Thu".to_string(),
//!         "Fri".to_string(),
//!         "Sat".to_string(),
//!         "Sun".to_string(),
//!     ],
//!     THEME_GRAFANA,
//! );
//! println!("{}", bar_chart.svg().unwrap());
//!```
//!
//!
//! # Add more font
//! The fonts will be initialized once, it can be changed before used.
//! ```rust
//! use charts_rs::{get_or_try_init_fonts, BarChart};
//! let data = include_bytes!("./Roboto.ttf") as &[u8];
//! get_or_try_init_fonts(Some(vec![data])).unwrap();
//! let bar_chart = BarChart::from_json(
//!     r###"{
//!         "width": 630,
//!         "height": 410,
//!         "font_family": "test",
//!         "margin": {
//!             "left": 10,
//!             "top": 5,
//!             "right": 10
//!         },
//!         "title_text": "Bar Chart",
//!         "title_font_color": "#345",
//!         "title_align": "right",
//!         "sub_title_text": "demo",
//!         "legend_align": "left",
//!         "series_list": [
//!             {
//!                 "name": "Email",
//!                 "label_show": true,
//!                 "data": [120.0, 132.0, 101.0, 134.0, 90.0, 230.0, 210.0]
//!             },
//!             {
//!                 "name": "Union Ads",
//!                 "data": [220.0, 182.0, 191.0, 234.0, 290.0, 330.0, 310.0]
//!             }
//!         ],
//!         "x_axis_data": [
//!             "Mon",
//!             "Tue",
//!             "Wed",
//!             "Thu",
//!             "Fri",
//!             "Sat",
//!             "Sun"
//!         ]
//!     }"###,
//! ).unwrap();
//! println!("{}", bar_chart.svg().unwrap());
//! ```
//!
//! # Basic bar chart
//! ```rust
//! use charts_rs::{BarChart, Series, Box};
//! let mut bar_chart = BarChart::new(
//!     vec![
//!         Series::new(
//!             "Email".to_string(),
//!             vec![120.0, 132.0, 101.0, 134.0, 90.0, 230.0, 210.0],
//!         ),
//!         Series::new(
//!             "Union Ads".to_string(),
//!             vec![220.0, 182.0, 191.0, 234.0, 290.0, 330.0, 310.0],
//!         ),
//!         Series::new(
//!             "Direct".to_string(),
//!             vec![320.0, 332.0, 301.0, 334.0, 390.0, 330.0, 320.0],
//!         ),
//!         Series::new(
//!             "Search Engine".to_string(),
//!             vec![820.0, 932.0, 901.0, 934.0, 1290.0, 1330.0, 1320.0],
//!         )
//!     ],
//!     vec![
//!         "Mon".to_string(),
//!         "Tue".to_string(),
//!         "Wed".to_string(),
//!         "Thu".to_string(),
//!         "Fri".to_string(),
//!         "Sat".to_string(),
//!         "Sun".to_string(),
//!     ],
//! );
//! bar_chart.y_axis_configs[0].axis_width = Some(55.0);
//! bar_chart.title_text = "Bar Chart".to_string();
//! bar_chart.legend_margin = Some(Box {
//!     top: 35.0,
//!     bottom: 10.0,
//!     ..Default::default()
//! });
//! println!("{}", bar_chart.svg().unwrap());
//!```
//!
//! # Bar line mixin chart
//! ```rust
//! use charts_rs::{BarChart, Series, Box, SeriesCategory};
//! let mut bar_chart = BarChart::new(
//!     vec![
//!         Series::new(
//!             "Email".to_string(),
//!             vec![120.0, 132.0, 101.0, 134.0, 90.0, 230.0, 210.0],
//!         ),
//!         Series::new(
//!             "Union Ads".to_string(),
//!             vec![220.0, 182.0, 191.0, 234.0, 290.0, 330.0, 310.0],
//!         ),
//!         Series::new(
//!             "Direct".to_string(),
//!             vec![320.0, 332.0, 301.0, 334.0, 390.0, 330.0, 320.0],
//!         ),
//!         Series::new(
//!             "Search Engine".to_string(),
//!             vec![820.0, 932.0, 901.0, 934.0, 1290.0, 1330.0, 1320.0],
//!         )
//!     ],
//!     vec![
//!         "Mon".to_string(),
//!         "Tue".to_string(),
//!         "Wed".to_string(),
//!         "Thu".to_string(),
//!         "Fri".to_string(),
//!         "Sat".to_string(),
//!         "Sun".to_string(),
//!     ],
//! );
//! bar_chart.series_list[0].category = Some(SeriesCategory::Line);
//! bar_chart.y_axis_configs[0].axis_width = Some(55.0);
//! bar_chart.title_text = "Bar Line Chart".to_string();
//! bar_chart.legend_margin = Some(Box {
//!     top: 35.0,
//!     bottom: 10.0,
//!     ..Default::default()
//! });
//! println!("{}", bar_chart.svg().unwrap());
//! ```
//!
//!
//! # Basic horizontal bar
//! ```rust
//! use charts_rs::{HorizontalBarChart, Series, Align};
//! let mut horizontal_bar_chart = HorizontalBarChart::new(
//!     vec![
//!         Series::new(
//!             "2011".to_string(),
//!             vec![18203.0, 23489.0, 29034.0, 104970.0, 131744.0, 630230.0],
//!         ),
//!         Series::new(
//!             "2012".to_string(),
//!             vec![19325.0, 23438.0, 31000.0, 121594.0, 134141.0, 681807.0],
//!         ),
//!     ],
//!     vec![
//!         "Brazil".to_string(),
//!         "Indonesia".to_string(),
//!         "USA".to_string(),
//!         "India".to_string(),
//!         "China".to_string(),
//!         "World".to_string(),
//!     ],
//! );
//! horizontal_bar_chart.title_text = "World Population".to_string();
//! horizontal_bar_chart.margin.right = 15.0;
//! horizontal_bar_chart.series_list[0].label_show = true;
//! horizontal_bar_chart.title_align = Align::Left;
//! println!("{}", horizontal_bar_chart.svg().unwrap());
//! ```
//!
//! # Basic line chart
//! ```rust
//! use charts_rs::{LineChart, Box};
//! let mut line_chart = LineChart::new(
//!     vec![
//!         ("Email", vec![120.0, 132.0, 101.0, 134.0, 90.0, 230.0, 210.0]).into(),
//!         ("Union Ads", vec![220.0, 182.0, 191.0, 234.0, 290.0, 330.0, 310.0]).into(),
//!         ("Direct", vec![320.0, 332.0, 301.0, 334.0, 390.0, 330.0, 320.0]).into(),
//!         ("Search Engine", vec![820.0, 932.0, 901.0, 934.0, 1290.0, 1330.0, 1320.0]).into(),
//!     ],
//!     vec![
//!         "Mon".to_string(),
//!         "Tue".to_string(),
//!         "Wed".to_string(),
//!         "Thu".to_string(),
//!         "Fri".to_string(),
//!         "Sat".to_string(),
//!         "Sun".to_string(),
//!     ],
//! );
//! line_chart.title_text = "Stacked Area Chart".to_string();
//! line_chart.sub_title_text = "Hello World".to_string();
//! line_chart.legend_margin = Some(Box {
//!     top: 50.0,
//!     bottom: 10.0,
//!     ..Default::default()
//! });
//! line_chart.series_list[3].label_show = true;
//! println!("{}", line_chart.svg().unwrap());
//! ```
//!
//! # Basic pie chart
//! ```rust
//! use charts_rs::{PieChart};
//! let mut pie_chart = PieChart::new(vec![
//!     ("rose 1", vec![40.0]).into(),
//!     ("rose 2", vec![38.0]).into(),
//!     ("rose 3", vec![32.0]).into(),
//!     ("rose 4", vec![30.0]).into(),
//!     ("rose 5", vec![28.0]).into(),
//!     ("rose 6", vec![26.0]).into(),
//!     ("rose 7", vec![22.0]).into(),
//!     ("rose 8", vec![18.0]).into(),
//! ]);
//! pie_chart.title_text = "Nightingale Chart".to_string();
//! pie_chart.sub_title_text = "Fake Data".to_string();
//! println!("{}", pie_chart.svg().unwrap());
//! ```
//!
//! # Basic radar chart
//! ```rust
//! use charts_rs::{RadarChart};
//! let radar_chart = RadarChart::new(
//!     vec![
//!         ("Allocated Budget", vec![4200.0, 3000.0, 20000.0, 35000.0, 50000.0, 18000.0, 9000.0]).into(),
//!         ("Actual Spending", vec![5000.0, 14000.0, 28000.0, 26000.0, 42000.0, 21000.0, 7000.0]).into(),
//!     ],
//!     vec![
//!         ("Sales", 6500.0).into(),
//!         ("Administration", 16000.0).into(),
//!         ("Information Technology", 30000.0).into(),
//!         ("Customer Support", 38000.0).into(),
//!         ("Development", 52000.0).into(),
//!         ("Marketing", 25000.0).into(),
//!         ("Online", 10000.0).into(),
//!     ],
//! );
//! println!("{}", radar_chart.svg().unwrap());
//! ```
//!
//! # Basic table
//! ```rust
//! use charts_rs::{TableChart};
//! let mut table_chart  = TableChart::new(vec![
//!     vec![
//!         "Name".to_string(),
//!         "Price".to_string(),
//!         "Change".to_string(),
//!     ],
//!     vec![
//!         "Datadog Inc".to_string(),
//!         "97.32".to_string(),
//!         "-7.49%".to_string(),
//!     ],
//!     vec![
//!         "Hashicorp Inc".to_string(),
//!         "28.66".to_string(),
//!         "-9.25%".to_string(),
//!     ],
//!     vec![
//!         "Gitlab Inc".to_string(),
//!         "51.63".to_string(),
//!         "+4.32%".to_string(),
//!     ],
//! ]);
//! table_chart.title_text = "NASDAQ".to_string();
//! println!("{}", table_chart.svg().unwrap());
//! ```

mod charts;
pub use charts::*;

const VERSION: &str = env!("CARGO_PKG_VERSION");
pub fn version() -> String {
    VERSION.to_string()
}
