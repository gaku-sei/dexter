use async_trait::async_trait;
use serde::Deserialize;

use crate::{Request, Result};

use super::{base_url, get_json};

// #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Deserialize)]
// pub struct Attributes {
//     pub data: Vec<String>,
//     pub hash: String,
// }

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Deserialize)]
struct Chapter {
    data: Vec<String>,
    hash: String,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Deserialize)]
struct ImageLinks {
    chapter: Chapter,
    #[serde(rename = "baseUrl")]
    base_url: String,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Deserialize)]
pub struct Description {
    pub filename: String,
    pub url: String,
}

type Response = Vec<Description>;

/// Get all image links for the given chapter id.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct GetImageLinks {
    chapter_id: String,
}

impl GetImageLinks {
    pub fn new(chapter_id: impl Into<String>) -> Self {
        Self {
            chapter_id: chapter_id.into(),
        }
    }
}

#[async_trait]
impl Request for GetImageLinks {
    type Response = Response;

    async fn request(self) -> Result<Response> {
        let mut url = base_url();
        url.set_path(&format!("at-home/server/{}", self.chapter_id));
        let image_links = get_json::<ImageLinks>(url, "get_image_links").await?;
        Ok(image_links
            .chapter
            .data
            .into_iter()
            .map(|image_filename| {
                let url = format!(
                    "{}/data/{}/{image_filename}",
                    image_links.base_url, image_links.chapter.hash
                );

                Description {
                    filename: image_filename,
                    url,
                }
            })
            .collect())
    }
}
