use dioxus::prelude::*;

#[must_use]
pub fn Loader(cx: Scope) -> Element {
    cx.render(rsx!(span { class: "loader" }))
}
