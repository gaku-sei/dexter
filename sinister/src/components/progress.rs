use dioxus::prelude::*;

#[must_use]
#[inline_props]
pub fn Progress(cx: Scope, label: String, percent: f32) -> Element {
    let left_size = 20.0 / 100.0 * *percent;
    let right_size = 20.0 - left_size;

    cx.render(rsx! {
        div {
            class: "flex flex-row relative h-8 w-80 flex-shrink-0",
            div {
                class: "h-full bg-green-800",
                style: "width: {left_size}rem",
            }
            div {
                class: "h-full bg-gray-400",
                style: "width: {right_size}rem",
            }
            div {
                class: "absolute text-white px-2 inset-0 w-full bg-transparent",
                title: "{label}",
                div { class: "leading-8 truncate", "{label}" }
            }
        }
    })
}
