use std::collections::VecDeque;

use emu::engine::AppContext;
use emu::runtime::{self, Spot};
use tracing::warn;

const TAP_FRAMES: u8 = 2;
const SETTLE_FRAMES: u8 = 10;
const GATE_FRAMES: u16 = 90;
const SWAP_SETTLE: u8 = 4;
const LEAVE_AFTER: u16 = 15;
const PAN_STEP: i32 = 0x18;
const PAN_REACH: i32 = 1 << 20;
const ZOOM_STEP: i32 = 10;
const STILL_FRAMES: u8 = 2;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Action {
    Slot(i32),
    Item(i32),
    Worker,
    Cannon,
    ZoomIn,
    ZoomOut,
    PanLeft,
    PanRight,
    Pause,
}

#[derive(Clone, Copy)]
enum Step {
    Press(Spot),
    Hold(u8),
    Release,
    Back,
    Swap(i32),
    AwaitRow(i32, u16),
    AwaitMenu(u16),
    AwaitDialog(u16),
}

#[derive(Default)]
pub struct Keys {
    plan: VecDeque<Step>,
    owner: Option<Action>,
    down: bool,
    back: bool,
    paused: Option<u16>,
    touch_down: bool,
    drop_pinch: bool,
    pan: i32,
    pan_key: Option<Action>,
    zoom: i32,
    zoom_key: Option<Action>,
    drag: Option<i32>,
    still: u8,
}

impl Keys {
    pub fn busy(&self) -> bool {
        self.down || self.drag.is_some() || !self.plan.is_empty()
    }

    pub fn reset(&mut self) {
        *self = Self::default();
    }

    pub fn press(&mut self, action: Action) {
        match action {
            Action::ZoomIn => {
                self.zoom = 1;
                self.zoom_key = Some(action);
            }
            Action::ZoomOut => {
                self.zoom = -1;
                self.zoom_key = Some(action);
            }
            Action::PanLeft => {
                self.release_touch();
                self.pan = 1;
                self.pan_key = Some(action);
            }
            Action::PanRight => {
                self.release_touch();
                self.pan = -1;
                self.pan_key = Some(action);
            }
            Action::Pause => self.paused = self.paused.or(Some(0)),
            Action::Slot(slot) => self.tap(action, Spot::Deck(slot), Some(slot)),
            Action::Item(item) => self.tap(action, Spot::Item(item), None),
            Action::Worker => self.tap(action, Spot::Worker, None),
            Action::Cannon => self.tap(action, Spot::Cannon, None),
        }
    }

    pub fn release(&mut self, action: Action) {
        match action {
            Action::ZoomIn | Action::ZoomOut if self.zoom_key == Some(action) => {
                self.zoom = 0;
                self.zoom_key = None;
                self.drop_pinch = true;
            }
            Action::PanLeft | Action::PanRight if self.pan_key == Some(action) => {
                self.pan = 0;
                self.pan_key = None;
            }
            Action::ZoomIn | Action::ZoomOut | Action::PanLeft | Action::PanRight => (),
            Action::Pause => {
                let tapped = self.paused.take().is_some_and(|held| held < LEAVE_AFTER);

                if tapped {
                    self.plan.push_back(Step::Back);
                }
            }
            _ if self.owner == Some(action) => {
                self.owner = None;
                self.lift();
            }
            _ => (),
        }
    }

    fn lift(&mut self) {
        if self.down {
            self.plan.push_back(Step::Release);
            self.down = false;
        }
    }

    fn release_touch(&mut self) {
        if self.down {
            self.cancel();
        }
    }

    fn cancel(&mut self) {
        self.plan.clear();
        self.still = 0;
        self.drag = None;
        self.down = false;
        self.owner = None;

        if self.touch_down {
            self.plan.push_back(Step::Release);
        }
    }

    fn tap(&mut self, action: Action, spot: Spot, slot: Option<i32>) {
        if self.owner == Some(action) {
            return;
        }

        self.cancel();

        if let Some(slot) = slot {
            self.plan.extend([Step::Swap(slot), Step::AwaitRow(slot, GATE_FRAMES)]);
        }

        self.plan.extend([Step::Press(spot), Step::Hold(TAP_FRAMES)]);
        self.owner = Some(action);
        self.down = true;
    }

    fn leave(&mut self) {
        self.lift();
        self.owner = None;
        self.plan.extend([
            Step::Back,
            Step::AwaitMenu(GATE_FRAMES),
            Step::Hold(SETTLE_FRAMES),
            Step::Press(Spot::Button(runtime::LEAVE_BUTTON)),
            Step::Hold(TAP_FRAMES),
            Step::Release,
            Step::AwaitDialog(GATE_FRAMES),
            Step::Hold(SETTLE_FRAMES),
            Step::Press(Spot::DialogButton(runtime::CONFIRM_BUTTON)),
            Step::Hold(TAP_FRAMES),
            Step::Release,
        ]);
    }

