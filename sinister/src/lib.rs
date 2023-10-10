#![deny(clippy::all)]
#![deny(clippy::pedantic)]
#![allow(non_snake_case)]
#![allow(clippy::ignored_unit_patterns)]

use std::{collections::HashMap, time::Duration};

use dexter_core::{GetChapters, GetManga, Request, Search};
use dioxus::prelude::*;
use dioxus_desktop::{Config, WindowBuilder};
use tokio::time::sleep;
use tracing::error;

use crate::components::{Loader, MangaList, MangaView, Progress};

pub mod components;

static MANGAS_LENGTH: u32 = 50;
pub(crate) static CHAPTERS_LIMIT: u32 = 100;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("unknown error: {0}")]
    Unknown(String),
}

pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug)]
pub struct AppProps;

/// Starts a new window with Sinister inside
pub fn run() {
    dioxus_desktop::launch_with_props(
        App,
        AppProps,
        Config::default()
            .with_custom_index(include_str!("index.html").to_string())
            .with_window(WindowBuilder::default().with_title("Sinister")),
    );
}

#[allow(clippy::too_many_lines)]
fn App(cx: Scope<AppProps>) -> Element {
    let mangas_search = use_ref(cx, String::new);
    let mangas = use_ref(cx, || None);
    let selected_manga_id = use_state(cx, || None::<String>);
    let selected_manga = use_state(cx, || None);
    let form_classes = use_state(cx, || "h-full");
    let manga_search_loading = use_state(cx, || false);
    let manga_loading = use_state(cx, || false);
    let download_progress = use_ref(cx, HashMap::<String, f32>::new);

    let onsubmit = move |evt: FormEvent| {
        if !**manga_search_loading {
            mangas_search.set(evt.values["title"][0].clone());
        }
    };

    use_effect(
        cx,
        (mangas, manga_search_loading),
        |(mangas, manga_search_loading)| {
            to_owned![form_classes];
            async move {
                if mangas.read().is_some() || *manga_search_loading {
                    form_classes.set("h-16 border-b border-slate-900");
                }
            }
        },
    );

    use_future!(cx, |mangas_search| {
        to_owned![mangas, manga_search_loading];
        async move {
            let mangas_search = mangas_search.read();
            if mangas_search.is_empty() {
                return;
            }
            mangas.set(None);
            manga_search_loading.set(true);
            sleep(Duration::from_secs(1)).await;
            let received_mangas = match Search::new(&*mangas_search)
                .with_limit(MANGAS_LENGTH)
                .request()
                .await
            {
                Ok(mangas) => mangas,
                Err(err) => {
                    error!("manga search error: {err}");
                    return;
                }
            };
            mangas.set(Some(received_mangas.data));
            manga_search_loading.set(false);
        }
    });

    use_future!(cx, |selected_manga_id| {
        to_owned![selected_manga, manga_loading];
        async move {
            let Some(manga_id) = &*selected_manga_id else {
                return;
            };
            manga_loading.set(true);
            sleep(Duration::from_secs(1)).await;
            let received_manga = match GetManga::new(manga_id).request().await {
                Ok(manga) => manga,
                Err(err) => {
                    error!("manga get error: {err}");
                    return;
                }
            };
            let received_chapters = match GetChapters::new(manga_id)
                .set_limit(CHAPTERS_LIMIT)
                .push_language("en")
                .request()
                .await
            {
                Ok(chapters) => chapters,
                Err(err) => {
                    error!("chapters get error: {err}");
                    return;
                }
            };
            selected_manga.set(Some((received_manga, received_chapters)));
            manga_loading.set(false);
        }
    });

    cx.render(rsx! {
        div { class: "w-screen h-screen flex flex-col text-slate-400",
            if !download_progress.read().is_empty() {
                rsx! {
                    div {
                        class: "absolute pointer-events-none flex flex-col max-h-80 w-80 top-1 right-1 gap-1 z-50 overflow-y-hidden",
                        for (file_name, percent) in download_progress.read().iter() {
                            Progress {
                                key: "{file_name}",
                                label: file_name.to_string(),
                                percent: *percent,
                            }
                        }
                    }
                }
            }
            div { class: "flex flex-shrink-0 w-full items-center justify-center transition-[height] {form_classes}",
                form {
                    onsubmit: onsubmit,
                    prevent_default: "onsubmit",
                    class: "flex flex-row gap-1 h-10 m-0",
                    input {
                        class: "h-full px-2 text-slate-900 outline-none",
                        r#type: "text",
                        autofocus: "on",
                        autocapitalize: "off",
                        autocomplete: "off",
                        name: "title"
                    }
                    button {
                        class: "h-full px-2 bg-slate-900 hover:bg-slate-600",
                        r#type: "submit",
                        disabled: "{manga_search_loading}",
                        "Search"
                    }
                }
            }
            if **manga_search_loading {
                rsx! {
                    div {
                        class: "flex flex-col h-full items-center justify-center overflow-hidden",
                        Loader {}
                    }
                }
            }
            if selected_manga_id.is_none() {
                rsx! {
                    MangaList {
                        mangas: mangas.clone(),
                        on_select: move |manga_id| selected_manga_id.set(Some(manga_id)),
                    }
                }
            }
            if **manga_loading || selected_manga.is_some() {
                rsx! {
                    MangaView {
                        manga: selected_manga.clone(),
                        download_progress: download_progress.clone(),
                        on_close: move |()| {
                            selected_manga_id.set(None);
                            selected_manga.set(None);
                        },
                    }
                }
            }
        }
    })
}
