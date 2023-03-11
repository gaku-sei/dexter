use std::collections::HashMap;

use camino::Utf8PathBuf;
use dexter_core::{
    download_images, ChaptersData, ChaptersResponse, ImageDownloadEvent, MangaResponse,
};
use dioxus::prelude::*;
use tokio::sync::mpsc;
use tracing::{error, info};

use super::Loader;

#[must_use]
#[inline_props]
pub fn MangaView<'a>(
    cx: Scope,
    manga: UseState<Option<(MangaResponse, ChaptersResponse)>>,
    download_progress: UseRef<HashMap<String, u8>>,
    on_close: EventHandler<'a, ()>,
) -> Element {
    let manga_state = manga;
    let Some((manga, chapters)) = &**manga_state else {
        return cx.render(rsx! {
            div {
                class: "flex flex-col h-full items-center justify-center",
                Loader {}
            }
        });
    };

    let download = move |chapter: &ChaptersData| {
        if download_progress.read().contains_key(&chapter.id) {
            return;
        }
        to_owned![download_progress];
        let chapter_id = chapter.id.clone();
        let file_name = format!(
            "{} - {} - {}.cbz",
            manga.data.attributes.title.en,
            chapter.attributes.chapter.as_deref().unwrap_or("unknown"),
            chapter.attributes.title.as_deref().unwrap_or("unknown"),
        );
        info!("downloading {file_name}");
        // let download_progress_entry =
        download_progress
            .with_mut(|download_progress| download_progress.insert(chapter_id.clone(), 0));
        let (tx, mut rx) = mpsc::channel(1000);
        cx.spawn(async move {
            let mut progress = 0.0;
            let mut size = 0.0;
            while let Some(event) = rx.recv().await {
                #[allow(
                    clippy::cast_precision_loss,
                    clippy::cast_sign_loss,
                    clippy::cast_possible_truncation
                )]
                match event {
                    ImageDownloadEvent::Init(s) => size = s as f32,
                    ImageDownloadEvent::Done => {
                        download_progress
                            .with_mut(|download_progress| download_progress.remove(&chapter_id));
                    }
                    ImageDownloadEvent::Download | ImageDownloadEvent::Zip => {
                        progress += 1.0;
                        download_progress.with_mut(|download_progress| {
                            download_progress
                                .insert(chapter_id.clone(), (progress / (size * 2.0) * 100.0) as u8)
                        });
                    }
                }
            }
        });

        let chapter_id = chapter.id.clone();
        tokio::spawn(async move {
            let cbz = download_images(&chapter_id, 10, tx).await.unwrap();
            let path = Utf8PathBuf::try_from(home::home_dir().unwrap())
                .unwrap()
                .join("Downloads")
                .join(&file_name);
            info!("{file_name} downloaded");
            if let Err(err) = cbz.write_to_path(path) {
                error!("cbz creation error: {err}");
            }
        });
    };

    cx.render(rsx! {
        div {
            class: "absolute inset-0 bg-slate-800",
            div {
                class: "flex flex w-full flex-shrink-0 justify-between items-center h-16 px-2 border-b border-slate-900 text-xl",
                div { "{manga.data.attributes.title.en}" }
                div {
                    i {
                        class: "bi bi-x-lg cursor-pointer",
                        onclick: move |_evt| on_close.call(()),
                    }
                }
            }
            div {
                class: "h-[calc(100%-8rem)] overflow-y-auto",
                for chapter in chapters.data.iter() {
                    div {
                        class: "flex flex-row gap-1 px-2",
                        div {
                            i {
                                class: "bi bi-download cursor-pointer",
                                title: "Download",
                                onclick: move |_evt| download(chapter),
                            }
                        }
                        div {
                            i {
                                class: "bi bi-book cursor-pointer",
                                title: "Read",
                                onclick: move |_evt| {}
                            }
                        }
                        div { "-" }
                        div { chapter.attributes.volume.as_deref().unwrap_or("unknown") }
                        div { "-" }
                        div { chapter.attributes.chapter.as_deref().unwrap_or("unknown") }
                        div { "-" }
                        div { chapter.attributes.title.as_deref().unwrap_or("unknown") }
                        div { "-" }
                        div { chapter.attributes.translated_language.as_deref().unwrap_or("unknown") }
                    }
                }
            }
            div {
                class: "flex items-center justify-center h-16 border-t border-slate-900 gap-2",
                if chapters.offset > 0 {
                    rsx!(div { class: "flex justify-center items-center cursor-pointer px-2 border border-slate-900 bg-slate-700 rounded hover:bg-slate-500 w-24", "Previous" })
                }
                if chapters.offset + chapters.limit < chapters.total {
                    rsx!(div { class: "flex justify-center items-center cursor-pointer px-2 border border-slate-900 bg-slate-700 rounded hover:bg-slate-500 w-24", "Next" })
                }
            }
        }
    })
}