    pub fn pump(&mut self, ctx: &mut AppContext, spread: &mut i32, gap: &mut Option<i32>) {
        if let Err(fault) = self.advance(ctx, spread, gap) {
            warn!("emu: a key could not be played: {fault}");
            self.reset();
        }
    }

    fn advance(&mut self, ctx: &mut AppContext, spread: &mut i32, gap: &mut Option<i32>) -> Result<(), emu::Fault> {
        if std::mem::take(&mut self.back) {
            runtime::queue_back(ctx, false)?;
        }

        if let Some(held) = self.paused.as_mut() {
            *held = held.saturating_add(1);

            if *held == LEAVE_AFTER {
                self.leave();
            }
        }

        *spread = spread.wrapping_add(self.zoom * ZOOM_STEP);

        if std::mem::take(&mut self.drop_pinch) {
            *spread = 0;
            *gap = None;
        }

        let Some(step) = self.plan.front().copied() else {
            return self.pan_field(ctx, spread, gap);
        };

        let done = match step {
            Step::Press(spot) => {
                if gap.is_some() {
                    *spread = 0;
                    *gap = None;
                }

                if runtime::pinch_latched(ctx)? {
                    false
                } else {
                    if let Some((x, y)) = runtime::spot_center(ctx, spot)? {
                        runtime::queue_spot_press(ctx, x, y)?;
                        self.touch_down = true;
                    }

                    true
                }
            }
            Step::Hold(frames) => {
                if frames > 1 {
                    self.plan[0] = Step::Hold(frames - 1);
                }

                frames <= 1
            }
            Step::Release => {
                runtime::queue_touch_release(ctx)?;
                self.touch_down = false;

                true
            }
            Step::Back => {
                if !runtime::option_menu_open(ctx)? || self.plan.len() == 1 {
                    runtime::queue_back(ctx, true)?;
                    self.back = true;
                }

                true
            }
            Step::Swap(slot) => {
                if runtime::deck_row_hidden(ctx, slot)? {
                    runtime::start_deck_row_swap(ctx)?;
                }

                true
            }
            Step::AwaitRow(slot, left) => {
                let shown = !runtime::deck_row_hidden(ctx, slot)? && !runtime::deck_row_swapping(ctx)?;

                if shown && left < GATE_FRAMES {
                    self.plan[0] = Step::Hold(SWAP_SETTLE);

                    false
                } else {
                    self.gate(left, shown, |left| Step::AwaitRow(slot, left))
                }
            }
            Step::AwaitMenu(left) => self.gate(left, runtime::option_menu_open(ctx)?, Step::AwaitMenu),
            Step::AwaitDialog(left) => self.gate(left, runtime::dialog_open(ctx), Step::AwaitDialog),
        };

        if done && !self.plan.is_empty() {
            self.plan.pop_front();
        }

        Ok(())
    }

    fn gate(&mut self, left: u16, open: bool, again: impl Fn(u16) -> Step) -> bool {
        if open {
            return true;
        }

        if left == 0 {
            self.plan.clear();
            self.owner = None;
            self.down = false;

            return false;
        }

        self.plan[0] = again(left - 1);

        false
    }

    fn pan_field(&mut self, ctx: &mut AppContext, spread: &mut i32, gap: &mut Option<i32>) -> Result<(), emu::Fault> {
        let Some((center, y)) = runtime::spot_center(ctx, Spot::Field)? else {
            return Ok(());
        };

        if self.still > 0 {
            self.still -= 1;

            if self.still == 0 {
                self.drag = None;
                self.touch_down = false;

                return runtime::queue_touch_release(ctx);
            }

            return Ok(());
        }

        match (self.pan, self.drag) {
            (0, None) => Ok(()),
            (0, Some(_)) => {
                self.still = STILL_FRAMES;

                Ok(())
            }
            (_, None) if self.down => Ok(()),
            (_, None) => {
                if gap.is_some() {
                    *spread = 0;
                    *gap = None;
                }

                if runtime::pinch_latched(ctx)? {
                    return Ok(());
                }

                self.drag = Some(center);
                self.touch_down = true;

                runtime::queue_spot_press(ctx, center, y)
            }
            (direction, Some(at)) => {
                let next = at.wrapping_add(direction * PAN_STEP);

                if next.abs() > PAN_REACH {
                    self.drag = Some(center);
                    self.touch_down = true;

                    return runtime::queue_spot_press(ctx, center, y);
                }

                self.drag = Some(next);

                runtime::queue_spot_move(ctx, next, y)
            }
        }
    }
}
