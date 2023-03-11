#![deny(clippy::all)]
#![deny(clippy::pedantic)]

use clap::Parser;

#[derive(Parser, Debug)]
#[clap(about, author, version)]
pub struct Args;

fn main() {
    tracing_subscriber::fmt::init();
    let _args = Args::parse();
    let rt = tokio::runtime::Runtime::new().unwrap();
    let _guard = rt.enter();

    sinister::run();
}
