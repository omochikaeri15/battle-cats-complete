use std::collections::BTreeMap;
use std::rc::Rc;
use std::sync::Arc;

use dioxus::prelude::*;

use codex::chapter::Category;
use kokoro::domains::sandbox::base::{self, Parts};
use kokoro::domains::sandbox::config::{CatGod, FILIBUSTER_MAP, StartSpeed, Tutorial};
use kokoro::domains::sandbox::{ITEMS, TECHS};
use kokoro::domains::stage::{GlobalMapId, StageRegistry, navigate};

use crate::domains::settings::{Treasures, Treasury};
use crate::platform::Platform;
use crate::systems::context::Shared;
use crate::widget::{Tip, Toggle};

use super::Staged;
use super::state::{self, Device, Panel, QUALITY_RANGE, QUALITY_STEP, SandboxState, Scaling, Volume};

const LEVEL_DIGITS: usize = 6;
const NO_CAP: &str = "None";
const NATIVE: &str = "Native";
const QUALITY_HINT: &str = "The resolution the battle is drawn at.\nLower draws fewer pixels, trading sharpness for speed on weaker hardware.\nThe Cycle Image Quality key steps through 15, 25, 35, 50, 70 and 100%, then back to 15%, mid-battle";
const SCALING_HINT: &str = "How a battle drawn below 100% quality fills the window.\nStretch: scaled up to fill the window\nPad: shown at its drawn size in the middle of the window";
const NATIVE_HINT: &str = "No scaling options for native game size";
const VOLUME_HINT: &str = "The four volume steps the battle settings cycle through";
const PANELS: [(Panel, &str); 7] = [
    (Panel::Settings, "Settings"),
    (Panel::Treasure, "Treasure"),
    (Panel::Tech, "Tech"),
    (Panel::Base, "Base"),
    (Panel::Items, "Items"),
    (Panel::Invasions, "Invasions"),
    (Panel::Tutorial, "Tutorial"),
];
const ONE_ROW: &str = "One Row";
const TWO_ROWS: &str = "Two Rows";
const ON: &str = "On";
const OFF: &str = "Off";

#[derive(Clone, Copy, PartialEq, Eq)]
enum TutorialStep {
    BattleCleared,
    DeckSeen,
    TwoRowsSeen,
    CatGodSeen,
    ShopSeen,
}

const TUTORIAL_STEPS: [(TutorialStep, &str, &str); 5] = [
    (TutorialStep::BattleCleared, "First Battle", "Off replays the first battle's guided deployment and continue prompts"),
    (TutorialStep::DeckSeen, "Deck Guide", "Off shows the deck guide popup when a battle starts"),
    (TutorialStep::TwoRowsSeen, "Two Rows Guide", "Off shows the two-row deck guide popup when a battle starts"),
    (TutorialStep::CatGodSeen, "Cat God Guide", "Off shows the Cat God guide popup on the first Cat God open"),
    (TutorialStep::ShopSeen, "Shop Guide", "Off shows the Cat Food shop guide popup on the first shop open"),
];

fn tutorial_step(tutorial: &Tutorial, step: TutorialStep) -> bool {
    match step {
        TutorialStep::BattleCleared => tutorial.battle_cleared,
        TutorialStep::DeckSeen => tutorial.deck_seen,
        TutorialStep::TwoRowsSeen => tutorial.two_rows_seen,
        TutorialStep::CatGodSeen => tutorial.cat_god_seen,
        TutorialStep::ShopSeen => tutorial.shop_seen,
    }
}

fn set_tutorial_step(tutorial: &mut Tutorial, step: TutorialStep, cleared: bool) {
    match step {
        TutorialStep::BattleCleared => tutorial.battle_cleared = cleared,
        TutorialStep::DeckSeen => tutorial.deck_seen = cleared,
        TutorialStep::TwoRowsSeen => tutorial.two_rows_seen = cleared,
        TutorialStep::CatGodSeen => tutorial.cat_god_seen = cleared,
        TutorialStep::ShopSeen => tutorial.shop_seen = cleared,
    }
}

