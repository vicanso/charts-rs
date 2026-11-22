# MCP 就绪实施计划

> 给后续实现者（人或模型）的自包含任务说明。目标：把 charts-rs 部署为 HTTP 服务，通过 MCP 让 AI 以 JSON 调用渲染图表。本文件只覆盖**库本身**需要补的能力和文档；MCP 服务端放在独立 crate / 仓库，参考设计见文末。
>
> **状态：T0–T6 已全部完成**。实现记录：`multi.child_charts` 在 Essential 档若内联全部 29 个子图 schema 会达到约 26KB（远超 Full 档的 5.4KB），因此保留开放对象 + 明确提示先按子图 `type` 获取对应 schema。

## 0. 背景与现状

### 已完成（第 1 阶段：schema 文档化 + 分档）

| 能力 | 位置 |
|---|---|
| `list_chart_types()` / `list_themes()` / `render_chart_from_json()` | [schema.rs](../src/charts/schema.rs)、[theme.rs](../src/charts/theme.rs) |
| `chart_json_schema(t)`（= Full 档） | schema.rs |
| `chart_json_schema_with(t, SchemaLevel::Essential \| Full)` | schema.rs，`essential_paths()` 定义每种图的精简键 |
| `chart_example(t)`：取 json.md 中的示例 | schema.rs |
| json.md 解析：说明、默认值、图表简介、示例 | [schema_doc.rs](../src/charts/schema_doc.rs) |
| `mermaid_capabilities()` / `mermaid_validate()` | [mermaid/mod.rs](../src/charts/mermaid/mod.rs) |
| 测试：每个属性都有 description、默认值可渲染、Essential ⊂ Full、Essential 路径都存在 | [tests/mcp_api.rs](../tests/mcp_api.rs)、schema.rs `every_essential_path_is_a_key` |

实测体积：Full 约 26–31KB，Essential 约 1.6–4KB。

### 关键约定（必须遵守）

1. **[docs/json.md](./json.md) 是唯一事实来源**。schema 里的说明、默认值、示例都由它在运行时生成（`include_str!`）。
   - 增删键：同时改校验表（schema.rs 里的 `*_FIELDS`）、`docs/json.md` 和 `docs/json-zh.md`，否则 `reference_documents_every_key` 会失败。
   - 表格行格式固定为 `` | `key` | type | default | description | ``。默认值单元格必须是**合法 JSON**（字符串要带引号）、`theme`、`auto`，或者留空。
   - 示例必须能渲染：`tests/json_reference.rs` 会检查，`schema_examples_and_defaults_render` 也会检查。
2. **JSON 键永不改名**，`from_json` 的接受范围只能放宽、不能收紧（以 `asset/**/*.json` 和现有测试为准）。
3. 公共 API 只增不改：`chart_json_schema`、`render_chart_from_json` 的签名不变；`Error` 是 `#[non_exhaustive]`，可以加变体，但**不要改变现有校验失败返回 `Error::Params` 的行为**。
4. 新增公共枚举、结构体时考虑加 `#[non_exhaustive]`。

### 测试命令

```bash
cargo test --features mermaid                      # 主测试集
cargo test                                         # 无 feature
cargo test --features mermaid --doc                # 文档测试
cargo clippy --features mermaid --lib --tests -- --deny=warnings
rustfmt --edition 2024 <改动过的文件>             # 仓库里有其它文件存在原有的 fmt 差异，见 T0
```

