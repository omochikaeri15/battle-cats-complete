mod arena;
mod combos;
mod config;
mod lineup;
mod orbs;
mod state;
mod warm;

use std::rc::Rc;

use dioxus::prelude::*;

use codex::chapter::Category;
use emulator::runtime::{CASTLE_PART, CatGod as EngineGod, Setup, SetupUnit, StageEntry, StageVariant, TechLevel, TutorialFlags, usable_items};
use kokoro::domains::cat::scanner::CatEntry;
use kokoro::domains::sandbox::altar::Altars;
use kokoro::domains::sandbox::base::Parts;
use kokoro::domains::sandbox::config::{self as sandbox_config, CatGod};
use kokoro::domains::sandbox::rules::Rules;
use kokoro::domains::sandbox::{TECHS, scored};
use kokoro::domains::stage::lottery::Lottery;
use kokoro::domains::stage::scanner::StageBundle;
use kokoro::domains::stage::{GlobalMapId, GlobalStageId, StageRegistry, navigate};

use crate::app::Route;
use crate::domains::stage;
use crate::platform::Platform;
use crate::platform::battle::{BattleCommand, OptionsWire};
use crate::systems::catalog::{self, Load};
use crate::systems::context::{self, Shared};
use crate::systems::library::Library;
use crate::theme;
use crate::widget::{Popup, Status};

pub(crate) use state::{SandboxState, provide};

use self::arena::Battle;
use self::state::{Device, Tab, Volume};

const ACKNOWLEDGMENT_INTRO: [&str; 2] = [
    "The purpose of this agreement is to ensure that you, the User, are aware of the potential quirks regarding Sandbox.",
    "By accepting this agreement, you must have read, understood, and accepted the following facts about Sandbox:",
];
const ACKNOWLEDGMENT_FACTS: [&str; 4] = [
    "Sandbox may have known but subtle inaccuracies, mechanical bugs, or logical quirks when compared to the real game.",
    "Sandbox may not be updated to support features and/or mechanics from the latest game version(s) within a timely manner.",
    "Sandbox is representative of the vanilla game logic and it's capabilities only, with no custom functionality added or planned.",
    "Sandbox is not an avenue that replaces the appeal and purpose of the game in any capacity, whether that be vanilla or modded.",
];
const ACKNOWLEDGMENT_CLOSING: &str = "You can accept the agreement by clicking the \"Agree\" button below. Selecting \"Disagree\" will redirect you to the Home menu.";
const ACKNOWLEDGE_WIDTH: f32 = 560.0;
const FAULT_WIDTH: f32 = 460.0;
const DIAGNOSTICS_WIDTH: f32 = 520.0;
const FNV_OFFSET: u64 = 0xcbf2_9ce4_8422_2325;
const FNV_PRIME: u64 = 0x0100_0000_01b3;
const TABS: [(Tab, &str); 4] = [(Tab::Lineup, "Lineup"), (Tab::Stage, "Stage"), (Tab::Config, "Config"), (Tab::Replay, "Replay")];

fn agreement() -> String {
    let mut hash = FNV_OFFSET;

    for text in ACKNOWLEDGMENT_INTRO.iter().chain(ACKNOWLEDGMENT_FACTS.iter()).chain(std::iter::once(&ACKNOWLEDGMENT_CLOSING)) {
        for byte in text.bytes() {
            hash ^= u64::from(byte);
            hash = hash.wrapping_mul(FNV_PRIME);
        }
    }

    format!("{hash:016x}")
}

#[derive(Clone)]
pub(crate) struct Cats(pub(crate) Rc<Vec<CatEntry>>);

impl PartialEq for Cats {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }
}

#[derive(Clone)]
pub(crate) struct Staged(pub(crate) Rc<StageBundle>);

impl PartialEq for Staged {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }
}

#[derive(Clone, PartialEq)]
pub(crate) struct Ruled(pub(crate) Rc<Rules>);

#[derive(Clone, PartialEq)]
struct Staging {
    entry: Option<StageEntry>,
    usable: [bool; 6],
    rules: Rc<Rules>,
    altar: Option<u32>,
}

pub(crate) struct Picked {
    pub(crate) stage: GlobalStageId,
    pub(crate) crown: u8,
}

fn invaded_slot(registry: &StageRegistry, picked: &GlobalStageId, state: &SandboxState) -> Option<u32> {
    let invasions = &state.config.invasions;

    navigate::is_invasion(registry, picked).then_some(if picked.category == Category::ZombieOutbreaks { invasions.filibuster_outbreak } else { invasions.filibuster })
}

