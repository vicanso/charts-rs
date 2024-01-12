use once_cell::sync::OnceCell;
use resvg::usvg::{fontdb, Tree, TreeParsing, TreeTextToPath};
use snafu::{ResultExt, Snafu};
use std::io::Cursor;

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("Io {file}: {source}"))]
    Io {
        file: String,
        source: std::io::Error,
    },
    #[snafu(display("Image size is invalid, width: {width}, height: {height}"))]
    Size { width: u32, height: u32 },
    #[snafu(display("Error to parse: {source}"))]
    Parse { source: resvg::usvg::Error },
    #[snafu(display("Encode fail: {source}"))]
    Image { source: image::ImageError },
}
pub type Result<T, E = Error> = std::result::Result<T, E>;

pub(crate) fn get_or_init_fontdb(fonts: Option<Vec<&[u8]>>) -> &fontdb::Database {
    static GLOBAL_FONT_DB: OnceCell<fontdb::Database> = OnceCell::new();
    GLOBAL_FONT_DB.get_or_init(|| {
        let mut fontdb = fontdb::Database::new();
        if let Some(value) = fonts {
            for item in value.iter() {
                fontdb.load_font_data((*item).to_vec());
            }
        } else {
            fontdb.load_system_fonts();
        }
        fontdb
    })
}

fn save_image(svg: &str, format: image::ImageOutputFormat) -> Result<Vec<u8>> {
    let fontdb = get_or_init_fontdb(None);
    let mut tree = Tree::from_str(svg, &resvg::usvg::Options::default()).context(ParseSnafu {})?;
    tree.convert_text(fontdb);
    let rtree = resvg::Tree::from_usvg(&tree);
    let pixmap_size = rtree.size.to_int_size();
    let mut pixmap = resvg::tiny_skia::Pixmap::new(pixmap_size.width(), pixmap_size.height())
        .ok_or(Error::Size {
            width: pixmap_size.width(),
            height: pixmap_size.height(),
        })?;
    rtree.render(resvg::tiny_skia::Transform::default(), &mut pixmap.as_mut());

    let rgba_image =
        image::RgbaImage::from_raw(pixmap.width(), pixmap.height(), pixmap.data().to_vec())
            .unwrap();
    let mut buf = Cursor::new(vec![]);

    rgba_image.write_to(&mut buf, format).context(ImageSnafu)?;
    Ok(buf.into_inner())
}

/// Converts svg to png
pub fn svg_to_png(svg: &str) -> Result<Vec<u8>> {
    save_image(svg, image::ImageOutputFormat::Png)
}

/// Converts svg to jpeg
pub fn svg_to_jpeg(svg: &str) -> Result<Vec<u8>> {
    save_image(svg, image::ImageOutputFormat::Jpeg(80))
}

/// Converts svg to webp
pub fn svg_to_webp(svg: &str) -> Result<Vec<u8>> {
    save_image(svg, image::ImageOutputFormat::WebP)
}

/// Converts svg to avif
pub fn svg_to_avif(svg: &str) -> Result<Vec<u8>> {
    save_image(svg, image::ImageOutputFormat::Avif)
}
