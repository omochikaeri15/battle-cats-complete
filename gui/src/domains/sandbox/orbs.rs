use std::cell::RefCell;
use std::collections::HashMap;
use std::fs;
use std::sync::Arc;

use iced::alignment::Vertical;
use iced::widget::image::Handle;
use iced::widget::{column, container, image as iced_image, pick_list, row, text, Space};
use iced::{Element, Length, Task};

use kore::common::formats::SpriteSheet as CoreSpriteSheet;
use kore::domains::sandbox::orb::{self, Allowance, Kind, GRADES};
use kore::{Vault, Vfs};
use image::{imageops, RgbaImage};
use nyanko::cat::unit::Equipment;

use crate::app::theme;
use crate::common::{ensure_sheet_loaded, SpriteSheet};

const EXPLANATION: &str = "equipment_explonation.tsv";
const PLACEHOLDER: &str = "%@";
const ROW_SPACING: f32 = 8.0;
const LABEL_WIDTH: f32 = 44.0;
const PICK_WIDTH: f32 = 230.0;
const PICK_TEXT: f32 = 12.0;
const PREVIEW_SIZE: f32 = 56.0;
const POPUP_PADDING: f32 = 10.0;
const UNIVERSAL: i32 = 12;
const OVERSAMPLE: f32 = 2.0;
const BEST_GRADE: usize = 4;
const IDLE_PADDING: f32 = 5.0;

const TRAITS: [&str; 12] =
    ["Red", "Floating", "Dark", "Metal", "Angel", "Alien", "Zombie", "Relic", "Traitless", "Witch", "Eva", "Aku"];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Layer {
    Attribute,
    Effect,
    Grade,
}

impl Layer {
    const ALL: [Self; 3] = [Self::Attribute, Self::Effect, Self::Grade];

    fn sheet(self) -> &'static str {
        match self {
            Self::Attribute => "equipment_attribute",
            Self::Effect => "equipment_effect",
            Self::Grade => "equipment_grade",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Choice {
    content: i32,
    label: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Aim(i32);

impl std::fmt::Display for Aim {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match usize::try_from(self.0).ok().and_then(|index| TRAITS.get(index)) {
            Some(named) => f.write_str(named),
            None => write!(f, "Trait {}", self.0),
        }
    }
}

impl std::fmt::Display for Choice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.label)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Grade(usize);

impl std::fmt::Display for Grade {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(GRADES.get(self.0).copied().unwrap_or("?"))
    }
}

#[derive(Clone)]
pub enum Message {
    Sheet(Layer, usize, Option<CoreSpriteSheet>),
    Effect(Choice),
    Aim(Aim),
    Grade(Grade),
    Clear,
}

impl std::fmt::Debug for Message {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Sheet(layer, index, _) => write!(f, "Sheet({layer:?}, {index})"),
            Self::Effect(choice) => write!(f, "Effect({choice})"),
            Self::Aim(aim) => write!(f, "Aim({aim})"),
            Self::Grade(grade) => write!(f, "Grade({grade})"),
            Self::Clear => write!(f, "Clear"),
        }
    }
}

pub enum Outcome {
    Keep,
    Equip(Option<u32>),
}

#[derive(Default)]
pub struct State {
    orbs: Arc<Vec<Equipment>>,
    kinds: Vec<Kind>,
    choices: Vec<Choice>,
    sheets: [Vec<SpriteSheet>; 3],
    drawn: RefCell<HashMap<(u32, u32), Handle>>,
}

impl State {
    pub fn load(&mut self, vault: &Vault) -> Task<Message> {
        let vfs = &vault.vfs;
        let effects = effect_names(vfs);

        self.orbs = vault.vds.cats.orbs(vfs);
        self.kinds = orb::kinds(&self.orbs);
        self.choices = orb::effects(&self.kinds)
            .into_iter()
            .map(|content| Choice { content, label: label(&effects, content) })
            .collect();

        self.drawn.borrow_mut().clear();

        Task::batch(Layer::ALL.into_iter().enumerate().map(|(slot, layer)| {
            for sheet in &mut self.sheets[slot] {
                sheet.mark_stale();
            }

            ensure_sheet_loaded(&mut self.sheets[slot], vfs, layer.sheet(), false)
                .map(move |(index, sheet)| Message::Sheet(layer, index, sheet))
        }))
    }