fn typable(entry: &str) -> bool {
    entry.len() <= LEVEL_DIGITS && entry.chars().all(|glyph| glyph.is_ascii_digit() || glyph == '+' || glyph == '%')
}

struct Labeled {
    cannons: Vec<(i32, String)>,
    styles: Vec<(i32, String)>,
    foundations: Vec<(i32, String)>,
    parts: Parts,
}

#[derive(Clone)]
struct Fitted(Rc<Labeled>);

impl PartialEq for Fitted {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }
}

fn offered(parts: &BTreeMap<i32, i32>, label: impl Fn(i32) -> String) -> Vec<(i32, String)> {
    parts.keys().map(|id| (*id, label(*id))).collect()
}

fn fit(globals: &Shared) -> Labeled {
    let vfs = &globals.0.vault().vfs;
    let names = &globals.0.names;
    let parts = Parts::load(vfs);
    let cannons = offered(&parts.cannons, |id| names.cannon(id).map_or_else(|| base::name(id), str::to_owned));
    let styles = offered(&parts.styles, |id| parts.style_label(id, names));
    let foundations = offered(&parts.foundations, |id| parts.foundation_label(id, names));

    Labeled { cannons, styles, foundations, parts }
}

fn invadable(registry: &StageRegistry, category: Category) -> Vec<(u32, String)> {
    let map = GlobalMapId { category, map: FILIBUSTER_MAP };

    navigate::invadable_stages(registry, &map).into_iter().map(|stage| (stage.stage_id, stage.name.clone())).collect()
}

#[component]
fn ComboRow(label: String, hint: String, options: Vec<String>, selected: String, #[props(default = true)] enabled: bool, onchange: EventHandler<String>) -> Element {
    rsx! {
        label { class: "setting-row inline",
            Tip { text: hint, span { "{label}" } }
            select { class: "combo", value: "{selected}", disabled: !enabled, onchange: move |event| onchange.call(event.value()),
                for option in options.iter() {
                    option { value: "{option}", selected: *option == selected, "{option}" }
                }
            }
        }
    }
}

#[component]
fn EntryRow(label: String, placeholder: String, value: String, oninput: EventHandler<String>) -> Element {
    rsx! {
        label { class: "entry-row",
            span { class: "entry-label", "{label}" }
            input { class: "input entry-input", r#type: "text", inputmode: "numeric", placeholder: "{placeholder}", value: "{value}",
                oninput: move |event| {
                    let entry = event.value();

                    if typable(&entry) {
                        oninput.call(entry);
                    }
                } }
        }
    }
}

#[component]
pub(crate) fn Config(globals: Shared, staged: Option<Staged>) -> Element {
    let platform = use_context::<Platform>();
    let sandbox = state::state();
    let held = sandbox.read();
    let panel = held.panel;

    drop(held);

    let tab_platform = platform.clone();

    rsx! {
        div { class: "settings-page sandbox-config",
            div { class: "settings-tabs",
                for (which, label) in PANELS {
                    {
                        let platform = tab_platform.clone();

                        rsx! {
                            button {
                                key: "{label}",
                                class: if panel == which { "list-row filled selected settings-tab" } else { "list-row filled settings-tab" },
                                onclick: move |_| state::apply(&platform, sandbox, |next| next.panel = which),
                                "{label}"
                            }
                        }
                    }
                }
            }
            div { class: "settings-body scroll config-body",
                match panel {
                    Panel::Settings => rsx! { GameSettings { platform: platform.clone(), sandbox } },
                    Panel::Treasure => rsx! { TreasurePanel { platform: platform.clone(), sandbox, globals: globals.clone() } },
                    Panel::Tech => rsx! { TechPanel { platform: platform.clone(), sandbox } },
                    Panel::Base => rsx! { BasePanel { platform: platform.clone(), sandbox, globals: globals.clone() } },
                    Panel::Items => rsx! { ItemsPanel { platform: platform.clone(), sandbox } },
                    Panel::Invasions => rsx! { InvasionsPanel { platform: platform.clone(), sandbox, staged: staged.clone() } },
                    Panel::Tutorial => rsx! { TutorialPanel { platform: platform.clone(), sandbox } },
                }
            }
        }
    }
}

