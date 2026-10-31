use std::cell::RefCell;
use std::rc::Rc;

use tracing::{trace, warn};

use kokoro::domains::sandbox::replay::Footage;

use super::assets::SheetCache;
use super::driver::{Driver, Equipment};
use super::session::CLOSED_FRAME;
use super::sink::Frame;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Reel {
    Opening,
    Rolling,
    Closing { frozen: usize, sweep: i32 },
    Over,
}

pub struct Film {
    driver: Driver,
    reel: Reel,
    played: usize,
    total: usize,
    size: (u32, u32),
}

fn even(size: f32) -> u32 {
    ((size.max(2.0) as u32) / 2 * 2).max(2)
}

impl Film {
    pub fn new(equipment: Equipment, footage: &Footage) -> Result<Self, String> {
        let frames = footage.cues()?;
        let total = frames.len();
        let size = (even(footage.save.screen.width), even(footage.save.screen.height));
        let mut driver = Driver::new(equipment);

        driver.arm_playback(&footage.save, frames, footage.faults());

        if !driver.boot() {
            return Err("the replay's game data could not be loaded".to_owned());
        }

        if !driver.enter_battle() {
            return Err("the replay's battle could not be entered".to_owned());
        }

        driver.draw_curtain_aside(CLOSED_FRAME);

        Ok(Self { driver, reel: Reel::Opening, played: 0, total, size })
    }

    pub fn size(&self) -> (u32, u32) {
        self.size
    }

    pub fn progress(&self) -> (usize, usize) {
        (self.played.min(self.total), self.total)
    }

    pub fn frame(&self) -> &Rc<RefCell<Frame>> {
        self.driver.frame()
    }

    pub fn sheets(&self) -> &Rc<RefCell<SheetCache>> {
        self.driver.sheets()
    }

    pub fn roll(&mut self) -> bool {
        match self.reel {
            Reel::Opening => {
                self.reel = Reel::Rolling;

                true
            }
            Reel::Rolling => self.advance(),
            Reel::Closing { frozen, sweep } => self.close(frozen, sweep),
            Reel::Over => false,
        }
    }

    fn advance(&mut self) -> bool {
        let advanced = self.driver.advance();

        self.played += 1;

        if let Err(reason) = advanced {
            if self.driver.forgive_scripted() {
                trace!("Replay video continues past a recorded fault on frame {}: {reason}", self.played);
            } else {
                warn!("Replay video closes at a fault on frame {}: {reason}", self.played);

                return self.close(self.drawn(), 1);
            }
        }

        if !self.driver.in_battle() {
            self.driver.draw_curtain_aside(CLOSED_FRAME);
            self.reel = Reel::Over;

            return true;
        }

        if self.driver.reel_finished() || self.driver.exit_requested() {
            return self.close(self.drawn(), 1);
        }

        true
    }

    fn drawn(&self) -> usize {
        self.driver.frame().borrow().quads.len()
    }

    fn close(&mut self, frozen: usize, sweep: i32) -> bool {
        if sweep > CLOSED_FRAME {
            self.reel = Reel::Over;

            return false;
        }

        self.driver.frame().borrow_mut().quads.truncate(frozen);
        self.driver.draw_curtain_over(sweep);
        self.reel = Reel::Closing { frozen, sweep: sweep + 1 };

        true
    }
}