fn staged_entry(globals: &Shared, registry: &StageRegistry, picked: &Picked, state: &SandboxState) -> Option<StageEntry> {
    let map = GlobalMapId { category: picked.stage.category.clone(), map: picked.stage.map };
    let global = registry.addresses.get(&map)?.global?;

    if let Some(target) = navigate::invaded_stage(registry, &picked.stage) {
        return Some(StageEntry {
            map_id: i32::try_from(target.map_id).ok()?,
            stage: i32::try_from(target.stage).ok()?,
            layout: None,
            crown: i32::from(target.crown),
            dungeon_stage: None,
            variant: StageVariant::ExInvasion,
        });
    }

    let map_id = i32::try_from(global).ok()?;
    let outbreak = picked.stage.category == Category::ZombieOutbreaks;
    let invaded = invaded_slot(registry, &picked.stage, state);
    let variant = match (outbreak, invaded.is_some()) {
        (false, false) => StageVariant::Plain,
        (true, false) => StageVariant::Outbreak,
        (false, true) => StageVariant::Invasion,
        (true, true) => StageVariant::ZombieInvasion,
    };
    let stage = invaded.unwrap_or(picked.stage.stage);
    let layout = navigate::map_screen_slot(map_id, i32::try_from(stage).ok()?);
    let slot = Lottery::load(&globals.0.vault().vfs).slot_for(map_id, layout);

    Some(StageEntry { map_id, stage: slot.unwrap_or(layout), layout: slot.map(|_| layout), crown: i32::from(picked.crown), dungeon_stage: None, variant })
}

pub(crate) struct Ruling {
    pub(crate) rules: Rules,
    pub(crate) usable: [bool; 6],
    pub(crate) castle: Option<i32>,
}

fn ruling(globals: &Shared, registry: &StageRegistry, picked: Option<&Picked>) -> Ruling {
    let stage = picked.and_then(|picked| registry.stages.get(&picked.stage));
    let map_key = picked.map(|picked| GlobalMapId { category: picked.stage.category.clone(), map: picked.stage.map });
    let rules = stage.zip(picked).map(|(stage, picked)| Rules::of(stage, map_key.as_ref().and_then(|key| registry.maps.get(key)), picked.crown as i8)).unwrap_or_default();
    let global = map_key.as_ref().and_then(|key| registry.addresses.get(key)?.global).and_then(|global| i32::try_from(global).ok());
    let usable = global.map_or([true; 6], |map_id| {
        let scored = scored::score_stage_maps(&globals.0.vault().vfs);

        usable_items(map_id, scored.contains(&map_id))
    });
    let castle = stage.and_then(|stage| i32::try_from(stage.anim_base_id).ok());

    Ruling { rules, usable, castle }
}

