// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use image::ImageFormat;
use once_cell::sync::OnceCell;
use resvg::{tiny_skia, usvg};
use snafu::ResultExt;
use std::io::Cursor;
use std::sync::Arc;
use usvg::fontdb;

// Crate-level error/result (see `error.rs`); re-exported to keep
// `encoder::Error` / `encoder::Result` paths working.
pub use super::error::{Error, Result};
use super::error::{ImageSnafu, ParseSnafu};

pub(crate) fn get_or_init_fontdb(fonts: Option<Vec<&[u8]>>) -> Arc<fontdb::Database> {
    static GLOBAL_FONT_DB: OnceCell<Arc<fontdb::Database>> = OnceCell::new();
    GLOBAL_FONT_DB
        .get_or_init(|| {
            let mut fontdb = fontdb::Database::new();
            if let Some(value) = fonts {
                for item in value.iter() {
                    fontdb.load_font_data((*item).to_vec());
                }
            } else {
                fontdb.load_system_fonts();
            }
            Arc::new(fontdb)
        })
        .clone()
}

fn parse_tree(svg: &str) -> Result<usvg::Tree> {
    let fontdb = get_or_init_fontdb(None);
    usvg::Tree::from_str(
        svg,
        &usvg::Options {
            fontdb,
            ..Default::default()
        },
    )
    .context(ParseSnafu {})
}

fn render_to_pixmap(
    tree: &usvg::Tree,
    target_width: Option<u32>,
    target_height: Option<u32>,
) -> Result<tiny_skia::Pixmap> {
    let svg_size = tree.size().to_int_size();
    let svg_w = svg_size.width();
    let svg_h = svg_size.height();
    if svg_w == 0 || svg_h == 0 {
        return Err(Error::Size {
            width: svg_w,
            height: svg_h,
        });
    }
    let (out_w, out_h, sx, sy) = match (target_width, target_height) {
        (Some(w), Some(h)) => (w, h, w as f32 / svg_w as f32, h as f32 / svg_h as f32),
        (Some(w), None) => {
            let sx = w as f32 / svg_w as f32;
            let h = (svg_h as f32 * sx).round() as u32;
            (w, h, sx, sx)
        }
        (None, Some(h)) => {
            let sy = h as f32 / svg_h as f32;
            let w = (svg_w as f32 * sy).round() as u32;
            (w, h, sy, sy)
        }
        (None, None) => (svg_w, svg_h, 1.0, 1.0),
    };
    let mut pixmap = tiny_skia::Pixmap::new(out_w, out_h).ok_or(Error::Size {
        width: out_w,
        height: out_h,
    })?;
    resvg::render(
        tree,
        tiny_skia::Transform::from_scale(sx, sy),
        &mut pixmap.as_mut(),
    );
    Ok(pixmap)
}

fn encode_pixmap(pixmap: tiny_skia::Pixmap, format: image::ImageFormat) -> Result<Vec<u8>> {
    let data = pixmap.data().to_vec();
    let size = data.len();
    let rgba_image = image::RgbaImage::from_raw(pixmap.width(), pixmap.height(), data)
        .ok_or(Error::Raw { size })?;
    let mut buf = Cursor::new(vec![]);
    if format == ImageFormat::Jpeg {
        image::DynamicImage::ImageRgba8(rgba_image)
            .to_rgb8()
            .write_to(&mut buf, format)
            .context(ImageSnafu)?;
    } else {
        rgba_image.write_to(&mut buf, format).context(ImageSnafu)?;
    }
    Ok(buf.into_inner())
}

fn save_image(svg: &str, format: image::ImageFormat) -> Result<Vec<u8>> {
    let tree = parse_tree(svg)?;
    let pixmap = render_to_pixmap(&tree, None, None)?;
    encode_pixmap(pixmap, format)
}

fn save_image_with_size(
    svg: &str,
    format: image::ImageFormat,
    width: Option<u32>,
    height: Option<u32>,
) -> Result<Vec<u8>> {
    let tree = parse_tree(svg)?;
    let pixmap = render_to_pixmap(&tree, width, height)?;
    encode_pixmap(pixmap, format)
}

/// Converts svg to png.
pub fn svg_to_png(svg: &str) -> Result<Vec<u8>> {
    save_image(svg, image::ImageFormat::Png)
}

/// Converts svg to png, scaling to the given width and/or height.
/// If only one dimension is provided the other is computed to preserve aspect ratio.
pub fn svg_to_png_with_size(svg: &str, width: Option<u32>, height: Option<u32>) -> Result<Vec<u8>> {
    save_image_with_size(svg, image::ImageFormat::Png, width, height)
}

/// Converts svg to jpeg.
pub fn svg_to_jpeg(svg: &str) -> Result<Vec<u8>> {
    save_image(svg, image::ImageFormat::Jpeg)
}

/// Converts svg to jpeg, scaling to the given width and/or height.
pub fn svg_to_jpeg_with_size(
    svg: &str,
    width: Option<u32>,
    height: Option<u32>,
) -> Result<Vec<u8>> {
    save_image_with_size(svg, image::ImageFormat::Jpeg, width, height)
}

/// Converts svg to webp.
pub fn svg_to_webp(svg: &str) -> Result<Vec<u8>> {
    save_image(svg, image::ImageFormat::WebP)
}

/// Converts svg to webp, scaling to the given width and/or height.
pub fn svg_to_webp_with_size(
    svg: &str,
    width: Option<u32>,
    height: Option<u32>,
) -> Result<Vec<u8>> {
    save_image_with_size(svg, image::ImageFormat::WebP, width, height)
}

/// Converts svg to avif.
pub fn svg_to_avif(svg: &str) -> Result<Vec<u8>> {
    save_image(svg, image::ImageFormat::Avif)
}

/// Converts svg to avif, scaling to the given width and/or height.
pub fn svg_to_avif_with_size(
    svg: &str,
    width: Option<u32>,
    height: Option<u32>,
) -> Result<Vec<u8>> {
    save_image_with_size(svg, image::ImageFormat::Avif, width, height)
}
