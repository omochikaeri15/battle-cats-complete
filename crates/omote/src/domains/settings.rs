use std::collections::HashSet;
use std::sync::Arc;

use dioxus::prelude::*;

use codex::chapter::treasure::ARC_SLOTS;
use kokoro::domains::cat::files as cat_files;
use kokoro::domains::sandbox::keybind::Bind;
use kokoro::domains::settings::Settings as GameSettings;
use kokoro::domains::settings::{SidebarBehavior, lang};
use kokoro::systems::treasure::{ARCS, Catalog, Config as TreasureConfig, FULL_PERCENT};

use crate::app::remember_settings;
use crate::platform::Platform;
use crate::systems::animation::{self, Rigs};
use crate::systems::catalog;
use crate::systems::context::{self, Shared};
use crate::systems::images;
use crate::systems::library::Library;
use crate::widget::Toggle;

const CHAPTER_TITLES: [[&str; 3]; 3] = [
    ["Empire of Cats Ch. 1", "Empire of Cats Ch. 2", "Empire of Cats Ch. 3"],
    ["Into the Future Ch. 1", "Into the Future Ch. 2", "Into the Future Ch. 3"],
    ["Cats of the Cosmos Ch. 1", "Cats of the Cosmos Ch. 2", "Cats of the Cosmos Ch. 3"],
];
const ENTRY_DIGITS: usize = 4;
const NONE_CODE: &str = "--";
const LANGUAGE_ROW_HEIGHT: f64 = 32.0;

#[derive(Clone, Copy, PartialEq)]
enum Drag {
    Idle,
    Pressed { index: usize, last: f64 },
    Moving { index: usize, last: f64, carry: f64 },
}

fn dragged(drag: Drag) -> Option<usize> {
    match drag {
        Drag::Idle => None,
        Drag::Pressed { index, .. } | Drag::Moving { index, .. } => Some(index),
    }
}

fn carry(order: &mut [String], index: usize, carry: f64) -> (usize, f64) {
    let last = order.len().saturating_sub(1);
    let (mut index, mut carry) = (index, carry);

    while carry >= LANGUAGE_ROW_HEIGHT && index < last {
        order.swap(index, index + 1);
        index += 1;
        carry -= LANGUAGE_ROW_HEIGHT;
    }

    while carry <= -LANGUAGE_ROW_HEIGHT && index > 0 {
        order.swap(index, index - 1);
        index -= 1;
        carry += LANGUAGE_ROW_HEIGHT;
    }

    if (index == last && carry > 0.0) || (index == 0 && carry < 0.0) {
        carry = 0.0;
    }

    (index, carry)
}

#[derive(Clone, Copy, PartialEq, Eq, Default)]
enum Tab {
    #[default]
    General,
    Cats,
    Enemies,
    Stages,
    Sandbox,
}

const TABS: [(Tab, &str); 5] = [(Tab::General, "General"), (Tab::Cats, "Cats"), (Tab::Enemies, "Enemies"), (Tab::Stages, "Stages"), (Tab::Sandbox, "Sandbox")];
const KEY_PROMPT: &str = "Press a key...";
const KEY_UNBOUND: &str = "Unbound";
const SPACE: &str = " ";
const SPACE_NAME: &str = "Space";

fn key_label(name: &str) -> String {
    match name {
        SPACE => SPACE_NAME.to_owned(),
        single if single.chars().count() == 1 => single.to_uppercase(),
        named => named.to_owned(),
    }
}

fn key_name(key: &Key) -> Option<String> {
    match key {
        Key::Character(typed) if typed == SPACE => Some(SPACE_NAME.to_owned()),
        Key::Character(typed) => Some(typed.to_lowercase()),
        Key::Unidentified => None,
        named => Some(format!("{named:?}")),
    }
}
const SIDEBAR_BEHAVIORS: [(SidebarBehavior, &str); 2] = [(SidebarBehavior::Cover, "Cover"), (SidebarBehavior::Push, "Push")];

#[derive(Clone)]
pub(crate) struct Treasury(pub(crate) Arc<Catalog>);

impl PartialEq for Treasury {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0)
    }
}

fn typable_percent(entry: &str) -> bool {
    entry.len() <= ENTRY_DIGITS && entry.chars().all(|glyph| glyph.is_ascii_digit() || glyph == '%')
}

