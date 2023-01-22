#![deny(clippy::all)]
#![deny(clippy::pedantic)]

use std::{fmt::Display, io::Cursor, iter::IntoIterator, sync::Arc};

use anyhow::{anyhow, Error, Result};
use cbz::{CbzWrite, CbzWriter, CbzWriterFinished, CbzWriterInsertionBuilder};
use futures::{stream, StreamExt, TryStreamExt};
use log::{error, info};
use reqwest_middleware::ClientBuilder;
use reqwest_retry::{policies::ExponentialBackoff, RetryTransientMiddleware};
use serde::Deserialize;
use tokio::sync::{mpsc::Sender, Mutex};
use url::Url;

#[derive(Debug, Deserialize)]
pub struct SearchTitle {
    pub en: String,
}

#[derive(Debug, Deserialize)]
pub struct SearchAttributes {
    pub title: SearchTitle,
}

#[derive(Debug, Deserialize)]
pub struct SearchData {
    pub attributes: SearchAttributes,
    pub id: String,
}

#[derive(Debug, Deserialize)]
pub struct SearchResponse {
    pub data: Vec<SearchData>,
}

/// Search for a manga by its title
///
/// # Errors
///
/// Any network or request error will make this function fail.
pub async fn search(title: impl Display, limit: u16) -> Result<SearchResponse> {
    let url = format!(
        "https://api.mangadex.org/manga?title={title}&limit={limit}&order[relevance]=desc",
    );

    let response = reqwest::get(url).await?;

    let search_response = response.json().await?;

    Ok(search_response)
}

#[derive(Debug, Deserialize)]
pub struct MangaTitle {
    pub en: String,
}

#[derive(Debug, Deserialize)]
pub struct MangaAttributes {
    pub title: MangaTitle,
}

#[derive(Debug, Deserialize)]
pub struct MangaData {
    pub id: String,
    pub attributes: MangaAttributes,
}

#[derive(Debug, Deserialize)]
pub struct MangaResponse {
    pub data: MangaData,
}

/// Get manga information for the given manga id.
///
/// # Errors
///
/// Any network or request error will make this function fail.
pub async fn get_manga(manga_id: impl AsRef<str>) -> Result<MangaResponse> {
    let url = Url::parse(&format!(
        "https://api.mangadex.org/manga/{}",
        manga_id.as_ref()
    ))?;

    let response = reqwest::get(url).await?;

    let manga_response = response.json().await?;

    Ok(manga_response)
}

