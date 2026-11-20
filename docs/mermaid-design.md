# Mermaid 渲染支持架构设计与规划方案

> 本文档记录 `charts-rs` 新增纯 Rust Mermaid 渲染能力的完整架构设计、模块划分与分阶段实施规划。
>
> **当前状态（v3 已封版）**：已完成全部三期共 **13 类** Mermaid 图表的纯 Rust 渲染支持（v1：`flowchart`/`graph`、`sequenceDiagram`、`pie`、`gantt`、`sankey-beta`、`xychart-beta`；v2：`stateDiagram`/`stateDiagram-v2`、`classDiagram`/`classDiagram-v2`、`erDiagram`；v3：`mindmap`、`quadrantChart`、`radar-beta`/`radar`、`treemap-beta`/`treemap`），零新增外部依赖，见文末第七、八节。

## 一、背景与可行性分析

### 1.1 现有基础

从当前代码库结构来看（[Cargo.toml](../Cargo.toml)、[lib.rs](../src/lib.rs)、[canvas.rs](../src/charts/canvas.rs)、[component.rs](../src/charts/component.rs)），`charts-rs` 已经具备构建纯 Rust Mermaid 渲染引擎的完整底层设施：

- **纯 Rust / 零浏览器 / 内存安全**：全 crate 开启 `#![forbid(unsafe_code)]` 与 `#![warn(missing_docs)]`，无需 Node.js 或无头浏览器即可输出确定性 SVG。
- **统一的 SVG 渲染管线**：`Canvas` 与 `Component` 提供了基础矢量图元、渐变与紧凑输出（`compact_svg`）。
- **精确字形测量**：基于 `ttf-parser` 的 `measure_text_width_family`（[font.rs](../src/charts/font.rs)）已内嵌 `Roboto.ttf` 并支持自定义 TTF/OTF/TTC 及 CJK 字符宽度测量。
- **主题与光栅导出**：10 套内置 `Theme`（[theme.rs](../src/charts/theme.rs)）与可选的 `resvg` + `image` 光栅导出管线（PNG/JPEG/WebP/AVIF，[encoder.rs](../src/charts/encoder.rs)）。

### 1.2 纯 SVG 路线与 `resvg` 兼容性

官方 `mermaid.js` 默认依赖浏览器 DOM 与 `<foreignObject>` HTML 排版计算节点文字尺寸，这在无头纯 Rust 环境下不可用，且会导致 `resvg` 无法直接光栅化。`charts-rs` 采用**纯 SVG 矢量图元 + 基于 `ttf-parser` 的纯 Rust 多行排版测量**，生成的 Mermaid SVG 可直接通过 `svg_to_png` 等接口导出为光栅图像。

### 1.3 现有图表复用盘点

在 `charts-rs` 现有的 29 种图表中，部分数据类 Mermaid 语法可直接复用现有实现：

- `pie` → [`PieChart`](../src/charts/pie_chart.rs)
- `gantt` → [`GanttChart`](../src/charts/gantt_chart.rs)
- `sankey-beta` → [`SankeyChart`](../src/charts/sankey_chart.rs)
- `xychart-beta` → [`BarChart`](../src/charts/bar_chart.rs)（原生支持 `SeriesCategory::Line` 实现柱线混排）

而 `flowchart`、`sequenceDiagram` 等关系/时序图则需要新增专用的纯 Rust 布局与绘制引擎。

---

## 二、已对齐的核心设计决策

| 维度 | 决策结论 |
|---|---|
| **架构定位** | **独立的纯 Rust / 纯 SVG Mermaid 渲染层 + 复用已有数据图表**：提供统一的 Mermaid 解析与渲染入口；`pie`/`gantt`/`sankey`/`xychart` 直接映射到现有图表，`flowchart`/`sequence` 实现专用布局与绘制引擎 |
| **首批范围 (v1)** | `flowchart` / `graph`、`sequenceDiagram`、复用现有数据图表（`pie`、`gantt`、`sankey-beta`、`xychart-beta`）；分层布局引擎模块化，为后续扩展 `stateDiagram`/`classDiagram`/`erDiagram` 留好边界 |
| **打包形态** | 作为 [Cargo.toml](../Cargo.toml) 的可选 feature **`mermaid`**（默认关闭），类似 `png`/`jpeg`，不影响现有用户编译时间与 WASM 体积 |
| **依赖策略** | **零新增外部依赖，全部纯 Rust 手写**：延续极简依赖、`#![forbid(unsafe_code)]` 与 100% 确定性快照输出 |
| **对外 API** | **`MermaidChart` 结构体 + 便捷函数**：对齐 `charts-rs` 体系（`ChartBase`、`new`/`new_with_theme`/`from_json`/`svg`、`Chart` trait、`MultiChart` 接入）+ `mermaid_to_svg` / `mermaid_to_svg_with_theme` |
| **画布尺寸策略** | **默认自适应内容尺寸 + 显式设置时等比缩放**：`flowchart`/`sequence` 默认按布局结果自然撑开宽高；数据类图沿用合理默认尺寸；若用户显式设置了 `width` 或 `height`，则通过 `viewBox` 等比缩放适配 |
| **主题与内联样式** | **默认跟随 `charts-rs` `Theme` + 支持 Mermaid 常用内联样式覆盖**：节点/连线/子图/时序块配色从当前 `Theme` 自动派生，同时支持 `style`/`classDef`/`class`/`:::`/`linkStyle`、YAML frontmatter `title` 与基础 `%%{init}%%` 映射 |
| **容错与文本处理** | **结构错误严格带行号报错 + 装饰性指令宽松降级**：不支持的图类型、未闭合块、无效语句返回带行号的 `Error`；`click`/`accTitle`/未知 `init` 键或未知 CSS 属性静默忽略；`<br/>`/`\n` 转多行，常见 HTML 实体与内联标签安全还原/剥离 |
| **Flowchart 视觉** | **平滑曲线（三次贝塞尔穿过分层路径点）+ 标准嵌套 `subgraph`**：支持嵌套子图及子图作为连线端点（子图内部独立 `direction` 可后续迭代补齐） |

