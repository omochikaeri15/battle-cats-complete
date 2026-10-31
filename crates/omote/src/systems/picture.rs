use std::rc::Rc;

use dioxus::prelude::*;

#[component]
pub(crate) fn Pic(url: Rc<str>, #[props(default)] class: String, #[props(default)] style: String, #[props(default)] alt: String) -> Element {
    rsx! {
        img { class: "{class}", style: "{style}", src: "{url}", alt: "{alt}", draggable: false }
    }
}
