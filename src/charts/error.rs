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

use snafu::Snafu;

#[derive(Debug, Snafu)]
#[snafu(visibility(pub(crate)))]
pub enum Error {
    #[snafu(display("Params is invalid: {message}"))]
    Params { message: String },
    #[snafu(display("Json is invalid: {source}"))]
    Json { source: serde_json::Error },
    #[snafu(display("Error font: {name} not found"))]
    FontNotFound { name: String },
    #[snafu(display("Error parse font: {message}"))]
    ParseFont { message: String },

    // Raster encoding (image-encoder feature); the source types live behind
    // the optional `resvg` / `image` dependencies, so the variants are gated.
    #[cfg(feature = "image-encoder")]
    #[snafu(display("Io {file}: {source}"))]
    Io {
        file: String,
        source: std::io::Error,
    },
    #[cfg(feature = "image-encoder")]
    #[snafu(display("Image size is invalid, width: {width}, height: {height}"))]
    Size { width: u32, height: u32 },
    #[cfg(feature = "image-encoder")]
    #[snafu(display("Image from raw is fail, size:{size}"))]
    Raw { size: usize },
    #[cfg(feature = "image-encoder")]
    #[snafu(display("Error to parse: {source}"))]
    Parse { source: resvg::usvg::Error },
    #[cfg(feature = "image-encoder")]
    #[snafu(display("Encode fail: {source}"))]
    Image { source: image::ImageError },
}

impl From<serde_json::Error> for Error {
    fn from(value: serde_json::Error) -> Self {
        Error::Json { source: value }
    }
}

impl From<&str> for Error {
    fn from(value: &str) -> Self {
        Error::ParseFont {
            message: value.to_string(),
        }
    }
}

pub type Result<T, E = Error> = std::result::Result<T, E>;
