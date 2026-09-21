use crate::{Fault, ops};

use super::*;

pub fn main_draw(ctx: &mut AppContext, flag: u8) -> Result<(), Fault> {
    ctx.zero(AppContext::DRAW_TEMP_0, 0x28)?;

    if ctx.i32_at(AppContext::CAT_GOD_SELECTED)? != 0 {
        let origin = ctx.i32_at(AppContext::LETTERBOX_PAD)?.wrapping_add(ctx.i32_at(AppContext::LETTERBOX_SHIFT)?);

        set_draw_origin(ctx, 0, origin)?;
    } else {
        for index in 0..3usize {
            let frame = ctx.i32_at(AppContext::CAT_GOD_FRAMES + index * 4)?;
            let delta = match frame {
                0x20 => Some(-0x28),
                0x22 => Some(0x14),
                0x24 => Some(-0xf),
                0x26 => Some(0xa),
                0x28 | 0x2c => Some(-0xa),
                0x2a | 0x2e | 0x31 | 0x33 | 0x35 | 0x37 => Some(0),
                0x30 => Some(-4),
                0x32 => Some(-6),
                0x34 => Some(4),
                0x36 => Some(-2),
                0x38 => Some(-1),
                _ if frame >= 0x39 => Some(0),
                _ => None,
            };

            if let Some(delta) = delta {
                let origin = ctx.i32_at(AppContext::LETTERBOX_SHIFT)?.wrapping_add(ctx.i32_at(AppContext::LETTERBOX_PAD)?).wrapping_add(delta);

                set_draw_origin(ctx, 0, origin)?;
            }
        }
    }

    let left = ((get_drawable_width(ctx)?.wrapping_add(-0x3c0) as f64) * 0.5) as f32;
    let lift = ctx.i32_at(AppContext::BATTLE_ZOOM_Y)?.wrapping_add(get_base_shake_offset(ctx));

    ctx.set_f32_at(AppContext::CAMERA_OFFSET, left)?;
    ctx.set_f32_at(AppContext::CAMERA_OFFSET + 4, lift as f32)?;

    let zoom = ctx.i32_at(AppContext::CAMERA_ZOOM)? as f32 / 10000.0;
    let skew = 0.0f32 * zoom;

    ctx.set_f32_at(AppContext::CAMERA_MATRIX, zoom)?;
    ctx.set_f32_at(AppContext::CAMERA_MATRIX + 4, skew)?;
    ctx.set_f32_at(AppContext::CAMERA_MATRIX + 8, skew)?;
    ctx.set_f32_at(AppContext::CAMERA_MATRIX + 0xc, zoom)?;

    let right = ((0x3c0i32.wrapping_sub(get_drawable_width(ctx)?) as f64) * 0.5) as f32;
    let zoom_y = ctx.i32_at(AppContext::BATTLE_ZOOM_Y)?;
    let down = camera_vertical_correction(ctx)?.wrapping_sub(zoom_y) as f32;
    let matrix = [
        ctx.f32_at(AppContext::CAMERA_MATRIX)?,
        ctx.f32_at(AppContext::CAMERA_MATRIX + 4)?,
        ctx.f32_at(AppContext::CAMERA_MATRIX + 8)?,
        ctx.f32_at(AppContext::CAMERA_MATRIX + 0xc)?,
    ];
    let offset_x = right * matrix[0] + matrix[2] * down + ctx.f32_at(AppContext::CAMERA_OFFSET)?;
    let offset_y = right * matrix[1] + matrix[3] * down + ctx.f32_at(AppContext::CAMERA_OFFSET + 4)?;

    ctx.set_f32_at(AppContext::CAMERA_OFFSET, offset_x)?;
    ctx.set_f32_at(AppContext::CAMERA_OFFSET + 4, offset_y)?;

    let transform = [matrix[0], matrix[1], matrix[2], matrix[3], offset_x, offset_y];
    let scale = ctx.screen_metrics.scale2;

    set_transform(draw_context(&mut ctx.draw)?, scale, &transform);
    set_tint(draw_context(&mut ctx.draw)?, 0, 0, 0, 0xff);

    let top = 0i32.wrapping_sub(ctx.i32_at(AppContext::LETTERBOX_SHIFT)?);
    let width = get_drawable_width(ctx)?;
    let height = get_design_height2(ctx);

    fill_rect(draw_context(&mut ctx.draw)?, 0, top, width, height);
    set_draw_scale(draw_context(&mut ctx.draw)?, scale);
    draw_background(ctx)?;
    draw_background_effects(ctx)?;
    ctx.set_block_at::<0x10>(AppContext::DRAW_LIST, [0; 0x10])?;
    ctx.set_i32_at(AppContext::DRAW_LIST + 0x10, 1)?;
    ctx.set_i32_at(AppContext::DRAW_LIST + 0x14, 0)?;
    ctx.set_i32_at(AppContext::DRAW_TEMP_0, 2)?;

    let mut slot = 1;

    while slot != 0x33 {
        if slot_occupied(ctx, 1, slot)? != 0 {
            let count = ctx.i32_at(AppContext::DRAW_TEMP_0)? as i64;
            let record = (AppContext::DRAW_LIST as i64 + count * 0xc) as usize;
            let z = ctx.i32_at(AppContext::entity_field(1, slot, Entity::Z_LAYER))?.wrapping_add(1);

            ctx.set_i32_at(record, z)?;

            let button = get_entity_button(ctx, 1, slot)?;
            let kind = if read_flag(ctx, AppContext::faction_flags(1))? & 2 != 0 { button.wrapping_add(0xa) } else { button.wrapping_add(0x82) };

            ctx.set_i32_at(record + 4, kind)?;

            if get_entity_state(ctx, 1, slot)? == 4 || get_entity_state(ctx, 1, slot)? == 0x15 {
                let count = ctx.i32_at(AppContext::DRAW_TEMP_0)? as i64;

                ctx.set_i32_at((AppContext::DRAW_LIST as i64 + count * 0xc + 4) as usize, button.wrapping_add(0x14))?;
            }

            let count = ctx.i32_at(AppContext::DRAW_TEMP_0)?;

            ctx.set_i32_at((AppContext::DRAW_LIST as i64 + count as i64 * 0xc + 8) as usize, slot)?;
            ctx.set_i32_at(AppContext::DRAW_TEMP_0, count.wrapping_add(1))?;
        }

        slot += 1;
    }

    let mut slot = 1;

    while slot != 0x33 {
        if slot_occupied(ctx, 0, slot)? != 0 {
            let count = ctx.i32_at(AppContext::DRAW_TEMP_0)? as i64;
            let record = (AppContext::DRAW_LIST as i64 + count * 0xc) as usize;
            let z = ctx.i32_at(AppContext::entity_field(0, slot, Entity::Z_LAYER))?.wrapping_add(1);

            ctx.set_i32_at(record, z)?;

            let button = get_entity_button(ctx, 0, slot)?;

            ctx.set_i32_at(record + 4, button.wrapping_add(0x64))?;

            if get_entity_state(ctx, 0, slot)? == 4 || get_entity_state(ctx, 0, slot)? == 0x15 {
                let count = ctx.i32_at(AppContext::DRAW_TEMP_0)? as i64;

                ctx.set_i32_at((AppContext::DRAW_LIST as i64 + count * 0xc + 4) as usize, button.wrapping_add(0x28))?;
            }

            let count = ctx.i32_at(AppContext::DRAW_TEMP_0)?;

            ctx.set_i32_at((AppContext::DRAW_LIST as i64 + count as i64 * 0xc + 8) as usize, slot)?;
            ctx.set_i32_at(AppContext::DRAW_TEMP_0, count.wrapping_add(1))?;
        }

        slot += 1;
    }

    let mut count = ctx.i32_at(AppContext::DRAW_TEMP_0)?;

    if count >= 2 {
        let mut first = 0i64;

        loop {
            let record = (AppContext::DRAW_LIST as i64 + first * 0xc) as usize;
            let head = ctx.i32_at(record)?;

            ctx.set_i32_at(AppContext::DRAW_TEMP_1, head)?;
            ctx.set_i32_at(AppContext::DRAW_TEMP_2, first as i32)?;

            let mut lowest = head;
            let mut pick = first as i32;
            let mut next = first + 1;

            while next < count as i64 {
                let z = ctx.i32_at((AppContext::DRAW_LIST as i64 + next * 0xc) as usize)?;

                if z < lowest {
                    ctx.set_i32_at(AppContext::DRAW_TEMP_1, z)?;
                    ctx.set_i32_at(AppContext::DRAW_TEMP_2, next as i32)?;
                    pick = next as i32;
                    lowest = z;
                }

                next += 1;
            }

            let held = ctx.block_at::<12>(record)?;

            ctx.set_block_at::<12>(AppContext::DRAW_SWAP, held)?;

            let chosen = (AppContext::DRAW_LIST as i64 + pick as i64 * 0xc) as usize;
            let moved = ctx.block_at::<8>(chosen)?;

            ctx.set_block_at::<8>(record, moved)?;

            let swapped = ctx.block_at::<8>(AppContext::DRAW_SWAP)?;

            ctx.set_block_at::<8>(chosen, swapped)?;

            let slot = ctx.i32_at(chosen + 8)?;

            ctx.set_i32_at(record + 8, slot)?;

            let held_slot = ctx.i32_at(AppContext::DRAW_SWAP + 8)?;
            let target = ctx.i32_at(AppContext::DRAW_TEMP_2)? as i64;

            ctx.set_i32_at((AppContext::DRAW_LIST as i64 + target * 0xc + 8) as usize, held_slot)?;
            count = ctx.i32_at(AppContext::DRAW_TEMP_0)?;
            first += 1;

            if first >= count as i64 - 1 {
                break;
            }
        }
    }

    if count >= 2 {
        let mut first = 0i64;

        loop {
            let record = (AppContext::DRAW_LIST as i64 + first * 0xc) as usize;
            let head = ctx.i32_at(record)?;

            ctx.set_i32_at(AppContext::DRAW_TEMP_1, head)?;
            ctx.set_i32_at(AppContext::DRAW_TEMP_2, first as i32)?;

            let mut lowest = ctx.i32_at(record + 8)?;

            ctx.set_i32_at(AppContext::DRAW_TEMP_3, lowest)?;

            let mut pick = first as i32;
            let mut next = first + 1;

            if next < count as i64 {
                while next < count as i64 {
                    let other = (AppContext::DRAW_LIST as i64 + next * 0xc) as usize;

                    if ctx.i32_at(other)? != head {
                        break;
                    }

                    let slot = ctx.i32_at(other + 8)?;

                    if slot < lowest {
                        ctx.set_i32_at(AppContext::DRAW_TEMP_3, slot)?;
                        ctx.set_i32_at(AppContext::DRAW_TEMP_2, next as i32)?;
                        pick = next as i32;
                        lowest = slot;
                    }

                    next += 1;
                }

                lowest = ctx.i32_at(record + 8)?;
            }

            let held = ctx.block_at::<8>(record)?;

            ctx.set_block_at::<8>(AppContext::DRAW_SWAP, held)?;

            let chosen = (AppContext::DRAW_LIST as i64 + pick as i64 * 0xc) as usize;
            let moved = ctx.block_at::<8>(chosen)?;

            ctx.set_block_at::<8>(record, moved)?;

            let swapped = ctx.block_at::<8>(AppContext::DRAW_SWAP)?;

            ctx.set_block_at::<8>(chosen, swapped)?;
            ctx.set_i32_at(AppContext::DRAW_SWAP + 8, lowest)?;

            let slot = ctx.i32_at(chosen + 8)?;

            ctx.set_i32_at(record + 8, slot)?;

            let held_slot = ctx.i32_at(AppContext::DRAW_SWAP + 8)?;
            let target = ctx.i32_at(AppContext::DRAW_TEMP_2)? as i64;

            ctx.set_i32_at((AppContext::DRAW_LIST as i64 + target * 0xc + 8) as usize, held_slot)?;
            count = ctx.i32_at(AppContext::DRAW_TEMP_0)?;
            first += 1;

            if first >= count as i64 - 1 {
                break;
            }
        }
    }

    if count > 0 {
        let mut record = 0i64;

        loop {
            let entry = (AppContext::DRAW_LIST as i64 + record * 0xc) as usize;
            let kind = ctx.i32_at(entry + 4)?;

            if kind as u32 <= 0x8c {
                let slot = ctx.i32_at(entry + 8)?;

                match kind {
                    0 => {
                        if !has_castle_enemy(ctx)? {
                            let base_x = get_base_pos_x(ctx, 1)?;
                            let size = get_castle_row(&ctx.enemy_castle, get_castle_id(ctx)?)?.size;
                            let inset = ops::div_neg_100(size.wrapping_mul(0x49c));
                            let shifted = ops::div_10(base_x.wrapping_sub(ctx.i32_at(AppContext::CAMERA_X)?).wrapping_add(inset));
                            let offset_x = get_castle_row(&ctx.enemy_castle, get_castle_id(ctx)?)?.offset_x;
                            let size = get_castle_row(&ctx.enemy_castle, get_castle_id(ctx)?)?.size;

                            ctx.set_i32_at(AppContext::DRAW_TEMP_1, ops::div_100(offset_x.wrapping_mul(size)).wrapping_add(shifted))?;

                            let base_y = ops::div_10(get_base_pos_y(ctx, 1)?);
                            let size = get_castle_row(&ctx.enemy_castle, get_castle_id(ctx)?)?.size;
                            let lifted = ops::div_neg_100(size.wrapping_mul(0xff)).wrapping_add(base_y);
                            let offset_y = get_castle_row(&ctx.enemy_castle, get_castle_id(ctx)?)?.offset_y;
                            let size = get_castle_row(&ctx.enemy_castle, get_castle_id(ctx)?)?.size;
                            let y = ops::div_100(offset_y.wrapping_mul(size)).wrapping_add(lifted);

                            ctx.set_i32_at(AppContext::DRAW_TEMP_2, y)?;
                            ctx.set_i32_at(AppContext::DRAW_TEMP_3, 0)?;

                            let mut tint = 0i32;

                            if ctx.i32_at(AppContext::entity_field(1, 0, Base::STATE))? != 0 {
                                let period = ctx.i32_at(AppContext::SPEED)?.wrapping_add(1);
                                let phase = ops::irem(ctx.i32_at(AppContext::BATTLE_FRAME_COUNTER)?, period).ok_or(Fault::divide(period as i64))?;

                                if phase == 1 {
                                    ctx.set_i32_at(AppContext::DRAW_TEMP_3, 4)?;
                                    tint = 4;
                                }
                            }

                            let x = tint.wrapping_add(ctx.i32_at(AppContext::DRAW_TEMP_1)?);

                            draw_castle(ctx, 1, x, y)?;
                            item_drop_particles(ctx)?;
                        }
                    }
                    1 => {
                        let mut tint = 0i32;
                        let base_x = get_base_pos_x(ctx, 0)?.wrapping_sub(ctx.i32_at(AppContext::CAMERA_X)?);

                        ctx.set_i32_at(AppContext::DRAW_TEMP_1, ops::div_10(base_x))?;

                        let y = ops::div_10(get_base_pos_y(ctx, 0)?).wrapping_add(-0x143);

                        ctx.set_i32_at(AppContext::DRAW_TEMP_2, y)?;
                        ctx.set_i32_at(AppContext::DRAW_TEMP_3, 0)?;

                        if ctx.i32_at(AppContext::entity_field(0, 0, Base::STATE))? != 0 {
                            let period = ctx.i32_at(AppContext::SPEED)?.wrapping_add(1);
                            let phase = ops::irem(ctx.i32_at(AppContext::BATTLE_FRAME_COUNTER)?, period).ok_or(Fault::divide(period as i64))?;

                            if phase == 1 {
                                ctx.set_i32_at(AppContext::DRAW_TEMP_3, 4)?;
                                tint = 4;
                            }
                        }

                        let x = tint.wrapping_add(ctx.i32_at(AppContext::DRAW_TEMP_1)?);

                        draw_castle(ctx, 0, x, y)?;

                        if get_battle_status(ctx)? == 0 {
                            let mut faction = 0;
                            let mut first = true;
                            let mut flip = false;

                            loop {
                                if has_cannon_ready_vfx(ctx, faction)?
                                    && ctx.i32_at(AppContext::entity_field(faction, 0, Base::CANNON_COUNTDOWN))? == 0
                                    && read_flag(ctx, AppContext::faction_flags(faction))? & 1 != 0
                                {
                                    let reach = cannon_reach_x(ctx, faction)?;

                                    ctx.set_i32_at(AppContext::DRAW_TEMP_1, reach)?;

                                    let step = get_base_level(ctx, faction)?.wrapping_mul(CANNON_SHOT_SPACING);
                                    let lead = if first { 0x5ci32.wrapping_sub(step) } else { step.wrapping_add(-0x5c) };
                                    let tip = reach.wrapping_add(lead).wrapping_add(-4) as f64;
                                    let tip = get_drawable_width(ctx)?.wrapping_add(-0x3c0) as f64 * 0.5 + tip;
                                    let edge = ops::div_10(0x12ci32.wrapping_sub(ctx.i32_at(AppContext::CAMERA_X)?)).wrapping_add(0x60) as f64;
                                    let edge = get_drawable_width(ctx)?.wrapping_add(-0x3c0) as f64 * 0.5 + edge;
                                    let arrow = if edge > tip {
                                        let edge = ops::div_10(0x12ci32.wrapping_sub(ctx.i32_at(AppContext::CAMERA_X)?)).wrapping_add(0x60) as f64;
                                        let x = ops::cvttsd2si(get_drawable_width(ctx)?.wrapping_add(-0x3c0) as f64 * 0.5 + edge);

                                        draw_cut(draw_context(&mut ctx.draw)?, ctx.img002_sheet.as_deref().ok_or(Fault::null_pointer())?, x, 0x19a, 0);
                                        set_alpha(draw_context(&mut ctx.draw)?, 0xd8);

                                        if flip {
                                            set_flip(draw_context(&mut ctx.draw)?, 1);
                                        }

                                        ops::div_10(0x12ci32.wrapping_sub(ctx.i32_at(AppContext::CAMERA_X)?)).wrapping_add(0x42)
                                    } else {
                                        let reach = ctx.i32_at(AppContext::DRAW_TEMP_1)?;
                                        let step = get_base_level(ctx, faction)?.wrapping_mul(CANNON_SHOT_SPACING);
                                        let lead = if first { 0x5ci32.wrapping_sub(step) } else { step.wrapping_add(-0x5c) };
                                        let tip = reach.wrapping_add(lead).wrapping_add(-4) as f64;
                                        let x = ops::cvttsd2si(get_drawable_width(ctx)?.wrapping_add(-0x3c0) as f64 * 0.5 + tip);

                                        draw_cut(draw_context(&mut ctx.draw)?, ctx.img002_sheet.as_deref().ok_or(Fault::null_pointer())?, x, 0x19a, 0);
                                        set_alpha(draw_context(&mut ctx.draw)?, 0xd8);

                                        if flip {
                                            set_flip(draw_context(&mut ctx.draw)?, 1);
                                        }

                                        let reach = ctx.i32_at(AppContext::DRAW_TEMP_1)?;
                                        let step = get_base_level(ctx, faction)?.wrapping_mul(CANNON_SHOT_SPACING);
                                        let lead = if first { 0x5ci32.wrapping_sub(step) } else { step.wrapping_add(-0x5c) };

                                        lead.wrapping_add(reach).wrapping_add(-0x22)
                                    };
                                    let x = ops::cvttsd2si(get_drawable_width(ctx)?.wrapping_add(-0x3c0) as f64 * 0.5 + arrow as f64);
                                    let bob = sin_deg(ctx.i32_at(AppContext::DRAW_FRAMES)?.wrapping_mul(0x3c) as f32) * 11.0 + 314.0;

                                    draw_cut(draw_context(&mut ctx.draw)?, ctx.img002_sheet.as_deref().ok_or(Fault::null_pointer())?, x, ops::cvttss2si(bob), 1);

                                    if flip {
                                        set_flip(draw_context(&mut ctx.draw)?, 0);
                                    }

                                    set_alpha(draw_context(&mut ctx.draw)?, 0xff);
                                }

                                faction = 1;
                                flip = true;

                                if !first {
                                    break;
                                }

                                first = false;
                            }
                        }
                    }
                    0xa..=0x13 | 0x64..=0x78 | 0x82..=0x8c => {
                        let (faction, button, enemy, cat) = match kind {
                            0xa..=0x13 => (1, kind.wrapping_add(-0xa), true, false),
                            0x64..=0x78 => (0, kind.wrapping_add(-0x64), false, true),
                            _ => (1, kind.wrapping_add(-0x82), true, false),
                        };
                        let (side, index) = get_unit_model(ctx, faction, button)?.ok_or(Fault::null_pointer())?;

                        if kind >= 0x82 {
                            mamodel_set_mirror(&mut ctx.unit_models[side][index], 1);
                        }

                        let pose = if get_entity_state(ctx, faction, slot)? == 0 {
                            Some((0, true))
                        } else if get_entity_state(ctx, faction, slot)? == 1
                            || get_entity_state(ctx, faction, slot)? == 0x10
                            || get_entity_state(ctx, faction, slot)? == 0x12
                            || get_entity_state(ctx, faction, slot)? == 0x13
                            || get_entity_state(ctx, faction, slot)? == 0x14
                        {
                            Some((1, true))
                        } else if get_entity_state(ctx, faction, slot)? == 2 {
                            Some((2, true))
                        } else if get_entity_state(ctx, faction, slot)? == 3
                            || get_entity_state(ctx, faction, slot)? == 5
                            || get_entity_state(ctx, faction, slot)? == 6
                            || get_entity_state(ctx, faction, slot)? == 0x11
                        {
                            Some((3, true))
                        } else if get_entity_state(ctx, faction, slot)? == 7 {
                            Some((0, false))
                        } else if get_entity_state(ctx, faction, slot)? == 0xa {
                            Some((7, true))
                        } else if get_entity_state(ctx, faction, slot)? == 0xb {
                            Some((4, true))
                        } else if get_entity_state(ctx, faction, slot)? == 0xc {
                            Some((5, true))
                        } else if get_entity_state(ctx, faction, slot)? == 0xd {
                            Some((6, true))
                        } else if get_entity_state(ctx, faction, slot)? == 0x13 {
                            Some((1, true))
                        } else {
                            None
                        };

                        if let Some((key, timed)) = pose {
                            let anim = get_unit_anim(ctx, faction, button, key)?.cloned();
                            let frame = if timed { get_entity_frame(ctx, faction, slot)? } else { 0 };

                            maanim_execute(&mut ctx.unit_models[side][index], anim.as_ref(), frame, 0)?;
                        }

                        let pos_x = ctx.i32_at(AppContext::entity_field(faction, slot, Entity::POS_X))?.wrapping_sub(ctx.i32_at(AppContext::CAMERA_X)?);

                        ctx.set_i32_at(AppContext::DRAW_TEMP_1, ops::div_10(pos_x))?;

                        let pos_y = ops::div_10(ctx.i32_at(AppContext::entity_field(faction, slot, Entity::POS_Y))?);
                        let depth = ctx.i32_at(AppContext::entity_field(faction, slot, Entity::Z_LAYER))?;

                        ctx.set_i32_at(AppContext::DRAW_TEMP_2, pos_y.wrapping_add(depth << 2))?;

                        let flash = if get_entity_state(ctx, faction, slot)? == 0x11 || get_hit_flash_timer(ctx, faction, slot)? > 0 {
                            ctx.set_i32_at(AppContext::DRAW_TEMP_3, 0)?;

                            if ctx.i32_at(AppContext::entity_field(1, 0, Base::STATE))? != 0 || get_hit_flash_timer(ctx, faction, slot)? > 0 {
                                let period = ctx.i32_at(AppContext::SPEED)?.wrapping_add(1);
                                let phase = ops::irem(ctx.i32_at(AppContext::BATTLE_FRAME_COUNTER)?, period).ok_or(Fault::divide(period as i64))?;

                                if phase == 1 {
                                    ctx.set_i32_at(AppContext::DRAW_TEMP_3, 4)?;
                                }
                            }

                            ctx.i32_at(AppContext::DRAW_TEMP_3)?
                        } else {
                            0
                        };

                        let model = &ctx.unit_models[side][index];
                        let part_index = mamodel_get_anchor_part(mamodel_get_anchor(model, 0)?);
                        let part = model.parts.get(part_index as i64 as usize).ok_or(Fault::index_out_of_range(part_index as i64, model.parts.len() as i64))?;
                        let anchor_x = mamodel_get_anchor_x(mamodel_get_anchor(model, 0)?);
                        let anchor_y = mamodel_get_anchor_y(mamodel_get_anchor(model, 0)?);
                        let mut out = 0i64;

                        transform_anchor(part, model, anchor_x, anchor_y, &mut out)?;
                        ctx.set_block_at::<8>(AppContext::ANCHOR_OUT, out.to_le_bytes())?;

                        if get_entity_state(ctx, faction, slot)? == 0x12 || get_entity_state(ctx, faction, slot)? == 0x14 {
                            let facing = read_flag(ctx, AppContext::faction_flags(faction))? & 1;
                            let model = &ctx.unit_models[side][index];
                            let part_index = mamodel_get_anchor_part(mamodel_get_anchor(model, facing)?);
                            let part = model.parts.get(part_index as i64 as usize).ok_or(Fault::index_out_of_range(part_index as i64, model.parts.len() as i64))?;
                            let anchor_x = mamodel_get_anchor_x(mamodel_get_anchor(model, facing)?);
                            let anchor_y = mamodel_get_anchor_y(mamodel_get_anchor(model, facing)?);
                            let mut pivot = 0i64;

                            transform_anchor(part, model, anchor_x, anchor_y, &mut pivot)?;

                            let pivot_x = pivot as i32;
                            let pivot_y = (pivot >> 32) as i32;

                            if get_entity_state(ctx, faction, slot)? == 0x12 {
                                let frame = get_entity_frame(ctx, faction, slot)?;

                                maanim_execute(&mut ctx.warp_chara_model, Some(&ctx.warp_anims[2]), frame, 0)?;
                            } else if get_entity_state(ctx, faction, slot)? == 0x14 {
                                let frame = get_entity_frame(ctx, faction, slot)?;

                                maanim_execute(&mut ctx.warp_chara_model, Some(&ctx.warp_anims[3]), frame, 0)?;
                            }

                            let origin = flash.wrapping_add(ctx.i32_at(AppContext::DRAW_TEMP_1)?) as f64;
                            let origin = ops::cvttsd2si(get_drawable_width(ctx)?.wrapping_add(-0x3c0) as f64 * 0.5 + origin);
                            let model = &ctx.warp_chara_model;
                            let part = mamodel_get_part(model, 0).ok_or(Fault::null_pointer())?;
                            let body = model.parts.get(part + 1).ok_or(Fault::index_out_of_range((part + 1) as i64, model.parts.len() as i64))?;
                            let y = ctx.i32_at(AppContext::DRAW_TEMP_2)?.wrapping_sub(pivot_y).wrapping_add(body.i32_at(0x40)).wrapping_add(body.i32_at(0x48));
                            let scale_x = body.i32_at(0x5c) as f32;
                            let scale_y = body.i32_at(0x64) as f32;
                            let unit_x = mamodel_get_scale_unit(model);
                            let unit_y = mamodel_get_scale_unit(model);
                            let opacity = body.i32_at(0x80);
                            let alpha = ops::idiv((opacity << 8).wrapping_sub(opacity), model.opacity_unit).ok_or(Fault::divide(model.opacity_unit as i64))?;

                            if alpha > 0 {
                                let scale = scale_x * scale_y / unit_x as f32 / unit_y as f32;

                                if get_revive_timer(ctx, faction, slot)? > 0 {
                                    let frame = get_entity_frame(ctx, faction, slot)?;

                                    maanim_execute(&mut ctx.zombie_model, Some(&ctx.zombie_down_anim), frame, 0)?;

                                    let x = origin.wrapping_sub(ctx.i32_at(AppContext::ANCHOR_OUT)?);

                                    draw_model_scaled(draw_context(&mut ctx.draw)?, &ctx.zombie_model, x, y, pivot_x, pivot_y, scale, alpha, 0, 0);
                                } else {
                                    let x = origin.wrapping_sub(ctx.i32_at(AppContext::ANCHOR_OUT)?);

                                    draw_model_scaled(draw_context(&mut ctx.draw)?, &ctx.unit_models[side][index], x, y, pivot_x, pivot_y, scale, alpha, 0, 0);
                                }
                            }

                            let portal = if get_entity_state(ctx, faction, slot)? == 0x12 {
                                Some(0)
                            } else if get_entity_state(ctx, faction, slot)? == 0x14 {
                                Some(1)
                            } else {
                                None
                            };

                            if let Some(portal) = portal {
                                let frame = get_entity_frame(ctx, faction, slot)?;

                                maanim_execute(&mut ctx.warp_model, Some(&ctx.warp_anims[portal]), frame, 0)?;

                                let shift = pivot_x.wrapping_sub(ctx.i32_at(AppContext::ANCHOR_OUT)?);
                                let x = if cat { shift } else { 0 }.wrapping_add(origin);
                                let y = ctx.i32_at(AppContext::DRAW_TEMP_2)?.wrapping_add(-0x15e);

                                draw_model(draw_context(&mut ctx.draw)?, &ctx.warp_model, x, y);
                            }
                        } else if get_entity_state(ctx, faction, slot)? != 0xf && get_entity_state(ctx, faction, slot)? != 0x13 {
                            if get_revive_timer(ctx, faction, slot)? <= 0 {
                                let origin = flash.wrapping_add(ctx.i32_at(AppContext::DRAW_TEMP_1)?) as f64;
                                let anchor = ctx.i32_at(AppContext::ANCHOR_OUT)? as f64;
                                let x = ops::cvttsd2si(get_drawable_width(ctx)?.wrapping_add(-0x3c0) as f64 * 0.5 + origin - anchor);
                                let y = ctx.i32_at(AppContext::DRAW_TEMP_2)?.wrapping_sub(ctx.i32_at(AppContext::ANCHOR_OUT + 4)?);

                                draw_model(draw_context(&mut ctx.draw)?, &ctx.unit_models[side][index], x, y);
                            } else {
                                let rising = get_entity_state(ctx, faction, slot)? == 3 || get_entity_state(ctx, faction, slot)? == 6;
                                let frame = get_entity_frame(ctx, faction, slot)?;

                                if rising {
                                    maanim_execute(&mut ctx.zombie_model, Some(&ctx.zombie_back_anim), frame, 0)?;
                                } else {
                                    maanim_execute(&mut ctx.zombie_model, Some(&ctx.zombie_down_anim), frame, 0)?;
                                }

                                let origin = ctx.i32_at(AppContext::DRAW_TEMP_1)?.wrapping_sub(ops::div_2(ctx.i32_at(AppContext::ANCHOR_OUT)?)) as f64;
                                let x = ops::cvttsd2si(get_drawable_width(ctx)?.wrapping_add(-0x3c0) as f64 * 0.5 + origin);
                                let y = ctx.i32_at(AppContext::DRAW_TEMP_2)?;

                                draw_model(draw_context(&mut ctx.draw)?, &ctx.zombie_model, x, y);
                            }
                        }

                        let spawn = get_spawn_anim_type(ctx, faction, slot)?;

                        if get_entity_state(ctx, faction, slot)? == 0xa && spawn >= 0 {
                            let anchor = ctx.i32_at(AppContext::ANCHOR_OUT)?;
                            let mut model = std::mem::take(std_map_int_mamodel_subscript(&mut ctx.effect_models, &spawn));
                            let anim = std::mem::take(std_map_int_maanim_subscript(&mut ctx.effect_anims, &spawn));
                            let frame = ctx.i32_at(AppContext::entity_field(faction, slot, Entity::FRAME))?;

                            maanim_execute(&mut model, Some(&anim), frame, 0)?;
                            *std_map_int_maanim_subscript(&mut ctx.effect_anims, &spawn) = anim;
                            *std_map_int_mamodel_subscript(&mut ctx.effect_models, &spawn) = model;

                            let origin = ctx.i32_at(AppContext::DRAW_TEMP_1)?.wrapping_sub(anchor).wrapping_add(ctx.i32_at(AppContext::ANCHOR_OUT)?) as f64;
                            let x = ops::cvttsd2si(get_drawable_width(ctx)?.wrapping_add(-0x3c0) as f64 * 0.5 + origin);
                            let y = ctx.i32_at(AppContext::DRAW_TEMP_2)?;

                            draw_model(draw_context(&mut ctx.draw)?, std_map_int_mamodel_subscript(&mut ctx.effect_models, &spawn), x, y);
                        } else if get_entity_state(ctx, faction, slot)? == 0xf || get_entity_state(ctx, faction, slot)? == 0x10 {
                            let frame = get_entity_frame(ctx, faction, slot)?;

                            maanim_execute(&mut ctx.zombie_model, Some(&ctx.zombie_revive_anim), frame, 0)?;

                            let origin = ctx.i32_at(AppContext::DRAW_TEMP_1)?.wrapping_sub(ops::div_2(ctx.i32_at(AppContext::ANCHOR_OUT)?)) as f64;
                            let x = ops::cvttsd2si(get_drawable_width(ctx)?.wrapping_add(-0x3c0) as f64 * 0.5 + origin);
                            let y = ctx.i32_at(AppContext::DRAW_TEMP_2)?;

                            draw_model(draw_context(&mut ctx.draw)?, &ctx.zombie_model, x, y);
                        }

                        if get_entity_state(ctx, faction, slot)? != 0xe
                            && get_entity_state(ctx, faction, slot)? != 0xf
                            && get_entity_state(ctx, faction, slot)? != 0x13
                            && get_entity_state(ctx, faction, slot)? != 0x12
                            && get_entity_state(ctx, faction, slot)? != 0x14
                        {
                            let step = if cat { 0x3c } else { -0x3c };
                            let mut offset = 0i32;

                            for badge in 0..4 {
                                let origin = ctx.i32_at(AppContext::DRAW_TEMP_1)? as f64;
                                let x = ops::cvttsd2si(get_drawable_width(ctx)?.wrapping_add(-0x3c0) as f64 * 0.5 + origin + offset as f64);

                                if get_proc_badge(ctx, faction, slot, badge)? == 5 {
                                    if get_dodge_vfx_frame(ctx, faction, slot)? > 0
                                        && (get_dodge_timer(ctx, faction, slot)? > 0 || get_behemoth_dodge_timer(ctx, faction, slot)? > 0 || get_orb_dodge_timer(ctx, faction, slot)? > 0)
                                    {
                                        let frame = get_dodge_vfx_frame(ctx, faction, slot)?;

                                        maanim_execute(&mut ctx.attack_invalid_model, Some(&ctx.attack_invalid_anim), frame, 0)?;

                                        let y = ctx.i32_at(AppContext::DRAW_TEMP_2)?;

                                        draw_model(draw_context(&mut ctx.draw)?, &ctx.attack_invalid_model, x, y);
                                    }
                                } else if get_proc_badge(ctx, faction, slot, badge)? == 1 {
                                    if get_freeze_timer(ctx, faction, slot)? > 0 && get_freeze_length(ctx, faction, slot)? > 0 {
                                        let y = ctx.i32_at(AppContext::DRAW_TEMP_2)?;
                                        let left = get_freeze_length(ctx, faction, slot)?.wrapping_sub(get_freeze_timer(ctx, faction, slot)?);

                                        skill_stop_draw(ctx, x, y, left, faction)?;
                                    } else if get_slow_timer(ctx, faction, slot)? > 0 && get_slow_length(ctx, faction, slot)? > 0 {
                                        let y = ctx.i32_at(AppContext::DRAW_TEMP_2)?;
                                        let left = get_slow_length(ctx, faction, slot)?.wrapping_sub(get_slow_timer(ctx, faction, slot)?);

                                        skill_slow_draw(ctx, x, y, left, faction)?;
                                    }
                                } else if get_proc_badge(ctx, faction, slot, badge)? == 2 {
                                    if get_weaken_timer(ctx, faction, slot)? > 0 && get_weaken_active(ctx, faction, slot)? > 0 {
                                        let y = ctx.i32_at(AppContext::DRAW_TEMP_2)?;
                                        let left = get_weaken_active(ctx, faction, slot)?.wrapping_sub(get_weaken_timer(ctx, faction, slot)?);

                                        skill_down_draw(ctx, x, y, left, faction)?;
                                    } else if get_status_bits(ctx, faction, slot)? > 0 {
                                        let y = ctx.i32_at(AppContext::DRAW_TEMP_2)?;
                                        let frame = ctx.i32_at(AppContext::entity_field(faction, slot, Entity::ATTACK_UP_VFX_FRAME))?;

                                        skill_up_draw(ctx, x, y, frame, faction)?;
                                    }
                                } else if get_proc_badge(ctx, faction, slot, badge)? == 3 {
                                    if ctx.i32_at(AppContext::entity_field(faction, slot, Entity::SURVIVE_USED))? > 0 {
                                        let y = ctx.i32_at(AppContext::DRAW_TEMP_2)?;
                                        let frame = ctx.i32_at(AppContext::entity_field(faction, slot, Entity::SURVIVE_VFX_FRAME))?;

                                        skill_shield_draw(ctx, x, y, frame, faction)?;
                                    }
                                } else if get_proc_badge(ctx, faction, slot, badge)? == 4 && get_curse_timer(ctx, faction, slot)? > 0 && get_curse_length(ctx, faction, slot)? > 0 {
                                    let y = ctx.i32_at(AppContext::DRAW_TEMP_2)?;
                                    let left = get_curse_length(ctx, faction, slot)?.wrapping_sub(get_curse_timer(ctx, faction, slot)?);

                                    skill_curse_draw(ctx, x, y, left, faction)?;
                                }

                                offset = offset.wrapping_add(step);
                            }
                        }

                        if ctx.i32_at(AppContext::entity_field(faction, slot, Entity::WAVE_IMMUNE_VFX_ACTIVE))? > 0 {
                            let origin = ctx.i32_at(AppContext::DRAW_TEMP_1)? as f64;
                            let x = ops::cvttsd2si(get_drawable_width(ctx)?.wrapping_add(-0x3c0) as f64 * 0.5 + origin);
                            let y = ctx.i32_at(AppContext::DRAW_TEMP_2)?;
                            let frame = ctx.i32_at(AppContext::entity_field(faction, slot, Entity::WAVE_IMMUNE_VFX_FRAME))?;

                            skill_wave_invalid_draw(ctx, x, y, frame, faction)?;
                        }

                        if ctx.i32_at(AppContext::entity_field(faction, slot, Entity::WAVE_BLOCK_VFX_ACTIVE))? > 0 {
                            let origin = ctx.i32_at(AppContext::DRAW_TEMP_1)? as f64;
                            let x = ops::cvttsd2si(get_drawable_width(ctx)?.wrapping_add(-0x3c0) as f64 * 0.5 + origin);
                            let y = ctx.i32_at(AppContext::DRAW_TEMP_2)?.wrapping_add(-0x2a);
                            let frame = ctx.i32_at(AppContext::entity_field(faction, slot, Entity::WAVE_BLOCK_VFX_FRAME))?;

                            skill_wave_stop_draw(ctx, x, y, frame, faction)?;
                        }

                        if ctx.i32_at(AppContext::entity_field(faction, slot, Entity::IMMUNE_VFX_ACTIVE))? > 0 {
                            let frame = ctx.i32_at(AppContext::entity_field(faction, slot, Entity::IMMUNE_VFX_FRAME))?;

                            maanim_execute(&mut ctx.skill_effect_invalid_model, Some(&ctx.skill_effect_invalid_anim), frame, 0)?;

                            let origin = ctx.i32_at(AppContext::DRAW_TEMP_1)? as f64;
                            let x = ops::cvttsd2si(get_drawable_width(ctx)?.wrapping_add(-0x3c0) as f64 * 0.5 + origin);
                            let y = ctx.i32_at(AppContext::DRAW_TEMP_2)?.wrapping_add(-0x2a);

                            draw_model(draw_context(&mut ctx.draw)?, &ctx.skill_effect_invalid_model, x, y);
                        }

                        if get_barrier_vfx_active(ctx, faction, slot)? {
                            let frame = get_barrier_vfx_frame(ctx, faction, slot)?;

                            maanim_execute(&mut ctx.barrier_model, Some(&ctx.barrier_anims[0]), frame, 0)?;

                            let origin = ctx.i32_at(AppContext::DRAW_TEMP_1)? as f64;
                            let x = ops::cvttsd2si(get_drawable_width(ctx)?.wrapping_add(-0x3c0) as f64 * 0.5 + origin);
                            let y = ctx.i32_at(AppContext::DRAW_TEMP_2)?.wrapping_add(-0x2a);

                            draw_model(draw_context(&mut ctx.draw)?, &ctx.barrier_model, x, y);
                        }

                        if get_shield_vfx(ctx, faction, slot)? != 0 {
                            let shield = get_shield_vfx(ctx, faction, slot)?.wrapping_sub(1);
                            let frame = get_shield_vfx_frame(ctx, faction, slot)?;
                            let anim = ctx.shield_anims.get(shield as i64 as usize).ok_or(Fault::index_out_of_range(shield as i64, 5))?;

                            maanim_execute(&mut ctx.demonshield_model, Some(anim), frame, 0)?;

                            let origin = ctx.i32_at(AppContext::DRAW_TEMP_1)? as f64;
                            let x = ops::cvttsd2si(get_drawable_width(ctx)?.wrapping_add(-0x3c0) as f64 * 0.5 + origin);
                            let y = ctx.i32_at(AppContext::DRAW_TEMP_2)?.wrapping_add(-0x2a);

                            draw_model(draw_context(&mut ctx.draw)?, &ctx.demonshield_model, x, y);
                        }

                        if enemy && has_castle_enemy(ctx)? && get_entity_base_idx(ctx)? == slot {
                            item_drop_particles(ctx)?;
                        }

                        let depth = ctx.i32_at(AppContext::entity_field(faction, slot, Entity::Z_LAYER))? << 2;

                        wave_draw(ctx, depth, slot, 2i32.wrapping_sub(cat as i32))?;
                        surge_draw(ctx, faction, slot)?;
                        counter_surge_draw(ctx, faction, slot)?;
                        explosion_draw(ctx, faction, slot)?;
                    }
                    0x14..=0x1e | 0x28..=0x3c => {
                        let faction = (kind < 0x28) as i32;
                        let button = if kind < 0x28 { kind.wrapping_add(-0x14) } else { kind.wrapping_add(-0x28) };

                        if get_entity_state(ctx, faction, slot)? == 0x15 {
                            let pos_x = ctx.i32_at(AppContext::entity_field(faction, slot, Entity::POS_X))?.wrapping_sub(ctx.i32_at(AppContext::CAMERA_X)?);

                            ctx.set_i32_at(AppContext::DRAW_TEMP_1, ops::div_10(pos_x))?;

                            let pos_y = ops::div_10(ctx.i32_at(AppContext::entity_field(faction, slot, Entity::POS_Y))?);
                            let depth = ctx.i32_at(AppContext::entity_field(1, slot, Entity::Z_LAYER))?;

                            ctx.set_i32_at(AppContext::DRAW_TEMP_2, pos_y.wrapping_add(depth << 2).wrapping_add(-0x61))?;

                            let frame = get_entity_frame(ctx, faction, slot)?;
                            let model = if faction == 0 { &mut ctx.demonsoul_01_model } else { &mut ctx.demonsoul_00_model };

                            maanim_execute(model, Some(&ctx.death_surge_anims[faction as usize]), frame, 0)?;

                            let origin = ctx.i32_at(AppContext::DRAW_TEMP_1)? as f64;
                            let x = ops::cvttsd2si(get_drawable_width(ctx)?.wrapping_add(-0x3c0) as f64 * 0.5 + origin);
                            let y = ctx.i32_at(AppContext::DRAW_TEMP_2)?;
                            let model = if faction == 0 { &ctx.demonsoul_01_model } else { &ctx.demonsoul_00_model };

                            draw_model(draw_context(&mut ctx.draw)?, model, x, y);
                        } else {
                            if get_gudetama_soul(ctx, faction, slot)? {
                                let (side, index) = get_unit_model(ctx, faction, button)?.ok_or(Fault::null_pointer())?;
                                let anim = get_unit_anim(ctx, faction, button, 8)?.cloned();
                                let frame = get_entity_frame(ctx, faction, slot)?;

                                maanim_execute(&mut ctx.unit_models[side][index], anim.as_ref(), frame, 0)?;

                                let model = &ctx.unit_models[side][index];
                                let part_index = mamodel_get_anchor_part(mamodel_get_anchor(model, 0)?);
                                let part = model.parts.get(part_index as i64 as usize).ok_or(Fault::index_out_of_range(part_index as i64, model.parts.len() as i64))?;
                                let anchor_x = mamodel_get_anchor_x(mamodel_get_anchor(model, 0)?);
                                let anchor_y = mamodel_get_anchor_y(mamodel_get_anchor(model, 0)?);
                                let mut out = 0i64;

                                transform_anchor(part, model, anchor_x, anchor_y, &mut out)?;
                                ctx.set_block_at::<8>(AppContext::ANCHOR_OUT, out.to_le_bytes())?;

                                let pos_x = ops::div_10(ctx.i32_at(AppContext::entity_field(faction, slot, Entity::POS_X))?.wrapping_sub(ctx.i32_at(AppContext::CAMERA_X)?));

                                ctx.set_i32_at(AppContext::DRAW_TEMP_1, pos_x)?;

                                let pos_y = ops::div_10(ctx.i32_at(AppContext::entity_field(faction, slot, Entity::POS_Y))?);
                                let depth = ctx.i32_at(AppContext::entity_field(faction, slot, Entity::Z_LAYER))?;

                                ctx.set_i32_at(AppContext::DRAW_TEMP_2, pos_y.wrapping_add(depth << 2))?;

                                let origin = pos_x.wrapping_sub(ctx.i32_at(AppContext::ANCHOR_OUT)?) as f64;
                                let x = ops::cvttsd2si(get_drawable_width(ctx)?.wrapping_add(-0x3c0) as f64 * 0.5 + origin);
                                let y = ctx.i32_at(AppContext::DRAW_TEMP_2)?.wrapping_sub(ctx.i32_at(AppContext::ANCHOR_OUT + 4)?);

                                draw_model(draw_context(&mut ctx.draw)?, &ctx.unit_models[side][index], x, y);
                            }

                            if get_soul_anim_type(ctx, faction, slot)? >= 0 {
                                let soul = get_soul_anim_type(ctx, faction, slot)?.wrapping_add(0x3e8);
                                let pos_x = ctx.i32_at(AppContext::entity_field(faction, slot, Entity::POS_X))?.wrapping_sub(ctx.i32_at(AppContext::CAMERA_X)?);

                                ctx.set_i32_at(AppContext::DRAW_TEMP_1, ops::div_10(pos_x))?;

                                let pos_y = ops::div_10(ctx.i32_at(AppContext::entity_field(faction, slot, Entity::POS_Y))?);
                                let depth = ctx.i32_at(AppContext::entity_field(1, slot, Entity::Z_LAYER))?;

                                ctx.set_i32_at(AppContext::DRAW_TEMP_2, pos_y.wrapping_add(depth << 2).wrapping_add(-0x61))?;

                                let mut model = std::mem::take(std_map_int_mamodel_subscript(&mut ctx.effect_models, &soul));
                                let anim = std::mem::take(std_map_int_maanim_subscript(&mut ctx.effect_anims, &soul));
                                let frame = get_entity_frame(ctx, faction, slot)?;

                                maanim_execute(&mut model, Some(&anim), frame, 0)?;
                                *std_map_int_maanim_subscript(&mut ctx.effect_anims, &soul) = anim;
                                *std_map_int_mamodel_subscript(&mut ctx.effect_models, &soul) = model;

                                let origin = ctx.i32_at(AppContext::DRAW_TEMP_1)? as f64;
                                let x = ops::cvttsd2si(get_drawable_width(ctx)?.wrapping_add(-0x3c0) as f64 * 0.5 + origin);
                                let y = ctx.i32_at(AppContext::DRAW_TEMP_2)?;

                                draw_model(draw_context(&mut ctx.draw)?, std_map_int_mamodel_subscript(&mut ctx.effect_models, &soul), x, y);
                            }
                        }

                        let depth = ctx.i32_at(AppContext::entity_field(faction, slot, Entity::Z_LAYER))? << 2;

                        wave_draw(ctx, depth, slot, faction.wrapping_add(1))?;
                        surge_draw(ctx, faction, slot)?;
                        counter_surge_draw(ctx, faction, slot)?;
                        explosion_draw(ctx, faction, slot)?;
                    }
                    _ => {}
                }
            }

            record += 1;

            if record >= ctx.i32_at(AppContext::DRAW_TEMP_0)? as i64 {
                break;
            }
        }
    }

    sniper_draw(ctx)?;
    draw_smoke(ctx)?;
    metal_killer_vfx_draw(ctx)?;
    draw_crit(ctx)?;
    draw_zombie_killed(ctx)?;
    barrier_vfx_draw(ctx)?;
    shield_vfx_draw(ctx)?;
    savage_vfx_draw(ctx)?;
    toxic_vfx_draw(ctx)?;
    drain_vfx_draw(ctx)?;
    base_guard_notice_draw(ctx)?;
    draw_cannon_anim(ctx, 0)?;
    draw_cannon_anim(ctx, 1)?;

    let base_x = get_base_pos_x(ctx, 1)?;
    let size = get_castle_row(&ctx.enemy_castle, get_castle_id(ctx)?)?.size;
    let inset = ops::div_neg_100(size.wrapping_mul(0x49c));
    let shifted = ops::div_10(base_x.wrapping_sub(ctx.i32_at(AppContext::CAMERA_X)?).wrapping_add(inset));
    let offset_x = get_castle_row(&ctx.enemy_castle, get_castle_id(ctx)?)?.offset_x;
    let size = get_castle_row(&ctx.enemy_castle, get_castle_id(ctx)?)?.size;

    ctx.set_i32_at(AppContext::DRAW_TEMP_1, ops::div_100(offset_x.wrapping_mul(size)).wrapping_add(shifted))?;

    let base_y = ops::div_10(get_base_pos_y(ctx, 1)?);
    let size = get_castle_row(&ctx.enemy_castle, get_castle_id(ctx)?)?.size;
    let lifted = ops::div_neg_100(size.wrapping_mul(0xff)).wrapping_add(base_y);
    let offset_y = get_castle_row(&ctx.enemy_castle, get_castle_id(ctx)?)?.offset_y;
    let size = get_castle_row(&ctx.enemy_castle, get_castle_id(ctx)?)?.size;

    ctx.set_i32_at(AppContext::DRAW_TEMP_2, ops::div_100(offset_y.wrapping_mul(size)).wrapping_add(lifted))?;

    for slot in 0..0x33 {
        if !is_boss(ctx, 1, slot)? {
            continue;
        }

        if get_shockwave_counter(ctx, 1, slot)? > maanim_get_max_keyframe(&ctx.boss_shockwave_anim)? {
            continue;
        }

        let frame = get_shockwave_counter(ctx, 1, slot)?;

        maanim_execute(&mut ctx.boss_welcome_model, Some(&ctx.boss_shockwave_anim), frame, 0)?;

        let origin = ctx.i32_at(AppContext::DRAW_TEMP_1)? as f64;
        let x = ops::cvttsd2si(get_drawable_width(ctx)?.wrapping_add(-0x3c0) as f64 * 0.5 + origin + 64.0);
        let y = ctx.i32_at(AppContext::DRAW_TEMP_2)?.wrapping_add(0xe0);

        draw_model(draw_context(&mut ctx.draw)?, &ctx.boss_welcome_model, x, y);
    }

    draw_foreground_effects(ctx)?;

    let map_id = get_global_map_id(ctx, 0)?;

    if get_special_rule(ctx, &ctx.special_rules, map_id, 0xc)? && is_fever_active(ctx, &ctx.special_rules)? {
        let scale = ctx.screen_metrics.scale2;

        set_draw_scale(draw_context(&mut ctx.draw)?, scale);

        let frame = get_fever_frame(&ctx.special_rules);

        maanim_execute(&mut ctx.fever_model, Some(&ctx.fever_anim), frame, 0)?;

        let opacity = fever_fade_lerp(&ctx.special_rules, ctx.fever_model.opacity_unit);
        let part = mamodel_get_part(&ctx.fever_model, 0).ok_or(Fault::null_pointer())?;

        ctx.fever_model.parts[part].set_i32_at(0x7c, opacity);

        let x = ops::div_2(get_drawable_width(ctx)?);
        let y = ops::div_2(get_design_height2(ctx));

        draw_model(draw_context(&mut ctx.draw)?, &ctx.fever_model, x, y);

        let scale = ctx.screen_metrics.scale2;

        set_transform(draw_context(&mut ctx.draw)?, scale, &transform);
    }

    if has_castle_enemy(ctx)? {
        let slot = get_entity_base_idx(ctx)?;
        let button = get_entity_button(ctx, 1, slot)?;
        let (side, index) = get_unit_model(ctx, 1, button)?.ok_or(Fault::null_pointer())?;
        let model = &ctx.unit_models[side][index];
        let part_index = mamodel_get_anchor_part(mamodel_get_anchor(model, 1)?);
        let part = model.parts.get(part_index as i64 as usize).ok_or(Fault::index_out_of_range(part_index as i64, model.parts.len() as i64))?;
        let anchor_x = mamodel_get_anchor_x(mamodel_get_anchor(model, 1)?).wrapping_sub(mamodel_get_anchor_x(mamodel_get_anchor(model, 0)?));
        let anchor_y = mamodel_get_anchor_y(mamodel_get_anchor(model, 1)?);
        let mut out = 0i64;

        transform_anchor(part, model, anchor_x, anchor_y, &mut out)?;
        ctx.set_block_at::<8>(AppContext::ANCHOR_OUT, out.to_le_bytes())?;

        let pos_x = ops::div_10(ctx.i32_at(AppContext::entity_field(1, slot, Entity::POS_X))?.wrapping_sub(ctx.i32_at(AppContext::CAMERA_X)?));

        ctx.set_i32_at(AppContext::DRAW_TEMP_1, pos_x)?;

        let pos_y = ops::div_10(ctx.i32_at(AppContext::entity_field(1, slot, Entity::POS_Y))?);
        let depth = ctx.i32_at(AppContext::entity_field(1, slot, Entity::Z_LAYER))?;

        ctx.set_i32_at(AppContext::DRAW_TEMP_2, pos_y.wrapping_add(depth << 2))?;

        let anchor = ctx.i32_at(AppContext::ANCHOR_OUT)?;
        let width = get_drawable_width(ctx)?;
        let shift_x = get_setting(&ctx.settings, b"battle_castle_x2", 0)?;
        let row = ctx.i32_at(AppContext::DRAW_TEMP_2)?;
        let lift = ctx.i32_at(AppContext::ANCHOR_OUT + 4)?;
        let shift_y = get_setting(&ctx.settings, b"battle_castle_y2", 0)?;
        let x = ops::cvttsd2si(width.wrapping_add(-0x3c0) as f64 * 0.5 + anchor.wrapping_add(pos_x) as f64 + shift_x as f64);
        let y = lift.wrapping_add(row).wrapping_add(shift_y);
        let sheet = ctx.img001_sheet.clone();
        let digits = sheet.as_deref().ok_or(Fault::null_pointer())?;

        if get_trait_dojo(ctx, 1, slot)? {
            let stride = imgcut_get_sprite_cut(digits, 0x6c)?[2].wrapping_mul(7);
            let left = stride.wrapping_add(x);

            draw_number_plain(draw_context(&mut ctx.draw)?, digits, 0x6c, 0, 0, left as f32, y as f32, 0.0, 0, 0x12, 7)?;
            draw_cut(draw_context(&mut ctx.draw)?, digits, left, y, 0x43);
            draw_number_plain(draw_context(&mut ctx.draw)?, digits, 0x6c, 0, 0, x.wrapping_add(stride).wrapping_add(0x12) as f32, y as f32, 0.0, 0, 0x10, 7)?;
        } else {
            let hp = get_hp(ctx, 1, slot)?;
            let cap = min_i32(get_max_hp(ctx, 1, slot)?, 0x5f5e0ff);
            let width = imgcut_get_sprite_cut(digits, 0x39)?[2];
            let stride = digit_count(cap).wrapping_mul(width);
            let left = stride.wrapping_add(x);

            draw_number_plain(draw_context(&mut ctx.draw)?, digits, 0x39, hp, 0, left as f32, y as f32, 0.0, 0, 2, 0)?;
            draw_cut(draw_context(&mut ctx.draw)?, digits, left, y, 0x43);
            draw_number_plain(draw_context(&mut ctx.draw)?, digits, 0x39, cap, 0, x.wrapping_add(stride).wrapping_add(0x12) as f32, y as f32, 0.0, 0, 0, 0)?;
        }
    } else {
        let base_x = get_base_pos_x(ctx, 1)?;
        let size = get_castle_row(&ctx.enemy_castle, get_castle_id(ctx)?)?.size;
        let camera = ctx.i32_at(AppContext::CAMERA_X)?;
        let offset_x = get_castle_row(&ctx.enemy_castle, get_castle_id(ctx)?)?.offset_x;
        let scale = get_castle_row(&ctx.enemy_castle, get_castle_id(ctx)?)?.size;
        let hp = ctx.i32_at(AppContext::entity_field(1, 0, Entity::HP))?;
        let cap = min_i32(ctx.i32_at(AppContext::entity_field(1, 0, Entity::MAX_HP))?, 0x5f5e0ff);
        let reach = get_castle_row(&ctx.enemy_castle, get_castle_id(ctx)?)?.size;
        let shift_x = get_setting(&ctx.settings, b"battle_castle_x1", 0)?;
        let shift_y = get_setting(&ctx.settings, b"battle_castle_y1", 0)?;
        let inset = ops::div_neg_100(size.wrapping_mul(0x49c));
        let origin = ops::div_10(base_x.wrapping_sub(camera).wrapping_add(inset));
        let offset = ops::div_100(scale.wrapping_mul(offset_x));
        let middle = ops::div_200((reach << 7).wrapping_sub(reach));
        let x = shift_x.wrapping_add(middle.wrapping_add(offset).wrapping_add(origin)).wrapping_add(0x96);
        let y = shift_y.wrapping_add(0x89);
        let sheet = ctx.img001_sheet.clone();
        let digits = sheet.as_deref().ok_or(Fault::null_pointer())?;
        let left = (get_drawable_width(ctx)?.wrapping_add(-0x3c0) as f64 * 0.5 + x.wrapping_add(-0x4e) as f64 + 14.0) as f32;

        draw_number_plain(draw_context(&mut ctx.draw)?, digits, 0x39, hp, 0, left, y as f32, 0.0, 0, 2, 0)?;

        let slash = ops::cvttsd2si(get_drawable_width(ctx)?.wrapping_add(-0x3c0) as f64 * 0.5 + x.wrapping_add(-0x40) as f64);

        draw_cut_scaled(draw_context(&mut ctx.draw)?, digits, slash, y, 0x12, 0x12, 0x43);

        let right = (get_drawable_width(ctx)?.wrapping_add(-0x3c0) as f64 * 0.5 + x.wrapping_add(-0x2e) as f64) as f32;

        draw_number_plain(draw_context(&mut ctx.draw)?, digits, 0x39, cap, 0, right, y as f32, 0.0, 0, 0, 0)?;
    }

    let base_x = get_base_pos_x(ctx, 0)?;
    let camera = ctx.i32_at(AppContext::CAMERA_X)?;
    let hp = ctx.i32_at(AppContext::entity_field(0, 0, Entity::HP))?;
    let cap = min_i32(ctx.i32_at(AppContext::entity_field(0, 0, Entity::MAX_HP))?, 0x5f5e0ff);
    let shift_x = get_setting(&ctx.settings, b"battle_castle_x0", 0)?;
    let shift_y = get_setting(&ctx.settings, b"battle_castle_y0", 0)?;
    let x = shift_x.wrapping_add(ops::div_10(base_x.wrapping_sub(camera))).wrapping_add(0x50);
    let y = shift_y.wrapping_add(0x89);
    let sheet = ctx.img001_sheet.clone();
    let digits = sheet.as_deref().ok_or(Fault::null_pointer())?;
    let left = (get_drawable_width(ctx)?.wrapping_add(-0x3c0) as f64 * 0.5 + x.wrapping_add(-0x76) as f64 + 14.0) as f32;

    draw_number_plain(draw_context(&mut ctx.draw)?, digits, 0x39, hp, 0, left, y as f32, 0.0, 0, 2, 0)?;

    let slash = ops::cvttsd2si(get_drawable_width(ctx)?.wrapping_add(-0x3c0) as f64 * 0.5 + x.wrapping_add(-0x68) as f64);

    draw_cut_scaled(draw_context(&mut ctx.draw)?, digits, slash, y, 0x12, 0x12, 0x43);

    let right = (get_drawable_width(ctx)?.wrapping_add(-0x3c0) as f64 * 0.5 + x.wrapping_add(-0x56) as f64) as f32;

    draw_number_plain(draw_context(&mut ctx.draw)?, digits, 0x39, cap, 0, right, y as f32, 0.0, 0, 0, 0)?;

    let skip_button = ctx.i32_at(AppContext::ENTRY_STAGE)? == 0x30
        || ctx.i32_at(AppContext::CHAPTER_MODE)? == 3
        || ctx.i32_at(AppContext::CHAPTER_MODE)? == 0x63
        || ctx.u8_at(AppContext::BATTLE_IS_OUTBREAK)? != 0
        || ctx.u8_at(AppContext::BATTLE_IS_INVASION)? != 0
        || ctx.u8_at(AppContext::BATTLE_IS_Z_INVASION)? != 0
        || (get_map_type(ctx, 0)? == -7 && get_stage_index(ctx)? == 0x2f)
        || ctx.i32_at(AppContext::CAT_GOD_AVAILABLE)? <= 0;

    if !skip_button {
        let press = ctx.i32_at(AppContext::CAT_GOD_BUTTON_PRESS)?;
        let bounce = *BUTTON_PRESS_BOUNCE.get(press as i64 as usize).ok_or(Fault::index_out_of_range(press as i64, BUTTON_PRESS_BOUNCE.len() as i64))?;
        let centre = ops::div_10(ops::div_2(ctx.i32_at(AppContext::STAGE_LENGTH)?).wrapping_sub(ctx.i32_at(AppContext::CAMERA_X)?));
        let origin = centre.wrapping_sub(ops::div_2(bounce)).wrapping_add(-0x41) as f64;
        let x = ops::cvttsd2si(get_drawable_width(ctx)?.wrapping_add(-0x3c0) as f64 * 0.5 + origin);
        let sink = ctx
            .i32_at(AppContext::LETTERBOX_SHIFT)?
            .wrapping_add(ctx.i32_at(AppContext::CAT_GOD_BUTTON_SINK)?)
            .wrapping_add(ctx.i32_at(AppContext::DECK_BAR_SLIDE)?)
            .wrapping_add(ops::div_2(bounce));
        let size = bounce.wrapping_add(0x83);
        let spin = ctx.f32_at(AppContext::CAT_GOD_SPIN)?;
        let sheet = ctx.img002_sheet.clone();
        let icons = sheet.as_deref().ok_or(Fault::null_pointer())?;

        draw_cut_rotated(draw_context(&mut ctx.draw)?, icons, x, (-0x18i32).wrapping_sub(sink), size, size, spin, 0, 0, 0, 5, 0x28);

        let press = ctx.i32_at(AppContext::CAT_GOD_BUTTON_PRESS)?;
        let bounce = *BUTTON_PRESS_BOUNCE.get(press as i64 as usize).ok_or(Fault::index_out_of_range(press as i64, BUTTON_PRESS_BOUNCE.len() as i64))?;
        let centre = ops::div_10(ops::div_2(ctx.i32_at(AppContext::STAGE_LENGTH)?).wrapping_sub(ctx.i32_at(AppContext::CAMERA_X)?));
        let origin = centre.wrapping_sub(ops::div_2(bounce)).wrapping_add(-0x26) as f64;
        let x = ops::cvttsd2si(get_drawable_width(ctx)?.wrapping_add(-0x3c0) as f64 * 0.5 + origin);
        let press = ctx.i32_at(AppContext::CAT_GOD_BUTTON_PRESS)?;
        let bounce = *BUTTON_PRESS_BOUNCE.get(press as i64 as usize).ok_or(Fault::index_out_of_range(press as i64, BUTTON_PRESS_BOUNCE.len() as i64))?;
        let sink = ctx
            .i32_at(AppContext::CAT_GOD_BUTTON_SINK)?
            .wrapping_add(ctx.i32_at(AppContext::LETTERBOX_SHIFT)?)
            .wrapping_add(ctx.i32_at(AppContext::DECK_BAR_SLIDE)?)
            .wrapping_add(ops::div_2(bounce));
        let size = bounce.wrapping_add(0x4c);

        draw_cut_scaled(draw_context(&mut ctx.draw)?, icons, x, 1i32.wrapping_sub(sink), size, size, 0x29);
    }

    let scale = ctx.screen_metrics.scale2;

    set_draw_scale(draw_context(&mut ctx.draw)?, scale);
    set_tint_alpha(draw_context(&mut ctx.draw)?, 0xff);
    draw_background_overlay(ctx)?;

    if get_castle_anim_state(ctx, 0)? == 2 && (get_castle_anim_frame(ctx, 0)? == 1 || get_castle_anim_frame(ctx, 0)? == 2) {
        set_tint(draw_context(&mut ctx.draw)?, 0xff, 0xff, 0xff, 0xff);

        let top = 0i32.wrapping_sub(ctx.i32_at(AppContext::LETTERBOX_SHIFT)?);
        let width = get_drawable_width(ctx)?;
        let height = get_design_height2(ctx);

        fill_rect(draw_context(&mut ctx.draw)?, 0, top, width, height);
    }

    set_alpha(draw_context(&mut ctx.draw)?, 0xff);

    let x = get_left_inset_logical(ctx).wrapping_add(0x3e);
    let y = 10i32.wrapping_sub(ctx.i32_at(AppContext::LETTERBOX_SHIFT)?);

    draw_cut(draw_context(&mut ctx.draw)?, ctx.stage_name_sheet.as_deref().ok_or(Fault::null_pointer())?, x, y, 0);

    let mut clock = None;

    if is_scored_stage(ctx)? {
        let state = ctx.i32_at(AppContext::SCORE_CHANGED)?;

        if state == 2 {
            let total = ctx.i32_at(AppContext::SCORE_TOTAL)?;
            let tick = ctx.i32_at(AppContext::SCORE_ANIM_TICK)?;

            ctx.set_i32_at(AppContext::SCORE_ZOOM, 0x96i32.wrapping_sub(tick.wrapping_mul(8)))?;
            ctx.set_i32_at(AppContext::SCORE_FROM, total)?;
            ctx.set_i32_at(AppContext::SCORE_SHOWN, total)?;
            ctx.set_i32_at(AppContext::SCORE_ANIM_TICK, tick.wrapping_add(1))?;

            if tick >= 5 {
                ctx.set_block_at::<8>(AppContext::SCORE_CHANGED, [0; 8])?;
            }
        } else if state == 1 {
            let total = ctx.i32_at(AppContext::SCORE_TOTAL)?;
            let tick = ctx.i32_at(AppContext::SCORE_ANIM_TICK)?;

            ctx.set_i32_at(AppContext::SCORE_ZOOM, tick.wrapping_mul(0xc).wrapping_add(100))?;

            let from = ctx.i32_at(AppContext::SCORE_FROM)?;
            let step = ops::div_4(total.wrapping_sub(from));

            ctx.set_i32_at(AppContext::SCORE_SHOWN, step.wrapping_mul(tick).wrapping_add(from))?;
            ctx.set_i32_at(AppContext::SCORE_ANIM_TICK, tick.wrapping_add(1))?;

            if tick >= 3 {
                ctx.set_i32_at(AppContext::SCORE_CHANGED, 2)?;
                ctx.set_i32_at(AppContext::SCORE_ANIM_TICK, 0)?;
            }
        } else if state == 0 {
            ctx.set_i32_at(AppContext::SCORE_ZOOM, 100)?;

            let total = ctx.i32_at(AppContext::SCORE_TOTAL)?;

            ctx.set_i32_at(AppContext::SCORE_FROM, total)?;
            ctx.set_i32_at(AppContext::SCORE_SHOWN, total)?;
        }

        let sheet = ctx.img001_sheet.clone();
        let digits = sheet.as_deref().ok_or(Fault::null_pointer())?;
        let x = ops::div_2(get_drawable_width(ctx)?).wrapping_add(-0x86);
        let lift = ctx.i32_at(AppContext::DECK_BAR_SLIDE)?.wrapping_add(ctx.i32_at(AppContext::LETTERBOX_SHIFT)?);
        let y = 10i32.wrapping_sub(lift);
        let border_x = imgcut_get_sprite_cut(digits, 0x6b)?[0].wrapping_sub(imgcut_get_sprite_cut(digits, 0x6a)?[0]);
        let border_y = imgcut_get_sprite_cut(digits, 0x6b)?[1].wrapping_sub(imgcut_get_sprite_cut(digits, 0x6a)?[1]);
        let inner_w = imgcut_get_sprite_cut(digits, 0x6b)?[2];
        let inner_h = imgcut_get_sprite_cut(digits, 0x6b)?[3];

        draw_nine_slice(draw_context(&mut ctx.draw)?, digits, x, y, 0x10d, 0x2a, 1.0, 0x6a, border_x, border_y, inner_w, inner_h);

        let x = ops::div_2(get_drawable_width(ctx)?).wrapping_add(-0x7a);
        let lift = ctx.i32_at(AppContext::DECK_BAR_SLIDE)?.wrapping_add(ctx.i32_at(AppContext::LETTERBOX_SHIFT)?);

        draw_cut(draw_context(&mut ctx.draw)?, digits, x, 0x11i32.wrapping_sub(lift), 0x68);

        let x = ops::div_2(get_drawable_width(ctx)?).wrapping_add(0x64);
        let lift = ctx.i32_at(AppContext::DECK_BAR_SLIDE)?.wrapping_add(ctx.i32_at(AppContext::LETTERBOX_SHIFT)?);

        draw_cut(draw_context(&mut ctx.draw)?, digits, x, 0x13i32.wrapping_sub(lift), 0x69);

        let lift = ctx.i32_at(AppContext::DECK_BAR_SLIDE)?.wrapping_add(ctx.i32_at(AppContext::LETTERBOX_SHIFT)?);
        let y = ops::div_2(imgcut_get_sprite_cut(digits, 0xe)?[3]).wrapping_sub(lift).wrapping_add(0x11);
        let centre = ops::div_2(get_drawable_width(ctx)?);
        let pitch = imgcut_get_sprite_cut(digits, 0xe)?[2].wrapping_add(-1);
        let span = digit_count(ctx.i32_at(AppContext::SCORE_SHOWN)?).wrapping_mul(pitch);
        let x = centre.wrapping_sub(ops::div_2(span)).wrapping_add(0x64);
        let zoom = ctx.i32_at(AppContext::SCORE_ZOOM)? as f32 / 100.0;
        let shown = ctx.i32_at(AppContext::SCORE_SHOWN)?;

        draw_number_scaled(draw_context(&mut ctx.draw)?, digits, 0xe, shown, 0, x as f32, y as f32, -1.0, zoom, 0, 5, 0)?;

        if get_battle_status(ctx)? != 4 {
            let left = get_score_time_limit(ctx)?.wrapping_sub(ctx.i32_at(AppContext::SCORE_ELAPSED)?);

            clock = Some(if left > 0 { left } else { 0 });
        }
    } else if is_score_stage(ctx.event_items.as_ref()) {
        let sheet = ctx.img001_sheet.clone();
        let digits = sheet.as_deref().ok_or(Fault::null_pointer())?;
        let x = ops::div_2(get_drawable_width(ctx)?).wrapping_add(-0x86);
        let lift = ctx.i32_at(AppContext::DECK_BAR_SLIDE)?.wrapping_add(ctx.i32_at(AppContext::LETTERBOX_SHIFT)?);
        let y = 10i32.wrapping_sub(lift);
        let border_x = imgcut_get_sprite_cut(digits, 0x6b)?[0].wrapping_sub(imgcut_get_sprite_cut(digits, 0x6a)?[0]);
        let border_y = imgcut_get_sprite_cut(digits, 0x6b)?[1].wrapping_sub(imgcut_get_sprite_cut(digits, 0x6a)?[1]);
        let inner_w = imgcut_get_sprite_cut(digits, 0x6b)?[2];
        let inner_h = imgcut_get_sprite_cut(digits, 0x6b)?[3];

        draw_nine_slice(draw_context(&mut ctx.draw)?, digits, x, y, 0x10d, 0x2a, 1.0, 0x6a, border_x, border_y, inner_w, inner_h);

        let x = ops::div_2(get_drawable_width(ctx)?).wrapping_add(-0x77);
        let lift = ctx.i32_at(AppContext::DECK_BAR_SLIDE)?.wrapping_add(ctx.i32_at(AppContext::LETTERBOX_SHIFT)?);

        draw_cut(draw_context(&mut ctx.draw)?, digits, x, 0xfi32.wrapping_sub(lift), 0x9f);

        let x = ops::div_2(get_drawable_width(ctx)?).wrapping_add(0x64);
        let lift = ctx.i32_at(AppContext::DECK_BAR_SLIDE)?.wrapping_add(ctx.i32_at(AppContext::LETTERBOX_SHIFT)?);

        draw_cut(draw_context(&mut ctx.draw)?, digits, x, 0x13i32.wrapping_sub(lift), 0x69);

        let lift = ctx.i32_at(AppContext::DECK_BAR_SLIDE)?.wrapping_add(ctx.i32_at(AppContext::LETTERBOX_SHIFT)?);
        let y = ops::div_2(imgcut_get_sprite_cut(digits, 0xe)?[3]).wrapping_sub(lift).wrapping_add(0x11);
        let score = get_stage_score(ctx.event_items.as_ref().ok_or(Fault::null_pointer())?);
        let centre = ops::div_2(get_drawable_width(ctx)?);
        let pitch = imgcut_get_sprite_cut(digits, 0xe)?[2].wrapping_add(-1);
        let x = centre.wrapping_sub(ops::div_2(digit_count(score).wrapping_mul(pitch))).wrapping_add(0x64);

        draw_number_plain(draw_context(&mut ctx.draw)?, digits, 0xe, score, 0, x as f32, y as f32, -1.0, 0, 5, 0)?;

        let map_id = get_global_map_id(ctx, 0)?;

        if get_special_rule(ctx, &ctx.special_rules, map_id, 0xc)? {
            let gauge = ctx.fever_sheet.clone();
            let gauge = gauge.as_deref().ok_or(Fault::null_pointer())?;
            let left = ops::div_2(get_drawable_width(ctx)?).wrapping_sub(ops::div_2(imgcut_get_sprite_cut(gauge, 8)?[2]));
            let lift = ctx.i32_at(AppContext::LETTERBOX_SHIFT)?.wrapping_add(ctx.i32_at(AppContext::DECK_BAR_SLIDE)?);
            let top = 0x46i32.wrapping_sub(lift);
            let fill = if is_fever_active(ctx, &ctx.special_rules)? {
                let frame = get_fever_frame(&ctx.special_rules);
                let blink = ops::div_3(frame);
                let blink = blink.wrapping_sub((blink.wrapping_add((blink as u32 >> 31) as i32)) & -2).wrapping_add(8);

                draw_cut(draw_context(&mut ctx.draw)?, gauge, left, top, blink);

                let remaining = ops::cvttsd2si((1.0 - fever_time_fraction(ctx)?) * 294.0);

                if remaining > 0 { Some((left.wrapping_sub(remaining).wrapping_add(0x12d), remaining)) } else { None }
            } else {
                draw_cut(draw_context(&mut ctx.draw)?, gauge, left, top, 8);

                let pixels = fever_gauge_pixels(ctx, score)?;

                if pixels > 0x125 { None } else { Some((left.wrapping_add(pixels).wrapping_add(7), 0x126i32.wrapping_sub(pixels))) }
            };

            if let Some((x, width)) = fill {
                draw_cut_scaled(draw_context(&mut ctx.draw)?, gauge, x, 0x4di32.wrapping_sub(lift), width, 0x20, 7);
            }

            let x = ops::div_2(imgcut_get_sprite_cut(gauge, 8)?[2]).wrapping_add(left).wrapping_sub(ops::div_2(imgcut_get_sprite_cut(gauge, 6)?[2]));
            let y = ops::div_2(imgcut_get_sprite_cut(gauge, 8)?[3]).wrapping_add(top).wrapping_sub(ops::div_2(imgcut_get_sprite_cut(gauge, 6)?[3]));

            draw_cut(draw_context(&mut ctx.draw)?, gauge, x, y, 6);
        }

        if get_battle_status(ctx)? == 0 && has_point_decay(ctx.event_items.as_ref().ok_or(Fault::null_pointer())?) {
            let left = point_decay_remaining(ctx.event_items.as_ref().ok_or(Fault::null_pointer())?);

            clock = Some(if left > 0 { left } else { 0 });
        }
    }

    if let Some(frames) = clock {
        if frames as u32 <= 0x384 {
            set_color(draw_context(&mut ctx.draw)?, 0xff, 0, 0, 0xff);
        }

        let frames = frames as u32;
        let minutes = ((frames as u64 * 0x91a2b3c5) >> 0x2a) as i32;
        let seconds = ((frames as u64 * 0x88888889) >> 0x24) as u32;
        let wrapped = seconds.wrapping_sub((((seconds as u64 * 0x8888889) >> 0x21) as u32).wrapping_mul(0x3c)) as i32;
        let ticks = frames.wrapping_sub(seconds.wrapping_mul(0x1e)) as i32;
        let hundredths = ((ticks.wrapping_mul(100) as u16 as u32 * 0x889) >> 0x10) as i32;
        let sheet = ctx.img001_sheet.clone();
        let digits = sheet.as_deref().ok_or(Fault::null_pointer())?;

        ctx.set_i32_at(AppContext::DRAW_TEMP_1, 8)?;
        ctx.set_i32_at(AppContext::DRAW_TEMP_2, 0x40)?;

        let y = 0x40i32.wrapping_sub(ctx.i32_at(AppContext::LETTERBOX_SHIFT)?);

        draw_number_plain(draw_context(&mut ctx.draw)?, digits, 0x53, minutes, 0, 8.0, y as f32, 0.0, 0, 0x10, 2)?;

        let x = ctx.i32_at(AppContext::DRAW_TEMP_1)?.wrapping_add(0x30);

        ctx.set_i32_at(AppContext::DRAW_TEMP_1, x)?;

        let y = ctx.i32_at(AppContext::DRAW_TEMP_2)?.wrapping_sub(ctx.i32_at(AppContext::LETTERBOX_SHIFT)?);

        draw_cut_scaled(draw_context(&mut ctx.draw)?, digits, x, y, 0x1b, 0x2e, 0x5d);

        let x = ctx.i32_at(AppContext::DRAW_TEMP_1)?.wrapping_add(0x1b);

        ctx.set_i32_at(AppContext::DRAW_TEMP_1, x)?;

        let y = ctx.i32_at(AppContext::DRAW_TEMP_2)?.wrapping_sub(ctx.i32_at(AppContext::LETTERBOX_SHIFT)?);

        draw_number_plain(draw_context(&mut ctx.draw)?, digits, 0x53, wrapped, 0, x as f32, y as f32, 0.0, 0, 0x10, 2)?;

        let x = ctx.i32_at(AppContext::DRAW_TEMP_1)?.wrapping_add(0x30);

        ctx.set_i32_at(AppContext::DRAW_TEMP_1, x)?;

        let y = ctx.i32_at(AppContext::DRAW_TEMP_2)?.wrapping_sub(ctx.i32_at(AppContext::LETTERBOX_SHIFT)?).wrapping_add(0x2e) as f32 + -38.333332;

        draw_cut_f(draw_context(&mut ctx.draw)?, digits, 0x5d, x as f32, y, 22.5, 38.333332);

        let x = ctx.i32_at(AppContext::DRAW_TEMP_1)?.wrapping_add(0x16);

        ctx.set_i32_at(AppContext::DRAW_TEMP_1, x)?;

        let y = ctx.i32_at(AppContext::DRAW_TEMP_2)?.wrapping_sub(ctx.i32_at(AppContext::LETTERBOX_SHIFT)?);

        draw_number_plain(draw_context(&mut ctx.draw)?, digits, 0x53, hundredths, 0, x as f32, y as f32, 0.0, 0, 0x10, 2)?;
        set_color(draw_context(&mut ctx.draw)?, 0xff, 0xff, 0xff, 0xff);
    }

    draw_money(ctx)?;

    let map_id = get_global_map_id(ctx, 0)?;
    if let Some(params) = get_special_rule_params(ctx, &ctx.special_rules, map_id, 7)? {
        let limit = *params.first().ok_or(Fault::out_of_range())?;
        let lift = ctx.i32_at(AppContext::LETTERBOX_SHIFT)?.wrapping_add(ctx.i32_at(AppContext::DECK_BAR_SLIDE)?);
        let y = 0xbi32.wrapping_sub(lift);
        let left = limit.wrapping_sub(ctx.i32_at(AppContext::DEPLOY_LIMIT_TOTAL)?);

        if left <= 10 {
            set_color(draw_context(&mut ctx.draw)?, 0xff, 0, 0, 0xff);
        } else if left as u32 <= 0x14 {
            set_color(draw_context(&mut ctx.draw)?, 0xff, 0xf0, 0x41, 0xff);
        }

        let sheet = ctx.img001_sheet.clone();
        let digits = sheet.as_deref().ok_or(Fault::null_pointer())?;
        let pitch = imgcut_get_sprite_cut(digits, 0x8d)?[2];
        let x = ops::div_2(get_drawable_width(ctx)?.wrapping_add(-0x3c0)).wrapping_add(pitch).wrapping_add(0x1f1);
        let bounds = draw_number_plain(draw_context(&mut ctx.draw)?, digits, 0x8d, left, 0, x as f32, y as f32, -5.0, 0, 2, 0)?;
        let edge = bounds.left + -3.0;
        let x = ops::cvttss2si(edge - imgcut_get_sprite_cut(digits, 0x97)?[2] as f32);

        draw_cut(draw_context(&mut ctx.draw)?, digits, x, 0x14i32.wrapping_sub(lift), 0x97);
        draw_cut(draw_context(&mut ctx.draw)?, digits, ops::cvttss2si(bounds.right), 0x12i32.wrapping_sub(lift), 0x98);
        set_color(draw_context(&mut ctx.draw)?, 0xff, 0xff, 0xff, 0xff);
    }

    if ctx.u8_at(AppContext::UNIT_INFO_OVERLAY_OPEN)? == 0 {
        draw_deck(ctx, 0)?;

        if ctx.i32_at(AppContext::TUTORIAL_CLEARED)? == 0 {
            let money = get_money(ctx, AppContext::faction_flags(0))?;
            let needed = get_setting(&ctx.settings, b"tutorial_money", 0x4b)?.wrapping_mul(100);
            let untouched = ctx.i32_at(AppContext::faction_flags(0) + AppContext::WALLET_DEPLOY_COUNTS)? == 0;

            if (money >= needed) & untouched {
                let origin = DECK_SLOT_X_TABLE[0].wrapping_add(7) as f64;
                let x = ops::cvttsd2si(get_drawable_width(ctx)?.wrapping_add(-0x3c0) as f64 * 0.5 + origin);
                let base = 0x192i32.wrapping_add(ctx.i32_at(AppContext::LETTERBOX_SHIFT)?) as f32;
                let ticks = ctx.i32_at(AppContext::BATTLE_TICKS)?;
                let y = ops::cvttss2si(sin_deg((ticks << 5).wrapping_sub(ticks.wrapping_add(ticks)) as f32) * 10.0 + base);

                draw_cut_scaled(draw_context(&mut ctx.draw)?, ctx.img039_sheet.as_deref().ok_or(Fault::null_pointer())?, x, y, 0x60, 0x60, 0);
            }
        }
    }

    draw_cannon(ctx)?;
    draw_worker_cat(ctx)?;

    if get_global_map_id(ctx, 0)? == 0xbb8 && get_stage_index(ctx)? == 1 && get_stage_record(ctx, -2, 0, 1, 0, 0)? == 0 {
        let elapsed = ctx.i32_at(AppContext::BATTLE_FRAME_COUNTER)?;

        if elapsed >= get_setting(&ctx.settings, b"tutorial_duration0", 0x384)? && get_worker_level(ctx, AppContext::faction_flags(0))? == 0 {
            let x = get_left_inset_logical(ctx).wrapping_add(0x1e);
            let base = 0x192i32.wrapping_add(ctx.i32_at(AppContext::LETTERBOX_SHIFT)?) as f32;
            let ticks = ctx.i32_at(AppContext::BATTLE_TICKS)?;
            let y = sin_deg((ticks << 5).wrapping_sub(ticks.wrapping_add(ticks)) as f32) * 10.0 + base;
            let y = ops::cvttss2si(get_top_inset_offset(ctx) as f32 + y);

            draw_cut_scaled(draw_context(&mut ctx.draw)?, ctx.img039_sheet.as_deref().ok_or(Fault::null_pointer())?, x, y, 0x60, 0x60, 0);
        }
    }

    if get_global_map_id(ctx, 0)? == 0xbb8
        && get_stage_index(ctx)? == 2
        && get_stage_record(ctx, -2, 0, 2, 0, 0)? == 0
        && get_cannon_countdown(ctx, 0)? == 0
        && ctx.u8_at(AppContext::faction_flags(0) + AppContext::WALLET_CANNON_FIRED)? == 0
    {
        let base = 0x192i32.wrapping_add(ctx.i32_at(AppContext::LETTERBOX_SHIFT)?) as f32;
        let x = ctx.i32_at(AppContext::CANNON_RECT)?.wrapping_add(0x1e);
        let ticks = ctx.i32_at(AppContext::BATTLE_TICKS)?;
        let y = ops::cvttss2si(sin_deg((ticks << 5).wrapping_sub(ticks.wrapping_add(ticks)) as f32) * 10.0 + base);

        draw_cut_scaled(draw_context(&mut ctx.draw)?, ctx.img039_sheet.as_deref().ok_or(Fault::null_pointer())?, x, y, 0x60, 0x60, 0);
    }

    ctx.set_i32_at(AppContext::DRAW_TEMP_1, 8)?;
    ctx.set_i32_at(AppContext::DRAW_TEMP_2, 0x40)?;

    let boom = ctx.i32_at(AppContext::BABY_BOOM_FRAMES)?;

    ctx.set_i32_at(AppContext::DRAW_TEMP_3, ops::div_neg_30(boom.wrapping_mul(100)).wrapping_add(0x1770))?;

    if ctx.i32_at(AppContext::BABY_BOOM_ACTIVE)? == 1 {
        if boom >= 0x5dd {
            set_color(draw_context(&mut ctx.draw)?, 0xff, 0, 0, 0xff);
        } else {
            set_color(draw_context(&mut ctx.draw)?, 0xff, 0xff, 0xff, 0xff);
        }

        let fade = 0xffi32.wrapping_sub(ctx.i32_at(AppContext::CAT_GOD_FRAMES)?);
        let alpha = if ctx.i32_at(AppContext::CAT_GOD_SELECTED)? != 3 { 0xff } else { fade };

        set_alpha(draw_context(&mut ctx.draw)?, alpha);

        let sheet = ctx.img001_sheet.clone();
        let digits = sheet.as_deref().ok_or(Fault::null_pointer())?;
        let y = ctx.i32_at(AppContext::DRAW_TEMP_2)?.wrapping_sub(ctx.i32_at(AppContext::LETTERBOX_SHIFT)?).wrapping_add(0x2e) as f32;
        let minutes = ops::div_6000(ctx.i32_at(AppContext::DRAW_TEMP_3)?);
        let bounds = draw_number_plain(draw_context(&mut ctx.draw)?, digits, 0x53, minutes, 0, 8.0, y, 1.0, 0, 0x18, 2)?;
        let x = ops::cvttss2si(bounds.right);
        let y = ctx.i32_at(AppContext::DRAW_TEMP_2)?.wrapping_sub(ctx.i32_at(AppContext::LETTERBOX_SHIFT)?);

        draw_cut_scaled(draw_context(&mut ctx.draw)?, digits, x, y, 0x1b, 0x2e, 0x5d);

        let x = x.wrapping_add(0x1b);
        let y = ctx.i32_at(AppContext::DRAW_TEMP_2)?.wrapping_sub(ctx.i32_at(AppContext::LETTERBOX_SHIFT)?).wrapping_add(0x2e) as f32;
        let seconds = ops::div_100(ctx.i32_at(AppContext::DRAW_TEMP_3)?);
        let bounds = draw_number_plain(draw_context(&mut ctx.draw)?, digits, 0x53, seconds, 0, x as f32, y, 0.0, 0, 0x18, 2)?;
        let x = ops::cvttss2si(bounds.right);
        let y = ctx.i32_at(AppContext::DRAW_TEMP_2)?.wrapping_sub(ctx.i32_at(AppContext::LETTERBOX_SHIFT)?).wrapping_add(0x2e) as f32 + -38.333332;

        draw_cut_f(draw_context(&mut ctx.draw)?, digits, 0x5d, ops::cvttss2si(bounds.right) as f32, y, 22.5, 38.333332);

        let x = x.wrapping_add(0x16);
        let y = ctx.i32_at(AppContext::DRAW_TEMP_2)?.wrapping_sub(ctx.i32_at(AppContext::LETTERBOX_SHIFT)?).wrapping_add(0x2e) as f32;
        let total = ctx.i32_at(AppContext::DRAW_TEMP_3)?;
        let hundredths = total.wrapping_sub(ops::div_100(total).wrapping_mul(100));

        draw_number_scaled(draw_context(&mut ctx.draw)?, digits, 0x53, hundredths, 0, x as f32, y, 0.0, 0.8333333, 0, 0x18, 2)?;
    }

    set_color(draw_context(&mut ctx.draw)?, 0xff, 0xff, 0xff, 0xff);

    let press = ctx.i32_at(AppContext::PAUSE_PRESS)?;
    let bounce = *BUTTON_PRESS_BOUNCE.get(press as i64 as usize).ok_or(Fault::index_out_of_range(press as i64, 6))?;
    let sink = ops::div_2(bounce).wrapping_add(ctx.i32_at(AppContext::DECK_BAR_SLIDE)?);
    let inset = get_left_inset_logical(ctx);
    let press = ctx.i32_at(AppContext::PAUSE_PRESS)?;
    let bounce = *BUTTON_PRESS_BOUNCE.get(press as i64 as usize).ok_or(Fault::index_out_of_range(press as i64, 6))?;
    let drop = ops::div_2(bounce).wrapping_add(ctx.i32_at(AppContext::LETTERBOX_SHIFT)?);
    let size = bounce.wrapping_add(0x3a);

    draw_cut_scaled(
        draw_context(&mut ctx.draw)?,
        ctx.img002_sheet.as_deref().ok_or(Fault::null_pointer())?,
        inset.wrapping_sub(sink).wrapping_add(4),
        4i32.wrapping_sub(drop),
        size,
        size,
        2,
    );
    draw_powerup_bar(ctx)?;

    if ctx.i32_at(AppContext::TUTORIAL_CLEARED)? == 0 && ctx.i32_at(AppContext::TUTORIAL_STEP)? == 3 {
        let width = get_drawable_width(ctx)?;
        let lift = ctx.i32_at(AppContext::LETTERBOX_SHIFT)?.wrapping_add(ctx.i32_at(AppContext::DECK_BAR_SLIDE)?);
        let inset = get_right_inset_logical(ctx)?.wrapping_neg();
        let top = ctx.i32_at(AppContext::LETTERBOX_PAD)?.wrapping_add(ctx.i32_at(AppContext::LETTERBOX_SHIFT)?);

        set_draw_origin(ctx, inset, top)?;
        set_flip(draw_context(&mut ctx.draw)?, 2);

        let base = 0xd9i32.wrapping_sub(lift) as f32;
        let ticks = ctx.i32_at(AppContext::BATTLE_TICKS)?;
        let y = ops::cvttss2si(sin_deg((ticks << 5).wrapping_sub(ticks.wrapping_add(ticks)).wrapping_add(0xb4) as f32) * 10.0 + base + -80.0);

        draw_cut_scaled(draw_context(&mut ctx.draw)?, ctx.img039_sheet.as_deref().ok_or(Fault::null_pointer())?, width.wrapping_add(-0x5c), y, 0x60, 0x60, 0);
        set_flip(draw_context(&mut ctx.draw)?, 0);

        let top = ctx.i32_at(AppContext::LETTERBOX_PAD)?.wrapping_add(ctx.i32_at(AppContext::LETTERBOX_SHIFT)?);

        set_draw_origin(ctx, 0, top)?;
    }

    let waited = if ctx.i32_at(AppContext::DECK_HOLD_SLOT)? != -1 {
        ctx.i32_at(AppContext::DECK_HOLD_FRAMES)? >= get_setting(&ctx.settings, b"battle_slot_desc_wait", 6)?
    } else {
        false
    };

    if (waited || ctx.i32_at(AppContext::DECK_HOLD_RELEASE)? > 0) && ctx.i32_at(AppContext::DECK_HOLD_SLOT)? >= 0 {
        {
            let held = ctx.i32_at(AppContext::DECK_HOLD_FRAMES)?;
            let wait = get_setting(&ctx.settings, b"battle_slot_desc_wait", 6)?;
            let stage = if held < wait {
                ctx.i32_at(AppContext::DECK_HOLD_RELEASE)?
            } else {
                let held = ctx.i32_at(AppContext::DECK_HOLD_FRAMES)?;
                let wait = get_setting(&ctx.settings, b"battle_slot_desc_wait", 6)?;

                min_i32(held.wrapping_sub(wait).wrapping_add(1), 3)
            };

            if stage >= 0 {
                let slot = ctx.i32_at(AppContext::DECK_HOLD_SLOT)?;
                let unit = get_button_unit_id(ctx, 0, slot)?;
                let slot = ctx.i32_at(AppContext::DECK_HOLD_SLOT)?;
                let form = get_button_unit_form(ctx, 0, slot)?;
                let conjure = stat_conjure_unit_id(ctx, 0, unit, form)?;
                let grow = *PANEL_GROWTH_TABLE.get(stage as i64 as usize).ok_or(Fault::index_out_of_range(stage as i64, 4))?;
                let span = grow.wrapping_mul(0x3b2);
                let width = ops::div_100(span);
                let height = ops::div_100(grow.wrapping_mul(0xb4));
                let mut rows = 0i32;
                let mut height = height;

                if conjure >= 0 {
                    rows = if ctx.unit_info_texts[1].is_none() {
                        0
                    } else if ctx.unit_info_texts[2].is_none() {
                        1
                    } else if ctx.unit_info_texts[3].is_none() {
                        2
                    } else {
                        3
                    };

                    height = height.wrapping_add(rows.wrapping_mul(0x27));
                }

                let lead = rows.wrapping_mul(0x27).wrapping_sub(height);
                let x = ops::div_2(get_drawable_width(ctx)?).wrapping_add(ops::div_neg_200(span));
                let y = ops::div_2(lead.wrapping_add(0xb4)).wrapping_add(0x5d);
                let sheet = ctx.img002_sheet.clone();
                let panel = sheet.as_deref().ok_or(Fault::null_pointer())?;

                draw_panel(draw_context(&mut ctx.draw)?, panel, x, y, width, height, 1.0, 0x30, 0x31);

                if unit >= 0 && ctx.i32_at(AppContext::DECK_HOLD_SLOT)? != -1 {
                    let held = ctx.i32_at(AppContext::DECK_HOLD_FRAMES)?;
                    let wait = get_setting(&ctx.settings, b"battle_slot_desc_wait", 6)?;

                    if stage as u32 >= 3 && held >= wait {
                        set_tint(draw_context(&mut ctx.draw)?, 0xff, 0xff, 0xff, 0xff);

                        let x = ops::div_2(get_drawable_width(ctx)?);
                        let line = ctx.unit_info_texts[0];

                        draw_surface_aligned(draw_context(&mut ctx.draw)?, Surface::Label(&line.ok_or(Fault::null_pointer())?), x, 0x6a, 1);

                        if rows != 0 {
                            let x = ops::div_2(get_drawable_width(ctx)?);
                            let line = ctx.unit_info_texts[1];

                            draw_surface_aligned(draw_context(&mut ctx.draw)?, Surface::Label(&line.ok_or(Fault::null_pointer())?), x, 0x107, 1);

                            if rows != 1 {
                                let x = ops::div_2(get_drawable_width(ctx)?);
                                let line = ctx.unit_info_texts[2];

                                draw_surface_aligned(draw_context(&mut ctx.draw)?, Surface::Label(&line.ok_or(Fault::null_pointer())?), x, 0x12e, 1);

                                if rows != 2 {
                                    let x = ops::div_2(get_drawable_width(ctx)?);
                                    let line = ctx.unit_info_texts[3];

                                    draw_surface_aligned(draw_context(&mut ctx.draw)?, Surface::Label(&line.ok_or(Fault::null_pointer())?), x, 0x155, 1);
                                }
                            }
                        }

                        let slot = ctx.i32_at(AppContext::DECK_HOLD_SLOT)?;
                        let form = get_button_unit_form(ctx, 0, slot)?;
                        let traits = get_trait_icon_set(ctx, 0, unit, form, 0)?;

                        ctx.trait_icons = traits;

                        let abilities = get_ability_icon_set(ctx, 0, unit, form, 0)?;

                        ctx.ability_icons = abilities;

                        let centre = ops::cvttsd2si(get_drawable_width(ctx)? as f64 * 0.5);

                        draw_unit_info_panel(ctx, unit, form, centre, 0x9f)?;
                    }
                }
            }
        }
    }

    if ctx.u8_at(AppContext::CAT_GOD_MENU_IS_OPEN)? == 0 && ctx.i32_at(AppContext::OUTRO_PHASE)? == 7 && flag == 0 {
        let centre = ops::div_2(get_drawable_width(ctx)?);
        let press = ctx.i32_at(AppContext::OUTRO_OK_PRESS)?;
        let bounce = *BUTTON_PRESS_BOUNCE.get(press as i64 as usize).ok_or(Fault::index_out_of_range(press as i64, 6))?;
        let x = centre.wrapping_sub(ops::div_2(bounce)).wrapping_add(-0xbe);
        let slide = ctx.i32_at(AppContext::OUTRO_OK_SLIDE)?;
        let top = ctx.i32_at(AppContext::LETTERBOX_SHIFT)?.wrapping_sub(ops::div_2(bounce));
        let y = top.wrapping_sub(get_bottom_inset_logical(ctx)?.wrapping_add(slide)).wrapping_add(0x280);
        let press = ctx.i32_at(AppContext::OUTRO_OK_PRESS)?;
        let bounce = *BUTTON_PRESS_BOUNCE.get(press as i64 as usize).ok_or(Fault::index_out_of_range(press as i64, 6))?;

        draw_cut_scaled(
            draw_context(&mut ctx.draw)?,
            ctx.img101_sheet.as_deref().ok_or(Fault::null_pointer())?,
            x,
            y,
            bounce.wrapping_add(0x17d),
            bounce.wrapping_add(0x48),
            3,
        );

        let centre = ops::div_2(get_drawable_width(ctx)?);
        let press = ctx.i32_at(AppContext::OUTRO_OK_PRESS)?;
        let bounce = *BUTTON_PRESS_BOUNCE.get(press as i64 as usize).ok_or(Fault::index_out_of_range(press as i64, 6))?;
        let x = centre.wrapping_sub(ops::div_2(bounce)).wrapping_add(-0x7f);
        let slide = ctx.i32_at(AppContext::OUTRO_OK_SLIDE)?;
        let top = ctx.i32_at(AppContext::LETTERBOX_SHIFT)?.wrapping_sub(ops::div_2(bounce));
        let y = top.wrapping_sub(get_bottom_inset_logical(ctx)?.wrapping_add(slide)).wrapping_add(0x289);
        let press = ctx.i32_at(AppContext::OUTRO_OK_PRESS)?;
        let bounce = *BUTTON_PRESS_BOUNCE.get(press as i64 as usize).ok_or(Fault::index_out_of_range(press as i64, 6))?;

        draw_cut_scaled(
            draw_context(&mut ctx.draw)?,
            ctx.img006_sheet.as_deref().ok_or(Fault::null_pointer())?,
            x,
            y,
            bounce.wrapping_add(0xfe),
            bounce.wrapping_add(0x37),
            1,
        );
    }

    let status = get_battle_status(ctx)?;

    if ctx.i32_at(AppContext::DECK_ROW_SWAPPING)? == 0
        && ctx.u8_at(AppContext::CURTAIN_ACTIVE)? == 0
        && status == 1
        && ctx.i32_at(AppContext::OUTRO_PHASE)? == 7
        && ctx.i32_at(AppContext::SWIPE_VELOCITY)? == 0
        && touch_is_down(ctx)? != 0
    {
        let left = ctx.i32_at(AppContext::OUTRO_OK_RECT)?;
        let top = ctx.i32_at(AppContext::OUTRO_OK_RECT + 4)?;
        let right = ctx.i32_at(AppContext::OUTRO_OK_RECT + 8)?;
        let bottom = ctx.i32_at(AppContext::OUTRO_OK_RECT + 0xc)?;

        if hit_test_rect(ctx, left, top, right, bottom)? && ctx.u8_at(AppContext::OUTRO_BUTTON_LOCK)? == 0 {
            let centre = ops::div_2(get_drawable_width(ctx)?).wrapping_add(-0xbe);
            let slide = ctx.i32_at(AppContext::OUTRO_OK_SLIDE)?;
            let shift = ctx.i32_at(AppContext::LETTERBOX_SHIFT)?;
            let y = shift.wrapping_sub(get_bottom_inset_logical(ctx)?.wrapping_add(slide)).wrapping_add(0x280);
            let ticks = ctx.i32_at(AppContext::BATTLE_TICKS)?;
            let phase = ticks.wrapping_sub(if ticks >= 0 { ticks } else { ticks.wrapping_add(3) } & 0xfc);
            let cut = ((phase as i8 / 2).wrapping_add(4)) as u8 as i32;

            draw_cut_scaled(
                draw_context(&mut ctx.draw)?,
                ctx.img101_sheet.as_deref().ok_or(Fault::null_pointer())?,
                centre,
                y,
                0x17d,
                0x48,
                cut,
            );
        }
    }

    let notice = ctx.i32_at(AppContext::DEPLOY_NOTICE_KIND)?;
    let notice = if notice == 1 && get_battle_status(ctx)? != 0 {
        -1
    } else {
        ctx.i32_at(AppContext::DEPLOY_NOTICE_KIND)?
    };

    if notice == 1 || notice & -2 == 2 {
        let banner = if notice == 1 {
            Some(0i32)
        } else if ctx.i32_at(AppContext::DEPLOY_NOTICE_TIMER)? != 0x1d && get_battle_status(ctx)? == 0 {
            Some(ctx.i32_at(AppContext::DEPLOY_NOTICE_KIND)?.wrapping_add(-1))
        } else {
            None
        };

        if let Some(banner) = banner {
            for row in 0xdai32..0xdf {
                for step in [-2i32, -1, 0, 1, 2] {
                    set_tint(draw_context(&mut ctx.draw)?, 0, 0, 0, 0xff);

                    let x = ops::div_2(get_drawable_width(ctx)?).wrapping_add(step);
                    let label = ctx
                        .restriction_warning_texts
                        .get(banner as i64 as usize)
                        .copied()
                        .flatten()
                        .ok_or(Fault::null_pointer())?;

                    draw_surface_aligned(draw_context(&mut ctx.draw)?, Surface::Label(&label), x, row, 1);
                }
            }

            set_tint(draw_context(&mut ctx.draw)?, 0xff, 0xff, 0, 0xff);

            let x = ops::div_2(get_drawable_width(ctx)?);
            let label = ctx
                .restriction_warning_texts
                .get(banner as i64 as usize)
                .copied()
                .flatten()
                .ok_or(Fault::null_pointer())?;

            draw_surface_aligned(draw_context(&mut ctx.draw)?, Surface::Label(&label), x, 0xdc, 1);
        }
    }

    combo_banner_draw(ctx)?;
    draw_demon_battle_banner(ctx)?;

    if get_battle_status(ctx)? == 1 {
        draw_outro_xp(ctx, flag)?;
    } else if get_battle_status(ctx)? == 2 {
        draw_outro_lose(ctx)?;
    } else if get_battle_status(ctx)? == 4 {
        draw_outro_dojo(ctx, flag)?;
    } else if get_battle_status(ctx)? == 7 {
        draw_outro_score(ctx)?;
    }

    if ctx.u8_at(AppContext::OPTION_MENU_IS_OPEN)? != 0 {
        option_window_draw(ctx)?;
    } else if ctx.u8_at(AppContext::UNIT_INFO_OVERLAY_OPEN)? != 0 {
        set_tint(draw_context(&mut ctx.draw)?, 0, 0, 0, 0xb2);

        let top = 0i32.wrapping_sub(ctx.i32_at(AppContext::LETTERBOX_SHIFT)?);
        let width = get_drawable_width(ctx)?;
        let height = get_design_height2(ctx);

        fill_rect(draw_context(&mut ctx.draw)?, 0, top, width, height);
        set_tint(draw_context(&mut ctx.draw)?, 0xff, 0xff, 0xff, 0xff);
        draw_deck(ctx, 1)?;

        let slot = ctx.i32_at(AppContext::UNIT_INFO_SLOT)?;
        let unit = get_button_unit_id(ctx, 0, slot)?;
        let slot = ctx.i32_at(AppContext::UNIT_INFO_SLOT)?;
        let form = get_button_unit_form(ctx, 0, slot)?;
        let conjure = stat_conjure_unit_id(ctx, 0, unit, form)?;
        let mut rows = 0i32;
        let height;

        if conjure < 0 {
            height = 0xb4;
        } else {
            rows = if ctx.unit_info_texts[1].is_none() {
                0
            } else if ctx.unit_info_texts[2].is_none() {
                1
            } else if ctx.unit_info_texts[3].is_none() {
                2
            } else {
                3
            };
            height = rows.wrapping_mul(0x27).wrapping_add(0xb4);
        }

        let x = ops::div_2(get_drawable_width(ctx)?).wrapping_add(-0x1d9);
        let sheet = ctx.img002_sheet.clone();
        let panel = sheet.as_deref().ok_or(Fault::null_pointer())?;

        draw_panel(draw_context(&mut ctx.draw)?, panel, x, 0x5d, 0x3b2, height, 1.0, 0x30, 0x31);
        set_tint(draw_context(&mut ctx.draw)?, 0xff, 0xff, 0xff, 0xff);

        let x = ops::div_2(get_drawable_width(ctx)?);
        let line = ctx.unit_info_texts[0];

        draw_surface_aligned(draw_context(&mut ctx.draw)?, Surface::Label(&line.ok_or(Fault::null_pointer())?), x, 0x6a, 1);

        if rows != 0 {
            let x = ops::div_2(get_drawable_width(ctx)?);
            let line = ctx.unit_info_texts[1];

            draw_surface_aligned(draw_context(&mut ctx.draw)?, Surface::Label(&line.ok_or(Fault::null_pointer())?), x, 0x107, 1);

            if rows != 1 {
                let x = ops::div_2(get_drawable_width(ctx)?);
                let line = ctx.unit_info_texts[2];

                draw_surface_aligned(draw_context(&mut ctx.draw)?, Surface::Label(&line.ok_or(Fault::null_pointer())?), x, 0x12e, 1);

                if rows != 2 {
                    let x = ops::div_2(get_drawable_width(ctx)?);
                    let line = ctx.unit_info_texts[3];

                    draw_surface_aligned(draw_context(&mut ctx.draw)?, Surface::Label(&line.ok_or(Fault::null_pointer())?), x, 0x155, 1);
                }
            }
        }

        let traits = get_trait_icon_set(ctx, 0, unit, form, 0)?;

        ctx.trait_icons = traits;

        let abilities = get_ability_icon_set(ctx, 0, unit, form, 0)?;

        ctx.ability_icons = abilities;

        let centre = ops::cvttsd2si(get_drawable_width(ctx)? as f64 * 0.5);

        draw_unit_info_panel(ctx, unit, form, centre, 0x9f)?;

        let button = button_bank_find(&ctx.buttons, 0x3ef).ok_or(Fault::null_pointer())?;

        new_button_draw(ctx, button, 0, 0)?;
    }

    if ctx.u8_at(AppContext::CAT_GOD_MENU_IS_OPEN)? != 0 {
        draw_cat_god_menu(ctx)?;
    }

    glow_set(draw_context(&mut ctx.draw)?, 0);
    set_alpha(draw_context(&mut ctx.draw)?, 0xff);

    let style = ctx.i32_at(AppContext::CURTAIN_STYLE)?;

    draw_screen_transition(ctx, style)?;

    Ok(())
}
