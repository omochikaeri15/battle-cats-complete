use std::collections::HashSet;

use iced::alignment::Vertical;
use iced::widget::{button, column, pick_list, row, rule, text, Column};
use iced::{Element, Length, Theme};

use kore::systems::treasure::{Catalog, Config, ARCS, FULL_PERCENT};
use nyanko::chapter::treasure::ARC_SLOTS;

use crate::app::theme;
use crate::widget::{entry_row, subsection};

const CHAPTER_TITLES: [[&str; 3]; 3] = [
    ["Empire of Cats Ch. 1", "Empire of Cats Ch. 2", "Empire of Cats Ch. 3"],
    ["Into the Future Ch. 1", "Into the Future Ch. 2", "Into the Future Ch. 3"],
    ["Cats of the Cosmos Ch. 1", "Cats of the Cosmos Ch. 2", "Cats of the Cosmos Ch. 3"],
];

const ROW_SPACING: f32 = 10.0;
const SET_SPACING: f32 = 4.0;
const FOLD_SPACING: f32 = 6.0;
const FOLD_TITLE_SIZE: f32 = 15.0;
const NOTE_SIZE: f32 = 12.0;
const ITEM_WIDTH: f32 = 190.0;
const ITEM_INDENT: f32 = 12.0;
const ENTRY_DIGITS: usize = 4;

#[derive(Debug, Clone)]
pub enum Message {
    Arc(usize, String),
    Grade(usize, usize, i32),
    Fold(usize, bool),
}

#[derive(Default)]
pub(crate) struct State {
    open: HashSet<usize>,
}

impl State {
    pub(crate) fn update(&mut self, message: Message, config: &mut Config) {
        match message {
            Message::Arc(arc, entry) => {
                if typable(&entry) {
                    config.set_arc(arc, entry);
                }
            }
            Message::Grade(slot, stage, level) => config.set_level(slot, stage, level),
            Message::Fold(slot, open) => {
                if open {
                    self.open.insert(slot);
                } else {
                    self.open.remove(&slot);
                }
            }
        }
    }

    pub(crate) fn view<'a>(&'a self, config: &'a Config, catalog: &'a Catalog) -> Element<'a, Message> {
        let placeholder = FULL_PERCENT.to_string();
        let mut arcs = Column::new().spacing(ROW_SPACING);

        for (arc, name) in ARCS.iter().enumerate() {
            let entry = entry_row(name, &placeholder, &config.arcs[arc], move |entry| Message::Arc(arc, entry));
            let mut line = row![entry].spacing(ROW_SPACING).align_y(Vertical::Center);

            if config.is_tuned(arc) {
                line = line.push(note("Adjusted per treasure below"));
            }

            arcs = arcs.push(line);
        }

        let mut chapters = Column::new().spacing(ROW_SPACING);

        for (arc, slots) in ARC_SLOTS.iter().enumerate() {
            for (chapter, slot) in slots.iter().copied().enumerate() {
                chapters = chapters.push(self.fold(CHAPTER_TITLES[arc][chapter], slot, || chapter_view(config, catalog, slot)));
            }
        }

        column![subsection("General", arcs), subsection("Advanced", chapters)].spacing(ROW_SPACING * 2.0).into()
    }

    fn fold<'a>(&self, title: &'a str, slot: usize, content: impl FnOnce() -> Element<'a, Message>) -> Element<'a, Message> {
        let open = self.open.contains(&slot);
        let mut heading = row![theme::bold_text(title).size(FOLD_TITLE_SIZE)].spacing(FOLD_SPACING).align_y(Vertical::Bottom);

        if !open {
            heading = heading.push(note("(collapsed)"));
        }

        let mut body = column![
            button(heading).padding(0).style(button::text).on_press(Message::Fold(slot, !open)),
            rule::horizontal(1),
        ]
        .spacing(FOLD_SPACING);

        if open {
            body = body.push(content());
        }

        body.into()
    }
}

fn chapter_view<'a>(config: &'a Config, catalog: &'a Catalog, slot: usize) -> Element<'a, Message> {
    let groups = catalog.groups(slot);

    if groups.is_empty() {
        return note("No treasure data was found for this chapter").into();
    }

    let grades = &catalog.grades;
    let mut sets = Column::new().spacing(ROW_SPACING);

    for (group, set) in groups.iter().enumerate() {
        let mut members = set.members().peekable();

        if members.peek().is_none() {
            continue;
        }

        let name: Element<'a, Message> = catalog
            .set(slot, group)
            .map_or_else(|| theme::bold_text(format!("Set {}", group + 1)).into(), |name| theme::bold_text(name).into());

        let mut title = row![name].spacing(FOLD_SPACING).align_y(Vertical::Center);

        if let Some(effect) = catalog.effect(slot, group) {
            title = title.push(note(effect));
        }

        let mut rows = Column::new().spacing(SET_SPACING).push(title);

        for stage in members {
            let Ok(index) = usize::try_from(stage) else { continue };

            let label: Element<'a, Message> = catalog
                .item(slot, stage)
                .map_or_else(|| text(format!("Treasure {}", stage + 1)).into(), |name| text(name).into());

            let level = config.level(slot, index);
            let selected = usize::try_from(level).ok().and_then(|level| grades.get(level));

            let picker = pick_list(&grades[..], selected, move |label: String| {
                let level = grades.iter().position(|grade| *grade == label).unwrap_or(0);

                Message::Grade(slot, index, level as i32)
            })
            .style(theme::combo_box)
            .menu_style(theme::combo_box_menu);

            rows = rows.push(
                row![iced::widget::container(label).width(Length::Fixed(ITEM_WIDTH)), picker]
                    .spacing(ROW_SPACING)
                    .align_y(Vertical::Center)
                    .padding(iced::Padding { left: ITEM_INDENT, ..iced::Padding::ZERO }),
            );
        }

        sets = sets.push(rows);
    }

    sets.into()
}

fn note<'a>(content: &'a str) -> iced::widget::Text<'a> {
    text(content).size(NOTE_SIZE).style(|theme: &Theme| text::Style { color: Some(theme::weak_text_color(theme)) })
}

fn typable(entry: &str) -> bool {
    entry.len() <= ENTRY_DIGITS && entry.chars().all(|glyph| glyph.is_ascii_digit() || glyph == '%')
}
