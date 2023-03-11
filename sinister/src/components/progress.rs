use dioxus::prelude::*;

#[must_use]
#[inline_props]
pub fn Progress(cx: Scope, label: String, percent: u8) -> Element {
    cx.render(rsx! {
        div {
            class: "absolute flex flex-row items-center justify-center h-8 w-full bottom-0 bg-green-800 text-white z-50",
            "{label}: {percent}%"
        }
    })
}
