use std::cell::RefCell;
use std::collections::HashMap;
use std::time::{Duration, Instant};

use iced::alignment::{Horizontal, Vertical};
use iced::widget::image::Handle;
use iced::widget::{column, container, image, mouse_area, pick_list, responsive, row, scrollable, space, text, text_input, Column, Row};
use iced::{Border, Color, Element, Length, Size, Theme};

use kore::common::context::GlobalContext;
use kore::domains::cat::combo::{self, CatCombo};
use kore::domains::sandbox::rules::Rules;
use kore::domains::sandbox::Lineup;

use crate::app::theme;
use crate::common::header_icon;
use crate::widget::{shrink_by, smooth_scroll};

const DOUBLE_CLICK: Duration = Duration::from_millis(400);
const ROW_HEIGHT: f32 = 58.0;
const ROW_GAP: f32 = 4.0;
const ROW_BUFFER: usize = 4;
const ICON_HEIGHT: f32 = 52.0;
const ICON_GAP: f32 = 3.0;
const NAME_SIZE: f32 = 13.0;
const EFFECT_SIZE: f32 = 12.0;
const FILTER_SIZE: f32 = 12.0;
const EVERY_KIND: &str = "All Types";
const GLYPH_SHARE: f32 = 0.62;
const PICK_CHROME: f32 = 40.0;
const TITLE_WIDTH: f32 = 70.0;
const SEARCH_FLOOR: f32 = 90.0;
const HEADER_GAP: f32 = 8.0;
const TEXT_WIDTH: f32 = 170.0;
const RESTRICTION_SIZE: f32 = 10.0;
const FIT_FLOOR: f32 = 7.0;
const LINE_GAP: f32 = 1.0;
const BOLD_MARGIN: f32 = 0.92;
const BLOCK_FLOOR: f32 = 60.0;
const SLOT_ASPECT: f32 = 110.0 / 85.0;
const SLOT_COUNT: f32 = 5.0;
const SCROLLBAR_RESERVE: f32 = 14.0;
const ROW_PADDING: f32 = 8.0;
const BODY_GAP: f32 = 8.0;
const SMALLEST: f32 = 0.7;
const TEXT_SMALLEST: f32 = 0.8;
const SLOT_FILL: f32 = 0.08;
const BANNED_FILL: f32 = 0.18;
const BANNED_NOTE: &str = "Disabled on this stage";
const PLAY_CLEARANCE: f32 = 60.0;
const HEADER_SIZE: f32 = 16.0;
const CARD_PADDING: f32 = 10.0;
const ACTIVE_FILL: f32 = 1.0;
const IDLE_FILL: f32 = 0.06;
const RADIUS: f32 = 6.0;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Kind {
    effect: Option<i32>,
    label: String,
}

impl std::fmt::Display for Kind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.label)
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum Strength {
    #[default]
    Any,
    Active,
    Inactive,
    Band(i32, String),
}

impl std::fmt::Display for Strength {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Any => "Any Strength",
            Self::Active => "Active",
            Self::Inactive => "Inactive",
            Self::Band(_, label) => label,
        })
    }
}

#[derive(Clone, Debug)]
pub enum Message {
    Search(String),
    Kind(Kind),
    Strength(Strength),
    Click(usize),
    Scrolled(f32),
}

struct Card {
    combo: CatCombo,
    folded: String,
    active: bool,
    banned: bool,
}

#[derive(Default)]
pub struct State {
    cards: Vec<Card>,
    shown: Vec<usize>,
    kinds: Vec<Kind>,
    strengths: Vec<Strength>,
    query: String,
    kind: Option<i32>,
    strength: Strength,
    focus: Option<(u32, usize)>,
    rules: Rules,
    kind_width: f32,
    strength_width: f32,
    clicked: Option<(usize, Instant)>,
    decoded: header_icon::Cache,
    fits: RefCell<HashMap<(usize, u32), f32>>,
    offset: f32,
    built: bool,
}

impl State {
    pub fn invalidate(&mut self) {
        self.built = false;
        self.decoded.borrow_mut().clear();
        self.fits.borrow_mut().clear();
    }

