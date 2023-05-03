pub use archive_download::ArchiveDownload;
use async_trait::async_trait;
pub use get_chapter::GetChapter;
pub use get_chapters::GetChapters;
pub use get_image_links::GetImageLinks;
pub use get_manga::GetManga;
use reqwest::IntoUrl;
use reqwest::Url;
pub use search::Search;
use serde::Deserialize;
use tracing::error;

use crate::Result;

pub mod archive_download;
pub mod get_chapter;
pub mod get_chapters;
pub mod get_image_links;
pub mod get_manga;
pub mod search;

/// Returns the base mangadex url
pub(super) fn base_url() -> Url {
    "https://api.mangadex.org/".parse().unwrap()
}

/// Send a get request to `url` and decode the json response as `T`
pub(super) async fn get_json<T: for<'de> Deserialize<'de>>(
    url: impl IntoUrl,
    context: &str,
) -> Result<T> {
    reqwest::get(url).await?.json().await.map_err(|err| {
        error!("error decoding {context}: {err}");
        err.into()
    })
}

#[async_trait]
pub trait Request {
    type Response;

    async fn request(self) -> Result<Self::Response>;
}
