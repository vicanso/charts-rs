# Mermaid 渲染美化实施方案

> 本文档是 Mermaid 渲染视觉优化的**可执行实施方案**，供后续开发者（或其他模型）按阶段独立完成。
> 设计背景见 [mermaid-design.md](./mermaid-design.md)，历史问题清单见 [mermaid-issues.md](./mermaid-issues.md)。
>
> - 制定日期：2026-11-24
> - 基线：`v2.4.0`（`8e22ccb`）
> - 实施状态（2026-11-24）：六个阶段全部完成——① `c576db6` 配色与样式、② `e9543d2` 端口 + 正交路由、③ `e93aa40` Brandes-Köpf、④ `c556662` median + transpose、⑤ `3240674` M-07 子图方向、⑥ `3b9b8c8` sequence / gantt / mindmap / quadrant，并已完成 UI 检视收口优化与文档同步。未做：sequence 参与者头部压缩、gantt 任务名放进条内（10.2 第 3 项，可选）、正交轨道过多时自动扩大层间距。
> - 视觉参考：[lukilabs/beautiful-mermaid](https://github.com/lukilabs/beautiful-mermaid)（ELK 正交路由 + 双色主题）。**目标不是复刻 Mermaid 官方外观**，而是做出 charts-rs 自己的、整洁的风格。

## 目录

- [一、背景与现状诊断](#一背景与现状诊断)
- [二、目标与非目标](#二目标与非目标)
- [三、已确定的设计决策](#三已确定的设计决策)
- [四、阶段总览](#四阶段总览)
- [五、阶段 ①：配色与基础样式](#五阶段-配色与基础样式)
- [六、阶段 ②：端口 + 正交路由](#六阶段-端口--正交路由)
- [七、阶段 ③：Brandes-Köpf 坐标分配](#七阶段-brandes-köpf-坐标分配)
- [八、阶段 ④：median + transpose 交叉优化](#八阶段-median--transpose-交叉优化)
- [九、阶段 ⑤：M-07 子图独立 direction](#九阶段-m-07-子图独立-direction)
- [十、阶段 ⑥：sequence / gantt / mindmap / quadrant](#十阶段-sequence--gantt--mindmap--quadrant)
- [十一、通用验收流程](#十一通用验收流程)
- [十二、风险与注意事项](#十二风险与注意事项)

---

## 一、背景与现状诊断

当前 13 种 Mermaid 图已经能正确渲染，但效果图不够美观。下面的诊断来自逐张检查 `asset/image/mermaid-*.png`：

| # | 问题 | 典型表现（示例图） | 根因（代码位置） |
|---|---|---|---|
| D1 | 连线斜、乱拐 | `mermaid-flowchart.png`：`Yes`/`No` 是斜线，`Hit` 长边呈波浪形；`mermaid-class.png`：关系线是长斜线；`mermaid-state.png`：`invalid` 长边扭曲 | `src/charts/mermaid/draw.rs` 中 `build_smooth_edge_path` 是默认实现，用 Catmull-Rom 穿过每个 dummy 点；端点由 `clip_point_to_shape` 按「朝对方中心的射线」裁剪，所以出入角度随意 |
| D2 | 布局不平衡 | flowchart 的 `401 Unauthorized` 偏位、右侧大片留白；class 的 `PaymentService` 被挤到子图外的左下 | `src/charts/mermaid/layout/sugiyama.rs` Stage 4（约 L473–L640）用迭代 barycenter 加非重叠推挤分配 X 坐标，长链不会被拉直，父子也不居中 |
| D3 | 交叉偏多 | 复杂图中存在可避免的交叉 | Stage 3（约 L336）只做 barycenter sweep，没有 transpose |
| D4 | 视觉偏重 | 节点描边粗、填充浓；子图标题居中、字号大；箭头偏大 | `src/charts/mermaid/style.rs` 中 `MermaidPalette::from_theme` 直接用 accent 混合，比例偏高 |
| D5 | 子图 direction 无效 | 见 [M-07](./mermaid-issues.md#m-07-flowchart-子图内-direction-被忽略) | 布局只有一个全局方向 |
| D6 | sequence 框重叠 | `mermaid-sequence.png`：`alt` 片段框压在 `box Edge` 分组边框上 | `sequence.rs` 计算片段框宽度时没有避开 participant box 的边界 |
| D7 | gantt 图例不一致 | `mermaid-gantt.png`：图例 `Planning` 是蓝圈，任务条却是灰色（done）；图例图标是折线图样式（线 + 圈） | `adapters/gantt.rs` 把 done 任务改成了灰色，但图例仍取分类色；`GanttChart` 沿用通用 legend 图标 |

---

## 二、目标与非目标

**目标**

1. 关系类图（flowchart / class / state / ER）默认输出**正交折线 + 小圆角拐弯**的连线，节点对齐整齐，交叉更少。
2. 所有原生 Mermaid 引擎（上述四种 + sequence / mindmap / quadrant）统一使用新的**双色基础配色**和扁平样式。
3. 修复 D5、D6、D7。
4. 整个过程**不引入新的 crate 依赖**，`mermaid` feature 继续保持零额外依赖。
5. 输出保持**确定性**：同样的输入必须得到逐字节相同的 SVG。

**非目标**

- 不追求和 mermaid.live 的输出像素一致。
- **不改** pie / xychart / radar / treemap / sankey 适配图：它们复用主库图表，改动会波及主库风格。
- 不新增 Mermaid 专用的内置主题集（Tokyo Night 等），可以以后再做。
- 不做扇入 / 扇出的边捆绑（edge bundling）。

---

## 三、已确定的设计决策

| # | 主题 | 决策 |
|---|---|---|
| 1 | 审美标准 | charts-rs 自有风格，参考 beautiful-mermaid |
| 2 | 默认连线 | **正交折线 + 小圆角拐弯**（圆角半径 6px，线段太短时自动缩小） |
| 3 | 连接点 | **端口式**：边从节点沿布局方向的那条边出入（TB：下边出、上边入；LR：右边出、左边入；BT/RL 镜像）。同一条边上有多条边时**均匀分布**，并按对端的横向位置排序，避免在端口处交叉 |
| 4 | 坐标分配 | 在 `sugiyama.rs` 里**自研 Brandes-Köpf**，取代迭代 barycenter |
| 5 | 交叉优化 | barycenter 换成 **median + transpose** |
| 6 | M-07 | 同期实现子图独立 `direction` |
| 7 | 配色 | 仍从 `Theme` 推导，但改成**双色基础**：只从 `bg` 和 `fg` 按比例混合出全部 UI 色，`accent` 可选 |
| 8 | 视觉细节 | 扁平、无阴影；节点描边 1px；节点圆角 6、子图圆角 8；小号实心三角箭头；边标签是带背景的圆角 pill；子图标题左对齐、字号更小并加粗；子图填充更淡 |
| 9 | 非 Sugiyama 图 | sequence：新样式 + 片段框避开 box + 参与者头部更紧凑；gantt：图例和任务条颜色一致、图例改方块、任务名可以放进条内；mindmap / quadrant：只换配色 |
| 10 | `curve` 兼容 | 不写 `curve` 时默认正交；写 `basis` 等平滑类时走平滑曲线，但**也改成端口垂直出入**；`linear` 保持现有行为；`step` 并入正交 |
| 11 | 推进方式 | 分 6 个阶段，**每个阶段单独提交**，每次都重新生成 gallery PNG 并做前后对比 |

---

## 四、阶段总览

```mermaid
flowchart LR
  P1["① 配色与样式"] --> P2["② 端口 + 正交路由"]
  P2 --> P3["③ Brandes-Köpf"]
  P3 --> P4["④ median + transpose"]
  P4 --> P5["⑤ M-07 子图 direction"]
  P5 --> P6["⑥ sequence / gantt / mindmap / quadrant"]
```

| 阶段 | 主要文件 | 依赖 | 预估改动量 |
|---|---|---|---|
| ① | `style.rs`、`draw.rs`、四个引擎的子图 / 标签绘制部分 | 无 | 中 |
| ② | `draw.rs`、`layout/mod.rs`、`layout/sugiyama.rs`（路由部分）、`flowchart.rs`、`state.rs`、`class.rs`、`er.rs`、`preprocess.rs` | ① 可选 | 大 |
| ③ | `layout/sugiyama.rs` Stage 4 | ② | 大 |
| ④ | `layout/sugiyama.rs` Stage 3 | 无，可以和 ③ 交换顺序 | 中 |
| ⑤ | `layout/sugiyama.rs`、`layout/mod.rs`、`flowchart.rs`、`preprocess.rs` | ③ | 大 |
| ⑥ | `sequence.rs`、`adapters/gantt.rs`、`mindmap.rs`、`quadrant.rs` | ① | 中 |

> [!IMPORTANT]
> 阶段 ② 和 ③ 是收益最大的两步。如果资源有限，至少完成 ①②③。

---

## 五、阶段 ①：配色与基础样式

### 5.1 双色调色板

改写 `src/charts/mermaid/style.rs` 中的 `MermaidPalette::from_theme`。字段名尽量保持不变（下游大量引用），需要的话再新增字段。

推导规则（`mix(a, b, t)` 即现有的 `mix_color`，t 表示 b 的占比）：

| 字段 | 亮色主题 | 暗色主题 | 说明 |
|---|---|---|---|
| `bg` / `opaque_bg` | 沿用现有逻辑 | 同左 | |
| `fg`（新增） | `theme.title.font.color`，无效时取 `(32,37,46)` | 无效时取 `(235,240,248)` | 即现有的 `text_color` |
| `text_color` | `fg` | `fg` | |
| `node_fill` | `mix(bg, fg, 0.04)`，或有 accent 时 `mix(bg, accent, 0.08)` | `mix(bg, fg, 0.07)`，或 `mix(bg, accent, 0.14)` | 比现在明显更淡 |
| `node_stroke` | `mix(bg, fg, 0.38)`，或 `mix(bg, accent, 0.55)` | `mix(bg, fg, 0.40)`，或 `mix(bg, accent, 0.60)` | |
| `edge_stroke` | `mix(bg, fg, 0.55)` | `mix(bg, fg, 0.55)` | 不再依赖 `x_axis.stroke_color` |
| `edge_label_bg` | `bg`（不透明） | `bg` | pill 背景 |
| `edge_label_stroke`（新增） | `mix(bg, fg, 0.12)` | `mix(bg, fg, 0.18)` | pill 的细边 |
| `muted_text`（新增） | `mix(bg, fg, 0.60)` | `mix(bg, fg, 0.62)` | 子图标题、ER 类型列、cardinality |
| `subgraph_stroke` | `mix(bg, fg, 0.18)` | `mix(bg, fg, 0.22)` | |
| `subgraph_fill(depth)` | `mix(bg, fg, 0.02 + 0.015·depth)` | `mix(bg, fg, 0.03 + 0.02·depth)` | depth 上限为 4 |
| `note_*` / `activation_fill` | 保留黄色系，比例下调约 30% | 同左 | |
| `accent` | `theme.series.colors` 第一个有效色 | 同左 | 只用于节点着色，**不用于线条** |

> [!NOTE]
> 是否用 accent 给节点上色，用常量 `USE_ACCENT_NODES: bool = true` 控制，便于对比。最终效果以 gallery 对比为准。

### 5.2 统一视觉常量

在 `draw.rs`（或新建 `style.rs` 常量区）里集中定义，替换各引擎中散落的魔法数：

```rust
pub(crate) const NODE_STROKE_WIDTH: f32 = 1.0;
pub(crate) const EDGE_STROKE_WIDTH: f32 = 1.25;   // 粗边（==>）为 2.0
pub(crate) const NODE_RADIUS: f32 = 6.0;          // 圆角矩形
pub(crate) const SUBGRAPH_RADIUS: f32 = 8.0;
pub(crate) const ARROW_LEN: f32 = 7.0;
pub(crate) const ARROW_HALF_WIDTH: f32 = 3.5;
pub(crate) const LABEL_PILL_PAD_X: f32 = 6.0;
pub(crate) const LABEL_PILL_PAD_Y: f32 = 2.0;
pub(crate) const LABEL_PILL_RADIUS: f32 = 4.0;
pub(crate) const SUBGRAPH_TITLE_FONT_SCALE: f32 = 0.85; // 相对于正文字号
```

### 5.3 具体改动

1. **`draw_edge_marker`**：普通箭头改为实心三角，尺寸用上面的常量；class / ER 的特殊标记（菱形、空心三角、鸦脚）按比例缩小，保持 1px 描边。
2. **`draw_edge_label`**：画成带 `edge_label_bg` 填充和 `edge_label_stroke` 细边的圆角矩形（pill），文字颜色用 `text_color`。
3. **子图**：标题改为**左对齐**（左内边距 10px），字号 × `SUBGRAPH_TITLE_FONT_SCALE`，`font-weight="600"`，颜色用 `muted_text`；圆角 `SUBGRAPH_RADIUS`；去掉标题下方的分隔线（state 的 concurrent region 分隔虚线保留）。涉及 `flowchart.rs`、`state.rs`、`class.rs` 中绘制子图 / 复合状态的代码。
4. **节点**：描边宽度统一为 `NODE_STROKE_WIDTH`；不加阴影 / filter。
5. **ER / class 表格**：表头填充 `mix(bg, fg, 0.06)`，行分隔线用 `subgraph_stroke`，类型列文字用 `muted_text`。
6. `classDef` / `style` 内联样式**优先级不变**，仍然覆盖调色板。

### 5.4 验收

- 所有引擎在 `THEME_LIGHT` / `THEME_DARK` 及其余 8 个内置主题下都清晰可读（重点检查暗色下的子图层级）。
- 原有测试全部通过（需要先用 `UPDATE_SNAPSHOTS=1` 更新快照）。

---

## 六、阶段 ②：端口 + 正交路由

### 6.1 数据结构调整（`layout/mod.rs`）

```rust
/// 边在节点上的连接侧（屏幕坐标系）。
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum PortSide { Top, Bottom, Left, Right }

#[derive(Clone, Debug)]
pub(crate) struct RoutedEdge {
    /// 正交模式：完整折线，首尾点就是端口点（已位于节点边框上）。
    /// 平滑 / linear 模式：与现在一致的控制点序列。
    pub(crate) waypoints: Vec<Point>,
    pub(crate) self_loop: bool,
    pub(crate) source_side: PortSide,   // 新增
    pub(crate) target_side: PortSide,   // 新增
    /// 标签中心建议位置（正交模式下位于最长的横段 / 竖段中点）。
    pub(crate) label_pos: Option<Point>, // 新增
}
```

`LayeredGraph` 新增 `routing: EdgeRouting { Orthogonal, Spline, Polyline }`，由各引擎根据 `preprocess::EdgeCurve` 映射后传入：

| `EdgeCurve` | `EdgeRouting` |
|---|---|
| 未配置（新默认） | `Orthogonal` |
| `Step` | `Orthogonal` |
| `Smooth`（basis / cardinal / monotone） | `Spline` |
| `Linear` | `Polyline` |

> [!IMPORTANT]
> `preprocess.rs` 中 `EdgeCurve` 的 `#[default]` 目前是 `Smooth`。要区分「未配置」和「显式 basis」，`flowchart_curve: Option<EdgeCurve>` 为 `None` 时应映射到 `Orthogonal`，**不要**直接用 `EdgeCurve::default()`。state / class / ER 没有 curve 配置，固定用 `Orthogonal`。

### 6.2 端口分配（sugiyama.rs 坐标分配之后、路由之前）

在 canonical TB 坐标系里做（最后统一变换到屏幕方向）：

1. 对每个真实节点 `v`，收集**出边**（指向下一层）和**入边**（来自上一层）。每条边按它在相邻层的「对端 item 的 x 坐标」排序，dummy 链取链上第一个 dummy 的 x。
2. 出边分到 `Bottom`，入边分到 `Top`。共有 `k` 条边时，端口 x 坐标为

   `x_i = cx - usable/2 + usable * (i + 0.5) / k`，其中 `usable = min(w - 2*NODE_RADIUS, k * PORT_GAP)`，`PORT_GAP = 12`。

   `k == 1` 时端口就在边中点。
3. **非矩形形状**（菱形、圆、六边形等）：只允许 1 个居中端口，所有边汇聚到这一点（菱形的顶点 / 圆的最低点）。可以给 `NodeShape` 加一个 `fn supports_spread_ports(&self) -> bool`。
4. **反向边**（break_cycles 反转过的边）：在 canonical 坐标里它也是向下走，所以端口照常分配，绘制时再把箭头放在逻辑终点（现有逻辑已经处理了反转）。
5. **自环**：从节点右侧（TB）出、绕一个 `16×(h/2)` 的矩形环回到右侧偏上的位置，四个拐角都是圆角。

### 6.3 正交路由

对一条 canonical 边，其 item 链为 `p0 (源端口) → d1 → d2 → … → pn (目标端口)`：

1. **相邻层之间的段**：从上方的点 `(x_a, y_a_bottom)` 到下方的点 `(x_b, y_b_top)`：
   - `|x_a - x_b| < 1.0`：直接竖线。
   - 否则走三段：`(x_a, y_a) → (x_a, y_mid) → (x_b, y_mid) → (x_b, y_b)`。`y_mid` 是两层之间间隙的「轨道」y 坐标。
2. **dummy 节点**直接作为竖直贯穿点（dummy 宽度为 0），有了 B-K 后长边大多是直线。
3. **轨道分配（避免横段重叠）**：同一层间隙里的所有横段区间 `[min(x_a,x_b), max(x_a,x_b)]` 按左端点排序，用**区间图贪心着色**分配到 `t` 条轨道上，轨道 y 坐标在间隙内均匀分布：
   `y_track_j = gap_top + gap_h * (j + 1) / (t + 1)`。
   着色时让「向右走」和「向左走」的边按端点顺序排，减少同一间隙内的交叉（经典做法：先排目标 x 小于源 x 的边）。
   如果 `t` 太大导致轨道间距小于 6px，就在 Stage 4 里**扩大这一层间隙**（需要把轨道数反馈给 rank 间距计算；可以先算一次轨道数，再调整 `rank_gap[r]`）。
4. **子图边框**：横段不能穿过子图标题区。如果横段的 y 落在某个子图的标题条里，就把它挪到标题条下方，或者挪到子图外侧的间隙。最简单的做法是：rank 间隙计算时已经为子图边界留了额外空间（现有逻辑在 L499 附近），轨道只放在「去掉子图 padding 之后」的纯间隙里。
5. **圆角**：在 `draw.rs` 新增

   ```rust
   pub(crate) fn build_orthogonal_edge_path(pts: &[Point], radius: f32)
       -> (String, Point, (f32, f32), (f32, f32))
   ```

   对每个拐点，用 `Q` 二次贝塞尔（或 `A` 圆弧）替代直角，半径取 `min(radius, 相邻两段长度/2)`。返回值格式和现有 `build_*_edge_path` 一致（path d、标签中点、起点方向、终点方向）。
6. **标签位置**：取折线中最长的一段的中点；如果这段是横段，标签垂直居中压在线上（pill 背景会盖住线）。如果 `has_label` 的边位于没有横段的竖直直线上，标签放在竖段中点。
7. **端点方向**：箭头方向由最后一段决定，正交模式下一定是轴对齐的，箭头会更整齐。

### 6.4 平滑模式（`Spline`）的调整

保留 Catmull-Rom，但首尾各插入一个「沿端口法线方向延伸 `min(12, gap/3)`」的控制点，让曲线垂直离开 / 进入节点；端点使用 6.2 的端口坐标，不再用射线裁剪。

### 6.5 各引擎的接入

- `flowchart.rs`（约 L1393 调 layout，L1597 绘制边）、`state.rs`（L1035 / L1289）、`class.rs`（L953 / L1101）、`er.rs`（L588 / L685）：
  - 传入 `routing`；
  - 绘制时如果 `routing == Orthogonal`，**跳过** `clip_point_to_shape` / `clip_point_to_rect`，直接用 `waypoints` 的首尾点；
  - 标签位置改用 `RoutedEdge::label_pos`。
- ER 的 cardinality 标记（鸦脚）需要轴对齐的端点方向，正交模式下天然满足。`er.rs` 当前直接调用 `build_smooth_edge_path`，要改成按 routing 分派。
- class 的平行多重边（M-05 的 `parallel` 分组，sugiyama.rs L1027–L1100 的 bow 逻辑）：正交模式下改为**端口错开 + 各占一条轨道**，不再弯曲。
- state 的 start / end 小圆点、fork / join 条、choice 菱形都只用单个居中端口。

### 6.6 验收

- `mermaid-flowchart.png`：`Yes` / `No` 变成横平竖直的折线；`Hit` 是一条干净的折线。
- `mermaid-class.png` / `mermaid-er.png` / `mermaid-state.png`：所有边都是轴对齐的线段加圆角。
- 显式 `curve: basis` 时仍是曲线，并且垂直出入节点；`curve: linear`（`examples/mermaid.rs` 使用）的行为不变。
- 新增单元测试：
  - 正交模式下所有 waypoints 的相邻点满足 `dx == 0 || dy == 0`（容差 0.01）；
  - 同一间隙内任意两条横段，在 y 相等时 x 区间不重叠；
  - 400 节点压力用例（mermaid-issues.md 第九节）耗时不超过现在的 2 倍。

---

## 七、阶段 ③：Brandes-Köpf 坐标分配

替换 `sugiyama.rs` Stage 4 中「Iterative barycenter X alignment」（约 L585–L640）。

### 7.1 算法要点

参考：Brandes & Köpf, *Fast and Simple Horizontal Coordinate Assignment*（2001），以及 dagre 的 `position/bk.js`（读算法即可，不要拷代码）。

1. **标记 type-1 冲突**：内段（dummy–dummy 边）和非内段交叉时，标记非内段，对齐时忽略它，保证长边优先拉直。
2. **四个方向的垂直对齐**：`(up | down) × (left | right)`。每个方向按 median 邻居对齐，得到 `root[]` / `align[]`。
3. **水平压缩**：按 block 做 `place_block`，最小间距 = `(w_u + w_v)/2 + node_sep`。这里的 `node_sep` 沿用现有的节点间距计算（含 `node_sep_extra` 和标签宽度）。
4. **平衡**：把四组坐标对齐到宽度最小的那组，然后对每个节点取 4 个值中**中间两个的平均**。
5. 全程只用 `Vec` 和索引，迭代顺序固定，保证确定性。

### 7.2 与现有约束的整合

现有 Stage 4 后半段还有这些后处理，**必须保留并在 B-K 之后执行**：

- concurrent region 兄弟子图的分离（L645 起）；
- 非成员节点被推出根子图的横向范围（L716 起）；
- 屏幕空间的碰撞消解（L933 起）；
- 附着 note 紧贴目标节点（L429 的排序约束，在 B-K 中把 note 和目标视为同一 block 的相邻项，或者 B-K 之后再修正）。

**子图连续性**：B-K 本身不知道子图。推荐做法：

- 在 Stage 3 的排序结果里，同一根子图的成员在每层上已经是连续的（现有初始排序会按根子图分组，④ 中要保持这一点）；
- `place_block` 时，相邻两项分属不同根子图（或一项在子图内、一项在子图外）的，最小间距额外加 `SUBGRAPH_PAD * 2`，这样后处理的推挤会大幅减少。

### 7.3 验收

- `mermaid-flowchart.png`：主链 `Client → API Gateway → JWT Valid?` 在同一条水平线上；`401 Unauthorized` 和它的源节点对齐。
- `mermaid-class.png`：`PaymentService` 不再孤立在左下角。
- 单元测试：一条长度 ≥ 4 的单链，所有节点的 cx 相等；对称二叉树里父节点位于两个子节点的中点（容差 0.5）。
- 已有测试 `handles_cycles_and_subgraphs_deterministically` 继续通过。

---

## 八、阶段 ④：median + transpose 交叉优化

替换 Stage 3「Crossing reduction via barycenter sweeps」（约 L336–L428）。

1. **median 值**：取相邻层邻居位置的中位数；邻居数为偶数时用 dagre / Gansner 的加权中位数；没有邻居的节点保持原位（固定）。
2. **sweep**：上下交替，最多 24 轮。每轮之后计算总交叉数（用 Barth–Jünger–Mutzel 累加树，O(E log V)），保留最优的排序；连续 4 轮没有改善就提前退出。
3. **transpose**：每轮 sweep 之后，对每层相邻的两项尝试交换，如果交叉数严格减少就交换，重复到不再改善为止。
4. **约束**（和现在一致，必须保持）：
   - 同一根子图的成员在每层上连续：先在「子图块」层面排序，再在块内排序（或者对跨块交换直接禁止）；
   - concurrent region 分组；
   - 附着 note 紧贴目标（L429 逻辑保留在排序之后）。
5. **确定性**：所有比较的 tie-break 一律使用 item id。

**验收**：新增交叉数单元测试（构造一个已知最优交叉数的小图），并且所有 gallery 图的交叉数不增加。可以临时加一个 debug 统计函数，在 PR 描述里贴出前后对比。

---

## 九、阶段 ⑤：M-07 子图独立 direction

现象和复现见 [M-07](./mermaid-issues.md#m-07-flowchart-子图内-direction-被忽略)。

### 9.1 方案：子图作为复合节点递归布局

1. `preprocess.rs` / `flowchart.rs`：解析子图内的 `direction XX`，存到子图元数据里；`LayeredSubgraphInput` 新增 `direction: Option<LayoutDirection>`。
2. `sugiyama::layout` 入口：
   - 找出所有「direction 与父级不同」的子图 `S`（Mermaid 的语义是：**只有当子图没有和外部节点直接相连的边**时，子图方向才生效；有外部边时回退到父方向。这一点要和 Mermaid 行为对齐，详见 Mermaid 文档 flowchart「Direction in subgraphs」一节）；
   - 对每个这样的 `S`，自底向上：取出 `S` 的成员和内部边，**递归调用 `layout`**（direction = S 的方向），得到内部布局和包围盒；
   - 在外层图里，用一个尺寸等于该包围盒（加上标题和 padding）的**代理节点**替代 `S`；
   - 指向 `S` 内节点的外部边改为指向代理节点（端口取代理节点边界上最靠近内部目标的位置）；
   - 外层布局完成后，把内部布局平移到代理节点的位置，合并 nodes / edges / subgraphs 的结果。
3. 外部边进入代理节点后，从代理边框到内部目标节点的「最后一段」用正交路由补上。

### 9.2 验收

- M-07 的复现用例：子图内 `a → b → c` 横向排列，外层仍是 TB。
- 嵌套两层、方向各不相同的用例渲染正确，没有重叠。
- 新增 gallery 图 `mermaid-flowchart-subgraph-direction.png`，并加到 `docs/gallery.md`。
- 完成后把 [mermaid-issues.md](./mermaid-issues.md) 中 M-07 的状态更新为「已完成」并填上提交号。

---

## 十、阶段 ⑥：sequence / gantt / mindmap / quadrant

### 10.1 sequence（`sequence.rs`）

1. 接入阶段 ① 的调色板和常量：参与者框圆角 6，描边 1px；消息箭头用实心小三角；消息文字不加 pill（保持在线上方）。
2. **片段框避开 box**（D6）：计算片段框（L1174 附近传播外边距的逻辑）的左右边界时，如果片段覆盖的参与者跨越了 `box` 分组的边界，就让片段框的边界**落在 box 边框外侧至少 8px**，或者完全落在 box 内侧至少 8px，绝不能和 box 边框重合或只差几个像素。实现上，可以给每个 box 的左右边界加入「片段边界禁区」，再对片段的 x 坐标做吸附。
3. **参与者头部更紧凑**：actor（小人）图标和下方文字的总高度压缩约 20%；普通 participant 框的上下 padding 从现值降到 8px。
4. 片段标签（`alt`、`loop` 的五边形标签）改用 `muted_text` 和 `mix(bg, fg, 0.06)` 填充。

### 10.2 gantt（`adapters/gantt.rs`，必要时改 `gantt_chart.rs`）

1. **图例和任务条颜色一致**（D7）：done 任务目前是灰色，而图例取分类色。两种方案任选其一，推荐 A：
   - A：done 任务保留分类色，只降低不透明度（约 0.45）并加 1px 同色描边，这样图例颜色天然一致；
   - B：图例颜色改为「该分类在图中出现最多的那种状态色」。
2. **图例图标改方块**：`GanttChart` 当前使用默认的 `LegendCategory::Normal`（线 + 圈）。主库 `src/charts/component.rs` 已有 `LegendCategory::RoundRect` / `Rect`。在 Mermaid 适配器里把 gantt 的 legend category 设为 `RoundRect`。如果 `GanttChart` 还没有暴露这个设置，就新增一个字段（默认仍是 `Normal`），确保主库 `GanttChart` 的默认外观不变。
3. **任务名放进条内（可选，复杂度高可推迟）**：新增适配器级开关，默认关闭。开启后，如果文字宽度 + 2×8px ≤ 条宽，就在条内画任务名，文字颜色按条颜色的亮度在白色和 `fg` 之间选择；y 轴标签保留不变。

### 10.3 mindmap / quadrant

只接入阶段 ① 的调色板和常量（描边宽度、圆角、文字颜色），**布局和结构不动**。mindmap 的分支彩色保持不变（来自 series colors）。

### 10.4 验收

- `mermaid-sequence.png`、`mermaid-sequence-lifecycle.png`：片段框和 box 边框不再重叠。
- `mermaid-gantt.png`：图例颜色和任务条颜色一致，图例是方块。
- 主库 `GanttChart` 自身的快照（`tests/gantt_chart.rs`）**不变**。

---

## 十一、通用验收流程

每个阶段完成后都执行：

```bash
# 1. 格式与静态检查
cargo fmt --all
cargo clippy --features "mermaid,image-encoder" --all-targets -- -D warnings

# 2. 测试（先确认失败的只有快照，再更新）
cargo test --features "mermaid,image-encoder"
UPDATE_SNAPSHOTS=1 cargo test --features "mermaid,image-encoder"

# 3. 不开 feature 也要能编译
cargo build --no-default-features

# 4. 检查变更
git diff --stat asset/
```

- `tests/image.rs::generate_mermaid_image` 会在 `UPDATE_SNAPSHOTS=1` 时重新生成 `asset/image/mermaid-*.png`。
- **逐张对比** `asset/image/mermaid-*.png` 的前后效果。做法：改动前把 PNG 复制到临时目录，改动后并排查看。PR 描述里附上关键图的前后对比。
- 检查确定性：同一输入渲染两次，SVG 逐字节相同（现有测试已覆盖，不要破坏）。
- 健壮性：mermaid-issues.md 第九节的异常输入不得出现 panic、`NaN` 或 `inf`。
- 每个阶段一个提交，信息格式参考仓库已有风格，例如 `feat(mermaid): orthogonal edge routing with port assignment`。

---

## 十二、风险与注意事项

| 风险 | 说明 | 缓解 |
|---|---|---|
| 快照大面积变化 | 每个阶段都会改动几乎所有 Mermaid SVG 快照 | 每个阶段单独提交，快照更新和代码放在同一个提交里，方便 review |
| 子图约束与 B-K 冲突 | B-K 不感知子图，后处理推挤可能破坏对齐 | 按 7.2 在 `place_block` 中加入子图间距；后处理只在确实重叠时才推挤 |
| 正交路由穿过节点 | 跨多层的边理论上由 dummy 保证不穿过节点，但子图的后处理推挤之后可能出现 | 路由放在**所有推挤之后**执行；新增测试检查折线和非端点节点的包围盒不相交 |
| 轨道过多导致层间距暴涨 | 扇出很大的节点（一对多 20 条） | 轨道间距下限 6px，层间距的扩张设上限（例如 3 倍）；超过上限时允许横段共线（同向同端点合并） |
| 性能 | median 加交叉计数、transpose 比 barycenter 慢 | 设置轮数上限和提前退出条件；400 节点压力用例耗时不超过 2 倍 |
| 主库波及 | gantt legend 图标 | 只通过新增选项在适配器里启用，主库默认不变 |
| `MermaidCapabilities` / 文档 | `curve` 的语义变化 | 更新 `docs/guide.md`、`docs/guide-zh.md`、`docs/json*.md` 中关于 `flowchart.curve` 的说明，并执行 `make llms` 重新生成 `llms-full.txt` |
