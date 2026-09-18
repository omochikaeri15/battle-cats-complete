use crate::Fault;

use super::{
    get_button_unit_form, get_entity_button, max_i32, read_flag, trait_aku, trait_alien, trait_angel, trait_dark, trait_eva, trait_floating, trait_metal, trait_red, trait_relic, trait_traitless, trait_witch, trait_zombie,
    AppContext, Entity,
};

pub fn count_target_traits(ctx: &mut AppContext, faction: i32, slot: i32) -> Result<i32, Fault> {
    if read_flag(ctx, AppContext::faction_flags(faction))? & 1 == 0 {
        return Ok(1);
    }

    let unit_id = ctx.i32_at(AppContext::entity_field(faction, slot, Entity::OCCUPANT))?.wrapping_add(-2);
    let form = get_button_unit_form(ctx, faction, get_entity_button(ctx, faction, slot)?)?;
    let red = trait_red(ctx, faction, unit_id, form, 0)? as i32;
    let floating = trait_floating(ctx, faction, unit_id, form, 0)? as i32;
    let dark = trait_dark(ctx, faction, unit_id, form, 0)? as i32;
    let metal = trait_metal(ctx, faction, unit_id, form, 0)? as i32;
    let traitless = trait_traitless(ctx, faction, unit_id, form, 0)? as i32;
    let angel = trait_angel(ctx, faction, unit_id, form, 0)? as i32;
    let alien = trait_alien(ctx, faction, unit_id, form, 0)? as i32;
    let zombie = trait_zombie(ctx, faction, unit_id, form, 0)? as i32;
    let witch = trait_witch(ctx, faction, unit_id, form, 0)? as i32;
    let eva = trait_eva(ctx, faction, unit_id, form, 0)? as i32;
    let relic = trait_relic(ctx, faction, unit_id, form, 0)? as i32;
    let aku = trait_aku(ctx, faction, unit_id, form, 0)? as i32;

    Ok(max_i32(1, red.wrapping_add(floating).wrapping_add(dark).wrapping_add(metal).wrapping_add(traitless).wrapping_add(angel).wrapping_add(alien).wrapping_add(zombie).wrapping_add(witch).wrapping_add(eva).wrapping_add(relic).wrapping_add(aku)))
}
