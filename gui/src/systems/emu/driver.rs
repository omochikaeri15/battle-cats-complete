use std::cell::{Cell, RefCell};
use std::collections::{HashSet, VecDeque};
use std::mem;
use std::path::PathBuf;
use std::rc::Rc;

use emu::engine::AppContext;
use emu::Site;
use emu::runtime::{
    BattleOptions, DECK_SLOTS, DeviceProfile, InertMeta, InertPlatform, InertScene, InertUi, Seeds, Setup, apply_battle_options,
    VERSION, fill_dummy_save, fill_dummy_talents, load_scene_sheets, plant_seeds, queue_touch_position, queue_touch_press, queue_touch_release,
    pump_stage_return, read_battle_options, relatch_battle_rects, seed_altar_records, seed_cat_god, stock_battle_items, unlock_dummy_combos,
};
use kore::Vfs;
use kore::domains::sandbox::replay::{self as tape, Cue, Recording};
use tracing::{info, trace, warn};

use super::assets::{DiskAssets, FileIndex, Ledger, SharedLedger, SheetCache};
use super::input::{Touch, TouchQueue};
use super::keys::{Action, Keys};
use super::replay;
use super::sink::{Frame, Recorder};
use super::sound::{SharedOutput, SharedVolumes, Speaker, Volumes};
use super::text::{Formatter, LABEL_PREFIX};

const CURTAIN_CLOSING: i32 = 1;
const IDENTITY: [f32; 6] = [1.0, 0.0, 0.0, 1.0, 0.0, 0.0];
const BATTLE_SCENE: i32 = 0x12c;
const TRANSITION_SCENE: i32 = 0x3e7;
const FADE_STEPS: usize = 0x10;
const FINGER_GAP: i32 = 400;
const PINCH_EASE: i32 = 4;
const PINCH_LIFT_FRAMES: u32 = 2;
const FULL_VOLUME: i32 = 100;
const DIM_ALPHA: i32 = 0x80;
const NOTCH_SHARE: f32 = 0.04;
const ALTAR_ROW_SHIFT: i32 = -2;

enum Tape {
    Off,
    Recording(Recording),
    Playing { frames: Vec<Vec<Cue>>, at: usize },
}

pub struct Driver {
    ctx: Box<AppContext>,
    frame: Rc<RefCell<Frame>>,
    sheets: Rc<RefCell<SheetCache>>,
    files: Rc<RefCell<FileIndex>>,
    touches: TouchQueue,
    volumes: SharedVolumes,
    output: SharedOutput,
    options: BattleOptions,
    setup: Setup,
    changed: bool,
    returning: Rc<Cell<bool>>,
    profile: Rc<DeviceProfile>,
    phone: bool,
    spread: i32,
    gap: Option<i32>,
    keys: Keys,
    reach: i32,
    idle: u32,
    booted: bool,
    forgiven: HashSet<Site>,
    tripped: Option<Site>,
    ledger: SharedLedger,
    seeds: Seeds,
    tape: Tape,
    pending: Vec<Cue>,
    window: (f32, f32),
    applied: (f32, f32),
    wanted_phone: bool,
    finished: bool,
    label: Label,
    recorded: Option<tape::Save>,
}

#[derive(Default)]
pub struct Label {
    pub map: String,
    pub stage: String,
    pub keepsakes: Vec<(Box<str>, PathBuf)>,
}

