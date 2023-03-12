#![deny(clippy::all)]
#![deny(clippy::pedantic)]
#![allow(non_snake_case)]

use base64::Engine;
use cbz::CbzRead;
use dioxus::{html::input_data::keyboard_types::Key, prelude::*};
use dioxus_desktop::{Config, WindowBuilder};
use tracing::debug;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("cbz error: {0}")]
    Cbz(#[from] cbz::Error),

    #[error("zip error: {0}")]
    Zip(#[from] zip::result::ZipError),
}

type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug)]
pub struct AppProps {
    imgs: Vec<String>,
}

/// Starts a new window with the CBZ reader inside
///
/// ## Errors
///
/// Fails on cbz read error
pub fn run(cbz_reader: &mut impl CbzRead) -> Result<()> {
    let mut imgs = Vec::new();
    cbz_reader.try_for_each(|file| {
        let mut file = file?;
        let bytes = file.to_bytes()?;
        let base64 = base64::engine::general_purpose::STANDARD.encode(&*bytes);
        imgs.push(base64);

        Ok::<_, Error>(())
    })?;

    dioxus_desktop::launch_with_props(
        App,
        AppProps { imgs },
        Config::default()
            .with_custom_head(r#"<script src="https://cdn.tailwindcss.com"></script>"#.to_string())
            .with_window(WindowBuilder::default().with_title("Cbz Reader")),
    );

    Ok(())
}

fn App(cx: Scope<AppProps>) -> Element {
    let imgs = &cx.props.imgs;
    let max_page = use_state(cx, || cx.props.imgs.len());
    let current_page = use_state(cx, || 1_usize);
    let img = use_state(cx, || cx.props.imgs.get(0).cloned());

    cx.render(rsx! {
        div {
            class: "p-2 w-full h-screen flex flex-col gap-1 items-center outline-none",
            autofocus: true,
            tabindex: -1,
            onkeydown: move |evt| {
                match evt.key() {
                    Key::ArrowLeft => {
                        let page = *current_page.get();
                        if page == 1 {
                            return;
                        }

                        current_page.set(page - 1);
                        debug!("reading index {}", page - 2);
                        img.set(imgs.get(page - 2).cloned());
                    },
                    Key::ArrowRight => {
                        let page = *current_page.get();
                        if page == *max_page.get() {
                            return;
                        }

                        current_page.set(page + 1);
                        debug!("reading index {}", page);
                        img.set(imgs.get(page).cloned());
                    },
                    _ => {}
                }
            },
            if let Some(img) = img.get() {
                rsx!(img {
                    class: "h-[calc(100%-2rem)]",
                    src: "data:image/png;base64,{img}"
                })
            }
            div {
                class: "flex flex-row items-center justify-center gap-1 h-8",
                button {
                    class: "border rounded-sm px-2 py-1 bg-gray-100 hover:bg-gray-50 cursor-pointer",
                    onclick: move |_evt| {
                        let page = *current_page.get();
                        if page == 1 {
                            return;
                        }

                        current_page.set(page - 1);
                        debug!("reading index {}", page - 2);
                        img.set(imgs.get(page - 2).cloned());
                    },
                    "prev"
                },
                span {
                    class: "border rounded-sm px-2 py-1 bg-gray-100",
                     "{current_page} / {max_page}"
                },
                button {
                    class: "border rounded-sm px-2 py-1 bg-gray-100 hover:bg-gray-50 cursor-pointer",
                    onclick: move |_evt| {
                        let page = *current_page.get();
                        if page == *max_page.get() {
                            return;
                        }

                        current_page.set(page + 1);
                        debug!("reading index {}", page);
                        img.set(imgs.get(page).cloned());
                    },
                    "next"
                },
            }
        }
    })
}
