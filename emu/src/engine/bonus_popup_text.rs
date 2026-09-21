use crate::Fault;

use super::{
    AppContext, find_item_by_kind, format_localized, format_string3_int1, get_item_name,
    query_localizable,
};

pub fn bonus_popup_text(ctx: &mut AppContext) -> Result<Vec<u8>, Fault> {
    let mut message = query_localizable(ctx, b"Castle_material_drop00");

    if (ctx
        .reward_queue
        .first()
        .ok_or(Fault::null_pointer())?
        .len() as i32)
        < 2
    {
        return Ok(message);
    }

    let mut shown = 0i32;
    let mut entry = 1i64;

    loop {
        let next_shown;

        if *ctx
            .reward_queue
            .first()
            .ok_or(Fault::null_pointer())?
            .get(entry as usize)
            .ok_or(Fault::index_out_of_range(entry, 0))?
            == 0
        {
            next_shown = shown;
        } else {
            if shown > 3 {
                break;
            }

            let item = find_item_by_kind(ctx, 7, (entry as i32).wrapping_sub(1))?;
            let item_name = get_item_name(ctx, item);
            let unit_suffix = query_localizable(ctx, b"Castle_material_drop01");
            let count = *ctx
                .reward_queue
                .first()
                .ok_or(Fault::null_pointer())?
                .get(entry as usize)
                .ok_or(Fault::index_out_of_range(entry, 0))?;

            message = format_string3_int1(
                ctx,
                b"%@ %@%@%d",
                &message,
                &item_name,
                &unit_suffix,
                count,
            )?;
            next_shown = shown.wrapping_add(1);

            if shown & 1 != 0 {
                message = format_localized(ctx, b"%@<br>", &message)?;
            }
        }

        entry = entry.wrapping_add(1);
        shown = next_shown;

        if entry
            >= ctx
                .reward_queue
                .first()
                .ok_or(Fault::null_pointer())?
                .len() as i32 as i64
        {
            break;
        }
    }

    Ok(message)
}