fn build_setup(state: &SandboxState, cats: &[CatEntry], globals: &Shared, entry: StageEntry) -> Setup {
    let vault = globals.0.vault();
    let vfs = &vault.vfs;
    let parts = Parts::load(vfs);
    let orbs = vault.vds.cats.orbs(vfs);
    let orb_slots = vault.vds.cats.orb_slots(vfs);
    let tutorial = &state.config.tutorial;
    let mut setup = Setup {
        stage: entry,
        tutorial: TutorialFlags {
            battle_cleared: tutorial.battle_cleared,
            deck_seen: tutorial.deck_seen,
            two_rows_seen: tutorial.two_rows_seen,
            cat_god_seen: tutorial.cat_god_seen,
            shop_seen: tutorial.shop_seen,
        },
        ..Setup::default()
    };

    setup.lineup = state
        .roster
        .current()
        .into_iter()
        .flat_map(|lineup| lineup.slots.iter())
        .map(|member| {
            let (level, plus) = member.levels();
            let groups = cats.iter().find(|cat| cat.id == member.id).and_then(|cat| cat.talent_data.as_ref());
            let slots = orb_slots.get(&member.id).copied().unwrap_or(0);

            SetupUnit {
                unit: member.id as i32,
                form: member.form as i32,
                level,
                plus,
                talents: member
                    .talents
                    .iter()
                    .filter_map(|(index, level)| {
                        let group = groups?.groups.get(*index as usize)?;

                        Some((i32::from(group.ability_id), i32::from(*level)))
                    })
                    .collect(),
                orbs: member.worn(slots, &orbs).map(|(slot, orb)| (slot as i32, orb as i32)).collect(),
            }
        })
        .collect();

    setup.tech[0] = TechLevel { level: 1, plus: 0 };

    for (index, tech) in TECHS.iter().enumerate() {
        let (level, plus) = state.config.tech(index);

        if let Some(held) = setup.tech.get_mut(tech.slot) {
            *held = TechLevel { level, plus };
        }
    }

    setup.treasures = state.config.treasure.levels();
    setup.altar = state.config.altar();
    setup.cat_god = match state.config.cat_god {
        CatGod::Absent => EngineGod::Absent,
        CatGod::Present => EngineGod::Present,
        CatGod::Discounted => EngineGod::Discounted,
    };

    for (item, off) in state.config.items_off.iter().enumerate() {
        if let Some(stocked) = setup.items.get_mut(item) {
            *stocked = !off;
        }
    }

    setup.speed_engaged = state.config.start_speed.engaged();
    setup.cannon = state.config.cannon.unwrap_or(0);
    setup.style = state.config.style.unwrap_or(0);
    setup.foundation = state.config.foundation.unwrap_or(0);

    let highest = |parts: &std::collections::BTreeMap<i32, i32>, id: i32| parts.get(&id).copied().unwrap_or(1);

    setup.parts.entry(setup.cannon).or_default().cannon = sandbox_config::level(&state.config.cannon_level, highest(&parts.cannons, setup.cannon));
    setup.parts.entry(setup.style).or_default().style = sandbox_config::level(&state.config.style_level, highest(&parts.styles, setup.style));
    setup.parts.entry(setup.foundation).or_default().foundation = sandbox_config::level(&state.config.foundation_level, highest(&parts.foundations, setup.foundation));
    setup.parts.entry(CASTLE_PART).or_default().cannon = state.config.castle(parts.castle);

    for levels in setup.parts.values_mut() {
        levels.cannon = levels.cannon.max(1);
    }

    setup
}

