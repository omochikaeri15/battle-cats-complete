use crate::Fault;

use super::{AppContext, ItemDefinition, event_items_has, get_release_point_cap};

pub fn get_item_cap(ctx: &mut AppContext, item: i32) -> Result<i32, Fault> {
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

    if ctx.i32_at(kind_at)? == 4
        || ctx.i32_at(kind_at)? == 5
        || ctx.i32_at(kind_at)? == 6
        || ctx.i32_at(kind_at)? == 7
        || ctx.i32_at(kind_at)? == 3
        || ctx.i32_at(kind_at)? == 8
    {
        return Ok(0x270f);
    }

    match item.wrapping_sub(6) as u32 {
        0x0 => return Ok(0x5f5e0ff),
        0x1 | 0xe | 0xf => return Ok(0x270f),
        0x10 => return Ok(0xf423f),
        _ => {}
    }

    if ctx.i32_at(kind_at)? == 1 {
        return Ok(0x270f);
    }

    if item > 0x7a {
        if item == 0x7b {
            return Ok(1);
        }

        if item == 0x91 || item == 0x9d {
            return Ok(0x270f);
        }
    } else {
        if item == 0x1d {
            return Ok(0x270f);
        }

        if item == 0x5c {
            return ctx.i32_at(AppContext::ITEM_5C_CAP);
        }

        if item == 0x69 {
            return Ok(0x270f);
        }
    }

    if ctx.i32_at(kind_at)? == 9 {
        return Ok(0x64);
    }

    if ctx.i32_at(kind_at)? == 0xa
        || ctx.i32_at(kind_at)? == 0xb
        || item == 0xd4
        || ctx.i32_at(kind_at)? == 0xc
    {
        return Ok(0x270f);
    }

    if item == 0xcf || item == 0xf7 {
        return Ok(0xf423f);
    }

    if event_items_has(ctx.event_items.as_ref().ok_or(Fault::null_pointer())?, item) {
        return get_release_point_cap(ctx, item);
    }

    Ok(0x270f)
}
