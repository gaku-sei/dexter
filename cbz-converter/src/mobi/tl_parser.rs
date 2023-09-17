use std::path::Path;

use anyhow::Result;
use cbz::image::Image;
use mobi::Mobi;
use tl::{HTMLTag, ParserOptions, VDom};
use tracing::{debug, error, warn};

use crate::utils::base_32;

use super::MobiVersion;

pub fn convert_to_imgs(path: impl AsRef<Path>) -> Result<Vec<Image>> {
    let mobi = Mobi::from_path(path)?;
    // Or is it `gen_version`? Both were equal in all the files I tested.
    let version = MobiVersion::try_from(mobi.metadata.mobi.format_version)?;
    debug!("mobi version {version:#?}");
    let imgs = mobi.image_records();
    debug!("found {} images", imgs.len());
    let html = mobi.content_as_string_lossy();
    let dom = tl::parse(&html, ParserOptions::default())?;
    let mut all_imgs = Vec::with_capacity(imgs.len());
    for_each_fid(version, &dom, |fid| {
        if let Some(img) = imgs.get(fid) {
            match Image::from_bytes(img.content) {
                Ok(img) => all_imgs.push(img),
                Err(err) => error!("failed to decode image: {err}"),
            };
        } else {
            warn!("unknown fid {fid}");
        }
    });
    Ok(all_imgs)
}

fn for_each_fid<F>(version: MobiVersion, dom: &VDom, mut f: F)
where
    F: FnMut(usize),
{
    // By no mean a perfect implementation of a mobi/azw3 interpreter,
    // the documentation is very sparse and no proper specs are accessible,
    // it's mostly guess work and best effort.
    match version {
        MobiVersion::Mobi6 => {
            for_each_tag(dom, "img[recindex]", |tag| {
                let Some(Some(recindex)) = tag.attributes().get("recindex") else {
                    return;
                };
                let fid = String::from_utf8_lossy(recindex.as_bytes())
                    .parse()
                    .unwrap();

                f(fid);
            });
        }
        MobiVersion::Mobi8 => {
            for_each_tag(dom, "img[src]", |tag| {
                let Some(Some(src)) = tag.attributes().get("src") else {
                    debug!("tag has no src attribute, or it's invalid {tag:#?}");
                    return;
                };
                let src = src.as_utf8_str();
                // Encoding may be broken, we use a "best effort" strategy
                // instead of simply extracting the fid and mime type from the string
                let Some(mime_index) = src.find("?mime=") else {
                    warn!("mime type not found for {src}");
                    return;
                };
                // We assume the code is running on a 64bit system, so it's safe to unwrap
                let fid = usize::try_from(base_32(src[mime_index - 4..mime_index].as_bytes()))
                    .unwrap()
                    - 1;
                // Mime is unused for now but could be handy later on, let's keep it for now
                // let Ok::<Mime, _>(mime_type) = src[mime_index + 6..].parse() else {
                //     warn!("invalid mime type for {mime_index} {src}");
                //     return;
                // };
                f(fid);
            });
        }
    }
}

fn for_each_tag<F>(dom: &VDom, selector: &str, mut f: F)
where
    F: FnMut(&HTMLTag<'_>),
{
    let Some(node_handles) = dom.query_selector(selector) else {
        debug!("no nodes found");
        return;
    };
    for node_handle in node_handles {
        let Some(node) = node_handle.get(dom.parser()) else {
            debug!("node not found {}", node_handle.get_inner());
            continue;
        };
        let Some(tag) = node.as_tag() else {
            debug!("node is not a tag {node:#?}");
            continue;
        };
        f(tag);
    }
}
