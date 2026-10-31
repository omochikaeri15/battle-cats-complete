use std::cell::RefCell;
use std::collections::HashSet;
use std::rc::Rc;
use std::sync::{Arc, Mutex, PoisonError};

use dioxus::prelude::*;
use futures_channel::mpsc::{UnboundedReceiver, UnboundedSender, unbounded};
use futures_util::StreamExt;
use tracing::{debug, warn};

use kokoro::domains::settings::lang;
use kokoro::{Lazy, Loader, Vault};

use crate::platform::{Fault, Phase, Platform};

const GAME: &str = "game";
const STARTING: &str = "Starting the data engine...";

struct Deferred {
    missing: Mutex<HashSet<Box<str>>>,
    nudge: UnboundedSender<()>,
}

impl Loader for Deferred {
    fn load(&self, name: &str) -> Option<Arc<[u8]>> {
        let fresh = self.missing.lock().unwrap_or_else(PoisonError::into_inner).insert(Box::from(name));

        if fresh {
            let _ = self.nudge.unbounded_send(());
        }

        None
    }
}

pub(crate) struct Core {
    vault: Vault,
    deferred: Arc<Deferred>,
    pending: RefCell<HashSet<Box<str>>>,
}

impl Core {
    pub(crate) fn vault(&self) -> &Vault {
        &self.vault
    }
}

#[derive(Clone, Copy)]
pub(crate) struct Library {
    core: Signal<Option<Rc<Core>>>,
    generation: Signal<u64>,
    fault: Signal<Option<Fault>>,
    busy: Signal<bool>,
    data: Signal<u64>,
    phase: Signal<Phase>,
}

const IMAGE_SUFFIXES: [&str; 4] = [".png", ".imgcut", ".mamodel", ".maanim"];

fn is_image(name: &str) -> bool {
    IMAGE_SUFFIXES.iter().any(|suffix| name.ends_with(suffix))
}

impl Library {
    pub(crate) fn provide(platform: Platform) -> Self {
        let library = use_context_provider(|| Self {
            core: Signal::new(None),
            generation: Signal::new(0),
            fault: Signal::new(None),
            busy: Signal::new(false),
            data: Signal::new(0),
            phase: Signal::new(Phase { label: Box::from(STARTING), done: 0, total: 0 }),
        });

        use_hook(move || {
            let (nudge, pulses) = unbounded();

            spawn(async move { library.open(platform, nudge, pulses).await })
        });

        library
    }

    async fn open(mut self, platform: Platform, nudge: UnboundedSender<()>, mut pulses: UnboundedReceiver<()>) {
        let phase = self.phase;
        let names = match platform
            .host()
            .names(Box::new(move |seen| {
                let mut phase = phase;

                phase.set(seen);
            }))
            .await
        {
            Ok(names) => names,
            Err(fault) => {
                warn!("The file host could not be reached: {}", fault);
                self.fault.set(Some(fault));
                return;
            }
        };
        let vault = Vault::with_priority(&lang::default_priority());
        let deferred = Arc::new(Deferred { missing: Mutex::new(HashSet::new()), nudge });

        if let Err(err) = vault.vfs.create(Lazy { key: GAME, names }) {
            warn!("The game mount could not be created: {}", err);
            self.fault.set(Some(Fault(err.to_string())));
            return;
        }

        vault.vfs.install(Arc::clone(&deferred) as Arc<dyn Loader>);

        let core = Rc::new(Core { vault, deferred, pending: RefCell::new(HashSet::new()) });

        self.core.set(Some(Rc::clone(&core)));
        self.generation += 1;
        self.data += 1;

        while pulses.next().await.is_some() {
            self.busy.set(true);
            self.flush(&platform, &core).await;

            let waiting = !core.deferred.missing.lock().unwrap_or_else(PoisonError::into_inner).is_empty() || !core.pending.borrow().is_empty();

            self.busy.set(waiting);
        }
    }

    async fn flush(&mut self, platform: &Platform, core: &Core) {
        let wanted: Vec<Box<str>> = {
            let mut missing = core.deferred.missing.lock().unwrap_or_else(PoisonError::into_inner);
            let mut pending = core.pending.borrow_mut();

            missing.drain().filter(|name| pending.insert(name.clone())).collect()
        };

        if wanted.is_empty() {
            return;
        }

        debug!(count = wanted.len(), "Fetching files the screens asked for");

        let outcome = platform.host().fetch(wanted.clone()).await;

        {
            let mut pending = core.pending.borrow_mut();

            for name in &wanted {
                pending.remove(name);
            }
        }

        match outcome {
            Ok(files) => {
                let names: Vec<Box<str>> = files.iter().map(|(name, _)| name.clone()).collect();

                core.vault.vfs.stock(GAME, files);

                let mut tables = false;

                for name in &names {
                    let evicted = core.vault.evict(name);

                    tables |= evicted.cats || evicted.enemies || evicted.stages || !is_image(name);
                }

                self.generation += 1;

                if tables {
                    self.data += 1;
                }
            }
            Err(fault) => {
                warn!("Files could not be fetched: {}", fault);
                self.fault.set(Some(fault));
            }
        }
    }

    pub(crate) fn relocalize(&mut self, order: &[String]) {
        if let Some(core) = self.core.read().as_ref() {
            core.vault.priority(order);
        }

        self.generation += 1;
        self.data += 1;
    }

    pub(crate) fn tables(&self) -> Option<Rc<Core>> {
        let _ = (self.data)();

        self.core.read().clone()
    }

    pub(crate) fn busy(&self) -> bool {
        (self.busy)()
    }

    pub(crate) fn waiting(&self) -> bool {
        self.core.peek().as_ref().is_some_and(|core| {
            !core.deferred.missing.lock().unwrap_or_else(PoisonError::into_inner).is_empty() || !core.pending.borrow().is_empty()
        })
    }

    pub(crate) fn fault(&self) -> Option<Fault> {
        (self.fault)()
    }

    pub(crate) fn phase(&self) -> Phase {
        (self.phase)()
    }

    pub(crate) fn detail(done: usize, total: usize) -> Option<String> {
        (total > 0).then(|| format!("{}% / 100%", done * 100 / total))
    }

    pub(crate) fn generation(&self) -> u64 {
        (self.generation)()
    }
}
