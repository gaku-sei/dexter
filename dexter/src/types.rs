use cli_table::{format::Justify, Table};
use dexter_core::{ChapterData, SearchData};

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
    #[table(title = "Volume")]
    volume: String,
    #[table(title = "Chapter")]
    chapter: String,
    #[table(title = "Language")]
    language: String,
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
    title: String,
}

impl From<(String, String)> for ImageLink {
    fn from((filename, title): (String, String)) -> Self {
        ImageLink { filename, title }
    }
}
