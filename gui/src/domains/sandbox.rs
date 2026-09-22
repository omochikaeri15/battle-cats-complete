mod combos;
mod config;
mod lineup;
mod orbs;

use iced::alignment::{Horizontal, Vertical};
use iced::widget::{button, column, container, markdown, row, rule, scrollable, stack, text, Space};
use iced::{Alignment, Element, Length, Size, Task, Theme};
use tracing::warn;

use std::cell::RefCell;
use std::rc::Rc;

use emu::runtime::{BattleOptions, Setup, SetupUnit, StageEntry, TechLevel, TREASURE_STAGES};
use kore::common::context::GlobalContext;
use kore::domains::cat::scanner::CatEntry;
use kore::domains::sandbox::config::CatGod;
use kore::domains::sandbox::keybind::Bind;
use kore::domains::sandbox::TECHS;
use kore::domains::settings::Settings;

use crate::app::state::{AppState, SandboxDevice, SandboxState, SandboxTab, SandboxVolume};
use crate::app::theme;
use crate::domains::{cat, stage};
use crate::systems::emu::{Action, Frame as EmuFrame, Session};
use crate::systems::emu::SheetCache;
use crate::widget::{popup, smooth_scroll};

const ACKNOWLEDGEMENT: &str = r#"
The purpose of this agreement is to ensure that you, the User, are aware of the potential quirks regarding Sandbox.
By accepting this agreement, you must have read, understood, and accepted the following facts about Sandbox:

- Sandbox may have known but subtle inaccuracies, mechanical bugs, or logical quirks when compared to the real game.
- Sandbox may not be updated to support features and/or mechanics from the latest game version(s) within a timely manner.
- Sandbox is representative of the vanilla game logic and it's capabilities only, with no custom functionality added or planned.
- Sandbox is not an avenue that replaces the appeal and purpose of the game in any capacity, whether that be vanilla or modded.

You can accept the agreement by clicking the "Agree" button below. Selecting "Disagree" will redirect you to the Home menu.
"#;
const ACKNOWLEDGE_POPUP: popup::Spec = popup::Spec::new(popup::Kind::Acknowledgement, Size::new(560.0, 435.0));
const FAULT_POPUP: popup::Spec = popup::Spec::new(popup::Kind::Fault, Size::new(460.0, 260.0));
const BODY_SIZE: f32 = 14.0;
const SPEED_UP_ITEM: i32 = 0;
const CAT_CPU_ITEM: i32 = 3;
const SNIPER_ITEM: i32 = 5;
const STATUS_LIFETIME: std::time::Duration = std::time::Duration::from_secs(2);
const RESTART_WINDOW: std::time::Duration = std::time::Duration::from_millis(400);
const BODY_PADDING: f32 = 20.0;
const SCROLLBAR_GAP: f32 = 8.0;
const CHOICE_SPACING: f32 = 12.0;
const CHOICE_GAP: f32 = 18.0;
const OPTION_SPACING: f32 = 12.0;
const PLAY_RESERVE: f32 = 70.0;
const PLAY_MARGIN: f32 = 14.0;
const PLAY_WIDTH: f32 = 160.0;
const TAB_WIDTH: f32 = 90.0;
const TAB_HEIGHT: f32 = 28.0;
const TAB_SIZE: f32 = 13.0;
const TAB_SPACING: f32 = 4.0;
const TAB_PADDING: f32 = 6.0;
const FULL_PERCENT: u32 = 100;
const SUPERIOR_TREASURE: i32 = 3;

#[derive(Debug, Clone)]
pub enum Message {
    Popup(popup::Message),
    OpenUrl(String),
    Agree,
    Disagree,
    Play,
    Tick,
    Tab(SandboxTab),
    Lineup(lineup::Message),
    Config(config::Message),
    Stage(stage::Message),
    FaultPopup(popup::Message),
    Terminate,
    Continue,
    Key(String, bool),
    StatusExpired,
}

pub struct State {
    prompt_open: bool,
    prompt: popup::State,
    fault: popup::State,
    terms: Vec<markdown::Item>,
    status: String,
    reported: String,
    session: Option<Session>,
    tapped: Option<std::time::Instant>,
    lineup: lineup::State,
    config: config::State,
}

