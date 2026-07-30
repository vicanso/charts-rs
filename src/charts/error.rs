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

//! The single crate-level error type. It replaces what used to be four
//! separate per-module enums (`canvas`, `font`, `component`, `encoder`), so
//! callers match one type and `?` composes across the whole crate. Each of
//! those modules now re-exports `Error`/`Result` from here, keeping the old
//! `canvas::Error`, `font::Error`, … paths valid.

use std::fmt;

#[derive(Debug)]
// Non-exhaustive so future variants (and the `image-encoder`-gated ones that
// appear/disappear with that feature) are not a breaking change for downstream
// `match` arms; callers must include a wildcard arm.
#[non_exhaustive]
pub enum Error {
    Params {
        message: String,
    },
    Json {
        source: serde_json::Error,
    },
    FontNotFound {
        name: String,
    },
    ParseFont {
        message: String,
    },

    // Raster encoding (image-encoder feature); the source types live behind
    // the optional `resvg` / `image` dependencies, so the variants are gated.
    #[cfg(feature = "raster")]
    Size {
        width: u32,
        height: u32,
    },
    #[cfg(feature = "raster")]
    Raw {
        size: usize,
    },
    #[cfg(feature = "raster")]
    Parse {
        source: resvg::usvg::Error,
    },
    #[cfg(feature = "raster")]
    Image {
        source: image::ImageError,
    },
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Params { message } => write!(f, "Params is invalid: {message}"),
            Error::Json { source } => write!(f, "Json is invalid: {source}"),
            Error::FontNotFound { name } => write!(f, "Error font: {name} not found"),
            Error::ParseFont { message } => write!(f, "Error parse font: {message}"),
            #[cfg(feature = "raster")]
            Error::Size { width, height } => {
                write!(f, "Image size is invalid, width: {width}, height: {height}")
            }
            #[cfg(feature = "raster")]
            Error::Raw { size } => write!(f, "Image from raw is fail, size:{size}"),
            #[cfg(feature = "raster")]
            Error::Parse { source } => write!(f, "Error to parse: {source}"),
            #[cfg(feature = "raster")]
            Error::Image { source } => write!(f, "Encode fail: {source}"),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::Json { source } => Some(source),
            #[cfg(feature = "raster")]
            Error::Parse { source } => Some(source),
            #[cfg(feature = "raster")]
            Error::Image { source } => Some(source),
            _ => None,
        }
    }
}

impl From<serde_json::Error> for Error {
    fn from(value: serde_json::Error) -> Self {
        Error::Json { source: value }
    }
}

pub type Result<T, E = Error> = std::result::Result<T, E>;
