use dioxus::prelude::*;

const GAP: f64 = 4.0;

#[component]
pub(crate) fn Tip(text: String, children: Element) -> Element {
    let mut open = use_signal(|| false);
    let mut anchor: Signal<Option<MountedEvent>> = use_signal(|| None);
    let mut at: Signal<Option<(f64, f64)>> = use_signal(|| None);
    let show = move || {
        let Some(anchor) = anchor() else {
            return;
        };

        spawn(async move {
            if let Ok(rect) = anchor.get_client_rect().await {
                at.set(Some((rect.origin.x + rect.size.width / 2.0, rect.origin.y - GAP)));
                open.set(true);
            }
        });
    };
    let shown = if open() { at() } else { None };

    rsx! {
        span {
            class: "tip",
            onmounted: move |event| anchor.set(Some(event)),
            onmouseenter: move |_| show(),
            onmouseleave: move |_| open.set(false),
            onclick: move |_| if open() { open.set(false) } else { show() },
            {children}
            if let Some((x, y)) = shown {
                span { class: "tip-body bordered-box", style: "left: {x}px; top: {y}px;", "{text}" }
            }
        }
    }
}
