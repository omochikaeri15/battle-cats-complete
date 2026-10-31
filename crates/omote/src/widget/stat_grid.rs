use dioxus::prelude::*;

use super::Superscript;

const CELL_TEXT_SIZE: f32 = 13.0;

#[component]
pub(crate) fn StatHeader(label: String) -> Element {
    rsx! {
        div { class: "stat-cell header", "{label}" }
    }
}

#[component]
pub(crate) fn StatValue(header: String, value: String) -> Element {
    rsx! {
        div { class: "stat-cell value", title: "{header}: {value}", "{value}" }
    }
}

#[component]
pub(crate) fn StatFrames(header: String, frames: i32) -> Element {
    let seconds = frames as f32 / 30.0;
    let raw = format!("{seconds:.2}s^{frames}f");

    rsx! {
        div { class: "stat-cell value", title: "{header}: {raw}",
            Superscript { text: raw.clone(), size: CELL_TEXT_SIZE }
        }
    }
}
