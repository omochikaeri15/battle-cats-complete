use dioxus::prelude::*;
use tracing::debug;

use crate::platform::Platform;
use crate::systems::library::Library;
use crate::systems::{animation, images};

const HOLD_MS: u32 = 3000;

#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) struct Live(pub(crate) bool);

#[component]
fn Slot(live: bool, children: Element) -> Element {
    use_context_provider(|| Live(live));

    rsx! {
        {children}
    }
}

#[derive(Clone)]
struct Page {
    stamp: String,
    content: Element,
}

#[component]
pub(crate) fn Hold(stamp: String, children: Element) -> Element {
    let library = use_context::<Library>();
    let platform = use_context::<Platform>();
    let pictures = images::pictures();
    let rigs = animation::rigs();
    let mut shown: Signal<Option<Page>> = use_signal(|| None);
    let mut pending: Signal<Option<Page>> = use_signal(|| None);
    let mut round = use_signal(|| 0u32);
    let current = Page { stamp: stamp.clone(), content: children };

    let same = shown.peek().as_ref().map(|page| page.stamp == stamp);

    match same {
        None => shown.set(Some(current.clone())),
        Some(true) => {
            let waiting = pending.peek().is_some();

            if waiting {
                pending.set(None);
            }

            shown.set(Some(current.clone()));
        }
        Some(false) => {
            let fresh = pending.peek().as_ref().is_none_or(|page| page.stamp != stamp);

            pending.set(Some(current));

            if fresh {
                let next = *round.peek() + 1;

                round.set(next);
                spawn(async move {
                    platform.host().sleep(HOLD_MS).await;

                    let due = *round.peek() == next;
                    let page = if due { pending.peek().clone() } else { None };

                    if let Some(page) = page {
                        shown.set(Some(page));
                        pending.set(None);
                    }
                });
            }
        }
    }

    use_effect(move || {
        let _ = library.generation();
        let _ = pictures.watch();
        rigs.watch();
        let waiting = pending.read().is_some();
        let busy = library.busy();

        if !waiting || busy || library.waiting() || !pictures.settled() || !rigs.settled() {
            return;
        }

        let page = pending.peek().clone();

        if let Some(page) = page {
            debug!(stamp = %page.stamp, "A held page is being shown");
            shown.set(Some(page));
            pending.set(None);
        }
    });

    let visible = shown().map(|page| page.content);
    let hidden = pending().map(|page| page.content);

    rsx! {
        div { class: "hold",
            div { class: "hold-shown", Slot { live: true, {visible} } }
            if let Some(content) = hidden {
                div { class: "hold-pending", Slot { live: false, {content} } }
            }
        }
    }
}
