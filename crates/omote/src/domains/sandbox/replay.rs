use std::collections::HashMap;
use std::rc::Rc;

use dioxus::core::spawn_forever;
use dioxus::prelude::*;

use kokoro::common::frames;
use kokoro::domains::cat::scanner::{self, CatEntry};
use kokoro::domains::sandbox::replay::{Save, Unit};

use crate::domains::cat::{self, SelectedCat};
use crate::domains::settings;
use crate::platform::Platform;
use crate::platform::battle::Filming;
use crate::systems::context::{self, Shared};
use crate::systems::images;
use crate::systems::picture::Pic;
use crate::widget::Superscript;

use super::arena::{self, Video, Work};
use super::orbs::{self, Book, Shelf};
use super::{Cats, Dialog};

const LATEST_LABEL: &str = "Latest Battle";
const UPLOADED_LABEL: &str = "Uploaded Battle";
const BCV_ACCEPT: &str = ".bcv";
const BCV_SUFFIX: &str = ".bcv";
const LABEL_SIZE: f32 = 14.0;
const MEGABYTE: f64 = 1024.0 * 1024.0;
const COLUMNS: usize = 5;
const LINES: usize = 2;
const CONTENT_WIDTH: f32 = 515.0;
const CARD_PADDING: f32 = 12.0;
const SMALLEST: f32 = 0.45;
const TEXT_FLOOR: f32 = 0.8;
const CELL_WIDTH: f32 = 110.0;
const CELL_HEIGHT: f32 = 85.0;
const CELL_GAP: f32 = 6.0;
const BADGE_SIZE: f32 = 12.0;
const ORB_SIZE: f32 = 26.0;
const ORB_STEP: f32 = 0.55;
const COIN_SIZE: f32 = 18.0;
const SLOT_ORB_SIZE: f32 = 44.0;
const FIRST_TALENT_FORM: usize = 2;
const DOUBLE_CLICK_MS: f64 = 400.0;
const NP_ICON: &str = "gatyaitemD_07_f.png";
const FORM_LETTERS: [char; 4] = ['f', 'c', 's', 'u'];
const CROWN_GLYPH: &str = "\u{1F732}";
const UNIT_WIDTH: f32 = 760.0;
const EXPORT_LABELS: [&str; 3] = ["Export to BCV", "Exporting...", "Exported!"];
const UPLOAD_LABELS: [&str; 3] = ["Upload", "Uploading...", "Uploaded!"];
const FAILURE_LABEL: &str = "Failed!";
const DELETE_LABEL: &str = "Delete";
const CONFIRM_LABEL: &str = "Continue?";
const CONFIRM_MS: u32 = 2000;
const VIDEO_LABEL: &str = "Export to MP4";
const ABORT_LABEL: &str = "Abort Export";
const ABORTING_LABEL: &str = "Aborting...";
const TERMINATED_LABEL: &str = "Export Terminated!";
const NO_VIDEO_HINT: &str = "Requires a browser that can encode video";
const QUALITY_LABEL: &str = "Quality %";
const COMPRESSION_LABEL: &str = "Compression %";
const QUALITY_HINT: &str = "80";
const COMPRESSION_HINT: &str = "30";
const DEFAULT_QUALITY: u32 = 80;
const DEFAULT_COMPRESSION: u32 = 30;
const PERCENT_MIN: u32 = 1;
const PERCENT_MAX: u32 = 100;
const FULL: &str = "replay-manage";
const HALF: &str = "replay-half";

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Confirm {
    Upload,
    Delete,
}
const READY_LABEL: &str = "Ready";
const MISSING: &str = "Missing";
const GAME_LABELS: [&str; 4] = ["App", "Version", "Map", "Stage"];
const UNKNOWN: &str = "Unknown";

#[derive(Clone, PartialEq)]
struct Tile {
    id: u32,
    form: usize,
    level: String,
    cost: Option<String>,
    inspected: String,
    talents: Vec<(i32, i32)>,
    talented: bool,
    orbs: Vec<u32>,
    icon: Option<String>,
}

fn clamped(level: u32, plus: u32, altar_cap: Option<i32>) -> (u32, u32) {
    match altar_cap {
        Some(cap) => (u32::try_from(cap).map_or(level, |cap| level.min(cap.saturating_add(1))), 0),
        None => (level, plus),
    }
}

