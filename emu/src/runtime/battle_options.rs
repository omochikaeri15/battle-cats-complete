use crate::{
    Fault,
    engine::{
        AppContext, get_bgm_volume_setting, get_se_volume_setting, set_bgm_volume_setting,
        set_se_volume_setting, sound_manager,
    },
};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct BattleOptions {
    pub music: i32,
    pub effects: i32,
    pub two_rows: bool,
    pub vibrate: bool,
}

pub fn apply_battle_options(ctx: &mut AppContext, options: BattleOptions) -> Result<(), Fault> {
    set_bgm_volume_setting(sound_manager(ctx)?, options.music);
    set_se_volume_setting(sound_manager(ctx)?, options.effects);
    ctx.set_block_at::<1>(AppContext::DECK_TWO_LINES, [options.two_rows as u8])?;
    ctx.set_block_at::<1>(AppContext::VIBRATION_ENABLED, [options.vibrate as u8])
}

pub fn read_battle_options(ctx: &mut AppContext) -> Result<BattleOptions, Fault> {
    let two_rows = if ctx.u8_at(AppContext::UNIT_INFO_OVERLAY_OPEN)? != 0 {
        ctx.u8_at(AppContext::UNIT_INFO_SAVED_TWO_LINES)?
    } else {
        ctx.u8_at(AppContext::DECK_TWO_LINES)?
    };

    Ok(BattleOptions {
        music: get_bgm_volume_setting(sound_manager(ctx)?),
        effects: get_se_volume_setting(sound_manager(ctx)?),
        two_rows: two_rows != 0,
        vibrate: ctx.u8_at(AppContext::VIBRATION_ENABLED)? != 0,
    })
}
