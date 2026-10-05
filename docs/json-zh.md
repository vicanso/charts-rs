# JSON 参数

[English](./json.md)

每种图表都可以通过各自的 `from_json` 从一份 JSON 文档创建，例如 `BarChart::from_json(json)`。本页列出这类文档可以使用的全部键。

- 省略的键，或值为 `null` 的键，保持默认值。
- 文档在使用前会先校验：未知的键（疑似拼写错误时会给出提示）、类型不对的值、未知的枚举值或超出范围的尺寸都会报错，而不是被悄悄忽略。
- 枚举值不区分大小写。
- 默认值为“主题”表示取自所用主题。
- 键名与图表结构体的字段名一致，因此 [API 文档](https://docs.rs/charts-rs)对 Builder API 给出了同样的说明。

## 目录

- [值类型](#值类型)
- [通用参数](#通用参数)
- 图表：[Bar](#bar)、[Horizontal bar](#horizontal-bar)、[Line](#line)、[Pie](#pie)、[Radar](#radar)、[Scatter](#scatter)、[Candlestick](#candlestick)、[Table](#table)、[Heatmap](#heatmap)、[Funnel](#funnel)、[Waterfall](#waterfall)、[Calendar](#calendar)、[Gauge](#gauge)、[Treemap](#treemap)、[Box plot](#box-plot)、[Sunburst](#sunburst)、[Sankey](#sankey)、[Tree](#tree)、[Graph](#graph)、[Parallel](#parallel)、[Theme river](#theme-river)、[Histogram](#histogram)、[Polar bar](#polar-bar)、[Chord](#chord)、[Gantt](#gantt)、[Map](#map)、[Multi chart](#multi-chart)
- [供工具使用的键](#供工具使用的键)

## 值类型

| 类型 | 写法 |
|---|---|
| 数字 | 任意 JSON 数字。 |
| 整数 | 非负整数。刻度数、分段数最大 1000，序号最大 1 000 000。 |
| 颜色 | `"#rgb"`、`"#rgba"`、`"#rrggbb"`、`"#rrggbbaa"`、`"rgb(r, g, b)"`、`"rgba(r, g, b, a)"` 或 `"transparent"`。 |
| 边距 | 一个数字（四边相同），或 `{"left": 5, "top": 5, "right": 5, "bottom": 5}`（省略的边为 `0`）。 |
| 刻度类型 | `"linear"`、`"log"`（以 10 为底）、`"log2"`、`"log10"`，或 `{"type": "log", "base": 5}`。 |
| x 值 | 数字，或字符串形式的日期：`"2024-03-01"`、`"2024-03-01 08:30"`、`"2024-03-01T08:30:00+08:00"`。时间轴上的数字是 unix 秒。 |
| 数字数组 | 数字组成的数组；`null` 表示缺失值。 |
| 颜色数组 | 颜色组成的数组；`null` 表示该位置保持默认。 |

## 通用参数

除表格和组合图外，所有图表都接受这些键。每种图表只使用对它有意义的部分：例如饼图没有坐标轴，轴相关的键对它不起作用。

<!-- keys: base -->

### 画布

| 键 | 类型 | 默认值 | 说明 |
|---|---|---|---|
| `theme` | 字符串 | `"light"` | 主题名称：`light`、`dark`、`ant`、`vintage`、`shine`、`walden`、`westeros`、`chalk`、`grafana`、`shadcn`，或通过 `add_theme` 注册的主题。未知名称会报错。 |
| `width` | 正数 | `600` | 图表宽度（像素）。 |
| `height` | 正数 | `400` | 图表高度（像素）。 |
| `x` | 数字 | `0` | 图表在 SVG 中的水平偏移。 |
| `y` | 数字 | `0` | 图表在 SVG 中的垂直偏移。 |
| `margin` | 边距 | `5` | 图表四周的外边距。 |
| `font_family` | 字符串 | `"Roboto"` | 所有文字使用的字体。字体需要先加载（见 README 的“加载更多字体”），文字宽度才能准确测量。 |
| `compact` | 布尔 | `false` | 输出精简的 SVG：画面相同，体积通常小 20–30%。 |
| `empty_text` | 字符串 |  | 没有数据可画时显示在绘图区中央的文字。 |

### 标题

| 键 | 类型 | 默认值 | 说明 |
|---|---|---|---|
| `title_text` | 字符串 |  | 标题文字；为空时不绘制标题。 |
| `title_font_size` | 数字 | `18` | 标题字号。 |
| `title_font_color` | 颜色 | 主题 | 标题字体颜色。 |
| `title_font_weight` | 字符串 | `"bold"` | 标题字重，例如 `"bold"`。 |
| `title_margin` | 边距 |  | 标题四周的边距。 |
| `title_align` | `"left"` / `"center"` / `"right"` | `"center"` | 标题的水平对齐方式。 |
| `title_height` | 数字 | `30` | 标题行占用的高度。 |

### 副标题

| 键 | 类型 | 默认值 | 说明 |
|---|---|---|---|
| `sub_title_text` | 字符串 |  | 副标题文字，显示在标题下方。 |
| `sub_title_font_size` | 数字 | `14` | 副标题字号。 |
| `sub_title_font_color` | 颜色 | 主题 | 副标题字体颜色。 |
| `sub_title_font_weight` | 字符串 |  | 副标题字重，例如 `"bold"`。 |
| `sub_title_margin` | 边距 |  | 副标题四周的边距。 |
| `sub_title_align` | `"left"` / `"center"` / `"right"` | `"center"` | 副标题的水平对齐方式。 |
| `sub_title_height` | 数字 | `20` | 副标题行占用的高度。 |

### 图例

| 键 | 类型 | 默认值 | 说明 |
|---|---|---|---|
| `legend_show` | 布尔 |  | 是否显示图例。默认显示，饼图、主题河流图和瀑布图默认隐藏。 |
| `legend_position` | `"top"` / `"bottom"` / `"left"` / `"right"` | `"top"` | 图例位置：顶部标题旁（默认）、底部，或在绘图区左侧 / 右侧纵向排列。 |
| `legend_align` | `"left"` / `"center"` / `"right"` | `"center"` | 图例的水平对齐方式。 |
| `legend_category` | `"normal"` / `"rect"` / `"round_rect"` / `"circle"` | `"normal"` | 图例标记的形状：带圆点的线段（`normal`）、矩形、圆角矩形或圆形。 |
| `legend_font_size` | 数字 | `14` | 图例字号。 |
| `legend_font_color` | 颜色 | 主题 | 图例字体颜色。 |
| `legend_font_weight` | 字符串 |  | 图例字重，例如 `"bold"`。 |
| `legend_margin` | 边距 |  | 图例四周的边距。 |

### X 轴

| 键 | 类型 | 默认值 | 说明 |
|---|---|---|---|
| `x_axis_data` | 字符串数组 |  | x 轴标签，即图表的类目。 |
| `x_axis_type` | `"category"` / `"value"` / `"time"` | `"category"` | 连续 x 轴上的值如何解读：普通数值（`value`）或时间戳（`time`）。只要给出 x 值，x 轴就是连续轴；日期字符串会自动选用 `time`。 |
| `x_axis_values` | x 值数组 |  | 每个数据点的 x 值，所有系列共用。设置后 x 轴变为连续轴：点按其数值定位，而不是等距排列（折线图、柱状图）。 |
| `x_axis_min` | x 值 |  | 连续 x 轴的固定起点；默认取第一个值。直方图中为第一个分箱的起点。 |
| `x_axis_max` | x 值 |  | 连续 x 轴的固定终点；默认取最后一个值。直方图中为最后一个分箱的终点。 |
| `x_axis_formatter` | 字符串 |  | x 轴标签的格式：`{c}` 代表标签本身；时间轴可使用由 `%Y %y %m %d %H %M %S %b` 组成的模式，例如 `"%m-%d %H:%M"`。 |
| `x_axis_time_offset` | 数字 | `0` | 时间轴显示所用的时区，以相对 UTC 的分钟数表示（东八区为 480）。 |
| `x_axis_title` | 字符串 |  | x 轴标题，显示在轴标签下方。 |
| `x_axis_hidden` | 布尔 | `false` | 隐藏 x 轴。极坐标柱状图中隐藏类目标签。 |
| `x_axis_height` | 数字 | `30` | x 轴占用的高度。 |
| `x_axis_stroke_color` | 颜色 | 主题 | x 轴轴线和刻度线的颜色。 |
| `x_axis_stroke_width` | 数字 | `1` | x 轴轴线和刻度线的宽度。 |
| `x_axis_font_size` | 数字 | `14` | x 轴标签字号。 |
| `x_axis_font_color` | 颜色 | 主题 | x 轴标签字体颜色。 |
| `x_axis_font_weight` | 字符串 |  | x 轴标签字重，例如 `"bold"`。 |
| `x_axis_name_gap` | 数字 | `5` | x 轴轴线与标签之间的间距。 |
| `x_axis_name_rotate` | 数字 | `0` | x 轴标签的旋转角度（弧度），例如 `0.785` 即 45°。 |
| `x_axis_margin` | 边距 |  | x 轴四周的边距。 |
| `x_boundary_gap` | 布尔 | `true` | `true` 时 x 轴两端留白，数据点位于类目中间；`false` 时首尾数据点落在轴的两端。 |
| `x_axis_label_overflow` | `"thin"` / `"rotate"` / `"ellipsis"` | `"thin"` | x 轴标签放不下时的处理方式：间隔省略（`thin`）、旋转（`rotate`）或截断加省略号（`ellipsis`）。 |

### Y 轴

| 键 | 类型 | 默认值 | 说明 |
|---|---|---|---|
| `y_axis_hidden` | 布尔 | `false` | 隐藏 y 轴。极坐标柱状图中隐藏数值标签。 |
| `y_axis_configs` | 对象数组 |  | 数值轴配置：第一项是左侧 y 轴，第二项是右侧 y 轴。 |

### 网格

| 键 | 类型 | 默认值 | 说明 |
|---|---|---|---|
| `grid_stroke_color` | 颜色 | 主题 | 网格线颜色。 |
| `grid_stroke_width` | 数字 | `1` | 网格线宽度。 |
| `grid_stroke_dash_array` | 字符串 |  | 网格线的虚线样式，写法同 SVG 的 `stroke-dasharray`（例如 `"4,2"`）。默认为实线。作用于带 x、y 轴的图表，甘特图、雷达图和极坐标柱状图的网格，以及单轴散点图（punch card）的行线和平行坐标图的轴线。 |

### 系列

| 键 | 类型 | 默认值 | 说明 |
|---|---|---|---|
| `series_list` | 对象数组 |  | 图表的数据系列。 |
| `series_colors` | 颜色数组 | 主题 | 调色板：每个系列按顺序取色，用完后循环。 |
| `series_stroke_width` | 数字 | `2` | 系列线条的宽度。 |
| `series_symbol` | 对象 |  | 所有折线数据点上的标记；设为 `null` 则不绘制。默认：半径等于 `series_stroke_width`、以背景色填充的圆。 |
| `series_smooth` | 布尔 | `false` | 折线系列绘制为平滑曲线。 |
| `series_fill` | 布尔 | `false` | 填充折线系列下方的区域。 |
| `series_fill_opacity` | 数字 |  | 面积填充的不透明度，0 到 1。默认折线下方为 `0.39`，雷达图为 `0.2`。 |
| `series_label_formatter` | 字符串 |  | 数据标签的格式：`{c}` 数值、`{a}` 系列名、`{b}` 类目名、`{d}` 百分比、`{t}` 千位格式的数值。 |
| `series_label_font_size` | 数字 | `14` | 数据标签字号。 |
| `series_label_font_color` | 颜色 | 主题 | 数据标签字体颜色。 |
| `series_label_font_weight` | 字符串 |  | 数据标签字重，例如 `"bold"`。 |
| `series_label_hide_overlap` | 布尔 | `false` | 隐藏会与已绘制标签重叠的数据标签。 |
| `stack_percent` | 布尔 | `false` | 把同一堆叠内的系列显示为各自的占比：每个堆叠合计 100%，数值轴以百分比显示（柱状图、条形图、折线图、极坐标柱状图）。 |

### 提示与动画

| 键 | 类型 | 默认值 | 说明 |
|---|---|---|---|
| `tooltip_show` | 布尔 | `false` | 为每个数据图形添加悬停提示（纯 CSS 实现，无需脚本），同时写入供辅助工具使用的 `<title>`。日历图、仪表盘、平行坐标图、雷达图和主题河流图不支持。 |
| `tooltip_font_size` | 数字 |  | 悬停提示的字号。默认与数据标签相同（`series_label_font_size`）；散点图默认不写字号，由查看器决定。 |
| `tooltip_font_color` | 颜色 |  | 悬停提示的字体颜色。默认与数据标签相同（`series_label_font_color`）；散点图默认不写颜色，由查看器决定。 |
| `tooltip_font_weight` | 字符串 |  | 悬停提示的字重，例如 `"bold"`。默认不设置。 |
| `animation` | 对象 |  | 图表出现时的动画；`{}` 表示使用默认值。支持柱状图、条形图、折线图、饼图、漏斗图、旭日图、矩形树图、桑基图、直方图、极坐标柱状图与和弦图。 |

### `series_list[]`

<!-- keys: base.series_list -->

| 键 | 类型 | 默认值 | 说明 |
|---|---|---|---|
| `name` | 字符串 |  | 系列名称，显示在图例和提示中。 |
| `data` | 数字数组 |  | 系列的数值，每个类目一个；`null` 表示缺失点。部分图表的读取方式不同，见各图表小节。 |
| `index` | 非负整数 |  | 在调色板中的位置；默认取系列的序号。 |
| `y_axis_index` | 非负整数 | `0` | 系列所属的 y 轴：`0`（左）或 `1`（右）。 |
| `label_show` | 布尔 | `false` | 在每个数据点上显示数值标签。 |
| `category` | `"line"` / `"bar"` |  | 柱状图中把该系列绘制为折线（`line`）而不是柱。 |
| `start_index` | 非负整数 | `0` | 第一个数值对应的类目序号。 |
| `mark_lines` | 对象数组 |  | 横跨绘图区的标记线，位于系列的统计值或固定值处。 |
| `mark_points` | 对象数组 |  | 标记系列的最小点或最大点。 |
| `mark_areas` | 对象数组 |  | 横跨绘图区、介于两个值之间的阴影带。 |
| `colors` | 颜色数组 |  | 每个数据点各自的颜色（柱状图、条形图、极坐标柱状图）；`null` 保持系列颜色。 |
| `stroke_dash_array` | 字符串 |  | 折线系列的虚线样式，例如 `"4 2"`。 |
| `stack` | 字符串 |  | 堆叠分组名：名称相同（且 y 轴相同）的系列会堆叠。 |
| `smooth` | 布尔 |  | 覆盖图表级的 `series_smooth`。 |
| `fill` | 布尔 |  | 覆盖图表级的 `series_fill`。 |
| `symbol` | 对象 |  | 覆盖图表级的 `series_symbol`；`null` 表示不绘制标记。 |
| `x_values` | x 值数组 |  | 该系列每个数据点的 x 值，用于采样位置与图表 `x_axis_values` 不同的系列。 |
| `band` | 对象 |  | 折线周围的填充区间带：每个数据点一个下界和一个上界。 |
| `step` | `"start"` / `"middle"` / `"end"` |  | 把折线绘制为阶梯线：两点之间保持水平，在当前点（`start`）、两点中间（`middle`）或下一个点（`end`）处变为下一个值。优先于 `smooth`。 |
| `error_bar` | 对象 |  | 误差线：每个数据点一个下界和一个上界，在柱或数据点上绘制为两端带短横的竖线（柱状图、折线图、散点图）。y 轴范围会自动包含上下界，提示中会显示区间。 |
| `ring` | 非负整数 | `0` | 饼图中该系列所属的环。同一环的系列共享一个圆，数值越小越靠内——即嵌套饼图。 |

#### `series_list[].mark_lines[]`

<!-- keys: base.series_list.mark_lines -->

| 键 | 类型 | 默认值 | 说明 |
|---|---|---|---|
| `category` | `"average"` / `"min"` / `"max"` / `"value"` |  | 标记线的位置：系列的平均值、最小值、最大值，或由 `value` 指定。 |
| `value` | 数字 |  | `category` 为 `value` 时标记线所在的数值。 |
| `color` | 颜色 |  | 标记线及其圆点、箭头的颜色。默认为系列颜色。 |
| `stroke_width` | 数字 | `1` | 标记线的宽度。 |
| `stroke_dash_array` | 字符串 | `"4,2"` | 标记线的虚线样式，写法同 SVG 的 `stroke-dasharray`；`""` 表示实线。 |

#### `series_list[].mark_points[]`

<!-- keys: base.series_list.mark_points -->

| 键 | 类型 | 默认值 | 说明 |
|---|---|---|---|
| `category` | `"min"` / `"max"` |  | 被标记的点。 |

#### `series_list[].mark_areas[]`

<!-- keys: base.series_list.mark_areas -->

| 键 | 类型 | 默认值 | 说明 |
|---|---|---|---|
| `from` | 数字 / `"average"` / `"min"` / `"max"` |  | 阴影带的一条边。 |
| `to` | 数字 / `"average"` / `"min"` / `"max"` |  | 阴影带的另一条边。 |
| `color` | 颜色 |  | 阴影带的颜色。默认为系列颜色。 |
| `opacity` | 数字 | `0.16` | 阴影带的不透明度，0 到 1。未设置时，如果 `color` 自带透明度（如 `"#ff000080"`）则沿用它。 |

#### `series_list[].symbol` 与 `series_symbol`

<!-- keys: base.series_list.symbol -->
<!-- keys: base.series_symbol = base.series_list.symbol -->

| 键 | 类型 | 默认值 | 说明 |
|---|---|---|---|
| `type` | `"circle"` / `"rect"` / `"square"` / `"triangle"` / `"diamond"` |  | 标记的形状（`square` 与 `rect` 相同）。 |
| `size` | 数字 |  | 标记的大小：半径，或边长的一半。 |
| `radius` | 数字 |  | 与 `size` 含义相同；两者都设置时以 `size` 为准。 |
| `color` | 颜色 |  | 标记的填充颜色；不设置时标记为空心。 |

#### `series_list[].band`

<!-- keys: base.series_list.band -->

| 键 | 类型 | 默认值 | 说明 |
|---|---|---|---|
| `lower` | 数字数组 |  | 每个点的下界；`null` 处区间带断开。 |
| `upper` | 数字数组 |  | 每个点的上界；`null` 处区间带断开。 |

#### `series_list[].error_bar`

<!-- keys: base.series_list.error_bar -->

与区间带同样的两个列表：上下界都给出的数据点才会画出误差线。

| 键 | 类型 | 默认值 | 说明 |
|---|---|---|---|
| `lower` | 数字数组 |  | 每个点误差线的下端；`null` 表示该点没有误差线。 |
| `upper` | 数字数组 |  | 每个点误差线的上端；`null` 表示该点没有误差线。 |
| `stroke_width` | 数字 | `1.5` | 误差线的宽度。 |

### `y_axis_configs[]`

<!-- keys: base.y_axis_configs -->

| 键 | 类型 | 默认值 | 说明 |
|---|---|---|---|
| `axis_font_size` | 数字 | `14` | 轴标签字号。 |
| `axis_font_color` | 颜色 | 主题 | 轴标签字体颜色。 |
| `axis_font_weight` | 字符串 |  | 轴标签字重，例如 `"bold"`。 |
| `axis_stroke_color` | 颜色 | `transparent` | 轴线颜色；默认透明。 |
| `axis_stroke_width` | 数字 | `1` | 轴线和刻度线的宽度。 |
| `axis_width` | 数字 |  | 轴占用的宽度；默认按标签所需宽度计算。 |
| `axis_split_number` | 0–1000 的整数 | `6` | 数值范围被分成的段数。 |
| `axis_name_gap` | 数字 | `8` | 轴线与标签之间的间距。 |
| `axis_formatter` | 字符串 |  | 轴标签的格式：`{c}` 数值、`{t}` 千位格式的数值，例如 `"{c} ms"`。 |
| `axis_margin` | 边距 |  | 轴四周的边距。 |
| `axis_min` | 数字 |  | 轴的固定下界；默认由数据推导。 |
| `axis_max` | 数字 |  | 轴的固定上界；默认由数据推导。 |
| `axis_scale` | 刻度类型 | `"linear"` | 轴的刻度类型：线性或对数。 |
| `axis_title` | 字符串 |  | 轴标题，沿轴方向显示。 |
| `axis_inverse` | 布尔 | `false` | 把轴上下颠倒：最小值在顶部，最大值在底部（用于排名，1 为最好的名次）。 |

### `animation`

<!-- keys: base.animation -->

| 键 | 类型 | 默认值 | 说明 |
|---|---|---|---|
| `duration` | 非负整数 | `1000` | 动画时长（毫秒）。 |
| `easing` | 字符串 | `"ease"` | CSS 缓动函数：`ease`、`linear`、`ease-in`、`ease-out`、`ease-in-out`。 |
| `delay` | 非负整数 | `80` | 相邻元素（柱、系列、扇区、层级）之间的延迟（毫秒）。 |

## Bar

`BarChart::from_json` — 竖直柱状图，`x_axis_data` 的每个类目一组柱。设置 `"category": "line"` 的系列绘制为折线；`stack` 相同的系列堆叠；`y_axis_index` 把系列绑定到右侧 y 轴。

<!-- keys: bar -->

| 键 | 类型 | 默认值 | 说明 |
|---|---|---|---|
| `radius` | 数字 |  | 柱的圆角半径。 |
| `series_label_position` | `"top"` / `"inside"` | `"top"` | 柱的数值标签位置：柱末端上方，或柱的中间（适合堆叠柱，开启 `stack_percent` 时默认使用）。 |

<!-- example: bar -->

```json
{
  "title_text": "Visits",
  "radius": 3,
  "x_axis_data": ["Mon", "Tue", "Wed"],
  "series_list": [
    {"name": "Email", "data": [120, 132, 101], "label_show": true},
    {"name": "Search", "data": [220, 182, 191], "category": "line"}
  ]
}
```

## Horizontal bar

`HorizontalBarChart::from_json` — 水平条形图：`x_axis_data` 的类目沿左侧排列，数值沿底部展开。

<!-- keys: horizontal_bar -->

| 键 | 类型 | 默认值 | 说明 |
|---|---|---|---|
| `series_label_position` | `"left"` / `"right"` / `"top"` / `"bottom"` / `"inside"` | `"right"` | 数值标签的位置；默认在条的右侧。 |

<!-- example: horizontal_bar -->

```json
{
  "x_axis_data": ["Brazil", "India", "China"],
  "series_label_position": "inside",
  "series_list": [
    {"name": "2023", "data": [182, 234, 290], "label_show": true}
  ]
}
```

## Line

`LineChart::from_json` — 折线图，x 轴为 `x_axis_data` 的类目，或在给出 `x_axis_values` 时为连续轴。它没有专属参数：平滑曲线、阶梯线、面积填充、标记、标记线和区间带都属于[通用参数](#通用参数)。

<!-- example: line -->

```json
{
  "series_smooth": true,
  "x_axis_data": ["Mon", "Tue", "Wed", "Thu"],
  "series_list": [
    {
      "name": "Email",
      "data": [120, 132, null, 134],
      "mark_lines": [
        {"category": "average"}
      ]
    },
    {"name": "Search", "data": [220, 182, 191, 234], "fill": true}
  ]
}
```

## Pie

`PieChart::from_json` — 饼图、环形图或南丁格尔玫瑰图。每个系列是一个扇区，数值为其 `data` 之和。`ring` 不同的系列构成嵌套饼图：每个环在自己的扇区之间分配角度，各环平分 `inner_radius` 与 `radius` 之间的空间，内环扇区的名称写在扇区上。

<!-- keys: pie -->

| 键 | 类型 | 默认值 | 说明 |
|---|---|---|---|
| `radius` | 数字 | `150` | 饼图外半径（像素）。 |
| `inner_radius` | 数字 | `40` | 内半径；大于 `0` 时为环形图。 |
| `rose_type` | 布尔 | `true` | 绘制南丁格尔玫瑰图：各扇区角度相同，半径随数值变化。`false` 为普通饼图。 |
| `border_radius` | 数字 |  | 扇区的圆角半径。 |
| `start_angle` | 数字 | `0` | 第一个扇区的起始角度，从 12 点方向顺时针计（度）。 |
| `end_angle` | 数字 |  | 最后一个扇区的终止角度：各扇区平分两个角度之间的圆弧（`-90` 到 `90` 是上半圆，即半环形图）。默认为 `start_angle` 之后一整圈。 |
| `series_label_position` | `"inside"` / `"outside"` | `"outside"` | 标签位置：扇区内部，或带引导线的外部（默认）。 |
| `min_show_label_angle` | 数字 | `0` | 角度小于该值的扇区不显示标签。 |
| `ring_gap` | 数字 | `6` | 嵌套饼图（设置了 `ring` 的系列）相邻两环之间的间隙，单位为像素。 |

<!-- example: pie -->

```json
{
  "rose_type": false,
  "radius": 120,
  "inner_radius": 60,
  "series_list": [
    {"name": "Email", "data": [40]},
    {"name": "Search", "data": [38]},
    {"name": "Direct", "data": [32]}
  ]
}
```

## Radar

`RadarChart::from_json` — 每个系列是覆盖在各 `indicators` 上的一个多边形，每个指标对应一个值。

<!-- keys: radar -->

| 键 | 类型 | 默认值 | 说明 |
|---|---|---|---|
| `indicators` | 对象数组 |  | 雷达图的各个指标轴；每个系列在每个指标上有一个值。 |
| `split_number` | 0–1000 的整数 | `5` | 网状图的圈数；`0` 表示 5。 |

### `indicators[]`

<!-- keys: radar.indicators -->

| 键 | 类型 | 默认值 | 说明 |
|---|---|---|---|
| `name` | 字符串 |  | 指标名称。 |
| `max` | 数字 |  | 轴外端对应的数值；数据中的最大值更大时取数据最大值。 |

<!-- example: radar -->

```json
{
  "indicators": [
    {"name": "Sales", "max": 6500},
    {"name": "Marketing", "max": 16000},
    {"name": "Support", "max": 30000}
  ],
  "series_list": [
    {"name": "Budget", "data": [4200, 3000, 20000]},
    {"name": "Spending", "data": [5000, 14000, 28000]}
  ]
}
```

## Scatter

`ScatterChart::from_json` — 两条数值轴上的散点。系列的 `data` 是扁平的 `x, y` 数对列表（开启 `bubble` 时为 `x, y, size` 三元组）。`x_boundary_gap` 不生效：点始终按刻度定位。

<!-- keys: scatter -->

| 键 | 类型 | 默认值 | 说明 |
|---|---|---|---|
| `series_symbol_sizes` | 数字数组 |  | 各系列标记的半径。 |
| `bubble` | 布尔 | `false` | 气泡图：系列数据为 `[x, y, size]` 三元组，标记面积随 size 变化。 |
| `bubble_min_size` | 数字 | `4` | 最小气泡的半径。 |
| `bubble_max_size` | 数字 | `30` | 最大气泡的半径。 |
| `series_symbols` | 数组 |  | 各系列的标记：用字符串写出形状（`"circle"`、`"triangle"`、`"rect"`、`"diamond"`），或写成带 `type` 和填充色 `color` 的对象。大小由 `series_symbol_sizes` 决定。默认各系列依次使用这四种形状，颜色为系列颜色。 |
| `x_axis_config` | 对象 |  | x 轴配置（散点图的 x 轴是数值轴）。默认与 `y_axis_configs` 的第一项相同。 |
| `regression` | `"linear"` / `"exponential"` / `"logarithmic"` / `"polynomial"` |  | 为每个系列绘制最贴合其数据点的该类型曲线（最小二乘拟合），颜色与系列相同。指数曲线只拟合 y 大于 0 的点，对数曲线只拟合 x 大于 0 的点。 |
| `regression_order` | 0–1000 的整数 | `2` | `polynomial` 回归的阶数，1 到 6。 |
| `regression_label_show` | 布尔 | `false` | 在每条拟合曲线的末端写出公式，例如 `y = 1.5x + 2`。 |

### `x_axis_config`

<!-- keys: scatter.x_axis_config = base.y_axis_configs -->

与 [`y_axis_configs[]`](#y_axis_configs) 的键相同。其中决定数值的键会生效——`axis_min`、`axis_max`、`axis_split_number`、`axis_formatter`；x 轴的字体、颜色和线宽仍由 `x_axis_*` 参数设置。

<!-- example: scatter -->

```json
{
  "series_symbol_sizes": [6, 6],
  "series_list": [
    {"name": "Female", "data": [161.2, 51.6, 167.5, 59, 159.5, 49.2]},
    {"name": "Male", "data": [174, 65.6, 175.3, 71.8, 193.5, 80.7]}
  ]
}
```

## Candlestick

`CandlestickChart::from_json` — 每个类目一根蜡烛。系列的 `data` 是扁平列表，每个类目四个值：开盘、收盘、最低、最高。

<!-- keys: candlestick -->

| 键 | 类型 | 默认值 | 说明 |
|---|---|---|---|
| `candlestick_up_color` | 颜色 | `#EC0000` | 收盘价高于开盘价的蜡烛的填充色。 |
| `candlestick_up_border_color` | 颜色 | `#8A0000` | 上述蜡烛的边框色。 |
| `candlestick_down_color` | 颜色 | `#00DA3C` | 收盘价低于开盘价的蜡烛的填充色。 |
| `candlestick_down_border_color` | 颜色 | `#008F28` | 上述蜡烛的边框色。 |
| `candlestick_style` | `"candle"` / `"ohlc"` | `"candle"` | 绘制蜡烛，或 OHLC 线：一条从最低价到最高价的竖线，左侧短横表示开盘价，右侧短横表示收盘价。 |

<!-- example: candlestick -->

```json
{
  "x_axis_data": ["2024-10-24", "2024-10-25", "2024-10-28"],
  "series_list": [
    {"name": "MA5", "data": [20, 34, 10, 38, 40, 35, 30, 50, 31, 38, 33, 44]}
  ]
}
```

## Table

`TableChart::from_json` — 表格。它不使用通用参数：下面列出的就是它接受的全部键。

<!-- keys: table -->

| 键 | 类型 | 默认值 | 说明 |
|---|---|---|---|
| `theme` | 字符串 | `"light"` | 主题名称：`light`、`dark`、`ant`、`vintage`、`shine`、`walden`、`westeros`、`chalk`、`grafana`、`shadcn`，或通过 `add_theme` 注册的主题。未知名称会报错。 |
| `width` | 正数 | `600` | 图表宽度（像素）。 |
| `height` | 正数 |  | 不生效：表格高度由各行高度决定。 |
| `x` | 数字 | `0` | 图表在 SVG 中的水平偏移。 |
| `y` | 数字 | `0` | 图表在 SVG 中的垂直偏移。 |
| `font_family` | 字符串 | `"Roboto"` | 所有文字使用的字体。字体需要先加载（见 README 的“加载更多字体”），文字宽度才能准确测量。 |
| `background_color` | 颜色 | 主题 | 标题区域的背景色；表头和表体各行分别使用 `header_background_color` 与 `body_background_colors`。 |
| `title_text` | 字符串 |  | 标题文字；为空时不绘制标题。 |
| `title_font_size` | 数字 | `18` | 标题字号。 |
| `title_font_color` | 颜色 | 主题 | 标题字体颜色。 |
| `title_font_weight` | 字符串 | `"bold"` | 标题字重，例如 `"bold"`。 |
| `title_margin` | 边距 |  | 标题四周的边距。 |
| `title_align` | `"left"` / `"center"` / `"right"` | `"center"` | 标题的水平对齐方式。 |
| `title_height` | 数字 | `45` | 标题行占用的高度。 |
| `sub_title_text` | 字符串 |  | 副标题文字，显示在标题下方。 |
| `sub_title_font_size` | 数字 | `14` | 副标题字号。 |
| `sub_title_font_color` | 颜色 | 主题 | 副标题字体颜色。 |
| `sub_title_font_weight` | 字符串 |  | 副标题字重，例如 `"bold"`。 |
| `sub_title_margin` | 边距 |  | 副标题四周的边距。 |
| `sub_title_align` | `"left"` / `"center"` / `"right"` | `"center"` | 副标题的水平对齐方式。 |
| `sub_title_height` | 数字 | `20` | 副标题行占用的高度。 |
| `data` | 数组 |  | 表格内容，每行是一个字符串数组；第一行为表头。 |
| `spans` | 数字数组 |  | 各列宽度：小于 1 的值表示占表格宽度的比例，更大的值表示像素；其余列平分剩余宽度。 |
| `text_aligns` | 字符串数组 |  | 各列文字的对齐方式：`"left"`、`"center"` 或 `"right"`。 |
| `border_color` | 颜色 | 主题 | 行间分隔线的颜色。 |
| `border_width` | 数字 | `1` | 行间分隔线及外边框的宽度。 |
| `header_row_padding` | 边距 | `{left: 10, top: 8, right: 10, bottom: 8}` | 表头行的内边距。 |
| `header_row_height` | 数字 | `30` | 表头行的最小高度。 |
| `header_font_size` | 数字 | `14` | 表头字号。 |
| `header_font_color` | 颜色 | 主题 | 表头字体颜色。 |
| `header_font_weight` | 字符串 |  | 表头字重，例如 `"bold"`。 |
| `header_background_color` | 颜色 | 主题 | 表头背景色。 |
| `body_row_padding` | 边距 | `{left: 10, top: 5, right: 10, bottom: 5}` | 表体各行的内边距。 |
| `body_row_height` | 数字 | `30` | 表体各行的最小高度。 |
| `body_font_size` | 数字 | `14` | 表体字号。 |
| `body_font_color` | 颜色 | 主题 | 表体字体颜色。 |
| `body_font_weight` | 字符串 |  | 表体字重，例如 `"bold"`。 |
| `body_background_colors` | 颜色数组 | 主题 | 表体各行的背景色，依次循环使用。 |
| `outlined` | 布尔 | `false` | 绘制表格外边框。 |
| `cell_styles` | 对象数组 |  | 单个单元格的样式。 |
| `compact` | 布尔 | `false` | 输出精简的 SVG：画面相同，体积通常小 20–30%。 |

### `cell_styles[]`

<!-- keys: table.cell_styles -->

| 键 | 类型 | 默认值 | 说明 |
|---|---|---|---|
| `indexes` | 数组 |  | 单元格位置 `[行, 列]`，从 0 开始计数，表头为第 0 行。 |
| `font_color` | 颜色 |  | 单元格字体颜色。 |
| `font_weight` | 字符串 |  | 单元格字重。 |
| `background_color` | 颜色 |  | 单元格背景色。 |

<!-- example: table -->

```json
{
  "title_text": "NASDAQ",
  "data": [
    ["Name", "Price", "Change"],
    ["Datadog Inc", "97.32", "-7.49%"],
    ["Hashicorp Inc", "28.66", "-9.25%"]
  ],
  "text_aligns": ["left", "center", "right"],
  "body_font_weight": "bold",
  "cell_styles": [
    {"indexes": [1, 2], "font_color": "#fff", "background_color": "#2d7c2b"}
  ]
}
```

## Heatmap

`HeatmapChart::from_json` — 由 `x_axis_data` 与 `y_axis_data` 的类目构成的网格，按数值着色。数据写在 `series` 中，而不是 `series_list`。

<!-- keys: heatmap -->

| 键 | 类型 | 默认值 | 说明 |
|---|---|---|---|
| `y_axis_data` | 字符串数组 |  | y 轴标签：自下而上的类目。 |
| `series` | 对象 |  | 单元格数据及数值到颜色的映射。 |

### `series`

<!-- keys: heatmap.series -->

| 键 | 类型 | 默认值 | 说明 |
|---|---|---|---|
| `min` | 数字 | `0` | 对应 `min_color` 的数值。 |
| `max` | 数字 | `0` | 对应 `max_color` 的数值；`0` 表示取数据中的最大值。 |
| `min_color` | 颜色 | `#F0D99C` | 最小值的颜色。 |
| `max_color` | 颜色 | `#BF444C` | 最大值的颜色。 |
| `min_font_color` | 颜色 | `#464646` | 低数值单元格上标签的字体颜色；使用 `colors`、`steps` 或 `thresholds` 时用于浅色单元格。 |
| `max_font_color` | 颜色 | `#EEEEEE` | 高数值单元格上标签的字体颜色；使用 `colors`、`steps` 或 `thresholds` 时用于深色单元格。 |
| `colors` | 颜色数组 |  | 色阶的颜色，从最小值到最大值。给出两个或更多时取代 `min_color` 和 `max_color`，色阶依次经过所有颜色。 |
| `steps` | 0–1000 的整数 | `0` | 把数值分成这么多个等宽的分段，每段一种颜色，而不是连续渐变；`0` 和 `1` 表示保持连续。`colors` 的数量与分段数相同时，每段正好对应其中一种颜色。 |
| `thresholds` | 数字数组 |  | 各分段的分界值；取代 `steps`。小于第一个分界值的数值属于第一段。 |
| `symbol` | `"rect"` / `"circle"` | `"rect"` | 填充整个单元格（`rect`），或在每个有数值的单元格内画一个面积随数值变化的圆（`circle`），即打卡图。同一行的圆排列在一条线上，不显示数值标签。 |
| `data` | 数组 |  | 单元格数据 `[index, value]`，其中 `index` = y 序号 × x 类目数 + x 序号。 |

<!-- example: heatmap -->

```json
{
  "x_axis_data": ["12a", "6a", "12p"],
  "y_axis_data": ["Sat", "Sun"],
  "series": {
    "min": 0,
    "max": 10,
    "data": [[0, 9], [1, 3], [2, 6], [3, 8], [4, 2], [5, 5]]
  }
}
```

## Funnel

`FunnelChart::from_json` — 流程的各个阶段，自上而下排列。每个系列是一个阶段，数值为其 `data` 之和。

<!-- keys: funnel -->

| 键 | 类型 | 默认值 | 说明 |
|---|---|---|---|
| `funnel_gap` | 数字 | `2` | 各阶段之间的间距（像素）。 |
| `min_width` | 数字 | `20` | 最窄阶段的宽度（像素）。 |
| `sort_ascending` | 布尔 | `false` | 最小值排在顶部（默认最大值在顶部）。 |
| `series_label_position` | `"inside"` / `"left"` / `"right"` | `"right"` | 标签位置：阶段内部、左侧或右侧。 |
| `funnel_align` | `"left"` / `"center"` / `"right"` | `"center"` | 各阶段的水平对齐方式。 |

<!-- example: funnel -->

```json
{
  "series_label_position": "inside",
  "series_list": [
    {"name": "Show", "data": [100]},
    {"name": "Click", "data": [80]},
    {"name": "Order", "data": [20]}
  ]
}
```

## Waterfall

`WaterfallChart::from_json` — 每根柱从上一根柱结束的位置开始。数值写在 `data` 中（`x_axis_data` 的每个类目一项），而不是 `series_list`。

<!-- keys: waterfall -->

| 键 | 类型 | 默认值 | 说明 |
|---|---|---|---|
| `label_show` | 布尔 | `true` | 在每根柱上显示数值标签。 |
| `connector_line_show` | 布尔 | `true` | 在相邻柱之间绘制虚线连接。 |
| `connector_line_dash_array` | 字符串 | `"4,4"` | 连接线的虚线样式，写法同 SVG 的 `stroke-dasharray`；`""` 表示实线。 |
| `bar_width_ratio` | 数字 | `0.6` | 柱宽占类目宽度的比例，0 到 1。 |
| `increase_color` | 颜色 | 主题 | 增加项的柱颜色。 |
| `decrease_color` | 颜色 | `#EE6666` | 减少项的柱颜色。 |
| `total_color` | 颜色 | 主题 | 合计项的柱颜色。 |
| `data` | 数组 |  | 每个类目一项：数字（变化量），或 `[value, is_total]`；合计项显示到此为止的累计值。 |

<!-- example: waterfall -->

```json
{
  "x_axis_data": ["Revenue", "Cost", "Tax", "Profit"],
  "data": [[900, false], [-345, false], [-108, false], [0, true]]
}
```

## Calendar

`CalendarChart::from_json` — 把一年（或任意日期范围）画成按数值着色的日期方格。图表大小由方格决定：`width` 和 `height` 不生效，请改用 `cell_size` 与 `cell_gap`。

<!-- keys: calendar -->

| 键 | 类型 | 默认值 | 说明 |
|---|---|---|---|
| `start_date` | 字符串 |  | 显示的第一天，格式 `"YYYY-MM-DD"`。默认为当年 1 月 1 日。 |
| `end_date` | 字符串 |  | 显示的最后一天，格式 `"YYYY-MM-DD"`。默认为当年 12 月 31 日。 |
| `min` | 数字 | `0` | 对应 `min_color` 的数值；`0` 表示取数据中的最小值。 |
| `max` | 数字 | `0` | 对应 `max_color` 的数值；`0` 表示取数据中的最大值。 |
| `min_color` | 颜色 | `#EBEDF0` | 数值最小的日期的颜色。 |
| `max_color` | 颜色 | `#216E39` | 数值最大的日期的颜色。 |
| `colors` | 颜色数组 |  | 色阶的颜色，从最小值到最大值。给出两个或更多时取代 `min_color` 和 `max_color`，色阶依次经过所有颜色。 |
| `steps` | 0–1000 的整数 | `0` | 把数值分成这么多个等宽的分段，每段一种颜色，而不是连续渐变；`0` 和 `1` 表示保持连续。`colors` 的数量与分段数相同时，每段正好对应其中一种颜色。 |
| `thresholds` | 数字数组 |  | 各分段的分界值；取代 `steps`。小于第一个分界值的数值属于第一段。 |
| `empty_color` | 颜色 | 主题 | 没有数据的日期的颜色。 |
| `cell_size` | 数字 | `13` | 每个日期方格的边长（像素）。 |
| `cell_gap` | 数字 | `3` | 方格之间的间距（像素）。 |
| `month_label_height` | 数字 | `20` | 网格上方月份名称行的高度。 |
| `week_label_width` | 数字 | `30` | 网格左侧星期名称列的宽度。 |
| `show_dow_labels` | 数组 | `[1, 3, 5]` | 显示名称的星期，以 0（周日）到 6（周六）的数字表示。 |
| `data` | 数组 |  | 数据：每天一项 `["YYYY-MM-DD", value]`。 |

<!-- example: calendar -->

```json
{
  "start_date": "2024-01-01",
  "end_date": "2024-03-31",
  "data": [["2024-01-05", 3], ["2024-02-10", 7], ["2024-03-21", 5]]
}
```

## Gauge

`GaugeChart::from_json` — 仪表盘。每个系列是刻度上的一个值（取其 `data` 的第一项）：各画一根指针，开启 `multi_ring` 时各画一个进度环。

<!-- keys: gauge -->

| 键 | 类型 | 默认值 | 说明 |
|---|---|---|---|
| `min` | 数字 | `0` | 刻度起点的数值。 |
| `max` | 数字 | `100` | 刻度终点的数值。 |
| `start_angle` | 数字 | `225` | 刻度的起始角度，从 12 点方向顺时针计（度）。`0` 表示使用默认值：要从 12 点方向开始请写 `360`。 |
| `sweep_angle` | 数字 | `270` | 刻度跨越的角度（顺时针，度）。 |
| `radius` | 数字 | `0` | 外半径（像素）；`0` 表示自适应绘图区。 |
| `arc_width` | 数字 | `15` | 弧的厚度（像素）。 |
| `background_arc_color` | 颜色 | `#E6E6E6` | 弧上未填充部分的颜色。 |
| `show_pointer` | 布尔 | `true` | 是否绘制指针。 |
| `pointer_color` | 颜色 |  | 指针颜色；默认使用第一个系列的颜色。 |
| `show_axis_label` | 布尔 | `true` | 是否在弧的两端显示最小值和最大值。 |
| `split_number` | 0–1000 的整数 | `5` | 主刻度之间的分段数。 |
| `value_formatter` | 字符串 | `"{c}"` | 中心数值的格式：`{c}` 代表数值。 |
| `thresholds` | 数字数组 |  | 刻度各分段的分界值。设置后弧上显示各个分段（每段一种颜色）而不是进度，指针使用它所指分段的颜色。 |
| `colors` | 颜色数组 |  | 各分段的颜色，从低到高。默认使用系列颜色。 |
| `multi_ring` | 布尔 | `false` | 每个系列画成一个独立的进度环，由外向内排列，数值列在中间，而不是同一表盘上的多根指针。 |

<!-- example: gauge -->

```json
{
  "min": 0,
  "max": 100,
  "value_formatter": "{c}%",
  "series_list": [
    {"name": "CPU", "data": [72]}
  ]
}
```

## Treemap

`TreemapChart::from_json` — 面积与数值成比例的矩形。每个系列是一个矩形（数值取其 `data` 的第一项）；`series_data` 可提供嵌套数据。

<!-- keys: treemap -->

| 键 | 类型 | 默认值 | 说明 |
|---|---|---|---|
| `item_gap` | 数字 | `2` | 相邻单元格之间的间距（像素）。 |
| `series_data` | 对象数组 |  | 嵌套数据；设置后代替 `series_list`，每个分支按其子节点再细分。 |

### `series_data[]`

<!-- keys: treemap.series_data -->

| 键 | 类型 | 默认值 | 说明 |
|---|---|---|---|
| `name` | 字符串 |  | 节点名称，作为标签显示。 |
| `value` | 数字 |  | 叶子节点的数值；有子节点的节点取子节点之和。 |
| `color` | 颜色 |  | 节点颜色；默认取自调色板。 |
| `children` | 对象数组 |  | 子节点，每个子节点使用相同的键。 |

<!-- example: treemap -->

```json
{
  "series_data": [
    {
      "name": "Docs",
      "children": [
        {"name": "Guides", "value": 6},
        {"name": "API", "value": 4}
      ]
    },
    {"name": "Media", "value": 5}
  ]
}
```

## Box plot

`BoxPlotChart::from_json` — `x_axis_data` 的每个类目、每个系列一个箱体，由五个数值确定。数据写在 `box_series` 中，而不是 `series_list`。

<!-- keys: box_plot -->

| 键 | 类型 | 默认值 | 说明 |
|---|---|---|---|
| `box_series` | 对象数组 |  | 箱线系列；代替 `series_list`。 |

### `box_series[]`

<!-- keys: box_plot.box_series -->

| 键 | 类型 | 默认值 | 说明 |
|---|---|---|---|
| `name` | 字符串 |  | 系列名称，显示在图例中。 |
| `index` | 非负整数 |  | 在调色板中的位置；默认取系列的序号。 |
| `data` | 数组 |  | 每个类目一个箱体：`[min, q1, median, q3, max]`。 |

<!-- example: box_plot -->

```json
{
  "x_axis_data": ["Mon", "Tue"],
  "box_series": [
    {
      "name": "Latency",
      "data": [[655, 850, 940, 980, 1175], [672, 800, 845, 885, 1012]]
    }
  ]
}
```

## Sunburst

`SunburstChart::from_json` — 用圆环表示层级：每一层是一个圆环，每个节点是一段长度与数值成比例的弧。

<!-- keys: sunburst -->

| 键 | 类型 | 默认值 | 说明 |
|---|---|---|---|
| `series_data` | 对象数组 |  | 层级结构的根节点；多个根节点共享整个圆。 |
| `radius` | 数字 | `0` | 外半径（像素）；`0` 表示自适应绘图区。 |
| `inner_radius` | 数字 | `0` | 中心空洞的半径。 |
| `start_angle` | 数字 | `0` | 第一个根节点的起始角度，从 12 点方向顺时针计（度）。 |
| `level_thickness` | 数字数组 |  | 各层圆环的相对厚度，由内向外；未指定的层按 `1` 计。 |

### `series_data[]`

<!-- keys: sunburst.series_data -->

| 键 | 类型 | 默认值 | 说明 |
|---|---|---|---|
| `name` | 字符串 |  | 节点名称，作为标签显示。 |
| `value` | 数字 |  | 叶子节点的数值；有子节点的节点取子节点之和。 |
| `color` | 颜色 |  | 节点颜色；默认取自调色板。 |
| `children` | 对象数组 |  | 子节点，每个子节点使用相同的键。 |

<!-- example: sunburst -->

```json
{
  "inner_radius": 30,
  "series_data": [
    {
      "name": "Grandpa",
      "children": [
        {"name": "Uncle Leo", "value": 15},
        {
          "name": "Father",
          "children": [
            {"name": "Me", "value": 5},
            {"name": "Brother", "value": 1}
          ]
        }
      ]
    }
  ]
}
```

## Sankey

`SankeyChart::from_json` — 从左到右的有向流量：节点按列排布，连接是宽度与数值成比例的流量带。

<!-- keys: sankey -->

| 键 | 类型 | 默认值 | 说明 |
|---|---|---|---|
| `nodes` | 对象数组 |  | 节点。可省略：节点会根据 `links` 中出现的名称按首次出现顺序生成；列出后可固定顺序或颜色。 |
| `links` | 对象数组 |  | 节点之间的有向流量。 |
| `node_width` | 数字 | `16` | 节点矩形的宽度（像素）。 |
| `node_gap` | 数字 | `8` | 同一列中节点之间的垂直间距（像素）。 |
| `link_opacity` | 数字 | `0.45` | 流量带的不透明度，0 到 1。 |
| `node_align` | `"left"` / `"right"` / `"justify"` | `"left"` | 节点所在的列：尽量靠左（`left`，默认）、尽量靠右（`right`），或在靠左的基础上把终点节点放到最后一列（`justify`）。 |
| `link_gradient` | 布尔 | `false` | 流量带使用从源节点颜色到目标节点颜色的渐变，而不是源节点颜色。 |
| `orient` | `"horizontal"` / `"vertical"` | `"horizontal"` | 流向：从左到右，或从上到下（节点的列变为行）。 |

### `nodes[]`

<!-- keys: sankey.nodes -->

| 键 | 类型 | 默认值 | 说明 |
|---|---|---|---|
| `name` | 字符串 |  | 节点名称，作为标签显示；连接通过它引用节点。 |
| `color` | 颜色 |  | 节点颜色；默认按位置取调色板颜色。 |

### `links[]`

<!-- keys: sankey.links -->

| 键 | 类型 | 默认值 | 说明 |
|---|---|---|---|
| `source` | 字符串 |  | 连接起点的节点名称。 |
| `target` | 字符串 |  | 连接终点的节点名称。 |
| `value` | 数字 |  | 流量大小；数值不为正的连接会被忽略。 |

<!-- example: sankey -->

```json
{
  "link_gradient": true,
  "links": [
    {"source": "Coal", "target": "Electricity", "value": 25},
    {"source": "Gas", "target": "Electricity", "value": 15},
    {"source": "Electricity", "target": "Homes", "value": 30},
    {"source": "Electricity", "target": "Industry", "value": 10}
  ]
}
```

## Tree

`TreeChart::from_json` — 用连线连接节点来表示层级。

<!-- keys: tree -->

| 键 | 类型 | 默认值 | 说明 |
|---|---|---|---|
| `series_data` | 对象数组 |  | 层级结构的根节点；多个根节点并排布局。 |
| `orient` | `"LR"` / `"RL"` / `"TB"` / `"BT"` | `"LR"` | 根节点的位置和树的生长方向：根在左侧（`LR`）、右侧（`RL`）、顶部（`TB`）或底部（`BT`）。 |
| `symbol_size` | 数字 | `6` | 节点圆的半径（像素）。 |
| `layout` | `"orthogonal"` / `"radial"` | `"orthogonal"` | `orthogonal` 按 `orient` 把各层并排布局；`radial` 把根节点放在中心，每一层排在围绕它的一个圆上。 |
| `edge_shape` | `"curve"` / `"polyline"` | `"curve"` | 连线的形状：曲线，或直角折线（径向布局下为直线）。 |

### `series_data[]`

<!-- keys: tree.series_data -->

| 键 | 类型 | 默认值 | 说明 |
|---|---|---|---|
| `name` | 字符串 |  | 节点名称，作为标签显示。 |
| `value` | 数字 |  | 节点的数值，显示在提示中；有子节点的节点取子节点之和。 |
| `color` | 颜色 |  | 节点颜色；默认取自调色板。 |
| `children` | 对象数组 |  | 子节点，每个子节点使用相同的键。 |

<!-- example: tree -->

```json
{
  "orient": "TB",
  "series_data": [
    {
      "name": "root",
      "children": [
        {
          "name": "a",
          "children": [
            {"name": "a1"},
            {"name": "a2"}
          ]
        },
        {"name": "b"}
      ]
    }
  ]
}
```

## Graph

`GraphChart::from_json` — 关系网络：节点之间可以任意连接。

<!-- keys: graph -->

| 键 | 类型 | 默认值 | 说明 |
|---|---|---|---|
| `nodes` | 对象数组 |  | 节点。可省略：节点会根据 `links` 中出现的名称按首次出现顺序生成。 |
| `links` | 对象数组 |  | 节点之间的边。 |
| `symbol_size` | 数字 | `10` | 节点圆的基础半径（像素）；设置了 `value` 的节点在此基础上缩放。 |
| `layout` | `"force"` / `"circular"` | `"force"` | 节点布局：力导向（`force`，默认）或均匀分布在圆周上（`circular`）。 |
| `categories` | 字符串数组 |  | 节点分类的名称（节点通过 `category` 引用）；设置后作为图例显示。 |

### `nodes[]`

<!-- keys: graph.nodes -->

| 键 | 类型 | 默认值 | 说明 |
|---|---|---|---|
| `name` | 字符串 |  | 节点名称，作为标签显示；连接通过它引用节点。 |
| `value` | 数字 |  | 节点的重要程度：数值越大圆越大。 |
| `color` | 颜色 |  | 节点颜色；默认取其分类（或其位置）对应的调色板颜色。 |
| `category` | 非负整数 |  | `categories` 中的序号；同一分类的节点颜色相同。 |

### `links[]`

<!-- keys: graph.links -->

| 键 | 类型 | 默认值 | 说明 |
|---|---|---|---|
| `source` | 字符串 |  | 连接起点的节点名称。 |
| `target` | 字符串 |  | 连接终点的节点名称。 |
| `value` | 数字 |  | 边的权重：数值越大线越粗。 |

<!-- example: graph -->

```json
{
  "categories": ["Team", "Tool"],
  "nodes": [
    {"name": "Ann", "category": 0, "value": 3},
    {"name": "Bob", "category": 0},
    {"name": "Git", "category": 1}
  ],
  "links": [
    {"source": "Ann", "target": "Bob", "value": 2},
    {"source": "Ann", "target": "Git"},
    {"source": "Bob", "target": "Git"}
  ]
}
```

## Parallel

`ParallelChart::from_json` — 平行坐标图：`x_axis_data` 是各个维度的名称（每个维度一条竖轴），每个系列是一条记录，在每个维度上有一个值。它没有专属参数。

<!-- example: parallel -->

```json
{
  "x_axis_data": ["Price", "Weight", "Rating"],
  "series_list": [
    {"name": "A", "data": [120, 3.2, 4.5]},
    {"name": "B", "data": [90, 4.1, 3.8]}
  ]
}
```

## Theme river

`ThemeRiverChart::from_json` — 主题河流图：每个系列是沿 `x_axis_data` 类目延伸的一条流带，厚度对应数值。

<!-- keys: theme_river -->

| 键 | 类型 | 默认值 | 说明 |
|---|---|---|---|
| `stream_opacity` | 数字 | `0.85` | 各条流带的不透明度，0 到 1。 |

<!-- example: theme_river -->

```json
{
  "x_axis_data": ["Jan", "Feb", "Mar", "Apr"],
  "series_list": [
    {"name": "News", "data": [10, 25, 18, 30]},
    {"name": "Sport", "data": [15, 12, 22, 16]}
  ]
}
```

## Histogram

`HistogramChart::from_json` — 样本在等宽分箱上的分布。系列的 `data` 是原始样本；`x_axis_min` 与 `x_axis_max` 固定分箱的范围。

<!-- keys: histogram -->

| 键 | 类型 | 默认值 | 说明 |
|---|---|---|---|
| `bin_count` | 0–1000 的整数 | `0` | 分箱数量；`0` 表示根据样本量自动选择，并让分箱边界取整。 |
| `bin_width` | 正数 |  | 分箱宽度；优先于 `bin_count`。分箱边界是它的整数倍。 |
| `percent` | 布尔 | `false` | 显示占各系列样本的百分比，而不是数量。 |
| `bar_gap` | 数字 | `1` | 相邻柱之间的间距（像素）。 |

<!-- example: histogram -->

```json
{
  "bin_width": 5,
  "x_axis_title": "Height (cm)",
  "series_list": [
    {
      "name": "Adults",
      "data": [162.4, 171, 168.3, 175.9, 158.2, 169.7, 173.1, 166.8]
    }
  ]
}
```

## Polar bar

`PolarBarChart::from_json` — 极坐标上的柱状图，类目来自 `x_axis_data`。数值轴使用 `y_axis_configs` 的第一项。

<!-- keys: polar_bar -->

| 键 | 类型 | 默认值 | 说明 |
|---|---|---|---|
| `category_axis` | `"angle"` / `"radius"` | `"angle"` | 类目所在的轴：沿圆周（`angle`，柱向外生长）或从圆心向外（`radius`，柱沿圆周延伸）。 |
| `radius` | 正数 |  | 最大外半径；默认填满绘图区。 |
| `inner_radius` | 数字 |  | 中心空洞的半径。默认：类目在角度轴时为 `0`，否则为半径的四分之一。 |
| `start_angle` | 数字 | `0` | 两条轴的起始角度，从 12 点方向顺时针计（度）。 |
| `end_angle` | 数字 |  | 类目在半径轴时数值轴的终止角度。默认为 `start_angle` 之后 270 度；最多一整圈。 |
| `round_cap` | 布尔 | `false` | 沿圆周延伸的柱两端为圆头。 |
| `category_gap` | 数字 | `0.2` | 类目之间留白占类目宽度的比例，0 到 0.9。 |

<!-- example: polar_bar -->

```json
{
  "category_axis": "radius",
  "round_cap": true,
  "y_axis_configs": [
    {"axis_max": 100}
  ],
  "x_axis_data": ["Sleep", "Steps", "Water"],
  "series_list": [
    {"name": "Done", "data": [92, 74, 61], "label_show": true}
  ]
}
```

## Chord

`ChordChart::from_json` — 圆周上节点之间的流量：每个节点是一段长度对应其流量的弧，每条连接是一条横跨圆内的色带。

<!-- keys: chord -->

| 键 | 类型 | 默认值 | 说明 |
|---|---|---|---|
| `nodes` | 对象数组 |  | 节点，按顺时针排列。可省略：节点会根据 `links` 中出现的名称按首次出现顺序生成；列出后可固定顺序或颜色。 |
| `links` | 对象数组 |  | 节点之间的流量。 |
| `radius` | 正数 |  | 最大外半径；默认填满绘图区。 |
| `node_width` | 数字 | `12` | 节点环的厚度（像素）。 |
| `node_gap` | 数字 | `3` | 相邻节点之间的间隔角度（度）。 |
| `start_angle` | 数字 | `0` | 第一个节点的起始角度，从 12 点方向顺时针计（度）。 |
| `link_opacity` | 数字 | `0.5` | 色带的不透明度，0 到 1。 |
| `link_gradient` | 布尔 | `false` | 色带使用从源节点颜色到目标节点颜色的渐变，而不是源节点颜色。 |

### `nodes[]`

<!-- keys: chord.nodes -->

| 键 | 类型 | 默认值 | 说明 |
|---|---|---|---|
| `name` | 字符串 |  | 节点名称，作为标签显示；连接通过它引用节点。 |
| `color` | 颜色 |  | 节点颜色；默认按位置取调色板颜色。 |

### `links[]`

<!-- keys: chord.links -->

| 键 | 类型 | 默认值 | 说明 |
|---|---|---|---|
| `source` | 字符串 |  | 连接起点的节点名称。 |
| `target` | 字符串 |  | 连接终点的节点名称。 |
| `value` | 数字 |  | 流量大小；数值不为正的连接会被忽略。 |

<!-- example: chord -->

```json
{
  "link_gradient": true,
  "links": [
    {"source": "Asia", "target": "Europe", "value": 60},
    {"source": "Asia", "target": "Americas", "value": 45},
    {"source": "Europe", "target": "Americas", "value": 50}
  ]
}
```

## Gantt

`GanttChart::from_json` — 甘特图：任务以横条形式画在时间轴上，可以每个任务一行，也可以多个任务共用一行。x 轴默认是时间轴（数字表示 unix 秒），把 `x_axis_type` 设为 `value` 则是普通数值轴；`x_axis_min` 和 `x_axis_max` 固定范围，`x_axis_formatter` 和 `x_axis_time_offset` 控制刻度标签。它不使用 `series_list`：图例是任务的类别。

<!-- keys: gantt -->

| 键 | 类型 | 默认值 | 说明 |
|---|---|---|---|
| `tasks` | 对象数组 |  | 任务列表；行由任务决定，按首次出现的顺序自上而下排列。 |
| `bar_height` | 数字 |  | 任务条的高度（像素）；默认为行高的 60%。 |
| `radius` | 数字 | `3` | 任务条的圆角半径。 |
| `label_show` | 布尔 | `true` | 在任务条上显示任务名称，放不下时显示在旁边。独占一行的任务不再重复显示：行名就是它的名称。 |
| `now` | x 值 |  | 用虚线标出的一个时刻，例如今天。 |

### `tasks[]`

<!-- keys: gantt.tasks -->

| 键 | 类型 | 默认值 | 说明 |
|---|---|---|---|
| `name` | 字符串 |  | 任务名称。 |
| `start` | x 值 |  | 任务开始的时刻；没有开始时刻的任务会被忽略。 |
| `end` | x 值 |  | 任务结束的时刻。省略或与开始时刻相同时，任务是一个里程碑，绘制为菱形。 |
| `row` | 字符串 |  | 任务所在的行；`row` 相同的任务共用一行。默认独占一行，行名即任务名称。 |
| `category` | 字符串 |  | 任务的类别：同一类别的任务颜色相同，类别作为图例显示。 |
| `progress` | 数字 |  | 任务的完成进度，0 到 1：已完成部分使用完整颜色，其余部分颜色较浅。 |
| `color` | 颜色 |  | 任务的颜色；默认使用其类别的颜色。 |

<!-- example: gantt -->

```json
{
  "now": "2024-03-19",
  "tasks": [
    {
      "name": "Research",
      "category": "Plan",
      "start": "2024-03-04",
      "end": "2024-03-08",
      "progress": 1
    },
    {
      "name": "Design",
      "category": "Build",
      "start": "2024-03-07",
      "end": "2024-03-22",
      "progress": 0.6
    },
    {"name": "Sign-off", "category": "Build", "start": "2024-03-22"},
    {"name": "Launch", "category": "Plan", "start": "2024-03-25", "end": "2024-03-29"}
  ]
}
```

## Map

`MapChart::from_json` — 地图：各区域按数值着色。区域随参数一起以 GeoJSON 形式传入，图表本身不内置任何地图数据。它不使用 `series_list`。

<!-- keys: map -->

| 键 | 类型 | 默认值 | 说明 |
|---|---|---|---|
| `geo_json` | 对象 |  | 区域数据：GeoJSON 的 `FeatureCollection`（或单个 `Feature`），几何类型为 `Polygon` 和 `MultiPolygon`，坐标为经纬度（度）。其它几何类型会被忽略。 |
| `name_property` | 字符串 | `"name"` | GeoJSON feature 中作为区域名称的属性；没有该属性时使用 feature 的 `id`。 |
| `data` | 数组 |  | 数值：每个有数值的区域一项 `["区域名称", 数值]`。 |
| `projection` | `"mercator"` / `"equirectangular"` | `"mercator"` | 经纬度到平面的投影方式：墨卡托投影保持形状（南北纬 85° 以外被截掉），等距圆柱投影直接使用经纬度。 |
| `min` | 数字 | `0` | 色阶起点对应的数值。`min` 和 `max` 都为 `0` 时，色阶从最小值到最大值。 |
| `max` | 数字 | `0` | 色阶终点对应的数值。 |
| `min_color` | 颜色 | 主题 | 最小值的颜色：默认是主题第一个颜色的浅色调。 |
| `max_color` | 颜色 | 主题 | 最大值的颜色：默认是主题的第一个颜色。 |
| `colors` | 颜色数组 |  | 色阶的颜色，从最小值到最大值。给出两个或更多时取代 `min_color` 和 `max_color`，色阶依次经过所有颜色。 |
| `steps` | 0–1000 的整数 | `0` | 把数值分成这么多个等宽的分段，每段一种颜色，而不是连续渐变；`0` 和 `1` 表示保持连续。 |
| `thresholds` | 数字数组 |  | 各分段的分界值；取代 `steps`。小于第一个分界值的数值属于第一段。 |
| `empty_color` | 颜色 | 主题 | 没有数值的区域的颜色。 |
| `border_color` | 颜色 | 主题 | 区域边界线的颜色：默认使用背景色。 |
| `border_width` | 数字 | `1` | 区域边界线的宽度；`0` 表示不绘制。 |
| `label_show` | 布尔 | `false` | 在每个区域上显示其名称；会与其它名称重叠的名称会被省略。 |
| `visual_map_show` | 布尔 | `true` | 在地图旁显示颜色的图例：从最小值到最大值的色条，或每个分段一个色块。 |

<!-- example: map -->

```json
{
  "label_show": true,
  "thresholds": [100, 300],
  "colors": ["#deebf7", "#9ecae1", "#3182bd"],
  "data": [["West", 80], ["East", 420]],
  "geo_json": {
    "type": "FeatureCollection",
    "features": [
      {
        "type": "Feature",
        "properties": {"name": "West"},
        "geometry": {
          "type": "Polygon",
          "coordinates": [[[100, 20], [106, 20], [107, 26], [101, 27], [100, 20]]]
        }
      },
      {
        "type": "Feature",
        "properties": {"name": "East"},
        "geometry": {
          "type": "Polygon",
          "coordinates": [[[106, 20], [113, 21], [112, 27], [107, 26], [106, 20]]]
        }
      }
    ]
  }
}
```

## Multi chart

`MultiChart::from_json` — 把多个图表合并到一张 SVG 中，自上而下排列或各自指定位置。它不使用通用参数。

<!-- keys: multi -->

| 键 | 类型 | 默认值 | 说明 |
|---|---|---|---|
| `theme` | 字符串 |  | 未单独设置主题的子图表所使用的主题。 |
| `margin` | 边距 | `10` | 整个组合图四周的外边距。 |
| `gap` | 数字 | `10` | 自动排列的子图表之间的垂直间距。 |
| `background_color` | 颜色 |  | 整个组合图的背景色；不设置时只有各子图表绘制自己的背景。 |
| `child_charts` | 数组 |  | 子图表，自上而下依次绘制：每项是对应图表类型的参数，外加下面的键。 |
| `compact` | 布尔 | `false` | 组合图及所有子图表都输出精简 SVG。 |

### `child_charts[]`

<!-- keys: multi.child_charts -->

| 键 | 类型 | 默认值 | 说明 |
|---|---|---|---|
| `type` | 字符串 |  | 子图表类型：`bar`（默认）、`line`、`horizontal_bar`、`pie`、`radar`、`table`、`scatter`、`candlestick`、`heatmap`、`funnel`、`waterfall`、`calendar`、`gauge`、`treemap`、`box_plot`、`sunburst`、`sankey`、`tree`、`graph`、`parallel`、`theme_river`、`histogram`、`polar_bar`、`chord`、`gantt` 或 `map`。 |
| `x` | 数字 |  | 子图表的水平位置；设置了 `x` 或 `y` 后，子图表放在指定位置而不是排在上一个下方。 |
| `y` | 数字 |  | 子图表的垂直位置。 |

<!-- example: multi -->

```json
{
  "gap": 10,
  "child_charts": [
    {
      "type": "bar",
      "x_axis_data": ["Mon", "Tue"],
      "series_list": [
        {"name": "Email", "data": [120, 132]}
      ]
    },
    {
      "type": "pie",
      "height": 300,
      "series_list": [
        {"name": "Email", "data": [40]},
        {"name": "Search", "data": [38]}
      ]
    }
  ]
}
```

## 供工具使用的键

另有两个键所有图表都接受但不产生任何效果，方便库外的工具（例如 Web 编辑器）把它们放在同一份文档里：

<!-- keys: envelope -->

| 键 | 类型 | 默认值 | 说明 |
|---|---|---|---|
| `type` | 字符串 |  | 图表类型，供把同一份文档交给多种图表的工具使用。`from_json` 会忽略它。 |
| `quality` | 数字 |  | 图片质量，供同时负责图片编码的工具使用。`from_json` 会忽略它。 |