    pub fn ensure(&mut self, ctx: GlobalContext<'_>, lineup: Option<&Lineup>) {
        if self.built {
            return;
        }

        self.cards = combo::every(ctx)
            .into_iter()
            .map(|combo| Card { folded: combo.name.to_lowercase(), combo, active: false, banned: false })
            .collect();

        let mut kinds: Vec<(i32, &str)> = Vec::new();
        let mut bands: Vec<(i32, &str)> = Vec::new();

        for card in &self.cards {
            if !kinds.iter().any(|(effect, _)| *effect == card.combo.effect_type) {
                kinds.push((card.combo.effect_type, card.combo.kind.as_str()));
            }

            if !card.combo.band.is_empty() && !bands.iter().any(|(level, _)| *level == card.combo.effect_level) {
                bands.push((card.combo.effect_level, card.combo.band.as_str()));
            }
        }

        kinds.sort_by_key(|(effect, _)| *effect);
        bands.sort_by_key(|(level, _)| *level);

        self.kinds = std::iter::once(Kind { effect: None, label: EVERY_KIND.to_owned() })
            .chain(kinds.into_iter().map(|(effect, label)| Kind { effect: Some(effect), label: label.to_owned() }))
            .collect();
        self.strengths = [Strength::Any, Strength::Active, Strength::Inactive]
            .into_iter()
            .chain(bands.into_iter().map(|(level, label)| Strength::Band(level, label.to_owned())))
            .collect();

        let widest = |labels: &mut dyn Iterator<Item = usize>| labels.max().unwrap_or(0) as f32 * FILTER_SIZE * GLYPH_SHARE + PICK_CHROME;

        self.kind_width = widest(&mut self.kinds.iter().map(|kind| kind.label.chars().count()));
        self.strength_width = widest(&mut self.strengths.iter().map(|strength| strength.to_string().chars().count()));

        if !self.kinds.iter().any(|kind| kind.effect == self.kind) {
            self.kind = None;
        }

        if !self.strengths.contains(&self.strength) {
            self.strength = Strength::Any;
        }

        self.built = true;
        self.refresh(ctx, lineup);
    }

    pub fn refresh(&mut self, ctx: GlobalContext<'_>, lineup: Option<&Lineup>) {
        let rows = ctx.vault.vds.cats.combos(&ctx.vault.vfs);

        for card in &mut self.cards {
            card.banned = self.rules.bans(card.combo.effect_type);
            card.active = !card.banned && lineup.is_some_and(|lineup| rows.get(card.combo.line).is_some_and(|row| lineup.active(row)));
        }

        self.sift();
    }

    pub fn ban(&mut self, rules: &Rules) {
        self.rules.clone_from(rules);

        for card in &mut self.cards {
            card.banned = rules.bans(card.combo.effect_type);
            card.active &= !card.banned;
        }

        self.sift();
    }

    pub fn focus(&mut self, unit: Option<(u32, usize)>) {
        if self.focus == unit {
            return;
        }

        self.focus = unit;
        self.sift();
    }

    fn sift(&mut self) {
        let cards = &self.cards;
        let query = self.query.trim().to_lowercase();
        let mut shown: Vec<usize> = (0..cards.len())
            .filter(|index| {
                let card = &cards[*index];
                let strong = match &self.strength {
                    Strength::Any => true,
                    Strength::Active => card.active,
                    Strength::Inactive => !card.active,
                    Strength::Band(level, _) => card.combo.effect_level == *level,
                };

                let joined = self.focus.is_none_or(|(unit, form)| card.combo.members.iter().any(|member| member.id == Some(unit) && member.form <= form));

                joined && strong && self.kind.is_none_or(|effect| card.combo.effect_type == effect) && card.folded.contains(&query)
            })
            .collect();

        shown.sort_by(|left, right| {
            let (left, right) = (&cards[*left], &cards[*right]);

            right
                .active
                .cmp(&left.active)
                .then_with(|| left.banned.cmp(&right.banned))
                .then_with(|| left.combo.effect_type.cmp(&right.combo.effect_type))
                .then_with(|| right.combo.effect_level.cmp(&left.combo.effect_level))
                .then_with(|| left.combo.name.cmp(&right.combo.name))
        });

        self.shown = shown;
    }

