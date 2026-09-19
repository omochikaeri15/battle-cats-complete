use crate::{operation, Fault};

use super::{bg_effect_roll_params, bg_param_enabled, call_rng, cos_deg, get_background_id, get_background_width, get_drawable_width, sin_deg, AppContext};

const SITE: &str = "background_particles";

pub fn background_particles(ctx: &mut AppContext) -> Result<(), Fault> {
    if get_background_id(ctx)? == 2
        || get_background_id(ctx)? == 0xe
        || get_background_id(ctx)? == 0x1a
        || get_background_id(ctx)? == 0x1b
        || get_background_id(ctx)? == 0x22
        || get_background_id(ctx)? == 0x432
    {
        for particle in 0..100usize {
            let record = AppContext::BG_PARTICLES.wrapping_add(particle * 0x14);
            let seed = (particle as i32).wrapping_mul(0xc0);
            let timer = ctx.i32_at(record.wrapping_add(8))?.wrapping_add(ctx.i32_at(record.wrapping_add(0xc))?);

            ctx.set_i32_at(record.wrapping_add(8), timer)?;

            if timer < 0x190 {
                continue;
            }

            let spread = call_rng(ctx, 0xc0).wrapping_add(seed);
            let width = operation::div_10(ctx.i32_at(AppContext::STAGE_LENGTH)?).wrapping_add(get_drawable_width(ctx)?.wrapping_mul(4)).wrapping_add(-0xf00);

            ctx.set_i32_at(record, operation::irem(spread, width).ok_or(Fault::divide(SITE, width as i64))?)?;

            if get_background_id(ctx)? == 2 || get_background_id(ctx)? == 0x1a || get_background_id(ctx)? == 0x1b || get_background_id(ctx)? == 0x432 {
                let height = call_rng(ctx, 0x64i32.wrapping_sub(ctx.i32_at(AppContext::CAMERA_MIN_ZOOM)?).wrapping_mul(0x14).wrapping_add(0xc8));
                let lift = 0x64i32.wrapping_sub(ctx.i32_at(AppContext::CAMERA_MIN_ZOOM)?).wrapping_mul(0x14);

                ctx.set_i32_at(record.wrapping_add(4), height.wrapping_sub(lift))?;
            } else if get_background_id(ctx)? == 0xe {
                let height = call_rng(ctx, 0x64i32.wrapping_sub(ctx.i32_at(AppContext::CAMERA_MIN_ZOOM)?).wrapping_mul(0x14).wrapping_add(0xc8));
                let lift = 0x64i32.wrapping_sub(ctx.i32_at(AppContext::CAMERA_MIN_ZOOM)?).wrapping_mul(0x14);

                ctx.set_i32_at(record.wrapping_add(4), height.wrapping_sub(lift).wrapping_add(-0x3c))?;
            } else if get_background_id(ctx)? == 0x22 {
                let height = call_rng(ctx, 0x64i32.wrapping_sub(ctx.i32_at(AppContext::CAMERA_MIN_ZOOM)?).wrapping_mul(0x14).wrapping_add(0xc8));
                let lift = 0x64i32.wrapping_sub(ctx.i32_at(AppContext::CAMERA_MIN_ZOOM)?).wrapping_mul(0x14);

                ctx.set_i32_at(record.wrapping_add(4), height.wrapping_sub(lift).wrapping_add(-0xc8))?;
            }

            ctx.set_i32_at(record.wrapping_add(8), 0)?;

            let speed = call_rng(ctx, 0x14).wrapping_add(5);

            ctx.set_i32_at(record.wrapping_add(0xc), speed)?;

            let kind = call_rng(ctx, 7);

            ctx.set_i32_at(record.wrapping_add(0x10), kind)?;
        }
    }

    if get_background_id(ctx)? == 3 || get_background_id(ctx)? == 0x1b || get_background_id(ctx)? == 0xc5 {
        for flake in 0..100usize {
            let record = AppContext::BG_DRIFTERS.wrapping_add(flake * 0x10);
            let turn = cos_deg(ctx.i32_at(record.wrapping_add(8))? as f32);
            let x = operation::cvttss2si(ctx.i32_at(record.wrapping_add(0xc))? as f32 * turn + ctx.i32_at(record)? as f32);

            ctx.set_i32_at(record, x)?;

            let stage = ctx.i32_at(AppContext::STAGE_LENGTH)?.wrapping_mul(5);
            let wrapped = x.wrapping_add(stage.wrapping_mul(2)).wrapping_add(get_drawable_width(ctx)?.wrapping_mul(0x190)).wrapping_add(-0x5dc00);
            let stage = ctx.i32_at(AppContext::STAGE_LENGTH)?.wrapping_mul(5);
            let span = get_drawable_width(ctx)?.wrapping_mul(0x190).wrapping_add(stage.wrapping_mul(2)).wrapping_add(-0x5dc00);

            ctx.set_i32_at(record, operation::irem(wrapped, span).ok_or(Fault::divide(SITE, span as i64))?)?;

            let lift = sin_deg(ctx.i32_at(record.wrapping_add(8))? as f32);
            let y = operation::cvttss2si(ctx.i32_at(record.wrapping_add(0xc))? as f32 * lift + ctx.i32_at(record.wrapping_add(4))? as f32);

            ctx.set_i32_at(record.wrapping_add(4), y)?;

            let height = 0x280i32.wrapping_sub(ctx.i32_at(AppContext::BATTLE_ZOOM_Y)?);
            let zoom = 0x64i32.wrapping_sub(ctx.i32_at(AppContext::CAMERA_MIN_ZOOM)?);
            let scaled = height.wrapping_mul(zoom);
            let floor = scaled.wrapping_sub(operation::div_100(scaled).wrapping_mul(0x64)).wrapping_sub(scaled).wrapping_add(0x1fbd0);

            if floor > y {
                continue;
            }

            ctx.set_i32_at(record.wrapping_add(4), zoom.wrapping_mul(-0x7d0))?;

            let angle = 0xafi32.wrapping_sub(call_rng(ctx, 0x55));

            ctx.set_i32_at(record.wrapping_add(8), angle)?;

            let speed = call_rng(ctx, 0x258).wrapping_add(0xc8);

            ctx.set_i32_at(record.wrapping_add(0xc), speed)?;
        }
    }

    if get_background_id(ctx)? == 0xd
        || get_background_id(ctx)? == 0xf
        || get_background_id(ctx)? == 0x48
        || get_background_id(ctx)? == 0x51
        || get_background_id(ctx)? == 0x65
        || get_background_id(ctx)? == 0x7b
        || get_background_id(ctx)? == 0x92
        || get_background_id(ctx)? == 0xa9
    {
        for particle in 0..100usize {
            let record = AppContext::BG_PARTICLES.wrapping_add(particle * 0x14);
            let seed = (particle as i32).wrapping_mul(0xc0);

            let y = if get_background_id(ctx)? == 0x51
                || get_background_id(ctx)? == 0x65
                || get_background_id(ctx)? == 0x7b
                || get_background_id(ctx)? == 0x92
                || get_background_id(ctx)? == 0xa9
            {
                ctx.i32_at(record.wrapping_add(4))?.wrapping_sub(1)
            } else {
                ctx.i32_at(record.wrapping_add(4))?.wrapping_add(-3)
            };

            ctx.set_i32_at(record.wrapping_add(4), y)?;
            ctx.set_i32_at(record.wrapping_add(8), ctx.i32_at(record.wrapping_add(8))?.wrapping_add(1))?;

            if y >= 0x64i32.wrapping_sub(ctx.i32_at(AppContext::CAMERA_MIN_ZOOM)?).wrapping_mul(0x14).wrapping_neg() {
                continue;
            }

            if call_rng(ctx, 0x7d0) != 0 {
                continue;
            }

            let spread = call_rng(ctx, 0xc0).wrapping_add(seed);
            let width = operation::div_10(ctx.i32_at(AppContext::STAGE_LENGTH)?).wrapping_add(get_drawable_width(ctx)?.wrapping_mul(4)).wrapping_add(-0xf00);

            ctx.set_i32_at(record, operation::irem(spread, width).ok_or(Fault::divide(SITE, width as i64))?)?;

            let height = 0x280i32.wrapping_sub(ctx.i32_at(AppContext::BATTLE_ZOOM_Y)?);
            let zoom = 0x64i32.wrapping_sub(ctx.i32_at(AppContext::CAMERA_MIN_ZOOM)?);

            ctx.set_i32_at(record.wrapping_add(4), operation::div_neg_100(zoom.wrapping_mul(height)).wrapping_add(0x500))?;

            let tilt = call_rng(ctx, 0x168);

            ctx.set_i32_at(record.wrapping_add(8), tilt)?;

            if get_background_id(ctx)? == 0x51
                || get_background_id(ctx)? == 0x65
                || get_background_id(ctx)? == 0x7b
                || get_background_id(ctx)? == 0x92
                || get_background_id(ctx)? == 0xa9
            {
                let speed = call_rng(ctx, 0x28).wrapping_add(0x64);

                ctx.set_i32_at(record.wrapping_add(0xc), speed)?;

                let kind = call_rng(ctx, 2);

                ctx.set_i32_at(record.wrapping_add(0x10), kind)?;
            }
        }
    }

    if get_background_id(ctx)? == 0x21 || get_background_id(ctx)? == 0x3a {
        for star in 0..30usize {
            let record = AppContext::BG_DRIFTERS.wrapping_add(star * 0x10);
            let width = operation::div_10(get_background_width(ctx)?).wrapping_add(get_drawable_width(ctx)?.wrapping_mul(4)).wrapping_add(-0xf00);
            let x = call_rng(ctx, width);

            ctx.set_i32_at(record, x)?;

            let y = call_rng(ctx, 0x5a).wrapping_add(0x1a4);

            ctx.set_i32_at(record.wrapping_add(4), y)?;
        }

        for star in 0..60usize {
            let record = AppContext::BG_STARS.wrapping_add(star * 0x10);
            let width = operation::div_10(get_background_width(ctx)?).wrapping_add(get_drawable_width(ctx)?.wrapping_mul(4)).wrapping_add(-0xf00);
            let x = call_rng(ctx, width);

            ctx.set_i32_at(record, x)?;

            let height = call_rng(ctx, 0x64i32.wrapping_sub(ctx.i32_at(AppContext::CAMERA_MIN_ZOOM)?).wrapping_mul(0x14).wrapping_add(0x280));
            let lift = 0x64i32.wrapping_sub(ctx.i32_at(AppContext::CAMERA_MIN_ZOOM)?).wrapping_mul(0x14);

            ctx.set_i32_at(record.wrapping_add(4), height.wrapping_sub(lift))?;

            let size = call_rng(ctx, 0x64).wrapping_add(0x64);

            ctx.set_i32_at(record.wrapping_add(8), size)?;
        }
    }

    if get_background_id(ctx)? == 0x28 {
        for record in 0..200usize {
            let sprite = AppContext::BG_SPRITES.wrapping_add(record * 0x20);
            let rise = operation::div_100(ctx.i32_at(sprite.wrapping_add(0x14))?.wrapping_add(0xaa).wrapping_mul(ctx.i32_at(sprite.wrapping_add(0xc))?));
            let y = ctx.i32_at(sprite.wrapping_add(4))?.wrapping_add(rise);

            ctx.set_i32_at(sprite.wrapping_add(4), y)?;
            ctx.set_i32_at(sprite.wrapping_add(8), ctx.i32_at(sprite.wrapping_add(8))?.wrapping_add(1))?;

            let height = 0x280i32.wrapping_sub(ctx.i32_at(AppContext::BATTLE_ZOOM_Y)?);
            let scaled = height.wrapping_mul(0x64i32.wrapping_sub(ctx.i32_at(AppContext::CAMERA_MIN_ZOOM)?));
            let floor = scaled.wrapping_sub(operation::div_100(scaled).wrapping_mul(0x64)).wrapping_sub(scaled).wrapping_add(0x1fbd0);

            if y < floor {
                continue;
            }

            let width = operation::div_10(ctx.i32_at(AppContext::STAGE_LENGTH)?).wrapping_add(get_drawable_width(ctx)?.wrapping_mul(4)).wrapping_add(-0xf00);
            let x = call_rng(ctx, width).wrapping_mul(0x64);

            ctx.set_i32_at(sprite, x)?;
            ctx.set_i32_at(sprite.wrapping_add(4), 0x64i32.wrapping_sub(ctx.i32_at(AppContext::CAMERA_MIN_ZOOM)?).wrapping_mul(-0x7d0))?;

            let angle = call_rng(ctx, 0x168);

            ctx.set_i32_at(sprite.wrapping_add(8), angle)?;

            let speed = call_rng(ctx, 0x96).wrapping_add(0x32);

            ctx.set_i32_at(sprite.wrapping_add(0xc), speed)?;

            let tint = call_rng(ctx, 0x64);

            ctx.set_i32_at(sprite.wrapping_add(0x10), tint)?;

            let sway = call_rng(ctx, 0x64);

            ctx.set_i32_at(sprite.wrapping_add(0x14), sway)?;
        }
    } else if get_background_id(ctx)? == 0x29 || get_background_id(ctx)? == 0x4b || get_background_id(ctx)? == 0x3f0 {
        for pair in 0..100usize {
            for half in 0..2usize {
                let sprite = AppContext::BG_SPRITES.wrapping_add(pair * 0x40).wrapping_add(half * 0x20);
                let fall = operation::div_neg_100(ctx.i32_at(sprite.wrapping_add(0x14))?.wrapping_mul(ctx.i32_at(sprite.wrapping_add(0x10))?));
                let y = ctx.i32_at(sprite.wrapping_add(4))?.wrapping_add(fall);

                ctx.set_i32_at(sprite.wrapping_add(4), y)?;
                ctx.set_i32_at(sprite.wrapping_add(8), ctx.i32_at(sprite.wrapping_add(8))?.wrapping_add(1))?;

                if y >= 0x64i32.wrapping_sub(ctx.i32_at(AppContext::CAMERA_MIN_ZOOM)?).wrapping_mul(-0x7d0) {
                    continue;
                }

                let width = operation::div_10(ctx.i32_at(AppContext::STAGE_LENGTH)?).wrapping_add(get_drawable_width(ctx)?.wrapping_mul(4)).wrapping_add(-0xf00);
                let x = call_rng(ctx, width).wrapping_mul(0x64);

                ctx.set_i32_at(sprite, x)?;

                let (bound, floor) = if half == 0 {
                    let y = 0x154i32.wrapping_sub(call_rng(ctx, 0x32)).wrapping_mul(0x64);

                    ctx.set_i32_at(sprite.wrapping_add(4), y)?;

                    let life = if call_rng(ctx, 0xa) < 1 { 0x15 } else { 0x14 };

                    ctx.set_i32_at(AppContext::BG_SPRITES.wrapping_add(pair * 0x40).wrapping_add(0xc), life)?;

                    (0x32, 0x32)
                } else {
                    let min_zoom = ctx.i32_at(AppContext::CAMERA_MIN_ZOOM)?;
                    let lift = operation::idiv(0x4e20, min_zoom).ok_or(Fault::divide(SITE, min_zoom as i64))?;
                    let y = call_rng(ctx, 0x32).wrapping_add(lift).wrapping_mul(0x64).wrapping_add(0xbb80);

                    ctx.set_i32_at(sprite.wrapping_add(4), y)?;
                    ctx.set_i32_at(sprite.wrapping_add(0xc), 0x14)?;

                    (0x64, 0xc8)
                };

                let speed = call_rng(ctx, bound).wrapping_add(floor);

                ctx.set_i32_at(sprite.wrapping_add(0x10), speed)?;

                if get_background_id(ctx)? == 0x4b || get_background_id(ctx)? == 0x3f0 {
                    ctx.set_i32_at(sprite.wrapping_add(0xc), ctx.i32_at(sprite.wrapping_add(0xc))?.wrapping_add(1))?;
                }

                ctx.set_i32_at(sprite.wrapping_add(8), 0)?;

                let sway = call_rng(ctx, 0x3c).wrapping_add(0x46);

                ctx.set_i32_at(sprite.wrapping_add(0x14), sway)?;

                let angle = call_rng(ctx, 0x168);

                ctx.set_i32_at(sprite.wrapping_add(0x18), angle)?;
            }
        }
    } else if get_background_id(ctx)? == 0x2e || get_background_id(ctx)? == 0x2f {
        for pair in 0..100usize {
            for half in 0..2usize {
                let sprite = AppContext::BG_SPRITES.wrapping_add(pair * 0x40).wrapping_add(half * 0x20);
                let speed = ctx.i32_at(sprite.wrapping_add(0xc))?;
                let x = operation::div_100(ctx.i32_at(sprite.wrapping_add(0x14))?.wrapping_add(0xaa).wrapping_mul(speed)).wrapping_add(ctx.i32_at(sprite)?);

                ctx.set_i32_at(sprite, x)?;

                let rise = operation::div_100(ctx.i32_at(sprite.wrapping_add(0x18))?.wrapping_add(0xaa).wrapping_mul(speed));

                ctx.set_i32_at(sprite.wrapping_add(4), ctx.i32_at(sprite.wrapping_add(4))?.wrapping_add(rise))?;
                ctx.set_i32_at(sprite.wrapping_add(8), ctx.i32_at(sprite.wrapping_add(8))?.wrapping_add(1))?;

                let stage = operation::div_10(ctx.i32_at(AppContext::STAGE_LENGTH)?);
                let edge = stage.wrapping_add(get_drawable_width(ctx)?.wrapping_mul(4)).wrapping_mul(0x64).wrapping_add(-0x5dc00);

                if x >= edge {
                    ctx.set_i32_at(sprite, 0)?;
                }

                let height = 0x280i32.wrapping_sub(ctx.i32_at(AppContext::BATTLE_ZOOM_Y)?);
                let scaled = height.wrapping_mul(0x64i32.wrapping_sub(ctx.i32_at(AppContext::CAMERA_MIN_ZOOM)?));
                let floor = scaled.wrapping_sub(operation::div_100(scaled).wrapping_mul(0x64)).wrapping_sub(scaled).wrapping_add(0x1fbd0);

                if ctx.i32_at(sprite.wrapping_add(4))? < floor {
                    continue;
                }

                let width = operation::div_10(ctx.i32_at(AppContext::STAGE_LENGTH)?).wrapping_add(get_drawable_width(ctx)?.wrapping_mul(4)).wrapping_add(-0xf00);
                let x = call_rng(ctx, width).wrapping_mul(0x64);

                ctx.set_i32_at(sprite, x)?;
                ctx.set_i32_at(sprite.wrapping_add(4), 0x64i32.wrapping_sub(ctx.i32_at(AppContext::CAMERA_MIN_ZOOM)?).wrapping_mul(-0x7d0))?;

                let angle = call_rng(ctx, 0x168);

                ctx.set_i32_at(sprite.wrapping_add(8), angle)?;

                let speed = call_rng(ctx, 0x96).wrapping_add(0x32);

                ctx.set_i32_at(sprite.wrapping_add(0xc), speed)?;

                let tint = call_rng(ctx, 0x64);

                ctx.set_i32_at(sprite.wrapping_add(0x10), tint)?;

                let drift = call_rng(ctx, 0x1770).wrapping_add(0x7d0);

                ctx.set_i32_at(sprite.wrapping_add(0x14), drift)?;
                ctx.set_i32_at(sprite.wrapping_add(0x18), 0xfa0)?;
            }
        }
    } else if get_background_id(ctx)? == 0x37 {
        for particle in 0..100usize {
            let record = AppContext::BG_PARTICLES.wrapping_add(particle * 0x14);
            let old = ctx.i32_at(record)?;

            ctx.set_i32_at(record, old.wrapping_add(1))?;

            if old.wrapping_add(1) == 0 {
                let width = operation::div_10(ctx.i32_at(AppContext::STAGE_LENGTH)?).wrapping_add(get_drawable_width(ctx)?.wrapping_mul(4)).wrapping_add(-0xf00);
                let x = call_rng(ctx, width).wrapping_mul(10);

                ctx.set_i32_at(record.wrapping_add(4), x)?;

                let y = call_rng(ctx, 0x1c2).wrapping_add(0x96);

                ctx.set_i32_at(record.wrapping_add(8), y)?;
            } else if old >= 0xa {
                let wait = !call_rng(ctx, 0x3c);

                ctx.set_i32_at(record, wait)?;
            }
        }
    }

    let mut instance = 0usize;

    while instance < ctx.bg_effects.instances.len() {
        let Some(effect) = ctx.bg_effects.instances.get_mut(instance) else {
            break;
        };

        if effect.wait > 0 {
            effect.wait = effect.wait.wrapping_sub(1);
            instance += 1;

            continue;
        }

        let (v, vx, vy, heading) = (effect.v, effect.vx, effect.vy, effect.move_angle);
        let step_x = v * cos_deg(heading) + vx;
        let step_y = v * sin_deg(heading) + vy;

        let effect = ctx.bg_effects.instances.get_mut(instance).ok_or(Fault::IndexOutOfRange { site: SITE, index: instance as i64, limit: 0 })?;

        effect.x += step_x;
        effect.y += step_y;
        effect.angle += effect.angular_v;

        let old = effect.frame;

        effect.frame = old.wrapping_add(1);

        let effect = effect.clone();
        let def = ctx.bg_effects.defs.get(effect.def_index as i64 as usize).ok_or(Fault::IndexOutOfRange { site: SITE, index: effect.def_index as i64, limit: ctx.bg_effects.defs.len() as i64 })?;

        let expired = (effect.life_time > 0 && old >= effect.life_time)
            || (bg_param_enabled(&def.destroy_left) != 0 && effect.destroy_left > effect.x)
            || (bg_param_enabled(&def.destroy_right) != 0 && effect.x > effect.destroy_right)
            || (bg_param_enabled(&def.destroy_top) != 0 && effect.destroy_top > effect.y)
            || (bg_param_enabled(&def.destroy_bottom) != 0 && effect.y > effect.destroy_bottom);

        if expired {
            bg_effect_roll_params(ctx, instance, 0)?;
        }

        instance += 1;
    }

    Ok(())
}