---

## 三、公开 API 与现有体系集成

### 3.1 新增公开类型与函数（`#[cfg(feature = "mermaid")]`）

#### `MermaidChart`

嵌入 `pub base: ChartBase` 并实现 `Deref` / `DerefMut<Target = ChartBase>` 以及 [`Chart`](../src/charts.rs) trait：

```rust
pub struct MermaidChart {
    /// 共享图表配置（尺寸、边距、字体、主题配色、标题、水印 texts、compact 等），
    /// 通过 Deref 直接暴露，如 `chart.margin`、`chart.compact`。
    pub base: ChartBase,
    /// Mermaid 源码文本。
    pub source: String,
}
```

- **构造方法与渲染入口**：
  - `MermaidChart::new(source: impl Into<String>) -> Self`
  - `MermaidChart::new_with_theme(source: impl Into<String>, theme: &str) -> Self`
  - `MermaidChart::from_json(json: &str) -> Result<Self>`
  - `MermaidChart::svg(&self) -> Result<String>`
  - `pub(crate) fn render(&self) -> Result<(String, f32, f32)>`：返回 `(svg_fragment, actual_width, actual_height)`，参照 [`TableChart`](../src/charts/table_chart.rs) 在 [`MultiChart`](../src/charts/multi_chart.rs) 中的动态高度模式。
- **JSON Schema 字段（[schema.rs](../src/charts/schema.rs)）**：
  - 严格校验 `MERMAID_FIELDS`（包含 `source: Kind::String`）+ `BASE_FIELDS` 中的通用选项（`theme`、`width`、`height`、`margin`、`font_family`、`background_color`、`compact`、`title_text`、`title_*`、`texts` 等）。
  - 注意：`from_json` 在未显式传入 `width` / `height` 时保持 `0.0`（代表 auto 自适应模式），而非强制填入主题默认的 `600×400`。

#### 顶层快捷函数

- `mermaid_to_svg(source: &str) -> Result<String>`
- `mermaid_to_svg_with_theme(source: &str, theme: &str) -> Result<String>`

### 3.2 现有文件最小改动清单

1. **[Cargo.toml](../Cargo.toml)**：
   - 在 `[features]` 下新增 `mermaid = []`，不添加任何新 crate 依赖。
2. **[src/charts/error.rs](../src/charts/error.rs)**：
   - 参照 `#[cfg(feature = "raster")]` 方式，在 `#[non_exhaustive] pub enum Error` 中新增：
     ```rust
     /// Failed to parse or render a Mermaid diagram.
     #[cfg(feature = "mermaid")]
     Mermaid {
         /// 1-based source line number where the error occurred, if available.
         line: Option<usize>,
         /// Human-readable description of the syntax or layout error.
         message: String,
     },
     ```
3. **[src/charts/canvas.rs](../src/charts/canvas.rs) & [src/charts/component.rs](../src/charts/component.rs)**：
   - `Canvas` 新增 `pub(crate) view_box: Option<(f32, f32)>`：当未设置时与现有一致输出 `viewBox="0 0 {width} {height}"`；当设置时输出 `width="{width}" height="{height}" viewBox="0 0 {vb_w} {vb_h}"`，由 SVG 原生完成等比缩放与居中。
   - `Component` 新增 `pub(crate) RawPath`（或等价内部路径组件，支持自定义 SVG `d` 字符串、`fill`、`stroke`、`stroke_width`、`stroke_dash_array`），用于绘制数据库圆柱体、胶囊弧、菱形/六边形/平行四边形边框、平滑三次贝塞尔连线及多种箭头末端。
   - `Rect` 可选增加 `pub stroke_dash_array: Option<String>`（仅在 `Some` 且非空时输出，不影响已有快照），用于绘制虚线 `subgraph` 或分组框。
4. **[src/charts/multi_chart.rs](../src/charts/multi_chart.rs) & [src/charts/schema.rs](../src/charts/schema.rs)**：
   - `ChildChart` 新增 `#[cfg(feature = "mermaid")] Mermaid(MermaidChart, Option<(f32, f32)>)`。
   - `MultiChart::from_json` 增加 `#[cfg(feature = "mermaid")] "mermaid"` 分支；`MultiChart::svg` 中通过 `c.render()?` 获取实际渲染宽高并推进 `x` / `y` 游标。
5. **[src/charts.rs](../src/charts.rs)**：
   - 增加 `#[cfg(feature = "mermaid")] mod mermaid;` 与 `#[cfg(feature = "mermaid")] pub use mermaid::{MermaidChart, mermaid_to_svg, mermaid_to_svg_with_theme};`。
   - 为 `MermaidChart` 实现 `Chart` trait。

---

## 四、模块架构与目录规划

所有 Mermaid 实现均收拢在 `src/charts/mermaid/` 目录下，并通过 `#[cfg(feature = "mermaid")]` 编译隔离：

