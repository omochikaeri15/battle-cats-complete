use dioxus::prelude::*;

#[component]
pub(crate) fn RangeRow(label: String, min: String, max: String, on_min: EventHandler<String>, on_max: EventHandler<String>) -> Element {
    rsx! {
        div { class: "range-row",
            span { class: "range-label", "{label}:" }
            input { class: "input range-input", r#type: "text", inputmode: "numeric", placeholder: "Any", value: "{min}", oninput: move |event| on_min.call(event.value()) }
            span { "~" }
            input { class: "input range-input", r#type: "text", inputmode: "numeric", placeholder: "Any", value: "{max}", oninput: move |event| on_max.call(event.value()) }
        }
    }
}
