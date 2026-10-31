use std::cell::RefCell;
use std::rc::Rc;

use serde::{Deserialize, Serialize};
use tracing::{trace, warn};

use emulator::runtime::{BattleOptions, Setup};
use kokoro::domains::sandbox::replay::Footage;

use super::assets::SheetCache;
use super::diagnostics::Diagnostics;
use super::driver::{Clock, Driver, Equipment, Label};
use super::input::TouchQueue;
use super::keys::Action;
use super::sink::Frame;

const BLACK_FROM: i32 = 0xb;
pub const CLOSED_FRAME: i32 = 0xc;
const LAST_FRAME: i32 = 0x18;
const FRAME_MS: f64 = 33.0;
const STEP_MS: f64 = 30.0;
const PRESENT_PATIENCE_MS: f64 = 2000.0;
const SLOW_STEP_MS: f64 = 100.0;

#[derive(Clone, Copy, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub enum Phase {
    Idle,
    Covering,
    Loading,
    Running,
    Faulted,
    Closing,
    Leaving,
}

pub struct Session {
    driver: Driver,
    frame: Rc<RefCell<Frame>>,
    clock: Rc<dyn Clock>,
    phase: Phase,
    sweep: i32,
    swept: f64,
    started: f64,
    entered: bool,
    frozen: usize,
    discard: bool,
    stepped: f64,
    failure: Option<String>,
    stale: bool,
    again: bool,
    presented: bool,
    warm: bool,
    reel: Option<Footage>,
    recording: bool,
    cut: bool,
}

impl Session {
    pub fn new(equipment: Equipment) -> Self {
        let clock = Rc::clone(&equipment.clock);
        let driver = Driver::new(equipment);
        let frame = Rc::clone(driver.frame());
        let now = clock.now_ms();

        Self {
            driver,
            frame,
            clock,
            phase: Phase::Idle,
            sweep: 0,
            swept: now,
            started: now,
            entered: false,
            frozen: 0,
            discard: false,
            stepped: now,
            failure: None,
            stale: false,
            again: false,
            presented: false,
            warm: true,
            reel: None,
            recording: true,
            cut: false,
        }
    }

    pub fn frame(&self) -> &Rc<RefCell<Frame>> {
        &self.frame
    }

    pub fn sheets(&self) -> &Rc<RefCell<SheetCache>> {
        self.driver.sheets()
    }

    pub fn touches(&self) -> &TouchQueue {
        self.driver.touches()
    }

    pub fn phase(&self) -> Phase {
        self.phase
    }

    pub fn surfaced(&self) -> bool {
        matches!(self.phase, Phase::Loading | Phase::Running | Phase::Faulted | Phase::Closing)
    }

    pub fn covered(&self) -> bool {
        match self.phase {
            Phase::Idle | Phase::Faulted => false,
            Phase::Running => self.driver.fading(),
            Phase::Closing => true,
            Phase::Covering | Phase::Loading | Phase::Leaving => (BLACK_FROM..=CLOSED_FRAME).contains(&self.sweep),
        }
    }

    pub fn running(&self) -> bool {
        self.phase != Phase::Idle
    }

    fn in_transition(&self) -> bool {
        matches!(self.phase, Phase::Covering | Phase::Loading | Phase::Closing | Phase::Leaving)
    }

    pub fn set_recording(&mut self, recording: bool) {
        self.recording = recording;
    }

    pub fn label(&mut self, label: Label) {
        self.driver.label(label);
    }

    pub fn watching(&self) -> bool {
        self.driver.watching()
    }

    pub fn watch(&mut self, footage: Footage) {
        if self.in_transition() {
            return;
        }

        self.begin();
        self.reel = Some(footage);
    }

    pub fn take_cut(&mut self) -> bool {
        std::mem::take(&mut self.cut)
    }

    pub fn refit(&mut self) {
        if self.phase != Phase::Idle {
            self.stale = true;

            return;
        }

        self.driver.renew();
        self.driver.forget_assets();
    }

    pub fn take_footage(&mut self) -> Option<Footage> {
        self.driver.take_footage()
    }

