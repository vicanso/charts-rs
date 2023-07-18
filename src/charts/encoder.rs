use once_cell::sync::OnceCell;
use resvg::usvg::{fontdb, Tree, TreeParsing, TreeTextToPath};
use snafu::{ResultExt, Snafu};

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
    Png { source: png::EncodingError },
}
pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Clone, Debug, Default)]
pub struct EncodeParams {
    pub font_file: String,
    pub font_dir: String,
    pub svg: String,
}
pub fn get_or_init_fontdb(fonts: Option<Vec<&[u8]>>) -> &fontdb::Database {
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

pub fn get_font_families() -> Vec<String> {
    let mut families = vec![];
    get_or_init_fontdb(None).faces().for_each(|item| {
        for (family, _) in item.families.iter() {
            if !families.contains(family) {
                families.push(family.clone());
            }
        }
    });
    families
}

/// Converts svg to png
pub fn svg_to_png(params: EncodeParams) -> Result<Vec<u8>, Error> {
    let fontdb = if params.font_file.is_empty() && params.font_dir.is_empty() {
        get_or_init_fontdb(None).to_owned()
    } else {
        let mut fontdb = fontdb::Database::new();
        if !params.font_file.is_empty() {
            fontdb.load_font_file(&params.font_file).context(IoSnafu {
                file: params.font_file.clone(),
            })?;
        }
        if !params.font_dir.is_empty() {
            fontdb.load_fonts_dir(&params.font_dir);
        }
        fontdb
    };

    let mut tree =
        Tree::from_str(&params.svg, &resvg::usvg::Options::default()).context(ParseSnafu {})?;
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
