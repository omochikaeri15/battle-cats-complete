use crate::Fault;

use super::{
    AppContext, ItemDefinition, event_items_count, event_items_has, get_listed_item_count,
    obf_value_read, obf_value_read_2,
};

pub fn get_item_count(ctx: &AppContext, item: i32) -> Result<i32, Fault> {
    let mut item = item;
    let mut definition;

    loop {
        definition = ((item as i64) << 6) as usize;

        let redirect = ctx.i32_at(
            AppContext::ITEM_DEFINITIONS
                .wrapping_add(definition)
                .wrapping_add(ItemDefinition::REDIRECT),
        )?;

        if redirect == -1 {
            break;
        }

        item = redirect;
    }

    let kind_at = AppContext::ITEM_DEFINITIONS
        .wrapping_add(definition)
        .wrapping_add(ItemDefinition::KIND);
    let index_at = AppContext::ITEM_DEFINITIONS
        .wrapping_add(definition)
        .wrapping_add(ItemDefinition::INDEX);

    if ctx.i32_at(kind_at)? == 4 {
        return get_listed_item_count(ctx, ctx.i32_at(index_at)?);
    }

    if ctx.i32_at(kind_at)? == 5 {
        let index = ctx.i32_at(index_at)? as i64;
        let cell =
            ctx.block_at::<8>(AppContext::ITEM_COUNTS_KIND_5.wrapping_add((index * 8) as usize))?;

        return Ok(((cell[7] ^ cell[0]) as u32
            | ((cell[6] ^ cell[1]) as u32) << 8
            | ((cell[5] ^ cell[2]) as u32) << 0x10
            | ((cell[4] ^ cell[3]) as u32) << 0x18) as i32);
    }

    if ctx.i32_at(kind_at)? == 6 {
        let index = ctx.i32_at(index_at)? as i64;
        let cell =
            ctx.block_at::<8>(AppContext::ITEM_COUNTS_KIND_6.wrapping_add((index * 8) as usize))?;

        return Ok(((cell[7] ^ cell[0]) as u32
            | ((cell[6] ^ cell[1]) as u32) << 8
            | ((cell[5] ^ cell[2]) as u32) << 0x10
            | ((cell[4] ^ cell[3]) as u32) << 0x18) as i32);
    }

    if ctx.i32_at(kind_at)? == 7 {
        let index = ctx.i32_at(index_at)? as i64;
        let cell =
            ctx.block_at::<8>(AppContext::ITEM_COUNTS_KIND_7.wrapping_add((index * 8) as usize))?;

        return Ok(((cell[7] ^ cell[0]) as u32
            | ((cell[6] ^ cell[1]) as u32) << 8
            | ((cell[5] ^ cell[2]) as u32) << 0x10
            | ((cell[4] ^ cell[3]) as u32) << 0x18) as i32);
    }

    if ctx.i32_at(kind_at)? == 3 {
        let index = ctx.i32_at(index_at)? as i64;

        return Ok(obf_value_read_2(
            &ctx.block_at::<8>(AppContext::ITEM_COUNTS_KIND_3.wrapping_add((index * 8) as usize))?,
        ) as i32);
    }

    if ctx.i32_at(kind_at)? == 8 {
        let index = ctx.i32_at(index_at)? as i64;

        return Ok(obf_value_read(
            &ctx.block_at::<8>(AppContext::ITEM_COUNTS_KIND_8.wrapping_add((index * 8) as usize))?,
        ) as i32);
    }

    match item.wrapping_sub(6) as u32 {
        0 => return Ok(obf_value_read(&ctx.block_at::<8>(AppContext::ITEM_6_COUNT)?) as i32),
        1 => return Ok(obf_value_read(&ctx.block_at::<8>(AppContext::ITEM_7_COUNT)?) as i32),
        0xe => return Ok(obf_value_read(&ctx.block_at::<8>(AppContext::ITEM_14_COUNT)?) as i32),
        0xf => return Ok(obf_value_read(&ctx.block_at::<8>(AppContext::ITEM_15_COUNT)?) as i32),
        0x10 => return Ok(obf_value_read(&ctx.block_at::<8>(AppContext::ITEM_16_COUNT)?) as i32),
        _ => {}
    }

    if ctx.i32_at(kind_at)? == 1 {
        let index = ctx.i32_at(index_at)? as i64;

        return ctx.i32_at(AppContext::ITEM_COUNTS_KIND_1.wrapping_add((index * 4) as usize));
    }

    if item > 0x7a {
        match item {
            0x7b => return Ok(ctx.i8_at(AppContext::ITEM_7B_COUNT)? as i32),
            0x91 => {
                return Ok(obf_value_read(&ctx.block_at::<8>(AppContext::ITEM_91_COUNT)?) as i32);
            }
            0x9d => {
                return Ok(obf_value_read(&ctx.block_at::<8>(AppContext::ITEM_9D_COUNT)?) as i32);
            }
            _ => {}
        }
    } else {
        match item {
            0x1d => {
                return Ok(obf_value_read(&ctx.block_at::<8>(AppContext::ITEM_1D_COUNT)?) as i32);
            }
            0x5c => {
                return Ok(obf_value_read_2(&ctx.block_at::<8>(AppContext::ITEM_5C_COUNT)?) as i32);
            }
            0x69 => return Ok(ctx.i16_at(AppContext::ITEM_69_COUNT)? as i32),
            _ => {}
        }
    }

    if ctx.i32_at(kind_at)? == 9 {
        let index = ctx.i32_at(index_at)? as i64;

        return Ok(ctx.i8_at(
            AppContext::ITEM_COUNTS_KIND_9
                .wrapping_add((index * AppContext::ITEM_COUNTS_KIND_9_STRIDE as i64) as usize),
        )? as i32);
    }

    if ctx.i32_at(kind_at)? == 0xa {
        let index = ctx.i32_at(index_at)? as u32 as u64;

        if index >= 2 {
            return Err(Fault::index_out_of_range(index as i64, 2));
        }

        return Ok(obf_value_read(
            &ctx.block_at::<8>(AppContext::ITEM_COUNTS_KIND_A.wrapping_add((index * 8) as usize))?,
        ) as i32);
    }

    if ctx.i32_at(kind_at)? == 0xb {
        let index = ctx.i32_at(index_at)? as u32 as u64;

        if index >= 4 {
            return Err(Fault::index_out_of_range(index as i64, 4));
        }

        return ctx.i32_at(AppContext::ITEM_COUNTS_KIND_B.wrapping_add((index * 4) as usize));
    }

    if ctx.i32_at(kind_at)? == 0xc {
        let index = ctx.i32_at(index_at)? as u32 as u64;

        if index >= 0x2a {
            return Err(Fault::index_out_of_range(index as i64, 0x2a));
        }

        return Ok(obf_value_read(
            &ctx.block_at::<8>(AppContext::ITEM_COUNTS_KIND_C.wrapping_add((index * 8) as usize))?,
        ) as i32);
    }

    if item == 0xf7 {
        return Ok(ctx.special_rules.item_f7_count);
    }

    if item == 0xd4 {
        return Ok(obf_value_read(&ctx.block_at::<8>(AppContext::ITEM_D4_COUNT)?) as i32);
    }

    if item == 0xcf {
        return ctx.i32_at(AppContext::ITEM_CF_COUNT);
    }

    if ctx.i32_at(kind_at)? == 0xd {
        let index = ctx.i32_at(index_at)? as i64;

        return ctx.i32_at(AppContext::ITEM_COUNTS_KIND_D.wrapping_add((index * 4) as usize));
    }

    let store = ctx
        .event_items
        .as_ref()
        .ok_or(Fault::null_pointer())?;

    if !event_items_has(store, item) {
        return Ok(0);
    }

    Ok(event_items_count(store, item))
}
