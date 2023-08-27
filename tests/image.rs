#[test]
#[cfg(feature = "image")]
fn generate_image() {
    use charts_rs::{
        svg_to_png, Align, BarChart, Box, CandlestickChart, HorizontalBarChart, LineChart,
        MarkLine, MarkLineCategory, MultiChart, PieChart, RadarChart, ScatterChart, SeriesCategory,
        TableCellStyle, TableChart, THEME_GRAFANA,
    };
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
    line_chart.margin.right = 50.0;
    line_chart.title_text = "Smoothed Line Chart".to_string();
    line_chart.legend_margin = Some(Box {
        top: line_chart.title_height,
        bottom: 5.0,
        ..Default::default()
    });
    line_chart.series_smooth = true;
    line_chart.series_list[3].label_show = true;
    line_chart.series_list[3].mark_lines = vec![MarkLine {
        category: MarkLineCategory::Average,
    }];
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

    // scatter chart
    let mut scatter_chart = ScatterChart::new_with_theme(
        vec![
            (
                "Female",
                vec![
                    161.2, 51.6, 167.5, 59.0, 159.5, 49.2, 157.0, 63.0, 155.8, 53.6, 170.0, 59.0,
                    159.1, 47.6, 166.0, 69.8, 176.2, 66.8, 160.2, 75.2, 172.5, 55.2, 170.9, 54.2,
                    172.9, 62.5, 153.4, 42.0, 160.0, 50.0, 147.2, 49.8, 168.2, 49.2, 175.0, 73.2,
                    157.0, 47.8, 167.6, 68.8, 159.5, 50.6, 175.0, 82.5, 166.8, 57.2, 176.5, 87.8,
                    170.2, 72.8,
                ],
            )
                .into(),
            (
                "Male",
                vec![
                    174.0, 65.6, 175.3, 71.8, 193.5, 80.7, 186.5, 72.6, 187.2, 78.8, 181.5, 74.8,
                    184.0, 86.4, 184.5, 78.4, 175.0, 62.0, 184.0, 81.6, 180.0, 76.6, 177.8, 83.6,
                    192.0, 90.0, 176.0, 74.6, 174.0, 71.0, 184.0, 79.6, 192.7, 93.8, 171.5, 70.0,
                    173.0, 72.4, 176.0, 85.9, 176.0, 78.8, 180.5, 77.8, 172.7, 66.2, 176.0, 86.4,
                    173.5, 81.8,
                ],
            )
                .into(),
        ],
        THEME_GRAFANA,
    );

    scatter_chart.title_text = "Male and female height and weight distribution".to_string();
    scatter_chart.margin.right = 20.0;
    scatter_chart.title_align = Align::Left;
    scatter_chart.sub_title_text = "Data from: Heinz 2003".to_string();
    scatter_chart.sub_title_align = Align::Left;
    scatter_chart.legend_align = Align::Right;
    scatter_chart.y_axis_configs[0].axis_min = Some(40.0);
    scatter_chart.y_axis_configs[0].axis_max = Some(130.0);
    scatter_chart.y_axis_configs[0].axis_formatter = Some("{c} kg".to_string());

    scatter_chart.x_axis_config.axis_min = Some(140.0);
    scatter_chart.x_axis_config.axis_max = Some(230.0);
    scatter_chart.x_axis_config.axis_formatter = Some("{c} cm".to_string());

    scatter_chart.series_symbol_sizes = vec![6.0, 6.0];
    let buf = svg_to_png(&scatter_chart.svg().unwrap()).unwrap();
    std::fs::write("./asset/image/scatter.png", buf).unwrap();

    let mut candlestick_chart = CandlestickChart::new_with_theme(
        vec![
            // 从第6个点开始
            (
                "MA5",
                vec![
                    2352.93, 2378.48, 2394.81, 2409.64, 2420.04, 2426.66, 2429.33, 2428.01,
                    2417.97, 2410.51, 2391.99, 2368.35, 2349.20, 2331.29, 2314.49, 2322.42,
                    2331.49, 2321.01, 2327.60, 2334.39, 2326.13, 2317.95, 2325.39, 2317.45,
                    2300.81, 2290.01, 2281.96, 2267.85, 2262.02, 2272.7, 2283.49, 2293.46, 2310.80,
                    2318.85, 2315.63, 2298.04, 2279.71, 2261.25, 2247.26, 2232.06, 2227.12,
                    2224.95, 2223.30, 2221.66, 2217.96, 2212.03, 2205.85, 2199.38, 2194.99,
                    2202.56, 2214.61, 2212.55, 2217.45, 2217.79, 2204.45,
                ],
            )
                .into(),
            (
                "K",
                vec![
                    2320.26, 2320.26, 2287.3, 2362.94, 2300.0, 2291.3, 2288.26, 2308.38, 2295.35,
                    2346.5, 2295.35, 2346.92, 2347.22, 2358.98, 2337.35, 2363.8, 2360.75, 2382.48,
                    2347.89, 2383.76, 2383.43, 2385.42, 2371.23, 2391.82, 2377.41, 2419.02,
                    2369.57, 2421.15, 2425.92, 2428.15, 2417.58, 2440.38, 2411.0, 2433.13, 2403.3,
                    2437.42, 2432.68, 2434.48, 2427.7, 2441.73, 2430.69, 2418.53, 2394.22, 2433.89,
                    2416.62, 2432.4, 2414.4, 2443.03, 2441.91, 2421.56, 2415.43, 2444.8, 2420.26,
                    2382.91, 2373.53, 2427.07, 2383.49, 2397.18, 2370.61, 2397.94, 2378.82,
                    2325.95, 2309.17, 2378.82, 2322.94, 2314.16, 2308.76, 2330.88, 2320.62,
                    2325.82, 2315.01, 2338.78, 2313.74, 2293.34, 2289.89, 2340.71, 2297.77,
                    2313.22, 2292.03, 2324.63, 2322.32, 2365.59, 2308.92, 2366.16, 2364.54,
                    2359.51, 2330.86, 2369.65, 2332.08, 2273.4, 2259.25, 2333.54, 2274.81, 2326.31,
                    2270.1, 2328.14, 2333.61, 2347.18, 2321.6, 2351.44, 2340.44, 2324.29, 2304.27,
                    2352.02, 2326.42, 2318.61, 2314.59, 2333.67, 2314.68, 2310.59, 2296.58,
                    2320.96, 2309.16, 2286.6, 2264.83, 2333.29, 2282.17, 2263.97, 2253.25, 2286.33,
                    2255.77, 2270.28, 2253.31, 2276.22, 2269.31, 2278.4, 2250.0, 2312.08, 2267.29,
                    2240.02, 2239.21, 2276.05, 2244.26, 2257.43, 2232.02, 2261.31, 2257.74,
                    2317.37, 2257.42, 2317.86, 2318.21, 2324.24, 2311.6, 2330.81, 2321.4, 2328.28,
                    2314.97, 2332.0, 2334.74, 2326.72, 2319.91, 2344.89, 2318.58, 2297.67, 2281.12,
                    2319.99, 2299.38, 2301.26, 2289.0, 2323.48, 2273.55, 2236.3, 2232.91, 2273.55,
                    2238.49, 2236.62, 2228.81, 2246.87, 2229.46, 2234.4, 2227.31, 2243.95, 2234.9,
                    2227.74, 2220.44, 2253.42, 2232.69, 2225.29, 2217.25, 2241.34, 2196.24,
                    2211.59, 2180.67, 2212.59, 2215.47, 2225.77, 2215.47, 2234.73, 2224.93,
                    2226.13, 2212.56, 2233.04, 2236.98, 2219.55, 2217.26, 2242.48, 2218.09,
                    2206.78, 2204.44, 2226.26, 2199.91, 2181.94, 2177.39, 2204.99, 2169.63,
                    2194.85, 2165.78, 2196.43, 2195.03, 2193.8, 2178.47, 2197.51, 2181.82, 2197.6,
                    2175.44, 2206.03, 2201.12, 2244.64, 2200.58, 2250.11, 2236.4, 2242.17, 2232.26,
                    2245.12, 2242.62, 2184.54, 2182.81, 2242.62, 2187.35, 2218.32, 2184.11,
                    2226.12, 2213.19, 2199.31, 2191.85, 2224.63, 2203.89, 2177.91, 2173.86,
                    2210.58,
                ],
            )
                .into(),
        ],
        vec![
            "2013/1/24".to_string(),
            "2013/1/25".to_string(),
            "2013/1/28".to_string(),
            "2013/1/29".to_string(),
            "2013/1/30".to_string(),
            "2013/1/31".to_string(),
            "2013/2/1".to_string(),
            "2013/2/4".to_string(),
            "2013/2/5".to_string(),
            "2013/2/6".to_string(),
            "2013/2/7".to_string(),
            "2013/2/8".to_string(),
            "2013/2/18".to_string(),
            "2013/2/19".to_string(),
            "2013/2/20".to_string(),
            "2013/2/21".to_string(),
            "2013/2/22".to_string(),
            "2013/2/25".to_string(),
            "2013/2/26".to_string(),
            "2013/2/27".to_string(),
            "2013/2/28".to_string(),
            "2013/3/1".to_string(),
            "2013/3/4".to_string(),
            "2013/3/5".to_string(),
            "2013/3/6".to_string(),
            "2013/3/7".to_string(),
            "2013/3/8".to_string(),
            "2013/3/11".to_string(),
            "2013/3/12".to_string(),
            "2013/3/13".to_string(),
            "2013/3/14".to_string(),
            "2013/3/15".to_string(),
            "2013/3/18".to_string(),
            "2013/3/18".to_string(),
            "2013/3/20".to_string(),
            "2013/3/21".to_string(),
            "2013/3/22".to_string(),
            "2013/3/25".to_string(),
            "2013/3/26".to_string(),
            "2013/3/27".to_string(),
            "2013/3/28".to_string(),
            "2013/3/29".to_string(),
            "2013/4/1".to_string(),
            "2013/4/2".to_string(),
            "2013/4/3".to_string(),
            "2013/4/8".to_string(),
            "2013/4/9".to_string(),
            "2013/4/10".to_string(),
            "2013/4/11".to_string(),
            "2013/4/12".to_string(),
            "2013/4/15".to_string(),
            "2013/4/16".to_string(),
            "2013/4/17".to_string(),
            "2013/4/18".to_string(),
            "2013/4/19".to_string(),
            "2013/4/22".to_string(),
            "2013/4/23".to_string(),
            "2013/4/24".to_string(),
            "2013/4/25".to_string(),
            "2013/4/26".to_string(),
        ],
        THEME_GRAFANA,
    );
    candlestick_chart.series_list[0].category = Some(SeriesCategory::Line);
    candlestick_chart.series_list[0].start_index = 5;
    candlestick_chart.y_axis_configs[0].axis_min = Some(2100.0);
    candlestick_chart.y_axis_configs[0].axis_max = Some(2460.0);
    candlestick_chart.y_axis_configs[0].axis_formatter = Some("{t}".to_string());
    let buf = svg_to_png(&candlestick_chart.svg().unwrap()).unwrap();
    std::fs::write("./asset/image/candlestick.png", buf).unwrap();

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
    let green = "#2d7c2b".into();
    let red = "#a93b01".into();
    table_chart.cell_styles = vec![
        TableCellStyle {
            indexes: vec![1, 2],
            font_weight: Some("bold".to_string()),
            background_color: Some(green),
            ..Default::default()
        },
        TableCellStyle {
            indexes: vec![2, 2],
            font_weight: Some("bold".to_string()),
            background_color: Some(green),
            ..Default::default()
        },
        TableCellStyle {
            indexes: vec![3, 2],
            font_weight: Some("bold".to_string()),
            background_color: Some(red),
            ..Default::default()
        },
    ];
    table_chart.title_text = "NASDAQ".to_string();
    let buf = svg_to_png(&table_chart.svg().unwrap()).unwrap();
    std::fs::write("./asset/image/table.png", buf).unwrap();

    let mut multi_chart = MultiChart::from_json(
        r###"{
        "child_charts": [
            {
            "quality": 80,
            "width": 400,
            "height": 300,
            "margin": {
                "left": 5,
                "top": 5,
                "right": 5,
                "bottom": 5
            },
            "font_family": "Roboto",
            "title_font_size": 18,
            "title_font_weight": "bold",
            "title_align": "right",
            "title_margin": {
                "left": 0,
                "top": 0,
                "right": 0,
                "bottom": 0
            },
            "title_height": 30,
            "sub_title_text": "Sub Title",
            "sub_title_font_size": 14,
            "sub_title_margin": {
                "left": 0,
                "top": 0,
                "right": 0,
                "bottom": 0
            },
            "sub_title_align": "center",
            "sub_title_height": 20,
            "legend_font_size": 14,
            "legend_align": "left",
            "legend_margin": {
                "left": 0,
                "top": 0,
                "right": 0,
                "bottom": 0
            },
            "legend_category": "normal",
            "legend_show": true,
            "x_axis_height": 30,
            "x_axis_font_size": 14,
            "x_axis_name_gap": 5,
            "x_axis_name_rotate": 0,
            "x_boundary_gap": true,
            "x_axis_margin": {
                "left": 0,
                "top": 0,
                "right": 0,
                "bottom": 0
            },
            "type": "bar",
            "title_text": "Bar Chart",
            "x_axis_data": [
                "Mon",
                "Tue",
                "Wed",
                "Thu",
                "Fri",
                "Sat",
                "Sun"
            ],
            "series_list": [
                {
                "name": "Email",
                "label_show": true,
                "data": [
                    120,
                    132,
                    101,
                    134,
                    90,
                    230,
                    210
                ]
                },
                {
                "name": "Union Ads",
                "label_show": true,
                "data": [
                    220,
                    182,
                    191,
                    234,
                    290,
                    330,
                    310
                ]
                }
            ]
            },
            {
            "quality": 80,
            "x": 420,
            "y": 10,
            "width": 400,
            "height": 300,
            "margin": {
                "left": 15,
                "top": 15,
                "right": 15,
                "bottom": 15
            },
            "font_family": "Roboto",
            "title_font_size": 18,
            "title_font_weight": "bold",
            "title_align": "left",
            "title_margin": {
                "left": 0,
                "top": 0,
                "right": 0,
                "bottom": 0
            },
            "title_height": 30,
            "sub_title_text": "Sub Title",
            "sub_title_font_size": 14,
            "sub_title_margin": {
                "left": 0,
                "top": 0,
                "right": 0,
                "bottom": 0
            },
            "sub_title_align": "center",
            "sub_title_height": 20,
            "legend_font_size": 14,
            "legend_align": "right",
            "legend_margin": {
                "left": 0,
                "top": 0,
                "right": 0,
                "bottom": 0
            },
            "legend_category": "round_rect",
            "legend_show": true,
            "x_axis_height": 30,
            "x_axis_font_size": 14,
            "x_axis_name_gap": 5,
            "x_axis_name_rotate": 0,
            "x_boundary_gap": false,
            "x_axis_margin": {
                "left": 0,
                "top": 0,
                "right": 0,
                "bottom": 0
            },
            "type": "line",
            "title_text": "Line Chart",
            "x_axis_data": [
                "Mon",
                "Tue",
                "Wed",
                "Thu",
                "Fri",
                "Sat",
                "Sun"
            ],
            "series_list": [
                {
                "name": "Email",
                "label_show": true,
                "data": [
                    120,
                    132,
                    101,
                    134,
                    90,
                    230,
                    210
                ]
                },
                {
                "name": "Union Ads",
                "label_show": true,
                "data": [
                    220,
                    182,
                    191,
                    234,
                    290,
                    330,
                    310
                ]
                }
            ]
            },
            {
            "quality": 80,
            "width": 400,
            "x": 210,
            "y": 320,
            "height": 300,
            "margin": {
                "left": 5,
                "top": 5,
                "right": 5,
                "bottom": 5
            },
            "font_family": "Roboto",
            "title_font_size": 18,
            "title_font_weight": "bold",
            "title_margin": {
                "left": 0,
                "top": 0,
                "right": 0,
                "bottom": 0
            },
            "title_align": "center",
            "title_height": 30,
            "sub_title_text": "Sub Title",
            "sub_title_font_size": 14,
            "sub_title_margin": {
                "left": 0,
                "top": 0,
                "right": 0,
                "bottom": 0
            },
            "sub_title_align": "center",
            "sub_title_height": 20,
            "legend_font_size": 14,
            "legend_align": "center",
            "legend_margin": {
                "top": 50
            },
            "legend_category": "normal",
            "legend_show": true,
            "x_axis_height": 30,
            "x_axis_font_size": 14,
            "x_axis_name_gap": 5,
            "x_axis_name_rotate": 0,
            "x_boundary_gap": true,
            "x_axis_margin": {
                "left": 0,
                "top": 0,
                "right": 0,
                "bottom": 0
            },
            "type": "pie",
            "title_text": "Nightingale Chart",
            "series_list": [
                {
                "name": "rose 1",
                "data": [
                    40
                ]
                },
                {
                "name": "rose 2",
                "data": [
                    38
                ]
                },
                {
                "name": "rose 3",
                "data": [
                    32
                ]
                },
                {
                "name": "rose 4",
                "data": [
                    30
                ]
                },
                {
                "name": "rose 5",
                "data": [
                    28
                ]
                },
                {
                "name": "rose 6",
                "data": [
                    26
                ]
                },
                {
                "name": "rose 7",
                "data": [
                    22
                ]
                },
                {
                "name": "rose 8",
                "data": [
                    18
                ]
                }
            ]
            }
        ],
        "theme": "grafana"
        }"###,
    )
    .unwrap();
    let buf = svg_to_png(&multi_chart.svg().unwrap()).unwrap();
    std::fs::write("./asset/image/multi-chart.png", buf).unwrap();
}
