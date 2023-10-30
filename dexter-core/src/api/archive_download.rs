use std::{io::Cursor, marker::PhantomData};

use async_trait::async_trait;
use camino::Utf8Path;
use eco_cbz::CbzWriter;
use futures::{stream, StreamExt, TryStreamExt};
use reqwest_middleware::ClientBuilder;
use reqwest_retry::{policies::ExponentialBackoff, RetryTransientMiddleware};
use tokio::sync::{mpsc, Mutex};
use tracing::{error, info};

use crate::{Error, GetImageLinks, Request, Result};

pub static DEFAULT_MAX_PARALLEL_DOWNLOAD: usize = 10;
pub static DEFAULT_MAX_DOWNLOAD_RETRIES: u32 = 10;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Event {
    Init(usize),
    Download,
    Zip,
    Done,
}

/// Downloads all images for a given chapter id, and create an archive containing all the downloaded images.
#[derive(Debug, Clone)]
pub struct ArchiveDownload<'a> {
    chapter_id: String,
    max_parallel_download: usize,
    max_download_retries: u32,
    sender: mpsc::UnboundedSender<Event>,
    _lifetime: PhantomData<&'a ()>,
}

impl<'a> ArchiveDownload<'a> {
    pub fn new(chapter_id: impl Into<String>) -> Self {
        let (tx, _rx) = mpsc::unbounded_channel();

        Self {
            chapter_id: chapter_id.into(),
            max_parallel_download: DEFAULT_MAX_PARALLEL_DOWNLOAD,
            max_download_retries: DEFAULT_MAX_DOWNLOAD_RETRIES,
            sender: tx,
            _lifetime: PhantomData,
        }
    }

    #[must_use]
    pub fn set_max_parallel_download(mut self, max_parallel_download: usize) -> Self {
        self.max_parallel_download = max_parallel_download;
        self
    }

    #[must_use]
    pub fn set_max_download_retries(mut self, max_download_retries: u32) -> Self {
        self.max_download_retries = max_download_retries;
        self
    }

    #[must_use]
    pub fn set_sender(mut self, sender: mpsc::UnboundedSender<Event>) -> Self {
        self.sender = sender;
        self
    }
}

#[async_trait]
impl<'a> Request for ArchiveDownload<'a> {
    type Response = CbzWriter<'a, Cursor<Vec<u8>>>;

    async fn request(self) -> Result<Self::Response> {
        let retry_policy =
            ExponentialBackoff::builder().build_with_max_retries(self.max_download_retries);
        let client = ClientBuilder::new(reqwest::Client::new())
            .with(RetryTransientMiddleware::new_with_policy(retry_policy))
            .build();
        let cbz_writer = Mutex::new(CbzWriter::default());
        let image_links = GetImageLinks::new(self.chapter_id).request().await?;
        let len = image_links.len();

        self.sender.send(Event::Init(len))?;

        stream::iter(image_links)
            .map(|description| {
                let client = client.clone();
                let tx = self.sender.clone();
                tokio::spawn(async move {
                    info!("Downloading {}", description.url);

                    let response = client.get(description.url).send().await?;

                    let bytes = response.bytes().await?;

                    tx.send(Event::Download)?;

                    Ok::<_, Error>((description.filename, bytes))
                })
            })
            .buffered(len.min(self.max_parallel_download))
            .map_err(|err| {
                error!("join handle error: {err}");
                Error::from(err)
            })
            .try_for_each(|res| async {
                let (filename, bytes) = match res {
                    Ok(ok) => ok,
                    Err(err) => {
                        error!("impossible to pack image, skipping: {err}");
                        return Ok(());
                    }
                };

                info!("Packing {filename}");

                let mut cbz_writer_guard = cbz_writer.lock().await;
                let extension = Utf8Path::new(&filename)
                    .extension()
                    .map(ToString::to_string)
                    .unwrap_or_default();
                cbz_writer_guard.insert(&bytes, &extension).map_err(|err| {
                    error!("failed to write content to archive file {filename}");
                    Error::from(err)
                })?;
                drop(cbz_writer_guard);

                self.sender.send(Event::Zip).map_err(|err| {
                    error!("failed to send message to channel");
                    Error::from(err)
                })?;

                Ok(())
            })
            .await?;

        self.sender.send(Event::Done)?;

        Ok(cbz_writer.into_inner())
    }
}
