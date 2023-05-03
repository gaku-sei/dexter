#![deny(clippy::all)]
#![deny(clippy::pedantic)]

pub use crate::{
    api::{ArchiveDownload, GetChapter, GetChapters, GetImageLinks, GetManga, Request, Search},
    errors::{Error, Result},
};

pub mod api;
pub mod errors;
