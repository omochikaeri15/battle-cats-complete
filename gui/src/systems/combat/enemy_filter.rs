use std::collections::HashSet;

use iced::alignment::{Horizontal, Vertical};
use iced::widget::{button, column, container, image as iced_image, operation, pick_list, row, scrollable, stack, text, text_input, tooltip, Id, Space};
use iced::{Element, Length, Size, Task};
use nyanko::combat::{AttrUnit, Identity, REGISTRY};

use kore::domains::enemy::filter::evaluation::get_identity_name;
use kore::systems::combat::NameBook;
use kore::domains::enemy::filter::{EnemyFilterState, MatchMode, ATTACK_TYPE_IDENTITIES};
use kore::domains::enemy::scanner::EnemyEntry;
use kore::systems::combat::present_identities;
use kore::systems::combat::registry::{get_display_def, AbilityIcon, DisplayGroup};

use crate::app::theme;
use crate::common::ability_icon;
use crate::common::{CustomAssets, SpriteSheet};
use crate::widget::popup;
use crate::widget::range_row;
use crate::widget::{fallback_icon, icons_per_row, smooth_scroll, ICON_SIZE};

const STAT_KEYS: [&str; 8] = [
    "Attack", "Dps", "Range", "Atk Cycle (f)", "Hitpoints", "Knockbacks", "Speed", "Cash Drop",
];

const ICON_SPACING: f32 = 4.0;
const CONTENT_PADDING: f32 = 24.0;
const CLEAR_BTN_CLEARANCE: f32 = 56.0;

#[derive(Debug, Clone)]
pub enum Message {
    Popup(popup::Message),
    Toggle,
    Clear,
    MatchModeChanged(MatchMode),
    MagChanged(String),
    StatMinChanged(&'static str, String),
    StatMaxChanged(&'static str, String),
    IdentityToggled(Identity),
    AdvMinChanged(Identity, &'static str, String),
    AdvMaxChanged(Identity, &'static str, String),
    Scrolled(f32),
}

pub(crate) struct Profile {
    pub(crate) spec: popup::Spec,
    pub(crate) title: &'static str,
    pub(crate) scroll_id: &'static str,
    pub(crate) magnification: &'static str,
}

#[derive(Default)]
pub(crate) struct Catalog {
    available: HashSet<Identity>,
    icons: ability_icon::Cache,
}

impl Catalog {
    pub(crate) fn clear_icons(&self) {
        self.icons.clear();
    }

    pub(crate) fn refresh(&mut self, enemies: &[EnemyEntry]) {
        self.available = present_identities(enemies.iter().map(|enemy| &enemy.stats));
    }
}

#[derive(Clone, Copy)]
pub(crate) struct Art<'a> {
    pub(crate) catalog: &'a Catalog,
    pub(crate) sheets: &'a [SpriteSheet],
    pub(crate) assets: &'a CustomAssets,
}

pub(crate) struct Panel {
    profile: Profile,
    popup: popup::State,
    scroll_offset: f32,
}

impl Panel {
    pub(crate) fn new(profile: Profile) -> Self {
        Self { profile, popup: popup::State::default(), scroll_offset: 0.0 }
    }

    pub(crate) fn update(&mut self, filter: &mut EnemyFilterState, message: Message) {
        match message {
            Message::Popup(msg) => {
                if self.popup.update(msg, self.profile.spec) {
                    filter.is_open = false;
                }
            }
            Message::Toggle => filter.is_open = !filter.is_open,
            Message::Clear => {
                *filter = EnemyFilterState { is_open: filter.is_open, treasure: filter.treasure, ..Default::default() };
            }
            Message::MatchModeChanged(mode) => filter.match_mode = mode,
            Message::MagChanged(mag) => filter.mag_input = mag,
            Message::StatMinChanged(stat, value) => {
                filter.stat_ranges.entry(stat).or_default().min = value;
            }
            Message::StatMaxChanged(stat, value) => {
                filter.stat_ranges.entry(stat).or_default().max = value;
            }
            Message::IdentityToggled(identity) => {
                if filter.active_identities.contains(&identity) {
                    filter.active_identities.remove(&identity);
                } else {
                    filter.active_identities.insert(identity);
                }
            }
            Message::AdvMinChanged(identity, attr, val) => {
                filter.adv_ranges.entry(identity).or_default().entry(attr).or_default().min = val;
            }
            Message::AdvMaxChanged(identity, attr, val) => {
                filter.adv_ranges.entry(identity).or_default().entry(attr).or_default().max = val;
            }
            Message::Scrolled(offset) => self.scroll_offset = offset,
        }
    }

    pub(crate) fn view<'a>(&'a self, filter: &'a EnemyFilterState, art: Art<'a>, window: Size, names: &'a NameBook) -> Element<'a, Message> {
        let spec = self.profile.spec;
        let per_row = icons_per_row(self.popup.body_width(spec, window) - CONTENT_PADDING * 2.0, ICON_SPACING);
        let face = Face { filter, art, names, per_row, scroll_id: self.profile.scroll_id, magnification: self.profile.magnification };

        self.popup.view(self.profile.title, spec, window, Message::Popup, move || face.content_view(), None)
    }

    pub(crate) fn restore_scroll<M: 'static>(&self) -> Task<M> {
        operation::scroll_to(Id::new(self.profile.scroll_id), scrollable::AbsoluteOffset { x: 0.0, y: self.scroll_offset })
    }
}

#[derive(Clone, Copy)]
struct Face<'a> {
    filter: &'a EnemyFilterState,
    art: Art<'a>,
    names: &'a NameBook,
    per_row: usize,
    scroll_id: &'static str,
    magnification: &'static str,
}

impl<'a> Face<'a> {
    fn content_view(self) -> Element<'a, Message> {
        let match_mode_label = if self.filter.match_mode == MatchMode::And { "And" } else { "Or" };