fn apply(platform: &Platform, mut settings: Signal<GameSettings>, change: impl FnOnce(&mut GameSettings)) {
    change(&mut settings.write());
    remember_settings(platform, &settings.read());
}

#[component]
pub(crate) fn Settings() -> Element {
    let platform = use_context::<Platform>();
    let settings = context::settings();
    let globals = context::globals();
    let mut tab = use_signal(Tab::default);
    let current = tab();

    rsx! {
        div { class: "settings-page",
            div { class: "settings-tabs",
                for (which, label) in TABS {
                    button {
                        class: if current == which { "list-row filled selected settings-tab" } else { "list-row filled settings-tab" },
                        onclick: move |_| tab.set(which),
                        "{label}"
                    }
                }
            }
            div { class: "settings-body scroll",
                match current {
                    Tab::General => rsx! { General { platform: platform.clone(), settings, globals } },
                    Tab::Cats => rsx! { Cats { platform: platform.clone(), settings } },
                    Tab::Enemies => rsx! { Enemies { platform: platform.clone(), settings } },
                    Tab::Stages => rsx! { Stages { platform: platform.clone(), settings } },
                    Tab::Sandbox => rsx! { SandboxTab { platform: platform.clone(), settings } },
                }
            }
        }
    }
}

fn relocalize(platform: &Platform, settings: Signal<GameSettings>, library: Library, catalogs: catalog::Catalogs, pictures: &images::Pictures, rigs: &Rigs, order: Vec<String>) {
    let mut library = library;
    let mut catalogs = catalogs;

    apply(platform, settings, |next| next.general.language_priority = order.clone());
    library.relocalize(&order);
    catalogs.reset();
    pictures.forget_all();
    rigs.forget_all();
}

#[component]
fn General(platform: Platform, settings: Signal<GameSettings>, globals: Option<Shared>) -> Element {
    let library = use_context::<Library>();
    let catalogs = catalog::catalogs();
    let pictures = images::pictures();
    let rigs = animation::rigs();
    let mut drag = use_signal(|| Drag::Idle);
    let mut working: Signal<Option<Vec<String>>> = use_signal(|| None);
    let held = settings.read();
    let saved = held.general.language_priority.clone();
    let priority = working.read().clone().unwrap_or_else(|| saved.clone());
    let none_index = priority.iter().position(|code| code == NONE_CODE);
    let for_reset = platform.clone();
    let pictures_reset = pictures.clone();
    let rigs_reset = rigs.clone();
    let dragging = dragged(drag());
    let mut release = {
        let platform = platform.clone();
        let pictures = pictures.clone();
        let rigs = rigs.clone();

        move || {
            drag.set(Drag::Idle);

            if let Some(order) = working.take()
                && order != settings.read().general.language_priority
            {
                relocalize(&platform, settings, library, catalogs, &pictures, &rigs, order);
            }
        }
    };
    let mut release_up = release.clone();

    rsx! {
        if dragging.is_some() {
            div {
                class: "drag-scrim",
                onpointermove: move |event| {
                    let y = event.data().page_coordinates().y;

                    match drag() {
                        Drag::Idle => {}
                        Drag::Pressed { index, last: _ } => drag.set(Drag::Moving { index, last: y, carry: 0.0 }),
                        Drag::Moving { index, last, carry: held } => {
                            let mut order = working.read().clone().unwrap_or_else(|| settings.read().general.language_priority.clone());
                            let (index, held) = carry(&mut order, index, held + (y - last));

                            working.set(Some(order));
                            drag.set(Drag::Moving { index, last: y, carry: held });
                        }
                    }
                },
                onpointerup: move |_| release_up(),
                onpointercancel: move |_| release(),
            }
        }
        div { class: "settings-sections",
            div { class: "settings-section",
                div { class: "settings-heading", "Language Priority" }
                button { class: "btn btn-danger btn-status", onclick: move |_| relocalize(&for_reset, settings, library, catalogs, &pictures_reset, &rigs_reset, lang::default_priority()), "Set to Default" }
                div { class: "card-outlined language-list",
                    for (index, code) in priority.iter().enumerate() {
                        {
                            let weak = none_index.is_some_and(|none| index > none);
                            let bold = Some(index) == none_index;
                            let label = lang::get_label_for_code(code);
                            let class = match (bold, weak) {
                                (true, _) => "language-label bold",
                                (_, true) => "language-label weak",
                                _ => "language-label",
                            };
                            let row_class = if dragging == Some(index) { "language-row dragging" } else { "language-row" };

                            rsx! {
                                div { key: "{code}", class: "{row_class}",
                                    span {
                                        class: "drag-handle symbols",
                                        onpointerdown: move |event| {
                                            event.prevent_default();
                                            working.set(Some(settings.read().general.language_priority.clone()));
                                            drag.set(Drag::Pressed { index, last: event.data().page_coordinates().y });
                                        },
                                        "\u{2630}"
                                    }
                                    span { class: "{class}", "{label}" }
                                }
                            }
                        }
                    }
                }
            }
            div { class: "settings-section",
                div { class: "settings-heading", "Treasures" }
                Treasures {
                    config: settings.read().general.treasures.clone(),
                    catalog: globals.as_ref().map(|globals| Treasury(Arc::clone(&globals.0.catalog))),
                    onarc: { let platform = platform.clone(); move |(arc, value): (usize, String)| apply(&platform, settings, |next| next.general.treasures.set_arc(arc, value)) },
                    onlevel: { let platform = platform.clone(); move |(slot, index, level): (usize, usize, i32)| apply(&platform, settings, |next| next.general.treasures.set_level(slot, index, level)) },
                }
            }
        }
    }
}

