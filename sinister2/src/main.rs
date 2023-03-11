#![deny(clippy::all)]
#![deny(clippy::pedantic)]

use clap::Parser;

#[derive(Parser, Debug)]
#[clap(about, author, version)]
pub struct Args;

fn main() -> Result<(), sinister2::Error> {
    tracing_subscriber::fmt::init();
    let _args = Args::parse();
    sinister2::run()?;
    Ok(())
}