```text
src/charts/mermaid/
├── mod.rs          # MermaidChart、mermaid_to_svg*、render() 总入口、画布尺寸/viewBox 调度与 title/texts 叠加
├── preprocess.rs   # YAML frontmatter (title/config)、%%{init:...}%% 指令、%% 注释清理、行号映射与图类型分发
├── style.rs        # Theme -> MermaidPalette 派生；style / classDef / class / linkStyle CSS 解析与颜色合并
├── text.rs         # 标签预处理（<br/>、\n、HTML 实体、<b>/<i> 清理）、基于 ttf-parser 的多行排版与尺寸测量
├── draw.rs         # 纯 SVG 通用绘制原语：节点多形状（14 种）、平滑三次贝塞尔边、箭头/圆头/叉头、多行文本盒
├── layout/
│   ├── mod.rs      # 通用分层有向图抽象接口（LayeredGraph -> LayeredLayout），供 flowchart 及 v2 扩展图复用
│   └── sugiyama.rs # 4 阶段 Sugiyama 确定性布局算法 + 嵌套 subgraph 包围盒与虚拟节点路由
├── flowchart.rs    # flowchart / graph 词法行解析 -> IR -> layout::sugiyama -> draw 渲染
├── sequence.rs     # sequenceDiagram 解析 -> IR -> 二维列/行时间线布局 -> Canvas 分层绘制
└── adapters/
    ├── mod.rs      # 复用图表通用适配工具（主题、标题、尺寸、margin、compact 透传）
    ├── pie.rs      # Mermaid pie -> PieChart
    ├── gantt.rs    # Mermaid gantt -> GanttChart（含 dateFormat / duration / after 依赖推算）
    ├── sankey.rs   # Mermaid sankey-beta -> SankeyChart
    └── xychart.rs  # Mermaid xychart-beta -> BarChart（支持 bar + line 混排）
```

---

## 五、核心子系统技术方案

### 5.1 预处理与分发（`preprocess.rs`）

输入源码先经过统一的预处理流水线，保留每一条有效语句的原始 **1-based 行号**：

1. **YAML Frontmatter**：识别源码开头的 `--- ... ---` 区块，提取 `title`（当 `MermaidChart.title.text` 为空时自动回填）和 `config.theme`。
2. **`%%{init: ...}%%` 指令**：支持跨行与单引号 JSON（Mermaid 常见写法 `%%{init: {'theme': 'dark'}}%%`），提取 `theme` 字段映射至内置主题（仅当用户未在 Rust/JSON 侧显式指定主题时生效）：
   - `"dark"` → `THEME_DARK`
   - `"default"` / `"base"` / `"neutral"` / `"forest"` → 映射至对应预设或当前主题（未知配置键静默忽略）。
3. **注释清理与行号跟踪**：在不破坏字符串字面量的前提下剥离 `%%` 行尾注释与空行，形成 `Vec<SourceLine { line_no: usize, text: String }>`。
4. **图类型识别与分发**：读取首条非空逻辑行头部关键词：
   - `flowchart` / `graph`（带 `TB` / `TD` / `BT` / `LR` / `RL`）→ `flowchart`
   - `sequenceDiagram` → `sequence`
   - `pie`（可选 `showData`）→ `adapters::pie`
   - `gantt` → `adapters::gantt`
   - `sankey-beta` → `adapters::sankey`
   - `xychart-beta` → `adapters::xychart`
   - 其他未支持头部（如 `gitGraph`、`C4Context` 等）→ 返回 `Error::Mermaid { line: Some(n), message: ... }`。

### 5.2 主题派生与内联样式（`style.rs`）

为保证在 `charts-rs` 全部 10 套主题（`light`、`dark`、`ant`、`grafana`、`vintage`、`walden`、`westeros`、`chalk`、`shine`、`shadcn`）下开箱美观，定义由 `Theme` 自动计算的 `MermaidPalette`：

- **画布背景**：`theme.background_color`
- **默认节点填充**：根据 `theme.is_light` 将 `series_colors[0]` 与 `background_color` 按柔和比例混色（浅色主题为淡底色，深色主题为微提亮底色）
- **默认节点描边 & 连线主色**：取 `series_colors[0]` 与 `axis_stroke_color`，确保与背景具备清晰对比度
- **节点/边/参与者文字颜色**：取自 `theme.title.font.color` / `theme.x_axis.font.color`
- **边标签背景板**：与 `background_color` 同色系半透明圆角矩形，防止连线穿过标签时影响阅读
- **容器与分组（`subgraph` / `alt` / `loop`）**：按嵌套深度递增微调底色明度/透明度 + 虚线/细实线边框

**内联样式覆盖优先级**：`Theme 派生默认值` < `classDef`（通过 `class A,B cls` 或节点后缀 `:::cls` 应用）< `style <nodeId>` / `linkStyle <index|default>`。
- CSS 属性支持：`fill`、`stroke`、`stroke-width`、`stroke-dasharray`、`color`（文字色）。
- 颜色格式支持：`#rgb`、`#rrggbb`、`#rrggbbaa`、`rgb(...)`、`rgba(...)` 与常用 CSS 命名色（扩展基于现有 [`Color::parse`](../src/charts/color.rs)）。
- 未识别的 CSS 属性或浏览器专属特性（如 `rx` 之外的未知属性）静默忽略。

### 5.3 多行文本与字体测量（`text.rs`）

基于 `measure_text_width_family`（[font.rs](../src/charts/font.rs)）实现纯 SVG 多行文本测量与渲染：

1. **换行标准化**：将 `<br>`, `<br/>`, `<br />` 及转义 `\n` 拆分为 `Vec<TextLine>`。
2. **实体与内联标签处理**：
   - 还原常见 XML/HTML/Mermaid 实体（`&amp;`、`&lt;`、`&gt;`、`&quot;`、`&#39;`、`#quot;`、`#35;`、`#59;` 等）。
   - 识别轻量 `<b>`/`<strong>` 设置行级加粗样式；安全剥离其他 HTML 标签，避免 `<span ...>` 或 `<i>` 原文出现在 SVG 文本中。