impl Default for State {
    fn default() -> Self {
        Self {
            prompt_open: false,
            prompt: popup::State::default(),
            fault: popup::State::default(),
            terms: crate::common::markdown::parse(ACKNOWLEDGEMENT),
            status: String::new(),
            reported: String::new(),
            session: None,
            tapped: None,
            lineup: lineup::State::new(0),
            config: config::State::default(),
        }
    }
}

pub(crate) fn key_name(key: &iced::keyboard::Key) -> Option<String> {
    match key {
        iced::keyboard::Key::Character(typed) => Some(typed.to_lowercase()),
        iced::keyboard::Key::Named(named) => Some(format!("{named:?}")),
        iced::keyboard::Key::Unidentified => None,
    }
}

fn game_key(event: iced::Event, _status: iced::event::Status, _window: iced::window::Id) -> Option<Message> {
    match event {
        iced::Event::Keyboard(iced::keyboard::Event::KeyPressed { key, repeat: false, .. }) => key_name(&key).map(|name| Message::Key(name, true)),
        iced::Event::Keyboard(iced::keyboard::Event::KeyReleased { key, .. }) => key_name(&key).map(|name| Message::Key(name, false)),
        _ => None,
    }
}

impl Message {
    pub(crate) fn dropping(cell: kore::domains::sandbox::Cell) -> Self {
        Self::Lineup(lineup::Message::Drop(cell))
    }
}

impl State {
    pub(crate) fn enter(&mut self, app_state: &AppState, settings: &Settings, ctx: GlobalContext<'_>) -> Task<Message> {
        self.prompt_open = !app_state.sandbox.acknowledged;
        self.config.enter(&ctx.vault.vfs);
        self.lineup.set_banner_form(settings.sandbox.banner_form);
        self.lineup.enter(app_state, ctx).map(Message::Lineup)
    }

    pub(crate) fn set_rules(&mut self, rules: kore::domains::sandbox::rules::Rules, app_state: &AppState) {
        self.lineup.set_rules(rules, app_state);
    }

    pub(crate) fn forget_assets(&mut self) {
        if let Some(session) = self.session.as_mut() {
            session.forget_assets();
        }
    }

    pub(crate) fn subscription(&self) -> iced::Subscription<Message> {
        if self.session.as_ref().is_some_and(Session::running) {
            return iced::event::listen_with(game_key);
        }

        self.lineup.subscription().map(Message::Lineup)
    }

    pub(crate) fn icon_stream(&mut self) -> Task<Message> {
        self.lineup.icon_stream().map(Message::Lineup)
    }

    pub(crate) fn adopt_cats(&mut self, cats: &[CatEntry], app_state: &AppState, ctx: GlobalContext<'_>) -> Task<Message> {
        self.config.reload(&ctx.vault.vfs);
        self.lineup.adopt_cats(cats, app_state, ctx).map(Message::Lineup)
    }

    pub(crate) fn set_banner_form(&mut self, banner_form: usize) {
        self.lineup.set_banner_form(banner_form);
    }

    pub(crate) fn inspector(&self) -> Option<&cat::State> {
        self.lineup.unit_open().then(|| self.lineup.inspector())
    }

    pub(crate) fn unit_popup_open(&self) -> bool {
        self.lineup.unit_open()
    }

    pub(crate) fn orb_popup_open(&self) -> bool {
        self.lineup.orb_open()
    }

    pub(crate) fn filter_popup_open(&self) -> bool {
        self.lineup.filter_open()
    }

