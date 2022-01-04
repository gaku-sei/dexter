use anyhow::Result;
use cbz_reader::run;
use clap::Parser;
use cli_table::{print_stdout, WithTitle};
use dexter_core::{
    download_images, get_cbz_size, get_chapters, get_image_links, search, ChapterResponse,
    SearchResponse,
};
use std::convert::TryFrom;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;
use types::{Chapter, ImageLink};

use crate::args::{Args, Chapters, Download, ImageLinks, Search, Subcommands};
use crate::types::Manga;

mod args;
mod types;

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();

    let args = Args::parse();

    match args.command {
        Subcommands::Search(Search { limit, title }) => {
            let SearchResponse { data } = search(title.as_str(), limit).await?;

            let mangas = data
                .into_iter()
                .map(|data| data.into())
                .collect::<Vec<Manga>>();

            print_stdout(mangas.with_title())?;
        }
        Subcommands::Chapters(Chapters {
            limit,
            manga_id,
            chapters,
            volumes,
        }) => {
            let ChapterResponse { data } =
                get_chapters(manga_id.as_str(), limit, volumes, chapters).await?;

            let chapters = data
                .into_iter()
                .map(|data| data.into())
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
        Subcommands::Download(Download {
            chapter_id,
            filename,
            open,
        }) => {
            let zip = download_images(chapter_id.as_str()).await?;

            let file_path = PathBuf::from(filename);

            let mut file = OpenOptions::new()
                .read(true)
                .write(true)
                .create(true)
                .open(file_path.as_path())?;

            file.write_all(zip.into_inner().as_ref())?;

            if open {
                let size = get_cbz_size(file)?;

                let size = i32::try_from(size)?;

                run(file_path, size)?;
            } else {
                println!("CBZ file created");
            }
        }
    }

    Ok(())
}