3. **纯 SVG 输出**：逐行调用 `measure_text_width_family` 计算最大行宽与总行高（`line_count * line_height`），渲染为多个垂直居中的 `<text>`（不依赖 `resvg` 支持受限的复杂 `<tspan>` 定位）。

### 5.4 Flowchart 引擎（`flowchart.rs` + `layout/sugiyama.rs` + `draw.rs`）

#### 语法覆盖范围

- **布局方向**：`TB` / `TD`（自上而下）、`BT`（自下而上）、`LR`（自左而右）、`RL`（自右而左）。
- **14 种常用节点形状**：
  - `[text]` 矩形、`(text)` 圆角矩形、`([text])` 胶囊型、`[[text]]` 子程序双竖线矩形、`[(text)]` 数据库圆柱体
  - `((text))` 圆形、`(((text)))` 双重圆形、`{text}` 菱形、`{{text}}` 六边形
  - `[/text/]` 平行四边形、`[\text\]` 反向平行四边形、`[/text\]` 正梯形、`[\text/]` 倒梯形、`>text]` 非对称旗形
  - 支持双引号包裹转义文本（`A["A & B (v1)"]`）以及内联类后缀 `:::className`。
- **连线类型与标签**：
  - 边样式：实线箭头 `-->`、实线无箭头 `---`、虚线箭头 `-.->`、虚线无箭头 `-.-`、粗线箭头 `==>`、粗线无箭头 `===`、隐形占位边 `~~~`
  - 末端与双向：`<-->`、`<==>`、`<-.->`、`--o`、`o--o`、`--x`、`x--x`
  - 边标签：`A -->|label| B`、`A -- label --> B`、`A -. label .-> B`、`A == label ==> B`
  - 语法糖：链式连线 `A --> B --> C`、多对多并行 `A & B --> C & D`、分号 `;` 分隔单行多语句
- **子图与样式**：
  - 多层嵌套 `subgraph id [Title] ... end`
  - 支持将 `subgraph` ID 直接作为连线端点（如 `A --> sub1`）
  - 支持 `style`、`classDef`、`class`、`linkStyle`；`click`、`accTitle`、`accDescr` 静默忽略。

#### 通用分层布局接口（`layout/mod.rs`）

将节点、边、子图分组抽离为独立于 Mermaid 语法的 `LayeredGraph` 输入与 `LayeredLayout` 输出（节点坐标、边路径控制点序列、子图包围盒），使 v2 的 `stateDiagram` / `classDiagram` / `erDiagram` 可直接复用。

#### 4 阶段 Sugiyama 确定性布局（`layout/sugiyama.rs`）

1. **消环（Cycle Removal）**：按声明确定性顺序执行 DFS，标记并临时反转回边，确保拓扑排序成立；自环边（`A --> A`）单独记录为节点旁侧环弧。
2. **分层（Rank Assignment）**：基于最长路径分配初始层级（rank），并做简单拉紧（tightening）缩短长边跨度；将跨越多层的边拆分为连续虚拟节点（dummy nodes）以参与后续排序与走线。
3. **交叉最小化（Crossing Reduction）**：多轮自上而下 / 自下而上的重心启发式（Barycenter heuristic）排序同层节点；对同属一个 `subgraph` 的节点施加聚类偏置约束，减少子图交错穿插。
4. **坐标分配与子图包围**：
   - 先沿标准 `TB` 坐标系计算节点中心点：按层内各节点实际 `ttf-parser` 测量尺寸与间距累加横坐标，利用上下层邻接重心多轮微调对齐垂直轴线。
   - 再根据目标方向（`BT` / `LR` / `RL`）统一执行坐标轴翻转/转置。
   - 自底向上计算所有 `subgraph` 的合并包围盒，预留顶部标题与内边距，同时将指向/离开 `subgraph` 的连线端点裁剪到子图边界框。
   - 对所有边路径点：将首尾点精确投影裁剪到对应节点几何轮廓（矩形、菱形、正圆、六边形、圆柱等），在 `draw.rs` 中生成**平滑三次贝塞尔曲线**与箭头末端，并在曲线中点渲染带底板的边标签。

### 5.5 SequenceDiagram 引擎（`sequence.rs`）

#### 语法覆盖范围

- **参与者与分组**：`participant A as Alice`、`actor U as User`（绘制火柴人矢量图标）；未显式声明的参与者按首次出现顺序自动注册；`box [color] [Title] ... end` 参与者分组背景；`create` / `destroy`。
- **消息与生命周期**：
  - 箭头类型：实线有箭 `->>`、虚线有箭 `-->>`、实线无箭 `->`、虚线无箭 `-->`、实线叉 `--x`/`-x`、虚线异步 `-)`/`--)`、双向 `<<->>`/`<<-->>`。
  - 激活条：消息后缀快捷 `+`/`-` 与独立 `activate` / `deactivate` 语句，支持同参与者多层嵌套激活（横向梯级偏置）。
  - 自发消息：`A ->> A: self` 绘制向右突出的回环折线。
- **注释与编号**：`Note right of A`、`Note left of A`、`Note over A` / `Note over A,B`、`autonumber`。
- **控制流与高亮块**：`alt`/`else`、`opt`、`loop`、`par`/`and`、`critical`/`option`、`break`、`rect rgb(...)`，支持嵌套与左上方五边形折角标签盒。

#### 二维列+行布局与分层绘制

