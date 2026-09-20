use std::cell::{Cell, RefCell};
use std::rc::Rc;

use emu::engine::AppContext;
use emu::runtime::{
    BattleOptions, DeviceProfile, InertMeta, InertPlatform, InertScene, InertUi, apply_battle_options,
    fill_dummy_save, fill_dummy_talents, load_scene_sheets,
    pump_stage_return, read_battle_options, relatch_battle_rects, stock_battle_items, unlock_dummy_combos,
};
use kore::Vfs;
use tracing::{info, warn};

use super::assets::{DiskAssets, FileIndex, SheetCache};
use super::input::{Touch, TouchQueue};
use super::sink::{Frame, Recorder};
use super::sound::{SharedVolumes, Speaker, Volumes};
use super::text::Formatter;

const CURTAIN_CLOSING: i32 = 1;
const IDENTITY: [f32; 6] = [1.0, 0.0, 0.0, 1.0, 0.0, 0.0];
const BATTLE_SCENE: i32 = 0x12c;
const TRANSITION_SCENE: i32 = 0x3e7;
const FADE_STEPS: usize = 0x10;
const FINGER_GAP: i32 = 400;
const FULL_VOLUME: i32 = 100;
const DIM_ALPHA: i32 = 0x80;
const MAP_BATTLE_MODE: i32 = 3;
const STAGE_CATEGORY: i32 = -16;
const STAGE_MAP: i32 = 52;
const STAGE_INDEX: i32 = 9;
const CAT_SIDE: i32 = 1;
const ENEMY_SIDE: i32 = 2;
const NOTCH_SHARE: f32 = 0.04;

pub struct Driver {
    ctx: Box<AppContext>,
    frame: Rc<RefCell<Frame>>,
    sheets: Rc<RefCell<SheetCache>>,
    files: Rc<RefCell<FileIndex>>,
    touches: TouchQueue,
    volumes: SharedVolumes,
    options: BattleOptions,
    changed: bool,
    returning: Rc<Cell<bool>>,
    profile: Rc<DeviceProfile>,
    phone: bool,
    spread: i32,
    gap: Option<i32>,
    booted: bool,
}

impl Driver {
    pub fn new() -> Self {
        let frame = Rc::new(RefCell::new(Frame::default()));
        let sheets: Rc<RefCell<SheetCache>> = Rc::new(RefCell::new(SheetCache::new()));
        let mut ctx = Box::new(AppContext::default());

        let files: Rc<RefCell<FileIndex>> = Rc::new(RefCell::new(FileIndex::new()));
        let volumes: SharedVolumes = Rc::new(RefCell::new(Volumes {
            music: FULL_VOLUME,
            effects: FULL_VOLUME,
        }));

        ctx.set_assets(Box::new(DiskAssets::new(Rc::clone(&files), Rc::clone(&sheets))));
        let profile = Rc::new(DeviceProfile::default());

        profile.tablet.set(true);
        ctx.set_platform(Box::new(InertPlatform {
            profile: Rc::clone(&profile),
        }));
        ctx.set_sound(Box::new(Speaker::new(Rc::clone(&files), Rc::clone(&volumes))));
        ctx.set_meta(Box::new(InertMeta));
        let returning = Rc::new(Cell::new(false));

        ctx.set_scene_host(Box::new(InertScene {
            returning: Rc::clone(&returning),
        }));
        ctx.set_text_renderer(Box::new(Formatter::new(Rc::clone(&sheets))));
        ctx.set_ui(Box::new(InertUi));
        ctx.draw = Some(Box::new(Recorder::new(Rc::clone(&frame))));

        if let Err(fault) = fill_dummy_save(&mut ctx) {
            warn!("emu: dummy save could not be filled: {fault}");
        }

        Self {
            ctx,
            frame,
            sheets,
            files,
            touches: super::input::queue(),
            volumes,
            options: BattleOptions {
                music: FULL_VOLUME,
                effects: FULL_VOLUME,
                two_rows: false,
                vibrate: false,
            },
            changed: false,
            returning,
            profile,
            phone: false,
            spread: 0,
            gap: None,
            booted: false,
        }
    }

    pub fn resolved(&self) -> usize {
        self.files.borrow().len()
    }