    pub fn begin(&mut self) {
        if self.in_transition() {
            return;
        }

        self.reel = None;

        self.phase = Phase::Covering;
        self.sweep = 0;
        self.started = self.clock.now_ms();
        self.swept = self.started;
        self.entered = false;
        self.failure = None;
    }

    pub fn forget_assets(&mut self) {
        if self.phase == Phase::Idle {
            self.driver.forget_assets();
        } else {
            self.stale = true;
        }
    }

    pub fn relocalize(&mut self, priority: Vec<String>) {
        self.driver.relocalize(priority);
        self.forget_assets();
    }

    pub fn equip(&mut self, setup: Setup) {
        self.driver.set_setup(setup);
    }

    pub fn configure(&mut self, options: BattleOptions) {
        self.driver.set_options(options);
    }

    pub fn take_options(&mut self) -> Option<BattleOptions> {
        self.driver.take_options()
    }

    pub fn set_phone(&mut self, phone: bool) {
        self.driver.set_phone(phone);
    }

    pub fn retune(&mut self, music: i32, effects: i32) {
        self.driver.set_volumes(music, effects);
    }

    pub fn faulted(&self) -> bool {
        self.phase == Phase::Faulted
    }

    pub fn resume(&mut self) {
        if self.phase != Phase::Faulted {
            return;
        }

        self.driver.forgive();
        self.failure = None;
        self.entered = true;
        self.phase = Phase::Running;
    }

    pub fn terminate(&mut self) {
        if !matches!(self.phase, Phase::Faulted | Phase::Running) {
            return;
        }

        if self.phase == Phase::Running {
            self.entered = false;
            self.driver.silence();
            self.frozen = self.frame.borrow().quads.len();
        }

        self.phase = Phase::Closing;
        self.discard = true;
        self.sweep = 0;
        self.started = self.clock.now_ms();
        self.swept = self.started;
    }

    pub fn diagnose(&self) -> Option<Diagnostics> {
        matches!(self.phase, Phase::Running | Phase::Faulted).then(|| self.driver.diagnose())
    }

    pub fn failure(&self) -> Option<&str> {
        self.failure.as_deref()
    }

    pub fn resize(&mut self, width: f32, height: f32) {
        self.driver.resize(width, height);
    }

    pub fn presented(&mut self) {
        self.presented = true;
    }

    pub fn awaiting_warmth(&self) -> bool {
        self.phase == Phase::Loading && !self.warm
    }

    pub fn warmed(&mut self) {
        self.warm = true;
    }

    pub fn expect_warmth(&mut self) {
        self.warm = false;
    }

