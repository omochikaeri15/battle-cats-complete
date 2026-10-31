use std::cell::{Cell, RefCell};
use std::collections::{HashSet, VecDeque};
use std::mem;
use std::rc::Rc;
use std::sync::Arc;

use tracing::{info, trace, warn};

use emulator::Site;
use emulator::engine::{AppContext, Platform, set_bgm_volume_setting, set_se_volume_setting, sound_manager, sound_pause_all};
use emulator::runtime::{
    BattleOptions, DeviceProfile, InertMeta, InertPlatform, InertScene, InertUi, Seeds, Setup, apply_battle_options, fill_dummy_cannon_parts, fill_dummy_save,
    fill_dummy_talents, plant_seeds, preload_sounds, pump_stage_return, read_battle_options, relatch_battle_rects, seed_altar_records, seed_cat_god, seed_point_cap,
    set_limits, stock_battle_items, unlock_dummy_combos,
};

use super::assets::{Assets, Files, SheetCache};
use super::diagnostics::{self, Diagnostics};
use super::input::{Touch, TouchQueue};
use super::keys::{Action, Keys, LEAVE_AFTER};
use super::sink::{Frame, Recorder};
use super::sound::{Backend, Pool};
use super::text::{Formatter, LABEL_PREFIX};

const CURTAIN_CLOSING: i32 = 1;
const TOUCH_PRESSED: i32 = 0;
const TOUCH_MOVED: i32 = 1;
const TOUCH_RELEASED: i32 = 2;
const BATTLE_SCENE: i32 = 0x12c;
const TRANSITION_SCENE: i32 = 0x3e7;
const FADE_STEPS: usize = 0x10;
const FINGER_GAP: i32 = 400;
const PINCH_EASE: i32 = 4;
const PINCH_LIFT_FRAMES: u32 = 2;
const FULL_VOLUME: i32 = 100;
const DIM_ALPHA: i32 = 0x80;
const NOTCH_SHARE: f32 = 0.04;

pub trait Clock {
    fn now_ms(&self) -> f64;
    fn epoch_micros(&self) -> i64;
}

struct HostPlatform {
    inner: InertPlatform,
    clock: Rc<dyn Clock>,
}

impl Platform for HostPlatform {
    fn set_keep_awake(&mut self, awake: bool) {
        self.inner.set_keep_awake(awake);
    }

    fn is_tablet(&mut self) -> bool {
        self.inner.is_tablet()
    }

    fn ad_showing(&mut self) -> bool {
        self.inner.ad_showing()
    }

    fn screen_window_ratio(&mut self) -> f32 {
        self.inner.screen_window_ratio()
    }

    fn safe_inset_left(&mut self) -> i32 {
        self.inner.safe_inset_left()
    }

    fn safe_inset_top(&mut self) -> i32 {
        self.inner.safe_inset_top()
    }

    fn safe_inset_right(&mut self) -> i32 {
        self.inner.safe_inset_right()
    }

    fn safe_inset_bottom(&mut self) -> i32 {
        self.inner.safe_inset_bottom()
    }

    fn system_clock_now(&mut self) -> i64 {
        self.clock.epoch_micros()
    }

    fn vibrate(&mut self, delay: f64, duration: f64, amplitude: f64) {
        self.inner.vibrate(delay, duration, amplitude);
    }

    fn cancel_vibration(&mut self) {
        self.inner.cancel_vibration();
    }

    fn has_vibrator(&mut self) -> bool {
        self.inner.has_vibrator()
    }

    fn web_view_is_open(&mut self) -> bool {
        self.inner.web_view_is_open()
    }

    fn share_image(&mut self, x: i32, y: i32, width: i32, height: i32) {
        self.inner.share_image(x, y, width, height);
    }

    fn has_inquiry_code(&mut self) -> bool {
        self.inner.has_inquiry_code()
    }

    fn connecting_show(&mut self, text: &[u8]) {
        self.inner.connecting_show(text);
    }

    fn connecting_hide(&mut self) {
        self.inner.connecting_hide();
    }

    fn ranking_submit(&mut self, entry: i32, score: i32) {
        self.inner.ranking_submit(entry, score);
    }

    fn is_network_available(&mut self) -> bool {
        self.inner.is_network_available()
    }

    fn feature_enabled(&mut self, feature: i32) -> bool {
        self.inner.feature_enabled(feature)
    }