#[component]
pub(crate) fn Sandbox() -> Element {
    let platform = use_context::<Platform>();
    let library = use_context::<Library>();
    let catalogs = catalog::catalogs();
    let settings = context::settings();
    let sandbox = state::state();
    let mut battle = arena::provide(&platform);
    let stage_state = stage::state();
    let navigator = use_navigator();
    let mut size = use_signal(|| (0f32, 0f32));
    let starter = platform.clone();

    use_hook(move || {
        let config = settings.read().scanner_config(None);

        catalogs.load_cats(&starter, config.clone());
        catalogs.load_enemies(&starter, config.clone());
        catalogs.load_stages(&starter, config);
        starter.host().battle(BattleCommand::Prepare { fonts: theme::GAME_FONTS.iter().map(|font| Box::from(font.to_string())).collect() });
    });

    let stopper = platform.clone();

    use_drop(move || {
        stopper.host().battle(BattleCommand::Stop);
        stopper.host().battle(BattleCommand::Forget);
    });

    let globals = context::globals();
    let cats = catalogs.cats();
    let stages = catalogs.stages();

    if let Some(fault) = library.fault() {
        return rsx! { Status { title: "The file host could not be reached", detail: fault.to_string() } };
    }

    let Some(globals) = globals else {
        let phase = library.phase();

        return rsx! { Status { title: phase.label.to_string(), detail: Library::detail(phase.done, phase.total) } };
    };

    let cats = match &cats {
        Load::Ready(cats) => Some(Cats(Rc::clone(cats))),
        _ => None,
    };
    let bundle = match &stages {
        Load::Ready(bundle) => Some(Staged(Rc::clone(bundle))),
        _ => None,
    };
    let held = sandbox.read();
    let agreed = held.agreement == agreement();
    let tab = held.tab;
    let fielded = held.roster.current().is_some_and(|lineup| !lineup.slots.is_empty());

    drop(held);

    let shared = globals.clone();
    let staged = bundle.clone();
    let staging = use_memo(use_reactive!(|(staged, shared)| {
        let bundle = staged;
        let picked = (stage_state.stage)().map(|stage| Picked { stage, crown: (stage_state.crown)() });
        let held = sandbox.read();
        let registry = bundle.as_ref().map(|bundle| &bundle.0.registry);
        let entry = registry.zip(picked.as_ref()).and_then(|(registry, picked)| staged_entry(&shared, registry, picked, &held));
        let ruled = registry.map(|registry| ruling(&shared, registry, picked.as_ref()));
        let altar_level = held.config.altar();
        let altar = ruled.as_ref().and_then(|ruled| ruled.castle).and_then(|castle| Altars::load(&shared.0.vault().vfs).level_cap(castle, altar_level));

        Staging {
            entry,
            usable: ruled.as_ref().map_or([true; 6], |ruled| ruled.usable),
            rules: Rc::new(ruled.map(|ruled| ruled.rules).unwrap_or_default()),
            altar,
        }
    }));
    let staging = staging();
    let playable = staging.entry.is_some() && fielded && cats.is_some();
    let usable = state::usable();

    if *usable.0.peek() != staging.usable {
        let mut usable = usable;

        usable.0.set(staging.usable);
    }

    let altar_cap = staging.altar;
    let rules = Rc::clone(&staging.rules);
    let status = (battle.status)().unwrap_or_default();
    let play_platform = platform.clone();
    let play_globals = globals.clone();
    let play_cats = cats.clone();
    let play_bundle = bundle.clone();
    let play = move |_| {
        let (Some(cats), Some(bundle)) = (&play_cats, &play_bundle) else {
            return;
        };
        let held = sandbox.read();
        let Some(picked) = (stage_state.stage)().map(|stage| Picked { stage, crown: (stage_state.crown)() }) else {
            return;
        };
        let Some(entry) = staged_entry(&play_globals, &bundle.0.registry, &picked, &held) else {
            return;
        };
        let setup = build_setup(&held, &cats.0, &play_globals, entry);
        let warmth = warm::collect(&setup, &bundle.0.registry, &picked, &cats.0);
        let (width, height) = size();
        let options = OptionsWire {
            music: Volume::nearest(held.music_volume).percent(),
            effects: Volume::nearest(held.effects_volume).percent(),
            two_rows: held.two_rows,
            vibrate: held.vibrate,
        };

        battle.failure.set(None);
        battle.status.set(None);
        play_platform.host().battle(BattleCommand::Start {
            setup: Box::new((&setup).into()),
            options,
            phone: held.device == Device::Phone,
            width,
            height,
            prefixes: warmth.prefixes,
            pools: warmth.pools,
            priority: settings.read().general.language_priority.clone(),
        });
    };
    let agree_platform = platform.clone();
    let tab_platform = platform.clone();

    rsx! {
        div {
            class: "sandbox-page",
            onresize: move |event| {
                if let Ok(box_size) = event.data().get_content_box_size() {
                    size.set((box_size.width as f32, box_size.height as f32));
                }
            },
            div { class: "sandbox-tabs",
                for (which, label) in TABS {
                    {
                        let class = crate::domains::cat::header_class(tab == which, true, false);
                        let platform = tab_platform.clone();

                        rsx! {
                            button { key: "{label}", class: "{class} sandbox-tab", onclick: move |_| state::apply(&platform, sandbox, |next| next.tab = which), "{label}" }
                        }
                    }
                }
            }
            div { class: "rule" }
            div { class: "sandbox-body",
                match tab {
                    Tab::Lineup => match cats.clone() {
                        Some(cats) => rsx! { lineup::Editor { cats, globals: globals.clone(), rules: Ruled(Rc::clone(&rules)), altar: altar_cap } },
                        None => rsx! { Status { title: "Loading Cats...", detail: None } },
                    },
                    Tab::Stage => rsx! { stage::Stages {} },
                    Tab::Config => rsx! { config::Config { globals: globals.clone(), staged: bundle.clone() } },
                    Tab::Replay => rsx! { div { class: "pick-prompt", "Replays coming soon?" } },
                }
            }
            if tab != Tab::Replay {
                div { class: "sandbox-play",
                    div { class: "sandbox-status", "{status}" }
                    button { class: "btn btn-primary sandbox-play-btn", disabled: !playable, onclick: play, "Play" }
                }
            }
        }
        if !agreed {
            Dialog { title: "Acknowledgment", width: ACKNOWLEDGE_WIDTH, modal: true, onclose: move |()| { navigator.push(Route::Home {}); },
                div { class: "sandbox-terms scroll",
                    for line in ACKNOWLEDGMENT_INTRO {
                        p { "{line}" }
                    }
                    ul {
                        for fact in ACKNOWLEDGMENT_FACTS {
                            li { "{fact}" }
                        }
                    }
                    p { "{ACKNOWLEDGMENT_CLOSING}" }
                }
                div { class: "sandbox-choices",
                    button { class: "btn btn-success btn-popup-action", onclick: move |_| state::apply(&agree_platform, sandbox, |next| next.agreement = agreement()), "Agree" }
                    button { class: "btn btn-danger btn-popup-action", onclick: move |_| { navigator.push(Route::Home {}); }, "Disagree" }
                }
            }
        }
        arena::Arena {}
        FaultDialog { battle }
        DiagnosticsDialog { battle }
        QualityToast { battle }
    }
}

