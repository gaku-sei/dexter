#![deny(clippy::all)]
#![deny(clippy::pedantic)]

use std::{
    env,
    fmt::Display,
    fs::create_dir,
    io::{BufRead, Cursor, Seek},
    ops::Deref,
    path::Path,
    sync::{Arc, Mutex},
};

use anyhow::{anyhow, bail, Context, Error, Result};
use camino::{Utf8Path, Utf8PathBuf};
use cbz::{CbzWrite, CbzWriter, CbzWriterInsertionBuilder, COUNTER_SIZE};
use clap::{Parser, ValueEnum};
use futures::future::try_join_all;
use glob::glob;
use image::{io::Reader as ImageReader, DynamicImage, ImageFormat};
#[cfg(feature = "pdf")]
use pdf::{
    enc::StreamFilter,
    file::FileOptions as PdfFileOptions,
    object::{Resolve, XObject},
};
use tracing::{debug, error};

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
    /// A glob that matches all the files to pack,
    /// if the `--pdf` flag is set this must be a path to the pdf file to pack
    pub files_descriptor: String,
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
    #[cfg(feature = "pdf")]
    /// Enable pdf auto extraction/repacking
    #[clap(long, action)]
    pub pdf: bool,
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

    fn from_reader(reader: impl BufRead + Seek) -> Result<Self> {
        Ok(Self(
            ImageReader::new(reader).with_guessed_format()?.decode()?,
        ))
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
    tracing_subscriber::fmt::init();

    let args = Args::parse();
    let Ok(current_dir) = Utf8PathBuf::from_path_buf(env::current_dir()?) else {
        bail!("current dir is not a valid utf-8 path");
    };
    let outdir = current_dir.join(&args.outdir);
    if !outdir.exists() {
        create_dir(&*outdir)?;
    }

    let out_cbz_writer = Arc::new(Mutex::new(CbzWriter::default()));

    #[cfg(feature = "pdf")]
    let contents = if args.pdf {
        get_images_from_pdf(&args.files_descriptor).await
    } else {
        get_images_from_glob(&args.files_descriptor).await
    }?;

    #[cfg(not(feature = "pdf"))]
    let contents = get_images_from_glob(&args.files_descriptor).await?;

    let handles = contents.into_iter().filter_map(|res| {
        let (index, img) = match res {
            Ok(pair) => pair,
            Err(err) => {
                error!("{err}");
                return None;
            }
        };
        let out_cbz_writer = Arc::clone(&out_cbz_writer);

        Some(tokio::spawn(async move {
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
                debug!("splitting landscape file");
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
        }))
    });

    for res in try_join_all(handles).await? {
        res?;
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

async fn get_images_from_glob(glob_expr: impl AsRef<str>) -> Result<Vec<Result<(usize, Image)>>> {
    try_join_all(glob(glob_expr.as_ref())?.enumerate().map(|(index, path)| {
        tokio::spawn(async move {
            let path = path?;
            let Some(path) = Utf8Path::from_path(&path) else {
                bail!("{path:?} is not a valid utf-8 path");
            };
            let img = Image::open(path)?;

            Ok((index, img))
        })
    }))
    .await
    .map_err(Into::into)
}

#[cfg(feature = "pdf")]
async fn get_images_from_pdf(path: impl AsRef<Path>) -> Result<Vec<Result<(usize, Image)>>> {
    let pdf = Arc::new(PdfFileOptions::cached().open(path)?);
    try_join_all(pdf.pages().enumerate().map(|(index, page)| {
        let pdf = Arc::clone(&pdf);
        tokio::spawn(async move {
            let page = page?;
            let mut found_image = None;
            for resource in page.resources()?.xobjects.values() {
                let resource = pdf.get(*resource)?;
                if let XObject::Image(image) = &*resource {
                    let (image, filter) = image.raw_image_data(Arc::as_ref(&pdf))?;
                    if let Some(StreamFilter::DCTDecode(_)) = filter {
                        found_image = Some(Image::from_reader(Cursor::new(&image))?);
                        break;
                    }
                }
            }

            match found_image {
                Some(image) => Ok::<_, Error>((index, image)),
                None => bail!("no image found"),
            }
        })
    }))
    .await
    .map_err(Into::into)
}
