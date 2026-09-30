use iced::widget::{column, container, scrollable, text, Column};
use iced::{Element, Length};

use kore::domains::sandbox::base::{self, Parts};
use kore::domains::sandbox::config::{CatGod, StartSpeed, Tutorial};
use kore::domains::sandbox::{config, ITEMS, TECHS};
use kore::systems::treasure::Catalog;
use kore::Vfs;

use crate::app::state::{SandboxDevice, SandboxPanel, SandboxState, SandboxVolume};
use crate::app::theme;
use crate::systems::treasure;
use crate::widget::{combo_row, entry_row, hover_hint, list_row, section, smooth_scroll, toggle_row};

const SIDEBAR_WIDTH: f32 = 110.0;
const PANEL_PADDING: f32 = 20.0;
const ROW_SPACING: f32 = 10.0;
const TAB_SPACING: f32 = 4.0;
const TAB_SIZE: f32 = 14.0;
const NO_CAP: &str = "None";
const LEVEL_DIGITS: usize = 6;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Formation {
    Single,
    Double,
}

impl Formation {
    const ALL: [Self; 2] = [Self::Single, Self::Double];
}

impl std::fmt::Display for Formation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Single => "One Row",
            Self::Double => "Two Rows",
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Part {
    id: i32,
    label: String,
}

