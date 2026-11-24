use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use charts_rs::{
    BarChart, Box as ChartBox, CandlestickChart, ChordChart, GanttChart, LineChart, MapChart,
    MermaidChart, PolarBarChart, SankeyChart, ScatterChart, SeriesCategory, THEME_CHALK,
    THEME_DARK, THEME_GRAFANA, THEME_LIGHT, THEME_SHADCN, THEME_VINTAGE, add_fonts,
    get_font_families, measure_text_width_family, svg_to_png,
};

const WIDTH: u32 = 1920;
const HEIGHT: u32 = 1080;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Lang {
    En,
    Zh,
}

struct CliArgs {
    lang: Lang,
    out: Option<PathBuf>,
    font: Option<PathBuf>,
    frames_only: bool,
}

struct SlideSpec {
    duration: f32,
    transition: &'static str,
    trans_duration: f32,
    svg: String,
}

struct ChartSlideMeta {
    category_en: &'static str,
    category_zh: &'static str,
    title_en: &'static str,
    title_zh: &'static str,
    subtitle_en: &'static str,
    subtitle_zh: &'static str,
    transition: &'static str,
    svg: String,
    theme_badge: Option<&'static str>,
}

fn parse_args() -> Result<CliArgs, Box<dyn std::error::Error>> {
    let mut lang = Lang::En;
    let mut out = None;
    let mut font = None;
    let mut frames_only = false;

    let mut args = env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--lang" => {
                let val = args.next().ok_or("missing value after --lang")?;
                lang = match val.as_str() {
                    "en" => Lang::En,
                    "zh" => Lang::Zh,
                    other => return Err(format!("unsupported --lang `{other}` (use en|zh)").into()),
                };
            }
            "--out" => {
                let val = args.next().ok_or("missing value after --out")?;
                out = Some(PathBuf::from(val));
            }
            "--font" => {
                let val = args.next().ok_or("missing value after --font")?;
                font = Some(PathBuf::from(val));
            }
            "--frames-only" => frames_only = true,
            "--help" | "-h" => {
                println!(
                    "Usage: cargo run --release --example showcase --features \"mermaid,png\" -- [--lang en|zh] [--out PATH] [--font PATH] [--frames-only]"
                );
                std::process::exit(0);
            }
            other => return Err(format!("unknown argument `{other}`").into()),
        }
    }

    Ok(CliArgs {
        lang,
        out,
        font,
        frames_only,
    })
}

fn setup_font_family(
    lang: Lang,
    explicit_font: Option<&Path>,
) -> Result<String, Box<dyn std::error::Error>> {
    if lang == Lang::En && explicit_font.is_none() {
        return Ok("Roboto".to_string());
    }

    let before = get_font_families().unwrap_or_default();
    let probe_paths: Vec<PathBuf> = if let Some(p) = explicit_font {
        vec![p.to_path_buf()]
    } else {
        [
            "/System/Library/Fonts/STHeiti Medium.ttc",
            "/System/Library/Fonts/Supplemental/Arial Unicode.ttf",
            "/Library/Fonts/Arial Unicode.ttf",
            "/usr/share/fonts/opentype/noto/NotoSansCJK-Regular.ttc",
            "/usr/share/fonts/truetype/noto/NotoSansCJK-Regular.ttc",
            "/usr/share/fonts/noto-cjk/NotoSansCJK-Regular.ttc",
        ]
        .into_iter()
        .map(PathBuf::from)
        .collect()
    };

    for path in probe_paths {
        if !path.exists() {
            continue;
        }
        let data = fs::read(&path)?;
        if add_fonts(&[&data]).is_ok() {
            let after = get_font_families().unwrap_or_default();
            if let Some(added) = after
                .iter()
                .find(|f| !before.contains(f) && f.as_str() != "Roboto")
                .or_else(|| after.iter().find(|f| f.as_str() != "Roboto"))
            {
                return Ok(format!("Roboto, {added}"));
            }
        }
    }

    if explicit_font.is_some() {
        return Err("failed to load the font passed via --font".into());
    }
    Ok("Roboto".to_string())
}

fn escape_xml(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for ch in s.chars() {
        match ch {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#39;"),
            _ => out.push(ch),
        }
    }
    out
}