    pub fn update(&mut self, message: Message, held: Option<u32>, allowance: Allowance) -> Outcome {
        match message {
            Message::Sheet(layer, index, sheet) => {
                let slot = layer as usize;

                if let Some(target) = self.sheets[slot].get_mut(index) {
                    target.apply(sheet);
                    self.drawn.borrow_mut().clear();
                }

                Outcome::Keep
            }
            Message::Effect(choice) => {
                let grade = held.and_then(|orb| orb::grade(&self.orbs, orb)).unwrap_or(BEST_GRADE);
                let aimed = held.and_then(|orb| orb::kind(&self.orbs, orb)).and_then(|kind| kind.attribute);
                let offered = self.aims(choice.content, allowance);
                let attribute = offered.iter().find(|aim| Some(aim.0) == aimed).or_else(|| offered.first()).map(|aim| aim.0);

                Outcome::Equip(orb::pick(&self.orbs, Kind { content: choice.content, attribute }, grade))
            }
            Message::Aim(aim) => {
                let Some(kind) = held.and_then(|orb| orb::kind(&self.orbs, orb)) else {
                    return Outcome::Keep;
                };
                let grade = held.and_then(|orb| orb::grade(&self.orbs, orb)).unwrap_or(BEST_GRADE);

                Outcome::Equip(orb::pick(&self.orbs, Kind { attribute: Some(aim.0), ..kind }, grade))
            }
            Message::Grade(grade) => {
                let Some(kind) = held.and_then(|orb| orb::kind(&self.orbs, orb)) else {
                    return Outcome::Keep;
                };

                Outcome::Equip(orb::pick(&self.orbs, kind, grade.0))
            }
            Message::Clear => Outcome::Equip(None),
        }
    }

    fn aims(&self, content: i32, allowance: Allowance) -> Vec<Aim> {
        orb::attributes(&self.kinds, content)
            .into_iter()
            .filter(|attribute| allowance.permits(Kind { content, attribute: Some(*attribute) }))
            .map(Aim)
            .collect()
    }

    fn cut(&self, layer: Layer, index: i32) -> Option<RgbaImage> {
        let index = usize::try_from(index).ok()?;

        self.sheets[layer as usize].iter().find_map(|sheet| {
            let cut = sheet.core.cuts_map.get(&index)?;
            let pixels = sheet.core.image_data.as_ref()?;
            let (left, top) = (u32::try_from(cut.x).ok()?, u32::try_from(cut.y).ok()?);
            let (width, height) = (u32::try_from(cut.width).ok()?, u32::try_from(cut.height).ok()?);

            (width > 1 && height > 1 && left + width <= pixels.width() && top + height <= pixels.height())
                .then(|| imageops::crop_imm(pixels.as_ref(), left, top, width, height).to_image())
        })
    }

    fn drawn(&self, orb: u32, edge: u32) -> Option<Handle> {
        if let Some(held) = self.drawn.borrow().get(&(orb, edge)) {
            return Some(held.clone());
        }

        let held = self.orbs.get(orb as usize)?;
        let mut face = self.cut(Layer::Attribute, held.attribute.unwrap_or(UNIVERSAL))?;

        for (layer, index) in [(Layer::Effect, held.content), (Layer::Grade, held.grade_id)] {
            if let Some(over) = index.and_then(|index| self.cut(layer, index)) {
                let (across, down) = (face.width().saturating_sub(over.width()) / 2, face.height().saturating_sub(over.height()) / 2);

                imageops::overlay(&mut face, &over, i64::from(across), i64::from(down));
            }
        }

        let scaled = imageops::resize(&face, edge, edge, imageops::FilterType::Lanczos3);
        let handle = Handle::from_rgba(edge, edge, scaled.into_raw());

        self.drawn.borrow_mut().insert((orb, edge), handle.clone());

        Some(handle)
    }

    pub fn picture<'a, M: 'a>(&self, orb: u32, size: f32) -> Element<'a, M> {
        let edge = ((size * OVERSAMPLE).round() as u32).max(1);

