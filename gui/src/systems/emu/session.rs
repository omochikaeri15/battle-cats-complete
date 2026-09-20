use std::cell::RefCell;
use std::rc::Rc;
use std::time::{Duration, Instant};

use kore::Vfs;
use tracing::warn;

use super::driver::Driver;
use super::assets::SheetCache;
use super::sink::Frame;

const BLACK_FROM: i32 = 0xb;
const CLOSED_FRAME: i32 = 0xc;
const LAST_FRAME: i32 = 0x18;
const FRAME_TIME: Duration = Duration::from_millis(33);

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Phase {
    Idle,
    Covering,
    Loading,
    Revealing,
    Running,
}

pub struct Session {
    driver: Driver,
    frame: Rc<RefCell<Frame>>,
    phase: Phase,
    sweep: i32,
    started: Instant,
    entered: bool,
    failure: Option<String>,
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
            started: Instant::now(),
            entered: false,
            failure: None,
        }
    }

    pub fn frame(&self) -> &Rc<RefCell<Frame>> {
        &self.frame
    }

    pub fn sheets(&self) -> &Rc<RefCell<SheetCache>> {
        self.driver.sheets()
    }

    pub fn design_height(&self) -> f32 {
        self.driver.design_height()
    }

    pub fn letterbox_shift(&self) -> f32 {
        self.driver.letterbox_shift()
    }

    pub fn covered(&self) -> bool {
        (BLACK_FROM..=CLOSED_FRAME).contains(&self.sweep) && self.phase != Phase::Running
    }

    pub fn running(&self) -> bool {
        self.phase != Phase::Idle
    }

    fn in_transition(&self) -> bool {
        matches!(
            self.phase,
            Phase::Covering | Phase::Loading | Phase::Revealing
        )
    }

    pub fn begin(&mut self) {
        if self.in_transition() {
            return;
        }

        self.driver.arm_curtain();
        self.phase = Phase::Covering;
        self.sweep = 0;
        self.started = Instant::now();
        self.entered = false;
        self.failure = None;
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
            Phase::Idle => (),
            Phase::Running => self.step(),
            Phase::Covering => {
                self.sweep = elapsed.min(CLOSED_FRAME);

                if self.sweep >= CLOSED_FRAME {
                    self.phase = Phase::Loading;
                }

                self.driver.draw_curtain(self.sweep);
            }
            Phase::Loading => {
                self.driver.index_assets(vfs);
                self.load();
                self.phase = Phase::Revealing;
                self.sweep = CLOSED_FRAME;
                self.started = Instant::now();
                self.driver.draw_curtain(self.sweep);
            }
            Phase::Revealing => {
                self.sweep = CLOSED_FRAME + elapsed;

                if self.sweep > LAST_FRAME {
                    if self.entered {
                        self.phase = Phase::Running;
                    } else {
                        self.phase = Phase::Idle;
                        self.frame.borrow_mut().clear();
                    }

                    return;
                }

                if self.entered {
                    self.step();
                    self.driver.draw_curtain_over(self.sweep);
                } else {
                    self.driver.draw_curtain(self.sweep);
                }
            }
        }
    }

    fn step(&mut self) {
        match self.driver.advance() {
            Ok(true) => (),
            Ok(false) => {
                self.failure = Some("the battle ended".to_owned());
                self.phase = Phase::Idle;
                self.frame.borrow_mut().clear();
            }
            Err(reason) => {
                self.failure = Some(reason);
                self.phase = Phase::Idle;
                self.frame.borrow_mut().clear();
            }
        }
    }

    fn load(&mut self) {
        if self.entered {
            return;
        }

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
