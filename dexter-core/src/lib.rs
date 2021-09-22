use anyhow::Result;
use bytes::Bytes;
use futures::{stream, StreamExt};
use log::info;
use reqwest::Client;
use serde::Deserialize;
use std::io::{self, Cursor, Read, Seek, Write};
use tokio::sync::Mutex;
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

        query.append_pair("limit", limit.to_string().as_str());

        query.append_pair("order[chapter]", "desc");

        for chapter in chapters {
            query.append_pair("chapter[]", chapter.as_str());
        }

        for volume in volumes {
            query.append_pair("volume[]", volume.as_str());
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
pub struct ImageLinksData {
    pub attributes: ImageLinksAttributes,
}

#[derive(Debug, Deserialize)]
pub struct ImageLinksResponse {
    pub data: ImageLinksData,
}

#[derive(Debug)]
pub struct ImageLinkDescription {
    pub filename: String,
    pub url: String,
}

pub async fn get_image_links(chapter_id: &str) -> Result<Vec<ImageLinkDescription>> {
    let url = format!(
        "https://api.mangadex.org/chapter/{chapter_id}",
        chapter_id = chapter_id
    );

    let response = reqwest::get(url).await?;

    let image_links_response: ImageLinksResponse = response.json().await?;

    let hash = image_links_response.data.attributes.hash;

    let image_links = image_links_response
        .data
        .attributes
        .data
        .into_iter()
        .map(|image_filename| {
            let url = format!(
                "https://uploads.mangadex.org/data/{hash}/{image_filename}",
                hash = hash,
                image_filename = image_filename.as_str()
            );

            ImageLinkDescription {
                filename: image_filename,
                url,
            }
        })
        .collect();

    Ok(image_links)
}

pub async fn download_images(chapter_id: &str) -> Result<Cursor<Vec<u8>>> {
    let client = Client::new();

    let buffer = Cursor::new(Vec::new());

    let zip = Mutex::new(ZipWriter::new(buffer));

    let image_links = get_image_links(chapter_id).await?;

    let len = image_links.len();

    let all_images_bytes = stream::iter(image_links)
        .map(|ImageLinkDescription { filename, url }| {
            let client = client.clone();

            tokio::spawn(async move {
                info!("Downloading {}", url);

                let response = client.get(url).send().await?;

                let bytes = response.bytes().await?;

                Ok((filename, bytes)) as Result<(String, Bytes)>
            })
        })
        .buffered(len);

    all_images_bytes
        .for_each(|bytes| async {
            if let Ok(Ok((filename, bytes))) = bytes {
                let mut zip = zip.lock().await;

                zip.start_file(filename.as_str(), Default::default())
                    .unwrap();

                zip.write_all(bytes.as_ref()).unwrap();
            }
        })
        .await;

    let zip = zip.lock().await.finish()?;

    Ok(zip)
}

pub fn get_cbz_size<R>(reader: R) -> Result<usize>
where
    R: Read + Seek,
{
    let zip = zip::ZipArchive::new(reader)?;

    Ok(zip.len())
}

pub fn read_from_cbz_by_index<R>(reader: R, index: usize) -> Result<Vec<u8>>
where
    R: Read + Seek,
{
    let mut zip = zip::ZipArchive::new(reader)?;

    let mut file = zip.by_index(index)?;

    let mut writer = Cursor::new(Vec::new());

    io::copy(&mut file, &mut writer)?;

    Ok(writer.into_inner())
}
