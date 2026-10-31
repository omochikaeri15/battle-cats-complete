use std::path::PathBuf;
use std::rc::Rc;

use dioxus::prelude::*;

use kokoro::domains::enemy::filter::evaluation::entity_passes_filter;
use kokoro::domains::enemy::scanner::EnemyEntry;

use crate::app::Route;
use crate::platform::Picture;
use crate::systems::picture::Pic;
use crate::systems::images::{self, Pictures};
use crate::widget::RowList;

use super::state;

#[derive(Clone)]
pub(crate) struct Entries(pub(crate) Rc<Vec<EnemyEntry>>);

impl PartialEq for Entries {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }
}

fn matches(enemy: &EnemyEntry, query: &str) -> bool {
    let by_id = query.chars().next().is_some_and(|glyph| glyph.is_ascii_digit());

    (by_id && enemy.id_str().to_lowercase().contains(query)) || enemy.name.to_lowercase().contains(query)
}

#[component]
pub(crate) fn Roster(enemies: Entries, selected: Option<u32>) -> Element {
    let mut state = state();
    let search = (state.search)();
    let active = state.filter.read().is_active();
    let shown = use_memo(use_reactive!(|enemies| {
        let query = state.search.read().to_lowercase();
        let filter = state.filter.read();

        Rc::new(
            enemies
                .0
                .iter()
                .enumerate()
                .filter(|(_, enemy)| entity_passes_filter(enemy, &filter))
                .filter(|(_, enemy)| query.is_empty() || matches(enemy, &query))
                .map(|(index, _)| index)
                .collect::<Vec<usize>>(),
        )
    }));
    let shown: Rc<Vec<usize>> = shown();
    let frame_style = images::pictures().frame_url().map_or_else(String::new, |url| format!("--banner-frame: url({url});"));
    let filter_class = if active { "btn btn-toggle active roster-filter" } else { "btn btn-toggle roster-filter" };

    let count = shown.len();
    let entries = Rc::clone(&enemies.0);
    let row = Callback::new(move |index: usize| {
        let Some(enemy) = shown.get(index).and_then(|at| entries.get(*at)) else {
            return rsx! {};
        };

        rsx! {
            Row {
                key: "{enemy.id}",
                id: enemy.id,
                path: enemy.icon_path.clone(),
                tooltip: format!("[ID] {}\n[Name] {}", enemy.id_str(), enemy.display_name()),
                selected: selected == Some(enemy.id),
            }
        }
    });

    rsx! {
        div { class: "roster-panel list-panel", style: "{frame_style}",
            button { class: "{filter_class}", onclick: move |_| state.filter_open.set(true), "Filter" }
            input {
                class: "input roster-search",
                r#type: "search",
                placeholder: "Search Enemy...",
                value: "{search}",
                oninput: move |event| state.search.set(event.value()),
            }
            RowList { count, row, empty: "No Enemies Found!" }
        }
    }
}

#[component]
fn Row(id: u32, path: Option<PathBuf>, tooltip: String, selected: bool) -> Element {
    let pictures = images::pictures();
    let banner = path.as_ref().and_then(|path| pictures.banner(Picture::EnemyBanner, path));
    let wanted = path.as_ref().and_then(|path| Pictures::banner_key(&Picture::EnemyBanner, path));
    let forget = pictures.clone();

    use_drop(move || {
        if let Some(key) = &wanted {
            forget.unwant(key);
        }
    });
    let class = if selected { "list-row roster-row selected" } else { "list-row roster-row" };

    rsx! {
        Link { to: Route::enemy(id), class: "{class}", title: "{tooltip}",
            div { class: "banner-slot",
                if let Some(url) = banner {
                    Pic { url, class: "banner" }
                }
            }
        }
    }
}
