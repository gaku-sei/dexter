use std::collections::HashMap;

use camino::Utf8PathBuf;
use dexter_core::{
    download_images, get_chapters, ChaptersData, ChaptersResponse, ImageDownloadEvent,
    MangaResponse,
};
use dioxus::prelude::*;
use tokio::sync::mpsc;
use tracing::{error, info};

use crate::CHAPTERS_LENGTH;

use super::Loader;

const CONCURRENT_IMAGE_DOWNLOAD: u32 = 10;

#[must_use]
#[inline_props]
pub fn MangaView<'a>(
    cx: Scope,
    manga: UseState<Option<(MangaResponse, ChaptersResponse)>>,
    download_progress: UseRef<HashMap<String, f32>>,
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
    let page = use_state(cx, || 1);
    let loading = use_state(cx, || false);

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
        download_progress
            .with_mut(|download_progress| download_progress.insert(file_name.clone(), 0.));
        let (tx, mut rx) = mpsc::channel(1000);
        {
            let file_name = file_name.clone();
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
                                .with_mut(|download_progress| download_progress.remove(&file_name));
                        }
                        ImageDownloadEvent::Download | ImageDownloadEvent::Zip => {
                            progress += 1.0;
                            download_progress.with_mut(|download_progress| {
                                download_progress
                                    .insert(file_name.clone(), progress / (size * 2.0) * 100.0)
                            });
                        }
                    }
                }
            });
        }

        tokio::spawn(async move {
            let cbz = download_images(&chapter_id, CONCURRENT_IMAGE_DOWNLOAD, tx)
                .await
                .unwrap();
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

    let close = move |_evt| {
        if download_progress.read().is_empty() {
            on_close.call(());
        }
    };

    let set_page = move |new_page| {
        if !**loading {
            page.set(new_page);
        }
    };

    use_future!(cx, |page| {
        to_owned![loading, manga, manga_state];
        loading.set(true);
        async move {
            let received_chapters = match get_chapters(
                &manga.data.id,
                CHAPTERS_LENGTH,
                (*page - 1) * CHAPTERS_LENGTH,
                Vec::<String>::new(),
                Vec::<String>::new(),
            )
            .await
            {
                Ok(chapters) => chapters,
                Err(err) => {
                    error!("chapters get error: {err}");
                    return;
                }
            };
            manga_state.with_mut(|manga| {
                if let Some(manga) = manga {
                    manga.1 = received_chapters;
                }
            });
            loading.set(false);
        }
    });

    cx.render(rsx! {
        div {
            class: "absolute inset-0 bg-slate-800",
            div {
                class: "flex flex w-full flex-shrink-0 justify-between items-center h-16 px-2 border-b border-slate-900 text-xl",
                div { "{manga.data.attributes.title.en}" }
                div {
                    i {
                        class: "bi bi-x-lg cursor-pointer",
                        onclick: close,
                    }
                }
            }
            div {
                class: "h-[calc(100%-8rem)] overflow-y-auto",
                for chapter in chapters.data.iter() {
                    div {
                        key: "{chapter.id}",
                        class: "flex flex-row gap-1 px-2",
                        div {
                            class: "flex items-center",
                            title: "Download",
                            onclick: move |_evt| download(chapter),
                            i { class: "bi bi-download cursor-pointer" }
                        }
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
                    rsx! {
                        div {
                            class: "flex justify-center items-center cursor-pointer px-2 border border-slate-900 bg-slate-700 rounded hover:bg-slate-500 w-24",
                            onclick: move |_evt| set_page(**page - 1),
                            "Previous"
                        }
                    }
                }
                if chapters.offset + chapters.limit < chapters.total {
                    rsx! {
                        div {
                            class: "flex justify-center items-center cursor-pointer px-2 border border-slate-900 bg-slate-700 rounded hover:bg-slate-500 w-24",
                            onclick: move |_evt| set_page(**page + 1),
                            "Next"
                        }
                    }
                }
            }
        }
    })
}