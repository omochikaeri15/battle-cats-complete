use crate::Fault;

use super::{AppContext, ENTITY_BASE, get_battle_status, prepare_battle_entry, set_keep_awake, stage_initialize};

pub fn set_scene(ctx: &mut AppContext, scene: i32) -> Result<(), Fault> {
    ctx.set_block_at::<0x28>(AppContext::DRAW_TEMP_0, [0; 0x28])?;
    ctx.set_i32_at(AppContext::SCENE_ID, scene)?;
    ctx.set_i32_at(AppContext::SCENE_ID + 4, scene)?;

    let awake =
        if (scene.wrapping_sub(0x61) as u32) <= 4 && 0x13u32 >> scene.wrapping_sub(0x61) & 1 != 0 {
            true
        } else if scene == 0x63 && ctx.i32_at(AppContext::SCENE_0X63_STATE)? != 4 {
            true
        } else if scene == 0x12c {
            (get_battle_status(ctx)? == 0 || get_battle_status(ctx)? == 3)
                && ctx.u8_at(AppContext::OPTION_MENU_IS_OPEN)? == 0
                && ctx.u8_at(AppContext::UNIT_INFO_OVERLAY_OPEN)? == 0
        } else {
            false
        };

    set_keep_awake(ctx, awake as u8)?;

    let current = ctx.i32_at(AppContext::SCENE_ID)?;

    match current {
        0x12c => stage_initialize(ctx),
        0x3e7 => {
            prepare_battle_entry(ctx)?;

            Ok(())
        }
        0x5a => {
            ctx.set_block_at::<0x30>(AppContext::SETUP_FRAMES, [0; 0x30])?;
            ctx.scene_host()
                .ok_or(Fault::host_missing())?
                .scene_setup(current);

            Ok(())
        }
        0x5e => ctx.set_block_at::<0x30>(AppContext::SETUP_FRAMES, [0; 0x30]),
        0x64 => {
            ctx.set_block_at::<0x13>(AppContext::CANNON_HELD, [0; 0x13])?;

            for offset in (0..0x18e70usize).step_by(0x10) {
                ctx.set_block_at::<0x10>(ENTITY_BASE + offset, [0; 0x10])?;
            }

            for offset in (0..0x130usize).step_by(0x10) {
                ctx.set_block_at::<0x10>(AppContext::CANNON_RECT + offset, [0; 0x10])?;
            }

            ctx.set_i32_at(AppContext::CAMERA_ZOOM, 0x2710)
        }
        0x66 => {
            ctx.scene_host()
                .ok_or(Fault::host_missing())?
                .scene_setup(current);
            ctx.set_block_at::<0x30>(AppContext::SETUP_FRAMES, [0; 0x30])
        }
        0x68 => {
            ctx.set_i32_at(AppContext::PENDING_SCENE, 0x31)?;
            ctx.set_i32_at(AppContext::PENDING_SCENE_ARMED, 1)?;
            ctx.scene_host()
                .ok_or(Fault::host_missing())?
                .scene_setup(current);

            Ok(())
        }
        4 | 5 | 0x61 | 0x62 | 0x63 | 0x65 => {
            ctx.scene_host()
                .ok_or(Fault::host_missing())?
                .scene_setup(current);

            Ok(())
        }
        _ => Ok(()),
    }
}
