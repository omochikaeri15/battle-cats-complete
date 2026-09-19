use crate::Fault;

use super::{new_button_hit_test, web_view_is_open, AppContext};

const SITE: &str = "button_bank_process";

pub fn button_bank_process(ctx: &mut AppContext) -> Result<(), Fault> {
    if ctx.buttons.enabled == 0 {
        return Ok(());
    }

    if web_view_is_open(ctx)? {
        return Ok(());
    }

    if ctx.buttons.buttons.is_empty() {
        return Ok(());
    }

    let mut busy = 0u8;

    for slot in ctx.buttons.buttons.values() {
        let button = slot.as_deref().ok_or(Fault::NullPointer { site: SITE })?;

        if button.enabled != 0 && button.state == 1 {
            busy = 1;
            break;
        }
    }

    let ids: Vec<i32> = ctx.buttons.buttons.keys().copied().collect();

    for id in ids {
        if !ctx.buttons.buttons.contains_key(&id) {
            continue;
        }

        new_button_hit_test(ctx, id, busy & 1)?;

        let state = ctx.buttons.buttons.entry(id).or_insert(None).as_deref().ok_or(Fault::NullPointer { site: SITE })?.state;

        if state == 1 {
            busy = 1;
        }
    }

    Ok(())
}
