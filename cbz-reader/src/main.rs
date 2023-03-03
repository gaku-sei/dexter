#![deny(clippy::all)]
#![deny(clippy::pedantic)]

use anyhow::Result;
use camino::Utf8PathBuf;
use cbz::CbzReader;
use clap::Parser;

#[derive(Parser, Debug)]
#[clap(about, author, version)]
pub struct Args {
    /// The path to the cbz archive file to read
    pub archive_path: Utf8PathBuf,
}

fn main() -> Result<()> {
    env_logger::init();
    let args = Args::parse();
    let mut cbz = CbzReader::from_path(args.archive_path)?;
    cbz_reader::run(&mut cbz)?;

    Ok(())
}