impl Driver {
    pub fn new() -> Self {
        let frame = Rc::new(RefCell::new(Frame::default()));
        let sheets: Rc<RefCell<SheetCache>> = Rc::new(RefCell::new(SheetCache::new()));
        let files: Rc<RefCell<FileIndex>> = Rc::new(RefCell::new(FileIndex::new()));
        let volumes: SharedVolumes = Rc::new(RefCell::new(Volumes {
            music: FULL_VOLUME,
            effects: FULL_VOLUME,
        }));
        let output = SharedOutput::open();
        let profile = Rc::new(DeviceProfile::default());
        let returning = Rc::new(Cell::new(false));

        profile.tablet.set(true);

        let mut driver = Self {
            ctx: Box::new(AppContext::default()),
            frame,
            sheets,
            files,
            touches: super::input::queue(),
            volumes,
            output,
            options: BattleOptions {
                music: FULL_VOLUME,
                effects: FULL_VOLUME,
                two_rows: false,
                vibrate: false,
            },
            setup: Setup::default(),
            changed: false,
            returning,
            profile,
            phone: false,
            spread: 0,
            gap: None,
            keys: Keys::default(),
            reach: FINGER_GAP,
            idle: 0,
            booted: false,
            forgiven: HashSet::new(),
            tripped: None,
            ledger: Rc::new(RefCell::new(Ledger::default())),
            seeds: Seeds::draw(),
            tape: Tape::Off,
            pending: Vec::new(),
            window: (0.0, 0.0),
            applied: (0.0, 0.0),
            wanted_phone: false,
            finished: false,
            label: Label::default(),
            recorded: None,
        };

        driver.host();
        driver
    }

    fn host(&mut self) {
        let ctx = &mut self.ctx;

        ctx.set_assets(Box::new(DiskAssets::new(
            Rc::clone(&self.files),
            Rc::clone(&self.sheets),
            Rc::clone(&self.ledger),
        )));
        ctx.set_platform(Box::new(InertPlatform {
            profile: Rc::clone(&self.profile),
        }));
        ctx.set_sound(Box::new(Speaker::new(
            Rc::clone(&self.files),
            Rc::clone(&self.ledger),
            Rc::clone(&self.volumes),
            self.output.share(),
        )));
        ctx.set_meta(Box::new(InertMeta));
        ctx.set_scene_host(Box::new(InertScene {
            returning: Rc::clone(&self.returning),
        }));
        ctx.set_text_renderer(Box::new(Formatter::new(Rc::clone(&self.sheets))));
        ctx.set_ui(Box::new(InertUi));
        ctx.draw = Some(Box::new(Recorder::new(Rc::clone(&self.frame))));

        if let Err(fault) = fill_dummy_save(ctx, &self.setup) {
            warn!("emu: dummy save could not be filled: {fault}");
        }
    }

    pub fn renew(&mut self) {
        self.forgiven.clear();
        self.tripped = None;

        let (width, height) = if self.watching() {
            self.phone = self.wanted_phone;
            self.window
        } else {
            (self.ctx.device_screen_w as f32, self.ctx.device_screen_h as f32)
        };
        let options = self.options;

        self.close_tape();
        self.seeds = Seeds::draw();

        *self.ctx = AppContext::default();
        self.sheets.borrow_mut().retain(|name, _| !name.starts_with(LABEL_PREFIX));
        self.touches.borrow_mut().clear();
        self.returning.set(false);
        self.spread = 0;
        self.gap = None;
        self.keys.reset();
        self.booted = false;
        self.host();
        self.apply_size(width, height);
        self.set_options(options);
    }

    fn close_tape(&mut self) {
        if let Tape::Recording(recording) = &mut self.tape
            && let Err(error) = recording.finish()
        {
            warn!("emu: the latest battle could not be written out: {error}");
        }

        self.tape = Tape::Off;
        self.recorded = None;
        self.pending.clear();
        self.finished = false;
        self.ledger.borrow_mut().disarm();
    }

