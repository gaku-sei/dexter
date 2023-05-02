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
    pub attributes: Attributes,
    pub id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Deserialize)]
pub struct Response {
    pub data: Vec<Data>,
}

/// Search for a manga by its title
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Search {
    title: String,
    limit: Option<u32>,
}

impl Search {
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            limit: None,
        }
    }

    #[must_use]
    pub fn set_limit(mut self, limit: Option<u32>) -> Self {
        self.limit = limit;
        self
    }

    #[must_use]
    pub fn with_limit(mut self, limit: u32) -> Self {
        self.limit = Some(limit);
        self
    }
}

#[async_trait]
impl Request for Search {
    type Response = Response;

    async fn request(self) -> Result<Self::Response> {
        let mut url = base_url();
        url.set_path("manga");
        url.query_pairs_mut()
            .append_pair("title", &self.title)
            .append_pair("order[relevance]", "desc");
        if let Some(limit) = self.limit {
            url.query_pairs_mut()
                .append_pair("limit", &limit.to_string());
        }
        get_json(url, "search").await
    }
}