> [!WARNING]
> `cargo test --all-features` 目前编译失败：[tests/mermaid.rs:511](../tests/mermaid.rs#L511) 用了 `Error` 却没有 import。这是原有问题，由 T0 修复。

---

## T0. 清理原有问题（小）

- [tests/mermaid.rs](../tests/mermaid.rs)：在 `use charts_rs::{...}` 里补上 `Error`。注意这个函数受 `png` feature 门控，要在 `--all-features` 下验证。
- `cargo fmt` 全仓库（`canvas.rs`、`examples/mermaid.rs` 等存在差异），**单独提交**，不要和功能改动混在一起。

**验收**：`cargo test --all-features` 和 `cargo clippy --all-features --all-targets -- --deny=warnings` 通过。

---

## T1. 为无类型数组补元素 schema 和校验（P0）

### 问题

下面这些字段的类型是 `Kind::Array`，schema 只有 `{"type":"array"}`，校验也不检查元素。实测 `calendar` 传 `{"data":[["x","y"]]}` 能通过校验，还生成了 59KB 的 SVG。

| 校验表 | 字段 | json.md 记载的形态 |
|---|---|---|
| `CALENDAR_FIELDS` | `data` | `[["YYYY-MM-DD", number]]` |
| `CALENDAR_FIELDS` | `show_dow_labels` | `integer 0–6` 的数组 |
| `WATERFALL_FIELDS` | `data` | 元素为 `number` 或 `[number, bool]` |
| `MAP_FIELDS` | `data` | `[[string, number]]` |
| `HEATMAP_SERIES_FIELDS` | `data` | `[[index, value]]`，其中 index = y × x 类别数 + x |
| `BOX_SERIES_FIELDS` | `data` | `[[min, q1, median, q3, max]]` |
| `LINES_FIELDS` | `lines` | `{from:[x,y], to:[x,y], value, name}` 或 `{coords:[[x,y],...], value}` |
| `TABLE_FIELDS` | `data` | 行数组；单元格为 `string \| number \| null \| number[]`（sparkline） |
| `CELL_STYLE_FIELDS` | `indexes` | `[row, column]` |
| `SCATTER_FIELDS` | `series_symbols` | 元素为 `string`（circle/triangle/rect/diamond）或 `{type, color}` |
| `MULTI_FIELDS` | `child_charts` | 已有特殊处理，**本任务不动**（见 T3） |

### 做法

1. **先读解析代码，确认真实接受的形态**。每个字段在对应 `*_chart.rs` 的 `from_json` 里，配合 [params.rs](../src/charts/params.rs) 的 `get_*_from_value`。校验必须**与解析器等价或更宽**。例如解析器若接受数字字符串、null、对象形式，校验也要接受。json.md 和解析器不一致时，以解析器为准并修正 json.md（中英文都改）。
2. 在 `Kind` 里增加可组合的变体，例如：
   ```rust
   Kind::Items(&'static Kind)          // 元素都满足某个 Kind 的数组
   Kind::Tuple(&'static [Kind])        // 定长元组 → JSON Schema prefixItems + minItems/maxItems(+items:false)
   Kind::AnyOf(&'static [Kind])        // 多选一
   Kind::Date                          // "YYYY-MM-DD"，可复用 params.rs 的日期解析
   ```
   具体命名可以调整，但要同时实现**校验**（`validate_value`）和 **schema 生成**（`SchemaBuilder::property`），两者从同一个 `Kind` 推导，保证不漂移。
3. 错误信息带上元素路径，例如 ``field `data[0][0]` must be a date such as "2024-01-05"``。
4. json.md 的类型列可以从 `array` 改成更具体的写法（如 `[date, number][]`），中英文同步。

### 验收

- 新增负例测试（放在 schema.rs 的 tests 模块）：每个字段至少 1 个非法输入被拒绝，并检查错误路径。
- 正例：`asset/` 下所有 JSON fixture、json.md 示例、现有集成测试**全部仍然通过**。可以加一个测试，遍历 `asset/**/*.json`，对其中带 `type` 的文档调用 `render_chart_from_json`，确认没有校验回归。
- `every_key_changes_the_chart`、`reference_documents_every_key` 通过。
- `chart_json_schema("calendar")["properties"]["data"]["items"]` 带 `prefixItems`。

---

## T2. 结构化校验结果（P0）

### 目标

模型自我修正时，需要机器可读的错误，并且最好一次拿到全部错误。

### API 设计

```rust
#[derive(Debug, Clone, serde::Serialize)]
#[non_exhaustive]
pub struct ValidationIssue {
    pub code: &'static str,        // "unknown_field" | "invalid_type" | "invalid_enum" | "out_of_range"
                                   // | "invalid_color" | "unknown_theme" | "missing_required" | "invalid_json" | "mermaid_syntax"
    pub path: String,              // "series_list[0].lable_show"，根为 ""
    pub message: String,           // 与现有 Params message 文本一致
    pub expected: Option<String>,  // "a positive number" / "one of left, center, right"
    pub suggestion: Option<String>,// typo 建议的完整路径，如 "series_list[0].label_show"
    pub line: Option<usize>,       // mermaid 用
}

/// 只校验、不渲染；收集全部问题（上限如 50 条，防止输出爆炸）。
pub fn validate_chart_json(chart_type: &str, options: &serde_json::Value) -> Result<Vec<ValidationIssue>>;
// 外层 Err 只用于未知 chart_type；空 Vec 表示通过。

impl Error { pub fn code(&self) -> &'static str }  // "params" | "json" | "font_not_found" | "mermaid" | ...
```

### 做法

- 把 schema.rs 的 `validate_object` / `validate_value` 改成往 `&mut Vec<ValidationIssue>` 里推，而不是 `return Err`。原有的 `validate()` 改为取第一条、包装成 `Error::Params { message }`，**`from_json` 的错误文本保持不变**（现有测试断言了文本）。
- `chart_type == "mermaid"`：先做字段校验，再对 `source` 调用 `mermaid_validate`，把 `Error::Mermaid { line, message }` 转成 issue。
- 主题：`theme` 现在是在 `get_theme` 阶段报 `unknown theme`，要在 validate 阶段也能报出来（code `unknown_theme`，`expected` 列出 `list_themes()`）。注意 `add_theme` 注册的自定义主题也要合法，请用 `list_theme_name()` 判断。
- 顺带修复：未知主题错误里的主题列表来自 HashMap，**顺序不稳定**（[theme.rs](../src/charts/theme.rs) `list_theme_name`），改成排序输出。

### 验收

- 测试：一个文档里有 3 处错误 → 返回 3 条，code/path/suggestion 正确；合法示例（所有 `chart_example`）→ 空 Vec。
- `ValidationIssue` 能 `serde_json::to_value`。
- 现有 `Params` 文本断言全部通过。

---

## T3. 便于服务端调用的入口（P1）

1. **`render_chart_from_value(chart_type, &serde_json::Value) -> Result<String>`**。先用 `to_string` 再调 `render_chart_from_json` 即可，正确性优先；如果要去掉二次解析，可以给 `ChartBase` 的解析入口（[base.rs](../src/charts/base.rs) 约 602 行附近）增加接收 `Value` 的变体，但不强制。
2. **`chart_catalog() -> Vec<ChartInfo>`**，`ChartInfo { name, title, description, example }` 实现 `Serialize`。其中 `description` 取 `schema_doc::reference().intro(name)`，`title` 取 `chart_schema_meta`。它是 MCP `list_chart_types` 工具的返回值，模型据此选图。
3. **multi 子图 schema**：`child_charts.items` 目前是开放对象。提供 `SchemaLevel::Essential` 下按 `type` 区分的 `oneOf`（每个分支 = 对应图的 Essential properties + `type` 的 `const`），Full 档保持现状，避免体积爆炸。实现后检查 Essential 下 multi schema 的大小（目标 < 40KB），超出就放弃 oneOf，只保留 description 提示。

**验收**：`chart_catalog().len() == list_chart_types().len()`，每项都有非空 description 和可渲染的 example；`render_chart_from_value` 与 `render_chart_from_json` 输出一致。

---

## T4. HTTP 部署所需的防护与资源（P1）

### 4.1 资源上限

只靠 `MAX_COUNT` / `MAX_INDEX` 不够。新增：

```rust
#[derive(Clone, Debug)]
#[non_exhaustive]
pub struct RenderLimits {
    pub max_json_bytes: usize,      // 默认如 1 MiB
    pub max_canvas_side: f32,       // width/height 上限，默认如 8192
    pub max_data_points: usize,     // 所有数组元素总数（递归计数），默认如 100_000
    pub max_mermaid_source: usize,  // 默认如 64 KiB
}
pub fn render_chart_from_json_with_limits(chart_type: &str, json: &str, limits: &RenderLimits) -> Result<String>;
```

超限时返回 `Error::Params`，并在 T2 里使用 code `limit_exceeded`。默认的 `render_chart_from_json` 不加限制，保持兼容。

### 4.2 字体

- 现状：`font_family` 写成不存在的字体时会**静默回退**并渲染成功（已实测）。请确认 [font.rs](../src/charts/font.rs) 的回退路径，并在 json.md 的 `font_family` 行写明这一行为。
- 已有 `get_font_families() -> Result<Vec<String>>`（[font.rs](../src/charts/font.rs) 第 536 行附近），确认它已导出，并在文档和 MCP 资源中使用，**不要另起新函数**。下文的 `list_fonts` 都指它。
- 可选：`chart_json_schema_with` 增加一个带字体列表的变体，把 `font_family` 渲染成 `enum`，或者由服务端自行改写 schema。在 llms.txt 中说明这两种做法。
- 在 guide.md 的 Fonts 一节加一段“服务端部署：预加载 CJK 字体，否则中文宽度测量错误”。

### 4.3 地图注册表

模型不应该生成或传输 GeoJSON。

- `register_geo_json(name: &str, geo_json: serde_json::Value) -> Result<()>` 和 `list_geo_json() -> Vec<String>`，用全局 `ArcSwap<HashMap>`，模式与 theme.rs 的注册表一致。
- `map` / `lines` 的 `geo_json` 字段同时接受对象和**字符串名称**：校验 Kind 改为 `AnyOf[Object-any, String]`，字符串必须是已注册的名称（code `unknown_map`）。
- json.md 两种语言同步，补一个按名称引用的示例；名称示例可以复用 `asset/geo/china`，但**仓库资源不进 crate 包**（见 Cargo.toml 的 `exclude`），只在文档中说明服务端如何加载。

**验收**：针对每项的单元测试；`map` 用已注册名称能渲染，用未注册名称报错且 code 正确。

---

## T5. 一致性修正（小）

- **图表数量说法**：README、README-zh、llms.txt、lib.rs、schema 描述里的“29 chart types”包含了 `multi`（组合容器）。统一表述为“28 种图表 + `multi` 组合（共 29 个类型名，启用 `mermaid` 后为 30 个）”。`tests/json_reference.rs` 里的计数注释同步修改。
- **Mermaid**：`MermaidCapabilities.supported_diagrams` 有 20 个名字（含别名），而文档写的是“13 种”。新增字段 `diagram_aliases: &[(&str, &str)]`（别名 → 规范名），或者把 supported 拆成规范名 + 别名。注意：`MermaidCapabilities` 的字段是 pub 的，新增字段对外部用字面量构造的代码是破坏性变更，请先确认该结构体是否 `#[non_exhaustive]`，不是的话就加上，并在 CHANGELOG 中注明。
- `chart_schema_meta` 里 mermaid 的 description（“13 diagram types”）与上面保持一致。

---

## T6. 文档与 llms（在 T1–T5 之后做，依赖 API 定型）

1. **[llms.txt](../llms.txt)**
   - **选图指南表**：数据形态 → 图表类型（时间序列→line、类别对比→bar/horizontal_bar、占比→pie/funnel、分布→histogram/box_plot、相关性→scatter、流向→sankey/chord、层级→treemap/sunburst/tree、网络→graph、OHLC→candlestick、日历→calendar、进度→gauge/bullet、日程→gantt、地理→map/lines、表格→table、流程/时序图→mermaid）。
   - **每种图一行最小 JSON**：由 `chart_example` 或 json.md 示例生成，不要手写。
   - **错误格式一节**：`ValidationIssue` 字段说明，以及推荐的“校验→根据 suggestion 修正→重试”流程。
   - **输出格式建议**：很多 MCP 客户端不显示 SVG；推荐 `format: "svg" | "png"`，PNG 用 MCP `image` content（base64），并需要 `png` feature。
   - 更新 MCP Recipe，工具集与文末参考设计保持一致。
2. **`llms-full.txt`**：由 llms.txt、docs/json.md、docs/guide.md 拼接生成。在 Makefile 里加 `llms` 目标，并加一个测试检查它与源文件同步（例如比较生成结果与文件内容，不一致就提示运行 `make llms`）。
3. **README / README-zh**：在 JSON / MCP 段落里提一句两档 schema 和 `chart_catalog`，保持精简（见 CLAUDE.md 的文档布局约定）。
4. **[CLAUDE.md](../CLAUDE.md)**：Key Modules 表加入 `schema.rs` / `schema_doc.rs`，并写明“json.md 驱动 schema 的说明、默认值和示例”。
5. **[CHANGELOG.md](../CHANGELOG.md)**：仓库使用 git-cliff（`cliff.toml`），确认是否手写；如果是生成的，就用规范的 commit message（`feat(schema): ...`）代替手改。

---

## 附：MCP 服务端参考设计（独立 crate）

| 工具 | 输入 | 实现 |
|---|---|---|
| `list_chart_types` | — | `chart_catalog()` |
| `get_chart_schema` | `chart_type`, `level?: "essential"\|"full"`（默认 essential） | `chart_json_schema_with` |
| `validate_chart` | `chart_type`, `options: object` | `validate_chart_json` |
| `render_chart` | `chart_type`, `options: object`, `format?: "svg"\|"png"`（默认 png） | `render_chart_from_value` + `RenderLimits` + `svg_to_png` |
| `validate_mermaid` | `source` | `mermaid_validate` |
| `render_mermaid` | `source`, `theme?`, `allow_links?`（默认 false） | `MermaidChart::new_with_theme` |

**Resources**：`charts://themes`（`list_themes`）、`charts://fonts`（`list_fonts`）、`charts://maps`（`list_geo_json`）、`charts://mermaid/capabilities`（`mermaid_capabilities`）、`charts://examples/{type}`（`chart_example`）。

**部署要点**：
- 启动时预加载 CJK 字体和地图；
- 所有渲染走 `RenderLimits`；
- 不可信输入的 mermaid 一律 `allow_links = false`；
- 渲染是同步 CPU 任务，在 async 服务里放进 `spawn_blocking` 并设置超时。