    pub fn arm_recording(&mut self) {
        self.close_tape();

        let Some(dir) = tape::scratch() else {
            warn!("emu: there is no state folder, so this battle is not recorded");

            return;
        };

        let recording = match Recording::begin(&dir) {
            Ok(recording) => recording,
            Err(error) => {
                warn!("emu: the latest battle could not be started at {}: {error}", dir.display());

                return;
            }
        };
        let save = tape::Save {
            version: VERSION.to_owned(),
            seeds: replay::seeds_to(self.seeds),
            screen: tape::Screen {
                width: self.applied.0,
                height: self.applied.1,
                phone: self.phone,
            },
            options: replay::options_to(self.options),
            setup: replay::setup_to(&self.setup, &self.label.map, &self.label.stage),
            icons: Vec::new(),
            costs: Vec::new(),
            altar_cap: None,
        };

        if let Err(error) = recording.write_save(&save) {
            warn!("emu: the latest battle's save could not be written: {error}");

            return;
        }

        {
            let mut ledger = self.ledger.borrow_mut();

            ledger.arm();

            for (name, path) in &self.label.keepsakes {
                ledger.note(name, path);
            }
        }

        self.tape = Tape::Recording(recording);
        self.recorded = Some(save);
    }

    pub fn stamp_deck(&mut self) {
        let Tape::Recording(recording) = &self.tape else {
            return;
        };
        let Some(save) = self.recorded.as_mut() else {
            return;
        };

        save.icons = self
            .ctx
            .unit_icon_textures
            .iter()
            .map(|texture| texture.as_ref().map_or_else(String::new, |texture| String::from_utf8_lossy(&texture.png).into_owned()))
            .collect();
        save.costs = (0..DECK_SLOTS as i32)
            .map(|slot| emu::engine::get_effective_deploy_cost(&mut self.ctx, 0, slot).map_or(-1, emu::ops::div_100))
            .collect();
        save.altar_cap = emu::engine::get_castle_enemy_row(&self.ctx).ok().and_then(|row| {
            let enemy = row.wrapping_add(ALTAR_ROW_SHIFT);
            let sealed = emu::engine::stage_not_sealed(&self.ctx, enemy).is_ok_and(|open| !open);

            sealed.then(|| emu::engine::get_altar_level_cap(&self.ctx, enemy).unwrap_or(-1))
        });

        if let Err(error) = recording.write_save(save) {
            warn!("emu: the latest battle's deck icons could not be written: {error}");
        }
    }

    pub fn label(&mut self, label: Label) {
        self.label = label;
    }

    pub fn arm_playback(&mut self, save: &tape::Save, frames: Vec<Vec<Cue>>, index: FileIndex) {
        self.close_tape();
        self.set_setup(replay::setup_from(&save.setup));
        self.phone = save.screen.phone;
        self.apply_size(save.screen.width, save.screen.height);
        self.set_options(replay::options_from(save.options));
        self.seeds = replay::seeds_from(save.seeds);
        self.adopt_index(index);
        self.tape = Tape::Playing { frames, at: 0 };
    }

    pub fn end_playback(&mut self) {
        if !self.watching() {
            return;
        }

        self.close_tape();
        self.phone = self.wanted_phone;

        let (width, height) = self.window;

        self.apply_size(width, height);
    }

    pub fn watching(&self) -> bool {
        matches!(self.tape, Tape::Playing { .. })
    }

    pub fn reel_finished(&self) -> bool {
        self.finished
    }

    pub fn frame_aspect(&self) -> f32 {
        let width = self.ctx.screen_metrics.design_w;

        if width <= 0 {
            return 0.0;
        }

        emu::engine::get_design_height2(&self.ctx) as f32 / width as f32
    }

    pub fn screen(&self) -> (i32, i32) {
        (self.ctx.device_screen_w, self.ctx.device_screen_h)
    }

    pub fn keep_assets(&mut self) {
        let Tape::Recording(recording) = &mut self.tape else {
            return;
        };

        let requested = self.ledger.borrow_mut().drain();

        if requested.is_empty() {
            return;
        }

        for (name, error) in recording.keep(requested) {
            warn!("emu: {name} could not be kept for the replay: {error}");
        }
    }

    pub fn resolved(&self) -> usize {
        self.files.borrow().len()
    }

    pub fn forget_assets(&mut self) {
        self.files.borrow_mut().clear();
        self.sheets.borrow_mut().clear();
    }