        let mode_row = row![
            text("Mode:").align_y(Vertical::Center),
            pick_list(vec!["And", "Or"], Some(match_mode_label), |s| {
                Message::MatchModeChanged(if s == "And" { MatchMode::And } else { MatchMode::Or })
            }).style(theme::combo_box).menu_style(theme::combo_box_menu),
        ].spacing(8).align_y(Vertical::Center);

        let mag_row = row![
            text("Target Magnification:").align_y(Vertical::Center),
            text_input(self.magnification, &self.filter.mag_input)
                .on_input(Message::MagChanged)
                .width(Length::Fixed(60.0))
                .style(theme::rounded_input),
            text("%"),
        ].spacing(8).align_y(Vertical::Center);

        let mut stats_col = column![].spacing(6);
        for pair in STAT_KEYS.chunks(2) {
            let mut stat_row = row![].spacing(16);
            for &stat in pair {
                stat_row = stat_row.push(stat_range_field(stat, self.filter));
            }
            stats_col = stats_col.push(stat_row);
        }

        let type_identities: Vec<Identity> = REGISTRY.iter()
            .map(|def| def.identity)
            .filter(|identity| self.art.catalog.available.contains(identity))
            .filter(|&identity| get_display_def(identity).group == DisplayGroup::Trait)
            .collect();
        let type_row = self.icon_wrap(&type_identities);

        let attack_row = self.icon_wrap(ATTACK_TYPE_IDENTITIES);

        let mut abilities_col = column![].spacing(0);

        for group in [DisplayGroup::Headline1, DisplayGroup::Headline2] {
            let group_identities = self.collect_group_identities(group);
            if !group_identities.is_empty() {
                abilities_col = abilities_col.push(self.icon_wrap(&group_identities));
                abilities_col = abilities_col.push(Space::new().height(Length::Fixed(8.0)));
            }
        }

        for group in [DisplayGroup::Body1, DisplayGroup::Body2] {
            let group_identities = self.collect_group_identities(group);
            if !group_identities.is_empty() {
                let mut col = column![].spacing(4);
                for identity in group_identities {
                    col = col.push(self.icon_row_with_label(identity));
                }
                abilities_col = abilities_col.push(col);
                abilities_col = abilities_col.push(Space::new().height(Length::Fixed(8.0)));
            }
        }

        let footer_identities = self.collect_group_identities(DisplayGroup::Footer);
        if !footer_identities.is_empty() {
            abilities_col = abilities_col.push(self.icon_wrap(&footer_identities));
        }

        let content = column![
            text("Attributes").size(18),
            mode_row,
            Space::new().height(Length::Fixed(16.0)),
            text("Stats").size(18),
            mag_row,
            stats_col,
            Space::new().height(Length::Fixed(16.0)),
            text("Trait Type").size(18),
            type_row,
            Space::new().height(Length::Fixed(16.0)),
            text("Attack Type").size(18),
            attack_row,
            Space::new().height(Length::Fixed(16.0)),
            text("Abilities").size(18),
            abilities_col,
            Space::new().height(Length::Fixed(CLEAR_BTN_CLEARANCE)),
        ].spacing(8).padding(CONTENT_PADDING);

        let scroll_layer = smooth_scroll(
            scrollable(content)
                .id(Id::new(self.scroll_id))
                .on_scroll(|viewport| Message::Scrolled(viewport.absolute_offset().y))
                .width(Length::Fill)
                .height(Length::Fill),
        );

        let clear_btn = button(text("Clear Filter"))
            .on_press(Message::Clear)
            .padding([8, 16])
            .style(button::danger);

        let clear_btn_layer = container(clear_btn)
            .width(Length::Fill)
            .height(Length::Fill)
            .align_x(Horizontal::Center)
            .align_y(Vertical::Bottom)
            .padding(16);

