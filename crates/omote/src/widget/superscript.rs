use dioxus::prelude::*;

use kokoro::common::superscript::{Segment, segments};

const SUPERSCRIPT_SHRINK: f32 = 3.0;

#[component]
pub(crate) fn Superscript(text: String, size: f32, #[props(default)] bold: bool, #[props(default)] white: bool) -> Element {
    let small = (size - SUPERSCRIPT_SHRINK).max(1.0);
    let mut class = String::from("sup-text");

    if bold {
        class.push_str(" bold");
    }

    if white {
        class.push_str(" white");
    }

    rsx! {
        div { class: "{class}", style: "font-size: {size}px;",
            for line in text.split('\n') {
                div { class: "sup-line",
                    for segment in segments(line) {
                        match segment {
                            Segment::Plain(body) => rsx! { span { "{body}" } },
                            Segment::Raised(body) => rsx! { span { class: "sup-raised", style: "font-size: {small}px;", "{body}" } },
                        }
                    }
                }
            }
        }
    }
}
