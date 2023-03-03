#![deny(clippy::all)]
#![deny(clippy::pedantic)]

use std::{
    env,
    fmt::Display,
    fs::create_dir,
    io::Cursor,
    ops::Deref,
    path::Path,
    sync::{Arc, Mutex},
};

use anyhow::{anyhow, bail, Context, Error, Result};
use camino::{Utf8Path, Utf8PathBuf};
use cbz::{CbzWrite, CbzWriter, CbzWriterInsertionBuilder, COUNTER_SIZE};
use clap::{Parser, ValueEnum};
use futures::future::join_all;
use glob::glob;
use image::{io::Reader as ImageReader, DynamicImage, ImageFormat};
use log::debug;

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum ReadingOrder {
    Rtl,
    Ltr,
}

impl Display for ReadingOrder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Ltr => "ltr",
                Self::Rtl => "rtl",
            }
        )
    }
}

#[derive(Parser, Debug)]
#[clap(about, author, version)]
pub struct Args {
    /// A glob that matches all the files to pack
    #[clap(short, long)]
    pub files_glob: String,
    /// The output directory for the merged archive
    #[clap(short, long, default_value = "./")]
    pub outdir: Utf8PathBuf,
    /// The merged archive name
    #[clap(short, long)]
    pub name: String,
    /// Adjust images contrast
    #[clap(long)]
    pub contrast: Option<f32>,
    /// Adjust images brightness
    #[clap(long)]
    pub brightness: Option<i32>,
    /// Blur image (slow with big numbers)
    #[clap(long)]
    pub blur: Option<f32>,
    /// Automatically split landscape images into 2 pages
    #[clap(long, action)]
    pub autosplit: bool,
    /// Reading order
    #[clap(long, default_value_t = ReadingOrder::Rtl)]
    pub reading_order: ReadingOrder,
}

#[derive(Debug, PartialEq)]
struct Image(DynamicImage);

impl Deref for Image {
    type Target = DynamicImage;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<DynamicImage> for Image {
    fn from(value: DynamicImage) -> Self {
        Self(value)
    }
}

impl Image {
    fn open(path: impl AsRef<Path>) -> Result<Self> {
        Ok(Self(ImageReader::open(&path)?.decode()?))
    }

    fn is_portrait(&self) -> bool {
        self.height() > self.width()
    }

    fn is_landscape(&self) -> bool {
        !self.is_portrait()
    }

    fn set_contrast(&self, contrast: f32) {
        self.adjust_contrast(contrast);
    }

    fn set_brightness(&self, brightness: i32) {
        self.brighten(brightness);
    }

    fn set_blur(&self, blur: f32) {
        self.blur(blur);
    }

    fn autosplit(self, reading_order: ReadingOrder) -> (Image, Image) {
        let img1 = self.crop_imm(0, 0, self.width() / 2, self.height()).into();
        let img2 = self
            .crop_imm(self.width() / 2, 0, self.width(), self.height())
            .into();
        match reading_order {
            ReadingOrder::Ltr => (img1, img2),
            ReadingOrder::Rtl => (img2, img1),
        }
    }

    fn insert_into_cbz_writer(
        self,
        cbz_writer: &mut impl CbzWrite,
        format: ImageFormat,
        name: impl AsRef<str>,
    ) -> Result<()> {
        let mut out = Cursor::new(Vec::new());
        self.write_to(&mut out, format)
            .context("writing image to memory")?;
        let insertion = CbzWriterInsertionBuilder::from_extension(format.extensions_str()[0])
            .set_bytes_ref(out.get_ref())
            .build_custom_str(name.as_ref())?;
        cbz_writer
            .insert_custom_str(insertion)
            .context("couldn't insert page into zip")?;
        debug!("inserted page into zip");

        Ok(())
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();

    let args = Args::parse();
    let Ok(current_dir) = Utf8PathBuf::from_path_buf(env::current_dir()?) else {
        bail!("current dir is not a valid utf-8 path");
    };
    let outdir = current_dir.join(&args.outdir);
    if !outdir.exists() {
        create_dir(&*outdir)?;
    }

    let out_cbz_writer = Arc::new(Mutex::new(CbzWriter::default()));

    let handles = glob(&args.files_glob)?
        .into_iter()
        .enumerate()
        .map(|(index, path)| {
            let out_cbz_writer = Arc::clone(&out_cbz_writer);

            tokio::spawn(async move {
                let path = path?;
                let Some(path) = Utf8Path::from_path(&path) else {
                bail!("{path:?} is not a valid utf-8 path");
            };
                let img = Image::open(path)?;
                if let Some(contrast) = args.contrast {
                    img.set_contrast(contrast);
                }
                if let Some(brightness) = args.brightness {
                    img.set_brightness(brightness);
                }
                if let Some(blur) = args.blur {
                    img.set_blur(blur);
                }

                if img.is_landscape() && args.autosplit {
                    debug!("splitting landscape file {path}");
                    let (img_left, img_right) = img.autosplit(args.reading_order);
                    let mut out_cbz_writer = out_cbz_writer.lock().unwrap();
                    img_left.insert_into_cbz_writer(
                        &mut *out_cbz_writer,
                        ImageFormat::Png,
                        format!("{index:0>COUNTER_SIZE$}-1"),
                    )?;
                    img_right.insert_into_cbz_writer(
                        &mut *out_cbz_writer,
                        ImageFormat::Png,
                        format!("{index:0>COUNTER_SIZE$}-2"),
                    )?;
                } else {
                    let mut out_cbz_writer = out_cbz_writer.lock().unwrap();
                    img.insert_into_cbz_writer(
                        &mut *out_cbz_writer,
                        ImageFormat::Png,
                        format!("{index:0>COUNTER_SIZE$}"),
                    )?;
                }

                Ok::<_, Error>(())
            })
        });

    for res in join_all(handles).await {
        res??;
    }

    let out_cbz_writer_finished = Arc::try_unwrap(out_cbz_writer)
        .map_err(|_err| anyhow!("cbz writer arc can't be unwrapped"))?
        .into_inner()
        .map_err(|err| anyhow!("{err}"))?
        .finish()?;

    let output_path = outdir.join(sanitize_filename::sanitize(format!("{}.cbz", args.name)));
    debug!("writing cbz file to {output_path}");

    out_cbz_writer_finished.write_to_path(output_path)?;

    Ok(())
}