    pub fn update(&mut self, message: Message) -> Option<usize> {
        match message {
            Message::Search(query) => {
                self.query = query;
                self.sift();

                None
            }
            Message::Kind(kind) => {
                self.kind = kind.effect;
                self.sift();

                None
            }
            Message::Strength(strength) => {
                self.strength = strength;
                self.sift();

                None
            }
            Message::Scrolled(offset) => {
                self.offset = offset;

                None
            }
            Message::Click(line) => {
                let now = Instant::now();
                let doubled = self.clicked.is_some_and(|(held, at)| held == line && now.duration_since(at) <= DOUBLE_CLICK);

                self.clicked = if doubled { None } else { Some((line, now)) };

                doubled.then_some(line)
            }
        }
    }

    pub fn view(&self, width: f32) -> Element<'_, Message> {
        let kind = self.kinds.iter().find(|kind| kind.effect == self.kind);
        let search = text_input("Name...", &self.query)
            .on_input(Message::Search)
            .padding(4)
            .size(FILTER_SIZE)
            .width(Length::Fill)
            .style(theme::rounded_input);
        let kinds = pick_list(self.kinds.as_slice(), kind, Message::Kind)
            .text_size(FILTER_SIZE)
            .style(theme::combo_box)
            .menu_style(theme::combo_box_menu);
        let strengths = pick_list(self.strengths.as_slice(), Some(&self.strength), Message::Strength)
            .text_size(FILTER_SIZE)
            .style(theme::combo_box)
            .menu_style(theme::combo_box_menu);
        let title = theme::bold_text("Combos").size(HEADER_SIZE);
        let single = TITLE_WIDTH + SEARCH_FLOOR + self.kind_width + self.strength_width + HEADER_GAP * 3.0;