    pub fn reindex(&mut self, vfs: &Vfs) {
        self.adopt_index(DiskAssets::index(vfs));
    }

    fn adopt_index(&mut self, fresh: FileIndex) {
        {
            let held = self.files.borrow();

            self.sheets.borrow_mut().retain(|name, _| held.get(name) == fresh.get(name));
        }

        *self.files.borrow_mut() = fresh;
    }

    pub fn sheets(&self) -> &Rc<RefCell<SheetCache>> {
        &self.sheets
    }

    pub fn frame(&self) -> &Rc<RefCell<Frame>> {
        &self.frame
    }

    pub fn touches(&self) -> &TouchQueue {
        &self.touches
    }

    pub fn key(&mut self, action: Action, pressed: bool) {
        if self.watching() {
            return;
        }

        if matches!(self.tape, Tape::Recording(_)) {
            self.pending.push(Cue::Key(replay::key_of(action), pressed));
        }

        if pressed {
            self.keys.press(action);
        } else {
            self.keys.release(action);
        }
    }

    fn pump_input(&mut self, script: Option<Vec<Cue>>) -> Vec<Cue> {
        self.keys.pump(&mut self.ctx, &mut self.spread, &mut self.gap);

        let mut played: Vec<Cue> = Vec::new();

        match script {
            Some(cues) => {
                self.touches.borrow_mut().clear();

                for cue in cues.into_iter().filter(|cue| !cue.before_input()) {
                    self.play(cue);
                }
            }
            None => {
                let keyed = self.keys.busy();
                let mut pending: VecDeque<Touch> = self.touches.borrow_mut().drain(..).collect();

                while let Some(touch) = pending.pop_front() {
                    let cue = match touch {
                        Touch::Pinched { spread: step } => Some(Cue::Pinch(step)),
                        Touch::Moved { .. } | Touch::Pressed { .. } | Touch::Released if keyed => None,
                        Touch::Moved { x, y } => Some(Cue::Move(x, y)),
                        Touch::Pressed { .. } if self.gap.is_some() => {
                            pending.push_front(touch);
                            self.touches.borrow_mut().extend(pending.drain(..));

                            Some(Cue::Settle)
                        }
                        Touch::Pressed { x, y } => Some(Cue::Press(x, y)),
                        Touch::Released => Some(Cue::Release),
                    };

                    let Some(cue) = cue else {
                        continue;
                    };

                    self.play(cue);

                    match (played.last_mut(), cue) {
                        (Some(last @ Cue::Move(..)), Cue::Move(..)) => *last = cue,
                        _ => played.push(cue),
                    }
                }
            }
        }

        self.ease_pinch();

        if let Err(fault) = emu::runtime::pump_pinch(&mut self.ctx, self.gap) {
            warn!("emu: pinch could not be pumped: {fault}");
        }

        if let Err(fault) = emu::runtime::pump_touch(&mut self.ctx) {
            warn!("emu: touch could not be pumped: {fault}");
        }

        played
    }

    fn play(&mut self, cue: Cue) {
        let fed = match cue {
            Cue::Pinch(step) => {
                self.spread = self.spread.wrapping_add(step);

                Ok(())
            }
            Cue::Settle => {
                self.spread = 0;
                self.gap = None;

                Ok(())
            }
            Cue::Move(x, y) => queue_touch_position(&mut self.ctx, x, y),
            Cue::Press(x, y) => queue_touch_press(&mut self.ctx, x, y),
            Cue::Release => queue_touch_release(&mut self.ctx),
            Cue::Key(key, pressed) => {
                let action = replay::action_of(key);

                if pressed {
                    self.keys.press(action);
                } else {
                    self.keys.release(action);
                }

                Ok(())
            }
            Cue::Resize(width, height) => {
                self.apply_size(width, height);

                Ok(())
            }
            Cue::Phone(phone) => {
                let (width, height) = self.screen();

                self.phone = phone;
                self.apply_size(width as f32, height as f32);

                Ok(())
            }
        };

        if let Err(fault) = fed {
            warn!("emu: touch could not be queued: {fault}");
        }
    }