#[component]
pub(crate) fn Treasures(config: TreasureConfig, catalog: Option<Treasury>, onarc: EventHandler<(usize, String)>, onlevel: EventHandler<(usize, usize, i32)>) -> Element {
    let mut open = use_signal(HashSet::<usize>::new);
    let placeholder = FULL_PERCENT.to_string();

    rsx! {
        div { class: "settings-sections",
            div { class: "settings-subsection",
                div { class: "subsection-title", "General" }
                for (arc, name) in ARCS.iter().enumerate() {
                    {
                        let entry = config.arcs[arc].clone();
                        let tuned = config.is_tuned(arc);

                        rsx! {
                            label { class: "entry-row",
                                span { class: "entry-label", "{name}" }
                                input { class: "input entry-input", r#type: "text", inputmode: "numeric", placeholder: "{placeholder}", value: "{entry}",
                                    oninput: move |event| {
                                        let value = event.value();

                                        if typable_percent(&value) {
                                            onarc.call((arc, value));
                                        }
                                    } }
                                if tuned {
                                    span { class: "note weak", "Adjusted per treasure below" }
                                }
                            }
                        }
                    }
                }
            }
            div { class: "settings-subsection",
                div { class: "subsection-title", "Advanced" }
                for (arc, slots) in ARC_SLOTS.iter().enumerate() {
                    for (chapter, slot) in slots.iter().copied().enumerate() {
                        {
                            let expanded = open.read().contains(&slot);
                            let title = CHAPTER_TITLES[arc][chapter];

                            rsx! {
                                div { class: "fold",
                                    button { class: "btn btn-text fold-head", onclick: move |_| {
                                        let mut held = open.write();

                                        if !held.remove(&slot) {
                                            held.insert(slot);
                                        }
                                    },
                                        span { class: "bold fold-title", "{title}" }
                                        if !expanded {
                                            span { class: "note weak", "(collapsed)" }
                                        }
                                    }
                                    div { class: "rule" }
                                    if expanded {
                                        match catalog.as_ref() {
                                            Some(catalog) => chapter_view(&config, &catalog.0, slot, onlevel),
                                            None => rsx! { div { class: "note weak", "Treasure data has not loaded yet" } },
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

fn chapter_view(config: &TreasureConfig, catalog: &Catalog, slot: usize, onlevel: EventHandler<(usize, usize, i32)>) -> Element {
    let groups = catalog.groups(slot);

    if groups.is_empty() {
        return rsx! { div { class: "note weak", "No treasure data was found for this chapter" } };
    }

    let grades = catalog.grades.clone();

    rsx! {
        div { class: "treasure-sets",
            for (group, set) in groups.iter().enumerate() {
                {
                    let members: Vec<i32> = set.members().collect();

                    if members.is_empty() {
                        rsx! {}
                    } else {
                        let name = catalog.set(slot, group).map_or_else(|| format!("Set {}", group + 1), str::to_owned);
                        let effect = catalog.effect(slot, group).map(str::to_owned);

                        rsx! {
                            div { class: "treasure-set",
                                div { class: "treasure-set-title",
                                    span { class: "bold", "{name}" }
                                    if let Some(effect) = effect {
                                        span { class: "note weak", "{effect}" }
                                    }
                                }
                                for stage in members {
                                    if let Ok(index) = usize::try_from(stage) {
                                        {
                                            let label = catalog.item(slot, stage).map_or_else(|| format!("Treasure {}", stage + 1), str::to_owned);
                                            let level = config.level(slot, index);
                                            let selected = usize::try_from(level).ok().and_then(|level| grades.get(level)).cloned().unwrap_or_default();
                                            let grades = grades.clone();

                                            rsx! {
                                                label { class: "treasure-row",
                                                    span { class: "treasure-label", "{label}" }
                                                    select { class: "combo", value: "{selected}",
                                                        onchange: move |event| {
                                                            let level = grades.iter().position(|grade| *grade == event.value()).unwrap_or(0) as i32;

                                                            onlevel.call((slot, index, level));
                                                        },
                                                        for grade in catalog.grades.iter() {
                                                            option { value: "{grade}", selected: *grade == selected, "{grade}" }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn Cats(platform: Platform, settings: Signal<GameSettings>) -> Element {
    let held = settings.read();
    let cat_data = held.cat_data.clone();
    let mut level_buffer = use_signal(|| cat_data.default_level.to_string());
    let p = [platform.clone(), platform.clone(), platform.clone(), platform.clone(), platform.clone(), platform.clone()];

    rsx! {
        div { class: "settings-sections",
            div { class: "settings-section",
                div { class: "settings-heading", "Cat List" }
                label { class: "setting-row inline",
                    span { "Preferred Banner Form" }
                    select { class: "combo", value: "{cat_files::form_name(cat_data.preferred_banner_form)}",
                        onchange: {
                            let platform = p[0].clone();
                            move |event| {
                                if let Some(form) = (0..cat_files::FORM_COUNT).find(|form| cat_files::form_name(*form) == event.value()) {
                                    apply(&platform, settings, |next| next.cat_data.preferred_banner_form = form);
                                }
                            }
                        },
                        for form in 0..cat_files::FORM_COUNT {
                            option { value: "{cat_files::form_name(form)}", selected: form == cat_data.preferred_banner_form, "{cat_files::form_name(form)}" }
                        }
                    }
                }
                Toggle { value: cat_data.show_invalid_cats, label: "Show Invalid Cats", onchange: { let platform = p[1].clone(); move |value| apply(&platform, settings, |next| next.cat_data.show_invalid_cats = value) } }
            }
            div { class: "settings-section",
                div { class: "settings-heading", "Ability Display" }
                Toggle { value: cat_data.expand_spirit_details, label: "Expand Spirit Details by Default", onchange: { let platform = p[2].clone(); move |value| apply(&platform, settings, |next| next.cat_data.expand_spirit_details = value) } }
            }
            div { class: "settings-section",
                div { class: "settings-heading", "Level Display" }
                label { class: "setting-row inline",
                    span { "Default Level" }
                    input { class: "input entry-input", r#type: "text", inputmode: "numeric", placeholder: "Level", value: "{level_buffer}", disabled: cat_data.auto_level_calculations,
                        oninput: {
                            let platform = p[3].clone();
                            move |event| {
                                let value = event.value();

                                if value.chars().all(|glyph| glyph.is_ascii_digit()) {
                                    level_buffer.set(value.clone());

                                    if let Ok(parsed) = value.parse::<i32>() {
                                        apply(&platform, settings, |next| next.cat_data.default_level = parsed.max(1));
                                    }
                                }
                            }
                        } }
                }
                Toggle {
                    value: cat_data.auto_level_calculations,
                    label: "Auto Level Calculations",
                    hint: "Automatically calculates the max reasonable level for a unit based on their level caps",
                    onchange: { let platform = p[4].clone(); move |value| apply(&platform, settings, |next| next.cat_data.auto_level_calculations = value) },
                }
                Toggle {
                    value: cat_data.bump_ultra_60,
                    label: "Lv60 For Ultra",
                    hint: "Automatically bumps the level to 60 (if not higher already) when an Ultra Form or Ultra Talent is selected",
                    onchange: { let platform = p[5].clone(); move |value| apply(&platform, settings, |next| next.cat_data.bump_ultra_60 = value) },
                }
            }
        }
    }
}

#[component]
fn Stages(platform: Platform, settings: Signal<GameSettings>) -> Element {
    let current = settings.read().stages.sidebar_behavior.clone();
    let label = SIDEBAR_BEHAVIORS.iter().find(|(behavior, _)| *behavior == current).map_or("Cover", |(_, label)| label);

    rsx! {
        div { class: "settings-sections",
            div { class: "settings-section",
                div { class: "settings-heading", "Stage List" }
                label { class: "setting-row",
                    span { "Sidebar Behavior" }
                    select { class: "combo", value: "{label}",
                        onchange: move |event| {
                            let chosen = SIDEBAR_BEHAVIORS.iter().find(|(_, label)| *label == event.value()).map(|(behavior, _)| behavior.clone());

                            if let Some(behavior) = chosen {
                                apply(&platform, settings, |next| next.stages.sidebar_behavior = behavior);
                            }
                        },
                        for (_, option) in SIDEBAR_BEHAVIORS {
                            option { value: "{option}", selected: option == label, "{option}" }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn Enemies(platform: Platform, settings: Signal<GameSettings>) -> Element {
    let show_invalid = settings.read().enemy_data.show_invalid_enemies;

    rsx! {
        div { class: "settings-sections",
            div { class: "settings-section",
                div { class: "settings-heading", "Enemy List" }
                Toggle { value: show_invalid, label: "Show Invalid Enemies", onchange: move |value| apply(&platform, settings, |next| next.enemy_data.show_invalid_enemies = value) }
            }
        }
    }
}

#[component]
fn SandboxTab(platform: Platform, settings: Signal<GameSettings>) -> Element {
    let held = settings.read();
    let banner_form = held.sandbox.banner_form;
    let keys = held.sandbox.keys.clone();

    drop(held);

    let mut capturing: Signal<Option<Bind>> = use_signal(|| None);
    let listening = capturing();
    let form_platform = platform.clone();
    let reset_platform = platform.clone();
    let capture_platform = platform.clone();

    rsx! {
        div { class: "settings-sections",
            div { class: "settings-section",
                div { class: "settings-heading", "Lineup List" }
                label { class: "setting-row inline",
                    span { "Lineup Banner Form" }
                    select { class: "combo", value: "{cat_files::form_name(banner_form)}",
                        onchange: move |event| {
                            if let Some(form) = (0..cat_files::FORM_COUNT).find(|form| cat_files::form_name(*form) == event.value()) {
                                apply(&form_platform, settings, |next| next.sandbox.banner_form = form);
                            }
                        },
                        for form in 0..cat_files::FORM_COUNT {
                            option { value: "{cat_files::form_name(form)}", selected: form == banner_form, "{cat_files::form_name(form)}" }
                        }
                    }
                }
            }
            div { class: "settings-section",
                div { class: "settings-heading", "Keybinds" }
                button { class: "btn btn-danger btn-status", onclick: move |_| apply(&reset_platform, settings, |next| next.sandbox.keys.reset()), "Set to Default" }
                div {
                    class: "keybind-list",
                    tabindex: "0",
                    onkeydown: move |event| {
                        let Some(bind) = *capturing.peek() else {
                            return;
                        };

                        event.prevent_default();

                        if event.data().is_auto_repeating() {
                            return;
                        }

                        if let Some(name) = key_name(&event.data().key()) {
                            apply(&capture_platform, settings, |next| next.sandbox.keys.set(bind, &name));
                        }

                        capturing.set(None);
                    },
                    onfocusout: move |_| capturing.set(None),
                    for bind in Bind::ALL {
                        {
                            let held = keys.key(bind);
                            let waiting = listening == Some(bind);
                            let shown = match (waiting, held.is_empty()) {
                                (true, _) => KEY_PROMPT.to_owned(),
                                (false, true) => KEY_UNBOUND.to_owned(),
                                (false, false) => key_label(held),
                            };
                            let class = if waiting { "btn btn-success keybind-btn" } else { "btn btn-neutral keybind-btn" };

                            rsx! {
                                div { key: "{bind.id()}", class: "keybind-row",
                                    span { class: "keybind-label", "{bind.label()}" }
                                    button { class: "{class}", onclick: move |_| capturing.set(Some(bind)), "{shown}" }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