fn tile(unit: &Unit, save: &Save, slot: usize) -> Tile {
    let (level, plus) = clamped(unit.level, unit.plus, save.altar_cap);
    let mut worn = unit.orbs.clone();

    worn.sort_unstable();

    Tile {
        id: u32::try_from(unit.unit).unwrap_or(0),
        form: usize::try_from(unit.form).unwrap_or(0),
        level: if plus == 0 { format!("Lv{level}") } else { format!("Lv{level}+{plus}") },
        cost: save.costs.get(slot).copied().filter(|cost| *cost >= 0).map(|cost| format!("{cost}\u{00a2}")),
        inspected: if plus == 0 { level.to_string() } else { format!("{level}+{plus}") },
        talents: unit.talents.clone(),
        talented: !unit.talents.is_empty(),
        orbs: worn.into_iter().filter_map(|(_, orb)| u32::try_from(orb).ok()).collect(),
        icon: save.icons.get(slot).filter(|name| !name.is_empty()).cloned(),
    }
}

fn work_button<'a>(work: Work, labels: [&'a str; 3], width: &str) -> (&'a str, String) {
    let (label, style) = match work {
        Work::Idle => (labels[0], "btn-primary"),
        Work::Running => (labels[1], "btn-status-warning"),
        Work::Done => (labels[2], "btn-status-success"),
        Work::Failed => (FAILURE_LABEL, "btn-status-danger"),
    };

    (label, format!("btn {style} {width}"))
}

fn video_button(video: Video) -> (&'static str, &'static str) {
    match video {
        Video::Idle => (VIDEO_LABEL, "btn btn-primary replay-manage"),
        Video::Running => (ABORT_LABEL, "btn btn-danger replay-manage"),
        Video::Aborting => (ABORTING_LABEL, "btn btn-danger replay-manage"),
        Video::Done => (EXPORT_LABELS[2], "btn btn-status-success replay-manage"),
        Video::Failed => (FAILURE_LABEL, "btn btn-status-danger replay-manage"),
        Video::Terminated => (TERMINATED_LABEL, "btn btn-status-danger replay-manage"),
    }
}

fn percentage(input: &str) -> Option<(Option<u32>, String)> {
    if !input.chars().all(|glyph| glyph.is_ascii_digit()) {
        return None;
    }

    if input.is_empty() {
        return Some((None, String::new()));
    }

    let clamped = input.parse::<u32>().unwrap_or(PERCENT_MAX).clamp(PERCENT_MIN, PERCENT_MAX);

    Some((Some(clamped), clamped.to_string()))
}

fn arm(platform: &Platform, mut confirming: Signal<Option<Confirm>>, mut round: Signal<u32>, which: Confirm) -> bool {
    let next = *round.peek() + 1;

    round.set(next);

    if *confirming.peek() == Some(which) {
        confirming.set(None);

        return true;
    }

    confirming.set(Some(which));

    let sleeper = platform.clone();

    spawn(async move {
        sleeper.host().sleep(CONFIRM_MS).await;

        if *round.peek() == next {
            confirming.set(None);
        }
    });

    false
}

fn bundle_name(file: &str) -> String {
    let stem = file.len().checked_sub(BCV_SUFFIX.len()).filter(|&cut| file.get(cut..).is_some_and(|tail| tail.eq_ignore_ascii_case(BCV_SUFFIX))).and_then(|cut| file.get(..cut));

    stem.unwrap_or(file).to_owned()
}

fn scale() -> f32 {
    let grid = CELL_WIDTH * COLUMNS as f32 + CELL_GAP * (COLUMNS as f32 - 1.0);

    ((CONTENT_WIDTH - CARD_PADDING * 2.0) / grid).clamp(SMALLEST, 1.0)
}

fn talent_levels(cat: &CatEntry, talents: &[(i32, i32)]) -> HashMap<u8, u8> {
    let Some(groups) = cat.talent_data.as_ref() else {
        return HashMap::new();
    };

    talents
        .iter()
        .filter_map(|(ability, level)| {
            let index = groups.groups.iter().position(|group| i32::from(group.ability_id) == *ability)?;

            Some((u8::try_from(index).ok()?, u8::try_from(*level).ok()?))
        })
        .collect()
}

#[component]
fn Card(title: String, children: Element) -> Element {
    rsx! {
        div { class: "card-outlined replay-card",
            div { class: "section",
                div { class: "section-title", "{title}" }
                div { class: "rule" }
                {children}
            }
        }
    }
}

#[component]
fn Field(label: String, children: Element) -> Element {
    rsx! {
        div { class: "replay-field",
            span { class: "replay-label", "{label}" }
            {children}
        }
    }
}

