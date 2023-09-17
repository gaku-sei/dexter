use std::{io::Cursor, path::Path};

use anyhow::Result;
use cbz::image::Image;
use pdf::{
    enc::StreamFilter,
    file::FileOptions as PdfFileOptions,
    object::{Resolve, XObject},
};
use tracing::error;

pub fn convert_to_imgs(path: impl AsRef<Path>) -> Result<Vec<Image>> {
    let pdf = PdfFileOptions::cached().open(path)?;
    // We may have actually less images than the count but never more,
    // at worse we request a slightly bigger capacity than necessary but at best we prevent any further allocations.
    let mut imgs = Vec::with_capacity(pdf.pages().count());

    for page in pdf.pages() {
        for resource in page?.resources()?.xobjects.values() {
            let resource = match pdf.get(*resource) {
                Ok(resource) => resource,
                Err(err) => {
                    error!("failed to get resource from pdf: {err}");
                    continue;
                }
            };
            if let XObject::Image(image) = &*resource {
                let (image, filter) = match image.raw_image_data(&pdf) {
                    Ok(image_data) => image_data,
                    Err(err) => {
                        error!("failed to get image data: {err}");
                        continue;
                    }
                };
                if let Some(StreamFilter::DCTDecode(_)) = filter {
                    let img = match Image::from_reader(Cursor::new(&image)) {
                        Ok(img) => img,
                        Err(err) => {
                            error!("image couldn't be read: {err}");
                            continue;
                        }
                    };
                    imgs.push(img);
                    break;
                }
            }
        }
    }

    Ok(imgs)
}
