use anyhow::Result;
use bytes::Bytes;
use futures::{stream, StreamExt};
use log::info;
use reqwest::Client;
use serde::Deserialize;
use std::{
    io::{self, Cursor, Read, Seek, Write},
    sync::Arc,
};
use tokio::sync::{mpsc::Sender, Mutex};
use url::Url;
use zip::ZipWriter;

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

pub async fn search(title: &str, limit: u16) -> Result<SearchResponse> {
    let url = format!(
        "https://api.mangadex.org/manga?title={title}&limit={limit}&order[relevance]=desc",
        title = title,
        limit = limit
    );

    let response = reqwest::get(url).await?;

    let search_response = response.json().await?;

    Ok(search_response)
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

pub async fn get_chapters(
    manga_id: &str,
    limit: u16,
    volumes: Vec<String>,
    chapters: Vec<String>,
) -> Result<ChapterResponse> {
    let mut url = Url::parse("https://api.mangadex.org/chapter")?;

    {
        let mut query = url.query_pairs_mut();

        query.append_pair("manga", manga_id);

        query.append_pair("limit", &limit.to_string());

        query.append_pair("order[chapter]", "desc");

        for chapter in chapters {
            query.append_pair("chapter[]", &chapter);
        }

        for volume in volumes {
            query.append_pair("volume[]", &volume);
        }

        query.finish();
    }

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

pub async fn get_image_links(chapter_id: &str) -> Result<Vec<ImageLinkDescription>> {
    let url = format!(
        "https://api.mangadex.org/at-home/server/{chapter_id}",
        chapter_id = chapter_id
    );

    let response = reqwest::get(url).await?;

    let image_links_response: ImageLinksResponse = response.json().await?;

    let base_url = image_links_response.base_url;

    let hash = image_links_response.chapter.hash;

    let image_links = image_links_response
        .chapter
        .data
        .into_iter()
        .map(|image_filename| {
            let url = format!("{}/data/{}/{}", base_url, hash, image_filename);

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

pub async fn download_images(
    chapter_id: &str,
    tx: Sender<ImageDownloadEvent>,
) -> Result<Cursor<Vec<u8>>> {
    let tx = Arc::new(tx);

    let client = Client::new();

    let buffer = Cursor::new(Vec::new());

    let zip = Mutex::new(ZipWriter::new(buffer));

    let image_links = get_image_links(chapter_id).await?;

    let len = image_links.len();

    tx.send(ImageDownloadEvent::Init(len)).await?;

    let all_images_bytes = stream::iter(image_links)
        .map(|ImageLinkDescription { filename, url }| {
            let client = client.clone();

            let tx = Arc::clone(&tx);

            tokio::spawn(async move {
                info!("Downloading {}", url);

                let response = client.get(url).send().await?;

                let bytes = response.bytes().await?;

                tx.send(ImageDownloadEvent::Download).await?;

                Ok((filename, bytes)) as Result<(String, Bytes)>
            })
        })
        .buffered(len);

    all_images_bytes
        .for_each(|bytes| async {
            if let Ok(Ok((filename, bytes))) = bytes {
                let mut zip = zip.lock().await;

                zip.start_file(&filename, Default::default()).unwrap();

                zip.write_all(bytes.as_ref()).unwrap();

                // TODO: Drop `unwrap`
                tx.send(ImageDownloadEvent::Zip).await.unwrap();
            }
        })
        .await;

    let zip = zip.lock().await.finish()?;

    tx.send(ImageDownloadEvent::Done).await?;

    Ok(zip)
}

pub fn get_reader_size<R>(reader: R) -> Result<usize>
where
    R: Read + Seek,
{
    let zip = zip::ZipArchive::new(reader)?;

    Ok(zip.len())
}

pub fn read_by_index<R>(reader: R, index: usize) -> Result<Vec<u8>>
where
    R: Read + Seek,
{
    let mut zip = zip::ZipArchive::new(reader)?;

    let mut file = zip.by_index(index)?;

    let mut writer = Cursor::new(Vec::new());

    io::copy(&mut file, &mut writer)?;

    Ok(writer.into_inner())
}