1. **列宽动态计算**：按每个参与者框体宽度以及跨列消息标签、`Note over` 宽度作为约束，自适应撑开相邻参与者间距，保证标签绝不跨线挤压重叠。
2. **行高累计**：逐条事件累加垂直 Y 坐标，为多行消息文本、自发消息、块头部/分隔线预留精确垂直空间。
3. **严格分层绘制顺序**：`box`/`rect` 背景框 → 垂直生命线 → 激活条矩形 → 控制流块边框与标签 → 消息线段与箭头 → 顶部（及底部镜像）参与者框 → `Note` 与文字。

### 5.6 现有数据图表映射器（`adapters/`）

| Mermaid 类型 | 目标图表 | 关键映射规则 |
|---|---|---|
| `pie [showData]` | [`PieChart`](../src/charts/pie_chart.rs) | `title ...` → `chart.title.text`；`"Label" : value` → `Series`；`showData` 时通过标签 formatter 展示数值与占比 |
| `gantt` | [`GanttChart`](../src/charts/gantt_chart.rs) | 解析 `title`、`dateFormat`（支持常见 `YYYY-MM-DD` / `YYYY-MM-DD HH:mm` 等纯 Rust 解析）、`section` → `GanttTask::category`、任务属性（`done`/`active`/`crit`/`milestone`、任务 ID、起止日期 / `after <id>` 依赖拓扑推算、时长 `3d`/`2w`/`12h`、`todayMarker` → `chart.now`） |
| `sankey-beta` | [`SankeyChart`](../src/charts/sankey_chart.rs) | 解析 CSV 三元组 `source,target,value`（处理双引号与空行）→ `SankeyLink`（由 `SankeyChart` 自动派生节点） |
| `xychart-beta` | [`BarChart`](../src/charts/bar_chart.rs) | 解析 `title`、`x-axis [a, b, ...]` / `min --> max`、`y-axis "Title" min --> max`、`bar [...]`、`line [...]`（利用 `SeriesCategory::Line` 实现柱线混排） |

### 5.7 画布尺寸与 `viewBox` 协同规则

`MermaidChart::new_with_theme` 初始化时保持 `base.width = 0.0`、`base.height = 0.0` 作为 **auto 模式**标记：

- **自定义引擎（`flowchart`、`sequence`）**：
  - `width == 0 && height == 0`：输出自然布局尺寸 `(content_w, content_h)` 与 `viewBox="0 0 content_w content_h"`。
  - 仅指定其一（如 `width = 800`）：另一边按比例计算 `height = content_h * (800 / content_w)`，并设置 `Canvas::view_box = Some((content_w, content_h))` 实现无失真等比缩放。
  - 两者均显式指定（`width = W, height = H`）：输出 `width="W" height="H" viewBox="0 0 content_w content_h"`，由 SVG / `resvg` 自动等比缩放居中。
- **复用数据图表（`pie`/`gantt`/`sankey`/`xychart`）**：
  - Auto 模式下使用各图表合理默认画布（如 `600×400`，`gantt` 根据任务行数适当增加默认高度避免拥挤）。
  - 用户显式指定 `width` / `height` 时，直接透传给底层目标图表，按目标分辨率原生重排文字与绘图区。

### 5.8 错误处理与语法容错矩阵

| 类别 | 具体场景 | 处理行为 |
|---|---|---|
| **结构错误（严格）** | 空源码、未知/未支持图类型、`subgraph`/`alt`/`loop`/`box` 未闭合或多余 `end`、非法连线语法、无效数值（如 `pie` 负值/非数字、`gantt` 无法解析的日期或循环 `after` 依赖） | 返回 `Error::Mermaid { line: Some(n), message }` |
| **装饰性指令（宽松）** | `click ...`、`accTitle`、`accDescr`、未知 `%%{init}%%` 键、节点内未知 CSS 属性（`style` / `classDef`） | 静默忽略，不影响出图 |
| **文本标签（安全清理）** | `<br/>` / `<br>` / `\n`、XML/HTML/Mermaid 字符实体、`<b>`/`<strong>`、`<i>` 等其他 HTML 标签 | 换行拆行、实体还原、加粗保留、其余标签安全剥离 |

---

## 六、分阶段实施路线与验收标准

### Phase 1 — Feature 骨架 + 公共层 + 数据图表映射

- 新增 [Cargo.toml](../Cargo.toml) `mermaid` feature、`Error::Mermaid`、`Canvas::view_box`、`RawPath` 原语与 `Rect::stroke_dash_array`。
- 实现 `MermaidChart`、`mermaid_to_svg*`、`schema.rs` 校验、`preprocess.rs`、`style.rs`、`text.rs`。
- 实现 `adapters/{pie, gantt, sankey, xychart}.rs`。
- **验收**：
  - `cargo test`（不启用 `mermaid`）零回归、无额外依赖；
  - `cargo test --features "mermaid,image-encoder"` 通过四类数据图表与基础错误处理单元测试。

### Phase 2 — Flowchart 引擎

- 实现 `layout/mod.rs`、`layout/sugiyama.rs`、`draw.rs`、`flowchart.rs`。
- 覆盖 `TB`/`TD`/`BT`/`LR`/`RL` 方向、14 种节点形状、实/虚/粗/无向/双向/自环/圆头/叉头连线、平滑三次贝塞尔路径与带底板标签、多层嵌套 `subgraph` 及 `style`/`classDef`/`linkStyle`。
- **验收**：新增快照测试（覆盖多方向、环路回边、子图嵌套、CJK 多行标签、明暗主题、`width`/`height` 缩放、PNG 光栅渲染验证）。

### Phase 3 — SequenceDiagram + `MultiChart` 集成 + 文档

