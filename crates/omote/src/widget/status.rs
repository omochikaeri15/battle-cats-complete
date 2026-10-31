use dioxus::prelude::*;

#[component]
pub(crate) fn Status(title: String, detail: Option<String>) -> Element {
    rsx! {
        div { class: "status",
            div { class: "title", "{title}" }
            if let Some(detail) = detail {
                div { class: "detail", "{detail}" }
            }
        }
    }
}
