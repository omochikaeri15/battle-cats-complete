use crate::Fault;

use super::{
    add_behemoth_dodge_timer, add_curse_timer, add_dodge_fx_frame, add_dodge_timer, add_freeze_timer, add_orb_dodge_timer,
    add_slow_timer, add_weaken_timer, get_anim_len, get_behemoth_dodge_timer, get_curse_timer, get_dodge_fx_frame,
    get_dodge_timer, get_freeze_timer, get_orb_dodge_timer, get_slow_timer, get_status_bits, get_weaken_timer,
    maanim_get_max_keyframe, set_curse_length, set_dodge_fx_frame, set_freeze_length, set_kb_proc_hit, set_slow_length,
    set_weaken_active, turn_off_proc_badge, turn_on_proc_badge, AppContext, Entity, Maanim,
};

#[derive(Default)]
pub struct BattleEffects {
    pub wave_immune_anim: Maanim,
    pub wave_block_anim: Maanim,
    pub survive_anim: Maanim,
    pub immune_anim: Maanim,
}

pub fn proc_update(ctx: &mut AppContext, effects: &BattleEffects) -> Result<(), Fault> {
    for faction in 0..2i32 {
        for slot in 0..0x33i32 {
            set_kb_proc_hit(ctx, faction, slot, 0)?;

            if get_dodge_timer(ctx, faction, slot)? > 0 {
                add_dodge_timer(ctx, faction, slot, -1)?;
                turn_on_proc_badge(ctx, faction, slot, 5)?;

                if get_dodge_timer(ctx, faction, slot)? == 0 {
                    set_dodge_fx_frame(ctx, faction, slot, 0)?;
                }
            }

            if get_behemoth_dodge_timer(ctx, faction, slot)? > 0 {
                add_behemoth_dodge_timer(ctx, faction, slot, -1)?;
                turn_on_proc_badge(ctx, faction, slot, 5)?;

                if get_behemoth_dodge_timer(ctx, faction, slot)? == 0 {
                    set_dodge_fx_frame(ctx, faction, slot, 0)?;
                }
            }

            if get_orb_dodge_timer(ctx, faction, slot)? > 0 {
                add_orb_dodge_timer(ctx, faction, slot, -1)?;
                turn_on_proc_badge(ctx, faction, slot, 5)?;

                if get_orb_dodge_timer(ctx, faction, slot)? == 0 {
                    set_dodge_fx_frame(ctx, faction, slot, 0)?;
                }
            }

            if get_freeze_timer(ctx, faction, slot)? > 0 {
                add_freeze_timer(ctx, faction, slot, -1)?;

                if get_freeze_timer(ctx, faction, slot)? == 0 {
                    set_freeze_length(ctx, faction, slot, 0)?;
                }
            }

            if get_slow_timer(ctx, faction, slot)? > 0 {
                add_slow_timer(ctx, faction, slot, -1)?;

                if get_slow_timer(ctx, faction, slot)? == 0 {
                    set_slow_length(ctx, faction, slot, 0)?;
                }
            }

            if get_freeze_timer(ctx, faction, slot)? == 0 && get_slow_timer(ctx, faction, slot)? == 0 {
                turn_off_proc_badge(ctx, faction, slot, 1)?;
            }

            if get_weaken_timer(ctx, faction, slot)? > 0 {
                add_weaken_timer(ctx, faction, slot, -1)?;

                if get_weaken_timer(ctx, faction, slot)? == 0 {
                    set_weaken_active(ctx, faction, slot, 0)?;
                }
            }

            let status = get_status_bits(ctx, faction, slot)?;
            let aura = AppContext::entity_field(faction, slot, Entity::ATTACK_UP_FX_FRAME);

            if status > 0 {
                ctx.set_i32_at(aura, ctx.i32_at(aura)?.wrapping_add(1))?;
                turn_on_proc_badge(ctx, faction, slot, 2)?;
            } else {
                ctx.set_i32_at(aura, 0)?;
            }

            if get_weaken_timer(ctx, faction, slot)? | status == 0 {
                turn_off_proc_badge(ctx, faction, slot, 2)?;
            }

            if ctx.i32_at(AppContext::entity_field(faction, slot, Entity::SURVIVE_USED))? > 0 {
                let at = AppContext::entity_field(faction, slot, Entity::SURVIVE_FX_FRAME);
                ctx.set_i32_at(at, ctx.i32_at(at)?.wrapping_add(1))?;

                let length = get_anim_len(&effects.survive_anim)?;
                let frame = ctx.i32_at(at)?;
                let limit = if length == -1 { maanim_get_max_keyframe(&effects.survive_anim)? } else { get_anim_len(&effects.survive_anim)? };

                if frame >= limit {
                    turn_off_proc_badge(ctx, faction, slot, 3)?;
                }
            }

            if ctx.i32_at(AppContext::entity_field(faction, slot, Entity::WAVE_IMMUNE_FX_ACTIVE))? > 0 {
                let at = AppContext::entity_field(faction, slot, Entity::WAVE_IMMUNE_FX_FRAME);
                ctx.set_i32_at(at, ctx.i32_at(at)?.wrapping_add(1))?;

                let length = get_anim_len(&effects.survive_anim)?;
                let frame = ctx.i32_at(at)?;
                let limit = if length == -1 { maanim_get_max_keyframe(&effects.wave_immune_anim)? } else { get_anim_len(&effects.wave_immune_anim)? };

                if frame >= limit {
                    ctx.set_i32_at(AppContext::entity_field(faction, slot, Entity::WAVE_IMMUNE_FX_FRAME), 0)?;
                    ctx.set_i32_at(AppContext::entity_field(faction, slot, Entity::WAVE_IMMUNE_FX_ACTIVE), 0)?;
                }
            }

            if ctx.i32_at(AppContext::entity_field(faction, slot, Entity::WAVE_BLOCK_FX_ACTIVE))? > 0 {
                let at = AppContext::entity_field(faction, slot, Entity::WAVE_BLOCK_FX_FRAME);
                ctx.set_i32_at(at, ctx.i32_at(at)?.wrapping_add(1))?;

                let length = get_anim_len(&effects.survive_anim)?;
                let frame = ctx.i32_at(at)?;
                let limit = if length == -1 { maanim_get_max_keyframe(&effects.wave_block_anim)? } else { get_anim_len(&effects.wave_block_anim)? };

                if frame >= limit {
                    ctx.set_i32_at(AppContext::entity_field(faction, slot, Entity::WAVE_BLOCK_FX_FRAME), 0)?;
                    ctx.set_i32_at(AppContext::entity_field(faction, slot, Entity::WAVE_BLOCK_FX_ACTIVE), 0)?;
                }
            }

            if ctx.i32_at(AppContext::entity_field(faction, slot, Entity::IMMUNE_FX_ACTIVE))? > 0 {
                let at = AppContext::entity_field(faction, slot, Entity::IMMUNE_FX_FRAME);
                ctx.set_i32_at(at, ctx.i32_at(at)?.wrapping_add(1))?;

                let length = get_anim_len(&effects.immune_anim)?;
                let frame = ctx.i32_at(at)?;
                let limit = if length == -1 { maanim_get_max_keyframe(&effects.immune_anim)? } else { get_anim_len(&effects.immune_anim)? };

                if frame >= limit {
                    ctx.set_i32_at(AppContext::entity_field(faction, slot, Entity::IMMUNE_FX_FRAME), 0)?;
                    ctx.set_i32_at(AppContext::entity_field(faction, slot, Entity::IMMUNE_FX_ACTIVE), 0)?;
                }
            }

            if get_dodge_fx_frame(ctx, faction, slot)? > 0 {
                add_dodge_fx_frame(ctx, faction, slot, 1)?;
            }

            if ctx.i32_at(AppContext::entity_field(faction, slot, Entity::DOUBLE_BOUNTY_STATE))? == 1 {
                ctx.set_i32_at(AppContext::entity_field(faction, slot, Entity::DOUBLE_BOUNTY_STATE), 0)?;
            }

            if get_curse_timer(ctx, faction, slot)? > 0 {
                add_curse_timer(ctx, faction, slot, -1)?;

                if get_curse_timer(ctx, faction, slot)? == 0 {
                    set_curse_length(ctx, faction, slot, 0)?;
                    turn_off_proc_badge(ctx, faction, slot, 4)?;
                }
            }
        }
    }

    Ok(())
}