        let header: Element<'_, Message> = if width <= 0.0 || width - CARD_PADDING * 2.0 >= single {
            row![
                title,
                search,
                kinds.width(Length::Fixed(self.kind_width)),
                strengths.width(Length::Fixed(self.strength_width)),
            ]
                .spacing(HEADER_GAP)
                .align_y(Vertical::Center)
                .into()
        } else {
            column![
                row![title, search].spacing(HEADER_GAP).align_y(Vertical::Center),
                row![kinds.width(Length::FillPortion(3)), strengths.width(Length::FillPortion(2))].spacing(HEADER_GAP),
            ]
                .spacing(6)
                .into()
        };

        let list = responsive(move |size: Size| {
            let tall = |scale: f32| self.shown.len() as f32 * (ROW_HEIGHT * scale + ROW_GAP) + PLAY_CLEARANCE > size.height;
            let reserve = if tall(Self::scale(size.width)) { SCROLLBAR_RESERVE } else { 0.0 };
            let inner = size.width - reserve;
            let scale = Self::scale(inner);
            let pitch = ROW_HEIGHT * scale + ROW_GAP;
            let reach = (self.shown.len() as f32 * pitch + PLAY_CLEARANCE - size.height).max(0.0);
            let first = ((self.offset.min(reach) / pitch) as usize).saturating_sub(ROW_BUFFER);
            let fitting = (size.height / pitch).ceil() as usize + ROW_BUFFER * 2;
            let last = (first + fitting).min(self.shown.len());
            let mut rows = Column::new().spacing(ROW_GAP).width(Length::Fill).padding(iced::Padding::default().right(reserve));

            if first > 0 {
                rows = rows.push(space().height(Length::Fixed(first as f32 * pitch - ROW_GAP)));
            }

            for index in &self.shown[first.min(last)..last] {
                rows = rows.push(self.view_card(&self.cards[*index], scale, inner));
            }

            if last < self.shown.len() {
                rows = rows.push(space().height(Length::Fixed((self.shown.len() - last) as f32 * pitch - ROW_GAP)));
            }

            rows = rows.push(space().height(Length::Fixed(PLAY_CLEARANCE)));

            smooth_scroll(
                scrollable(rows)
                    .on_scroll(|viewport| Message::Scrolled(viewport.absolute_offset().y))
                    .width(Length::Fill)
                    .height(Length::Fill),
            )
            .into()
        });

        container(column![header, list].spacing(8))
            .padding(CARD_PADDING)
            .width(Length::Fill)
            .height(Length::Fill)
            .style(theme::outlined_card_container)
            .into()
    }

    fn scale(width: f32) -> f32 {
        let slots = ICON_HEIGHT * SLOT_ASPECT * SLOT_COUNT;

        ((Self::spare(width) - Self::gaps()) / (slots + TEXT_WIDTH)).clamp(SMALLEST, 1.0)
    }

    fn gaps() -> f32 {
        ICON_GAP * (SLOT_COUNT - 1.0)
    }

    fn spare(width: f32) -> f32 {
        width - ROW_PADDING * 2.0 - BODY_GAP
    }

    fn view_card<'a>(&'a self, card: &'a Card, scale: f32, width: f32) -> Element<'a, Message> {
        let (slot_width, slot_height) = (ICON_HEIGHT * SLOT_ASPECT * scale, ICON_HEIGHT * scale);
        let mut icons = Row::new().spacing(ICON_GAP).align_y(Vertical::Center);

        for member in &card.combo.members {
            let drawn = member.icon.as_ref().and_then(|source| header_icon::load(&self.decoded, source));

            let slot: Element<'a, Message> = match drawn {
                Some(icon) => {
                    let (across, down) = icon.scale(slot_width, slot_height);
                    let handle: Handle = icon.handle;

                    container(image(handle).width(Length::Fixed(across)).height(Length::Fixed(down)))
                        .center_x(Length::Fixed(slot_width))
                        .center_y(Length::Fixed(slot_height))
                        .into()
                }
                None => container(space())
                    .width(Length::Fixed(slot_width))
                    .height(Length::Fixed(slot_height))
                    .style(|theme: &Theme| container::Style {
                        background: Some(Color { a: SLOT_FILL, ..theme.palette().text }.into()),
                        border: Border::default().rounded(theme::RADIUS_SM),
                        ..container::Style::default()
                    })
                    .into(),
            };

            icons = icons.push(slot);
        }

        let shrunk = scale.max(TEXT_SMALLEST);
        let taken = slot_width * SLOT_COUNT + Self::gaps();
        let block = (Self::spare(width) - taken).min(TEXT_WIDTH * scale).max(BLOCK_FLOOR);
        let mut lines = vec![(card.combo.name.clone(), NAME_SIZE * shrunk), (card.combo.effect.clone(), EFFECT_SIZE * shrunk)];

        match &card.combo.restriction {
            _ if card.banned => lines.push((BANNED_NOTE.to_owned(), RESTRICTION_SIZE * shrunk)),
            Some(restriction) => lines.push((restriction.clone(), RESTRICTION_SIZE * shrunk)),
            None => (),
        }

        let shrink = *self
            .fits
            .borrow_mut()
            .entry((card.combo.line, block.round() as u32 * 2 + u32::from(card.banned)))
            .or_insert_with(|| shrink_by(&lines, block * BOLD_MARGIN, FIT_FLOOR));
        let mut labels = Column::new().spacing(LINE_GAP).width(Length::Fixed(block)).align_x(Horizontal::Center).clip(true);

        for (place, (content, size)) in lines.into_iter().enumerate() {
            let sized = (size - shrink).max(FIT_FLOOR);
            let line = if place == 0 { theme::bold_text(content) } else { text(content) };

            labels = labels.push(line.size(sized).wrapping(text::Wrapping::None));
        }

        let (active, banned) = (card.active, card.banned);
        let body = container(row![labels, icons].spacing(BODY_GAP).align_y(Vertical::Center))
            .height(Length::Fixed(ROW_HEIGHT * scale))
            .width(Length::Fill)
            .padding([0.0, ROW_PADDING])
            .align_y(Vertical::Center)
            .style(move |theme: &Theme| {
                let palette = theme.palette();
                let fill = match (active, banned) {
                    (_, true) => Color { a: BANNED_FILL, ..palette.danger },
                    (true, false) => Color { a: ACTIVE_FILL, ..palette.success },
                    (false, false) => Color { a: IDLE_FILL, ..palette.text },
                };

                container::Style {
                    background: Some(fill.into()),
                    border: Border::default().rounded(RADIUS),
                    ..container::Style::default()
                }
            });

        mouse_area(body).on_press(Message::Click(card.combo.line)).into()
    }
}
