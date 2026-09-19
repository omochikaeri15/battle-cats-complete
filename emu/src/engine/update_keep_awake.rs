use crate::Fault;

use super::{AppContext, get_battle_status, set_keep_awake};

#[allow(clippy::if_same_then_else)]
pub fn update_keep_awake(ctx: &mut AppContext, enable: u8) -> Result<(), Fault> {
    let scene = ctx.i32_at(AppContext::SCENE_ID)?;
    let menu = scene.wrapping_sub(0x61) as u32;

    if menu <= 4 && (0x13u32 >> menu) & 1 != 0 {
        return set_keep_awake(ctx, enable);
    }

    if scene == 0x63 && ctx.i32_at(AppContext::SCENE_0X63_STATE)? != 4 {
        return set_keep_awake(ctx, enable);
    }

    let mut awake = 0u8;

    if scene == 0x12c {
        let running = if get_battle_status(ctx)? == 0 {
            ctx.u8_at(AppContext::OPTION_MENU_IS_OPEN)? == 0
        } else if get_battle_status(ctx)? == 3 {
            ctx.u8_at(AppContext::OPTION_MENU_IS_OPEN)? == 0
        } else {
            false
        };

        if running && ctx.u8_at(AppContext::UNIT_INFO_OVERLAY_OPEN)? == 0 {
            awake = enable;
        }
    }

    set_keep_awake(ctx, awake)
}
