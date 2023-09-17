#![deny(clippy::all)]
#![deny(clippy::pedantic)]

use anyhow::{bail, Result};
use camino::{Utf8Path, Utf8PathBuf};
use cbz::{CbzRead, CbzReader, CbzWrite, CbzWriter, CbzWriterInsertionBuilder};
use clap::Parser;
use glob::glob;

#[derive(Parser, Debug)]
#[clap(about, author, version)]
pub struct Args {
    /// A glob that matches all the archive to merge
    #[clap(short, long)]
    pub archives_glob: String,
    /// The output directory for the merged archive
    #[clap(short, long)]
    pub outdir: Utf8PathBuf,
    /// The merged archive name
    #[clap(short, long)]
    pub name: String,
}

fn main() -> Result<()> {
    let args = Args::parse();

    let mut merged_cbz_writer = CbzWriter::default();

    for path in glob(&args.archives_glob)? {
        let mut current_cbz = CbzReader::from_path(path?)?;

        current_cbz.try_for_each(|file| {
            let file = file?;

            let Some(extension) = Utf8Path::new(file.name())
                .extension()
                .map(ToString::to_string)
            else {
                bail!("Extension couldn't be read from {}", file.name());
            };

            let insertion = CbzWriterInsertionBuilder::from_extension(&extension)
                .set_bytes_from_reader(file)?
                .build()?;

            merged_cbz_writer.insert(insertion)?;

            Ok::<(), anyhow::Error>(())
        })?;
    }

    let merged_cbz_writer_finished = merged_cbz_writer.finish()?;

    let output_path = args
        .outdir
        .join(sanitize_filename::sanitize(format!("{}.cbz", args.name)));

    merged_cbz_writer_finished.write_to_path(output_path)?;

    Ok(())
}