impl std::fmt::Display for Part {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.label)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Slot {
    Cannon,
    Style,
    Foundation,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TutorialStep {
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

#[derive(Clone, Debug)]
pub enum Message {
    Panel(SandboxPanel),
    Device(SandboxDevice),
    Music(SandboxVolume),
    Effects(SandboxVolume),
    Formation(Formation),
    Vibrate(bool),
    Treasure(treasure::Message),
    Tech(usize, String),
    Part(Slot, Part),
    Level(Slot, String),
    Castle(String),
    Item(usize, bool),
    Altar(String),
    CatGod(CatGod),
    StartSpeed(StartSpeed),
    Tutorial(TutorialStep, bool),
}

#[derive(Default)]
pub struct State {
    parts: Parts,
    cannons: Vec<Part>,
    styles: Vec<Part>,
    foundations: Vec<Part>,
    loaded: bool,
    treasure: treasure::State,
}

fn typable(entry: &str) -> bool {
    entry.len() <= LEVEL_DIGITS && entry.chars().all(|glyph| glyph.is_ascii_digit() || glyph == '+' || glyph == '%')
}

fn offered(parts: &std::collections::BTreeMap<i32, i32>, label: impl Fn(i32) -> String) -> Vec<Part> {
    parts.keys().map(|id| Part { id: *id, label: label(*id) }).collect()
}

impl State {
    pub fn enter(&mut self, vfs: &Vfs) {
        if self.loaded {
            return;
        }

        self.reload(vfs);
    }

    pub fn reload(&mut self, vfs: &Vfs) {
        self.parts = Parts::load(vfs);
        self.cannons = offered(&self.parts.cannons, base::name);
        self.styles = offered(&self.parts.styles, |id| self.parts.style_label(id));
        self.foundations = offered(&self.parts.foundations, |id| self.parts.foundation_label(id));
        self.loaded = true;
    }

    pub fn parts(&self) -> &Parts {
        &self.parts
    }

    pub fn update(&mut self, message: Message, options: &mut SandboxState) {
        match message {
            Message::Panel(panel) => options.panel = panel,
            Message::Device(device) => options.device = device,
            Message::Music(volume) => options.music_volume = volume.percent(),
            Message::Effects(volume) => options.effects_volume = volume.percent(),
            Message::Formation(formation) => options.two_rows = formation == Formation::Double,
            Message::Vibrate(enabled) => options.vibrate = enabled,
            Message::Treasure(message) => self.treasure.update(message, &mut options.config.treasure),
            Message::Tech(index, entry) => {
                if let Some(held) = options.config.techs.get_mut(index).filter(|_| typable(&entry)) {
                    *held = entry;
                }
            }
            Message::Part(slot, part) => match slot {
                Slot::Cannon => options.config.cannon = Some(part.id),
                Slot::Style => options.config.style = Some(part.id),
                Slot::Foundation => options.config.foundation = Some(part.id),
            },
            Message::Tutorial(step, cleared) => {
                let tutorial = &mut options.config.tutorial;

                match step {
                    TutorialStep::BattleCleared => tutorial.battle_cleared = cleared,
                    TutorialStep::DeckSeen => tutorial.deck_seen = cleared,
                    TutorialStep::TwoRowsSeen => tutorial.two_rows_seen = cleared,
                    TutorialStep::CatGodSeen => tutorial.cat_god_seen = cleared,
                    TutorialStep::ShopSeen => tutorial.shop_seen = cleared,
                }
            }
            Message::CatGod(picked) => options.config.cat_god = picked,
            Message::StartSpeed(picked) => options.config.start_speed = picked,
            Message::Castle(entry) => {
                if typable(&entry) {
                    options.config.castle_level = entry;
                }
            }
            Message::Altar(entry) => {
                if typable(&entry) {
                    options.config.altar_level = entry;
                }
            }
            Message::Item(item, stocked) => {
                if let Some(off) = options.config.items_off.get_mut(item) {
                    *off = !stocked;
                }
            }
            Message::Level(slot, entry) => {
                if !typable(&entry) {
                    return;
                }

                match slot {
                    Slot::Cannon => options.config.cannon_level = entry,
                    Slot::Style => options.config.style_level = entry,
                    Slot::Foundation => options.config.foundation_level = entry,
                }
            }
        }
    }

    pub fn view<'a>(&'a self, options: &'a SandboxState, catalog: &'a Catalog, bottom: f32) -> Element<'a, Message> {
        let tabs = [
            (SandboxPanel::Settings, "Settings"),
            (SandboxPanel::Treasure, "Treasure"),
            (SandboxPanel::Tech, "Tech"),
            (SandboxPanel::Base, "Base"),
            (SandboxPanel::Items, "Items"),
            (SandboxPanel::Tutorial, "Tutorial"),
        ];

        let mut tab_list = column![].spacing(TAB_SPACING);

        for (panel, label) in tabs {
            let content = container(theme::button_label(label).size(TAB_SIZE)).padding([8, 12]).width(Length::Fill);

            tab_list = tab_list.push(list_row(content, options.panel == panel, true, Length::Fill, Message::Panel(panel)));
        }

        let sidebar = container(tab_list)
            .width(Length::Fixed(SIDEBAR_WIDTH))
            .height(Length::Fill)
            .padding(8)
            .style(theme::list_panel_container);

        let body = match options.panel {
            SandboxPanel::Settings => Self::settings(options),
            SandboxPanel::Treasure => section("Treasures", Length::Fill, self.treasure.view(&options.config.treasure, catalog).map(Message::Treasure)),
            SandboxPanel::Tech => Self::tech(options),
            SandboxPanel::Base => self.base(options),
            SandboxPanel::Items => Self::items(options),
            SandboxPanel::Tutorial => Self::tutorial(options),
        };

        let page = container(body).padding(iced::Padding { bottom, ..iced::Padding::new(PANEL_PADDING) }).width(Length::Fill);

        iced::widget::row![
            sidebar,
            smooth_scroll(scrollable(page).width(Length::Fill).height(Length::Fill)),
        ]
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    fn settings(options: &SandboxState) -> Element<'_, Message> {
        let formation = if options.two_rows { Formation::Double } else { Formation::Single };

        section(
            "Settings",
            Length::Fill,
            column![
                combo_row(
                    "Device",
                    "Phone insets the battle controls from the screen edges like a notched phone",
                    SandboxDevice::ALL,
                    Some(options.device),
                    Some(Message::Device),
                ),
                combo_row(
                    "Music Volume",
                    "The four volume steps the battle settings cycle through",
                    SandboxVolume::ALL,
                    Some(SandboxVolume::nearest(options.music_volume)),
                    Some(Message::Music),
                ),
                combo_row(
                    "Sound Volume",
                    "The four volume steps the battle settings cycle through",
                    SandboxVolume::ALL,
                    Some(SandboxVolume::nearest(options.effects_volume)),
                    Some(Message::Effects),
                ),
                combo_row(
                    "Formation",
                    "Whether the deck shows one row of five at a time or both rows at once",
                    Formation::ALL,
                    Some(formation),
                    Some(Message::Formation),
                ),
                toggle_row(options.vibrate, text("Vibration"), Some(Message::Vibrate)),
            ]
                .spacing(ROW_SPACING),
        )
    }

    fn tutorial(options: &SandboxState) -> Element<'_, Message> {
        let mut rows = Column::new().spacing(ROW_SPACING);

        for (step, label, hint) in TUTORIAL_STEPS {
            rows = rows.push(hover_hint(
                toggle_row(tutorial_step(&options.config.tutorial, step), text(label), Some(move |cleared| Message::Tutorial(step, cleared))),
                hint,
            ));
        }

