//! The title and the legend at the top of a chart: side by side where there
//! is room for both on a row, the legend below where they would be written
//! over each other.

use charts_rs::{BarChart, LineChart, PieChart};

const DATA: &str = r#""x_axis_data": ["Mon", "Tue", "Wed"],
    "series_list": [{"name": "Email", "data": [1, 3, 2]}, {"name": "Search", "data": [2, 1, 4]}]"#;

/// How far down the title ends: its row of 30 below the margin of the chart,
/// 5. The sub-title has a row of 20 below it.
const TITLE: f32 = 35.0;
const SUB_TITLE: f32 = 20.0;

/// The y the first text with this content is written at.
fn y_of(svg: &str, text: &str) -> f32 {
    let ending = format!(">\n{text}\n</text>");
    let end = svg
        .find(&ending)
        .unwrap_or_else(|| panic!("{text:?} is not in {svg}"));
    let tag = &svg[svg[..end].rfind("<text").unwrap()..end];
    let start = tag.find(" y=\"").unwrap() + 4;
    tag[start..start + tag[start..].find('"').unwrap()]
        .parse()
        .unwrap()
}

fn line(options: &str) -> String {
    LineChart::from_json(&format!("{{{DATA}{options}}}"))
        .unwrap()
        .svg()
        .unwrap()
}

/// The y of the highest tick label of the y axis: where the plot starts.
fn plot_top(svg: &str) -> f32 {
    y_of(svg, "4.2")
}

#[test]
fn a_legend_goes_below_the_title_it_would_cover() {
    let alone = line("");
    let (legend, plot) = (y_of(&alone, "Email"), plot_top(&alone));

    // Both in the middle of the top, as they are by default: the legend is
    // moved down to below the title, and the plot by as much.
    let titled = line(r#", "title_text": "Visits""#);
    assert_eq!(legend + TITLE, y_of(&titled, "Email"));
    assert_eq!(legend + TITLE, y_of(&titled, "Search"));
    assert_eq!(plot + TITLE, plot_top(&titled));
    assert!(y_of(&titled, "Visits") < y_of(&titled, "Email"));
    // Below the sub-title as well.
    let both = line(r#", "title_text": "Visits", "sub_title_text": "per day""#);
    assert_eq!(legend + TITLE + SUB_TITLE, y_of(&both, "Email"));
    assert_eq!(plot + TITLE + SUB_TITLE, plot_top(&both));

    // So do the other charts: all of them have the same header.
    let bar = |options: &str| {
        BarChart::from_json(&format!("{{{DATA}{options}}}"))
            .unwrap()
            .svg()
            .unwrap()
    };
    let moved = y_of(&bar(r#", "title_text": "Visits""#), "Email") - y_of(&bar(""), "Email");
    assert_eq!(TITLE, moved);
    let pie = |options: &str| {
        PieChart::from_json(&format!(r#"{{{DATA}, "legend_show": true{options}}}"#))
            .unwrap()
            .svg()
            .unwrap()
    };
    let moved = y_of(&pie(r#", "title_text": "Visits""#), "Email") - y_of(&pie(""), "Email");
    assert_eq!(TITLE, moved);
}

#[test]
fn a_legend_stays_beside_a_title_it_does_not_meet() {
    let alone = line("");
    let (legend, plot) = (y_of(&alone, "Email"), plot_top(&alone));

    // A title on the left and a legend on the right share the row.
    for options in [
        r#", "title_text": "Visits", "title_align": "left", "legend_align": "right""#,
        r#", "title_text": "Visits", "title_align": "right", "legend_align": "left""#,
        r#", "title_text": "Visits", "title_align": "left""#,
    ] {
        let svg = line(options);
        assert_eq!(legend, y_of(&svg, "Email"), "{options}");
        // The plot starts below the higher of the two: the title.
        let top = plot_top(&svg);
        assert!(top > plot && top < plot + TITLE, "{options}");
    }
    // Unless the title is long enough to reach the legend.
    let long = line(
        r#", "title_align": "left", "title_text": "Visits of the last week, by the channel they came in""#,
    );
    assert_eq!(legend + TITLE, y_of(&long, "Email"));

    // A sub-title on its own row is below a legend of one row, not under it.
    let sub = line(r#", "sub_title_text": "per day""#);
    assert_eq!(legend, y_of(&sub, "Email"));

    // A legend with a top margin has been put where it is wanted.
    let placed = line(r#", "title_text": "Visits", "legend_margin": {"top": 12}"#);
    assert_eq!(legend + 12.0, y_of(&placed, "Email"));
    let placed = line(r#", "title_text": "Visits", "legend_margin": {"top": 35, "bottom": 5}"#);
    assert_eq!(legend + 35.0, y_of(&placed, "Email"));
    // One with other margins only is still moved.
    let moved = line(r#", "title_text": "Visits", "legend_margin": {"bottom": 8}"#);
    assert_eq!(legend + TITLE, y_of(&moved, "Email"));

    // Elsewhere than at the top there is nothing to meet.
    for position in ["bottom", "left", "right"] {
        let plain = line(&format!(r#", "legend_position": "{position}""#));
        let titled = line(&format!(
            r#", "legend_position": "{position}", "title_text": "Visits""#
        ));
        let moved = y_of(&titled, "Email") - y_of(&plain, "Email");
        // At the bottom not at all; at a side it starts below the title,
        // as it always did.
        let expected = if position == "bottom" { 0.0 } else { TITLE };
        assert_eq!(expected, moved, "{position}");
    }
    // No legend, no move.
    let hidden = line(r#", "title_text": "Visits", "legend_show": false"#);
    assert!(!hidden.contains(">\nEmail\n</text>"));
}
