use dioxus::prelude::*;

#[component]
pub(crate) fn Toggle(value: bool, label: String, #[props(default)] hint: Option<String>, onchange: EventHandler<bool>) -> Element {
    let class = if value { "toggle on" } else { "toggle" };

    rsx! {
        button { class: "{class}", title: hint.unwrap_or_default(), onclick: move |_| onchange.call(!value),
            span { class: "toggle-track" }
            span { "{label}" }
        }
    }
}
