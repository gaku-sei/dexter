use anyhow::Result;
use clap::Parser;
use dexter_core::get_reader_size;
use std::{convert::TryFrom, fs::File, path::PathBuf};

use crate::lib::run;

mod lib;

#[derive(Parser, Debug)]
#[clap(about, author, version)]
pub struct Options {
    /// The path to the cbz archive
    #[clap(short, long)]
    pub input: String,
}

pub fn main() -> Result<()> {
    let options = Options::parse();

    let path = PathBuf::from(options.input);

    let file = File::open(path.as_path())?;

    let size = get_reader_size(file)?;

    let size = i32::try_from(size)?;

    run(path, size)?;

    Ok(())
}
