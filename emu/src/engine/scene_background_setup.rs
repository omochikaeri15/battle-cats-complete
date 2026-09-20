use crate::{Fault, operation};

use super::{
    AppContext, get_button_unit_id, get_scene_id, load_unit_icon, load_unit_rig, query_localizable,
    texture_cache_load,
};

pub fn scene_background_setup(ctx: &mut AppContext) -> Result<(), Fault> {
    match get_scene_id(ctx)? {
        0x61 => {
            ctx.download_sheet =
                texture_cache_load(ctx, b"download.png", b"download.imgcut", 0x2601)?;

            Ok(())
        }
        0x64 => {
            ctx.scene_host()
                .ok_or(Fault::host_missing())?
                .scene_background_setup();

            Ok(())
        }
        0x12c => {
            for slot in 0..10 {
                ctx.unit_icon_textures[slot] = None;
            }

            for slot in 0..10usize {
                let mut pair = [0u8; 8];

                pair[..4].copy_from_slice(&ctx.block_at::<4>(AppContext::BATTLE_DECK + slot * 4)?);
                pair[4..].copy_from_slice(&ctx.block_at::<4>(AppContext::BATTLE_DECK_KEY)?);

                if operation::xor_row_decode(&pair, 1, 0).ok_or(Fault::index_out_of_range(0, 1))? as i32
                    == -1
                {
                    let png = query_localizable(ctx, b"uni.png");
                    let cut = query_localizable(ctx, b"uni.imgcut");

                    ctx.unit_icon_textures[slot] = texture_cache_load(ctx, &png, &cut, 0x2601)?;
                    continue;
                }

                if ctx.unit_icon_textures[slot].is_some() {
                    continue;
                }

                let unit = get_button_unit_id(ctx, 0, slot as i32)?;
                let form = ctx.i32_at(AppContext::BUTTON_UNIT_FORMS + slot * 4)?;

                ctx.unit_icon_textures[slot] = load_unit_icon(ctx, unit, form, 0)?;
            }

            load_unit_rig(ctx, 0)?;
            load_unit_rig(ctx, 1)?;

            Ok(())
        }
        _ => Ok(()),
    }
}
