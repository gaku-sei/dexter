#![deny(clippy::all)]
#![deny(clippy::pedantic)]

use std::{
    env::current_dir,
    fs::{create_dir_all, OpenOptions},
    path::Path,
};

use anyhow::{anyhow, Error, Result};
use async_recursion::async_recursion;
use clap::Parser;
use cli_table::{print_stdout, WithTitle};
use dexter_core::{
    api::archive_download, ArchiveDownload as DexterArchiveDownload,
    GetChapter as DexterGetChapter, GetChapters as DexterGetChapters,
    GetImageLinks as DexterGetImageLinks, GetManga as DexterGetManga, Request,
    Search as DexterSearch,
};
use dialoguer::theme::ColorfulTheme;
use dialoguer::{Input, Select};
use eco_cbz::CbzReader;
use eco_viewer::run;
use indicatif::{ProgressBar, ProgressStyle};
use tokio::sync::mpsc;
use types::{Chapter, ImageLink};

use crate::args::{Args, Chapters, Download, ImageLinks, InteractiveSearch, Search, Subcommands};
use crate::types::Manga;

mod args;
mod types;

#[async_recursion]
async fn find_manga() -> Result<Manga> {
    let manga_title: String = Input::new().with_prompt("Manga title").interact_text()?;

    let search_response = DexterSearch::new(manga_title)
        .with_limit(10)
        .request()
        .await?;

    let mangas = search_response
        .data
        .into_iter()
        .map(Into::into)
        .collect::<Vec<Manga>>();

    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Select a manga")
        .items(&mangas)
        .default(0)
        .interact_opt()?;

    match selection {
        Some(selection) => mangas
            .into_iter()
            .nth(selection)
            .ok_or_else(|| anyhow!("{selection} index not found in manga list")),
        None => find_manga().await,
    }
}

#[async_recursion]
async fn find_chapter(manga: &Manga) -> Result<Chapter> {
    let chapter_number: String = Input::new().with_prompt("Chapter number").interact_text()?;

    let chapter_response = DexterGetChapters::new(&manga.id)
        .set_limit(10)
        .push_chapter(chapter_number)
        .request()
        .await?;

    let chapters = chapter_response
        .data
        .into_iter()
        .map(Into::into)
        .collect::<Vec<Chapter>>();

    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Select a chapter")
        .items(&chapters)
        .default(0)
        .interact_opt()?;

    match selection {
        Some(selection) => chapters
            .into_iter()
            .nth(selection)
            .ok_or_else(|| anyhow!("{selection} index not found in chapter list")),
        None => find_chapter(manga).await,
    }
}

async fn download(
    chapter_id: &str,
    filepath: &Path,
    max_download_retries: u32,
    open: bool,
) -> Result<()> {
    let (tx, mut rx) = mpsc::unbounded_channel();

    let progress_handle = tokio::spawn(async move {
        let mut bar = ProgressBar::new(0);

        while let Some(event) = rx.recv().await {
            match event {
                archive_download::Event::Init(len) => {
                    bar = ProgressBar::new((len * 2) as u64);

                    bar.set_style(
                        ProgressStyle::default_bar()
                            .template("[{elapsed_precise}] [{wide_bar}] {percent}%")
                            .map_err(|err| {
                                anyhow::anyhow!("couldn't set progress template: {err}")
                            })?,
                    );
                }
                archive_download::Event::Download | archive_download::Event::Zip => {
                    bar.inc(1);
                }
                archive_download::Event::Done => {
                    bar.finish();
                }
            }
        }

        Ok::<(), Error>(())
    });

    let cbz_writer_finished = DexterArchiveDownload::new(chapter_id)
        .set_max_download_retries(max_download_retries)
        .set_sender(tx)
        .request()
        .await?;

    let file = OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .open(filepath)?;

    cbz_writer_finished.write_to(&file)?;

    if open {
        let mut cbz = CbzReader::from_reader(file)?;

        run(&mut cbz)?;
    }

    progress_handle.await??;

    Ok(())
}

#[tokio::main]
#[allow(clippy::too_many_lines)]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let args = Args::parse();

    match args.command {
        Subcommands::InteractiveSearch(InteractiveSearch {
            manga_id,
            chapter_number,
            volume_number,
            accepts_default_filename,
            outdir,
            language,
            max_download_retries,
        }) => {
            let manga = match manga_id {
                Some(manga_id) => DexterGetManga::new(manga_id).request().await?.data.into(),
                None => find_manga().await?,
            };

            let chapter = match chapter_number {
                Some(chapter_number) => {
                    let mut chapter_response = DexterGetChapter::new(&manga.id, &chapter_number)
                        .with_language(&language)
                        .set_volume_number(volume_number)
                        .request()
                        .await?;

                    let Some(chapter) = chapter_response.data.pop() else {
                        panic!("chapter number {chapter_number} not found for manga {manga} and language {language}");
                    };

                    chapter.into()
                }
                None => find_chapter(&manga).await?,
            };

            let default_filename = sanitize_filename::sanitize(format!("{manga} - {chapter}.cbz"));

            let filename = if accepts_default_filename {
                default_filename
            } else {
                Input::new()
                    .with_prompt("Filename")
                    .with_initial_text(&default_filename)
                    .interact_text()?
            };

            let outdir = outdir.unwrap_or(current_dir()?);

            if !outdir.exists() {
                create_dir_all(&outdir)?;
            }

            let filepath = outdir.join(filename);

            download(&chapter.id, &filepath, max_download_retries, false).await?;

            println!("CBZ file created");
        }

        Subcommands::Search(Search { limit, title }) => {
            let search_response = DexterSearch::new(title).with_limit(limit).request().await?;

            let mangas = search_response
                .data
                .into_iter()
                .map(Into::into)
                .collect::<Vec<Manga>>();

            print_stdout(mangas.with_title())?;
        }
        Subcommands::Chapters(Chapters {
            limit,
            manga_id,
            chapters,
            volumes,
        }) => {
            let chapter_response = DexterGetChapters::new(manga_id)
                .set_limit(limit)
                .with_volumes(volumes)
                .with_chapters(chapters)
                .request()
                .await?;

            let chapters = chapter_response
                .data
                .into_iter()
                .map(Into::into)
                .collect::<Vec<Chapter>>();

            print_stdout(chapters.with_title())?;
        }
        Subcommands::ImageLinks(ImageLinks { chapter_id }) => {
            let image_links = DexterGetImageLinks::new(chapter_id).request().await?;

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
            outdir,
            max_download_retries,
        }) => {
            let outdir = outdir.unwrap_or(current_dir()?);

            if !outdir.exists() {
                create_dir_all(&outdir)?;
            }

            let filepath = outdir.join(filename);

            download(&chapter_id, &filepath, max_download_retries, open).await?;

            println!("CBZ file created");
        }
    }

    Ok(())
}
