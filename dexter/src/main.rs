use anyhow::Result;
use async_recursion::async_recursion;
use cbz_reader::run;
use clap::Parser;
use cli_table::{print_stdout, WithTitle};
use dexter_core::{
    download_images, get_chapters, get_image_links, get_reader_size, search, ChapterResponse,
    ImageDownloadEvent, SearchResponse,
};
use dialoguer::theme::ColorfulTheme;
use dialoguer::{Input, Select};
use indicatif::{ProgressBar, ProgressStyle};
use std::convert::TryFrom;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;
use tokio::sync::mpsc;
use types::{Chapter, ImageLink};

use crate::args::{Args, Chapters, Download, ImageLinks, Search, Subcommands};
use crate::types::Manga;

mod args;
mod types;

#[async_recursion]
async fn find_manga() -> Result<Manga> {
    let manga_title: String = Input::new().with_prompt("Manga title").interact_text()?;

    let SearchResponse { data } = search(&manga_title, 10).await?;

    let mangas = data
        .into_iter()
        .map(|data| data.into())
        .collect::<Vec<Manga>>();

    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Select a manga")
        .items(&mangas)
        .default(0)
        .interact_opt()?;

    if let Some(selection) = selection {
        Ok(mangas[selection].clone())
    } else {
        find_manga().await
    }
}

#[async_recursion]
async fn find_chapter(manga: &Manga) -> Result<Chapter> {
    let chapter_number: String = Input::new().with_prompt("Chapter number").interact_text()?;

    let ChapterResponse { data } =
        get_chapters(&manga.id, 10, Vec::new(), vec![chapter_number]).await?;

    let chapters = data
        .into_iter()
        .map(|data| data.into())
        .collect::<Vec<Chapter>>();

    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Select a chapter")
        .items(&chapters)
        .default(0)
        .interact_opt()?;

    if let Some(selection) = selection {
        Ok(chapters[selection].clone())
    } else {
        find_chapter(manga).await
    }
}

async fn download(chapter_id: &str, filename: &str, open: bool) -> Result<()> {
    let (tx, mut rx) = mpsc::channel(1);

    tokio::spawn(async move {
        let mut bar = ProgressBar::new(0);

        while let Some(event) = rx.recv().await {
            match event {
                ImageDownloadEvent::Init(len) => {
                    bar = ProgressBar::new((len * 2) as u64);

                    bar.set_style(
                        ProgressStyle::default_bar()
                            .template("[{elapsed_precise}] [{wide_bar}] {percent}%"),
                    );
                }
                ImageDownloadEvent::Download | ImageDownloadEvent::Zip => {
                    bar.inc(1);
                }
                ImageDownloadEvent::Done => {
                    bar.finish();
                }
            }
        }
    });

    let zip = download_images(chapter_id, tx).await?;

    let file_path = PathBuf::from(filename);

    let mut file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open(file_path.as_path())?;

    file.write_all(zip.into_inner().as_ref())?;

    if open {
        let size = get_reader_size(file)?;

        let size = i32::try_from(size)?;

        run(file_path, size)?;
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();

    let args = Args::parse();

    match args.command {
        Subcommands::InteractiveSearch => {
            let manga = find_manga().await?;

            let chapter = find_chapter(&manga).await?;

            let filename: String = Input::new()
                .with_prompt("Filename")
                .with_initial_text(&format!(
                    "{} - {}.cbz",
                    manga.to_string(),
                    chapter.to_string()
                ))
                .interact_text()?;

            download(&chapter.id, &filename, false).await?;

            println!("CBZ file created");
        }

        Subcommands::Search(Search { limit, title }) => {
            let SearchResponse { data } = search(&title, limit).await?;

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
                get_chapters(&manga_id, limit, volumes, chapters).await?;

            let chapters = data
                .into_iter()
                .map(|data| data.into())
                .collect::<Vec<Chapter>>();

            print_stdout(chapters.with_title())?;
        }
        Subcommands::ImageLinks(ImageLinks { chapter_id }) => {
            let image_links = get_image_links(&chapter_id).await?;

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
            download(&chapter_id, &filename, open).await?;

            println!("CBZ file created");
        }
    }

    Ok(())
}
