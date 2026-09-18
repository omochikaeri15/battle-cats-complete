use crate::Fault;

use super::{get_scene_id, has_fixed_lineup, has_talent, is_trait_targeted_talent, std_map_int_int_from_list, AppContext};

pub fn talent_targets_trait(
    ctx: &mut AppContext,
    team: i32,
    unit_id: i32,
    form: i32,
    trait_bit: i32,
) -> Result<bool, Fault> {
    if !ctx.talent_definitions.contains_key(&unit_id) {
        return Ok(false);
    }

    if form < 2 {
        return Ok(false);
    }

    if get_scene_id(ctx)? == 0x12c && has_fixed_lineup(ctx, -1, -1, -1)? {
        return Ok(false);
    }

    let mut target_talents = std_map_int_int_from_list(&[
        (0x1, 0x21),
        (0x2, 0x22),
        (0x4, 0x23),
        (0x8, 0x24),
        (0x10, 0x25),
        (0x20, 0x26),
        (0x40, 0x27),
        (0x80, 0x28),
        (0x100, 0x29),
        (0x200, 0x2a),
        (0x400, 0x2b),
        (0x800, 0x39),
    ]);

    if target_talents.contains_key(&trait_bit) {
        let abil = *target_talents.entry(trait_bit).or_default();

        if has_talent(ctx, team, unit_id, form, abil)? {
            return Ok(true);
        }
    }

    let mut talent_slot = 0;

    loop {
        let definition = ctx.talent_definitions.entry(unit_id).or_insert([0; 0x71]);

        if is_trait_targeted_talent(definition[talent_slot * 0xe + 1]) {
            let levels = ctx.talent_levels.entry(unit_id).or_default();
            let definition = ctx.talent_definitions.entry(unit_id).or_insert([0; 0x71]);
            let level = levels.entry(definition[talent_slot * 0xe + 1]).or_default();

            if *level > 0 {
                let definition = ctx.talent_definitions.entry(unit_id).or_insert([0; 0x71]);

                return Ok(definition[0] & trait_bit != 0);
            }
        }

        talent_slot += 1;

        if talent_slot == 8 {
            return Ok(false);
        }
    }
}