    fn ease_pinch(&mut self) {
        let spread = mem::take(&mut self.spread);

        let Some(gap) = self.gap else {
            if spread != 0 {
                self.gap = Some(FINGER_GAP);
                self.reach = FINGER_GAP.wrapping_add(spread).max(1);
                self.idle = 0;
            }

            return;
        };

        self.reach = self.reach.wrapping_add(spread).max(1);

        let left = self.reach.wrapping_sub(gap);

        if left == 0 {
            self.idle = self.idle.saturating_add(1);

            if self.idle > PINCH_LIFT_FRAMES {
                self.gap = None;
            }

            return;
        }

        let eased = left / PINCH_EASE;
        let moved = if eased == 0 { left.signum() } else { eased };

        self.idle = 0;
        self.gap = Some(gap.wrapping_add(moved));
    }

    pub fn resize(&mut self, width: f32, height: f32) {
        if width < 1.0 || height < 1.0 {
            return;
        }

        self.window = (width, height);

        if self.watching() {
            return;
        }

        if matches!(self.tape, Tape::Recording(_)) {
            self.pending.push(Cue::Resize(width, height));
        }

        self.apply_size(width, height);
    }

    fn apply_size(&mut self, width: f32, height: f32) {
        if width < 1.0 || height < 1.0 {
            return;
        }

        self.applied = (width, height);

        self.ctx.device_screen_w = width as i32;
        self.ctx.device_screen_h = height as i32;
        self.profile.tablet.set(!self.phone);
        self.profile
            .side_inset
            .set(if self.phone { (width * NOTCH_SHARE).round() as i32 } else { 0 });

        if let Err(fault) = emu::engine::compute_layout_metrics(&mut self.ctx) {
            warn!("emu: layout metrics could not be computed: {fault}");
        }

        if !self.in_battle() {
            return;
        }

        if let Err(fault) = relatch_battle_rects(&mut self.ctx) {
            warn!("emu: battle rects could not be re-latched: {fault}");
        }
    }

    pub fn set_phone(&mut self, phone: bool) {
        self.wanted_phone = phone;

        if self.watching() {
            return;
        }

        if matches!(self.tape, Tape::Recording(_)) {
            self.pending.push(Cue::Phone(phone));
        }

        self.phone = phone;

        let width = self.ctx.device_screen_w as f32;
        let height = self.ctx.device_screen_h as f32;

        self.apply_size(width, height);
    }

    pub fn design_width(&self) -> f32 {
        self.ctx.screen_metrics.design_w as f32
    }

    pub fn in_battle(&self) -> bool {
        emu::engine::get_scene_id(&self.ctx).is_ok_and(|scene| scene == BATTLE_SCENE)
    }

    pub fn fading(&self) -> bool {
        self.ctx
            .u8_at(AppContext::CURTAIN_ACTIVE)
            .is_ok_and(|active| active != 0)
    }

    pub fn boot(&mut self) -> bool {
        if self.booted {
            return true;
        }

        plant_seeds(&mut self.ctx, self.seeds);

        if let Err(fault) = emu::engine::load_misc_data_tables(&mut self.ctx) {
            warn!("emu: text tables failed to load: {fault}");

            return false;
        }

        match emu::engine::initialize_game_data(&mut self.ctx) {
            Ok(true) => {
                unlock_dummy_combos(&mut self.ctx);

                fill_dummy_talents(&mut self.ctx, &self.setup);

                if let Err(fault) = load_scene_sheets(&mut self.ctx) {
                    warn!("emu: scene sheets failed to load: {fault}");
                }

                self.booted = true;
                info!("emu: game data loaded");
            }
            Ok(false) => warn!("emu: game data refused to load"),
            Err(fault) => warn!("emu: game data failed to load: {fault}"),
        }

        self.booted
    }