#[component]
pub(crate) fn Replays(globals: Shared, cats: Option<Cats>) -> Element {
    let platform = use_context::<Platform>();
    let battle = arena::battle();
    let mut cat_state = cat::provide_locked();
    let pictures = images::pictures();
    let shared = globals.clone();
    let shelf = use_memo(use_reactive!(|shared| Shelf(Rc::new(Book::load(&shared)))));
    let shelf = shelf();
    let mut open: Signal<Option<usize>> = use_signal(|| None);
    let mut clicked: Signal<Option<(usize, f64)>> = use_signal(|| None);
    let latest = (battle.latest)();
    let exporting = (battle.exporting)();
    let uploading = (battle.uploading)();
    let job = (battle.job)();
    let video = (battle.video)();
    let settings = context::settings();
    let confirming: Signal<Option<Confirm>> = use_signal(|| None);
    let confirm_round = use_signal(|| 0u32);
    let mut quality_text = use_signal(|| settings.peek().sandbox.video_quality.map(|value| value.to_string()).unwrap_or_default());
    let mut compression_text = use_signal(|| settings.peek().sandbox.video_compression.map(|value| value.to_string()).unwrap_or_default());
    let vault = globals.0.vault();
    let vfs = &vault.vfs;
    let save = latest.as_ref().map(|recorded| &recorded.save);
    let tiles: Vec<Tile> = save.map_or_else(Vec::new, |save| save.setup.lineup.iter().enumerate().map(|(slot, unit)| tile(unit, save, slot)).collect());
    let roster = cats.as_ref().map(|cats| Rc::clone(&cats.0));
    let cat_of = |id: u32| roster.as_ref().and_then(|roster| roster.iter().find(|cat| cat.id == id));
    let icon_of = |tile: &Tile| -> Option<Rc<str>> {
        let drawn = tile.icon.as_deref().and_then(|name| vfs.find(name));
        let scanned = cat_of(tile.id).and_then(|cat| scanner::deploy_icon(vfs, cat, tile.form));
        let path = drawn.or(scanned).or_else(|| FORM_LETTERS.get(tile.form).and_then(|letter| vfs.find(format!("uni{:03}_{letter}00.png", tile.id).as_str())));

        path.and_then(|path| pictures.trimmed(&path)).map(|sized| sized.url)
    };
    let coin = vfs.find(NP_ICON).and_then(|path| pictures.trimmed(&path)).map(|sized| sized.url);
    let scale = scale();
    let (cell_width, cell_height, gap) = (CELL_WIDTH * scale, CELL_HEIGHT * scale, CELL_GAP * scale);
    let badge = BADGE_SIZE * scale.max(TEXT_FLOOR);
    let orb_size = ORB_SIZE * scale;
    let time = latest.as_ref().map_or_else(|| MISSING.to_owned(), |recorded| frames::label(i32::try_from(recorded.frames).unwrap_or(i32::MAX)));
    let screen = save.map_or_else(|| MISSING.to_owned(), |save| format!("{} \u{00d7} {}{}", save.screen.width, save.screen.height, if save.screen.phone { " (Phone)" } else { "" }));
    let assets = latest.as_ref().map_or_else(|| MISSING.to_owned(), |recorded| format!("{} files", recorded.assets));
    let size = latest.as_ref().map_or_else(|| MISSING.to_owned(), |recorded| format!("{:.1} MB", recorded.bytes as f64 / MEGABYTE));
    let ready = latest.is_some();
    let filming = matches!(video, Video::Running | Video::Aborting);
    let busy = exporting == Work::Running || uploading == Work::Running || filming;
    let can_film = platform.host().can_film();
    let confirm = confirming();
    let (export_label, export_class) = work_button(exporting, EXPORT_LABELS, FULL);
    let (upload_label, upload_class) = match (uploading, confirm) {
        (Work::Idle, Some(Confirm::Upload)) => (CONFIRM_LABEL, format!("btn btn-primary {HALF}")),
        (work, _) => work_button(work, UPLOAD_LABELS, HALF),
    };
    let delete_label = if confirm == Some(Confirm::Delete) { CONFIRM_LABEL } else { DELETE_LABEL };
    let (video_label, video_class) = video_button(video);
    let video_enabled = match video {
        Video::Running => true,
        Video::Aborting => false,
        _ => ready && !busy && can_film,
    };
    let video_hint = (!can_film).then_some(NO_VIDEO_HINT);
    let (status, ratio) = job.map_or_else(|| (READY_LABEL.to_owned(), 100.0), |job| (job.label, job.ratio * 100.0));
    let title = match latest.as_ref().and_then(|recorded| recorded.uploaded.as_deref()) {
        Some("") => UPLOADED_LABEL.to_owned(),
        Some(name) => name.to_owned(),
        None => LATEST_LABEL.to_owned(),
    };
    let exporter = platform.clone();
    let filmer = platform.clone();
    let quality_platform = platform.clone();
    let compression_platform = platform.clone();
    let uploader = platform.clone();
    let deleter = platform.clone();
    let opened = open().and_then(|slot| tiles.get(slot).cloned()).and_then(|tile| cat_of(tile.id).cloned().map(|cat| (tile, Rc::new(cat))));

    rsx! {
        div { class: "replay-page scroll",
            input { class: "input replay-title", r#type: "text", readonly: true, value: "{title}" }
            div { class: "rule replay-rule" }
            Card { title: "Meta",
                Field { label: "Time",
                    if latest.is_some() {
                        Superscript { text: time.clone(), size: LABEL_SIZE }
                    } else {
                        span { "{time}" }
                    }
                }
                Field { label: "Screen", span { "{screen}" } }
                Field { label: "Assets", span { "{assets}" } }
                Field { label: "Size", span { "{size}" } }
            }
            Card { title: "Game",
                match save {
                    Some(save) => {
                        let entry = &save.setup.stage;
                        let map = if entry.map_name.is_empty() { format!("Map {}", entry.map_id) } else { entry.map_name.clone() };
                        let named = if entry.stage_name.is_empty() { format!("Stage {}", entry.stage + 1) } else { entry.stage_name.clone() };
                        let app = if save.app.is_empty() { UNKNOWN.to_owned() } else { save.app.clone() };
                        let version = if save.version.is_empty() { UNKNOWN.to_owned() } else { save.version.clone() };
                        let crowns = entry.crown + 1;

                        rsx! {
                            Field { label: "App", span { "{app}" } }
                            Field { label: "Version", span { "{version}" } }
                            Field { label: "Map", span { "{map}" } }
                            Field { label: "Stage",
                                span { "{named} [{crowns}" }
                                span { class: "symbols", "{CROWN_GLYPH}" }
                                span { "]" }
                            }
                        }
                    }
                    None => rsx! {
                        for label in GAME_LABELS {
                            Field { key: "{label}", label: label.to_owned(), span { "{MISSING}" } }
                        }
                    },
                }
            }
            Card { title: "Lineup",
                div { class: "deck-grid", style: "gap: {gap}px;",
                    for line in 0..LINES {
                        div { key: "{line}", class: "deck-row", style: "gap: {gap}px;",
                            for column in 0..COLUMNS {
                                {
                                    let slot = line * COLUMNS + column;
                                    let size_style = format!("width: {cell_width}px; height: {cell_height}px;");

                                    match tiles.get(slot) {
                                        None => rsx! { div { key: "{slot}", class: "deck-cell", style: "{size_style}" } },
                                        Some(held) => {
                                            let icon = icon_of(held);
                                            let fallback = format!("{:03}-{}", held.id, held.form + 1);
                                            let coin = coin.clone().filter(|_| held.talented);
                                            let presser = platform.clone();
                                            let seeded = held.clone();
                                            let roster = roster.clone();

                                            rsx! {
                                                div {
                                                    key: "{slot}",
                                                    class: "deck-cell filled clickable",
                                                    style: "{size_style}",
                                                    onclick: move |_| {
                                                        let now = presser.host().now();
                                                        let doubled = clicked.take().is_some_and(|(held, at)| held == slot && now - at <= DOUBLE_CLICK_MS);

                                                        if doubled {
                                                            if let Some(cat) = roster.as_ref().and_then(|roster| roster.iter().find(|cat| cat.id == seeded.id)) {
                                                                cat_state.seed(cat, seeded.form, &seeded.inspected, &talent_levels(cat, &seeded.talents));
                                                            }

                                                            open.set(Some(slot));
                                                        } else {
                                                            clicked.set(Some((slot, now)));
                                                        }
                                                    },
                                                    match icon {
                                                        Some(url) => rsx! { Pic { url, class: "deck-face" } },
                                                        None => rsx! { div { class: "deck-fallback", style: "font-size: {badge}px;", "{fallback}" } },
                                                    }
                                                    div { class: "deck-orbs", style: "height: {orb_size}px;",
                                                        for (place, orb) in held.orbs.iter().enumerate() {
                                                            div { key: "{place}", class: "deck-orb", style: "left: {place as f32 * orb_size * ORB_STEP}px;",
                                                                {orbs::picture(&pictures, &shelf.0.orbs, *orb, orb_size)}
                                                            }
                                                        }
                                                    }
                                                    if let Some(coin) = coin {
                                                        Pic { url: coin, class: "deck-coin", style: format!("height: {}px;", COIN_SIZE * scale) }
                                                    }
                                                    div { class: "deck-badge deck-level", style: "font-size: {badge}px;", "{held.level}" }
                                                    if let Some(cost) = held.cost.as_ref() {
                                                        div { class: "deck-badge deck-cost", style: "font-size: {badge}px;", "{cost}" }
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
            Card { title: "Manage",
                div { class: "replay-manage-column",
                    div { class: "replay-manage-buttons",
                        button { class: "{export_class}", disabled: !ready || busy, onclick: move |_| battle.export(&exporter), "{export_label}" }
                        div { class: "replay-hint", title: video_hint,
                            button {
                                class: "{video_class}",
                                disabled: !video_enabled,
                                onclick: move |_| match video {
                                    Video::Running => battle.abort_film(&filmer),
                                    Video::Aborting => {}
                                    _ => {
                                        let held = settings.peek();
                                        let filming = Filming {
                                            source: held.sandbox.replay_source,
                                            priority: held.general.language_priority.clone(),
                                            quality: held.sandbox.video_quality.unwrap_or(DEFAULT_QUALITY),
                                            compression: held.sandbox.video_compression.unwrap_or(DEFAULT_COMPRESSION),
                                        };

                                        drop(held);
                                        battle.film(&filmer, filming);
                                    }
                                },
                                "{video_label}"
                            }
                        }
                        label { class: "replay-knob",
                            span { "{QUALITY_LABEL}" }
                            input {
                                class: "input replay-knob-input",
                                r#type: "text",
                                inputmode: "numeric",
                                placeholder: QUALITY_HINT,
                                value: "{quality_text}",
                                oninput: move |event| {
                                    if let Some((value, text)) = percentage(&event.value()) {
                                        settings::apply(&quality_platform, settings, |next| next.sandbox.video_quality = value);
                                        quality_text.set(text);
                                    }
                                },
                            }
                        }
                        label { class: "replay-knob",
                            span { "{COMPRESSION_LABEL}" }
                            input {
                                class: "input replay-knob-input",
                                r#type: "text",
                                inputmode: "numeric",
                                placeholder: COMPRESSION_HINT,
                                value: "{compression_text}",
                                oninput: move |event| {
                                    if let Some((value, text)) = percentage(&event.value()) {
                                        settings::apply(&compression_platform, settings, |next| next.sandbox.video_compression = value);
                                        compression_text.set(text);
                                    }
                                },
                            }
                        }
                        div { class: "replay-pair",
                            button {
                                class: "{upload_class}",
                                disabled: busy,
                                onclick: move |_| {
                                    let platform = uploader.clone();

                                    if ready && !arm(&platform, confirming, confirm_round, Confirm::Upload) {
                                        return;
                                    }

                                    let picking = platform.host().pick_file(BCV_ACCEPT, u64::MAX);

                                    spawn_forever(async move {
                                        match picking.await {
                                            Ok(Some(picked)) => match picked.bytes {
                                                Some(bytes) => battle.upload(&platform, bundle_name(&picked.name), bytes.to_vec()),
                                                None => battle.unreadable(&platform, format!("the file is {} bytes, too large to read", picked.size)),
                                            },
                                            Ok(None) => {}
                                            Err(fault) => battle.unreadable(&platform, format!("the file could not be read: {fault}")),
                                        }
                                    });
                                },
                                "{upload_label}"
                            }
                            button {
                                class: "btn btn-danger {HALF}",
                                disabled: !ready || busy,
                                onclick: move |_| {
                                    if arm(&deleter, confirming, confirm_round, Confirm::Delete) {
                                        battle.delete(&deleter);
                                    }
                                },
                                "{delete_label}"
                            }
                        }
                    }
                    div { class: "replay-status", "{status}" }
                    div { class: "progress", div { style: "width: {ratio}%;" } }
                }
            }
        }
        if let Some((held, cat)) = opened {
            {
                let header = (held.form >= FIRST_TALENT_FORM && !held.orbs.is_empty()).then(|| {
                    rsx! {
                        div { class: "orb-header",
                            span { class: "bold orb-header-title", "Orbs" }
                            div { class: "orb-slots",
                                for (place, orb) in held.orbs.iter().enumerate() {
                                    div { key: "{place}", class: "orb-slot static",
                                        {orbs::picture(&pictures, &shelf.0.orbs, *orb, SLOT_ORB_SIZE)}
                                    }
                                }
                            }
                        }
                    }
                });
                let title = cat.display_name(held.form);

                rsx! {
                    Dialog { title, width: UNIT_WIDTH, class: "unit-popup", onclose: move |()| open.set(None),
                        cat::Detail { cat: SelectedCat(Rc::clone(&cat)), globals: globals.clone(), extra: header }
                    }
                }
            }
        }
    }
}