fn extract_attr<'a>(tag: &'a str, attr: &str) -> Option<&'a str> {
    let pat = format!("{attr}=\"");
    let start = tag.find(&pat)? + pat.len();
    let end = tag[start..].find('"')?;
    Some(&tag[start..start + end])
}

fn extract_svg_parts(svg: &str) -> (f32, f32, String, &str) {
    let open_end = svg.find('>').unwrap_or(0);
    let open_tag = &svg[..open_end];
    let w = extract_attr(open_tag, "width")
        .and_then(|s| s.parse::<f32>().ok())
        .unwrap_or(600.0);
    let h = extract_attr(open_tag, "height")
        .and_then(|s| s.parse::<f32>().ok())
        .unwrap_or(400.0);
    let close_start = svg.rfind("</svg>").unwrap_or(svg.len());
    let inner = &svg[open_end + 1..close_start];

    let bg = inner
        .find("<rect ")
        .and_then(|pos| {
            let end = inner[pos..].find('>')?;
            extract_attr(&inner[pos..pos + end], "fill")
        })
        .filter(|c| *c != "none" && !c.is_empty())
        .unwrap_or("#FFFFFF")
        .to_string();

    (w, h, bg, inner)
}

fn is_dark_hex(color: &str) -> bool {
    let hex = color.trim().trim_start_matches('#');
    if hex.len() < 6 {
        return false;
    }
    let parse_byte = |i: usize| u8::from_str_radix(&hex[i..i + 2], 16).unwrap_or(255) as f32;
    let r = parse_byte(0);
    let g = parse_byte(2);
    let b = parse_byte(4);
    (0.299 * r + 0.587 * g + 0.114 * b) < 130.0
}

fn pill_width(font_family: &str, text: &str, font_size: f32, pad_x: f32) -> f32 {
    let family = if text.is_ascii() {
        font_family.split(',').next().unwrap_or("Roboto").trim()
    } else {
        font_family
            .split(',')
            .next_back()
            .unwrap_or("Roboto")
            .trim()
    };
    let measured = measure_text_width_family(family, font_size, text)
        .map(|b| b.width())
        .unwrap_or_else(|_| text.chars().count() as f32 * font_size * 0.75);
    measured + pad_x * 2.0
}

fn svg_backdrop_defs() -> &'static str {
    r##"<defs>
  <linearGradient id="showcase_bg" x1="0%" y1="0%" x2="100%" y2="100%">
    <stop offset="0%" stop-color="#060A14"/>
    <stop offset="55%" stop-color="#0B1222"/>
    <stop offset="100%" stop-color="#111C33"/>
  </linearGradient>
  <radialGradient id="showcase_glow_a" cx="18%" cy="14%" r="55%">
    <stop offset="0%" stop-color="#3B82F6" stop-opacity="0.22"/>
    <stop offset="100%" stop-color="#3B82F6" stop-opacity="0"/>
  </radialGradient>
  <radialGradient id="showcase_glow_b" cx="86%" cy="84%" r="52%">
    <stop offset="0%" stop-color="#6366F1" stop-opacity="0.18"/>
    <stop offset="100%" stop-color="#6366F1" stop-opacity="0"/>
  </radialGradient>
  <linearGradient id="showcase_accent" x1="0%" y1="0%" x2="100%" y2="0%">
    <stop offset="0%" stop-color="#38BDF8"/>
    <stop offset="100%" stop-color="#818CF8"/>
  </linearGradient>
</defs>
<rect x="0" y="0" width="1920" height="1080" fill="url(#showcase_bg)"/>
<rect x="0" y="0" width="1920" height="1080" fill="url(#showcase_glow_a)"/>
<rect x="0" y="0" width="1920" height="1080" fill="url(#showcase_glow_b)"/>"##
}

