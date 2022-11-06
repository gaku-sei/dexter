#![deny(clippy::all)]
#![deny(clippy::pedantic)]

use std::{
    fs::File,
    io::{self, BufReader, Cursor, Write},
    path::PathBuf,
};

use anyhow::Result;
use clap::Parser;
use glob::glob;
use zip::{write::FileOptions, ZipArchive, ZipWriter};

#[derive(Parser, Debug)]
#[clap(about, author, version)]
pub struct Args {
    /// A glob that matches all the archive to merge
    #[clap(short, long)]
    pub archives_glob: String,
    /// The output directory for the merged archive
    #[clap(short, long)]
    pub outdir: PathBuf,
    /// The merged archive name
    #[clap(short, long)]
    pub name: String,
}

fn main() -> Result<()> {
    let args = Args::parse();

    let mut merged_archive_writer = ZipWriter::new(Cursor::new(Vec::new()));

    for (index, entry) in glob(&args.archives_glob)?.enumerate() {
        let entry = entry?;

        let archive = File::open(&entry)?;

        let mut archive_reader = ZipArchive::new(BufReader::new(archive))?;

        for i in 0..archive_reader.len() {
            let mut file = archive_reader.by_index(i)?;

            let filename = format!("{:0>2} - {}", index + 1, file.name());

            merged_archive_writer.start_file(&filename, FileOptions::default())?;

            io::copy(&mut file, &mut merged_archive_writer)?;
        }
    }

    let merged_archive_buffer = merged_archive_writer.finish()?;

    let mut merged_archive = File::create(
        args.outdir
            .join(sanitize_filename::sanitize(format!("{}.cbz", args.name))),
    )?;

    merged_archive.write_all(&merged_archive_buffer.into_inner())?;

    Ok(())
}
