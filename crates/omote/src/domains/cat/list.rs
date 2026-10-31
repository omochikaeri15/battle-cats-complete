use std::path::PathBuf;
use std::rc::Rc;

use dioxus::prelude::*;

use kokoro::domains::cat::filter::evaluation;
use kokoro::domains::cat::scanner::CatEntry;

use crate::app::Route;
use crate::platform::Picture;
use crate::systems::picture::Pic;
use crate::systems::images::{self, Pictures};
use crate::widget::RowList;

use super::{FORM_LABELS, state};

#[derive(Clone)]
pub(crate) struct Entries(pub(crate) Rc<Vec<CatEntry>>);

impl PartialEq for Entries {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }
}

fn matches(cat: &CatEntry, query: &str) -> bool {
    cat.base_id_str().contains(query) || cat.names.iter().flatten().any(|name| name.to_lowercase().contains(query))
}

fn tooltip(cat: &CatEntry) -> String {
    let mut lines = vec![format!("[ID] {}", cat.base_id_str())];

    for (index, label) in FORM_LABELS.iter().enumerate() {
        if cat.forms[index] {
            lines.push(format!("[{label}] {}", cat.display_name(index)));
        }
    }

    lines.join("\n")
}

#[component]
pub(crate) fn Roster(cats: Entries, selected: Option<u32>) -> Element {
    let mut state = state();
    let search = (state.search)();
    let active = state.filter.read().is_active();
    let shown = use_memo(use_reactive!(|cats| {
        let query = state.search.read().to_lowercase();
        let filter = state.filter.read();

        Rc::new(
            cats.0
                .iter()
                .enumerate()
                .filter(|(_, cat)| evaluation::entity_passes_filter(cat, &filter))
                .filter(|(_, cat)| query.is_empty() || matches(cat, &query))
                .map(|(index, _)| index)
                .collect::<Vec<usize>>(),
        )
    }));
    let shown: Rc<Vec<usize>> = shown();
    let frame_style = images::pictures().frame_url().map_or_else(String::new, |url| format!("--banner-frame: url({url});"));
    let filter_class = if active { "btn btn-toggle active roster-filter" } else { "btn btn-toggle roster-filter" };
    let count = shown.len();
    let entries = Rc::clone(&cats.0);
    let row = Callback::new(move |index: usize| {
        let Some(cat) = shown.get(index).and_then(|at| entries.get(*at)) else {
            return rsx! {};
        };

        rsx! {
            Row { key: "{cat.id}", id: cat.id, path: cat.image_path.clone(), tooltip: tooltip(cat), selected: selected == Some(cat.id) }
        }
    });

    rsx! {
        div { class: "roster-panel list-panel", style: "{frame_style}",
            button { class: "{filter_class}", onclick: move |_| state.filter_open.set(true), "Filter" }
            input {
                class: "input roster-search",
                r#type: "search",
                placeholder: "Search Cat...",
                value: "{search}",
                oninput: move |event| state.search.set(event.value()),
            }
            RowList { count, row, empty: "No Cats Found!" }
        }
    }
}

#[component]
fn Row(id: u32, path: Option<PathBuf>, tooltip: String, selected: bool) -> Element {
    let pictures = images::pictures();
    let banner = path.as_ref().and_then(|path| pictures.banner(Picture::CatBanner, path));
    let wanted = path.as_ref().and_then(|path| Pictures::banner_key(&Picture::CatBanner, path));
    let forget = pictures.clone();

    use_drop(move || {
        if let Some(key) = &wanted {
            forget.unwant(key);
        }
    });
    let class = if selected { "list-row roster-row selected" } else { "list-row roster-row" };

    rsx! {
        Link { to: Route::cat(id), class: "{class}", title: "{tooltip}",
            div { class: "banner-slot",
                if let Some(url) = banner {
                    Pic { url, class: "banner" }
                }
            }
        }
    }
}
