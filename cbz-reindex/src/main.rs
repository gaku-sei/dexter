#![deny(clippy::all)]
#![deny(clippy::pedantic)]

use std::{
    env::current_dir,
    ffi::OsStr,
    io,
    io::{Cursor, Write},
    path::PathBuf,
    {fs::File, io::BufReader},
};

use anyhow::{anyhow, Result};
use clap::Parser;
use dexter_core::update_filename_index;
use zip::{write::FileOptions, ZipArchive, ZipWriter};

#[derive(Parser, Debug)]
#[clap(about, author, version)]
pub struct Args {
    /// The archive to update the indexes of
    #[clap(short, long)]
    pub archive_path: PathBuf,
    /// The output directory for the repaired archive (must be different than the location of the archive itself)
    #[clap(short, long)]
    pub outdir: PathBuf,
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();

    let args = Args::parse();

    let archive = File::open(&args.archive_path)?;

    let Some(archive_filename) = args.archive_path.file_name().and_then(OsStr::to_str) else {
        panic!("archive name couldn't be read");
    };

    let mut fixed_archive_writer = ZipWriter::new(Cursor::new(Vec::new()));

    let mut invalid_archive_reader = ZipArchive::new(BufReader::new(archive))?;

    for i in 0..invalid_archive_reader.len() {
        let mut file = invalid_archive_reader.by_index(i)?;

        let new_filename = update_filename_index(file.name(), 3)?;

        fixed_archive_writer
            .start_file(&new_filename, FileOptions::default())
            .map_err(|_| anyhow!("failed to create archive file"))?;

        io::copy(&mut file, &mut fixed_archive_writer)
            .map_err(|_| anyhow!("failed to write content to archive file"))?;
    }

    let fixed_archive_buffer = fixed_archive_writer.finish()?;

    let mut fixed_archive = File::create(current_dir()?.join(args.outdir).join(archive_filename))?;

    fixed_archive.write_all(&fixed_archive_buffer.into_inner())?;

    Ok(())
}
