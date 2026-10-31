use std::rc::Rc;

use dioxus::prelude::*;

use kokoro::domains::cat::scanner::CatEntry;
use kokoro::domains::enemy::scanner::EnemyEntry;
use kokoro::domains::settings::ScannerConfig;
use kokoro::domains::stage::scanner::StageBundle;

use crate::platform::{Fault, Pending, Platform, Progress, Report};

#[derive(Clone)]
pub(crate) enum Load<T> {
    Idle,
    Loading(Progress),
    Ready(Rc<T>),
    Failed(Fault),
}

#[derive(Clone, Copy)]
pub(crate) struct Catalogs {
    cats: Signal<Load<Vec<CatEntry>>>,
    enemies: Signal<Load<Vec<EnemyEntry>>>,
    stages: Signal<Load<StageBundle>>,
}

pub(crate) fn provide() -> Catalogs {
    use_context_provider(|| Catalogs { cats: Signal::new(Load::Idle), enemies: Signal::new(Load::Idle), stages: Signal::new(Load::Idle) })
}

pub(crate) fn catalogs() -> Catalogs {
    use_context::<Catalogs>()
}

fn start<T: 'static>(mut slot: Signal<Load<T>>, pending: impl FnOnce(Report) -> Pending<Result<T, Fault>>) {
    if !matches!(&*slot.read(), Load::Idle | Load::Failed(_)) {
        return;
    }

    slot.set(Load::Loading(Progress { done: 0, total: 0 }));

    let report: Report = Box::new(move |progress| {
        let mut slot = slot;

        if let Load::Loading(held) = &mut *slot.write()
            && progress.done >= held.done
        {
            *held = progress;
        }
    });
    let outcome = pending(report);

    spawn(async move {
        match outcome.await {
            Ok(data) => slot.set(Load::Ready(Rc::new(data))),
            Err(fault) => slot.set(Load::Failed(fault)),
        }
    });
}

impl Catalogs {
    pub(crate) fn cats(&self) -> Load<Vec<CatEntry>> {
        (self.cats)()
    }

    pub(crate) fn enemies(&self) -> Load<Vec<EnemyEntry>> {
        (self.enemies)()
    }

    pub(crate) fn stages(&self) -> Load<StageBundle> {
        (self.stages)()
    }

    pub(crate) fn load_cats(&self, platform: &Platform, config: ScannerConfig) {
        start(self.cats, |report| platform.host().cats(config, report));
    }

    pub(crate) fn load_enemies(&self, platform: &Platform, config: ScannerConfig) {
        start(self.enemies, |report| platform.host().enemies(config, report));
    }

    pub(crate) fn load_stages(&self, platform: &Platform, config: ScannerConfig) {
        start(self.stages, |report| platform.host().stages(config, report));
    }

    pub(crate) fn reset(&mut self) {
        self.cats.set(Load::Idle);
        self.enemies.set(Load::Idle);
        self.stages.set(Load::Idle);
    }
}