        stack![scroll_layer, clear_btn_layer]
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    fn icon_wrap(self, identities: &[Identity]) -> Element<'a, Message> {
        let mut col = column![].spacing(ICON_SPACING);
        for chunk in identities.chunks(self.per_row) {
            let mut wrapped_row = row![].spacing(ICON_SPACING).align_y(Vertical::Center);
            for &identity in chunk {
                wrapped_row = wrapped_row.push(self.icon_with_tooltip(identity));
            }
            col = col.push(wrapped_row);
        }
        col.into()
    }

    fn icon_with_tooltip(self, identity: Identity) -> Element<'a, Message> {
        let name = get_identity_name(identity, self.names);

        tooltip(
            self.icon_button(identity),
            container(text(name)).padding(6).style(container::bordered_box),
            tooltip::Position::Top,
        ).into()
    }

    fn icon_row_with_label(self, identity: Identity) -> Element<'a, Message> {
        let name = get_identity_name(identity, self.names);
        let schema = ability_schema(identity);
        let expanded = self.filter.active_identities.contains(&identity) && !schema.is_empty();

        let label_btn = button(text(name))
            .padding(0)
            .style(button::text)
            .on_press(Message::IdentityToggled(identity));

        let header = row![
            self.icon_button(identity),
            label_btn,
        ].spacing(10).align_y(Vertical::Center);

        if !expanded {
            return header.into();
        }

        let mut grid_col = column![].spacing(6);
        for &(attr, _) in schema {
            grid_col = grid_col.push(self.adv_range_row(identity, attr));
        }

        container(column![header, grid_col].spacing(6))
            .padding(8)
            .style(theme::card_container)
            .into()
    }

    fn adv_range_row(self, identity: Identity, attr: &'static str) -> Element<'a, Message> {
        let range = self.filter.adv_ranges.get(&identity).and_then(|ranges| ranges.get(attr));
        let min = range.map_or("", |r| r.min.as_str());
        let max = range.map_or("", |r| r.max.as_str());

        range_row(
            attr,
            min,
            max,
            move |v| Message::AdvMinChanged(identity, attr, v),
            move |v| Message::AdvMaxChanged(identity, attr, v),
        )
    }

    fn icon_button(self, identity: Identity) -> Element<'a, Message> {
        button(self.icon_image(identity))
            .padding(0)
            .style(button::text)
            .on_press(Message::IdentityToggled(identity))
            .into()
    }

    fn icon_image(self, identity: Identity) -> Element<'a, Message> {
        let opacity: f32 = if self.filter.active_identities.contains(&identity) { 1.0 } else { 0.4 };
        let display_def = get_display_def(identity);

        match display_def.icon {
            AbilityIcon::Custom(custom_icon) => {
                if let Some(handle) = self.art.assets.get_icon_texture(custom_icon) {
                    return iced_image(handle).width(Length::Fixed(ICON_SIZE)).height(Length::Fixed(ICON_SIZE)).opacity(opacity).into();
                }
            }
            AbilityIcon::Standard(icon_id) => {
                if let Some(handle) = self.art.catalog.icons.handle(icon_id, self.art.sheets) {
                    return iced_image(handle).width(Length::Fixed(ICON_SIZE)).height(Length::Fixed(ICON_SIZE)).opacity(opacity).into();
                }
            }
            AbilityIcon::None => {}
        }

        fallback_icon("?")
    }

    fn collect_group_identities(self, target_group: DisplayGroup) -> Vec<Identity> {
        let mut identities_in_group = Vec::new();

        for def in REGISTRY.iter() {
            if !self.art.catalog.available.contains(&def.identity) { continue; }

            let display_def = get_display_def(def.identity);
            if display_def.group != target_group { continue; }
            if display_def.group == DisplayGroup::Trait { continue; }
            if ATTACK_TYPE_IDENTITIES.contains(&def.identity) { continue; }
            if identities_in_group.contains(&def.identity) { continue; }

            identities_in_group.push(def.identity);
        }

        identities_in_group
    }
}

fn ability_schema(identity: Identity) -> &'static [(&'static str, AttrUnit)] {
    REGISTRY.iter()
        .find(|def| def.identity == identity)
        .map(|def| def.schema)
        .unwrap_or(&[])
}

fn stat_range_field<'a>(stat: &'static str, filter_state: &'a EnemyFilterState) -> Element<'a, Message> {
    let range = filter_state.stat_ranges.get(stat);
    let min = range.map_or("", |r| r.min.as_str());
    let max = range.map_or("", |r| r.max.as_str());

    range_row(
        stat,
        min,
        max,
        move |v| Message::StatMinChanged(stat, v),
        move |v| Message::StatMaxChanged(stat, v),
    )
}
