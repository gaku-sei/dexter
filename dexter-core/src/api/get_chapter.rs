use async_trait::async_trait;
use serde::Deserialize;

use crate::{Request, Result};

use super::{base_url, get_json};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Deserialize)]
pub struct Attributes {
    pub volume: Option<String>,
    pub chapter: Option<String>,
    pub title: Option<String>,
    #[serde(rename = "translatedLanguage")]
    pub translated_language: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Deserialize)]
pub struct Data {
    pub id: String,
    pub attributes: Attributes,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Deserialize)]
pub struct Response {
    pub data: Vec<Data>,
}

/// Get one specific chapter given manga id and number.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct GetChapter {
    manga_id: String,
    chapter_number: String,
    language: Option<String>,
    volume_number: Option<String>,
}

impl GetChapter {
    pub fn new(manga_id: impl Into<String>, chapter_number: impl Into<String>) -> Self {
        Self {
            manga_id: manga_id.into(),
            chapter_number: chapter_number.into(),
            language: None,
            volume_number: None,
        }
    }

    #[must_use]
    pub fn set_language(mut self, language: Option<String>) -> Self {
        self.language = language;
        self
    }

    #[must_use]
    pub fn with_language(mut self, language: impl Into<String>) -> Self {
        self.language = Some(language.into());
        self
    }

    #[must_use]
    pub fn set_volume_number(mut self, volume_number: Option<String>) -> Self {
        self.volume_number = volume_number;
        self
    }

    #[must_use]
    pub fn with_volume_number(mut self, volume_number: impl Into<String>) -> Self {
        self.volume_number = Some(volume_number.into());
        self
    }
}

#[async_trait]
impl Request for GetChapter {
    type Response = Response;

    async fn request(mut self) -> Result<Self::Response> {
        let mut url = base_url();
        url.set_path("chapter");
        url.query_pairs_mut()
            .append_pair("manga", &self.manga_id)
            .append_pair("chapter[]", &self.chapter_number);
        if let Some(language) = &self.language {
            url.query_pairs_mut()
                .append_pair("translatedLanguage[]", language);
        };
        if let Some(volume_number) = &self.volume_number {
            url.query_pairs_mut().append_pair("volume[]", volume_number);
        };
        get_json(url, "get_chapter").await
    }
}
