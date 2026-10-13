# Regions of China

GeoJSON of the provinces of China and of the regions below each of them, for
maps drawn with `MapChart`. The files are in the repository only: `asset` is
left out of the published crate (`exclude` in `Cargo.toml`).

They are copies, byte for byte, of what
[DataV.GeoAtlas](https://datav.aliyun.com/portal/school/atlas/area_selector)
serves, downloaded on 2026-10-13:

```
https://geo.datav.aliyun.com/areas_v3/bound/{adcode}_full.json   the regions below one
https://geo.datav.aliyun.com/areas_v3/bound/{adcode}.json        its outline alone
```

`adcode` is the six-digit code of an administrative division: `100000` for
the country, `XX0000` for a province, `XXXX00` for a city. A district has no
`_full` file.

| File | Of | Regions |
|---|---|---|
| `100000_full.json` | 全国 | 34 provinces, and the dashes in the South China Sea |
| `110000_full.json` | 北京市 | 16 districts |
| `120000_full.json` | 天津市 | 16 districts |
| `130000_full.json` | 河北省 | 11 cities |
| `140000_full.json` | 山西省 | 11 cities |
| `150000_full.json` | 内蒙古自治区 | 12 cities |
| `210000_full.json` | 辽宁省 | 14 cities |
| `220000_full.json` | 吉林省 | 9 cities |
| `230000_full.json` | 黑龙江省 | 13 cities |
| `310000_full.json` | 上海市 | 16 districts |
| `320000_full.json` | 江苏省 | 13 cities |
| `330000_full.json` | 浙江省 | 11 cities |
| `340000_full.json` | 安徽省 | 16 cities |
| `350000_full.json` | 福建省 | 9 cities |
| `360000_full.json` | 江西省 | 11 cities |
| `370000_full.json` | 山东省 | 16 cities |
| `410000_full.json` | 河南省 | 18 cities |
| `420000_full.json` | 湖北省 | 17 cities |
| `430000_full.json` | 湖南省 | 14 cities |
| `440000_full.json` | 广东省 | 21 cities |
| `450000_full.json` | 广西壮族自治区 | 14 cities |
| `460000_full.json` | 海南省 | 19 cities |
| `500000_full.json` | 重庆市 | 38 districts |
| `510000_full.json` | 四川省 | 21 cities |
| `520000_full.json` | 贵州省 | 9 cities |
| `530000_full.json` | 云南省 | 16 cities |
| `540000_full.json` | 西藏自治区 | 7 cities |
| `610000_full.json` | 陕西省 | 10 cities |
| `620000_full.json` | 甘肃省 | 14 cities |
| `630000_full.json` | 青海省 | 8 cities |
| `640000_full.json` | 宁夏回族自治区 | 5 cities |
| `650000_full.json` | 新疆维吾尔自治区 | 24 cities |
| — | 台湾省 | no regions below it at the source |
| `810000_full.json` | 香港特别行政区 | 18 districts |
| `820000_full.json` | 澳门特别行政区 | 8 districts |

## Using them

```rust
let text = std::fs::read_to_string("asset/geo/china/440000_full.json")?;
let regions = MapRegion::from_geo_json(&text, "name")?;
let chart = MapChart::new(regions, vec![("广州市".to_string(), 420.0)]);
```

- The name of a region is its property `name`, which is what `MapChart`
  reads by default, and it is the full name: `广东省`, `新疆维吾尔自治区`,
  `广州市`. `data` has to write it the same way.
- The last feature of `100000_full.json` has no name (its `adcode` is
  `100000_JD`): the dashes in the South China Sea. It makes the map reach
  further south, and is best left in.
- Below the four municipalities, and below Hong Kong and Macao, there are
  districts, not cities.
- A label on a region needs a font with Chinese in it: register one with
  `add_fonts` before `label_show` is turned on.

## Updating them

```bash
cd asset/geo/china
for file in *_full.json; do
  curl -sf -o "$file" "https://geo.datav.aliyun.com/areas_v3/bound/$file"
done
```

## Terms

The terms of use of the source have not been checked. Check them before the
files are redistributed or used in something that is published, and see the
note on borders in the [guide](../../../docs/guide.md#map-chart).
