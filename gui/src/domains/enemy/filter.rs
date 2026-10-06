use iced::{Element, Size, Task};

use kore::domains::enemy::filter::EnemyFilterState;
use kore::domains::enemy::scanner::EnemyEntry;
use kore::systems::combat::NameBook;

use crate::common::{CustomAssets, SpriteSheet};
use crate::systems::combat::enemy_filter::{Art, Catalog, Panel, Profile};
use crate::widget::popup;

pub use crate::systems::combat::enemy_filter::Message;

const PROFILE: Profile = Profile {
    spec: popup::Spec::new(popup::Kind::EnemyFilter, Size::new(600.0, 528.0)),
    title: "Advanced Enemy Filter",
    scroll_id: "enemy-filter-scroll",
    magnification: "100",
};

pub struct State {
    pub filter_state: EnemyFilterState,
    catalog: Catalog,
    panel: Panel,
}

impl Default for State {
    fn default() -> Self {
        Self { filter_state: EnemyFilterState::default(), catalog: Catalog::default(), panel: Panel::new(PROFILE) }
    }
}

impl State {
    pub(super) fn clear_icons(&self) {
        self.catalog.clear_icons();
    }

    pub fn update(&mut self, message: Message) {
        self.panel.update(&mut self.filter_state, message);
    }

    pub(super) fn art<'a>(&'a self, sheets: &'a [SpriteSheet], assets: &'a CustomAssets) -> Art<'a> {
        Art { catalog: &self.catalog, sheets, assets }
    }

    pub fn view<'a>(&'a self, sheets: &'a [SpriteSheet], assets: &'a CustomAssets, window: Size, names: &'a NameBook) -> Element<'a, Message> {
        self.panel.view(&self.filter_state, self.art(sheets, assets), window, names)
    }

    pub(crate) fn restore_scroll<M: 'static>(&self) -> Task<M> {
        self.panel.restore_scroll()
    }

    pub(super) fn refresh_available(&mut self, enemies: &[EnemyEntry]) -> Task<Message> {
        self.catalog.refresh(enemies);

        self.restore_scroll()
    }
}