        self.drawn(orb, edge).map_or_else(
            || Space::new().width(Length::Fixed(size)).height(Length::Fixed(size)).into(),
            |handle| iced_image(handle).width(Length::Fixed(size)).height(Length::Fixed(size)).into(),
        )
    }

    pub fn view(&self, held: Option<u32>, allowance: Allowance) -> Element<'_, Message> {
        let kind = held.and_then(|orb| orb::kind(&self.orbs, orb));
        let offered: Vec<Choice> = self
            .choices
            .iter()
            .filter(|choice| orb::attributes(&self.kinds, choice.content).is_empty() || !self.aims(choice.content, allowance).is_empty())
            .cloned()
            .collect();
        let chosen = kind.and_then(|kind| offered.iter().find(|choice| choice.content == kind.content)).cloned();
        let aims = kind.map_or_else(Vec::new, |kind| self.aims(kind.content, allowance));
        let aim = kind.and_then(|kind| kind.attribute).map(Aim);
        let trait_pick: Element<'_, Message> = if aims.is_empty() {
            container(text("None").size(PICK_TEXT).style(|theme: &iced::Theme| text::Style { color: Some(theme::weak_text_color(theme)) }))
                .padding(IDLE_PADDING)
                .width(Length::Fixed(PICK_WIDTH))
                .style(theme::combo_box_idle)
                .into()
        } else {
            pick_list(aims, aim, Message::Aim)
                .placeholder("None")
                .width(Length::Fixed(PICK_WIDTH))
                .text_size(PICK_TEXT)
                .style(theme::combo_box)
                .menu_style(theme::combo_box_menu)
                .into()
        };
        let grades: Vec<Grade> = kind.map_or_else(Vec::new, |kind| {
            let mut offered: Vec<Grade> = orb::grades(&self.orbs, kind).into_iter().map(|(grade, _)| Grade(grade)).collect();

            offered.sort_by_key(|grade| grade.0);
            offered
        });
        let grade = held.and_then(|orb| orb::grade(&self.orbs, orb)).map(Grade);

        let preview: Element<'_, Message> = held.map_or_else(
            || Space::new().width(Length::Fixed(PREVIEW_SIZE)).height(Length::Fixed(PREVIEW_SIZE)).into(),
            |orb| self.picture(orb, PREVIEW_SIZE),
        );

        column![
            preview,
            row![
                text("Type").size(PICK_TEXT).width(Length::Fixed(LABEL_WIDTH)),
                pick_list(offered, chosen, Message::Effect)
                    .placeholder("None")
                    .width(Length::Fixed(PICK_WIDTH))
                    .text_size(PICK_TEXT)
                    .style(theme::combo_box)
                    .menu_style(theme::combo_box_menu),
            ]
                .spacing(ROW_SPACING)
                .align_y(Vertical::Center),
            row![
                text("Trait").size(PICK_TEXT).width(Length::Fixed(LABEL_WIDTH)),
                trait_pick,
            ]
                .spacing(ROW_SPACING)
                .align_y(Vertical::Center),
            row![
                text("Grade").size(PICK_TEXT).width(Length::Fixed(LABEL_WIDTH)),
                pick_list(grades, grade, Message::Grade)
                    .placeholder("None")
                    .width(Length::Fixed(PICK_WIDTH))
                    .text_size(PICK_TEXT)
                    .style(theme::combo_box)
                    .menu_style(theme::combo_box_menu),
            ]
                .spacing(ROW_SPACING)
                .align_y(Vertical::Center),
            theme::sized_button("None", theme::POPUP_ACTION_BUTTON_WIDTH, theme::danger_button)
                .on_press_maybe(held.map(|_| Message::Clear)),
        ]
            .spacing(ROW_SPACING)
            .align_x(iced::Alignment::Center)
            .width(Length::Fill)
            .padding(POPUP_PADDING)
            .into()
    }
}

fn effect_names(vfs: &Vfs) -> Vec<String> {
    let named = vfs.variants(EXPLANATION).into_iter().next().unwrap_or_else(|| EXPLANATION.to_owned());

    vfs.find(named.as_str())
        .and_then(|path| fs::read_to_string(path).ok())
        .map_or_else(Vec::new, |content| {
            content
                .lines()
                .map(|line| {
                    let head = line.split('\t').next().unwrap_or_default();

                    head.split(PLACEHOLDER).next().unwrap_or_default().trim().trim_end_matches(':').trim().to_owned()
                })
                .collect()
        })
}

fn label(effects: &[String], content: i32) -> String {
    usize::try_from(content)
        .ok()
        .and_then(|index| effects.get(index))
        .filter(|name| !name.is_empty())
        .map_or_else(|| format!("Effect {content}"), Clone::clone)
}
