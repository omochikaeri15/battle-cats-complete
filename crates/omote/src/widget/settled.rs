use dioxus::prelude::*;

use crate::systems::library::Library;

use super::Status;

#[component]
pub(crate) fn Settled(children: Element) -> Element {
    let library = use_context::<Library>();
    let mut revealed = use_signal(|| false);

    use_effect(move || {
        let _ = library.generation();
        let busy = library.busy();

        if !busy && !library.waiting() && !*revealed.peek() {
            revealed.set(true);
        }
    });

    let shown = revealed();
    let class = if shown { "settled" } else { "settled veiled" };

    rsx! {
        div { class: "{class}",
            div { class: "settled-body", {children} }
            if !shown {
                Status { title: "Loading...", detail: None }
            }
        }
    }
}
