use std::{io, result};

use zip::result::ZipError;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("IO error {0}")]
    IO(#[from] io::Error),

    #[error("Zip error {0}")]
    Zip(#[from] ZipError),

    #[error("Cbz file size couldn't be converted")]
    CbzFileSizeConversion,

    #[error("Cbz file name is empty")]
    CbzFileNameEmpty,

    #[error("Cbz file invalid index {0}")]
    CbzFileInvalidIndex(String),

    #[error("File at index {0} not found in cbz")]
    CbzNotFound(usize),

    #[error("Cbz is too large, it can contain a maximum of {0} files")]
    CbzTooLarge(usize),

    #[error("Cbz file insertion's extension not provided")]
    CbzInsertionNoExtension,

    #[error("Cbz file insertion: no bytes set")]
    CbzInsertionNoBytes,

    #[error("Image error: {0}")]
    Image(#[from] image::ImageError),
}

pub type Result<T, E = Error> = result::Result<T, E>;