fn volume_labels() -> Vec<String> {
    Volume::ALL.iter().map(ToString::to_string).collect()
}

fn volume_of(label: &str) -> Option<Volume> {
    Volume::ALL.into_iter().find(|step| step.to_string() == label)
}

#[component]
fn GameSettings(platform: Platform, sandbox: Signal<SandboxState>) -> Element {
    let held = sandbox.read();
    let music = Volume::nearest(held.music_volume).to_string();
    let effects = Volume::nearest(held.effects_volume).to_string();
    let vibration = if held.vibrate { ON } else { OFF };
    let formation = if held.two_rows { TWO_ROWS } else { ONE_ROW };
    let device = held.device.to_string();
    let native = held.native();
    let scaling = held.scaling().to_string();
    let quality = held.quality();

    drop(held);

    let p = [platform.clone(), platform.clone(), platform.clone(), platform.clone(), platform.clone(), platform.clone(), platform.clone()];

    rsx! {
        div { class: "settings-sections",
            div { class: "settings-section",
                div { class: "settings-heading", "Game" }
                ComboRow { label: "Music Volume", hint: VOLUME_HINT, options: volume_labels(), selected: music, onchange: { let platform = p[0].clone(); move |label: String| if let Some(step) = volume_of(&label) { state::apply(&platform, sandbox, |next| next.music_volume = step.percent()) } } }
                ComboRow { label: "Sound Volume", hint: VOLUME_HINT, options: volume_labels(), selected: effects, onchange: { let platform = p[1].clone(); move |label: String| if let Some(step) = volume_of(&label) { state::apply(&platform, sandbox, |next| next.effects_volume = step.percent()) } } }
                ComboRow { label: "Vibration", hint: "Whether the battle settings show vibration as switched on", options: vec![ON.to_owned(), OFF.to_owned()], selected: vibration.to_owned(), onchange: { let platform = p[2].clone(); move |label: String| state::apply(&platform, sandbox, |next| next.vibrate = label == ON) } }
                ComboRow { label: "Formation", hint: "Whether the deck shows one row of five at a time or both rows at once", options: vec![ONE_ROW.to_owned(), TWO_ROWS.to_owned()], selected: formation.to_owned(), onchange: { let platform = p[3].clone(); move |label: String| state::apply(&platform, sandbox, |next| next.two_rows = label == TWO_ROWS) } }
            }
            div { class: "settings-section",
                div { class: "settings-heading", "Device" }
                ComboRow { label: "Type", hint: "Phone insets the battle controls from the screen edges like a notched phone", options: Device::ALL.iter().map(ToString::to_string).collect(), selected: device, onchange: { let platform = p[4].clone(); move |label: String| if let Some(picked) = Device::ALL.into_iter().find(|device| device.to_string() == label) { state::apply(&platform, sandbox, |next| next.device = picked) } } }
                if native {
                    ComboRow { label: "Scaling", hint: NATIVE_HINT, options: vec![NATIVE.to_owned()], selected: NATIVE.to_owned(), enabled: false, onchange: move |_| {} }
                } else {
                    ComboRow { label: "Scaling", hint: SCALING_HINT, options: Scaling::ALL.iter().map(ToString::to_string).collect(), selected: scaling, onchange: { let platform = p[5].clone(); move |label: String| if let Some(picked) = Scaling::ALL.into_iter().find(|scaling| scaling.to_string() == label) { state::apply(&platform, sandbox, |next| next.scaling = Some(picked)) } } }
                }
                label { class: "setting-row inline",
                    Tip { text: QUALITY_HINT, span { "Quality" } }
                    input { class: "slider quality-slider", r#type: "range", min: "{QUALITY_RANGE.start()}", max: "{QUALITY_RANGE.end()}", step: "{QUALITY_STEP}", value: "{quality}",
                        oninput: { let platform = p[6].clone(); move |event| if let Ok(value) = event.value().parse::<u8>() { state::apply(&platform, sandbox, |next| next.image_quality = Some(value)) } } }
                    span { "{quality}%" }
                }
            }
        }
    }
}

