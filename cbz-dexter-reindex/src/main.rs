#![deny(clippy::all)]
#![deny(clippy::pedantic)]

use std::{
    env::current_dir,
    fs::File,
    io,
    io::{Cursor, Write},
};

use anyhow::{anyhow, bail, Result};
use camino::Utf8PathBuf;
use cbz::{CbzFile, CbzRead, CbzReader};
use clap::Parser;
use zip::{write::FileOptions, ZipWriter};

#[derive(Parser, Debug)]
#[clap(about, author, version)]
pub struct Args {
    /// The archive to update the indices of
    #[clap(short, long)]
    pub archive_path: Utf8PathBuf,
    /// The output directory for the repaired archive (must be different than the location of the archive itself)
    #[clap(short, long)]
    pub outdir: Utf8PathBuf,
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let args = Args::parse();

    let Some(archive_filename) = args.archive_path.file_name() else {
        bail!("archive name couldn't be read");
    };

    let mut fixed_archive_writer = ZipWriter::new(Cursor::new(Vec::new()));

    let mut invalid_archive_reader = CbzReader::from_path(&args.archive_path)?;

    invalid_archive_reader.try_for_each(|cbz_file| {
        let mut cbz_file = cbz_file?;

        let new_filename = reformat_index(&cbz_file, 3)?;

        fixed_archive_writer
            .start_file(&new_filename, FileOptions::default())
            .map_err(|_| anyhow!("failed to create archive file"))?;

        io::copy(&mut cbz_file, &mut fixed_archive_writer)
            .map_err(|_| anyhow!("failed to write content to archive file"))?;

        Ok::<(), anyhow::Error>(())
    })?;

    let fixed_archive_buffer = fixed_archive_writer.finish()?;

    let mut fixed_archive = File::create(current_dir()?.join(args.outdir).join(archive_filename))?;

    fixed_archive.write_all(&fixed_archive_buffer.into_inner())?;

    Ok(())
}

/// Indices are often ill formatted or invalid (x1, R2, etc...).
/// This function will clean up the index and add some padding (3).
///
/// ## Errors
///
/// Fails if the filename is empty or the index is invalid (longer than the `expected_length` or not a valid unsigned integer).
pub fn reformat_index(cbz_file: &CbzFile, expected_length: usize) -> Result<String> {
    let mut name_chars = cbz_file.name().chars();

    let Some(first_char) = name_chars.next() else {
        bail!("filename is empty");
    };

    let mut index = if first_char.is_numeric() {
        first_char.to_string()
    } else {
        String::new()
    };

    #[allow(clippy::while_let_on_iterator)]
    while let Some(c) = name_chars.next() {
        if c == '-' {
            break;
        }

        index.push(c);
    }

    if index.len() > expected_length || index.parse::<u16>().is_err() {
        bail!("invalid index {index}");
    }

    Ok(format!(
        "{index:0>expected_length$}-{}",
        name_chars.as_str()
    ))
}
