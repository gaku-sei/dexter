use async_trait::async_trait;
use serde::Deserialize;

use crate::{Request, Result};

use super::{base_url, get_json};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Deserialize)]
pub struct Title {
    pub en: String,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Deserialize)]
pub struct Attributes {
    pub title: Title,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Deserialize)]
pub struct Data {
    pub id: String,
    pub attributes: Attributes,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Deserialize)]
pub struct Response {
    pub data: Data,
}

/// Get manga information for the given manga id.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct GetManga {
    manga_id: String,
}

impl GetManga {
    pub fn new(manga_id: impl Into<String>) -> Self {
        Self {
            manga_id: manga_id.into(),
        }
    }
}

#[async_trait]
impl Request for GetManga {
    type Response = Response;

    async fn request(self) -> Result<Self::Response> {
        let mut url = base_url();
        url.set_path(&format!("manga/{}", self.manga_id));
        get_json(url, "get_manga").await
    }
}
