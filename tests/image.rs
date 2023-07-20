use charts_rs::{
    Align, BarChart, Box, HorizontalBarChart, LineChart, PieChart, RadarChart, SeriesCategory,
    TableChart, THEME_GRAFANA,
};

#[test]
#[cfg(feature = "image")]
fn generate_image() {
    use charts_rs::svg_to_png;
    // bar chart
    let mut bar_chart = BarChart::new_with_theme(
        vec![
            ("Evaporation", vec![2.0, 4.9, 7.0, 23.2, 25.6, 76.7, 135.6]).into(),
            (
                "Precipitation",
                vec![2.6, 5.9, 9.0, 26.4, 28.7, 70.7, 175.6],
            )
                .into(),
            ("Temperature", vec![2.0, 2.2, 3.3, 4.5, 6.3, 10.2, 20.3]).into(),
        ],
        vec![
            "Mon".to_string(),
            "Tue".to_string(),
            "Wed".to_string(),
            "Thu".to_string(),
            "Fri".to_string(),
            "Sat".to_string(),
            "Sun".to_string(),
        ],
        THEME_GRAFANA,
    );
    bar_chart.title_text = "Mixed Line and Bar".to_string();
    bar_chart.legend_margin = Some(Box {
        top: bar_chart.title_height,
        bottom: 5.0,
        ..Default::default()
    });
    bar_chart.series_list[2].category = Some(SeriesCategory::Line);
    bar_chart.series_list[2].y_axis_index = 1;
    bar_chart.series_list[2].label_show = true;

    bar_chart
        .y_axis_configs
        .push(bar_chart.y_axis_configs[0].clone());
    bar_chart.y_axis_configs[0].axis_formatter = Some("{c} ml".to_string());
    bar_chart.y_axis_configs[1].axis_formatter = Some("{c} °C".to_string());

    let buf = svg_to_png(&bar_chart.svg().unwrap()).unwrap();
    std::fs::write("./asset/image/mix-line-bar.png", buf).unwrap();

    // horizontal bar chart
    let mut horizontal_bar_chart = HorizontalBarChart::new_with_theme(
        vec![
            (
                "2011",
                vec![18203.0, 23489.0, 29034.0, 104970.0, 131744.0, 630230.0],
            )
                .into(),
            (
                "2012",
                vec![19325.0, 23438.0, 31000.0, 121594.0, 134141.0, 681807.0],
            )
                .into(),
        ],
        vec![
            "Brazil".to_string(),
            "Indonesia".to_string(),
            "USA".to_string(),
            "India".to_string(),
            "China".to_string(),
            "World".to_string(),
        ],
        THEME_GRAFANA,
    );
    for series_list in horizontal_bar_chart.series_list.iter_mut() {
        series_list.label_show = true;
    }
    horizontal_bar_chart.margin.right = 30.0;
    horizontal_bar_chart.title_text = "World Population".to_string();
    horizontal_bar_chart.title_align = Align::Left;
    let buf = svg_to_png(&horizontal_bar_chart.svg().unwrap()).unwrap();
    std::fs::write("./asset/image/horizontal-bar.png", buf).unwrap();

    // line chart
    let mut line_chart = LineChart::new_with_theme(
        vec![
            (
                "Email",
                vec![120.0, 132.0, 101.0, 134.0, 90.0, 230.0, 210.0],
            )
                .into(),
            (
                "Union Ads",
                vec![220.0, 182.0, 191.0, 234.0, 290.0, 330.0, 310.0],
            )
                .into(),
            (
                "Direct",
                vec![320.0, 332.0, 301.0, 334.0, 390.0, 330.0, 320.0],
            )
                .into(),
            (
                "Search Engine",
                vec![820.0, 932.0, 901.0, 934.0, 1290.0, 1330.0, 1320.0],
            )
                .into(),
        ],
        vec![
            "Mon".to_string(),
            "Tue".to_string(),
            "Wed".to_string(),
            "Thu".to_string(),
            "Fri".to_string(),
            "Sat".to_string(),
            "Sun".to_string(),
        ],
        THEME_GRAFANA,
    );
    line_chart.title_text = "Smoothed Line Chart".to_string();
    line_chart.legend_margin = Some(Box {
        top: line_chart.title_height,
        bottom: 5.0,
        ..Default::default()
    });
    line_chart.series_smooth = true;
    line_chart.series_list[3].label_show = true;
    let buf = svg_to_png(&line_chart.svg().unwrap()).unwrap();
    std::fs::write("./asset/image/line.png", buf).unwrap();

    // pie chart
    let mut pie_chart = PieChart::new_with_theme(
        vec![
            ("rose 1", vec![40.0]).into(),
            ("rose 2", vec![38.0]).into(),
            ("rose 3", vec![32.0]).into(),
            ("rose 4", vec![30.0]).into(),
            ("rose 5", vec![28.0]).into(),
            ("rose 6", vec![26.0]).into(),
            ("rose 7", vec![22.0]).into(),
            ("rose 8", vec![18.0]).into(),
        ],
        THEME_GRAFANA,
    );
    pie_chart.title_text = "Nightingale Chart".to_string();
    pie_chart.sub_title_text = "Fake Data".to_string();
    let buf = svg_to_png(&pie_chart.svg().unwrap()).unwrap();
    std::fs::write("./asset/image/pie.png", buf).unwrap();

    // radar chart
    let radar_chart = RadarChart::new_with_theme(
        vec![
            (
                "Allocated Budget",
                vec![4200.0, 3000.0, 20000.0, 35000.0, 50000.0, 18000.0],
            )
                .into(),
            (
                "Actual Spending",
                vec![5000.0, 14000.0, 28000.0, 26000.0, 42000.0, 21000.0],
            )
                .into(),
        ],
        vec![
            ("Sales", 6500.0).into(),
            ("Administration", 16000.0).into(),
            ("Information Technology", 30000.0).into(),
            ("Customer Support", 38000.0).into(),
            ("Development", 52000.0).into(),
            ("Marketing", 25000.0).into(),
        ],
        THEME_GRAFANA,
    );
    let buf = svg_to_png(&radar_chart.svg().unwrap()).unwrap();
    std::fs::write("./asset/image/radar.png", buf).unwrap();

    // table chart
    let mut table_chart = TableChart::new_with_theme(
        vec![
            vec![
                "Name".to_string(),
                "Price".to_string(),
                "Change".to_string(),
            ],
            vec![
                "Datadog Inc".to_string(),
                "97.32".to_string(),
                "-7.49%".to_string(),
            ],
            vec![
                "Hashicorp Inc".to_string(),
                "28.66".to_string(),
                "-9.25%".to_string(),
            ],
            vec![
                "Gitlab Inc".to_string(),
                "51.63".to_string(),
                "+4.32%".to_string(),
            ],
        ],
        THEME_GRAFANA,
    );
    table_chart.title_text = "NASDAQ".to_string();
    let buf = svg_to_png(&table_chart.svg().unwrap()).unwrap();
    std::fs::write("./asset/image/table.png", buf).unwrap();
}
