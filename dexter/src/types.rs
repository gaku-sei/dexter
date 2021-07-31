use std::fmt::Display;

use cli_table::{format::Justify, Table};
use dexter_core::{ChapterData, ImageLinkDescription, SearchData};

fn display_otional_value<Value>(value: &Option<Value>) -> impl Display
where
    Value: Display,
{
    match value {
        None => "-".to_string(),
        Some(value) => format!("{}", value),
    }
}

#[derive(Table)]
pub struct Manga {
    #[table(title = "Title")]
    title: String,
    #[table(title = "ID", justify = "Justify::Right")]
    id: String,
}

impl From<SearchData> for Manga {
    fn from(SearchData { attributes, id }: SearchData) -> Self {
        Manga {
            id,
            title: attributes.title.en,
        }
    }
}

#[derive(Table)]
pub struct Chapter {
    #[table(title = "Title")]
    title: String,
    #[table(title = "ID", justify = "Justify::Right")]
    id: String,
    #[table(title = "Volume", display_fn = "display_otional_value")]
    volume: Option<String>,
    #[table(title = "Chapter", display_fn = "display_otional_value")]
    chapter: Option<String>,
    #[table(title = "Language", display_fn = "display_otional_value")]
    language: Option<String>,
}

impl From<ChapterData> for Chapter {
    fn from(ChapterData { attributes, id }: ChapterData) -> Self {
        Chapter {
            id,
            title: attributes.title,
            volume: attributes.volume,
            chapter: attributes.chapter,
            language: attributes.translated_language,
        }
    }
}

#[derive(Table)]
pub struct ImageLink {
    #[table(title = "Filename")]
    filename: String,
    #[table(title = "URL")]
    url: String,
}

impl From<ImageLinkDescription> for ImageLink {
    fn from(image_link_description: ImageLinkDescription) -> Self {
        ImageLink {
            filename: image_link_description.filename,
            url: image_link_description.url,
        }
    }
}
