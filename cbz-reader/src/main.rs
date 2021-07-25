use anyhow::Result;
use clap::{crate_version, Clap};
use dexter_core::open_cbz;
use std::{fs::File, path::PathBuf};

use crate::lib::run;

mod lib;

#[derive(Clap, Debug)]
#[clap(name = "cbz-reader", version = crate_version!())]
pub struct Options {
    /// The path to the cbz archive
    #[clap(short, long)]
    pub input: String,
}

pub fn main() -> Result<()> {
    let options = Options::parse();

    let path = PathBuf::from(options.input);

    let file = File::open(path.as_path())?;

    let size = open_cbz(file)?;

    run(path, size as i32)?;

    Ok(())
}
