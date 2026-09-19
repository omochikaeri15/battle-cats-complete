use crate::{operation, Fault};

use super::{
    add_money, call_rng, cannon_fire, cannon_reach_x, cannon_target_in_range, deck_slot_filled, deploy_unit, get_button_unit_row, get_castle_anim_state, get_deck_cooldown,
    get_effective_deploy_cost, get_max_money, get_money, get_worker_level, get_worker_upgrade_cost, get_worker_upgrade_level, is_deploy_blocked, play_sound,
    recount_deploy_rarities, slot_conjure_ready, slot_deploy_permitted, slot_occupied, sound_manager, upgrade_worker, AppContext, Base, Entity,
};

pub fn cat_cpu_tick(ctx: &mut AppContext, faction: i32) -> Result<(), Fault> {
    let side = faction as i64 as usize;

    if ctx.u8_at(AppContext::CPU_ENABLED.wrapping_add(side))? == 0 {
        return Ok(());
    }

    recount_deploy_rarities(ctx)?;

    let action = AppContext::CPU_PENDING_ACTION.wrapping_add(side.wrapping_mul(4));
    let pick = AppContext::CPU_PICK.wrapping_add(side.wrapping_mul(4));
    let usable = AppContext::CPU_USABLE.wrapping_add(side.wrapping_mul(AppContext::CPU_FACTION_STRIDE));
    let candidates = AppContext::CPU_CANDIDATES.wrapping_add(side.wrapping_mul(AppContext::CPU_FACTION_STRIDE));
    let wallet = AppContext::faction_flags(faction);
    let other = 1i32.wrapping_sub(faction);

    'decide: {
        if ctx.i32_at(AppContext::DEPLOY_NOTICE_TIMER)? > 0 {
            ctx.set_i32_at(action, 0)?;
        } else if ctx.i32_at(action)? != 0 {
            break 'decide;
        }

        let mut cell = 0usize;

        while cell != 10 {
            ctx.set_i32_at(candidates.wrapping_add(cell.wrapping_mul(4)), -1)?;
            cell += 1;
        }

        let mut full = false;

        if slot_occupied(ctx, faction, 1)? != 0 {
            let mut scan = 2i32;

            while scan != 51 && slot_occupied(ctx, faction, scan)? != 0 {
                scan += 1;
            }

            full = scan.wrapping_sub(1) as u32 >= 0x32;
        }

        let mut button = 0i32;

        while button != 10 {
            let open = deck_slot_filled(ctx, faction, button)?
                && slot_deploy_permitted(ctx, faction, button, 1)?
                && !is_deploy_blocked(ctx, button)?
                && !slot_conjure_ready(ctx, faction, button)?
                && !((get_deck_cooldown(ctx, wallet, button)? != 0) | full);

            ctx.set_i32_at(usable.wrapping_add((button as usize).wrapping_mul(4)), if open { button } else { -1 })?;
            button += 1;
        }

        let roll = call_rng(ctx, 0x64);
        let next;

        if roll < 0x1e || ctx.i32_at(AppContext::DEPLOY_NOTICE_TIMER)? == 1 {
            if get_worker_level(ctx, wallet)? != 7 {
                next = 2;
            } else {
                let mut listed = 0i32;
                let mut button = 0i32;

                while button != 10 {
                    if ctx.i32_at(usable.wrapping_add((button as usize).wrapping_mul(4)))? != -1
                        && deck_slot_filled(ctx, faction, button)?
                        && slot_deploy_permitted(ctx, faction, button, 1)?
                        && !is_deploy_blocked(ctx, button)?
                        && get_max_money(ctx, wallet)? >= get_effective_deploy_cost(ctx, faction, button)?
                    {
                        ctx.set_i32_at(candidates.wrapping_add((listed as i64 as usize).wrapping_mul(4)), button)?;
                        listed = listed.wrapping_add(1);
                    }

                    button += 1;
                }

                if listed == 0 {
                    next = 0;
                } else if ctx.i32_at(AppContext::DEPLOY_NOTICE_TIMER)? != 0 {
                    break 'decide;
                } else {
                    let chosen = call_rng(ctx, listed);

                    ctx.set_i32_at(pick, chosen)?;
                    next = 3;
                }
            }
        } else {
            let mut listed = 0u32;
            let mut button = 0i32;

            while button != 10 {
                if ctx.i32_at(usable.wrapping_add((button as usize).wrapping_mul(4)))? != -1 {
                    ctx.set_i32_at(candidates.wrapping_add((listed as usize).wrapping_mul(4)), button)?;
                    listed = listed.wrapping_add(1);
                }

                button += 1;
            }

            'worker: {
                if listed == 0 {
                    ctx.set_i32_at(action, 0)?;

                    break 'worker;
                }

                if ctx.i32_at(AppContext::DEPLOY_NOTICE_TIMER)? == 0 {
                    let chosen = call_rng(ctx, listed as i32);

                    ctx.set_i32_at(pick, chosen)?;
                    ctx.set_i32_at(action, 3)?;
                } else if ctx.i32_at(action)? == 0 {
                    break 'worker;
                }

                let budget = get_max_money(ctx, wallet)?;
                let chosen = ctx.i32_at(pick)? as i64;
                let target = ctx.i32_at(candidates.wrapping_add((chosen * 4) as usize))?;

                if budget >= get_effective_deploy_cost(ctx, faction, target)? {
                    break 'decide;
                }

                ctx.set_i32_at(action, if get_worker_level(ctx, wallet)? != 7 { 2 } else { 0 })?;

                break 'decide;
            }

            if get_worker_level(ctx, wallet)? == 7 {
                next = 0;
            } else if get_worker_upgrade_level(ctx)? <= 6 {
                break 'decide;
            } else {
                next = 2;
            }
        }

        ctx.set_i32_at(action, next)?;
    }

    let cannon_state = AppContext::CPU_CANNON_STATE.wrapping_add(side.wrapping_mul(4));
    let countdown = AppContext::entity_field(faction, 0, Base::CANNON_COUNTDOWN);

    if ctx.i32_at(countdown)? == 0 && get_castle_anim_state(ctx, faction)? == 0 {
        let mut slot = 1i32;

        while slot != 51 {
            if ctx.i32_at(AppContext::entity_field(other, slot, Entity::OCCUPANT))? != 0 && ctx.i32_at(AppContext::entity_field(other, slot, Entity::STATE))? != 4 {
                let screen_x = operation::div_10(ctx.i32_at(AppContext::entity_field(other, slot, Entity::POS_X))?.wrapping_sub(ctx.i32_at(AppContext::CAMERA_X)?));

                ctx.set_i32_at(AppContext::SCRATCH_2, screen_x)?;

                if cannon_target_in_range(ctx, faction, screen_x)? && ctx.i32_at(cannon_state)? == 0 {
                    ctx.set_i32_at(cannon_state, 1)?;
                }
            }

            slot += 1;
        }

        let mut slot = 1i32;

        while slot != 51 {
            let reach = cannon_reach_x(ctx, faction)?;

            ctx.set_i32_at(AppContext::SCRATCH_1, reach)?;

            if ctx.i32_at(AppContext::entity_field(other, slot, Entity::OCCUPANT))? != 0 {
                let screen_x = operation::div_10(ctx.i32_at(AppContext::entity_field(other, slot, Entity::POS_X))?.wrapping_sub(ctx.i32_at(AppContext::CAMERA_X)?));

                ctx.set_i32_at(AppContext::SCRATCH_2, screen_x)?;

                let close = match faction {
                    1 => screen_x < reach.wrapping_add(-0xc8),
                    0 => screen_x > reach.wrapping_add(-0xc8),
                    _ => false,
                };

                if close {
                    ctx.set_i32_at(action, 1)?;
                    ctx.set_i32_at(cannon_state, 0)?;
                }
            }

            slot += 1;
        }
    }

    'cannon: {
        let next = match ctx.i32_at(cannon_state)? {
            1 => {
                let wait = call_rng(ctx, 0x96);

                ctx.set_i32_at(AppContext::CPU_CANNON_WAIT.wrapping_add(side.wrapping_mul(8)), wait)?;

                2
            }
            2 => {
                let wait = ctx.i32_at(AppContext::CPU_CANNON_WAIT.wrapping_add(side.wrapping_mul(8)))?;

                ctx.set_i32_at(AppContext::CPU_CANNON_WAIT.wrapping_add(side.wrapping_mul(8)), wait.wrapping_sub(1))?;

                if wait > 1 {
                    break 'cannon;
                }

                call_rng(ctx, 2).wrapping_add(3)
            }
            3 => {
                if ctx.i32_at(countdown)? == 0 && get_castle_anim_state(ctx, faction)? == 0 {
                    let mut slot = 1i32;

                    while slot != 51 {
                        if ctx.i32_at(AppContext::entity_field(other, slot, Entity::OCCUPANT))? != 0 && ctx.i32_at(AppContext::entity_field(other, slot, Entity::STATE))? != 4 {
                            let screen_x = operation::div_10(ctx.i32_at(AppContext::entity_field(other, slot, Entity::POS_X))?.wrapping_sub(ctx.i32_at(AppContext::CAMERA_X)?));

                            ctx.set_i32_at(AppContext::SCRATCH_2, screen_x)?;

                            if cannon_target_in_range(ctx, faction, screen_x)? {
                                ctx.set_i32_at(action, 1)?;
                            }
                        }

                        slot += 1;
                    }
                }

                0
            }
            4 => 0,
            _ => break 'cannon,
        };

        ctx.set_i32_at(cannon_state, next)?;
    }

    let chosen = ctx.i32_at(pick)? as i64;

    if ctx.i32_at(candidates.wrapping_add((chosen * 4) as usize))? >= 0 {
        let money = get_money(ctx, wallet)?;
        let chosen = ctx.i32_at(pick)? as i64;
        let target = ctx.i32_at(candidates.wrapping_add((chosen * 4) as usize))?;

        if money >= get_effective_deploy_cost(ctx, faction, target)? {
            let chosen = ctx.i32_at(pick)? as i64;
            let target = ctx.i32_at(candidates.wrapping_add((chosen * 4) as usize))?;
            let shown = ctx.i32_at(AppContext::DECK_ROW_SHOWN)?;

            if target >= 0 && (target as u32 >= 5 && shown == 0 || target as u32 <= 4 && shown == 1) {
                if ctx.u8_at(AppContext::DECK_ROW_SWAPPING)? == 0 {
                    ctx.set_i32_at(AppContext::DECK_ROW_SWAP_DIRECTION, 1)?;
                    ctx.set_i32_at(AppContext::DECK_ROW_SWAP_TARGET, 1)?;
                } else {
                    match ctx.i32_at(AppContext::DECK_ROW_SWAP_DIRECTION)? {
                        -1 if ctx.i32_at(AppContext::DECK_ROW_SWAP_TARGET)? == 1 => ctx.set_i32_at(AppContext::DECK_ROW_SWAP_DIRECTION, 1)?,
                        1 if ctx.i32_at(AppContext::DECK_ROW_SWAP_TARGET)? == 0 => ctx.set_i32_at(AppContext::DECK_ROW_SWAP_DIRECTION, -1)?,
                        _ => {}
                    }
                }

                ctx.set_block_at::<1>(AppContext::DECK_ROW_SWAPPING, [1])?;
            }
        }
    }

    let pending = ctx.i32_at(action)?;

    if pending >= 3 && ctx.i32_at(AppContext::DEPLOY_NOTICE_TIMER)? > 0 {
        return ctx.set_i32_at(action, 0);
    }

    match pending {
        2 => {
            if get_worker_level(ctx, wallet)? == 7 {
                return ctx.set_i32_at(action, 0);
            }

            if get_max_money(ctx, wallet)? < get_worker_upgrade_cost(ctx, wallet)? {
                ctx.set_i32_at(action, 0)?;
            }

            if get_money(ctx, wallet)? < get_worker_upgrade_cost(ctx, wallet)? {
                return Ok(());
            }

            ctx.set_i32_at(action, 0)?;

            let mut open = 0i32;
            let mut button = 0i32;

            while button != 10 {
                if deck_slot_filled(ctx, faction, button)? && slot_deploy_permitted(ctx, faction, button, 1)? {
                    open = open.wrapping_add((is_deploy_blocked(ctx, button)? as u8 ^ 1) as i32);
                }

                button += 1;
            }

            let chosen = if open != 0 { call_rng(ctx, open) as i64 } else { 0 };
            let saving = AppContext::CPU_SAVING_FOR.wrapping_add(side.wrapping_mul(4));
            let target = ctx.i32_at(candidates.wrapping_add((chosen * 4) as usize))?;

            ctx.set_i32_at(saving, target)?;

            let mut button = 0i32;

            while button != 10 {
                if deck_slot_filled(ctx, faction, button)?
                    && slot_deploy_permitted(ctx, faction, button, 1)?
                    && !is_deploy_blocked(ctx, button)?
                    && ctx.i32_at(saving)? == get_button_unit_row(ctx, faction, button)?
                    && get_max_money(ctx, wallet)? < get_effective_deploy_cost(ctx, faction, button)?
                {
                    ctx.set_i32_at(saving, -1)?;

                    break;
                }

                button += 1;
            }

            let price = get_worker_upgrade_cost(ctx, wallet)?;

            add_money(ctx, wallet, price.wrapping_neg())?;
            upgrade_worker(ctx, wallet)?;
            ctx.set_i32_at(AppContext::WORKER_UPGRADE_FX, 0xe)?;
            play_sound(sound_manager(ctx)?, 0x13, None);

            Ok(())
        }
        1 => cannon_fire(ctx, 0),
        _ => {
            if pending < 3 {
                return Ok(());
            }

            let money = get_money(ctx, wallet)?;
            let chosen = ctx.i32_at(pick)? as i64;
            let target = ctx.i32_at(candidates.wrapping_add((chosen * 4) as usize))?;

            if money < get_effective_deploy_cost(ctx, faction, target)? || ctx.u8_at(AppContext::DECK_ROW_SWAPPING)? != 0 {
                return Ok(());
            }

            let chosen = ctx.i32_at(pick)? as i64;
            let target = ctx.i32_at(candidates.wrapping_add((chosen * 4) as usize))?;

            match ctx.i32_at(AppContext::DECK_ROW_SHOWN)? {
                0 => {
                    if target as u32 > 4 {
                        return Ok(());
                    }
                }
                1 => {
                    if target < 5 {
                        return Ok(());
                    }
                }
                _ => return Ok(()),
            }

            deploy_unit(ctx, faction, target, 0)?;
            ctx.set_i32_at(action, 0)
        }
    }
}
