use crate::{Fault, ops};

use super::{AppContext, call_rng, get_background_id, get_drawable_width};

pub fn background_particles_init(ctx: &mut AppContext) -> Result<(), Fault> {
    if get_background_id(ctx)? == 2
        || get_background_id(ctx)? == 0xe
        || get_background_id(ctx)? == 0x1a
        || get_background_id(ctx)? == 0x1b
        || get_background_id(ctx)? == 0x22
        || get_background_id(ctx)? == 0x432
    {
        let mut particle = 0usize;

        while particle != 100 {
            let record = AppContext::BG_PARTICLES.wrapping_add(particle * 0x14);
            let roll = call_rng(ctx, 0xc0).wrapping_add((particle * 0xc0) as i32);
            let span = ops::div_10(ctx.i32_at(AppContext::STAGE_LENGTH)?)
                .wrapping_add(get_drawable_width(ctx)?.wrapping_mul(4))
                .wrapping_add(-0xf00);

            ctx.set_i32_at(
                record,
                ops::irem(roll, span).ok_or(Fault::divide(span as i64))?,
            )?;

            if get_background_id(ctx)? == 2
                || get_background_id(ctx)? == 0x1a
                || get_background_id(ctx)? == 0x1b
                || get_background_id(ctx)? == 0x432
            {
                let depth = 100i32
                    .wrapping_sub(ctx.i32_at(AppContext::CAMERA_MIN_ZOOM)?)
                    .wrapping_mul(20);
                let roll = call_rng(ctx, depth.wrapping_add(0xc8));
                let depth = 100i32
                    .wrapping_sub(ctx.i32_at(AppContext::CAMERA_MIN_ZOOM)?)
                    .wrapping_mul(20);

                ctx.set_i32_at(record + 4, roll.wrapping_sub(depth))?;
            } else if get_background_id(ctx)? == 0xe {
                let depth = 100i32
                    .wrapping_sub(ctx.i32_at(AppContext::CAMERA_MIN_ZOOM)?)
                    .wrapping_mul(20);
                let roll = call_rng(ctx, depth.wrapping_add(0xc8));
                let depth = 100i32
                    .wrapping_sub(ctx.i32_at(AppContext::CAMERA_MIN_ZOOM)?)
                    .wrapping_mul(20);

                ctx.set_i32_at(record + 4, roll.wrapping_sub(depth).wrapping_add(-0x3c))?;
            } else if get_background_id(ctx)? == 0x22 {
                let depth = 100i32
                    .wrapping_sub(ctx.i32_at(AppContext::CAMERA_MIN_ZOOM)?)
                    .wrapping_mul(20);
                let roll = call_rng(ctx, depth.wrapping_add(0xc8));
                let depth = 100i32
                    .wrapping_sub(ctx.i32_at(AppContext::CAMERA_MIN_ZOOM)?)
                    .wrapping_mul(20);

                ctx.set_i32_at(record + 4, roll.wrapping_sub(depth).wrapping_add(-0xc8))?;
            }

            ctx.set_i32_at(record + 8, 0)?;

            let speed = call_rng(ctx, 0x14).wrapping_add(5);

            ctx.set_i32_at(record + 0xc, speed)?;

            let kind = call_rng(ctx, 7);

            ctx.set_i32_at(record + 0x10, kind)?;
            particle += 1;
        }
    }

    if get_background_id(ctx)? == 3
        || get_background_id(ctx)? == 0x1b
        || get_background_id(ctx)? == 0xc5
    {
        let mut drifter = 0usize;

        while drifter != 100 {
            let record = AppContext::BG_DRIFTERS.wrapping_add(drifter * 0x10);
            let roll = call_rng(ctx, 0x38).wrapping_add((drifter * 0x38) as i32);
            let length = ops::div_10(ctx.i32_at(AppContext::STAGE_LENGTH)?);
            let x = ops::irem(roll, length).ok_or(Fault::divide(length as i64))?;
            let x = x
                .wrapping_add(get_drawable_width(ctx)?.wrapping_mul(4))
                .wrapping_mul(100)
                .wrapping_add(-0x5dc00);

            ctx.set_i32_at(record, x)?;

            let depth = 100i32
                .wrapping_sub(ctx.i32_at(AppContext::CAMERA_MIN_ZOOM)?)
                .wrapping_mul(20);
            let roll = call_rng(ctx, depth.wrapping_add(0x280));
            let depth = 100i32
                .wrapping_sub(ctx.i32_at(AppContext::CAMERA_MIN_ZOOM)?)
                .wrapping_mul(20);

            ctx.set_i32_at(record + 4, roll.wrapping_sub(depth).wrapping_mul(100))?;

            let angle = 0xafi32.wrapping_sub(call_rng(ctx, 0x55));

            ctx.set_i32_at(record + 8, angle)?;

            let speed = call_rng(ctx, 0x258).wrapping_add(0xc8);

            ctx.set_i32_at(record + 0xc, speed)?;
            drifter += 1;
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
        let mut particle = 0usize;

        while particle != 100 {
            let record = AppContext::BG_PARTICLES.wrapping_add(particle * 0x14);
            let rising = call_rng(ctx, 5);
            let roll = call_rng(ctx, 0xc0).wrapping_add((particle * 0xc0) as i32);
            let span = ops::div_10(ctx.i32_at(AppContext::STAGE_LENGTH)?)
                .wrapping_add(get_drawable_width(ctx)?.wrapping_mul(4))
                .wrapping_add(-0xf00);

            ctx.set_i32_at(
                record,
                ops::irem(roll, span).ok_or(Fault::divide(span as i64))?,
            )?;

            let y = if rising != 0 {
                100i32
                    .wrapping_sub(ctx.i32_at(AppContext::CAMERA_MIN_ZOOM)?)
                    .wrapping_mul(20)
                    .wrapping_neg()
            } else {
                let reach = 0x280i32.wrapping_sub(ctx.i32_at(AppContext::BATTLE_ZOOM_Y)?);
                let spread = 100i32
                    .wrapping_sub(ctx.i32_at(AppContext::CAMERA_MIN_ZOOM)?)
                    .wrapping_mul(reach);

                if spread > 0x1f3ff {
                    0
                } else {
                    call_rng(ctx, ops::div_neg_100(spread).wrapping_add(0x500))
                }
            };

            ctx.set_i32_at(record + 4, y)?;

            let timer = call_rng(ctx, 0x168);

            ctx.set_i32_at(record + 8, timer)?;

            if get_background_id(ctx)? == 0x51
                || get_background_id(ctx)? == 0x65
                || get_background_id(ctx)? == 0x7b
                || get_background_id(ctx)? == 0x92
                || get_background_id(ctx)? == 0xa9
            {
                let speed = call_rng(ctx, 0x28).wrapping_add(100);

                ctx.set_i32_at(record + 0xc, speed)?;

                let kind = call_rng(ctx, 2);

                ctx.set_i32_at(record + 0x10, kind)?;
            }

            particle += 1;
        }

        return Ok(());
    }

    if get_background_id(ctx)? == 0x28 {
        let mut pair = 0usize;

        while pair != 100 {
            let sprite = AppContext::BG_SPRITES.wrapping_add(pair * 0x40);

            for half in [0usize, 0x20] {
                let width = ops::div_10(ctx.i32_at(AppContext::STAGE_LENGTH)?)
                    .wrapping_add(get_drawable_width(ctx)?.wrapping_mul(4))
                    .wrapping_add(-0xf00);
                let x = call_rng(ctx, width).wrapping_mul(100);

                ctx.set_i32_at(sprite + half, x)?;

                let depth = 100i32
                    .wrapping_sub(ctx.i32_at(AppContext::CAMERA_MIN_ZOOM)?)
                    .wrapping_mul(20);
                let roll = call_rng(ctx, depth.wrapping_add(0x280));
                let depth = 100i32
                    .wrapping_sub(ctx.i32_at(AppContext::CAMERA_MIN_ZOOM)?)
                    .wrapping_mul(20);

                ctx.set_i32_at(
                    sprite + half + 4,
                    roll.wrapping_sub(depth).wrapping_mul(100),
                )?;

                let angle = call_rng(ctx, 0x168);

                ctx.set_i32_at(sprite + half + 8, angle)?;

                let scale = call_rng(ctx, 100).wrapping_add(0x32);

                ctx.set_i32_at(sprite + half + 0xc, scale)?;

                let spin = call_rng(ctx, 0x96);

                ctx.set_i32_at(sprite + half + 0x10, spin)?;

                let phase = call_rng(ctx, 100);

                ctx.set_i32_at(sprite + half + 0x14, phase)?;
            }

            pair += 1;
        }

        return Ok(());
    }

    if get_background_id(ctx)? == 0x29
        || get_background_id(ctx)? == 0x4b
        || get_background_id(ctx)? == 0x3f0
    {
        let mut pair = 0usize;

        while pair != 100 {
            let sprite = AppContext::BG_SPRITES.wrapping_add(pair * 0x40);
            let mut first = true;
            let mut half = 0usize;

            loop {
                let zoom = ctx.i32_at(AppContext::CAMERA_MIN_ZOOM)?;
                let (bound, base) = if first {
                    let reach =
                        (-0x122i32).wrapping_sub(100i32.wrapping_sub(zoom).wrapping_mul(20));
                    let y = ops::div_100(call_rng(ctx, 100).wrapping_mul(reach))
                        .wrapping_mul(100)
                        .wrapping_add(0x7148);

                    ctx.set_i32_at(sprite + 4, y)?;

                    let pick = call_rng(ctx, 10);

                    ctx.set_i32_at(
                        sprite + 0xc,
                        0x14i32.wrapping_add(((pick as u32) < 1) as i32),
                    )?;

                    (0x32, 0x32)
                } else {
                    let lift =
                        ops::idiv(0x4e20, zoom).ok_or(Fault::divide(zoom as i64))?;
                    let reach = lift
                        .wrapping_add(100i32.wrapping_sub(zoom).wrapping_mul(20))
                        .wrapping_add(0x1e0)
                        .wrapping_neg();
                    let y = ops::div_100(call_rng(ctx, 100).wrapping_mul(reach))
                        .wrapping_add(lift)
                        .wrapping_add(0x1e0)
                        .wrapping_mul(100);

                    ctx.set_i32_at(sprite + half * 0x20 + 4, y)?;
                    ctx.set_i32_at(sprite + 0xc + 0x20 * half, 0x14)?;

                    (0x64, 0xc8)
                };

                let size = call_rng(ctx, bound).wrapping_add(base);

                ctx.set_i32_at(sprite + half * 0x20 + 0x10, size)?;

                if get_background_id(ctx)? == 0x4b || get_background_id(ctx)? == 0x3f0 {
                    let field = sprite + half * 0x20 + 0xc;

                    ctx.set_i32_at(field, ctx.i32_at(field)?.wrapping_add(1))?;
                }

                let width = ops::div_10(ctx.i32_at(AppContext::STAGE_LENGTH)?)
                    .wrapping_add(get_drawable_width(ctx)?.wrapping_mul(4))
                    .wrapping_add(-0xf00);
                let x = call_rng(ctx, width).wrapping_mul(100);

                ctx.set_i32_at(sprite + half * 0x20, x)?;
                ctx.set_i32_at(sprite + half * 0x20 + 8, 0x1e)?;

                let life = call_rng(ctx, 0x3c).wrapping_add(0x46);

                ctx.set_i32_at(sprite + half * 0x20 + 0x14, life)?;

                let angle = call_rng(ctx, 0x168);

                ctx.set_i32_at(sprite + half * 0x20 + 0x18, angle)?;
                half = 1;

                if !first {
                    break;
                }

                first = false;
            }

            pair += 1;
        }

        return Ok(());
    }

    if get_background_id(ctx)? == 0x2e || get_background_id(ctx)? == 0x2f {
        let mut pair = 0usize;

        while pair != 100 {
            let sprite = AppContext::BG_SPRITES.wrapping_add(pair * 0x40);

            for half in [0usize, 0x20] {
                let width = ops::div_10(ctx.i32_at(AppContext::STAGE_LENGTH)?)
                    .wrapping_add(get_drawable_width(ctx)?.wrapping_mul(4))
                    .wrapping_add(-0xf00);
                let x = call_rng(ctx, width).wrapping_mul(100);

                ctx.set_i32_at(sprite + half, x)?;

                let depth = 100i32
                    .wrapping_sub(ctx.i32_at(AppContext::CAMERA_MIN_ZOOM)?)
                    .wrapping_mul(20);
                let roll = call_rng(ctx, depth.wrapping_add(0x280));
                let depth = 100i32
                    .wrapping_sub(ctx.i32_at(AppContext::CAMERA_MIN_ZOOM)?)
                    .wrapping_mul(20);

                ctx.set_i32_at(
                    sprite + half + 4,
                    roll.wrapping_sub(depth).wrapping_mul(100),
                )?;

                let angle = call_rng(ctx, 0x168);

                ctx.set_i32_at(sprite + half + 8, angle)?;

                let scale = call_rng(ctx, 100).wrapping_add(0x32);

                ctx.set_i32_at(sprite + half + 0xc, scale)?;

                let spin = call_rng(ctx, 0x96);

                ctx.set_i32_at(sprite + half + 0x10, spin)?;

                let life = call_rng(ctx, 0xbb8).wrapping_add(0x3e8);

                ctx.set_i32_at(sprite + half + 0x14, life)?;
                ctx.set_i32_at(sprite + half + 0x18, 0x7d0)?;
            }

            pair += 1;
        }

        return Ok(());
    }

    if get_background_id(ctx)? == 0x37 {
        let mut particle = 0usize;

        while particle != 100 {
            let spark = !call_rng(ctx, 0x3c);

            ctx.set_i32_at(
                AppContext::BG_PARTICLES.wrapping_add(particle * 0x14),
                spark,
            )?;
            particle += 1;
        }
    }

    Ok(())
}