#[derive(Debug, Deserialize)]
pub struct ChaptersAttributes {
    pub volume: Option<String>,
    pub chapter: Option<String>,
    pub title: String,
    #[serde(rename = "translatedLanguage")]
    pub translated_language: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ChaptersData {
    pub id: String,
    pub attributes: ChaptersAttributes,
}

#[derive(Debug, Deserialize)]
pub struct ChaptersResponse {
    pub data: Vec<ChaptersData>,
}

/// Get all chapters for the given manga id. Optionally volumes and chapters can be provided.
///
/// # Errors
///
/// Any network or request error will make this function fail.
pub async fn get_chapters(
    manga_id: impl AsRef<str>,
    limit: u16,
    volumes: impl IntoIterator<Item = impl AsRef<str>>,
    chapters: impl IntoIterator<Item = impl AsRef<str>>,
) -> Result<ChaptersResponse> {
    let mut url = Url::parse("https://api.mangadex.org/chapter")?;

    {
        let mut query = url.query_pairs_mut();

        query.append_pair("manga", manga_id.as_ref());

        query.append_pair("limit", &limit.to_string());

        query.append_pair("order[chapter]", "desc");

        for chapter in chapters {
            query.append_pair("chapter[]", chapter.as_ref());
        }

        for volume in volumes {
            query.append_pair("volume[]", volume.as_ref());
        }

        query.finish();
    }

    let response = reqwest::get(url).await?;

    let chapters_response = response.json().await?;

    Ok(chapters_response)
}

#[derive(Debug, Deserialize)]
pub struct ChapterAttributes {
    pub volume: Option<String>,
    pub chapter: Option<String>,
    pub title: String,
    #[serde(rename = "translatedLanguage")]
    pub translated_language: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ChapterData {
    pub id: String,
    pub attributes: ChapterAttributes,
}

#[derive(Debug, Deserialize)]
pub struct ChapterResponse {
    pub data: Vec<ChapterData>,
}

/// Get one specific chapter given manga id and number.
///
/// # Errors
///
/// Any network or request error will make this function fail.
pub async fn get_chapter(
    manga_id: impl AsRef<str>,
    language: impl AsRef<str>,
    chapter_number: impl AsRef<str>,
    volume_number: Option<impl AsRef<str>>,
) -> Result<ChapterResponse> {
    let mut url = Url::parse(&format!(
        "https://api.mangadex.org/chapter?manga={}&chapter[]={}&translatedLanguage[]={}",
        manga_id.as_ref(),
        chapter_number.as_ref(),
        language.as_ref()
    ))?;

    if let Some(volume_number) = volume_number {
        let mut query = url.query_pairs_mut();

        query.append_pair("volume[]", volume_number.as_ref());
    };

    let response = reqwest::get(url).await?;

    let chapter_response = response.json().await?;

    Ok(chapter_response)
}

#[derive(Debug, Deserialize)]
pub struct ImageLinksAttributes {
    pub data: Vec<String>,
    pub hash: String,
}

#[derive(Debug, Deserialize)]
pub struct ImageLinksChapter {
    pub data: Vec<String>,
    pub hash: String,
}

#[derive(Debug, Deserialize)]
pub struct ImageLinksResponse {
    pub chapter: ImageLinksChapter,
    #[serde(rename = "baseUrl")]
    pub base_url: String,
}

#[derive(Debug)]
pub struct ImageLinkDescription {
    pub filename: String,
    pub url: String,
}

/// Get all image links for the given chapter id.
///
/// # Errors
///
/// Any network or request error will make this function fail.
pub async fn get_image_links(chapter_id: impl Display) -> Result<Vec<ImageLinkDescription>> {
    let url = format!("https://api.mangadex.org/at-home/server/{chapter_id}");

    let response = reqwest::get(url).await?;

    let image_links_response: ImageLinksResponse = response.json().await?;

    let base_url = image_links_response.base_url;

    let hash = image_links_response.chapter.hash;

    let image_links = image_links_response
        .chapter
        .data
        .into_iter()
        .map(|image_filename| {
            let url = format!("{base_url}/data/{hash}/{image_filename}");

            ImageLinkDescription {
                filename: image_filename,
                url,
            }
        })
        .collect();

    Ok(image_links)
}

#[derive(Debug)]
pub enum ImageDownloadEvent {
    Init(usize),
    Download,
    Zip,
    Done,
}

/// Downloads all images for a given chapter id, and create an archive containing all the downloaded images.
///
/// # Errors
///
/// Any network or request error will make this function fail.
///
/// Archive creation errors will also make this fail.
pub async fn download_images(
    chapter_id: impl Display,
    download_max_retries: u32,
    tx: Sender<ImageDownloadEvent>,
) -> Result<CbzWriterFinished<Cursor<Vec<u8>>>> {
    let tx = Arc::new(tx);

    let retry_policy = ExponentialBackoff::builder().build_with_max_retries(download_max_retries);
    let client = ClientBuilder::new(reqwest::Client::new())
        .with(RetryTransientMiddleware::new_with_policy(retry_policy))
        .build();

    let cbz_writer = Mutex::new(CbzWriter::default());

    let image_links = get_image_links(chapter_id).await?;

    let len = image_links.len();

    tx.send(ImageDownloadEvent::Init(len)).await?;

    let all_images_bytes = stream::iter(image_links)
        .map(|ImageLinkDescription { filename, url }| {
            let client = client.clone();

            let tx = Arc::clone(&tx);

            tokio::spawn(async move {
                info!("Downloading {url}");

                let response = client.get(url).send().await?;

                let bytes = response.bytes().await?;

                tx.send(ImageDownloadEvent::Download).await?;

                Ok::<_, Error>((filename, bytes))
            })
        })
        .buffered(len);

    all_images_bytes
        .map_err(|error| anyhow!("join handle error: {error}"))
        .try_for_each(|res| async {
            let (filename, bytes) = match res {
                Ok(ok) => ok,
                Err(err) => {
                    error!("impossible to pack image: {err:?}");

                    return Ok(());
                }
            };

            info!("Packing {filename}");

            let mut cbz_writer = cbz_writer.lock().await;

            let insertion = CbzWriterInsertionBuilder::from_filename(&filename)
                .set_bytes(bytes)
                .build()?;

            cbz_writer
                .insert(insertion)
                .map_err(|_| anyhow!("failed to write content to archive file {filename}"))?;

            tx.send(ImageDownloadEvent::Zip)
                .await
                .map_err(|_| anyhow!("failed to send message to channel"))?;

            Ok(())
        })
        .await?;

    let cbz_writer_finished = cbz_writer.into_inner().finish()?;

    tx.send(ImageDownloadEvent::Done).await?;

    Ok(cbz_writer_finished)
}
