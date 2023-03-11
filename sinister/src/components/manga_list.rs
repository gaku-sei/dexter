use dexter_core::SearchData;
use dioxus::prelude::*;

#[must_use]
#[inline_props]
pub fn MangaList<'a>(
    cx: Scope,
    mangas: UseRef<Option<Vec<SearchData>>>,
    on_select: EventHandler<'a, String>,
) -> Element {
    let Some(mangas) = &*mangas.read() else {
        return None;
    };

    cx.render(rsx! {
        div {
            class: "flex flex-col overflow-y-auto",
            for manga in mangas.iter() {
                div {
                    class: "flex flex-row flex-shrink-0 items-center cursor-pointer h-8 w-full hover:bg-slate-600 px-2",
                    onclick: {
                        let manga_id = manga.id.clone();
                        move |_evt| on_select.call(manga_id.clone())
                    },
                    "{manga.attributes.title.en}"
                }
            }
        }
    })
}