- 实现 `sequence.rs`（`participant`/`actor`/`box`、全箭头种类、嵌套激活条、自发消息、`Note`、`autonumber`、`alt`/`opt`/`loop`/`par`/`critical`/`break`/`rect`）。
- 接入 `ChildChart::Mermaid` 到 [multi_chart.rs](../src/charts/multi_chart.rs) 与 JSON Schema。
- 完善 `tests/` 集成测试与 `examples/mermaid.rs`，按 [CLAUDE.md](../CLAUDE.md) 规范更新 `docs/guide.md`、`docs/json.md` 等文档，并在最终合并前执行全量检查：
  ```bash
  cargo test
  cargo test --features "mermaid,image-encoder"
  cargo clippy --all-features --all-targets -- --deny=warnings
  ```

---

## 七、v2 扩展方案（`stateDiagram-v2` / `classDiagram` / `erDiagram`）

在已完成 v1 首批 6 种 Mermaid 图表（`flowchart`/`graph`、`sequenceDiagram`、`pie`、`gantt`、`sankey-beta`、`xychart-beta`）的基础上，v2 重点补齐工程技术文档中最核心的三类图：**`stateDiagram` / `stateDiagram-v2`**、**`classDiagram` / `classDiagram-v2`** 与 **`erDiagram`**，完成后 `charts-rs` 将支持 **9 种** Mermaid 图表。

### 7.1 v2 范围与核心决策汇总

| 维度 | 决策结论 |
|---|---|
| **新增范围 (v2)** | `stateDiagram` / `stateDiagram-v2`、`classDiagram` / `classDiagram-v2`、`erDiagram`（完成后累计支持 9 种 Mermaid 图表） |
| **Feature 与依赖** | 继续统一收拢在现有 `mermaid` Cargo feature 下（不拆分子 feature）；零新增外部依赖，`#![forbid(unsafe_code)]`，全面兼容 10 套内置主题、`auto`/`viewBox` 缩放、`MultiChart` 嵌入与 `resvg` 光栅导出 |
| **架构复用策略** | 全部复用 [preprocess.rs](../src/charts/mermaid/preprocess.rs)、[style.rs](../src/charts/mermaid/style.rs)、[text.rs](../src/charts/mermaid/text.rs)、[layout/sugiyama.rs](../src/charts/mermaid/layout/sugiyama.rs) 与 [draw.rs](../src/charts/mermaid/draw.rs)，只按需扩展通用绘制原语与布局约束 |
| **Note 与并发分区策略** | **防重叠优先**：`note` 作为独立节点通过虚线连接目标参与 Sugiyama 布局（同层施加邻近亲和约束贴靠目标左右侧）；`stateDiagram` 的 `--` 并发分区拆为独立并排子区域并在区域间绘制虚线分隔，杜绝压线与撞框 |
| **交付清单** | 新增 `src/charts/mermaid/{state,class,er}.rs`、单元与集成测试、`asset/mermaid_chart/{state,class,er}` JSON/SVG 快照、`asset/image/mermaid-{state,class,er}.png` 预览图（逐张视觉验收），同步更新 [gallery.md](./gallery.md)、[guide.md](./guide.md)、[guide-zh.md](./guide-zh.md)、[json.md](./json.md) 与 [json-zh.md](./json-zh.md) |

### 7.2 共享层与目录扩展

新增三个独立图种模块，`src/charts/mermaid/` 目录演进为：

```text
src/charts/mermaid/
├── ... (v1 既有模块)
├── state.rs        # stateDiagram / stateDiagram-v2 解析与渲染（复用 sugiyama + draw）
├── class.rs        # classDiagram / classDiagram-v2 解析与渲染（复用 sugiyama + draw）
└── er.rs           # erDiagram 解析与渲染（复用 sugiyama + draw）
```

共享模块扩展要点：

1. **[preprocess.rs](../src/charts/mermaid/preprocess.rs)**：
   - `DiagramKind` 新增 `State`（匹配首行 `stateDiagram` / `stateDiagram-v2`）、`Class`（匹配 `classDiagram` / `classDiagram-v2`）、`Er`（匹配 `erDiagram`）。
   - 统一保留 YAML frontmatter `title` / `config.theme`、`%%{init:...}%%` 指令解析与 1-based 行号注释清理。
2. **[draw.rs](../src/charts/mermaid/draw.rs)**：
   - 扩展 `EdgeMarker`，新增 UML 端符（`OpenArrow` 开放箭头、`HollowTriangle` 继承/实现空心三角、`FilledDiamond` 组合实心菱形、`HollowDiamond` 聚合空心菱形、`Lollipop` 接口空心圆）与 Crow's Foot 端符（`ErOnlyOne` `||`、`ErZeroOrOne` `|o`/`o|`、`ErOneOrMore` `}|`/`|{`、`ErZeroOrMore` `}o`/`o{`）。
   - 新增边端点基数标签绘制辅助函数：沿曲线首/尾切线方向偏移并在法向错位渲染，配与 `palette.edge_label_bg` 一致的轻量圆角底板。
   - 抽取通用折角便签（`Note`）绘制原语与节点多段线/表格分栏绘制辅助函数，供 `state.rs`、`class.rs`、`er.rs` 复用。
