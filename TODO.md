# TODO

- [ ] 图表中的所有属性均可配置(颜色、宽度、类型、margin等)，定义完整的style struct
- [x] 支持图表的font-weight，每个label都使用其为默认值
- [x] fontdue与fontdb中对于bold等类型字体名称不一致(light, bold等的处理)
- [x] series label format的方法支持更多类型，{a}：系列名。{c}：数据值。{d}：百分比。{t}：千分位格式化。
- [x] 填充曲线图中文本输出顺序调整
- [x] table支持定义每一个格的背景色、字体颜色
- [x] 支持scatter
- [x] 支持多图合并处理
- [x] 支持空值处理(中间值为空值)
- [x] 支持candlestick
- [x] 平均值线、最大最小值标记
- [x] 柱状图指定单个背景色
- [x] json是否兼容null的处理
- [x] label inside bar chart
- [x] series label format的自定义
- [ ] fontdue与fontdb是否可统一，现两个字库重复加载内存占用较大
- [x] table中文本计算宽度，自动换行
- [x] 饼图支持普通形式
- [x] 饼图需要支持最少尺寸(少于1px的场景)

## 对比 ECharts 待补充的图表

基于 1.3.0，对照 [ECharts 示例](https://echarts.apache.org/examples/en/index.html)（24 个 2D 分类，377 个示例）整理。已覆盖其中 19 个分类，未覆盖的为 `map`、`lines`、`pictorialBar`、`matrix`、`custom`。以下按建议的先后顺序排列，参数名仅为建议。

### 低成本：现有图表的变体

- [x] 折线图支持阶梯线（Step Line），series 增加 `step`：`start` / `middle` / `end`
- [x] 柱状图支持百分比堆叠（Stacked Bar Normalization），同一 `stack` 的系列归一化为 100%
- [x] 饼图支持半环（Half Doughnut），增加 `end_angle`（极坐标柱状图已有同名参数）
- [ ] 饼图支持嵌套多环（Nested Pies）
- [x] 坐标轴支持反向，`y_axis_configs` 增加 `axis_inverse`，用于排名图（Bump Chart）

### 新图表：甘特图

- [ ] 甘特图（ECharts 以 custom 系列实现：Gantt Chart of Airport Flights），用于排期、时间线类报表，可复用 1.3.0 的时间轴

### 中等成本

- [ ] 散点图支持回归线（线性、指数、多项式、对数，对应 ECharts 的 4 个 Regression 示例）
- [ ] 误差线（Error Bar / Error Scatter），数据形式可参考折线的区间带 `band`
- [ ] 热力图支持分段配色与多色渐变（Discrete Mapping of Color），现仅有 `min_color`、`max_color` 两色，日历图同理
- [ ] 仪表盘支持分段着色、多指针或多环进度（ECharts 有 12 个仪表盘示例），现多个 series 仅展示第一个

### 新图表：地图

- [ ] 地图（`map` 分类 25 个示例，为未覆盖分类中最多的），以区域着色为主。GeoJSON 由调用方传入，不内置地图数据；需要实现投影，并依赖上面的多色映射

### 可选

- [ ] 树图支持径向布局、`RL` / `BT` 方向以及折线连接，现仅有 `LR`、`TB`
- [ ] 桑基图支持纵向布局
- [ ] 蜡烛图支持 OHLC 样式
- [ ] 单轴散点图（Punch Card）

### 不计划支持

- 3D / GL、航线图（`lines`）、涟漪散点、缩放、刷选、下钻、动态排序：依赖 WebGL 或交互，静态 SVG / PNG 输出用不上
- `matrix`：相关性矩阵、混淆矩阵可用热力图实现，格子内放小图可用 `MultiChart` 拼合
- 象形柱图（`pictorialBar`）：装饰性强，且需要支持自定义图形
- `custom` 中的火焰图、六边形分箱、风向标、圆堆积、极坐标热力图：使用场景少

### 已支持，无需处理

- 堆叠折线、堆叠面积（series 的 `stack`）
- 蜡烛图叠加均线（series 的 `category: "line"`），成交量副图可用 `MultiChart` 拼合
- 置信区间（`band`）、时间轴、气泡图、极坐标柱状图