#[component]
fn TreasurePanel(platform: Platform, sandbox: Signal<SandboxState>, globals: Shared) -> Element {
    let config = sandbox.read().config.treasure.clone();
    let catalog = Some(Treasury(Arc::clone(&globals.0.catalog)));
    let arc_platform = platform.clone();

    rsx! {
        div { class: "settings-sections",
            div { class: "settings-section",
                div { class: "settings-heading", "Treasures" }
                Treasures {
                    config,
                    catalog,
                    onarc: move |(arc, value): (usize, String)| state::apply(&arc_platform, sandbox, |next| next.config.treasure.set_arc(arc, value)),
                    onlevel: move |(slot, index, level): (usize, usize, i32)| state::apply(&platform, sandbox, |next| next.config.treasure.set_level(slot, index, level)),
                }
            }
        }
    }
}

#[component]
fn TechPanel(platform: Platform, sandbox: Signal<SandboxState>) -> Element {
    let held = sandbox.read();
    let techs = held.config.techs.clone();
    let cat_god = held.config.cat_god.to_string();
    let altar = held.config.altar_level.clone();

    drop(held);

    let god_platform = platform.clone();
    let altar_platform = platform.clone();

    rsx! {
        div { class: "settings-sections",
            div { class: "settings-section",
                div { class: "settings-heading", "Tech Levels" }
                for (index, tech) in TECHS.iter().enumerate() {
                    {
                        let platform = platform.clone();

                        rsx! {
                            EntryRow { key: "{tech.name}", label: tech.name, placeholder: tech.hint(), value: techs[index].clone(), oninput: move |entry: String| state::apply(&platform, sandbox, |next| next.config.techs[index] = entry) }
                        }
                    }
                }
            }
            div { class: "settings-section",
                div { class: "settings-heading", "Cat God" }
                ComboRow {
                    label: "Availability",
                    hint: "Absent: not unlocked, so his button never appears\nPresent: unlocked, miracles at full price\nDiscounted: the 70% off reward for clearing Cats of the Cosmos chapter 1",
                    options: CatGod::ALL.iter().map(ToString::to_string).collect(),
                    selected: cat_god,
                    onchange: move |label: String| if let Some(picked) = CatGod::ALL.into_iter().find(|god| god.to_string() == label) { state::apply(&god_platform, sandbox, |next| next.config.cat_god = picked) },
                }
            }
            div { class: "settings-section",
                div { class: "settings-heading", "Aku Altar" }
                EntryRow { label: "Level Cap", placeholder: NO_CAP, value: altar, oninput: move |entry: String| state::apply(&altar_platform, sandbox, |next| next.config.altar_level = entry) }
            }
        }
    }
}