3. **[layout/mod.rs](../src/charts/mermaid/layout/mod.rs) & [layout/sugiyama.rs](../src/charts/mermaid/layout/sugiyama.rs)**：
   - **Note 邻近布局**：在节点排序/对齐阶段为关联了 `note` 的目标边施加邻近亲和约束（紧邻 `left`/`right` 目标节点放置并计入同层防重叠间距），避免 `note` 飘远或遮挡其他节点与连线。
   - **并发分区容器**：扩展子图元数据支持同级并发区域分组（父容器绘制统一外框与标题，相邻并发分区的内部区域自动并排隔离，并在交界处渲染虚线分隔线）。
   - 沿用现有子图与外部节点跨轴防重叠约束、内部跨层虚拟节点归属判定逻辑，保证复合状态与 `namespace` 内外部元素零重叠。

### 7.3 `stateDiagram` / `stateDiagram-v2` 设计（`state.rs` — 完整常用集）

- **头部与方向**：支持 `stateDiagram`、`stateDiagram-v2`，以及图表与复合状态内的 `direction TB`/`TD`/`LR`/`BT`/`RL`。
- **状态与伪状态**：
  - `[*]` 伪状态：起始渲染为实心圆，终止渲染为牛眼双圆；当图内或复合状态内多次出现 `[*]` 时拆分为独立布局实例（区分起点与终点），避免引入伪环与扭曲拓扑层级。
  - 状态声明与描述：支持简写状态节点、`s1 : Description`（多次出现追加为多行描述，节点框由标题区 + 水平分隔细线 + 描述区构成）与 `state "Display Label" as s1`。
  - 进阶伪状态：`state cond <<choice>>`（菱形分支）、`state f <<fork>>` / `state j <<join>>`（随主方向旋转的高对比度同步粗条）。
- **复合状态与并发分区**：
  - 支持多层嵌套 `state Composite { ... }` 与 `state "Title" as Comp { ... }`。
  - 支持复合状态内以 `--` 分隔并发区域，各分区在父状态框内独立排布并绘制虚线分界线。
- **转移、注释与样式**：
  - 转移连线 `A --> B`、`A --> B : trigger [guard] / action`，带平滑三次贝塞尔曲线与带底板边标签。
  - `note left of State` / `note right of State`（支持单行 `note left of S : text` 与多行 `note right of S ... end note`），作为带虚线连接的便签节点参与 Sugiyama 布局防重叠。
  - 支持 `style`、`classDef`、`class` 与 `:::` 内联类；`accTitle`、`accDescr`、`click` 静默忽略；未闭合 `{}`/`end note` 或非法转移返回带 1-based 行号的 `Error::Mermaid`。

### 7.4 `classDiagram` / `classDiagram-v2` 设计（`class.rs` — 完整工程集）

- **类声明与泛型别名**：
  - `classDiagram` / `classDiagram-v2` 与 `direction TB`/`TD`/`LR`/`BT`/`RL`。
  - 支持 `class Foo { ... }` 代码块与 `Foo : +member` 单行成员声明；支持显示别名 `class Foo["Display Label"]`。
  - 泛型语法 `~T~`（含嵌套如 `Map~String, List~Item~~`）规范还原为 `<T>`。
  - 类构造型注解：`<<interface>>`、`<<abstract>>`、`<<service>>`、`<<enumeration>>` 等置顶居中斜体/浅色展示，支持内联或块首声明。
- **三段式类框与成员修饰**：
  - 按 UML 标准拆分为**头部（注解 + 粗体类名）｜属性区｜方法区**，空属性区或空方法区自动折叠以节省画布空间，并基于 `ttf-parser` 精确测量三区最大行宽。
  - 可见性符号（`+` public、`-` private、`#` protected、`~` package/internal）清晰对齐展示。
  - 分类修饰符：尾部 `*` 标记为抽象（斜体/标识），`$` 标记为静态（通过纯 SVG 矢量 `<line>` 绘制精确下划线，确保 `resvg` 与浏览器渲染完全一致）。
- **UML 关系、接口与基数**：
  - 完整支持 8 种 UML 关系端符及双向组合：继承 `<|--`（实线 + 空心三角）、实现 `<|..`（虚线 + 空心三角）、组合 `*--`（实线 + 实心菱形）、聚合 `o--`（实线 + 空心菱形）、关联 `-->`、依赖 `..>`、实线连接 `--`、虚线连接 `..`。
  - 支持 lollipop 接口端点 `()--` 与 `--()`。
  - 支持关系两端基数标注（如 `Customer "1" --> "*" Order : places`）及中点关系标签。
- **分组、注释与样式**：
  - `namespace PackageName { ... }` 复用 Sugiyama 嵌套容器绘制分组框与标题。
  - `note "..."`（独立注释）与 `note for ClassName "..."` 参与 Sugiyama 布局防重叠并用虚线指向目标类。
  - 支持 `style`、`classDef`、`cssClass` 与 `:::` 内联类；`link`、`click`、`callback`、`accTitle`、`accDescr` 静默忽略。

### 7.5 `erDiagram` 设计（`er.rs` — 完整对齐版）

- **实体与别名**：
  - 支持仅在关系中隐式出现的实体、带属性块的 `ENTITY { ... }`、显示别名 `ENTITY["Display Name"]`，以及 `direction TB`/`TD`/`LR`/`BT`/`RL` 和 `style`/`classDef`/`:::` 样式覆盖。
- **实体属性表（多列对齐 + 斑马纹）**：
  - 解析 `type name [PK|FK|UK|PK,FK,...] ["comment"]` 行语法。
  - 无属性实体渲染为简洁单框；含属性实体渲染为头部（主题强调底色 + 粗体实体名）+ 多列等宽/测宽对齐表格：`类型`｜`字段名`｜`PK/FK/UK` 键徽章（仅当表内存在键约束时显示此列）｜`注释`（仅当表内存在注释时显示此列），奇数/偶数行配柔和斑马纹底色与水平细分隔线，适配全部 10 套明暗主题。
