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
use snafu::{ResultExt, Snafu};
use std::io::Cursor;
use std::sync::Arc;
use usvg::fontdb;

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("Io {file}: {source}"))]
    Io {
        file: String,
        source: std::io::Error,
    },
    #[snafu(display("Image size is invalid, width: {width}, height: {height}"))]
    Size { width: u32, height: u32 },
    #[snafu(display("Image from raw is fail, size:{size}"))]
    Raw { size: usize },
    #[snafu(display("Error to parse: {source}"))]
    Parse { source: usvg::Error },
    #[snafu(display("Encode fail: {source}"))]
    Image { source: image::ImageError },
}
pub type Result<T, E = Error> = std::result::Result<T, E>;

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

fn save_image(svg: &str, format: image::ImageFormat) -> Result<Vec<u8>> {
    let fontdb = get_or_init_fontdb(None);
    let tree = usvg::Tree::from_str(
        svg,
        &usvg::Options {
            fontdb,
            ..Default::default()
        },
    )
    .context(ParseSnafu {})?;
    let pixmap_size = tree.size().to_int_size();
    let mut pixmap =
        tiny_skia::Pixmap::new(pixmap_size.width(), pixmap_size.height()).ok_or(Error::Size {
            width: pixmap_size.width(),
            height: pixmap_size.height(),
        })?;
    resvg::render(&tree, tiny_skia::Transform::default(), &mut pixmap.as_mut());

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

/// Converts svg to png.
pub fn svg_to_png(svg: &str) -> Result<Vec<u8>> {
    save_image(svg, image::ImageFormat::Png)
}

/// Converts svg to jpeg, the quality is 80.
pub fn svg_to_jpeg(svg: &str) -> Result<Vec<u8>> {
    save_image(svg, image::ImageFormat::Jpeg)
}

/// Converts svg to webp.
pub fn svg_to_webp(svg: &str) -> Result<Vec<u8>> {
    save_image(svg, image::ImageFormat::WebP)
}

/// Converts svg to avif.
pub fn svg_to_avif(svg: &str) -> Result<Vec<u8>> {
    save_image(svg, image::ImageFormat::Avif)
}
