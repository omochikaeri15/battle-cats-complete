use crate::Fault;

use super::{combo_list_rebuild, get_built_deck_rows, get_built_deck_stage_key, get_button_unit_id, get_button_unit_row, get_scene_id, has_built_deck, AppContext};

pub fn evaluate_active_combos(ctx: &mut AppContext) -> Result<(), Fault> {
    if ctx.i32_at(AppContext::SCENE_0X64_PAGE)? != 0xa {
        combo_list_rebuild(ctx, 0, 0, -1, 0)?;
    }

    let mut units = [0i32; 5];
    let mut forms = [0i32; 5];

    for (slot, unit) in units.iter_mut().enumerate() {
        *unit = get_button_unit_id(ctx, 0, slot as i32)?;
    }

    for (slot, form) in forms.iter_mut().enumerate() {
        if get_button_unit_row(ctx, 0, slot as i32)? < 2 {
            *form = -1;
            continue;
        }

        if get_scene_id(ctx)? == 0x12c && ctx.u8_at(AppContext::USE_BUILT_DECK)? != 0 {
            let stage_key = get_built_deck_stage_key(ctx)?;

            if has_built_deck(ctx, stage_key)? {
                let stage_key = get_built_deck_stage_key(ctx)?;

                *form = get_built_deck_rows(ctx, stage_key)?[slot].1 as i32;
                continue;
            }
        }

        let unit = get_button_unit_id(ctx, 0, slot as i32)?;

        *form = ctx.i32_at(AppContext::UNIT_FORMS.wrapping_add((unit as i64 as usize).wrapping_mul(4)))?;
    }

    for record in ctx.combo_store.records.iter_mut().chain(ctx.combo_store.secondary_records.iter_mut()) {
        let mut enabled = 0u8;

        if record.availability != -1 && record.state != 2 {
            let mut member = 0usize;
            let mut slot = 0usize;

            loop {
                enabled = 1;

                if record.unit[member] == -1 {
                    break;
                }

                if record.unit[member] == units[slot] && record.form[member] <= forms[slot] {
                    member += 1;
                    slot = 0;

                    if member < 5 {
                        continue;
                    }

                    break;
                }

                slot += 1;

                if slot == 5 {
                    enabled = 0;
                    break;
                }

                if member >= 5 {
                    break;
                }
            }
        }

        record.enabled = enabled;
    }

    for record in ctx.combo_store.records.iter_mut() {
        record.banner_pending = record.enabled;
    }

    Ok(())
}
