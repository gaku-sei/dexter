use std::fmt::Display;

use cli_table::{format::Justify, Table};
use dexter_core::api::{get_chapter, get_chapters, get_image_links, get_manga, search};

fn display_otional_value<Value>(value: &Option<Value>) -> impl Display
where
    Value: Display,
{
    match value {
        None => String::from("-"),
        Some(value) => format!("{value}"),
    }
}

#[derive(Debug, Clone, Table)]
pub struct Manga {
    #[table(title = "Title")]
    title: String,
    #[table(title = "ID", justify = "Justify::Right")]
    pub id: String,
}

impl From<search::Data> for Manga {
    fn from(search::Data { attributes, id }: search::Data) -> Self {
        Manga {
            id,
            title: attributes.title.en,
        }
    }
}

impl From<get_manga::Data> for Manga {
    fn from(get_manga::Data { attributes, id }: get_manga::Data) -> Self {
        Manga {
            id,
            title: attributes.title.en,
        }
    }
}

impl Display for Manga {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.title)
    }
}

#[derive(Debug, Clone, Table)]
pub struct Chapter {
    #[table(title = "ID", justify = "Justify::Right")]
    pub id: String,
    #[table(title = "Title", display_fn = "display_otional_value")]
    title: Option<String>,
    #[table(title = "Volume", display_fn = "display_otional_value")]
    volume: Option<String>,
    #[allow(clippy::struct_field_names)]
    #[table(title = "Chapter", display_fn = "display_otional_value")]
    chapter: Option<String>,
    #[table(title = "Language", display_fn = "display_otional_value")]
    language: Option<String>,
}

impl From<get_chapter::Data> for Chapter {
    fn from(get_chapter::Data { attributes, id }: get_chapter::Data) -> Self {
        Chapter {
            id,
            title: attributes.title,
            volume: attributes.volume,
            chapter: attributes.chapter,
            language: attributes.translated_language,
        }
    }
}

impl From<get_chapters::Data> for Chapter {
    fn from(get_chapters::Data { attributes, id }: get_chapters::Data) -> Self {
        Chapter {
            id,
            title: attributes.title,
            volume: attributes.volume,
            chapter: attributes.chapter,
            language: attributes.translated_language,
        }
    }
}

impl Display for Chapter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(volume) = &self.volume {
            write!(f, "{volume:0>2} - ")?;
        }

        if let Some(chapter) = &self.chapter {
            write!(f, "{chapter:0>3} - ")?;
        }

        if let Some(title) = &self.title {
            write!(f, "{title}")?;
        }

        Ok(())
    }
}

#[derive(Debug, Clone, Table)]
pub struct ImageLink {
    #[table(title = "Filename")]
    filename: String,
    #[table(title = "URL")]
    url: String,
}

impl From<get_image_links::Description> for ImageLink {
    fn from(image_link_description: get_image_links::Description) -> Self {
        ImageLink {
            filename: image_link_description.filename,
            url: image_link_description.url,
        }
    }
}
