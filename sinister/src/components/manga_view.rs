use std::collections::HashMap;

use camino::Utf8PathBuf;
use dexter_core::api::{
    archive_download, get_chapters, get_manga, ArchiveDownload, GetChapters, Request,
};
use dioxus::prelude::*;
use tokio::sync::mpsc;
use tracing::{error, info};

use crate::CHAPTERS_LIMIT;

use super::Loader;

const CONCURRENT_IMAGE_DOWNLOAD: u32 = 10;

#[must_use]
#[inline_props]
pub fn MangaView<'a>(
    cx: Scope,
    manga: UseState<Option<(get_manga::Response, get_chapters::Response)>>,
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
    let language = use_state(cx, || {
        isolang::Language::Eng.to_639_1().unwrap().to_string()
    });

    let download = move |chapter: &get_chapters::Data| {
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
        let (tx, mut rx) = mpsc::unbounded_channel();
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
                        archive_download::Event::Init(s) => size = s as f32,
                        archive_download::Event::Done => {
                            download_progress
                                .with_mut(|download_progress| download_progress.remove(&file_name));
                        }
                        archive_download::Event::Download | archive_download::Event::Zip => {
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
            let cbz = ArchiveDownload::new(&chapter_id)
                .set_max_download_retries(CONCURRENT_IMAGE_DOWNLOAD)
                .set_sender(tx)
                .request()
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

    let change_language = move |evt: FormEvent| {
        if !**loading {
            page.set(1);
            language.set(evt.value.clone());
        }
    };

    use_future!(cx, |page, language| {
        to_owned![loading, manga, manga_state];
        loading.set(true);
        async move {
            let received_chapters = match GetChapters::new(&manga.data.id)
                .set_limit(CHAPTERS_LIMIT)
                .push_language(&*language)
                .set_offset((*page - 1) * CHAPTERS_LIMIT)
                .request()
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
        div { class: "absolute inset-0 bg-slate-800",
            div { class: "flex flex w-full flex-shrink-0 justify-between items-center h-16 px-2 border-b border-slate-900 text-xl",
                div { "{manga.data.attributes.title.en}" }
                div { class: "flex flex-row items-center gap-2",
                    div {
                        select {
                            class: "h-6 px-2 text-slate-900 outline-none text-sm",
                            name: "language",
                            oninput: change_language,
                            value: "{language}",
                            option { value: "{isolang::Language::Eng.to_639_1().unwrap()}",
                                "English"
                            }
                            option { value: "{isolang::Language::Fra.to_639_1().unwrap()}",
                                "French"
                            }
                            for language in isolang::languages() {
                                if !matches!(language, isolang::Language::Fra | isolang::Language::Eng) {
                                    if let Some(code) = language.to_639_1() {
                                        let name = language.to_name();
                                        cx.render(rsx! {
                                            option { value: "{code}", "{name}" }
                                        })
                                    } else {
                                        None
                                    }
                                }
                            }
                        }
                    }
                    div { i { class: "bi bi-x-lg cursor-pointer", onclick: close } }
                }
            }
            div { class: "h-[calc(100%-8rem)] overflow-y-auto",
                for chapter in chapters.data.iter() {
                    div { key: "{chapter.id}", class: "flex flex-row gap-1 px-2",
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
            div { class: "flex items-center justify-center h-16 border-t border-slate-900 gap-2",
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