#[component]
pub(super) fn Dialog(title: String, width: f32, #[props(default)] modal: bool, #[props(default)] class: String, onclose: EventHandler<()>, children: Element) -> Element {
    let open = use_signal(|| true);

    use_effect(move || {
        if !open() {
            onclose.call(());
        }
    });

    rsx! {
        Popup { title, open, width, modal, class,
            div { class: "dialog-body", {children} }
        }
    }
}

#[component]
fn FaultDialog(battle: Battle) -> Element {
    let platform = use_context::<Platform>();
    let standing = (battle.standing)();

    if standing.phase != crate::host::Phase::Faulted {
        return rsx! {};
    }

    let reason = (battle.failure)().unwrap_or_default();
    let terminator = platform.clone();

    rsx! {
        Dialog { title: "FAULT", width: FAULT_WIDTH, onclose: move |()| terminator.host().battle(BattleCommand::Stop),
            div { class: "sandbox-fault",
                div { "The game attempted to crash due to a fault" }
                div { class: "sandbox-fault-reason scroll", "{reason}" }
            }
            div { class: "sandbox-choices",
                {
                    let resumer = platform.clone();
                    let stopper = platform.clone();

                    rsx! {
                        button { class: "btn btn-success btn-popup-action", onclick: move |_| resumer.host().battle(BattleCommand::Resume), "Continue" }
                        button { class: "btn btn-danger btn-popup-action", onclick: move |_| stopper.host().battle(BattleCommand::Stop), "Terminate" }
                    }
                }
            }
        }
    }
}

#[component]
fn DiagnosticsDialog(battle: Battle) -> Element {
    let platform = use_context::<Platform>();
    let shown = (battle.diagnosing)();
    let probe = (battle.diagnostics)();

    let (Some(probe), true) = (probe, shown) else {
        return rsx! {};
    };
    let rows = |side: &[crate::host::Vitals], base: &str, fallback: &str| -> Vec<(String, String)> {
        side.iter()
            .map(|vitals| {
                let label = match &vitals.occupant {
                    crate::host::Occupant::Base => base.to_owned(),
                    crate::host::Occupant::Unit { id, name } if name.is_empty() => format!("{fallback} {id:03}"),
                    crate::host::Occupant::Unit { name, .. } => name.clone(),
                };

                (label, vitals.hp.to_string())
            })
            .collect()
    };
    let enemies = rows(&probe.enemies, "Enemy Base", "Enemy");
    let cats = rows(&probe.cats, "Cat Base", "Cat");
    let seed = probe.rng.to_string();

    rsx! {
        Dialog { title: "Live Diagnostics", width: DIAGNOSTICS_WIDTH, onclose: move |()| {
                battle.diagnosing.set(false);
                platform.host().battle(BattleCommand::Diagnose(false));
            },
            div { class: "sandbox-diagnostics",
                div { class: "zebra-table readout-seed",
                    div { class: "zebra-cells zebra-header", div { class: "zebra-cell bold", "Seed" } }
                    div { class: "zebra-cells zebra-row-even", div { class: "zebra-cell", "{seed}" } }
                }
                div { class: "readout-sides",
                    {readout_side("Enemies", &enemies)}
                    {readout_side("Cats", &cats)}
                }
            }
        }
    }
}

fn readout_side(title: &str, rows: &[(String, String)]) -> Element {
    rsx! {
        div { class: "readout-side",
            div { class: "zebra-cells zebra-header readout-row",
                span { class: "bold readout-name", "{title} ({rows.len()})" }
                span { class: "bold readout-value", "HP" }
            }
            div { class: "readout-list scroll",
                for (index, (name, hp)) in rows.iter().enumerate() {
                    div { class: if index % 2 == 0 { "zebra-cells zebra-row-even readout-row" } else { "zebra-cells zebra-row-odd readout-row" },
                        span { class: "readout-name", "{name}" }
                        span { class: "readout-value", "{hp}" }
                    }
                }
            }
        }
    }
}

#[component]
fn QualityToast(battle: Battle) -> Element {
    let notice = (battle.notice)();
    let class = if notice.is_some() { "toast info-banner" } else { "toast info-banner hidden" };
    let text = notice.unwrap_or_default();

    rsx! {
        div { class: "toast-slot sandbox-toast",
            div { class: "{class}", "{text}" }
        }
    }
}
