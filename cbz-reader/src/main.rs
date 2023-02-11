#![deny(clippy::all)]
#![deny(clippy::pedantic)]

use anyhow::Result;
use camino::Utf8PathBuf;
use cbz::CbzReader;
use cbz_reader::run;
use clap::Parser;

#[derive(Parser, Debug)]
#[clap(about, author, version)]
pub struct Args {
    /// The path to the cbz archive
    #[clap(short, long)]
    pub input: Utf8PathBuf,
}

#[allow(clippy::missing_errors_doc)]
pub fn main() -> Result<()> {
    let args = Args::parse();

    let cbz = CbzReader::from_path(args.input)?;

    run(cbz)?;

    Ok(())
}
