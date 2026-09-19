use crate::Fault;

use super::{
    AppContext, build_trait_mask, get_unit_form, stat_massive_damage, stat_resist,
    stat_strong_against, stat_surge_immune, trait_aku, trait_alien, trait_angel, trait_dark,
    trait_eva, trait_floating, trait_metal, trait_red, trait_relic, trait_traitless, trait_witch,
    trait_zombie,
};

const SITE: &str = "orb_applies_to_unit";

pub fn orb_applies_to_unit(
    ctx: &mut AppContext,
    abil: i32,
    trait_index: i32,
    unit_id: i32,
) -> Result<i32, Fault> {
    if abil == 0x11 {
        let form = get_unit_form(ctx, unit_id)?;

        if stat_surge_immune(ctx, 0, unit_id, form)? {
            return Ok(4);
        }
    }

    if (abil.wrapping_add(-2) as u32) > 2 {
        return Ok(0);
    }

    let owned =
        ctx.i32_at(AppContext::REWARD_FORMS_OWNED.wrapping_add(((unit_id as i64) * 4) as usize))?;
    let forms = i32::from(owned == 2).wrapping_add(3);
    let mut result = 1;
    let mut form = 2;

    while form != forms {
        let matched = match abil {
            2 => stat_strong_against(ctx, 0, unit_id, form)?,
            3 => stat_massive_damage(ctx, 0, unit_id, form)?,
            4 => stat_resist(ctx, 0, unit_id, form)?,
            _ => false,
        };

        if !matched {
            form += 1;

            continue;
        }

        let traits = build_trait_mask(&[
            (0x1, trait_red(ctx, 0, unit_id, form, 1)?),
            (0x2, trait_floating(ctx, 0, unit_id, form, 1)?),
            (0x4, trait_dark(ctx, 0, unit_id, form, 1)?),
            (0x8, trait_metal(ctx, 0, unit_id, form, 1)?),
            (0x10, trait_angel(ctx, 0, unit_id, form, 1)?),
            (0x20, trait_alien(ctx, 0, unit_id, form, 1)?),
            (0x40, trait_zombie(ctx, 0, unit_id, form, 1)?),
            (0x80, trait_relic(ctx, 0, unit_id, form, 1)?),
            (0x100, trait_traitless(ctx, 0, unit_id, form, 1)?),
            (0x200, trait_witch(ctx, 0, unit_id, form, 1)?),
            (0x400, trait_eva(ctx, 0, unit_id, form, 1)?),
            (0x800, trait_aku(ctx, 0, unit_id, form, 1)?),
        ]);
        let mask = *ctx
            .orb_store
            .trait_masks
            .get(trait_index as i64 as usize)
            .ok_or(Fault::IndexOutOfRange {
                site: SITE,
                index: trait_index as i64,
                limit: ctx.orb_store.trait_masks.len() as i64,
            })?;
        let mut hit = false;

        for (bit, present) in traits.iter() {
            if *present && bit & mask != 0 {
                hit = true;

                break;
            }
        }

        if hit {
            result = 2;

            if form == get_unit_form(ctx, unit_id)? {
                return Ok(3);
            }
        }

        form += 1;
    }

    Ok(result)
}