    pub fn enter_battle(&mut self) -> bool {
        if let Err(fault) = stock_battle_items(&mut self.ctx, &self.setup) {
            warn!("emu: battle items could not be stocked: {fault}");
        }

        if let Err(fault) = seed_cat_god(&mut self.ctx, &self.setup) {
            warn!("emu: Cat God could not be seeded: {fault}");
        }

        if let Err(fault) = seed_altar_records(&mut self.ctx, &self.setup) {
            warn!("emu: altar records could not be seeded: {fault}");
        }

        if let Err(fault) = self.select_stage() {
            warn!("emu: stage could not be selected: {fault}");

            return false;
        }

        let prepared = match emu::runtime::is_extra_entry(&self.ctx) {
            Ok(true) => emu::runtime::prepare_extra_entry(&mut self.ctx),
            _ => emu::engine::prepare_battle_entry(&mut self.ctx),
        };

        match prepared {
            Ok(true) => {}
            Ok(false) => return false,
            Err(fault) => {
                warn!("emu: battle entry faulted: {fault}");

                return false;
            }
        }

        if let Err(fault) = self.fade_into_battle() {
            warn!("emu: stage setup faulted: {fault}");

            return false;
        }

        self.in_battle()
    }

    fn fade_into_battle(&mut self) -> Result<(), emu::Fault> {
        emu::engine::set_scene(&mut self.ctx, TRANSITION_SCENE)?;
        self.ctx.set_block_at::<1>(AppContext::CURTAIN_ACTIVE, [1])?;
        self.ctx.set_i32_at(AppContext::CURTAIN_STYLE, CURTAIN_CLOSING)?;
        self.ctx.set_block_at::<1>(AppContext::FADE_STARTED, [0])?;
        self.ctx.set_i32_at(AppContext::FADE_FRAME, 0)?;

        for _ in 0..FADE_STEPS {
            if self.in_battle() {
                break;
            }

            emu::engine::fade_update(&mut self.ctx, CURTAIN_CLOSING)?;
        }

        Ok(())
    }

    fn select_stage(&mut self) -> Result<(), emu::Fault> {
        emu::runtime::select_stage(&mut self.ctx, self.setup.stage)
    }

    pub fn set_setup(&mut self, setup: Setup) {
        if self.setup == setup {
            return;
        }

        self.setup = setup;
        self.renew();
    }

    pub fn advance(&mut self) -> Result<(), String> {
        match self.run_frame() {
            Ok(()) => Ok(()),
            Err((site, reason)) if self.forgiven.contains(&site) => {
                trace!("emu: {reason} (forgiven for this battle)");

                Ok(())
            }
            Err((site, reason)) => {
                self.tripped = Some(site);

                Err(reason)
            }
        }
    }

    pub fn exit_requested(&mut self) -> bool {
        self.keys.take_exit()
    }

    pub fn forgive(&mut self) {
        let Some(site) = self.tripped.take() else {
            return;
        };

        if self.forgiven.insert(site) {
            info!("emu: {site} was continued past, so it stays quiet for the rest of this battle");
        }
    }

