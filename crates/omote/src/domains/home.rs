use dioxus::prelude::*;
use tracing::warn;

use crate::app::Route;
use crate::platform::Platform;

const REPO: &str = "https://github.com/omochikaeri15/battle-cats-complete";
const SITE: &str = "https://omochikaeri.app";
const VERSION: &str = env!("CARGO_PKG_VERSION");
const COPIED: &str = "Link Copied!";
const DISCLAIMER: &str = "Not affiliated with PONOS Corp.";
const COPIED_MS: u32 = 2000;
const TIP_GAP: f64 = 4.0;

#[component]
pub(crate) fn Home() -> Element {
    let platform = use_context::<Platform>();
    let mut anchor: Signal<Option<MountedEvent>> = use_signal(|| None);
    let mut copied: Signal<Option<(f64, f64)>> = use_signal(|| None);
    let mut round = use_signal(|| 0u32);
    let share = move |_| {
        let platform = platform.clone();

        spawn(async move {
            if let Err(fault) = platform.host().copy_text(SITE).await {
                warn!("The site link could not be copied: {fault}");

                return;
            }

            let Some(anchor) = anchor() else {
                return;
            };
            let Ok(rect) = anchor.get_client_rect().await else {
                return;
            };
            let next = *round.peek() + 1;

            round.set(next);
            copied.set(Some((rect.origin.x + rect.size.width / 2.0, rect.origin.y - TIP_GAP)));
            platform.host().sleep(COPIED_MS).await;

            if *round.peek() == next {
                copied.set(None);
            }
        });
    };

    rsx! {
        div { class: "home",
            div { class: "home-main",
                div { class: "home-title", "Battle Cats Complete" }
                div { class: "home-subtitle weak", "All-In-One Battle Cats Toolkit" }
                div { class: "home-group", "Game" }
                div { class: "home-row",
                    Link { to: Route::Sandbox {}, class: "btn btn-primary home-nav", "Sandbox" }
                    Link { to: Route::Cats { segments: Vec::new() }, class: "btn btn-primary home-nav", "Cats" }
                    Link { to: Route::Enemies { segments: Vec::new() }, class: "btn btn-primary home-nav", "Enemies" }
                    Link { to: Route::Stages { segments: Vec::new() }, class: "btn btn-primary home-nav", "Stages" }
                }
                div { class: "home-group", "Data" }
                div { class: "home-row",
                    Link { to: Route::Mods {}, class: "btn btn-primary home-nav", "Mods" }
                }
                div { class: "home-group", "Other" }
                div { class: "home-row",
                    Link { to: Route::Settings {}, class: "btn btn-primary home-nav", "Settings" }
                }
            }
            div { class: "home-footer",
                div { class: "home-links home-links-left",
                    a { class: "btn btn-text", href: "{REPO}/releases/tag/v{VERSION}", target: "_blank", rel: "noopener", "v{VERSION}" }
                    span { class: "home-sep", "|" }
                    a { class: "btn btn-text", href: "{REPO}/releases", target: "_blank", rel: "noopener", "Changelogs" }
                }
                div { class: "home-disclaimer weak", "{DISCLAIMER}" }
                div { class: "home-links home-links-right",
                    button { class: "btn btn-text", onmounted: move |event| anchor.set(Some(event)), onclick: share, "Share" }
                    if let Some((x, y)) = copied() {
                        span { class: "tip-body bordered-box", style: "left: {x}px; top: {y}px;", "{COPIED}" }
                    }
                    span { class: "home-sep", "|" }
                    a { class: "btn btn-text", href: REPO, target: "_blank", rel: "noopener", "GitHub" }
                }
            }
        }
    }
}
