# charts-rs

[English](./README.md)

**不依赖浏览器的 Rust 图表库。** 几行 Rust 代码或一份 JSON，就能生成 27 种图表的 SVG，也可以导出 PNG、JPEG、WebP、AVIF，风格参考 Apache ECharts。

[![Crates.io][crates-badge]][crates-url]
[![Apache licensed][apache-badge]][apache-url]
[![Build status](https://github.com/vicanso/charts-rs/actions/workflows/test.yml/badge.svg?branch=main)](https://github.com/vicanso/charts-rs/actions/workflows/test.yml)

[crates-badge]: https://img.shields.io/crates/v/charts-rs.svg
[crates-url]: https://crates.io/crates/charts-rs
[apache-badge]: https://img.shields.io/badge/license-apache2-blue.svg
[apache-url]: https://github.com/vicanso/charts-rs/blob/main/LICENSE

<p align="center">
  <a href="https://github.com/vicanso/charts-rs/blob/main/docs/gallery.md"><img src="https://raw.githubusercontent.com/vicanso/charts-rs/main/asset/image/mix-line-bar.png" width="32%" alt="双 y 轴的柱状图与折线"></a>
  <a href="https://github.com/vicanso/charts-rs/blob/main/docs/gallery.md"><img src="https://raw.githubusercontent.com/vicanso/charts-rs/main/asset/image/pie-nested.png" width="32%" alt="嵌套饼图"></a>
  <a href="https://github.com/vicanso/charts-rs/blob/main/docs/gallery.md"><img src="https://raw.githubusercontent.com/vicanso/charts-rs/main/asset/image/chord.png" width="32%" alt="和弦图"></a>
</p>

<p align="center">
  <a href="https://github.com/vicanso/charts-rs/blob/main/docs/gallery.md"><img src="https://raw.githubusercontent.com/vicanso/charts-rs/main/asset/image/gantt.png" width="32%" alt="甘特图"></a>
  <a href="https://github.com/vicanso/charts-rs/blob/main/docs/gallery.md"><img src="https://raw.githubusercontent.com/vicanso/charts-rs/main/asset/image/candlestick.png" width="32%" alt="蜡烛图"></a>
  <a href="https://github.com/vicanso/charts-rs/blob/main/docs/gallery.md"><img src="https://raw.githubusercontent.com/vicanso/charts-rs/main/asset/image/map.png" width="32%" alt="地图"></a>
</p>

<p align="center">
  <a href="https://github.com/vicanso/charts-rs/blob/main/docs/gallery.md"><img src="https://raw.githubusercontent.com/vicanso/charts-rs/main/asset/image/sankey.png" width="32%" alt="桑基图"></a>
  <a href="https://github.com/vicanso/charts-rs/blob/main/docs/gallery.md"><img src="https://raw.githubusercontent.com/vicanso/charts-rs/main/asset/image/heatmap-scale.png" width="32%" alt="热力图"></a>
  <a href="https://github.com/vicanso/charts-rs/blob/main/docs/gallery.md"><img src="https://raw.githubusercontent.com/vicanso/charts-rs/main/asset/image/line.png" width="32%" alt="折线图"></a>
</p>

<p align="center">
  <a href="https://github.com/vicanso/charts-rs/blob/main/docs/gallery.md"><b>查看全部 27 种图表 →</b></a>
  &nbsp;·&nbsp;
  <a href="https://charts.npmtrend.com/"><b>在线试用 →</b></a>
</p>

## 为什么选择 charts-rs

- **只需要一个 crate。** 不用无头浏览器，也不用 Node.js：生成图表就是一次函数调用，返回 SVG。适合在服务端生成报表、邮件、机器人消息和看板。
- **快。** 生成 SVG 远低于 1 毫秒，生成 PNG 只需几毫秒（见[性能](#性能)）。
- **27 种图表**，从柱状图、折线图、饼图，到蜡烛图、热力图、桑基图、甘特图和地图，还可以把多个图表拼在一张图里。
- **Rust 或 JSON。** 可以用类型化的 API，也可以传入参数相同的 JSON。每个参数都有[文档](./docs/json-zh.md)并经过校验：写错参数名会报错并给出提示，而不是悄悄被忽略。
- **主题与字体。** 内置十种主题，可自定义；支持加载自己的字体（TTF、OTF、TTC），包括中日韩字体，占用的内存只有字体文件本身大小。

## 快速上手

```toml
[dependencies]
charts-rs = "2"
```

```rust
use charts_rs::BarChart;

let mut chart = BarChart::new(
    vec![
        ("Email", vec![120.0, 132.0, 101.0, 134.0, 90.0]).into(),
        ("Search", vec![220.0, 182.0, 191.0, 234.0, 290.0]).into(),
    ],
    ["Mon", "Tue", "Wed", "Thu", "Fri"].map(String::from).to_vec(),
);
chart.title.text = "Visits".to_string();

let svg = chart.svg().unwrap();
```

同一张图表也可以用 JSON 生成——适合直接使用请求参数或配置文件：

```rust
use charts_rs::BarChart;

let chart = BarChart::from_json(
    r#"{
        "title_text": "Visits",
        "x_axis_data": ["Mon", "Tue", "Wed", "Thu", "Fri"],
        "series_list": [
            {"name": "Email", "data": [120, 132, 101, 134, 90]},
            {"name": "Search", "data": [220, 182, 191, 234, 290]}
        ]
    }"#,
)
.unwrap();

let svg = chart.svg().unwrap();
```

导出图片需要开启 `png` feature（`jpeg`、`webp`、`avif` 各有对应的 feature）：

```toml
[dependencies]
charts-rs = { version = "2", features = ["png"] }
```

```rust
use charts_rs::{LineChart, add_fonts, svg_to_png};

// 中文需要先加载中文字体，字体族名称取自字体文件
let font = std::fs::read("NotoSansSC-Regular.ttf").unwrap();
add_fonts(&[&font]).unwrap();

let chart = LineChart::from_json(
    r#"{
        "font_family": "Noto Sans SC",
        "title_text": "访问量",
        "x_axis_data": ["周一", "周二", "周三"],
        "series_list": [{"name": "邮件", "data": [120, 132, 101]}]
    }"#,
)
.unwrap();

let png = svg_to_png(&chart.svg().unwrap()).unwrap();
std::fs::write("visits.png", png).unwrap();
```

字体的加载与命名规则见[使用指南的字体一节](./docs/guide-zh.md#字体)。

## 功能一览

- **图表**：柱状图、条形图、折线图、饼图、雷达图、散点图与气泡图、蜡烛图、表格、热力图、漏斗图、瀑布图、日历图、仪表盘、矩形树图、箱线图、旭日图、桑基图、树图、关系图、平行坐标图、主题河流图、直方图、极坐标柱状图、和弦图、甘特图、地图，以及把多个图表拼在同一画布上的 `MultiChart`。
- **系列**：堆叠与百分比堆叠、柱线混合、双 y 轴、平滑线与阶梯线、面积、区间带与误差线、标记线 / 标记点 / 标记区域、回归曲线、空值。
- **坐标轴**：类目轴、数值轴、时间轴和对数轴，支持轴标题与标签格式化。
- **外观**：主题、渐变填充、图例可放在任意一侧、数据标签自动避让、SVG 动画、悬停提示。
- **输出**：SVG（可选精简输出，体积小 20–30%），以及任意尺寸的 PNG、JPEG、WebP、AVIF。

## 文档

| | |
|---|---|
| [使用指南](./docs/guide-zh.md) | 用 Rust 和 JSON 创建图表，坐标轴、标签、字体、图片输出 |
| [JSON 参数参考](./docs/json-zh.md) | 每种图表的每个参数：类型、默认值、作用 |
| [图表一览](./docs/gallery.md) | 所有图表类型及其变体 |
| [主题](./theme.md) | 十种内置主题 |
| [示例](./examples) | `cargo run --example bar` 等 11 个可运行示例 |
| [API 文档](https://docs.rs/charts-rs) | docs.rs 上的 Rust API |
| [更新日志](./CHANGELOG.md) | 版本变更，以及从 1.x 迁移到 2.0 的说明 |
| [在线示例](https://charts.npmtrend.com/) | 修改 JSON 即时查看图表（[源码](https://github.com/vicanso/charts-rs-web)） |

## 性能

生成一张图表的 SVG 远低于 1 毫秒；转成 PNG 需要几毫秒，时间花在解析 SVG、光栅化和编码上。以下为 600 × 400 像素的图表，Apple M4 Pro 单线程，Rust 1.99：

| 图表 | SVG | PNG（由该 SVG 生成） |
|------|----:|--------------------:|
| 柱状图：4 个系列各 7 个值，其中一个系列为折线，带数据标签 | 41 µs | 1.7 ms |
| 折线图：2 个系列各 100 个点，平滑并填充 | 158 µs | 3.4 ms |
| 饼图：12 个扇区 | 52 µs | 1.4 ms |
| 桑基图：8 个节点、10 条连线 | 71 µs | 1.2 ms |
| 上面的柱状图，输出为 1200 × 800 像素 | 41 µs | 3.4 ms |

数据来自 `cargo bench --features png`（[`benches/bench.rs`](./benches/bench.rs)，其中还有更多图表的 SVG 基准），具体数值取决于机器。

## 最低 Rust 版本

charts-rs 2.x 需要 Rust 1.88 及以上（edition 2024）。

## 开源协议声明

This project is licensed under the [Apache-2.0 license].

[Apache-2.0 license]: https://github.com/vicanso/charts-rs/blob/main/LICENSE
