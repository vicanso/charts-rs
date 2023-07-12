//! Charts support multi charts: bar chart, horizontal bar chart,
//! line chart, pie chart, radar chart and table.
//! 
//! It supports four themes and very easy to use. 
//! Each attribute can be customized, it can be save as svg or png.
//! 
//! ```rust
//! use charts_rs::{BarChart, Series, svg_to_png};
//! let mut bar_chart = BarChart::new(
//!     vec![
//!         Series::new(
//!             "Email".to_string(),
//!             vec![120.0, 132.0, 101.0, 134.0, 90.0, 230.0, 210.0],
//!         ),
//!         Series::new(
//!             "Union Ads".to_string(),
//!             vec![220.0, 182.0, 191.0, 234.0, 290.0, 330.0, 310.0],
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
//!     ]
//! );
//! println!("{}", bar_chart.svg().unwrap());
//! svg_to_png(&bar_chart.svg().unwrap()).unwrap();


mod charts;
pub use charts::*;