fn render_intro_slide(lang: Lang, font: &str) -> String {
    let (badge, tagline, pills): (&str, &str, [&str; 4]) = match lang {
        Lang::En => (
            "PURE RUST DATA VISUALIZATION & MERMAID",
            "Charts & Mermaid for Rust, without a browser.",
            [
                "28 Chart Types",
                "13 Mermaid Diagrams",
                "10 Built-in Themes",
                "< 1ms SVG Render",
            ],
        ),
        Lang::Zh => (
            "纯 RUST 数据可视化与 MERMAID 引擎",
            "无需浏览器的 Rust 图表与 Mermaid 渲染库",
            [
                "28 种常规图表",
                "13 种 Mermaid 图表",
                "10 套内置主题",
                "< 1ms 生成 SVG",
            ],
        ),
    };

    let badge_w = pill_width(font, badge, 17.0, 26.0).max(260.0);
    let badge_x = (WIDTH as f32 - badge_w) * 0.5;

    let mut pills_svg = String::new();
    let gap = 22.0_f32;
    let widths: Vec<f32> = pills
        .iter()
        .map(|p| pill_width(font, p, 20.0, 28.0).max(185.0))
        .collect();
    let total_w: f32 = widths.iter().sum::<f32>() + gap * (widths.len() as f32 - 1.0);
    let mut px = (WIDTH as f32 - total_w) * 0.5;
    for (label, w) in pills.iter().zip(widths) {
        pills_svg.push_str(&format!(
            r##"<rect x="{px:.1}" y="660" width="{w:.1}" height="52" rx="26" fill="#131E36" stroke="#334155" stroke-width="1.5"/>
<text x="{cx:.1}" y="686" font-family="{font}" font-size="20" font-weight="bold" fill="#E2E8F0" text-anchor="middle" dominant-baseline="central">{label}</text>
"##,
            cx = px + w * 0.5,
            font = escape_xml(font),
            label = escape_xml(label),
        ));
        px += w + gap;
    }

    format!(
        r##"<svg width="{WIDTH}" height="{HEIGHT}" viewBox="0 0 {WIDTH} {HEIGHT}" xmlns="http://www.w3.org/2000/svg">
{defs}
<circle cx="960" cy="540" r="420" fill="none" stroke="#1E293B" stroke-width="1"/>
<circle cx="960" cy="540" r="300" fill="none" stroke="#1E293B" stroke-width="1"/>
<rect x="{badge_x:.1}" y="310" width="{badge_w:.1}" height="42" rx="21" fill="#0F172A" stroke="#38BDF8" stroke-opacity="0.55" stroke-width="1.5"/>
<text x="960" y="331" font-family="{font}" font-size="16" font-weight="bold" fill="#7DD3FC" text-anchor="middle" dominant-baseline="central">{badge}</text>
<text x="960" y="458" font-family="{font}" font-size="108" font-weight="bold" fill="#F8FAFC" text-anchor="middle" dominant-baseline="central">charts-rs</text>
<rect x="860" y="528" width="200" height="5" rx="2.5" fill="url(#showcase_accent)"/>
<text x="960" y="588" font-family="{font}" font-size="32" fill="#CBD5E1" text-anchor="middle" dominant-baseline="central">{tagline}</text>
{pills_svg}
</svg>"##,
        defs = svg_backdrop_defs(),
        font = escape_xml(font),
        badge = escape_xml(badge),
        tagline = escape_xml(tagline),
    )
}

fn render_outro_slide(lang: Lang, font: &str) -> String {
    let (headline, sub, cta): (&str, &str, &str) = match lang {
        Lang::En => (
            "Ship crisp SVG & PNG charts in pure Rust.",
            "Typed Rust builder  ·  ECharts-style JSON  ·  Zero headless browser",
            "github.com/vicanso/charts-rs   ·   charts.npmtrend.com",
        ),
        Lang::Zh => (
            "用纯 Rust 交付高清 SVG 与 PNG 图表",
            "强类型 Rust API  ·  类 ECharts JSON  ·  零无头浏览器依赖",
            "github.com/vicanso/charts-rs   ·   charts.npmtrend.com",
        ),
    };
    let cmd = "$ cargo add charts-rs --features \"mermaid,png\"";

    format!(
        r##"<svg width="{WIDTH}" height="{HEIGHT}" viewBox="0 0 {WIDTH} {HEIGHT}" xmlns="http://www.w3.org/2000/svg">
{defs}
<circle cx="960" cy="540" r="410" fill="none" stroke="#1E293B" stroke-width="1"/>
<text x="960" y="340" font-family="{font}" font-size="92" font-weight="bold" fill="#F8FAFC" text-anchor="middle" dominant-baseline="central">charts-rs</text>
<text x="960" y="434" font-family="{font}" font-size="30" fill="#CBD5E1" text-anchor="middle" dominant-baseline="central">{headline}</text>
<rect x="490" y="510" width="940" height="84" rx="16" fill="#0B1325" stroke="#38BDF8" stroke-opacity="0.55" stroke-width="1.5"/>
<text x="960" y="552" font-family="{font}" font-size="28" font-weight="bold" fill="#7DD3FC" text-anchor="middle" dominant-baseline="central">{cmd}</text>
<text x="960" y="654" font-family="{font}" font-size="22" fill="#94A3B8" text-anchor="middle" dominant-baseline="central">{sub}</text>
<text x="960" y="726" font-family="{font}" font-size="22" font-weight="bold" fill="#E2E8F0" text-anchor="middle" dominant-baseline="central">{cta}</text>
</svg>"##,
        defs = svg_backdrop_defs(),
        font = escape_xml(font),
        headline = escape_xml(headline),
        cmd = escape_xml(cmd),
        sub = escape_xml(sub),
        cta = escape_xml(cta),
    )
}

