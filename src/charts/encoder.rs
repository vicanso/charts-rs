use resvg::usvg::{TreeParsing, TreeTextToPath};
use snafu::{ResultExt, Snafu};

#[derive(Debug, Snafu)]
pub enum Error {
    #[snafu(display("Image size is invalid, width: {width}, height: {height}"))]
    Size { width: u32, height: u32 },
    #[snafu(display("Error to parse: {source}"))]
    Parse { source: resvg::usvg::Error },
    #[snafu(display("Encode fail: {source}"))]
    Png { source: png::EncodingError },
}

pub fn svg_to_png(data: &str) -> Result<Vec<u8>, Error> {
    let mut fontdb = resvg::usvg::fontdb::Database::new();
    fontdb.load_system_fonts();
    let mut tree = resvg::usvg::Tree::from_str(data, &resvg::usvg::Options::default())
        .context(ParseSnafu {})?;
    tree.convert_text(&fontdb);
    let rtree = resvg::Tree::from_usvg(&tree);
    let pixmap_size = rtree.size.to_int_size();
    let mut pixmap = resvg::tiny_skia::Pixmap::new(pixmap_size.width(), pixmap_size.height())
        .ok_or(Error::Size {
            width: pixmap_size.width(),
            height: pixmap_size.height(),
        })?;
    rtree.render(resvg::tiny_skia::Transform::default(), &mut pixmap.as_mut());
    pixmap.encode_png().context(PngSnafu {})
}