    fn run_frame(&mut self) -> Result<(), (Site, String)> {
        if !self.in_battle() {
            return Ok(());
        }

        let script = match &mut self.tape {
            Tape::Playing { frames, at } => {
                let Some(cues) = frames.get_mut(*at).map(mem::take) else {
                    self.finished = true;

                    return Ok(());
                };

                *at += 1;

                Some(cues)
            }
            Tape::Off | Tape::Recording(_) => None,
        };

        if let Some(cues) = &script {
            for cue in cues.iter().copied().filter(|cue| cue.before_input()) {
                self.play(cue);
            }
        }

        let before = mem::take(&mut self.pending);
        let played = self.pump_input(script);

        if let Tape::Recording(recording) = &mut self.tape {
            let line: Vec<Cue> = before.into_iter().chain(played).collect();

            if let Err(error) = recording.frame(&line) {
                warn!("emu: the latest battle's input could not be written: {error}");
            }
        }

        self.keep_assets();
        emu::engine::dialog_manager_process(&mut self.ctx)
            .map_err(|fault| (fault.site(), format!("dialog_manager_process:{fault}")))?;

        emu::engine::button_bank_process(&mut self.ctx)
            .map_err(|fault| (fault.site(), format!("button_bank_process:{fault}")))?;
        emu::engine::main_battle_loop(&mut self.ctx)
            .map_err(|fault| (fault.site(), format!("main_battle_loop:{fault}")))?;
        pump_stage_return(&mut self.ctx, &self.returning)
            .map_err(|fault| (fault.site(), format!("pump_stage_return:{fault}")))?;
        self.sync_options();

        if !self.in_battle() {
            return Ok(());
        }

        self.frame.borrow_mut().clear();
        self.begin_draw();
        emu::engine::main_draw(&mut self.ctx, 0)
            .map_err(|fault| (fault.site(), format!("main_draw:{fault}")))?;
        emu::engine::dialog_manager_draw(&mut self.ctx)
            .map_err(|fault| (fault.site(), format!("dialog_manager_draw:{fault}")))?;
        self.draw_letterbox_bars();

        Ok(())
    }

    fn begin_draw(&mut self) {
        let scale = self.ctx.screen_metrics.scale2;
        let cleared = emu::engine::draw_context(&mut self.ctx.draw).map(|sink| {
            emu::engine::set_transform(sink, scale, &IDENTITY);
            emu::engine::set_color(sink, 0xff, 0xff, 0xff, 0xff);
            emu::engine::set_alpha(sink, 0xff);
        });

        if let Err(fault) = cleared {
            warn!("emu: draw state could not be reset: {fault}");

            return;
        }

        let origin = self
            .ctx
            .i32_at(AppContext::LETTERBOX_PAD)
            .and_then(|pad| {
                self.ctx
                    .i32_at(AppContext::LETTERBOX_SHIFT)
                    .map(|shift| pad.wrapping_add(shift))
            })
            .and_then(|top| emu::engine::set_draw_origin(&mut self.ctx, 0, top));

        if let Err(fault) = origin {
            warn!("emu: draw origin could not be set: {fault}");
        }
    }

    fn draw_letterbox_bars(&mut self) {
        let Ok(pad) = self.ctx.i32_at(AppContext::LETTERBOX_PAD) else {
            return;
        };

        if pad <= 0 {
            return;
        }

        if let Err(fault) = emu::engine::set_draw_origin(&mut self.ctx, 0, 0) {
            warn!("emu: letterbox origin could not be set: {fault}");

            return;
        }

        let Ok(width) = emu::engine::get_drawable_width(&self.ctx) else {
            return;
        };
        let height = emu::engine::get_design_height2(&self.ctx);

        let barred = emu::engine::draw_context(&mut self.ctx.draw).map(|sink| {
            emu::engine::set_tint(sink, 0, 0, 0, 0xff);
            emu::engine::fill_rect(sink, 0, 0, width, pad);
            emu::engine::fill_rect(sink, 0, height.wrapping_sub(pad), width, pad);
            emu::engine::set_tint(sink, 0xff, 0xff, 0xff, 0xff);
        });

        if let Err(fault) = barred {
            warn!("emu: letterbox bars could not be drawn: {fault}");
        }
    }

    pub fn silence(&mut self) {
        if let Some(sound) = self.ctx.sound() {
            sound.pause_all();
        }
    }

    pub fn set_volumes(&mut self, music: i32, effects: i32) {
        *self.volumes.borrow_mut() = Volumes { music, effects };

        let Some(sound) = self.ctx.sound() else {
            return;
        };

        sound.set_bgm_duck(FULL_VOLUME);
    }