- **关系与 Crow's Foot 基数**：
  - 支持全部 4 种 Crow's Foot 符号端点：精确一（`||`）、零或一（`|o` / `o|`）、一或多（`}|` / `|{`）、零或多（`}o` / `o{`），支持标识实线 `--`、非标识虚线 `..` 及关系标签 `: label`。
  - 同时兼容 Mermaid 自然语言基数别名（左/右基数 `only one`/`1`、`zero or one`/`one or zero`、`one or more`/`one or many`/`1+`、`zero or more`/`zero or many`/`0+`，搭配 `to` 实线 / `optionally to` 虚线）。

### 7.6 v2 分阶段实施计划与验收标准

1. **Phase 4 — 共享原语扩展 + `stateDiagram-v2`**：
   - 扩展 `preprocess.rs`（`State`/`Class`/`Er` 识别）、`draw.rs`（UML / Crow's Foot 端符、基数标签、`Note` 折角卡片原语）、`layout/sugiyama.rs`（`note` 邻近不重叠布局、并发分区 `--`）。
   - 实现 `state.rs` 及核心解析、复合状态、伪状态与并发分区单元测试。
2. **Phase 5 — `classDiagram`**：
   - 实现 `class.rs`（类与成员词法、泛型与修饰符、三段式类框、8 种 UML 端符 + lollipop + 基数标签、`namespace`、`note`）及全面单元测试。
3. **Phase 6 — `erDiagram` + 快照/画廊/文档与全量验收**：
   - 实现 `er.rs`（实体别名、多列对齐属性表、Crow's Foot 符号与自然语言别名）。
   - 新增 `tests/mermaid.rs` v2 集成测试与 `asset/mermaid_chart/{state,class,er}` JSON/SVG 快照，通过 `tests/image.rs` 生成 `asset/image/mermaid-{state,class,er}.png` 并逐张视觉核查排版、文字间距、主题对比度与防重叠效果。
   - 更新 [docs/gallery.md](./gallery.md)、[docs/guide.md](./guide.md)、[docs/guide-zh.md](./guide-zh.md)、[docs/json.md](./json.md) 与 [docs/json-zh.md](./json-zh.md)，并完成 `cargo fmt --check`、全 feature 矩阵 `cargo test` 与 `cargo clippy --all-features --all-targets --all -- --deny=warnings`。

## 八、v3 封版扩展（mindmap / quadrantChart / radar-beta / treemap-beta）

v3 在**零新增外部依赖**、统一由 Cargo `mermaid` feature 控制的约束下，完成了面向知识树、战略象限、多维雷达与层级矩形树图的 4 类高频图表接入，使 `charts-rs` 支持的 Mermaid 图表种类达到 **13 类**并正式封版（其余低频试验性语法暂不列入支持计划）：

1. **统一缩进预处理（`src/charts/mermaid/preprocess.rs`）**：
   - `SourceLine` 新增 `indent: usize` 字段（空格按 `1`、制表符按 `4` 折算），统一支撑缩进敏感语法解析。
   - `DiagramKind` 新增 `Mindmap`、`Quadrant`、`Radar`、`Treemap` 四个分支，覆盖 `mindmap`、`quadrantChart`、`radar-beta`/`radar` 与 `treemap-beta`/`treemap`。
2. **双侧对称思维导图引擎（`src/charts/mermaid/mindmap.rs`）**：
   - 严格校验单根节点及缩进层级并对多根节点报精确行号错误；完整支持 7 种节点形状（默认圆角块、方框 `[...]`、胶囊 `(...)`、圆形 `((...))`、六边形 `{{...}}`、云朵 `)...(`、爆炸星芒 `))...((`）。
   - 支持 ``"`...`"`` Markdown 文本清理、`::icon(...)` 宽容忽略、全局 `classDef` 与节点内联/下挂 `:::` 样式覆盖。
   - 根节点居中、一级分支按 `div_ceil(2)` 左右均衡分流、按当前主题自动分配分支色彩并绘制平滑三次贝塞尔 S 形连线；画布按内容自适应尺寸，支持 `width`/`height` 等比 `viewBox` 缩放。
3. **纯 SVG 四象限散点引擎（`src/charts/mermaid/quadrant.rs`）**：
   - 支持 `title`、单/双端 `x-axis` / `y-axis`、`quadrant-1..4` 四象限区域主题淡色填充与虚线十字准线、`-90°` 旋转 Y 轴标签（围绕画布偏移基准点精确旋转）。
   - 支持 `classDef` 与点级属性（`color`、`radius`、`stroke-color`、`stroke-width`）、`[0.0, 1.0]` 坐标越界报错以及四方位散点标签贪心避让布局。
4. **图表适配层扩展（`src/charts/mermaid/adapters/radar.rs` & `treemap.rs`）**：
   - `radar-beta` / `radar` 映射至 `RadarChart`：支持多行 `axis`、位置数组与按键字典两种 `curve` 写法、`graticule polygon|circle`、`min`/`max`/`ticks`/`showLegend`。
   - `treemap-beta` / `treemap` 映射至 `TreemapChart`：借助 `SourceLine.indent` 递归构建 `SunburstData` 分组树，支持节/叶 `classDef` 与内联 `:::` 背景色覆盖。
5. **跨期回归与视觉验收**：
   - 新增 `asset/mermaid_chart/{mindmap,quadrant,radar,treemap}{.json,_json.svg}` 快照与 `asset/image/mermaid-{mindmap,quadrant,radar,treemap}.png` 预览图并逐张核验视觉布局；校验 v1/v2 既有 9 组 SVG 快照字节级零回归。