    fn config_int(&mut self, key: &[u8], min: i32, max: i32) -> i32 {
        self.inner.config_int(key, min, max)
    }

    fn reward_ad_ready(&mut self, kind: i32, flag: i32) -> bool {
        self.inner.reward_ad_ready(kind, flag)
    }

    fn show_rewarded_ad(&mut self, kind: i32) {
        self.inner.show_rewarded_ad(kind);
    }

    fn web_popup_open(&mut self, kind: i32, map: i32, stage: i32) {
        self.inner.web_popup_open(kind, map, stage);
    }

    fn labyrinth_submit(&mut self, cleared: i32, units: i32) {
        self.inner.labyrinth_submit(cleared, units);
    }

    fn ad_prepare(&mut self, kind: i32) {
        self.inner.ad_prepare(kind);
    }

    fn event_schedule_active(&mut self, now: f64) -> Vec<Vec<i32>> {
        self.inner.event_schedule_active(now)
    }

    fn config_json_int(&mut self, section: &[u8], name: &[u8]) -> Option<i32> {
        self.inner.config_json_int(section, name)
    }

    fn notification_schedule(&mut self, flag: u8, kind: i32) {
        self.inner.notification_schedule(flag, kind);
    }
}

pub struct Equipment {
    pub files: Rc<dyn Files>,
    pub backend: Rc<dyn Backend>,
    pub clock: Rc<dyn Clock>,
    pub fonts: Vec<Arc<[u8]>>,
    pub priority: Vec<String>,
}

pub struct Driver {
    ctx: Box<AppContext>,
    frame: Rc<RefCell<Frame>>,
    sheets: Rc<RefCell<SheetCache>>,
    equipment: Equipment,
    touches: TouchQueue,
    options: BattleOptions,
    setup: Setup,
    changed: bool,
    returning: Rc<Cell<bool>>,
    profile: Rc<DeviceProfile>,
    phone: bool,
    spread: i32,
    gap: Option<i32>,
    finger: (i32, i32),
    keys: Keys,
    reach: i32,
    idle: u32,
    booted: bool,
    forgiven: HashSet<Site>,
    tripped: Option<Site>,
    reason: String,
    seeds: Seeds,
    applied: (f32, f32),
    leaving: Option<u16>,
    resolved: usize,
}

impl Driver {
    pub fn new(equipment: Equipment) -> Self {
        let frame = Rc::new(RefCell::new(Frame::default()));
        let sheets: Rc<RefCell<SheetCache>> = Rc::new(RefCell::new(SheetCache::new()));
        let profile = Rc::new(DeviceProfile::default());
        let returning = Rc::new(Cell::new(false));

        profile.tablet.set(true);

        let mut driver = Self {
            ctx: Box::new(AppContext::default()),
            frame,
            sheets,
            equipment,
            touches: super::input::queue(),
            options: BattleOptions { music: FULL_VOLUME, effects: FULL_VOLUME, two_rows: false, vibrate: false },
            setup: Setup::default(),
            changed: false,
            returning,
            profile,
            phone: false,
            spread: 0,
            gap: None,
            finger: (0, 0),
            keys: Keys::default(),
            reach: FINGER_GAP,
            idle: 0,
            booted: false,
            forgiven: HashSet::new(),
            tripped: None,
            reason: String::new(),
            seeds: Seeds::draw(),
            applied: (0.0, 0.0),
            leaving: None,
            resolved: 0,
        };

        driver.host();
        driver
    }

    fn host(&mut self) {
        let ctx = &mut self.ctx;
        let assets = Assets::new(Rc::clone(&self.equipment.files), Rc::clone(&self.sheets), &self.equipment.priority);
        let wanted = assets.limits();
        let applied = set_limits(ctx, wanted);

        self.resolved = assets.resolved();

        if applied != wanted {
            trace!("emu: limits wanted {wanted:?}, applied {applied:?}");
        }

        ctx.set_assets(Box::new(assets));
        ctx.set_platform(Box::new(HostPlatform { inner: InertPlatform { profile: Rc::clone(&self.profile) }, clock: Rc::clone(&self.equipment.clock) }));
        ctx.set_audio(Box::new(Pool::new(Rc::clone(&self.equipment.files), Rc::clone(&self.equipment.backend))));
        ctx.set_meta(Box::new(InertMeta));
        ctx.set_scene_host(Box::new(InertScene { returning: Rc::clone(&self.returning) }));
        ctx.set_text_renderer(Box::new(Formatter::new(Rc::clone(&self.sheets), &self.equipment.fonts)));
        ctx.set_ui(Box::new(InertUi));
        ctx.draw = Some(Box::new(Recorder::new(Rc::clone(&self.frame))));
        ctx.default_font = b"FONT_SYSTEM_BOLD".to_vec();

        if let Err(fault) = fill_dummy_save(ctx, &self.setup) {
            warn!("emu: dummy save could not be filled: {fault}");
        }
    }