    pub fn set_options(&mut self, options: BattleOptions) {
        self.options = options;

        if let Err(fault) = apply_battle_options(&mut self.ctx, options) {
            warn!("emu: battle options could not be applied: {fault}");
        }
    }

    pub fn take_options(&mut self) -> Option<BattleOptions> {
        let changed = mem::take(&mut self.changed);

        (changed && !self.watching()).then_some(self.options)
    }

    fn sync_options(&mut self) {
        match read_battle_options(&mut self.ctx) {
            Ok(options) if options != self.options => {
                self.options = options;
                self.changed = true;
            }
            Ok(_) => (),
            Err(fault) => warn!("emu: battle options could not be read: {fault}"),
        }
    }

    pub fn dim(&mut self) {
        let Ok(width) = emu::engine::get_drawable_width(&self.ctx) else {
            return;
        };
        let height = emu::engine::get_design_height2(&self.ctx);

        if let Err(fault) = emu::engine::set_draw_origin(&mut self.ctx, 0, 0) {
            warn!("emu: dim origin could not be set: {fault}");

            return;
        }

        let dimmed = emu::engine::draw_context(&mut self.ctx.draw).map(|sink| {
            emu::engine::set_draw_scale(sink, 1.0);
            emu::engine::set_tint(sink, 0, 0, 0, DIM_ALPHA);
            emu::engine::fill_rect(sink, 0, 0, width, height);
            emu::engine::set_tint(sink, 0xff, 0xff, 0xff, 0xff);
        });

        if let Err(fault) = dimmed {
            warn!("emu: frame could not be dimmed: {fault}");
        }
    }

    pub fn draw_curtain_over(&mut self, sweep: i32) {
        let armed = self
            .ctx
            .set_block_at::<1>(AppContext::CURTAIN_ACTIVE, [1])
            .and_then(|()| self.ctx.set_i32_at(AppContext::FADE_FRAME, sweep));

        if let Err(fault) = armed {
            warn!("emu: curtain frame could not be set: {fault}");

            return;
        }

        self.begin_draw();

        if let Err(fault) = emu::engine::draw_screen_transition(&mut self.ctx, CURTAIN_CLOSING) {
            warn!("emu: curtain draw faulted: {fault}");
        }

        self.draw_letterbox_bars();
    }

    pub fn release_curtain(&mut self) {
        let cleared = self
            .ctx
            .set_block_at::<1>(AppContext::CURTAIN_ACTIVE, [0])
            .and_then(|()| self.ctx.set_i32_at(AppContext::FADE_FRAME, 0));

        if let Err(fault) = cleared {
            warn!("emu: curtain could not be released: {fault}");
        }
    }

    pub fn draw_curtain(&mut self, sweep: i32) {
        let pad = match self.ctx.i32_at(AppContext::LETTERBOX_PAD) {
            Ok(pad) => pad,
            Err(fault) => {
                warn!("emu: letterbox could not be read: {fault}");

                return;
            }
        };

        if let Err(fault) = self.ctx.set_i32_at(AppContext::LETTERBOX_PAD, 0) {
            warn!("emu: letterbox could not be lifted: {fault}");

            return;
        }

        self.frame.borrow_mut().clear();
        self.begin_draw();

        let armed = self
            .ctx
            .set_block_at::<1>(AppContext::CURTAIN_ACTIVE, [1])
            .and_then(|()| self.ctx.set_i32_at(AppContext::FADE_FRAME, sweep));

        match armed {
            Ok(()) => {
                if let Err(fault) = emu::engine::draw_screen_transition(&mut self.ctx, CURTAIN_CLOSING) {
                    warn!("emu: curtain draw faulted: {fault}");
                }
            }
            Err(fault) => warn!("emu: curtain frame could not be set: {fault}"),
        }

        if let Err(fault) = self.ctx.set_i32_at(AppContext::LETTERBOX_PAD, pad) {
            warn!("emu: letterbox could not be restored: {fault}");
        }
    }
}