#[component]
fn BasePanel(platform: Platform, sandbox: Signal<SandboxState>, globals: Shared) -> Element {
    let shared = globals.clone();
    let fitted = use_memo(use_reactive!(|shared| Fitted(Rc::new(fit(&shared)))));
    let fitted = fitted();
    let held = sandbox.read();
    let config = held.config.clone();

    drop(held);

    let labeled = &fitted.0;
    let pick = |offered: &[(i32, String)], id: Option<i32>| -> String {
        offered.iter().find(|(part, _)| *part == id.unwrap_or(0)).or_else(|| offered.first()).map_or_else(String::new, |(_, label)| label.clone())
    };
    let highest = |parts: &BTreeMap<i32, i32>, id: Option<i32>| parts.get(&id.unwrap_or(0)).copied().unwrap_or(1).to_string();
    let cannon = pick(&labeled.cannons, config.cannon);
    let style = pick(&labeled.styles, config.style);
    let foundation = pick(&labeled.foundations, config.foundation);
    let labels = |offered: &[(i32, String)]| offered.iter().map(|(_, label)| label.clone()).collect::<Vec<String>>();
    let p = [platform.clone(), platform.clone(), platform.clone(), platform.clone(), platform.clone(), platform.clone(), platform.clone()];
    let cannons = fitted.clone();
    let styles = fitted.clone();
    let foundations = fitted.clone();

    rsx! {
        div { class: "settings-sections",
            div { class: "settings-section",
                div { class: "settings-heading", "Cat Base" }
                ComboRow { label: "Cannon", hint: "The cannon the base fires", options: labels(&labeled.cannons), selected: cannon, onchange: { let platform = p[0].clone(); move |label: String| if let Some((id, _)) = cannons.0.cannons.iter().find(|(_, held)| *held == label) { let id = *id; state::apply(&platform, sandbox, |next| next.config.cannon = Some(id)) } } }
                ComboRow { label: "Style", hint: "The Style part fitted to the base", options: labels(&labeled.styles), selected: style, onchange: { let platform = p[1].clone(); move |label: String| if let Some((id, _)) = styles.0.styles.iter().find(|(_, held)| *held == label) { let id = *id; state::apply(&platform, sandbox, |next| next.config.style = Some(id)) } } }
                ComboRow { label: "Foundation", hint: "The Foundation part fitted to the base", options: labels(&labeled.foundations), selected: foundation, onchange: { let platform = p[2].clone(); move |label: String| if let Some((id, _)) = foundations.0.foundations.iter().find(|(_, held)| *held == label) { let id = *id; state::apply(&platform, sandbox, |next| next.config.foundation = Some(id)) } } }
                EntryRow { label: "Cannon Level", placeholder: highest(&labeled.parts.cannons, config.cannon), value: config.cannon_level.clone(), oninput: { let platform = p[3].clone(); move |entry: String| state::apply(&platform, sandbox, |next| next.config.cannon_level = entry) } }
                EntryRow { label: "Style Level", placeholder: highest(&labeled.parts.styles, config.style), value: config.style_level.clone(), oninput: { let platform = p[4].clone(); move |entry: String| state::apply(&platform, sandbox, |next| next.config.style_level = entry) } }
                EntryRow { label: "Foundation Level", placeholder: highest(&labeled.parts.foundations, config.foundation), value: config.foundation_level.clone(), oninput: { let platform = p[5].clone(); move |entry: String| state::apply(&platform, sandbox, |next| next.config.foundation_level = entry) } }
                EntryRow { label: "Base Health Level", placeholder: labeled.parts.castle.to_string(), value: config.castle_level.clone(), oninput: { let platform = p[6].clone(); move |entry: String| state::apply(&platform, sandbox, |next| next.config.castle_level = entry) } }
            }
        }
    }
}