    pub fn renew(&mut self) {
        self.forgiven.clear();
        self.tripped = None;

        let (width, height) = (self.ctx.device_screen_w as f32, self.ctx.device_screen_h as f32);
        let options = self.options;

        self.leaving = None;
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

    pub fn relocalize(&mut self, priority: Vec<String>) {
        self.equipment.priority = priority;
    }

    pub fn diagnose(&self) -> Diagnostics {
        diagnostics::read(&self.ctx)
    }

    pub fn resolved(&self) -> usize {
        self.resolved
    }

    pub fn forget_assets(&mut self) {
        self.sheets.borrow_mut().clear();
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
        if pressed {
            self.keys.press(action);
        } else {
            self.keys.release(action);
        }
    }

    fn pump_input(&mut self) {
        self.keys.pump(&mut self.ctx, &mut self.spread, &mut self.gap);

        let keyed = self.keys.busy();
        let mut pending: VecDeque<Touch> = self.touches.borrow_mut().drain(..).collect();

        while let Some(touch) = pending.pop_front() {
            let fed = match touch {
                Touch::Pinched { spread: step } => {
                    self.spread = self.spread.wrapping_add(step);

                    Ok(())
                }
                Touch::Moved { .. } | Touch::Pressed { .. } | Touch::Released if keyed => Ok(()),
                Touch::Moved { x, y } => {
                    self.finger = (x, y);

                    emulator::engine::app_on_touch(&mut self.ctx, TOUCH_MOVED, x, y, 0)
                }
                Touch::Pressed { .. } if self.gap.is_some() => {
                    self.spread = 0;
                    self.gap = None;
                    pending.push_front(touch);
                    self.touches.borrow_mut().extend(pending.drain(..));

                    Ok(())
                }
                Touch::Pressed { x, y } => {
                    self.finger = (x, y);

                    emulator::engine::app_on_touch(&mut self.ctx, TOUCH_PRESSED, x, y, 0)
                }
                Touch::Released => emulator::engine::app_on_touch(&mut self.ctx, TOUCH_RELEASED, self.finger.0, self.finger.1, 0),
            };

            if let Err(fault) = fed {
                warn!("emu: touch could not be queued: {fault}");
            }
        }

        self.ease_pinch();

        if let Err(fault) = emulator::runtime::pump_pinch(&mut self.ctx, self.gap) {
            warn!("emu: pinch could not be pumped: {fault}");
        }

        if let Err(fault) = emulator::runtime::pump_touch(&mut self.ctx) {
            warn!("emu: touch could not be pumped: {fault}");
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
        self.apply_size(width, height);
    }

    fn apply_size(&mut self, width: f32, height: f32) {
        if width < 1.0 || height < 1.0 {
            return;
        }

        self.applied = (width, height);

        trace!(width, height, phone = self.phone, in_battle = self.in_battle(), "emu: screen size applied");

        let before = emulator::runtime::layout_snapshot(&mut self.ctx);

        self.ctx.device_screen_w = width as i32;
        self.ctx.device_screen_h = height as i32;
        self.profile.tablet.set(!self.phone);
        self.profile.side_inset.set(if self.phone { (width * NOTCH_SHARE).round() as i32 } else { 0 });

        if let Err(fault) = emulator::engine::compute_layout_metrics(&mut self.ctx) {
            warn!("emu: layout metrics could not be computed: {fault}");
        }

        if !self.in_battle() {
            return;
        }

        let relatched = before.and_then(|before| relatch_battle_rects(&mut self.ctx, before));

        if let Err(fault) = relatched {
            warn!("emu: battle rects could not be re-latched: {fault}");
        }
    }

    pub fn set_phone(&mut self, phone: bool) {
        self.phone = phone;

        let width = self.ctx.device_screen_w as f32;
        let height = self.ctx.device_screen_h as f32;

        self.apply_size(width, height);
    }

    pub fn in_battle(&self) -> bool {
        emulator::engine::get_scene_id(&self.ctx).is_ok_and(|scene| scene == BATTLE_SCENE)
    }

    pub fn fading(&self) -> bool {
        self.ctx.u8_at(AppContext::CURTAIN_ACTIVE).is_ok_and(|active| active != 0)
    }

    pub fn boot(&mut self) -> bool {
        if self.booted {
            return true;
        }

        plant_seeds(&mut self.ctx, self.seeds);

        let mark = self.equipment.clock.now_ms();

        if let Err(fault) = emulator::engine::load_misc_data_tables(&mut self.ctx) {
            warn!("emu: text tables failed to load: {fault}");

            return false;
        }

        trace!(ms = (self.equipment.clock.now_ms() - mark) as u32, "emu: text tables loaded");

        let mark = self.equipment.clock.now_ms();
        let initialized = emulator::engine::initialize_game_data(&mut self.ctx);

        trace!(ms = (self.equipment.clock.now_ms() - mark) as u32, files = self.resolved, "emu: game data initialized");

        match initialized {
            Ok(true) => {
                if let Err(fault) = preload_sounds(&mut self.ctx) {
                    warn!("emu: sound effects failed to load: {fault}");
                }

                unlock_dummy_combos(&mut self.ctx);

                fill_dummy_talents(&mut self.ctx, &self.setup);
                fill_dummy_cannon_parts(&mut self.ctx, &self.setup);

                if let Err(fault) = emulator::engine::img_dialog_context_init(&mut self.ctx) {
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
        let mark = self.equipment.clock.now_ms();

        trace!(map = self.setup.stage.map_id, stage = self.setup.stage.stage, units = self.setup.lineup.len(), "emu: entering the battle");

        if let Err(fault) = stock_battle_items(&mut self.ctx, &self.setup) {
            warn!("emu: battle items could not be stocked: {fault}");
        }

        if let Err(fault) = seed_cat_god(&mut self.ctx, &self.setup) {
            warn!("emu: Cat God could not be seeded: {fault}");
        }

        if let Err(fault) = seed_altar_records(&mut self.ctx, &self.setup) {
            warn!("emu: altar records could not be seeded: {fault}");
        }

        seed_point_cap(&mut self.ctx);

        if let Err(fault) = emulator::runtime::select_stage(&mut self.ctx, self.setup.stage) {
            warn!("emu: stage could not be selected: {fault}");

            return false;
        }

        if let Err(fault) = emulator::runtime::seed_stage_variant(&mut self.ctx, self.setup.stage) {
            warn!("emu: stage variant could not be seeded: {fault}");

            return false;
        }

        let prepared = match emulator::runtime::is_extra_entry(&self.ctx) {
            Ok(true) => emulator::runtime::prepare_extra_entry(&mut self.ctx),
            _ => emulator::engine::prepare_battle_entry(&mut self.ctx),
        };

        trace!(ms = (self.equipment.clock.now_ms() - mark) as u32, outcome = ?prepared, "emu: battle entry prepared");

        match prepared {
            Ok(true) => {}
            Ok(false) => {
                warn!("emu: battle entry was refused by the stage data");

                return false;
            }
            Err(fault) => {
                warn!("emu: battle entry faulted: {fault}");

                return false;
            }
        }

        if let Err(fault) = emulator::runtime::select_dungeon_stage(&mut self.ctx, self.setup.stage) {
            warn!("emu: dungeon stage could not be selected: {fault}");

            return false;
        }

        if let Err(fault) = self.fade_into_battle() {
            warn!("emu: stage setup faulted: {fault}");

            return false;
        }

        let entered = self.in_battle();

        trace!(entered, scene = ?emulator::engine::get_scene_id(&self.ctx), "emu: stage set up");

        entered
    }

    fn fade_into_battle(&mut self) -> Result<(), emulator::Fault> {
        emulator::engine::set_scene(&mut self.ctx, TRANSITION_SCENE)?;
        emulator::runtime::reload_entry_map(&mut self.ctx)?;
        self.ctx.set_block_at::<1>(AppContext::CURTAIN_ACTIVE, [1])?;
        self.ctx.set_i32_at(AppContext::CURTAIN_STYLE, CURTAIN_CLOSING)?;
        self.ctx.set_block_at::<1>(AppContext::FADE_STARTED, [0])?;
        self.ctx.set_i32_at(AppContext::FADE_FRAME, 0)?;

        for _ in 0..FADE_STEPS {
            if self.in_battle() {
                break;
            }

            emulator::engine::fade_update(&mut self.ctx, CURTAIN_CLOSING)?;
        }

        Ok(())
    }

    pub fn set_setup(&mut self, setup: Setup) {
        if self.setup == setup {
            return;
        }

        self.setup = setup;
        self.renew();
    }

    pub fn advance(&mut self) -> Result<(), String> {
        if let Some(held) = self.leaving.as_mut() {
            *held = held.saturating_add(1);
        }

        match self.run_frame() {
            Ok(()) => Ok(()),
            Err((site, reason)) if self.forgiven.contains(&site) => {
                trace!("emu: {reason} (forgiven for this battle)");

                Ok(())
            }
            Err((site, reason)) => {
                self.tripped = Some(site);
                self.reason.clone_from(&reason);

                Err(reason)
            }
        }
    }

    pub fn exit_requested(&mut self) -> bool {
        let held = self.leaving.is_some_and(|held| held >= LEAVE_AFTER);

        if held {
            self.leaving = None;
        }

        self.keys.take_exit() || held
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

        self.pump_input();

        let notice = emulator::engine::notice_popup_update(&mut self.ctx).map_err(|fault| (fault.site(), format!("notice_popup_update::{fault}")))?;

        if emulator::engine::ad_is_showing(&mut self.ctx).map_err(|fault| (fault.site(), format!("ad_is_showing::{fault}")))? {
            return Ok(());
        }

        emulator::engine::dialog_manager_process(&mut self.ctx).map_err(|fault| (fault.site(), format!("dialog_manager_process::{fault}")))?;
        emulator::engine::button_bank_process(&mut self.ctx).map_err(|fault| (fault.site(), format!("button_bank_process::{fault}")))?;
        emulator::engine::medal_popup_update(&mut self.ctx).map_err(|fault| (fault.site(), format!("medal_popup_update::{fault}")))?;
        emulator::engine::mission_popup_update(&mut self.ctx).map_err(|fault| (fault.site(), format!("mission_popup_update::{fault}")))?;

        let shop_open = self.ctx.u8_at(AppContext::CAT_FOOD_SHOP_OPEN).unwrap_or(0) != 0;
        let tutorial_open = self.ctx.u8_at(AppContext::TUTORIAL_POPUP_OPEN).unwrap_or(0) != 0;

        let mut continued = true;

        if !(notice || shop_open || tutorial_open) {
            continued = emulator::engine::main_battle_loop(&mut self.ctx).map_err(|fault| (fault.site(), format!("main_battle_loop::{fault}")))?;
        }

        pump_stage_return(&mut self.ctx, &self.returning).map_err(|fault| (fault.site(), format!("pump_stage_return::{fault}")))?;
        self.sync_options();

        if !continued {
            return Ok(());
        }

        if tutorial_open {
            emulator::engine::tutorial_popup_update(&mut self.ctx).map_err(|fault| (fault.site(), format!("tutorial_popup_update::{fault}")))?;
        } else {
            emulator::engine::cat_food_shop_update(&mut self.ctx).map_err(|fault| (fault.site(), format!("cat_food_shop_update::{fault}")))?;

            if self.ctx.i32_at(AppContext::SHOP_UPDATE_CONSUMED).unwrap_or(0) != 0 {
                self.ctx.set_i32_at(AppContext::SHOP_UPDATE_CONSUMED, 0).map_err(|fault| (fault.site(), format!("app_on_process::{fault}")))?;
            }
        }

        if !self.in_battle() {
            return Ok(());
        }

        self.frame.borrow_mut().clear();
        emulator::engine::app_on_draw(&mut self.ctx).map_err(|fault| (fault.site(), format!("app_on_draw::{fault}")))?;
        self.stamp_frame();

        Ok(())
    }

    fn stamp_frame(&self) {
        let mut frame = self.frame.borrow_mut();

        frame.design_width = self.ctx.screen_metrics.design_w as f32;
        frame.design_height = emulator::engine::get_design_height2(&self.ctx) as f32;
        frame.letterbox = self.ctx.i32_at(AppContext::LETTERBOX_PAD).map_or(0.0, |pad| pad.max(0) as f32);
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
        let Ok(width) = emulator::engine::get_drawable_width(&self.ctx) else {
            return;
        };
        let height = emulator::engine::get_design_height2(&self.ctx);

        if let Err(fault) = emulator::engine::set_draw_origin(&mut self.ctx, 0, 0) {
            warn!("emu: dim origin could not be set: {fault}");

            return;
        }

        let dimmed = emulator::engine::draw_context(&mut self.ctx.draw).map(|sink| {
            emulator::engine::set_draw_scale(sink, 1.0);
            emulator::engine::set_tint(sink, 0, 0, 0, DIM_ALPHA);
            emulator::engine::fill_rect(sink, 0, 0, width, height);
            emulator::engine::set_tint(sink, 0xff, 0xff, 0xff, 0xff);
        });

        if let Err(fault) = dimmed {
            warn!("emu: frame could not be dimmed: {fault}");
        }
    }

    pub fn silence(&mut self) {
        if let Err(fault) = sound_manager(&mut self.ctx).and_then(sound_pause_all) {
            warn!("emu: sound could not be paused: {fault}");
        }
    }

    pub fn set_volumes(&mut self, music: i32, effects: i32) {
        let applied = sound_manager(&mut self.ctx).and_then(|mgr| {
            set_bgm_volume_setting(mgr, music);
            set_se_volume_setting(mgr, effects)
        });

        if let Err(fault) = applied {
            warn!("emu: sound volumes could not be applied: {fault}");
        }
    }

    pub fn set_options(&mut self, options: BattleOptions) {
        self.options = options;

        if let Err(fault) = apply_battle_options(&mut self.ctx, options) {
            warn!("emu: battle options could not be applied: {fault}");
        }
    }

    pub fn take_options(&mut self) -> Option<BattleOptions> {
        mem::take(&mut self.changed).then_some(self.options)
    }

    pub fn draw_curtain_over(&mut self, sweep: i32) {
        let armed = self.ctx.set_block_at::<1>(AppContext::CURTAIN_ACTIVE, [1]).and_then(|()| self.ctx.set_i32_at(AppContext::FADE_FRAME, sweep));

        if let Err(fault) = armed {
            warn!("emu: curtain frame could not be set: {fault}");

            return;
        }

        if let Err(fault) = self.paint_curtain() {
            warn!("emu: curtain draw faulted: {fault}");
        }
    }

    fn paint_curtain(&mut self) -> Result<(), emulator::Fault> {
        self.stamp_frame();

        let scale = self.ctx.screen_metrics.scale2;

        emulator::engine::set_draw_scale(emulator::engine::draw_context(&mut self.ctx.draw)?, scale);

        let top = self.ctx.i32_at(AppContext::LETTERBOX_PAD)?.wrapping_add(self.ctx.i32_at(AppContext::LETTERBOX_SHIFT)?);

        emulator::engine::set_draw_origin(&mut self.ctx, 0, top)?;
        emulator::engine::draw_screen_transition(&mut self.ctx, CURTAIN_CLOSING)?;
        emulator::engine::set_draw_origin(&mut self.ctx, 0, 0)?;

        let pad = self.ctx.i32_at(AppContext::LETTERBOX_PAD)?;

        if pad <= 0 {
            return Ok(());
        }

        let width = emulator::engine::get_drawable_width(&self.ctx)?;
        let height = emulator::engine::get_design_height2(&self.ctx);
        let sink = emulator::engine::draw_context(&mut self.ctx.draw)?;

        emulator::engine::set_tint(sink, 0, 0, 0, 0xff);
        emulator::engine::fill_rect(sink, 0, 0, width, pad);
        emulator::engine::fill_rect(sink, 0, height.wrapping_sub(pad), width, pad);
        emulator::engine::set_tint(sink, 0xff, 0xff, 0xff, 0xff);

        Ok(())
    }

    pub fn release_curtain(&mut self) {
        let cleared = self.ctx.set_block_at::<1>(AppContext::CURTAIN_ACTIVE, [0]).and_then(|()| self.ctx.set_i32_at(AppContext::FADE_FRAME, 0));

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

        let armed = self.ctx.set_block_at::<1>(AppContext::CURTAIN_ACTIVE, [1]).and_then(|()| self.ctx.set_i32_at(AppContext::FADE_FRAME, sweep));

        match armed {
            Ok(()) => {
                if let Err(fault) = self.paint_curtain() {
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
