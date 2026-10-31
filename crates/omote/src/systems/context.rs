use std::rc::Rc;
use std::sync::Arc;

use dioxus::prelude::*;

use codex::files::{Localizable, Param};
use kokoro::common::context::GlobalContext;
use kokoro::common::game;
use kokoro::domains::settings::Settings;
use kokoro::systems::combat::NameBook;
use kokoro::systems::treasure::{Bonus, Catalog};
use kokoro::Vault;

use crate::systems::library::{Core, Library};

pub(crate) struct Globals {
    pub(crate) core: Rc<Core>,
    pub(crate) param: Param,
    pub(crate) localizable: Localizable,
    pub(crate) names: Arc<NameBook>,
    pub(crate) catalog: Arc<Catalog>,
    pub(crate) treasure: Bonus,
}

#[derive(Clone)]
pub(crate) struct Shared(pub(crate) Rc<Globals>);

impl PartialEq for Shared {
    fn eq(&self, other: &Self) -> bool {
        Rc::ptr_eq(&self.0, &other.0)
    }
}

impl Globals {
    fn build(core: Rc<Core>, settings: &Settings) -> Self {
        let vault = core.vault();
        let param = game::param(&vault.vfs).unwrap_or_default();
        let localizable = game::localizable(&vault.vfs);
        let names = if vault.vds.names.english(&vault.vfs) { Arc::default() } else { Arc::new(vault.vds.names.book(&vault.vfs, &localizable)) };
        let catalog = vault.vds.treasures.catalog(&vault.vfs);
        let treasure = Bonus::resolve(&catalog, &settings.general.treasures);

        Self { core, param, localizable, names, catalog, treasure }
    }

    pub(crate) fn vault(&self) -> &Vault {
        self.core.vault()
    }

    pub(crate) fn context(&self) -> GlobalContext<'_> {
        GlobalContext { param: &self.param, localizable: &self.localizable, vault: self.core.vault(), treasure: &self.treasure, names: &self.names }
    }
}

pub(crate) fn provide(library: Library, settings: Signal<Settings>) -> Memo<Option<Shared>> {
    let globals = use_memo(move || library.tables().map(|core| Shared(Rc::new(Globals::build(core, &settings.read())))));

    use_context_provider(|| globals);

    globals
}

pub(crate) fn globals() -> Option<Shared> {
    use_context::<Memo<Option<Shared>>>()()
}

pub(crate) fn settings() -> Signal<Settings> {
    use_context::<Signal<Settings>>()
}
