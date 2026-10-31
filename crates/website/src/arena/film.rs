use std::cell::{Cell, RefCell};
use std::collections::HashSet;
use std::rc::Rc;
use std::sync::Arc;

use tracing::{info, warn};
use wasm_bindgen_futures::spawn_local;

use kokoro::domains::settings::ReplaySource;
use omote::host::{Backend, Clip, Equipment, Film, Playing};
use omote::platform::battle::{AudioCommand, BattleEvent, Rolled, SheetWire};

use crate::worker::{self, Engine};

use super::{FONTS, WorkerClock, WorkerFiles, latest, layering, post};

const GAME_FPS: f64 = 30.0;
const GONE: &str = "the replay video is no longer running";

#[derive(Default)]
struct FilmAudio {
    commands: RefCell<Vec<(u32, AudioCommand)>>,
    loaded: RefCell<HashSet<Box<str>>>,
    next: Cell<u32>,
    frame: Cell<u32>,
}

impl FilmAudio {
    fn push(&self, command: AudioCommand) {
        self.commands.borrow_mut().push((self.frame.get(), command));
    }
}

struct FilmVoice {
    id: u32,
    audio: Rc<FilmAudio>,
    ends: Option<u32>,
    stopped: Cell<bool>,
}

impl Playing for FilmVoice {
    fn set_level(&self, level: f32) {
        self.audio.push(AudioCommand::Level { voice: self.id, level });
    }

    fn stop(&self) {
        if !self.stopped.replace(true) {
            self.audio.push(AudioCommand::Stop { voice: self.id });
        }
    }

    fn busy(&self) -> bool {
        !self.stopped.get() && self.ends.is_none_or(|end| self.audio.frame.get() < end)
    }
}

struct FilmLink(Rc<FilmAudio>);

impl Backend for FilmLink {
    fn start(&self, clip: &Clip, looped: bool, level: f32) -> Option<Box<dyn Playing>> {
        let audio = &self.0;
        let (name, ends) = match clip {
            Clip::Effect { name, effect } => {
                if audio.loaded.borrow_mut().insert(Box::from(name.as_str())) {
                    audio.push(AudioCommand::Effect { clip: Box::from(name.as_str()), channels: effect.channels, rate: effect.rate, samples: effect.samples.to_vec() });
                }

                let frames = effect.samples.len() as f64 / f64::from(effect.channels.max(1));
                let seconds = frames / f64::from(effect.rate.max(1));

                (name.as_str(), (!looped).then(|| audio.frame.get().saturating_add((seconds * GAME_FPS).ceil() as u32)))
            }
            Clip::Track { name, bytes } => {
                if audio.loaded.borrow_mut().insert(Box::from(name.as_str())) {
                    audio.push(AudioCommand::Track { clip: Box::from(name.as_str()), bytes: bytes.to_vec() });
                }

                (name.as_str(), None)
            }
        };
        let id = audio.next.get().wrapping_add(1);

        audio.next.set(id);
        audio.push(AudioCommand::Start { voice: id, clip: Box::from(name), looped, level });

        Some(Box::new(FilmVoice { id, audio: Rc::clone(audio), ends, stopped: Cell::new(false) }))
    }
}

struct Reeling {
    film: Film,
    audio: Rc<FilmAudio>,
    sent: HashSet<u64>,
    drawn: u32,
}

thread_local! {
    static REELING: RefCell<Option<Reeling>> = const { RefCell::new(None) };
}

pub(super) fn start(engine: &Rc<Engine>, source: ReplaySource, priority: Vec<String>) {
    unroll();

    let engine = Rc::clone(engine);

    spawn_local(async move {
        if let Err(reason) = open(&engine, source, priority).await {
            warn!("A replay video could not be started: {reason}");
            post(BattleEvent::Rolled(Box::new(Rolled { failure: Some(reason), over: true, ..Rolled::default() })));
        }
    });
}

async fn open(engine: &Rc<Engine>, source: ReplaySource, priority: Vec<String>) -> Result<(), String> {
    let kept = latest(engine).await.ok_or("there is no latest battle")?;
    let reel = kept.reel();
    let (overlay, frames) = layering(engine, &reel.footage, reel.files, reel.pinned, source);

    worker::warm_frames(engine, frames, false, |_, _| {}).await;

    let audio = Rc::new(FilmAudio::default());
    let equipment = Equipment {
        files: Rc::new(WorkerFiles { loader: Arc::clone(&engine.loader), overlay: Rc::new(RefCell::new(overlay)) }),
        backend: Rc::new(FilmLink(Rc::clone(&audio))),
        clock: Rc::new(WorkerClock),
        fonts: FONTS.with(|fonts| fonts.borrow().clone()),
        priority,
    };
    let film = Film::new(equipment, &reel.footage)?;
    let (played, total) = film.progress();
    let size = film.size();

    info!(width = size.0, height = size.1, frames = total, "A replay video was started");
    REELING.with(|slot| *slot.borrow_mut() = Some(Reeling { film, audio, sent: HashSet::new(), drawn: 0 }));
    post(BattleEvent::Rolled(Box::new(Rolled { size, played, total, ..Rolled::default() })));

    Ok(())
}

pub(super) fn roll(count: u32) {
    let rolled = REELING.with(|slot| {
        let mut slot = slot.borrow_mut();
        let reeling = slot.as_mut()?;
        let mut frames = Vec::new();
        let mut over = false;

        for _ in 0..count {
            reeling.audio.frame.set(reeling.drawn);

            if !reeling.film.roll() {
                over = true;
                break;
            }

            frames.push(reeling.film.frame().borrow().clone());
            reeling.drawn += 1;
        }

        let sheets: Vec<SheetWire> = {
            let held = reeling.film.sheets().borrow();
            let fresh = held
                .iter()
                .filter(|(_, sheet)| !reeling.sent.contains(&sheet.stamp))
                .map(|(name, sheet)| SheetWire { name: name.clone(), width: sheet.width, height: sheet.height, pixels: sheet.pixels.to_vec() })
                .collect();

            reeling.sent.extend(held.values().map(|sheet| sheet.stamp));
            fresh
        };
        let audio = std::mem::take(&mut *reeling.audio.commands.borrow_mut());
        let (played, total) = reeling.film.progress();
        let rolled = Rolled { size: reeling.film.size(), frames, sheets, audio, played, total, over, failure: None };

        if over {
            *slot = None;
        }

        Some(rolled)
    });
    let rolled = rolled.unwrap_or_else(|| Rolled { failure: Some(GONE.to_owned()), over: true, ..Rolled::default() });

    post(BattleEvent::Rolled(Box::new(rolled)));
}

pub(super) fn unroll() {
    REELING.with(|slot| *slot.borrow_mut() = None);
}
