use anyhow::Result;
use clap::Clap;
use cli_table::{print_stdout, WithTitle};
use dexter_core::{
    download_images, get_chapters, get_image_links, search, ChapterResponse, ChapterResult,
    SearchResponse, SearchResult,
};
use std::fs::File;
use std::io::Write;
use types::{Chapter, ImageLink};

use crate::options::{Chapters, Download, ImageLinks, Options, Search, Subcommands};
use crate::types::Manga;

mod options;
mod types;

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();

    let options = Options::parse();

    match options.command {
        Subcommands::Search(Search { limit, title }) => {
            let SearchResponse { results } = search(title.as_str(), limit).await?;

            let mangas = results
                .into_iter()
                .map(|SearchResult { data }| data.into())
                .collect::<Vec<Manga>>();

            print_stdout(mangas.with_title())?;
        }
        Subcommands::Chapters(Chapters {
            limit,
            manga_id,
            chapters,
            volumes,
        }) => {
            let ChapterResponse { results } =
                get_chapters(manga_id.as_str(), limit, volumes, chapters).await?;

            let chapters = results
                .into_iter()
                .map(|ChapterResult { data }: ChapterResult| data.into())
                .collect::<Vec<Chapter>>();

            print_stdout(chapters.with_title())?;
        }
        Subcommands::ImageLinks(ImageLinks { chapter_id }) => {
            let image_links = get_image_links(chapter_id.as_str()).await?;

            let image_links = image_links
                .into_iter()
                .map(ImageLink::from)
                .collect::<Vec<ImageLink>>();

            print_stdout(image_links.with_title())?;
        }
        Subcommands::Download(Download { chapter_id, output }) => {
            let zip = download_images(chapter_id.as_str()).await?;

            let mut file = File::create(output)?;

            file.write_all(zip.into_inner().as_ref())?;

            println!("CBZ file created");
        }
    }

    Ok(())
}