    pub fn tick(&mut self) -> bool {
        let now = self.clock.now_ms();

        match self.phase {
            Phase::Idle | Phase::Faulted => false,
            Phase::Closing => {
                self.paced(now);
                self.sweep = self.sweep.min(CLOSED_FRAME);
                self.frame.borrow_mut().quads.truncate(self.frozen);
                self.driver.draw_curtain_over(self.sweep);

                if self.sweep >= CLOSED_FRAME {
                    self.phase = if self.again { Phase::Loading } else { Phase::Leaving };

                    if self.driver.watching() {
                        self.driver.end_playback();
                        self.driver.draw_curtain(self.sweep);
                    }

                    self.started = now;
                    self.swept = now;

                    if self.phase == Phase::Loading {
                        self.presented = false;
                    } else {
                        self.sweep = CLOSED_FRAME + 1;
                    }
                }

                true
            }
            Phase::Running => {
                if now - self.stepped >= STEP_MS {
                    self.step(now);

                    return true;
                }

                false
            }
            Phase::Covering => {
                self.paced(now);
                self.sweep = self.sweep.min(CLOSED_FRAME);

                if self.sweep >= CLOSED_FRAME {
                    self.phase = Phase::Loading;
                }

                self.driver.draw_curtain(self.sweep);

                if self.phase == Phase::Loading {
                    self.started = now;
                    self.presented = false;
                }

                true
            }
            Phase::Loading => {
                let waited = now - self.started;

                if !self.presented {
                    if waited < PRESENT_PATIENCE_MS {
                        return false;
                    }

                    warn!(ms = waited as u32, "emu: the closed curtain never reached the screen; loading the battle anyway");
                }

                if !self.warm {
                    return false;
                }

                trace!(waited_ms = waited as u32, "emu: loading the battle behind the closed curtain");

                if std::mem::take(&mut self.again) {
                    self.driver.renew();
                }

                let mark = self.clock.now_ms();

                match self.reel.take() {
                    Some(footage) => match footage.cues() {
                        Ok(frames) => {
                            self.driver.arm_playback(&footage.save, frames, footage.faults());
                            self.driver.draw_curtain(self.sweep);
                            self.load();
                        }
                        Err(reason) => {
                            warn!("emu: the replay could not be loaded: {reason}");
                            self.failure = Some(reason);
                        }
                    },
                    None => {
                        if self.recording {
                            self.driver.arm_recording();
                        }

                        self.load();
                        self.driver.stamp_deck();
                        self.driver.keep_assets();
                    }
                }

                trace!(entered = self.entered, ms = (self.clock.now_ms() - mark) as u32, failure = ?self.failure, "emu: battle load finished");

                if self.entered {
                    self.phase = Phase::Running;
                    self.stepped = self.clock.now_ms();
                } else {
                    self.phase = Phase::Leaving;
                    self.started = self.clock.now_ms();
                    self.swept = self.started;
                    self.sweep = CLOSED_FRAME + 1;
                    self.driver.end_playback();
                    self.driver.draw_curtain(self.sweep);
                }

                true
            }
            Phase::Leaving => {
                self.paced(now);

                if self.sweep > LAST_FRAME {
                    self.phase = Phase::Idle;
                    self.driver.release_curtain();
                    self.frame.borrow_mut().clear();

                    if self.discard {
                        self.discard = false;
                        self.failure = None;
                    }

                    self.driver.renew();

                    if std::mem::take(&mut self.stale) {
                        self.driver.forget_assets();
                    }

                    return true;
                }

                self.driver.draw_curtain(self.sweep);

                true
            }
        }
    }

    pub fn key(&mut self, action: Action, pressed: bool) {
        if self.phase == Phase::Running {
            self.driver.key(action, pressed);
        }
    }

    pub fn restart(&mut self) {
        if self.phase != Phase::Running || self.driver.watching() {
            return;
        }

        self.again = true;
        self.entered = false;
        self.driver.silence();
        self.frozen = self.frame.borrow().quads.len();
        self.phase = Phase::Closing;
        self.sweep = 0;
        self.started = self.clock.now_ms();
        self.swept = self.started;
    }

    fn paced(&mut self, now: f64) {
        if now - self.swept >= FRAME_MS {
            self.sweep += 1;
            self.swept = now;
        }
    }

    fn step(&mut self, now: f64) {
        self.stepped = now;

        let advanced = self.driver.advance();
        let spent = self.clock.now_ms() - now;

        if spent >= SLOW_STEP_MS {
            trace!(ms = spent as u32, in_battle = self.driver.in_battle(), fading = self.driver.fading(), "emu: slow battle frame");
        }

        if let Err(reason) = advanced {
            if self.driver.watching() {
                if self.driver.forgive_scripted() {
                    return;
                }

                warn!("emu: the replay was terminated by a fault: {reason}");
                self.cut = true;
                self.terminate();
                self.discard = false;

                return;
            }

            self.failure = Some(reason);
            self.phase = Phase::Faulted;
            self.entered = false;
            self.driver.silence();
            self.driver.dim();
            self.frozen = self.frame.borrow().quads.len();

            return;
        }

        if self.driver.exit_requested() || self.driver.reel_finished() {
            self.cut |= self.driver.reel_finished() && self.driver.cut();
            self.terminate();

            return;
        }

        if self.entered && !self.driver.in_battle() {
            self.entered = false;
            self.driver.silence();
            self.phase = Phase::Leaving;
            self.sweep = CLOSED_FRAME;
            self.driver.end_playback();
            self.started = self.clock.now_ms();
            self.driver.draw_curtain(self.sweep);
        }
    }

    fn load(&mut self) {
        if !self.driver.boot() {
            self.failure = Some(format!("game data did not load\n{} files visible to the emulator", self.driver.resolved()));

            return;
        }

        if !self.driver.enter_battle() {
            self.failure = Some("battle entry refused - see the log".to_owned());

            return;
        }

        self.entered = true;
    }
}