#[component]
fn ItemsPanel(platform: Platform, sandbox: Signal<SandboxState>) -> Element {
    let usable = (state::usable().0)();
    let held = sandbox.read();
    let items_off = held.config.items_off;
    let speed_up = usable.first().copied().unwrap_or(true) && !items_off.first().copied().unwrap_or(false);
    let shown = if speed_up { StartSpeed::Triple } else { held.config.start_speed };

    drop(held);

    let speed_platform = platform.clone();

    rsx! {
        div { class: "settings-sections",
            div { class: "settings-section",
                div { class: "settings-heading", "Battle Items" }
                ComboRow {
                    label: "Start Speed",
                    hint: "The speed the battle opens at.\n1x: the speed button starts off\n2x: it starts engaged\nStocking Speed Up raises the ceiling to 3x and the battle opens there",
                    options: if speed_up { vec![shown.to_string()] } else { StartSpeed::SELECTABLE.iter().map(ToString::to_string).collect() },
                    selected: shown.to_string(),
                    enabled: !speed_up,
                    onchange: move |label: String| if let Some(picked) = StartSpeed::SELECTABLE.into_iter().find(|speed| speed.to_string() == label) { state::apply(&speed_platform, sandbox, |next| next.config.start_speed = picked) },
                }
                for (item, name) in ITEMS.iter().enumerate() {
                    {
                        let allowed = usable.get(item).copied().unwrap_or(true);
                        let stocked = allowed && !items_off.get(item).copied().unwrap_or(false);
                        let platform = platform.clone();

                        rsx! {
                            div { key: "{name}", class: if allowed { "item-row" } else { "item-row weak" },
                                Toggle { value: stocked, label: name.to_string(), onchange: move |enabled: bool| if allowed { state::apply(&platform, sandbox, |next| next.config.items_off[item] = !enabled) } }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn InvasionsPanel(platform: Platform, sandbox: Signal<SandboxState>, staged: Option<Staged>) -> Element {
    let held = sandbox.read();
    let invasions = held.config.invasions;

    drop(held);

    let (filibuster, outbreak) = staged.as_ref().map_or((Vec::new(), Vec::new()), |staged| {
        (invadable(&staged.0.registry, Category::CatsOfTheCosmos), invadable(&staged.0.registry, Category::ZombieOutbreaks))
    });
    let label_of = |offered: &[(u32, String)], slot: u32| offered.iter().find(|(held, _)| *held == slot).map_or_else(String::new, |(_, label)| label.clone());
    let names = |offered: &[(u32, String)]| offered.iter().map(|(_, label)| label.clone()).collect::<Vec<String>>();
    let picked_filibuster = label_of(&filibuster, invasions.filibuster);
    let picked_outbreak = label_of(&outbreak, invasions.filibuster_outbreak);
    let filibuster_platform = platform.clone();
    let filibuster_options = filibuster.clone();
    let outbreak_options = outbreak.clone();

    rsx! {
        div { class: "settings-sections",
            div { class: "settings-section",
                div { class: "settings-heading", "Filibuster Invasion" }
                ComboRow {
                    label: "Invasion Stage",
                    hint: "The Cats of the Cosmos chapter 3 stage the Filibuster Invasion takes over",
                    options: names(&filibuster),
                    selected: picked_filibuster,
                    onchange: move |label: String| if let Some((slot, _)) = filibuster_options.iter().find(|(_, held)| *held == label) { let slot = *slot; state::apply(&filibuster_platform, sandbox, |next| next.config.invasions.filibuster = slot) },
                }
                ComboRow {
                    label: "Outbreak Stage",
                    hint: "The Zombie Outbreaks chapter 3 stage the zombie Filibuster Invasion takes over",
                    options: names(&outbreak),
                    selected: picked_outbreak,
                    onchange: move |label: String| if let Some((slot, _)) = outbreak_options.iter().find(|(_, held)| *held == label) { let slot = *slot; state::apply(&platform, sandbox, |next| next.config.invasions.filibuster_outbreak = slot) },
                }
            }
        }
    }
}

#[component]
fn TutorialPanel(platform: Platform, sandbox: Signal<SandboxState>) -> Element {
    let tutorial = sandbox.read().config.tutorial;

    rsx! {
        div { class: "settings-sections",
            div { class: "settings-section",
                div { class: "settings-heading", "Cleared Tutorials" }
                for (step, label, hint) in TUTORIAL_STEPS {
                    {
                        let platform = platform.clone();

                        rsx! {
                            Toggle { key: "{label}", value: tutorial_step(&tutorial, step), label: label.to_string(), hint: hint.to_string(), onchange: move |cleared: bool| state::apply(&platform, sandbox, |next| set_tutorial_step(&mut next.config.tutorial, step, cleared)) }
                        }
                    }
                }
            }
        }
    }
}
