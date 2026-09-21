use std::cell::RefCell;
use std::rc::Rc;
use std::time::{Duration, Instant};

use emu::runtime::BattleOptions;
use kore::Vfs;
use tracing::warn;

use super::driver::Driver;
use super::assets::SheetCache;
use super::input::TouchQueue;
use super::sink::Frame;

const BLACK_FROM: i32 = 0xb;
const CLOSED_FRAME: i32 = 0xc;
const LAST_FRAME: i32 = 0x18;
const FRAME_TIME: Duration = Duration::from_millis(33);
const STEP_TIME: Duration = Duration::from_millis(30);

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
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
    phase: Phase,
    sweep: i32,
    started: Instant,
    entered: bool,
    frozen: usize,
    discard: bool,
    stepped: Instant,
    failure: Option<String>,
    stale: bool,
}

impl Session {
    pub fn new() -> Self {
        let driver = Driver::new();
        let frame = Rc::clone(driver.frame());

        Self {
            driver,
            frame,
            phase: Phase::Idle,
            sweep: 0,
            stale: false,
            started: Instant::now(),
            entered: false,
            frozen: 0,
            discard: false,
            stepped: Instant::now(),
            failure: None,
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

    pub fn design_width(&self) -> f32 {
        self.driver.design_width()
    }

    pub fn covered(&self) -> bool {
        match self.phase {
            Phase::Idle | Phase::Faulted => false,
            Phase::Running => self.driver.fading(),
            Phase::Closing => true,
            Phase::Covering | Phase::Loading | Phase::Leaving => {
                (BLACK_FROM..=CLOSED_FRAME).contains(&self.sweep)
            }
        }
    }

    pub fn running(&self) -> bool {
        self.phase != Phase::Idle
    }

    fn in_transition(&self) -> bool {
        matches!(
            self.phase,
            Phase::Covering | Phase::Loading | Phase::Closing | Phase::Leaving
        )
    }

    pub fn begin(&mut self) {
        if self.in_transition() {
            return;
        }

        self.phase = Phase::Covering;
        self.sweep = 0;
        self.started = Instant::now();
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

    pub fn equip(&mut self, setup: emu::runtime::Setup) {
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

        self.failure = None;
        self.entered = true;
        self.phase = Phase::Running;
    }

    pub fn terminate(&mut self) {
        if self.phase != Phase::Faulted {
            return;
        }

        self.phase = Phase::Closing;
        self.discard = true;
        self.sweep = 0;
        self.started = Instant::now();
    }

    pub fn failure(&self) -> Option<&str> {
        self.failure.as_deref()
    }

    pub fn resize(&mut self, width: f32, height: f32) {
        self.driver.resize(width, height);
    }

    pub fn tick(&mut self, vfs: &Vfs) {
        let elapsed = self.started.elapsed().div_duration_f32(FRAME_TIME) as i32;

        match self.phase {
            Phase::Idle | Phase::Faulted => (),
            Phase::Closing => {
                self.sweep = elapsed.min(CLOSED_FRAME);
                self.frame.borrow_mut().quads.truncate(self.frozen);
                self.driver.draw_curtain_over(self.sweep);

                if self.sweep >= CLOSED_FRAME {
                    self.phase = Phase::Leaving;
                    self.started = Instant::now();
                }
            }
            Phase::Running => {
                if self.stepped.elapsed() >= STEP_TIME {
                    self.step();
                }
            }
            Phase::Covering => {
                self.sweep = elapsed.min(CLOSED_FRAME);

                if self.sweep >= CLOSED_FRAME {
                    self.phase = Phase::Loading;
                }

                self.driver.draw_curtain(self.sweep);
            }
            Phase::Loading => {
                self.driver.reindex(vfs);
                self.load();

                if self.entered {
                    self.phase = Phase::Running;
                    self.stepped = Instant::now();
                } else {
                    self.phase = Phase::Leaving;
                    self.started = Instant::now();
                    self.driver.draw_curtain(self.sweep);
                }
            }
            Phase::Leaving => {
                self.sweep = CLOSED_FRAME + 1 + elapsed;

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

                    return;
                }

                self.driver.draw_curtain(self.sweep);
            }
        }
    }

    fn step(&mut self) {
        self.stepped = Instant::now();

        if let Err(reason) = self.driver.advance() {
            self.failure = Some(reason);
            self.phase = Phase::Faulted;
            self.entered = false;
            self.driver.silence();
            self.driver.dim();
            self.frozen = self.frame.borrow().quads.len();

            return;
        }

        if self.entered && !self.driver.in_battle() {
            self.entered = false;
            self.driver.silence();
            self.phase = Phase::Leaving;
            self.sweep = CLOSED_FRAME;
            self.started = Instant::now();
            self.driver.draw_curtain(self.sweep);
        }
    }

    fn load(&mut self) {
        if !self.driver.boot() {
            self.failure = Some(format!(
                "game data did not load ({} files visible to the emulator)",
                self.driver.resolved()
            ));

            return;
        }

        if !self.driver.enter_battle() {
            self.failure = Some("battle entry refused - see the log".to_owned());

            return;
        }

        self.entered = true;
    }
}

impl Drop for Session {
    fn drop(&mut self) {
        if self.in_transition() {
            warn!("emu: session dropped mid-transition");
        }
    }
}
