# charts-rs

`charts-rs` 是纯 Rust 实现的图表库，使用简单而且性能高效，生成 SVG 低于 10ms，而 PNG 也低于 50ms，便于在各种无法直接渲染 SVG 的场景下使用，现已支持更多的图片格式，如：jpeg、webp 以及 avif。

[![Crates.io][crates-badge]][crates-url]
[![Apache licensed][apache-badge]][apache-url]
[![Build status](https://github.com/vicanso/charts-rs/actions/workflows/test.yml/badge.svg?branch=main)](https://github.com/vicanso/charts-rs/actions/workflows/test.yml)

[crates-badge]: https://img.shields.io/crates/v/charts-rs.svg
[crates-url]: https://crates.io/crates/charts-rs
[apache-badge]: https://img.shields.io/badge/license-apache2-blue.svg
[apache-url]: https://github.com/vicanso/charts-rs/blob/main/LICENSE

## 概要

`charts-rs` 提供简洁的图表生成方案，支持 `svg`、`png`、`jpeg`、`webp` 以及 `avif` 等多种输出格式。该库提供十种不同的主题：`light`、`dark`、`grafana`、`ant`、`vintage`、`walden`、`westeros`、`chalk`、`shine` 以及 `shadcn`，默认主题为 `light`。

该库支持二十七种图表类型：`Bar`、`HorizontalBar`、`Line`、`Pie`、`Radar`、`Scatter`、`Candlestick`、`Table`、`Heatmap`、`Funnel`、`Waterfall`、`MultiChart`、`Calendar`、`Gauge`、`Treemap`、`BoxPlot`、`Sunburst`、`Sankey`、`Tree`、`Graph`、`Parallel`、`ThemeRiver`、`Histogram`、`PolarBar`、`Chord`、`Gantt` 以及 `Map`。参考 `Apache ECharts` 的设计理念，`charts-rs` 使开发者能够创建具有相似功能和外观的图表。

## 更多主题色

[更多主题色](./theme.md)

## 特性

- 十种内置主题，支持通过 `add_theme()` 添加自定义主题
- 支持从 ttf 或 otf 文件加载自定义字体
- 曲线图高级功能：平滑曲线、区域填充、标记点和标记线
- 所有图表支持多种图例样式：圆角矩形、圆形以及矩形
- 双 Y 轴支持（`y_axis_configs` + `series.y_axis_index`），增强数据可视化效果
- 对数坐标轴支持（`"log"`、`"log2"` 或 `{"type":"log","base":N}`）
- 渐变填充支持，可用于柱状图、面积图和饼图（`Fill::LinearGradient`）
- 同一图表中混合多种系列类型（柱状 + 折线）
- 系列堆叠、自定义虚线样式、按柱自定义颜色
- 柱状图、折线图、饼图、旭日图、漏斗图、矩形树图与桑基图的 SVG 动画支持（时长、缓动函数、错开延迟）
- 通过 `Option<f32>` 支持空值 / 缺失数据点（JSON 中使用 `null`；旧的 `NIL_VALUE` 仍兼容）
- 所有图表类型均支持基于 JSON 的配置方式，并提供[完整的参数参考](./docs/json-zh.md)
- 多种输出格式：svg、png、jpeg、webp、avif
- 支持指定目标尺寸的图片导出（`svg_to_png_with_size` 及各格式对应函数）
- 基于 Web 的 JSON 编辑器，支持交互式图表配置和测试
- 折线区间带（`series.band`）：置信区间、预测范围、最低–最高范围
- 阶梯线（`series.step`）、百分比堆叠（`stack_percent`）、半环 / 部分圆饼图（`end_angle`）、嵌套多环饼图（`series.ring`）、反向数值轴（`axis_inverse`，用于排名图）
- 柱、折线、散点上的误差线（`series.error_bar`），以及散点图的回归曲线（`regression`：线性、指数、对数、多项式）
- 热力图和日历图支持多色渐变与分段配色（`colors`、`steps`、`thresholds`）
- 仪表盘支持分段着色、多指针，以及每个系列一个进度环（`thresholds`、`multi_ring`）

## 安装

在 `Cargo.toml` 中添加 `charts-rs`：

```toml
[dependencies]
charts-rs = "1"
```

默认构建即可生成 SVG。若需导出位图格式（`png`、`jpeg`、`webp`、`avif`），即下文用到的
`svg_to_png`、`svg_to_png_with_size` 等函数，需按格式开启 `png` / `jpeg` / `webp` / `avif` feature（`image-encoder` 为全部开启的总开关，`avif` 会引入较重的 `rav1e` 编码器）：

```toml
[dependencies]
charts-rs = { version = "1", features = ["png"] }
```

## 示例

可以使用网页版尝试使用 `charts-rs` 的相关图表示例，可以直接改动配置后，重新生成效果图，非常简单而有用。

示例地址：[https://charts.npmtrend.com/](https://charts.npmtrend.com/)

示例项目代码：[https://github.com/vicanso/charts-rs-web](https://github.com/vicanso/charts-rs-web)

<p align="center">
    <img src="https://raw.githubusercontent.com/vicanso/charts-rs/main/asset/image/charts-demo.png" alt="charts-rs">
</p>

## Mix line bar

<p align="center">
    <img src="https://raw.githubusercontent.com/vicanso/charts-rs/main/asset/image/mix-line-bar.png" alt="charts-rs">
</p>

<p align="center">
    <img src="https://raw.githubusercontent.com/vicanso/charts-rs/main/asset/image/bar-stack-percent.png" alt="charts-rs">
</p>

<p align="center">
    <img src="https://raw.githubusercontent.com/vicanso/charts-rs/main/asset/image/bar-error-bar.png" alt="charts-rs">
</p>

## Horizontal bar

<p align="center">
    <img src="https://raw.githubusercontent.com/vicanso/charts-rs/main/asset/image/horizontal-bar.png" alt="charts-rs">
</p>

## Line

<p align="center">
    <img src="https://raw.githubusercontent.com/vicanso/charts-rs/main/asset/image/line.png" alt="charts-rs">
</p>

<p align="center">
    <img src="https://raw.githubusercontent.com/vicanso/charts-rs/main/asset/image/line-step.png" alt="charts-rs">
</p>

<p align="center">
    <img src="https://raw.githubusercontent.com/vicanso/charts-rs/main/asset/image/line-bump.png" alt="charts-rs">
</p>

## Line band

<p align="center">
    <img src="https://raw.githubusercontent.com/vicanso/charts-rs/main/asset/image/line-band.png" alt="charts-rs">
</p>

## Pie

<p align="center">
    <img src="https://raw.githubusercontent.com/vicanso/charts-rs/main/asset/image/pie.png" alt="charts-rs">
</p>

<p align="center">
    <img src="https://raw.githubusercontent.com/vicanso/charts-rs/main/asset/image/pie-half.png" alt="charts-rs">
</p>

<p align="center">
    <img src="https://raw.githubusercontent.com/vicanso/charts-rs/main/asset/image/pie-nested.png" alt="charts-rs">
</p>

## Radar

<p align="center">
    <img src="https://raw.githubusercontent.com/vicanso/charts-rs/main/asset/image/radar.png" alt="charts-rs">
</p>

## Scatter

<p align="center">
    <img src="https://raw.githubusercontent.com/vicanso/charts-rs/main/asset/image/scatter.png" alt="charts-rs">
</p>

<p align="center">
    <img src="https://raw.githubusercontent.com/vicanso/charts-rs/main/asset/image/scatter-regression.png" alt="charts-rs">
</p>

## Candlestick

<p align="center">
    <img src="https://raw.githubusercontent.com/vicanso/charts-rs/main/asset/image/candlestick.png" alt="charts-rs">
</p>

<p align="center">
    <img src="https://raw.githubusercontent.com/vicanso/charts-rs/main/asset/image/candlestick-ohlc.png" alt="charts-rs">
</p>

## Table

<p align="center">
    <img src="https://raw.githubusercontent.com/vicanso/charts-rs/main/asset/image/table.avif" alt="charts-rs">
</p>

## Heatmap

<p align="center">
    <img src="https://raw.githubusercontent.com/vicanso/charts-rs/main/asset/image/heatmap.png" alt="charts-rs">
</p>

<p align="center">
    <img src="https://raw.githubusercontent.com/vicanso/charts-rs/main/asset/image/heatmap-scale.png" alt="charts-rs">
</p>

<p align="center">
    <img src="https://raw.githubusercontent.com/vicanso/charts-rs/main/asset/image/heatmap-punch-card.png" alt="charts-rs">
</p>

## Funnel

<p align="center">
    <img src="https://raw.githubusercontent.com/vicanso/charts-rs/main/asset/image/funnel.png" alt="charts-rs">
</p>

## Waterfall

<p align="center">
    <img src="https://raw.githubusercontent.com/vicanso/charts-rs/main/asset/image/waterfall.png" alt="charts-rs">
</p>

## Calendar

<p align="center">
    <img src="https://raw.githubusercontent.com/vicanso/charts-rs/main/asset/image/calendar.png" alt="charts-rs">
</p>

## Gauge

<p align="center">
    <img src="https://raw.githubusercontent.com/vicanso/charts-rs/main/asset/image/gauge.png" alt="charts-rs">
</p>

<p align="center">
    <img src="https://raw.githubusercontent.com/vicanso/charts-rs/main/asset/image/gauge-segments.png" alt="charts-rs">
</p>

<p align="center">
    <img src="https://raw.githubusercontent.com/vicanso/charts-rs/main/asset/image/gauge-rings.png" alt="charts-rs">
</p>

## Treemap

<p align="center">
    <img src="https://raw.githubusercontent.com/vicanso/charts-rs/main/asset/image/treemap.png" alt="charts-rs">
</p>

## Box Plot

<p align="center">
    <img src="https://raw.githubusercontent.com/vicanso/charts-rs/main/asset/image/box-plot.png" alt="charts-rs">
</p>

## Sunburst

<p align="center">
    <img src="https://raw.githubusercontent.com/vicanso/charts-rs/main/asset/image/sunburst.png" alt="charts-rs">
</p>

## Multi Chart

<p align="center">
    <img src="https://raw.githubusercontent.com/vicanso/charts-rs/main/asset/image/multi-chart.webp" alt="charts-rs">
</p>

## Sankey

<p align="center">
    <img src="https://raw.githubusercontent.com/vicanso/charts-rs/main/asset/image/sankey.png" alt="charts-rs">
</p>

<p align="center">
    <img src="https://raw.githubusercontent.com/vicanso/charts-rs/main/asset/image/sankey-vertical.png" alt="charts-rs">
</p>

## Tree

<p align="center">
    <img src="https://raw.githubusercontent.com/vicanso/charts-rs/main/asset/image/tree.png" alt="charts-rs">
</p>

<p align="center">
    <img src="https://raw.githubusercontent.com/vicanso/charts-rs/main/asset/image/tree-radial.png" alt="charts-rs">
</p>

## Graph

<p align="center">
    <img src="https://raw.githubusercontent.com/vicanso/charts-rs/main/asset/image/graph.png" alt="charts-rs">
</p>

## Parallel

<p align="center">
    <img src="https://raw.githubusercontent.com/vicanso/charts-rs/main/asset/image/parallel.png" alt="charts-rs">
</p>

## Theme River

<p align="center">
    <img src="https://raw.githubusercontent.com/vicanso/charts-rs/main/asset/image/theme-river.png" alt="charts-rs">
</p>

## Histogram

<p align="center">
    <img src="https://raw.githubusercontent.com/vicanso/charts-rs/main/asset/image/histogram.png" alt="charts-rs">
</p>

## Polar bar

<p align="center">
    <img src="https://raw.githubusercontent.com/vicanso/charts-rs/main/asset/image/polar-bar.png" alt="charts-rs">
</p>

<p align="center">
    <img src="https://raw.githubusercontent.com/vicanso/charts-rs/main/asset/image/polar-bar-radial.png" alt="charts-rs">
</p>

## Chord

<p align="center">
    <img src="https://raw.githubusercontent.com/vicanso/charts-rs/main/asset/image/chord.png" alt="charts-rs">
</p>

## Gantt

<p align="center">
    <img src="https://raw.githubusercontent.com/vicanso/charts-rs/main/asset/image/gantt.png" alt="charts-rs">
</p>

## Map

<p align="center">
    <img src="https://raw.githubusercontent.com/vicanso/charts-rs/main/asset/image/map.png" alt="charts-rs">
</p>

## 最低 Rust 版本

charts-rs 1.x 需要 Rust 1.88 及以上（edition 2024）。

## Rust 示例

可运行示例位于 [`examples/`](./examples) 目录，每个都会生成一个 `svg` 文件：

```bash
cargo run --example bar       # 基础柱状图
cargo run --example line      # 平滑折线 + 面积填充 + 均值线 / 标记点
cargo run --example pie       # 南丁格尔（玫瑰）图
cargo run --example sunburst  # 旭日图：标签格式化、分层厚度、动画
cargo run --example sankey     # 桑基流向图（节点由 links 自动推导）
cargo run --example tree       # 节点-连线树图（曲线连线，LR 布局）
cargo run --example histogram  # 直方图：样本在等宽分箱上的分布
cargo run --example polar_bar  # 极坐标柱状图：环绕堆叠的柱，以及圆头环形柱
cargo run --example chord      # 和弦图：节点之间的流量（渐变色带）
cargo run --example gantt      # 甘特图：带进度、里程碑和当前日期的项目计划
cargo run --example map        # 地图：区域（GeoJSON）按数值分段着色
```

### 使用 Builder API 创建图表

```rust
use charts_rs::{
    BarChart, Box, SeriesCategory, THEME_GRAFANA
};
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
bar_chart.title.text = "Mixed Line and Bar".to_string();
bar_chart.legend.margin = Some(Box {
    top: bar_chart.title.height,
    bottom: 5.0,
    ..Default::default()
});
bar_chart.series_list[2].category = Some(SeriesCategory::Line);
bar_chart.series_list[2].y_axis_index = 1;
bar_chart.series_list[2].label_show = true;

bar_chart
    .y_axis_configs
    .push(bar_chart.y_axis_configs[0].clone());
bar_chart.y_axis_configs[0].formatter = Some("{c} ml".to_string());
bar_chart.y_axis_configs[1].formatter = Some("{c} °C".to_string());

println!("{}", &bar_chart.svg().unwrap());
svg_to_png(&bar_chart.svg().unwrap()).unwrap();
```

### Rust 与 JSON 中的参数

两种方式的参数相同，本文档统一用 JSON 参数名来称呼它们。在 Rust 中，共享的参数按所属元素分组——`title`、`sub_title`、`legend`、`x_axis`、`grid`、`series`、`tooltip`——JSON 参数名就是字段路径用 `_` 连接：

| JSON | Rust |
|------|------|
| `"title_font_size": 18` | `chart.title.font.size = 18.0` |
| `"legend_position": "bottom"` | `chart.legend.position = Some(Position::Bottom)` |
| `"x_axis_data": [...]` | `chart.x_axis.data = vec![...]` |
| `"series_label_font_color": "#333"` | `chart.series.label.font.color = "#333".into()` |
| `"tooltip_show": true` | `chart.tooltip.show = true` |

例外：`x_boundary_gap` 对应 `x_axis.boundary_gap`，`x_axis_type` 对应 `x_axis.kind`，y 轴配置的参数去掉 `axis_` 前缀（`"axis_min"` 对应 `y_axis_configs[0].min`），热力图的 `series` 对象对应 `heatmap_series`（`series` 在所有图表上都是系列的样式分组）。所有 JSON 参数见 [JSON 参数文档](./docs/json-zh.md)。

### 通过 JSON 字符串配置创建图表

各图表的 JSON 文档可以使用的全部键（类型、默认值和作用）见 [JSON 参数参考](./docs/json-zh.md)。

```rust,no_run
use charts_rs::{BarChart, svg_to_png};
let bar_chart = BarChart::from_json(
    r###"{
        "width": 630,
        "height": 410,
        "margin": {
            "left": 10,
            "top": 5,
            "right": 10
        },
        "title_text": "Bar Chart",
        "title_font_color": "#345",
        "title_align": "right",
        "sub_title_text": "demo",
        "legend_align": "left",
        "series_list": [
            {
                "name": "Email",
                "label_show": true,
                "data": [120.0, 132.0, 101.0, 134.0, 90.0, 230.0, 210.0]
            },
            {
                "name": "Union Ads",
                "data": [220.0, 182.0, 191.0, 234.0, 290.0, 330.0, 310.0]
            }
        ],
        "x_axis_data": [
            "Mon",
            "Tue",
            "Wed",
            "Thu",
            "Fri",
            "Sat",
            "Sun"
        ]
    }"###,
).unwrap();
println!("{}", bar_chart.svg().unwrap());
svg_to_png(&bar_chart.svg().unwrap()).unwrap();
```

### 指定尺寸的图片导出

```rust
use charts_rs::{BarChart, svg_to_png_with_size};
let chart = BarChart::from_json(r###"{ ... }"###).unwrap();
let svg = chart.svg().unwrap();

// 缩放到指定的 800×400
let png = svg_to_png_with_size(&svg, Some(800), Some(400)).unwrap();

// 仅指定宽度，高度按比例自动计算
let png = svg_to_png_with_size(&svg, Some(800), None).unwrap();
```

### 空值数据点

在 JSON 数组中使用 `null` 表示缺失数据，图表会跳过该位置的渲染。

```json
{
  "series_list": [{
    "name": "销售额",
    "data": [120.0, null, 101.0, null, 90.0]
  }]
}
```

在 Rust 中使用 `Option<f32>` 构造系列（`None` 即缺失点）：

```rust
use charts_rs::Series;
let series = Series::new_nullable(
    "销售额".to_string(),
    vec![Some(120.0), None, Some(101.0), None, Some(90.0)],
);
// 或使用元组转换
let series: Series = ("销售额", vec![Some(120.0), None, Some(101.0)]).into();
```

旧的 `NIL_VALUE` 哨兵值（`Series::new` / `Vec<f32>`）仍然可用，会被当作 `None` 处理。

### 标签格式化占位符

`series_label_formatter`、`axis_formatter`、`value_formatter`（仪表盘）等字段均支持以下占位符：

| 占位符 | 含义 |
|--------|------|
| `{c}` | 数据值 |
| `{a}` | 系列名称 |
| `{b}` | 分类名（X 轴标签） |
| `{d}` | 百分比（饼图 / 漏斗图） |
| `{t}` | 千位格式（1.2K、5.6M） |

### 数值轴与时间轴

折线图、柱状图的 x 轴默认是类目轴（各点等距）。给出 x 值后，x 轴变为连续轴：每个点按其 x 值定位，不等间隔的采样保持真实间距。

```json
{
  "x_axis_title": "时间 (UTC)",
  "y_axis_configs": [{ "axis_title": "温度 (°C)" }],
  "x_axis_values": ["2024-03-01 00:00", "2024-03-01 00:20", "2024-03-01 04:00", "2024-03-01 12:00"],
  "series_list": [
    { "name": "室内", "data": [21.5, 21.8, 20.1, 24.6] },
    { "name": "室外", "x_values": ["2024-03-01 02:00", "2024-03-01 11:00"], "data": [8.2, 14.9] }
  ]
}
```

- `x_axis_values` 由所有系列共享；采样时刻不同的系列可以自带 `x_values`。
- 日期字符串（`2024-03-01`、`2024-03-01 08:30`、`2024-03-01T08:30:00+08:00`）会自动使用时间轴；数字是普通数值，配合 `"x_axis_type": "time"` 时表示 unix 秒。
- 刻度落在整数值或整点时间上。`x_axis_min` / `x_axis_max` 固定范围，`x_axis_formatter` 设置标签格式（`"{c} km"`，时间轴可用 `"%m-%d %H:%M"` 这类模式），`x_axis_time_offset`（相对 UTC 的分钟数，东八区为 480）用于按本地时间显示时间戳。

Rust 中设置 `chart.x_axis.values`（`Vec<f64>`），时间戳再加上 `chart.x_axis.kind = AxisType::Time`。

### 坐标轴标题

`x_axis_title` 显示在 x 轴下方；y 轴配置中的 `axis_title` 沿该轴竖向显示。

### 气泡图

散点图设置 `"bubble": true` 后，系列数据按 `[x, y, size]` 三元组读取，圆的面积随 size 变化（`bubble_min_size` / `bubble_max_size` 设置半径范围）。

### 直方图

`HistogramChart` 的系列数据就是原始样本，图表会把它统计到等宽的分箱里：

```json
{
  "x_axis_title": "身高 (cm)",
  "series_list": [{ "name": "成年人", "data": [162.4, 171.0, 168.3, 175.9, 158.2, 169.7] }]
}
```

- 默认根据样本自动选择分箱（边界取整数）；可用 `bin_width` 或 `bin_count` 指定，`x_axis_min` / `x_axis_max` 固定范围。
- `"percent": true` 显示各样本内的占比而不是数量。
- 多个系列共用分箱并叠加显示（半透明）；设置相同的 `stack` 名称则改为堆叠。

### 极坐标柱状图

`PolarBarChart` 把柱状图画在极坐标上。类目来自 `x_axis_data`；同一类目内多个系列并排显示，设置相同的 `stack` 名称则堆叠。

```json
{
  "inner_radius": 30,
  "x_axis_data": ["一月", "二月", "三月", "四月"],
  "series_list": [
    { "name": "北部", "stack": "total", "data": [42, 38, 51, 64] },
    { "name": "南部", "stack": "total", "data": [30, 34, 40, 58] }
  ]
}
```

- 默认类目沿圆周排列，柱从圆心向外生长。设置 `"category_axis": "radius"` 后每个类目占一个圆环，柱沿圆周延伸：轴上最大值默认对应四分之三圈（可用 `end_angle` 调整），`"round_cap": true` 让柱的两端变为圆头。
- `start_angle`（从 12 点方向顺时针的角度）旋转图表，`inner_radius` 在中心留出空洞，`radius` 限制大小，`category_gap`（0 到 0.9，默认 0.2）设置类目之间的留白。
- 数值轴使用 `y_axis_configs` 的第一项（`axis_min`、`axis_max`、`axis_split_number`、`axis_formatter`）；`x_axis_hidden` / `y_axis_hidden` 分别隐藏类目标签和数值标签。
- `label_show`、`colors`（每根柱单独的颜色）、`tooltip_show`、`animation` 与柱状图用法一致。

Rust 中使用 `PolarBarChart::new(series_list, x_axis_data)`，再设置 `chart.category_axis = PolarAxis::Radius`。

### 和弦图

`ChordChart` 展示节点之间相互的流量：每个节点是圆周上的一段弧，弧长对应经过它的流量；每条连接是一条横跨圆内的色带，两端的宽度对应它的数值。

```json
{
  "link_gradient": true,
  "links": [
    { "source": "亚洲", "target": "欧洲", "value": 60 },
    { "source": "亚洲", "target": "美洲", "value": 45 },
    { "source": "欧洲", "target": "美洲", "value": 50 }
  ]
}
```

- 节点默认由 `links` 中出现的名称按首次出现的顺序生成；在 `nodes`（`{"name": ..., "color": ...}`）中列出可以固定顺序或颜色。允许节点连接到自身。
- 色带默认使用源节点的颜色；`"link_gradient": true` 时从源节点颜色渐变到目标节点颜色。`link_opacity`（默认 0.5）让重叠的色带互相透出。
- `node_width`（默认 12）是节点环的厚度，`node_gap`（默认 3）是节点之间的间隔角度，`start_angle` 旋转图表，`radius` 限制大小。
- 标签默认是节点名称；`series_label_formatter` 可以加上流量（`{c}`）或占比（`{d}`），例如 `"{b} ({d})"`。`tooltip_show`、`animation` 与其它图表用法一致。

Rust 中使用 `ChordChart::new(vec![], vec![("亚洲", "欧洲", 60.0).into()])`。

### 甘特图

`GanttChart` 把任务以横条形式画在时间轴上：

```json
{
  "now": "2024-03-19",
  "tasks": [
    { "name": "调研", "category": "计划", "start": "2024-03-04", "end": "2024-03-08", "progress": 1 },
    { "name": "设计", "category": "实施", "start": "2024-03-07", "end": "2024-03-22", "progress": 0.6 },
    { "name": "评审", "category": "实施", "start": "2024-03-22" }
  ]
}
```

- 每个任务默认独占一行，行名即任务名；`row` 相同的任务共用一行（如会议室的预订、登机口的航班），此时任务名显示在任务条上。
- `category` 决定任务颜色并作为图例，`progress`（0 到 1）表示完成进度，没有 `end` 的任务是里程碑，`now` 用虚线标出一个时刻。
- x 轴沿用折线图的时间轴：`x_axis_min` / `x_axis_max`、`x_axis_formatter`、`x_axis_time_offset` 均可使用，`"x_axis_type": "value"` 则改为普通数值轴。

Rust 中使用 `GanttChart::new(vec![("调研", start, end).into()])`，时间为 unix 秒。

### 地图

`MapChart` 按数值为地图上的各个区域着色。区域数据以 GeoJSON 形式随参数传入——库本身不内置任何地图数据：

```json
{
  "label_show": true,
  "thresholds": [100, 300],
  "colors": ["#deebf7", "#9ecae1", "#3182bd"],
  "data": [["West", 80], ["East", 420]],
  "geo_json": { "type": "FeatureCollection", "features": [] }
}
```

- `geo_json` 是由 `Polygon` 和 `MultiPolygon` 几何（支持孔洞和岛屿）组成的 `FeatureCollection`；`data` 通过区域名称给出数值，名称取自 feature 的 `name` 属性（可用 `name_property` 指定其它属性）。
- 色阶与热力图一致：从 `min_color` 到 `max_color`，或经过多个 `colors`，或分段（`steps`、`thresholds`）；色阶图例显示在地图旁边。没有数值的区域使用 `empty_color`。
- `projection` 可选 `mercator`（默认）或 `equirectangular`。

Rust 中使用 `MapChart::new(MapRegion::from_geo_json(text, "name")?, data)`。

### 折线区间带

折线系列可以附带 `band`：每个点一个下界和一个上界，两者之间的区域用系列颜色半透明填充，用于置信区间、预测范围、每日最低/最高值等场景。

```json
{
  "x_axis_data": ["周一", "周二", "周三", "周四"],
  "series_list": [{
    "name": "预测",
    "data": [120, 132, 128, 141],
    "band": { "lower": [116, 126, 119, 129], "upper": [124, 138, 137, 153] }
  }]
}
```

```rust
use charts_rs::{Series, SeriesBand};
let mut series: Series = ("预测", vec![120.0, 132.0, 128.0, 141.0]).into();
series.band = Some(SeriesBand::new(
    vec![116.0, 126.0, 119.0, 129.0],
    vec![124.0, 138.0, 137.0, 153.0],
));
```

- 区间带跟随折线：系列平滑时区间带也平滑；类目轴、数值轴、时间轴均可用，柱状图中的折线系列同样支持。y 轴范围会自动包含上下界。
- 上界或下界为 `null` 的位置，区间带会断开。
- 只有 `band`、没有 `data` 的系列只绘制区间带（如最低–最高范围）。
- 开启 `tooltip_show` 后，数据点的提示会带上区间，并输出 `data-lower` / `data-upper` 属性。
- 上下界是绝对值：区间带不参与系列堆叠。

### 数据标签防重叠

`"series_label_hide_overlap": true` 会隐藏与已绘制标签重叠的数据标签。

## 精简输出

`chart.compact = true`（JSON `"compact": true`）会按优化器的方式输出 SVG：去掉空白、路径用相对坐标和 `h`/`v` 简写、网格线与刻度合并为单个 path、公共属性提升到分组、去掉默认值与长十六进制颜色。渲染结果不变，体积通常小 20–30%。对已有的输出可直接调用 `charts_rs::compact_svg(&svg)`。

## 加载更多字体

```rust
let buf = fs::read(file).unwrap();
add_fonts(&[&buf]).unwrap();
```

字体按其字体族名称注册，并且包含各种语言的名称：`"font_family": "PingFang SC"` 与 `"苹方-简"` 是同一个字体。字体集合（`.ttc`）中的每个字体都会注册，文字宽度按该字体族的常规体测量。`get_font_families()` 可列出全部名称。

未注册的 `font_family` 仍会写入 SVG（由查看器解析），但文字宽度按默认字体测量。

字体只保留原始字节，字形在测量文字时按需读取，因此加载大体积的中文字体也只占用字体文件本身大小的内存。

## 开源协议声明

This project is licensed under the [Apache-2.0 license].

[Apache-2.0 license]: https://github.com/vicanso/charts-rs/blob/main/LICENSE