    pub fn index_assets(&mut self, vfs: &Vfs) {
        if self.files.borrow().is_empty() {
            *self.files.borrow_mut() = DiskAssets::index(vfs);
        }
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

    fn pump_input(&mut self) {
        let pending: Vec<Touch> = self.touches.borrow_mut().drain(..).collect();

        for touch in pending {
            let fed = match touch {
                Touch::Pinched { spread: step } => {
                    self.spread = self.spread.wrapping_add(step);

                    Ok(())
                }
                Touch::Moved { x, y } => {
                    emu::runtime::queue_touch_position(&mut self.ctx, x, y)
                }
                Touch::Pressed { x, y } => emu::runtime::queue_touch_press(&mut self.ctx, x, y),
                Touch::Released => emu::runtime::queue_touch_release(&mut self.ctx),
            };

            if let Err(fault) = fed {
                warn!("emu: touch could not be queued: {fault}");
            }
        }

        self.gap = match (self.spread, self.gap) {
            (0, _) => None,
            (_, None) => Some(FINGER_GAP),
            (spread, Some(gap)) => {
                self.spread = 0;

                Some(gap.wrapping_add(spread).max(1))
            }
        };

        if let Err(fault) = emu::runtime::pump_pinch(&mut self.ctx, self.gap) {
            warn!("emu: pinch could not be pumped: {fault}");
        }

        if let Err(fault) = emu::runtime::pump_touch(&mut self.ctx) {
            warn!("emu: touch could not be pumped: {fault}");
        }
    }

    pub fn resize(&mut self, width: f32, height: f32) {
        if width < 1.0 || height < 1.0 {
            return;
        }

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
        self.phone = phone;

        let width = self.ctx.device_screen_w as f32;
        let height = self.ctx.device_screen_h as f32;

        self.resize(width, height);
    }

    pub fn design_height(&self) -> f32 {
        self.ctx.screen_metrics.design_h2 as f32
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

        if let Err(fault) = emu::engine::load_misc_data_tables(&mut self.ctx) {
            warn!("emu: text tables failed to load: {fault}");

            return false;
        }

        match emu::engine::initialize_game_data(&mut self.ctx) {
            Ok(true) => {
                unlock_dummy_combos(&mut self.ctx);

                fill_dummy_talents(&mut self.ctx);

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
        if let Err(fault) = stock_battle_items(&mut self.ctx) {
            warn!("emu: battle items could not be stocked: {fault}");
        }

        if let Err(fault) = self.select_stage() {
            warn!("emu: stage could not be selected: {fault}");

            return false;
        }

        match emu::engine::prepare_battle_entry(&mut self.ctx) {
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
        self.ctx
            .set_i32_at(AppContext::CHAPTER_MODE, MAP_BATTLE_MODE)?;
        self.ctx.set_i32_at(
            AppContext::SAVED_MAP_TYPE,
            emu::engine::map_type_as_index(STAGE_CATEGORY),
        )?;
        self.ctx.set_i32_at(AppContext::MAP_INDEX, STAGE_MAP)?;
        self.ctx.set_i32_at(AppContext::ENTRY_STAGE, STAGE_INDEX)?;
        self.ctx
            .set_i32_at(AppContext::faction_flags(0), CAT_SIDE)?;
        self.ctx
            .set_i32_at(AppContext::faction_flags(1), ENEMY_SIDE)
    }

    pub fn advance(&mut self) -> Result<(), String> {
        if !self.in_battle() {
            return Ok(());
        }

        self.pump_input();
        emu::engine::dialog_manager_process(&mut self.ctx)
            .map_err(|fault| format!("dialog_manager_process:{fault}"))?;

        emu::engine::button_bank_process(&mut self.ctx)
            .map_err(|fault| format!("button_bank_process:{fault}"))?;
        emu::engine::main_battle_loop(&mut self.ctx)
            .map_err(|fault| format!("main_battle_loop:{fault}"))?;
        pump_stage_return(&mut self.ctx, &self.returning)
            .map_err(|fault| format!("pump_stage_return:{fault}"))?;
        self.sync_options();

        if !self.in_battle() {
            return Ok(());
        }

        self.frame.borrow_mut().clear();
        self.begin_draw();
        emu::engine::main_draw(&mut self.ctx, 0).map_err(|fault| format!("main_draw:{fault}"))?;
        emu::engine::dialog_manager_draw(&mut self.ctx)
            .map_err(|fault| format!("dialog_manager_draw:{fault}"))?;
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
        std::mem::take(&mut self.changed).then_some(self.options)
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
        self.frame.borrow_mut().clear();
        self.begin_draw();

        let armed = self
            .ctx
            .set_block_at::<1>(AppContext::CURTAIN_ACTIVE, [1])
            .and_then(|()| self.ctx.set_i32_at(AppContext::FADE_FRAME, sweep));

        if let Err(fault) = armed {
            warn!("emu: curtain frame could not be set: {fault}");

            return;
        }

        if let Err(fault) = emu::engine::draw_screen_transition(&mut self.ctx, CURTAIN_CLOSING) {
            warn!("emu: curtain draw faulted: {fault}");
        }

        self.draw_letterbox_bars();
    }
}
