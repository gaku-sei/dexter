use std::iter::IntoIterator;

use async_trait::async_trait;
use serde::Deserialize;

use crate::{Request, Result};

use super::{base_url, get_json};

pub static DEFAULT_CHAPTERS_LIMIT: u32 = 100;

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
    pub limit: u32,
    pub offset: u32,
    pub total: u32,
    pub data: Vec<Data>,
}

/// Get all chapters for the given manga id. Optionally volumes and chapters can be provided.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct GetChapters {
    manga_id: String,
    limit: u32,
    offset: u32,
    chapters: Option<Vec<String>>,
    volumes: Option<Vec<String>>,
    languages: Option<Vec<String>>,
}

impl GetChapters {
    pub fn new(manga_id: impl Into<String>) -> Self {
        Self {
            manga_id: manga_id.into(),
            limit: DEFAULT_CHAPTERS_LIMIT,
            offset: 0,
            chapters: None,
            volumes: None,
            languages: None,
        }
    }

    #[must_use]
    pub fn set_limit(mut self, limit: u32) -> Self {
        self.limit = limit;
        self
    }

    #[must_use]
    pub fn set_offset(mut self, offset: u32) -> Self {
        self.offset = offset;
        self
    }

    #[must_use]
    pub fn set_chapters(mut self, chapters: Option<Vec<String>>) -> Self {
        self.chapters = chapters;
        self
    }

    #[must_use]
    pub fn with_chapters(mut self, chapters: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.chapters = Some(chapters.into_iter().map(Into::into).collect());
        self
    }

    #[must_use]
    pub fn push_chapter(mut self, chapter: impl Into<String>) -> Self {
        let chapter = chapter.into();
        match &mut self.chapters {
            Some(chapters) => chapters.push(chapter),
            None => self.chapters = Some(vec![chapter]),
        };
        self
    }

    #[must_use]
    pub fn set_volumes(mut self, volumes: Option<Vec<String>>) -> Self {
        self.volumes = volumes;
        self
    }

    #[must_use]
    pub fn with_volumes(mut self, volumes: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.volumes = Some(volumes.into_iter().map(Into::into).collect());
        self
    }

    #[must_use]
    pub fn push_volume(mut self, volume: impl Into<String>) -> Self {
        let volume = volume.into();
        match &mut self.volumes {
            Some(volumes) => volumes.push(volume),
            None => self.volumes = Some(vec![volume]),
        };
        self
    }

    #[must_use]
    pub fn set_languages(mut self, languages: Option<Vec<String>>) -> Self {
        self.languages = languages;
        self
    }

    #[must_use]
    pub fn with_languages(
        mut self,
        languages: impl IntoIterator<Item = impl Into<String>>,
    ) -> Self {
        self.languages = Some(languages.into_iter().map(Into::into).collect());
        self
    }

    #[must_use]
    pub fn push_language(mut self, language: impl Into<String>) -> Self {
        let language = language.into();
        match &mut self.languages {
            Some(languages) => languages.push(language),
            None => self.languages = Some(vec![language]),
        };
        self
    }
}

#[async_trait]
impl Request for GetChapters {
    type Response = Response;

    async fn request(mut self) -> Result<Self::Response> {
        let mut url = base_url();
        url.set_path("chapter");
        url.query_pairs_mut()
            .append_pair("manga", &self.manga_id)
            .append_pair("limit", &self.limit.to_string())
            .append_pair("order[chapter]", "desc");
        if self.offset > 0 {
            url.query_pairs_mut()
                .append_pair("offset", &self.offset.to_string());
        }
        if let Some(chapters) = &self.chapters {
            for chapter in chapters {
                url.query_pairs_mut().append_pair("chapter[]", chapter);
            }
        }
        if let Some(languages) = &self.languages {
            for language in languages {
                url.query_pairs_mut()
                    .append_pair("translatedLanguage[]", language);
            }
        }
        if let Some(volumes) = &self.volumes {
            for volume in volumes {
                url.query_pairs_mut().append_pair("volume[]", volume);
            }
        }
        get_json(url, "get_chapters").await
    }
}
