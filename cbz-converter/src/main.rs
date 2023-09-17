#![deny(clippy::pedantic)]

use std::fs;

use anyhow::Result;
use camino::Utf8PathBuf;
use cbz::image::ReadingOrder;
use cbz_pack::pack_imgs_to_cbz;
use clap::{Parser, ValueEnum};
use tracing::{debug, info};

use crate::mobi::convert_to_imgs as mobi_to_imgs;
use crate::pdf::convert_to_imgs as pdf_to_imgs;

mod mobi;
mod pdf;
mod utils;

#[derive(Debug, Clone, Copy, ValueEnum)]
enum Format {
    Mobi,
    Azw3,
    Pdf,
}

#[derive(Debug, Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Path to the source file
    path: Utf8PathBuf,
    /// Source format
    #[clap(long, short)]
    from: Format,
    /// Dir to output images
    #[clap(long, short)]
    outdir: Utf8PathBuf,
    /// The archive name
    #[clap(long, short)]
    name: String,
    /// Adjust images contrast
    #[clap(long)]
    contrast: Option<f32>,
    /// Adjust images brightness
    #[clap(long)]
    brightness: Option<i32>,
    /// Blur image (slow with big numbers)
    #[clap(long)]
    blur: Option<f32>,
    /// Automatically split landscape images into 2 pages
    #[clap(long, action)]
    autosplit: bool,
    /// Reading order
    #[clap(long, default_value_t = ReadingOrder::Rtl)]
    reading_order: ReadingOrder,
}

fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let args = Args::parse();
    fs::create_dir_all(&args.outdir)?;
    let imgs = match args.from {
        Format::Mobi | Format::Azw3 => mobi_to_imgs(args.path)?,
        Format::Pdf => pdf_to_imgs(args.path)?,
    };
    info!("found {} imgs", imgs.len());

    let out_cbz_writer_finished = pack_imgs_to_cbz(
        imgs,
        args.contrast,
        args.brightness,
        args.blur,
        args.autosplit,
        args.reading_order,
    )?;

    let output_path = args
        .outdir
        .join(sanitize_filename::sanitize(format!("{}.cbz", args.name)));
    debug!("writing cbz file to {output_path}");

    out_cbz_writer_finished.write_to_path(output_path)?;

    Ok(())
}