fn render_chart_slide(
    slide: &ChartSlideMeta,
    index: usize,
    total_chart_slides: usize,
    lang: Lang,
    font: &str,
) -> String {
    let (orig_w, orig_h, card_bg, inner) = extract_svg_parts(&slide.svg);
    let dark_card = is_dark_hex(&card_bg);
    let card_border = if dark_card { "#334155" } else { "#CBD5E1" };

    let (category, title, subtitle, brand_note) = match lang {
        Lang::En => (
            slide.category_en,
            slide.title_en,
            slide.subtitle_en,
            "Pure Rust  ·  No Browser  ·  <1ms SVG",
        ),
        Lang::Zh => (
            slide.category_zh,
            slide.title_zh,
            slide.subtitle_zh,
            "纯 Rust  ·  无浏览器  ·  <1ms 生成 SVG",
        ),
    };

    let cat_w = pill_width(font, category, 15.0, 22.0).max(160.0);
    let progress_w = 240.0_f32;
    let fill_w = (progress_w * (index as f32 / total_chart_slides.max(1) as f32)).max(12.0);
    let counter = format!("{index:02} / {total_chart_slides:02}");

    let theme_pill = if let Some(theme_name) = slide.theme_badge {
        let label = match lang {
            Lang::En => format!("Theme: {theme_name}"),
            Lang::Zh => format!("主题: {theme_name}"),
        };
        let w = pill_width(font, &label, 16.0, 20.0).max(150.0);
        let x = 1800.0 - w - 20.0;
        format!(
            r##"<rect x="{x:.1}" y="142" width="{w:.1}" height="34" rx="17" fill="#0F172A" fill-opacity="0.92" stroke="#38BDF8" stroke-width="1.5"/>
<text x="{cx:.1}" y="159" font-family="{font}" font-size="16" font-weight="bold" fill="#7DD3FC" text-anchor="middle" dominant-baseline="central">{label}</text>"##,
            cx = x + w * 0.5,
            font = escape_xml(font),
            label = escape_xml(&label),
        )
    } else {
        String::new()
    };

    format!(
        r##"<svg width="{WIDTH}" height="{HEIGHT}" viewBox="0 0 {WIDTH} {HEIGHT}" xmlns="http://www.w3.org/2000/svg">
{defs}
<rect x="120" y="54" width="{cat_w:.1}" height="34" rx="17" fill="#131E36" stroke="#38BDF8" stroke-opacity="0.5" stroke-width="1.2"/>
<text x="{cat_cx:.1}" y="71" font-family="{font}" font-size="15" font-weight="bold" fill="#93C5FD" text-anchor="middle" dominant-baseline="central">{category}</text>
<text x="1540" y="71" font-family="{font}" font-size="15" font-weight="bold" fill="#94A3B8" text-anchor="end" dominant-baseline="central">{counter}</text>
<rect x="1560" y="68" width="{progress_w:.1}" height="6" rx="3" fill="#1E293B"/>
<rect x="1560" y="68" width="{fill_w:.1}" height="6" rx="3" fill="url(#showcase_accent)"/>

<rect x="114" y="114" width="1692" height="808" rx="24" fill="#020617" fill-opacity="0.45"/>
<rect x="120" y="120" width="1680" height="796" rx="20" fill="{card_bg}" stroke="{card_border}" stroke-width="1.5"/>
<svg x="156" y="150" width="1608" height="736" viewBox="0 0 {orig_w} {orig_h}" preserveAspectRatio="xMidYMid meet">
{inner}
</svg>
{theme_pill}

<text x="120" y="974" font-family="{font}" font-size="28" font-weight="bold" fill="#F8FAFC" dominant-baseline="central">{title}</text>
<text x="120" y="1014" font-family="{font}" font-size="18" fill="#94A3B8" dominant-baseline="central">{subtitle}</text>
<text x="1800" y="974" font-family="{font}" font-size="24" font-weight="bold" fill="#E2E8F0" text-anchor="end" dominant-baseline="central">charts-rs</text>
<text x="1800" y="1014" font-family="{font}" font-size="16" fill="#64748B" text-anchor="end" dominant-baseline="central">{brand_note}</text>
</svg>"##,
        defs = svg_backdrop_defs(),
        cat_cx = 120.0 + cat_w * 0.5,
        font = escape_xml(font),
        category = escape_xml(category),
        counter = escape_xml(&counter),
        card_bg = escape_xml(&card_bg),
        title = escape_xml(title),
        subtitle = escape_xml(subtitle),
        brand_note = escape_xml(brand_note),
    )
}