    pub(crate) fn unit_popup_view<'a>(
        &'a self,
        window: Size,
        settings: &'a Settings,
        app_state: &'a AppState,
        ctx: GlobalContext<'a>,
    ) -> Option<Element<'a, Message>> {
        self.lineup.unit_popup_view(window, settings, app_state, ctx).map(|view| view.map(Message::Lineup))
    }

    pub(crate) fn orb_popup_view<'a>(&'a self, window: Size, app_state: &'a AppState) -> Option<Element<'a, Message>> {
        self.lineup.orb_popup_view(window, app_state).map(|view| view.map(Message::Lineup))
    }

    pub(crate) fn filter_popup_view(&self, window: Size) -> Option<Element<'_, Message>> {
        self.lineup.filter_popup_view(window).map(|view| view.map(Message::Lineup))
    }

    pub(crate) fn setup(&self, app_state: &AppState, stage: StageEntry) -> Setup {
        let options = &app_state.sandbox;
        let parts = self.config.parts();
        let mut setup = Setup { stage, ..Setup::default() };

        setup.lineup = options
            .roster
            .current()
            .into_iter()
            .flat_map(|lineup| lineup.slots.iter())
            .map(|member| {
                let (level, plus) = member.levels();
                let groups = self.lineup.inspector().cat(member.id).and_then(|cat| cat.talent_data.as_ref());

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
                    orbs: member.equipped().map(|(slot, orb)| (slot as i32, orb as i32)).collect(),
                }
            })
            .collect();

        setup.tech[0] = TechLevel { level: 1, plus: 0 };

        for (index, tech) in TECHS.iter().enumerate() {
            let (level, plus) = options.config.tech(index);

            if let Some(held) = setup.tech.get_mut(tech.slot) {
                *held = TechLevel { level, plus };
            }
        }

        for (chapter, stages) in setup.treasures.iter_mut().enumerate() {
            let percent = if chapter < options.config.treasures.len() { options.config.treasure(chapter) } else { FULL_PERCENT };
            let owned = (TREASURE_STAGES as u32 * percent).div_ceil(FULL_PERCENT) as usize;

            for level in stages.iter_mut().take(owned) {
                *level = SUPERIOR_TREASURE;
            }
        }

        setup.altar = options.config.altar();
        setup.cat_god = match options.config.cat_god {
            CatGod::Absent => ::emu::runtime::CatGod::Absent,
            CatGod::Present => ::emu::runtime::CatGod::Present,
            CatGod::Discounted => ::emu::runtime::CatGod::Discounted,
        };

        for (item, off) in options.config.items_off.iter().enumerate() {
            if let Some(stocked) = setup.items.get_mut(item) {
                *stocked = !off;
            }
        }

        setup.speed_engaged = options.config.start_speed.engaged();

        setup.cannon = options.config.cannon.unwrap_or(0);
        setup.style = options.config.style.unwrap_or(0);
        setup.foundation = options.config.foundation.unwrap_or(0);

        setup.parts.entry(setup.cannon).or_default().cannon =
            config::level(&options.config.cannon_level, &parts.cannons, options.config.cannon);
        setup.parts.entry(setup.style).or_default().style =
            config::level(&options.config.style_level, &parts.styles, options.config.style);
        setup.parts.entry(setup.foundation).or_default().foundation =
            config::level(&options.config.foundation_level, &parts.foundations, options.config.foundation);

        for levels in setup.parts.values_mut() {
            levels.cannon = levels.cannon.max(1);
        }

        setup
    }

    pub fn transitioning(&self) -> bool {
        self.session.as_ref().is_some_and(Session::running)
    }

    pub fn curtain_frame(&self) -> Option<&Rc<RefCell<EmuFrame>>> {
        self.session.as_ref().map(Session::frame)
    }

    pub fn curtain_covered(&self) -> bool {
        self.session.as_ref().is_some_and(Session::covered)
    }

    pub fn curtain_sheets(&self) -> Option<&Rc<RefCell<SheetCache>>> {
        self.session.as_ref().map(Session::sheets)
    }

    pub fn design_width(&self) -> f32 {
        self.session.as_ref().map_or(0.0, Session::design_width)
    }

    pub fn curtain_touches(&self) -> Option<&crate::systems::emu::TouchQueue> {
        self.session.as_ref().map(Session::touches)
    }

    pub fn resize(&mut self, width: f32, height: f32) {
        if let Some(session) = self.session.as_mut() {
            session.resize(width, height);
        }
    }

    pub(crate) fn start(&mut self, width: f32, height: f32, options: &SandboxState, setup: Setup) {
        let session = self.session.get_or_insert_with(Session::new);

        session.equip(setup);
        session.set_phone(options.device == SandboxDevice::Phone);
        session.resize(width, height);
        session.configure(BattleOptions {
            music: SandboxVolume::nearest(options.music_volume).percent(),
            effects: SandboxVolume::nearest(options.effects_volume).percent(),
            two_rows: options.two_rows,
            vibrate: options.vibrate,
        });
        session.begin();
        self.status = String::new();
        self.reported = String::new();
    }

    pub(crate) fn update(&mut self, message: Message, settings: &mut Settings, app_state: &mut AppState, ctx: GlobalContext<'_>) -> Task<Message> {
        match message {
            Message::Popup(msg) => {
                if self.prompt.update(msg, ACKNOWLEDGE_POPUP) {
                    return Task::done(Message::Disagree);
                }

                Task::none()
            }
            Message::OpenUrl(url) => {
                if let Err(err) = open::that(&url) {
                    warn!("Failed to open URL {}: {}", url, err);
                }

                Task::none()
            }
            Message::Agree => {
                app_state.sandbox.acknowledged = true;
                self.prompt_open = false;

                Task::none()
            }
            Message::Disagree => {
                self.prompt_open = false;

                Task::none()
            }
            Message::Play => Task::none(),
            Message::FaultPopup(msg) => {
                if self.fault.update(msg, FAULT_POPUP) {
                    return Task::done(Message::Terminate);
                }

                Task::none()
            }
            Message::Continue => {
                if let Some(session) = self.session.as_mut() {
                    session.resume();
                }

                Task::none()
            }
            Message::StatusExpired => {
                self.status.clear();

                Task::none()
            }
            Message::Key(name, pressed) => {
                let Some(session) = self.session.as_mut() else {
                    return Task::none();
                };
                let Some(bind) = settings.sandbox.keys.bound(&name) else {
                    return Task::none();
                };

                let action = match bind {
                    Bind::Slot(slot) => Action::Slot(i32::from(slot)),
                    Bind::SpeedUp => Action::Item(SPEED_UP_ITEM),
                    Bind::CatCpu => Action::Item(CAT_CPU_ITEM),
                    Bind::Sniper => Action::Item(SNIPER_ITEM),
                    Bind::Worker => Action::Worker,
                    Bind::Cannon => Action::Cannon,
                    Bind::ZoomOut => Action::ZoomOut,
                    Bind::ZoomIn => Action::ZoomIn,
                    Bind::Left => Action::PanLeft,
                    Bind::Right => Action::PanRight,
                    Bind::Pause => Action::Pause,
                    Bind::Restart => {
                        if pressed {
                            let now = std::time::Instant::now();
                            let doubled = self.tapped.take().is_some_and(|at| now.duration_since(at) <= RESTART_WINDOW);

                            if doubled {
                                session.restart();
                            } else {
                                self.tapped = Some(now);
                            }
                        }

                        return Task::none();
                    }
                };

                session.key(action, pressed);

                Task::none()
            }
            Message::Terminate => {
                if let Some(session) = self.session.as_mut() {
                    session.terminate();
                }

                Task::none()
            }
            Message::Tab(tab) => {
                app_state.sandbox.tab = tab;

                Task::none()
            }
            Message::Stage(_) => Task::none(),
            Message::Lineup(msg) => self.lineup.update(msg, settings, app_state, ctx).map(Message::Lineup),
            Message::Config(msg) => {
                self.config.update(msg, &mut app_state.sandbox);

                if let Some(session) = self.session.as_mut() {
                    session.set_phone(app_state.sandbox.device == SandboxDevice::Phone);
                }

                self.retune(&app_state.sandbox);

                Task::none()
            }
            Message::Tick => {
                if let Some(session) = self.session.as_mut() {
                    session.tick(&ctx.vault.vfs);

                    if let Some(options) = session.take_options() {
                        app_state.sandbox.music_volume = options.music;
                        app_state.sandbox.effects_volume = options.effects;
                        app_state.sandbox.two_rows = options.two_rows;
                        app_state.sandbox.vibrate = options.vibrate;
                    }

                    let failure = session.failure().filter(|_| !session.faulted()).map_or_else(String::new, str::to_owned);

                    if failure != self.reported {
                        self.reported.clone_from(&failure);
                        self.status = failure;

                        if !self.status.is_empty() {
                            return Task::future(smol::Timer::after(STATUS_LIFETIME)).map(|_| Message::StatusExpired);
                        }
                    }
                }

                Task::none()
            }
        }
    }

    pub fn fault_open(&self) -> bool {
        self.session.as_ref().is_some_and(Session::faulted)
    }

    pub fn frozen(&self) -> bool {
        self.fault_open()
    }

    pub fn fault_view(&self, window: Size) -> Option<Element<'_, Message>> {
        let reason = self.session.as_ref().filter(|session| session.faulted())?.failure()?;

        Some(self.fault.view("FAULT", FAULT_POPUP, window, Message::FaultPopup, move || Self::fault_content(reason), None))
    }

    fn fault_content(reason: &str) -> Element<'_, Message> {
        column![
            text("The game attempted to crash due to a fault").size(BODY_SIZE),
            smooth_scroll(scrollable(text(reason).size(BODY_SIZE)).width(Length::Fill).height(Length::Fill).spacing(SCROLLBAR_GAP)),
            Space::new().height(CHOICE_GAP),
            row![
                theme::sized_button("Continue", theme::POPUP_ACTION_BUTTON_WIDTH, theme::success_button).on_press(Message::Continue),
                theme::sized_button("Terminate", theme::POPUP_ACTION_BUTTON_WIDTH, theme::danger_button).on_press(Message::Terminate),
            ]
                .spacing(CHOICE_SPACING),
        ]
            .spacing(OPTION_SPACING)
            .align_x(Alignment::Center)
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(BODY_PADDING)
            .into()
    }

    pub fn prompt_open(&self) -> bool {
        self.prompt_open
    }

    pub fn prompt_view(&self, window: Size, ui_theme: Theme) -> Option<Element<'_, Message>> {
        self.prompt_open.then(|| {
            self.prompt.view("Acknowledgement", ACKNOWLEDGE_POPUP, window, Message::Popup, move || self.prompt_content(&ui_theme), None)
        })
    }

    fn prompt_content(&self, ui_theme: &Theme) -> Element<'_, Message> {
        column![
            smooth_scroll(
                scrollable(
                    markdown::view(&self.terms, markdown::Settings::with_text_size(BODY_SIZE, theme::markdown_style(ui_theme)))
                        .map(Message::OpenUrl)
                )
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .spacing(SCROLLBAR_GAP)
            ),
            Space::new().height(CHOICE_GAP),
            row![
                theme::sized_button("Agree", theme::POPUP_ACTION_BUTTON_WIDTH, theme::success_button).on_press(Message::Agree),
                theme::sized_button("Disagree", theme::POPUP_ACTION_BUTTON_WIDTH, theme::danger_button).on_press(Message::Disagree),
            ]
                .spacing(CHOICE_SPACING),
        ]
            .align_x(Alignment::Center)
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(BODY_PADDING)
            .into()
    }

    fn retune(&mut self, options: &SandboxState) {
        if let Some(session) = self.session.as_mut() {
            session.retune(options.music_volume, options.effects_volume);
        }
    }

    pub(crate) fn view<'a>(
        &'a self,
        app_state: &'a AppState,
        staged: Option<Element<'a, stage::Message>>,
        playable: bool,
    ) -> Element<'a, Message> {
        let options = &app_state.sandbox;
        let tabs = [(SandboxTab::Lineup, "Lineup"), (SandboxTab::Stage, "Stage"), (SandboxTab::Config, "Config")];
        let mut bar = row![].spacing(TAB_SPACING);

        for (tab, label) in tabs {
            let chosen = options.tab == tab;

            bar = bar.push(
                button(theme::centered_text(label).size(TAB_SIZE))
                    .width(Length::Fixed(TAB_WIDTH))
                    .height(Length::Fixed(TAB_HEIGHT))
                    .on_press(Message::Tab(tab))
                    .style(move |theme: &Theme, status| theme::header_toggle_button(theme, status, chosen, true)),
            );
        }

        let body: Element<'a, Message> = match (options.tab, staged) {
            (SandboxTab::Stage, Some(staged)) => staged.map(Message::Stage),
            (SandboxTab::Config, _) => self.config.view(options, PLAY_RESERVE).map(Message::Config),
            _ => self.lineup.view(app_state).map(Message::Lineup),
        };

        let page = column![
            container(bar).width(Length::Fill).align_x(Horizontal::Center).padding([TAB_PADDING, 0.0]),
            rule::horizontal(1),
            body,
        ]
            .width(Length::Fill)
            .height(Length::Fill);

        let play = container(
            column![
                text(self.status.as_str()).size(BODY_SIZE),
                theme::sized_button("Play", PLAY_WIDTH, theme::primary_button).on_press_maybe(playable.then_some(Message::Play)),
            ]
                .spacing(4)
                .align_x(Alignment::Center),
        )
            .width(Length::Fill)
            .height(Length::Fill)
            .align_x(Horizontal::Center)
            .align_y(Vertical::Bottom)
            .padding(PLAY_MARGIN);

        stack![page, play].into()
    }

    pub(crate) fn playable(app_state: &AppState, staged: bool) -> bool {
        staged && app_state.sandbox.roster.current().is_some_and(|lineup| !lineup.slots.is_empty())
    }
}