        section("Cleared Tutorials", Length::Fill, rows)
    }

    fn items(options: &SandboxState) -> Element<'_, Message> {
        let speed_up = options.usable_items.first().copied().unwrap_or(true)
            && !options.config.items_off.first().copied().unwrap_or(false);
        let shown = if speed_up { StartSpeed::Triple } else { options.config.start_speed };
        let mut rows = Column::new().spacing(ROW_SPACING).push(combo_row(
            "Start Speed",
            "The speed the battle opens at.\n1x: the speed button starts off\n2x: it starts engaged\nStocking Speed Up raises the ceiling to 3x and the battle opens there",
            StartSpeed::SELECTABLE,
            Some(shown),
            (!speed_up).then_some(Message::StartSpeed),
        ));

        for (item, name) in ITEMS.iter().enumerate() {
            let allowed = options.usable_items.get(item).copied().unwrap_or(true);
            let stocked = allowed && !options.config.items_off.get(item).copied().unwrap_or(false);
            let label = text(*name).style(move |theme: &iced::Theme| text::Style {
                color: (!allowed).then(|| theme::weak_text_color(theme)),
            });

            rows = rows.push(toggle_row(stocked, label, allowed.then_some(move |enabled| Message::Item(item, enabled))));
        }

        section("Battle Items", Length::Fill, rows)
    }

    fn tech(options: &SandboxState) -> Element<'_, Message> {
        let mut rows = Column::new().spacing(ROW_SPACING);

        for (index, tech) in TECHS.iter().enumerate() {
            rows = rows.push(entry_row(tech.name, &tech.hint(), &options.config.techs[index], move |entry| {
                Message::Tech(index, entry)
            }));
        }

        column![
            section("Tech Levels", Length::Fill, rows),
            section(
                "Cat God",
                Length::Fill,
                combo_row(
                    "Availability",
                    "Absent: not unlocked, so his button never appears\nPresent: unlocked, miracles at full price\nDiscounted: the 70% off reward for clearing Cats of the Cosmos chapter 1",
                    CatGod::ALL,
                    Some(options.config.cat_god),
                    Some(Message::CatGod),
                ),
            ),
            section(
                "Aku Altar",
                Length::Fill,
                entry_row("Level Cap", NO_CAP, &options.config.altar_level, Message::Altar),
            ),
        ]
            .spacing(PANEL_PADDING)
            .into()
    }

    fn base<'a>(&'a self, options: &'a SandboxState) -> Element<'a, Message> {
        let held = |offered: &'a [Part], id: Option<i32>| {
            offered.iter().find(|part| part.id == id.unwrap_or(0)).or_else(|| offered.first())
        };
        let highest = |parts: &std::collections::BTreeMap<i32, i32>, id: Option<i32>| {
            parts.get(&id.unwrap_or(0)).copied().unwrap_or(1).to_string()
        };

        let cannon = held(&self.cannons, options.config.cannon);
        let style = held(&self.styles, options.config.style);
        let foundation = held(&self.foundations, options.config.foundation);

        section(
            "Cat Base",
            Length::Fill,
            column![
                combo_row("Cannon", "The cannon the base fires", self.cannons.as_slice(), cannon, Some(|part| Message::Part(Slot::Cannon, part))),
                combo_row("Style", "The Style part fitted to the base", self.styles.as_slice(), style, Some(|part| Message::Part(Slot::Style, part))),
                combo_row(
                    "Foundation",
                    "The Foundation part fitted to the base",
                    self.foundations.as_slice(),
                    foundation,
                    Some(|part| Message::Part(Slot::Foundation, part)),
                ),
                entry_row("Cannon Level", &highest(&self.parts.cannons, options.config.cannon), &options.config.cannon_level, |entry| {
                    Message::Level(Slot::Cannon, entry)
                }),
                entry_row("Style Level", &highest(&self.parts.styles, options.config.style), &options.config.style_level, |entry| {
                    Message::Level(Slot::Style, entry)
                }),
                entry_row(
                    "Foundation Level",
                    &highest(&self.parts.foundations, options.config.foundation),
                    &options.config.foundation_level,
                    |entry| Message::Level(Slot::Foundation, entry),
                ),
                entry_row("Base Health Level", &self.parts.castle.to_string(), &options.config.castle_level, Message::Castle),
            ]
                .spacing(ROW_SPACING),
        )
    }
}

pub fn level(entry: &str, parts: &std::collections::BTreeMap<i32, i32>, id: Option<i32>) -> i32 {
    config::level(entry, parts.get(&id.unwrap_or(0)).copied().unwrap_or(1))
}