fn build_mixed_bar_chart(theme: &str) -> Result<String, Box<dyn std::error::Error>> {
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
        theme,
    );
    bar_chart.title.text = "Mixed Line and Bar".to_string();
    bar_chart.legend.margin = Some(ChartBox {
        top: bar_chart.title.height,
        bottom: 5.0,
        ..Default::default()
    });
    bar_chart.series_list[2].category = Some(SeriesCategory::Line);
    bar_chart.series_list[2].y_axis_index = 1;
    bar_chart.series_list[2].label_show = true;
    bar_chart.x_axis.name_rotate = 0.785;
    bar_chart
        .y_axis_configs
        .push(bar_chart.y_axis_configs[0].clone());
    bar_chart.y_axis_configs[0].formatter = Some("{c} ml".to_string());
    bar_chart.y_axis_configs[1].formatter = Some("{c} °C".to_string());
    Ok(bar_chart.svg()?)
}

fn build_candlestick_chart() -> Result<String, Box<dyn std::error::Error>> {
    let mut candlestick = CandlestickChart::new_with_theme(
        vec![
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
        [
            "2013/1/24",
            "2013/1/25",
            "2013/1/28",
            "2013/1/29",
            "2013/1/30",
            "2013/1/31",
            "2013/2/1",
            "2013/2/4",
            "2013/2/5",
            "2013/2/6",
            "2013/2/7",
            "2013/2/8",
            "2013/2/18",
            "2013/2/19",
            "2013/2/20",
            "2013/2/21",
            "2013/2/22",
            "2013/2/25",
            "2013/2/26",
            "2013/2/27",
            "2013/2/28",
            "2013/3/1",
            "2013/3/4",
            "2013/3/5",
            "2013/3/6",
            "2013/3/7",
            "2013/3/8",
            "2013/3/11",
            "2013/3/12",
            "2013/3/13",
            "2013/3/14",
            "2013/3/15",
            "2013/3/18",
            "2013/3/18",
            "2013/3/20",
            "2013/3/21",
            "2013/3/22",
            "2013/3/25",
            "2013/3/26",
            "2013/3/27",
            "2013/3/28",
            "2013/3/29",
            "2013/4/1",
            "2013/4/2",
            "2013/4/3",
            "2013/4/8",
            "2013/4/9",
            "2013/4/10",
            "2013/4/11",
            "2013/4/12",
            "2013/4/15",
            "2013/4/16",
            "2013/4/17",
            "2013/4/18",
            "2013/4/19",
            "2013/4/22",
            "2013/4/23",
            "2013/4/24",
            "2013/4/25",
            "2013/4/26",
        ]
        .into_iter()
        .map(String::from)
        .collect(),
        THEME_GRAFANA,
    );
    candlestick.series_list[0].category = Some(SeriesCategory::Line);
    candlestick.series_list[0].start_index = 5;
    candlestick.y_axis_configs[0].min = Some(2100.0);
    candlestick.y_axis_configs[0].max = Some(2460.0);
    candlestick.y_axis_configs[0].formatter = Some("{t}".to_string());
    Ok(candlestick.svg()?)
}

fn collect_chart_slides() -> Result<Vec<ChartSlideMeta>, Box<dyn std::error::Error>> {
    let mut slides = vec![
        ChartSlideMeta {
            category_en: "CORE CHARTS",
            category_zh: "基础图表",
            title_en: "Mixed Line & Dual-Axis Bar",
            title_zh: "柱线混合双 Y 轴图表",
            subtitle_en: "Combine bar and line series with independent left/right scales",
            subtitle_zh: "支持柱状与折线系列混排，以及左右独立刻度映射",
            transition: "fade",
            svg: build_mixed_bar_chart(THEME_GRAFANA)?,
            theme_badge: None,
        },
        ChartSlideMeta {
            category_en: "CORE CHARTS",
            category_zh: "基础图表",
            title_en: "Bump & Ranking Splines",
            title_zh: "排名变化趋势图",
            subtitle_en: "Smooth monotone splines tracking rank changes over time",
            subtitle_zh: "单调平滑样条曲线呈现多系列排名的连续演变",
            transition: "fade",
            svg: LineChart::from_json(include_str!("../asset/line_chart/bump.json"))?.svg()?,
            theme_badge: None,
        },
        ChartSlideMeta {
            category_en: "FINANCIAL",
            category_zh: "金融图表",
            title_en: "Candlestick & Moving Average",
            title_zh: "K 线图与 MA5 均线",
            subtitle_en: "OHLC candlesticks overlaid with a 5-day moving average line",
            subtitle_zh: "开盘/收盘/高低价蜡烛图叠加均线指标与千分位格式化",
            transition: "smoothleft",
            svg: build_candlestick_chart()?,
            theme_badge: None,
        },
        ChartSlideMeta {
            category_en: "STATISTICAL",
            category_zh: "统计分析",
            title_en: "Scatter & 2D Density Contours",
            title_zh: "散点图与二维核密度等高线",
            subtitle_en: "Automatic KDE contour bands over multi-group scatter points",
            subtitle_zh: "在多分组散点上自动叠加核密度等高线区间",
            transition: "smoothleft",
            svg: ScatterChart::from_json(include_str!("../asset/scatter_chart/contour.json"))?
                .svg()?,
            theme_badge: None,
        },
        ChartSlideMeta {
            category_en: "FLOW & RELATIONS",
            category_zh: "关系与流向",
            title_en: "Gradient Chord Diagram",
            title_zh: "渐变色和弦图",
            subtitle_en: "Directed pairwise flows with smooth gradient ribbons",
            subtitle_zh: "用渐变色带直观展示分组之间的双向流量分布",
            transition: "smoothleft",
            svg: ChordChart::from_json(include_str!("../asset/chord_chart/gradient.json"))?
                .svg()?,
            theme_badge: None,
        },
        ChartSlideMeta {
            category_en: "FLOW & RELATIONS",
            category_zh: "关系与流向",
            title_en: "Multi-Stage Sankey Flow",
            title_zh: "多级桑基流向图",
            subtitle_en: "Energy and resource transfers weighted across columns",
            subtitle_zh: "按流量比例自动布局多阶段节点与平滑贝塞尔流带",
            transition: "fade",
            svg: SankeyChart::from_json(include_str!("../asset/sankey_chart/integration.json"))?
                .svg()?,
            theme_badge: None,
        },
        ChartSlideMeta {
            category_en: "RADIAL",
            category_zh: "极坐标图表",
            title_en: "Radial Polar Bar",
            title_zh: "极坐标环形柱状图",
            subtitle_en: "Concentric rounded progress arcs with crisp dark contrast",
            subtitle_zh: "同心圆弧进度环与极坐标角度刻度",
            transition: "smoothleft",
            svg: PolarBarChart::from_json(include_str!("../asset/polar_bar_chart/radial.json"))?
                .svg()?,
            theme_badge: None,
        },
        ChartSlideMeta {
            category_en: "GEOSPATIAL",
            category_zh: "地理空间",
            title_en: "GeoJSON Choropleth Map",
            title_zh: "GeoJSON 分级填色地图",
            subtitle_en: "Pure-Rust GeoJSON projection with discrete color scale legend",
            subtitle_zh: "原生 GeoJSON 投影渲染与连续/分段色阶图例",
            transition: "smoothleft",
            svg: MapChart::from_json(include_str!("../asset/map_chart/basic.json"))?.svg()?,
            theme_badge: None,
        },
        ChartSlideMeta {
            category_en: "PROJECT PLANNING",
            category_zh: "项目排期",
            title_en: "Gantt Timeline & Milestones",
            title_zh: "甘特图与里程碑",
            subtitle_en: "Progress bars, section colors, diamonds and reference lines",
            subtitle_zh: "支持任务进度条、分组配色、里程碑菱形与参考标线",
            transition: "smoothleft",
            svg: GanttChart::from_json(include_str!("../asset/gantt_chart/basic.json"))?.svg()?,
            theme_badge: None,
        },
        ChartSlideMeta {
            category_en: "PURE-RUST MERMAID",
            category_zh: "纯 RUST MERMAID",
            title_en: "Mermaid Flowchart",
            title_zh: "Mermaid 流程图",
            subtitle_en: "Orthogonal ports, subgraphs, cylinders and custom node styling",
            subtitle_zh: "正交端口路由、子图容器、圆柱节点与自定义样式",
            transition: "smoothleft",
            svg: MermaidChart::from_json(include_str!("../asset/mermaid_chart/flowchart.json"))?
                .svg()?,
            theme_badge: None,
        },
        ChartSlideMeta {
            category_en: "PURE-RUST MERMAID",
            category_zh: "纯 RUST MERMAID",
            title_en: "Mermaid Sequence Diagram",
            title_zh: "Mermaid 时序图",
            subtitle_en: "Participant create/destroy, activation bars and loop/opt frames",
            subtitle_zh: "支持参与者动态创建/销毁、激活条以及 loop/opt 控制框",
            transition: "fade",
            svg: MermaidChart::from_json(include_str!(
                "../asset/mermaid_chart/sequence_lifecycle.json"
            ))?
            .svg()?,
            theme_badge: None,
        },
        ChartSlideMeta {
            category_en: "PURE-RUST MERMAID",
            category_zh: "纯 RUST MERMAID",
            title_en: "Mermaid Class Diagram",
            title_zh: "Mermaid 类图",
            subtitle_en: "Namespaces, stereotypes, static/abstract cues and UML arrows",
            subtitle_zh: "命名空间分框、构造型标记、静态/抽象样式与 UML 关系箭头",
            transition: "fade",
            svg: MermaidChart::from_json(include_str!("../asset/mermaid_chart/class.json"))?
                .svg()?,
            theme_badge: None,
        },
        ChartSlideMeta {
            category_en: "PURE-RUST MERMAID",
            category_zh: "纯 RUST MERMAID",
            title_en: "Mermaid Mindmap",
            title_zh: "Mermaid 思维导图",
            subtitle_en: "Balanced left/right radial layout with clouds, hexagons and pills",
            subtitle_zh: "左右均衡辐射布局，支持云朵、六边形与圆角分支形状",
            transition: "fade",
            svg: MermaidChart::from_json(include_str!("../asset/mermaid_chart/mindmap.json"))?
                .svg()?,
            theme_badge: None,
        },
    ];

    for (idx, theme_name) in [
        THEME_DARK,
        THEME_LIGHT,
        THEME_SHADCN,
        THEME_GRAFANA,
        THEME_CHALK,
        THEME_VINTAGE,
    ]
    .into_iter()
    .enumerate()
    {
        slides.push(ChartSlideMeta {
            category_en: "10 BUILT-IN THEMES",
            category_zh: "10 套内置主题",
            title_en: "Instant Theme Switching",
            title_zh: "一键切换整图主题配色",
            subtitle_en: "One chart definition rendered across dark, light and editorial palettes",
            subtitle_zh: "同一份数据无缝切换暗色、亮色及多种出版级主题",
            transition: if idx == 0 { "smoothleft" } else { "fade" },
            svg: build_mixed_bar_chart(theme_name)?,
            theme_badge: Some(theme_name),
        });
    }

    Ok(slides)
}

fn encode_mp4(
    slides: &[SlideSpec],
    frame_paths: &[PathBuf],
    out_path: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    if slides.len() < 2 {
        return Err("need at least 2 slides to encode video".into());
    }
    if let Some(parent) = out_path.parent()
        && !parent.as_os_str().is_empty()
    {
        fs::create_dir_all(parent)?;
    }

    let mut cmd = Command::new("ffmpeg");
    cmd.arg("-y");
    for (slide, path) in slides.iter().zip(frame_paths.iter()) {
        cmd.arg("-framerate")
            .arg("60")
            .arg("-loop")
            .arg("1")
            .arg("-t")
            .arg(format!("{:.3}", slide.duration))
            .arg("-i")
            .arg(path);
    }

    let mut filter = String::new();
    let mut cum_dur = slides[0].duration;
    let last = slides.len() - 1;
    for (i, slide) in slides.iter().enumerate().skip(1) {
        let in_a = if i == 1 {
            "[0:v]".to_string()
        } else {
            format!("[v{}]", i - 1)
        };
        let in_b = format!("[{i}:v]");
        let out_lbl = if i == last {
            "[vlast]".to_string()
        } else {
            format!("[v{i}]")
        };
        let offset = (cum_dur - slide.trans_duration).max(0.0);
        filter.push_str(&format!(
            "{in_a}{in_b}xfade=transition={tr}:duration={td:.3}:offset={offset:.3}{out_lbl};",
            tr = slide.transition,
            td = slide.trans_duration,
        ));
        cum_dur = offset + slide.duration;
    }
    filter.push_str("[vlast]format=yuv420p[out]");

    let status = cmd
        .arg("-filter_complex")
        .arg(&filter)
        .arg("-map")
        .arg("[out]")
        .arg("-r")
        .arg("60")
        .arg("-c:v")
        .arg("libx264")
        .arg("-preset")
        .arg("slow")
        .arg("-crf")
        .arg("17")
        .arg("-movflags")
        .arg("+faststart")
        .arg(out_path)
        .status()?;

    if !status.success() {
        return Err(format!("ffmpeg exited with status {status}").into());
    }
    println!(
        "Encoded {} ({:.2}s, {} slides)",
        out_path.display(),
        cum_dur,
        slides.len()
    );
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = parse_args()?;
    let font = setup_font_family(args.lang, args.font.as_deref())?;

    let chart_slides = collect_chart_slides()?;
    let total_charts = chart_slides.len();

    let mut slides = Vec::with_capacity(total_charts + 2);
    slides.push(SlideSpec {
        duration: 2.40,
        transition: "fade",
        trans_duration: 0.0,
        svg: render_intro_slide(args.lang, &font),
    });

    for (idx, meta) in chart_slides.iter().enumerate() {
        let is_theme = meta.theme_badge.is_some();
        let trans_duration = if is_theme && meta.transition == "fade" {
            0.18
        } else {
            0.35
        };
        slides.push(SlideSpec {
            duration: if is_theme { 0.75 } else { 1.80 },
            transition: meta.transition,
            trans_duration,
            svg: render_chart_slide(meta, idx + 1, total_charts, args.lang, &font),
        });
    }

    slides.push(SlideSpec {
        duration: 2.80,
        transition: "fade",
        trans_duration: 0.35,
        svg: render_outro_slide(args.lang, &font),
    });

    let base_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("target")
        .join("showcase");
    let frames_dir = base_dir.join(match args.lang {
        Lang::En => "frames",
        Lang::Zh => "frames-zh",
    });
    fs::create_dir_all(&frames_dir)?;

    let mut frame_paths = Vec::with_capacity(slides.len());
    for (idx, slide) in slides.iter().enumerate() {
        let png = svg_to_png(&slide.svg)?;
        let path = frames_dir.join(format!("frame_{idx:02}.png"));
        fs::write(&path, png)?;
        frame_paths.push(path);
    }
    println!(
        "Rendered {} PNG frames into {}",
        frame_paths.len(),
        frames_dir.display()
    );

    if !args.frames_only {
        let out_path = args.out.unwrap_or_else(|| {
            base_dir.join(match args.lang {
                Lang::En => "charts-rs-showcase.mp4",
                Lang::Zh => "charts-rs-showcase-zh.mp4",
            })
        });
        encode_mp4(&slides, &frame_paths, &out_path)?;
    }

    Ok(())
}
