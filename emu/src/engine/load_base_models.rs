use std::rc::Rc;

use crate::Fault;

use super::{
    AppContext, asset_stream_open, get_castle_row, get_map_type, is_space_map, maanim_load,
    mamodel_load, mamodel_set_sheet_table, query_localizable, read_flag, string_format_int,
    string_format_int2, string_format_int3, texture_cache_load,
};

const SITE: &str = "load_base_models";

pub fn load_base_models(
    ctx: &mut AppContext,
    faction: i32,
    cannon: i32,
    foundation: i32,
    decor: i32,
) -> Result<(), Fault> {
    if read_flag(ctx, AppContext::faction_flags(faction))? & 1 != 0 {
        let parts = [cannon, cannon, foundation, decor];

        for part in 0..4 {
            if part == 1 {
                let value = parts[1];
                let png = string_format_int3(ctx, b"nyankoCastle_%03d_%02d_%02d.png", 1, value, 0)?;
                let cut =
                    string_format_int3(ctx, b"nyankoCastle_%03d_%02d_%02d.imgcut", 1, value, 0)?;

                let sheet = texture_cache_load(ctx, &png, &cut, 0x2601)?;

                ctx.base_sheets
                    .get(1)
                    .ok_or(Fault::IndexOutOfRange {
                        site: SITE,
                        index: 1,
                        limit: ctx.base_sheets.len() as i64,
                    })?
                    .set(sheet);

                let mut variant = 0;

                loop {
                    let name = string_format_int3(
                        ctx,
                        b"nyankoCastle_%03d_%02d_%02d.mamodel",
                        1,
                        value,
                        variant,
                    )?;

                    if !asset_stream_open(ctx, &name)? {
                        break;
                    }

                    let name = string_format_int3(
                        ctx,
                        b"nyankoCastle_%03d_%02d_%02d.mamodel",
                        1,
                        value,
                        variant,
                    )?;
                    let slot = variant as usize;
                    let limit = ctx.base_models.len() as i64;
                    let mut model = std::mem::take(ctx.base_models.get_mut(slot).ok_or(
                        Fault::IndexOutOfRange {
                            site: SITE,
                            index: slot as i64,
                            limit,
                        },
                    )?);

                    mamodel_load(ctx, &mut model, &name)?;
                    ctx.base_models[slot] = model;

                    let name = string_format_int3(
                        ctx,
                        b"nyankoCastle_%03d_%02d_%02d.maanim",
                        1,
                        value,
                        variant,
                    )?;
                    let limit = ctx.base_anims.len() as i64;
                    let mut anim = std::mem::take(ctx.base_anims.get_mut(slot).ok_or(
                        Fault::IndexOutOfRange {
                            site: SITE,
                            index: slot as i64,
                            limit,
                        },
                    )?);

                    maanim_load(ctx, &mut anim, &name)?;
                    ctx.base_anims[slot] = anim;

                    let table = Rc::clone(&ctx.base_sheets);

                    mamodel_set_sheet_table(&mut ctx.base_models[slot], &table);
                    variant += 1;
                }
            } else {
                let value = parts[part as usize];
                let png = string_format_int2(ctx, b"nyankoCastle_%03d_%02d.png", part, value)?;
                let cut = string_format_int2(ctx, b"nyankoCastle_%03d_%02d.imgcut", part, value)?;

                let sheet = texture_cache_load(ctx, &png, &cut, 0x2601)?;

                ctx.base_sheets
                    .get(part as usize)
                    .ok_or(Fault::IndexOutOfRange {
                        site: SITE,
                        index: part as i64,
                        limit: ctx.base_sheets.len() as i64,
                    })?
                    .set(sheet);
            }
        }

        return Ok(());
    }

    if read_flag(ctx, AppContext::faction_flags(faction))? & 2 == 0 {
        return Ok(());
    }

    let sheet = if get_map_type(ctx, 0)? == -2 || get_map_type(ctx, 0)? == -12 {
        let name = string_format_int(ctx, b"ec%03d.png", ctx.i32_at(AppContext::CASTLE_ID)?)?;
        let png = query_localizable(ctx, &name);
        let name = string_format_int(ctx, b"castle_ec%02d.imgcut", 0)?;
        let cut = query_localizable(ctx, &name);

        texture_cache_load(ctx, &png, &cut, 0x2601)?
    } else if get_map_type(ctx, 0)? == -3 || get_map_type(ctx, 0)? == -13 {
        let name = string_format_int(ctx, b"wc%03d.png", ctx.i32_at(AppContext::CASTLE_ID)?)?;
        let png = query_localizable(ctx, &name);
        let name = string_format_int(ctx, b"castle_wc%02d.imgcut", 0)?;
        let cut = query_localizable(ctx, &name);

        texture_cache_load(ctx, &png, &cut, 0x2601)?
    } else if is_space_map(ctx)? {
        let name = string_format_int(ctx, b"sc%03d.png", ctx.i32_at(AppContext::CASTLE_ID)?)?;
        let png = query_localizable(ctx, &name);
        let name = string_format_int(ctx, b"castle_wc%02d.imgcut", 0)?;
        let cut = query_localizable(ctx, &name);

        texture_cache_load(ctx, &png, &cut, 0x2601)?
    } else {
        let name = string_format_int(ctx, b"rc%03d.png", ctx.i32_at(AppContext::STAGE_CASTLE_ID)?)?;
        let png = query_localizable(ctx, &name);
        let variant =
            get_castle_row(&ctx.enemy_castle, ctx.i32_at(AppContext::CASTLE_ID)?)?.art_variant;
        let name = string_format_int(ctx, b"castle_rc%02d.imgcut", variant)?;
        let cut = query_localizable(ctx, &name);

        texture_cache_load(ctx, &png, &cut, 0x2601)?
    };

    ctx.enemy_castle_sheet = sheet;

    Ok(())
}
