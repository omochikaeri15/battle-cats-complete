use crate::{Fault, ops};

use super::{Maanim, Mamodel, deploy_part, std_vector_int_assign};

pub fn maanim_animate(
    model: &mut Mamodel,
    anim: Option<&Maanim>,
    frame: i32,
    step: i32,
    steps: i32,
    flags: u8,
) -> Result<(), Fault> {
    if flags & 1 == 0 {
        let scale_unit = model.scale_unit;
        let opacity_unit = model.opacity_unit;

        for part in model.parts.iter_mut() {
            part.set_i32_at(0x20, 0);
            part.set_i32_at(0x28, 0);
            part.set_i32_at(0x30, 0);
            part.set_i32_at(0x38, 0);
            part.set_i32_at(0x44, 0);
            part.set_i32_at(0x48, 0);
            part.set_i32_at(0x54, 0);
            part.set_i32_at(0x58, 0);
            part.set_i32_at(0x70, scale_unit);
            part.set_i32_at(0x64, scale_unit);
            part.set_i32_at(0x78, 0);
            part.set_i32_at(0x84, opacity_unit);
            part.set_u8_at(0x8a, 0);
            part.set_u8_at(0x88, 0);
        }
    }

    if let Some(anim) = anim {
        let part_count = model.parts.len() as u64;
        let mut track_index = 0i64;

        while track_index < anim.track_count as i64 {
            let track = anim
                .tracks
                .get(track_index as usize)
                .ok_or(Fault::index_out_of_range(track_index, anim.tracks.len() as i64))?;

            track_index += 1;

            let keyframe_count = track.keyframe_count as i64;

            if keyframe_count == 0 {
                continue;
            }

            let keyframe = |index: i64| {
                track
                    .keyframes
                    .get(index as usize)
                    .ok_or(Fault::index_out_of_range(index, track.keyframes.len() as i64))
            };

            let first = keyframe(0)?[0];

            if frame < first {
                continue;
            }

            let last_index = keyframe_count - 1;
            let last = keyframe(last_index)?[0];
            let mut time = frame;

            if last <= frame && last != first {
                let span = last.wrapping_sub(first);
                let repeats = track.header[2];

                if repeats == -1 {
                    time = ops::irem(frame.wrapping_sub(first), span)
                        .ok_or(Fault::divide(span as i64))?
                        .wrapping_add(first);
                } else {
                    time = last;

                    if repeats > 0 {
                        let elapsed = frame.wrapping_sub(first);
                        let lap = ops::idiv(elapsed, span)
                            .ok_or(Fault::divide(span as i64))?;

                        if lap < repeats {
                            time = ops::irem(elapsed, span)
                                .ok_or(Fault::divide(span as i64))?
                                .wrapping_add(first);
                        }
                    }
                }
            }

            let value;

            if last == first {
                value = keyframe(0)?[1];
            } else if time == last {
                value = keyframe(last_index)?[1];
            } else {
                let segments = if (last_index as i32) > 0 {
                    last_index as i32 as u32 as i64
                } else {
                    0
                };
                let mut index = 0i64;
                let mut found = 0i32;

                loop {
                    if segments == index {
                        break;
                    }

                    let from_frame = keyframe(index)?[0];

                    index += 1;

                    let to_frame = keyframe(index)?[0];
                    let elapsed = time.wrapping_sub(from_frame);

                    if time < from_frame || time >= to_frame {
                        continue;
                    }

                    let from = keyframe(index - 1)?;

                    match from[2] as u32 {
                        0 => {
                            let change = keyframe(index)?[1].wrapping_sub(from[1]);
                            let top = elapsed
                                .wrapping_mul(steps)
                                .wrapping_add(step)
                                .wrapping_mul(change);
                            let bottom = to_frame.wrapping_sub(from_frame).wrapping_mul(steps);

                            found = ops::idiv(top, bottom)
                                .ok_or(Fault::divide(bottom as i64))?
                                .wrapping_add(from[1]);
                        }
                        1 => {
                            found = from[1];
                        }
                        2 => {
                            let power = from[3];
                            let change = keyframe(index)?[1].wrapping_sub(from[1]);
                            let top = elapsed.wrapping_mul(steps).wrapping_add(step);
                            let bottom = to_frame.wrapping_sub(from_frame).wrapping_mul(steps);
                            let start = from[1] as f64;
                            let change = change as f64;
                            let progress = (top as f64) / (bottom as f64);

                            if power < 0 {
                                let eased = (1.0
                                    - (1.0 - progress).powf(power.wrapping_neg() as f64))
                                .sqrt();

                                found = ops::cvttsd2si(change * eased + start);
                            } else {
                                let eased = 1.0 - (1.0 - progress.powf(power as f64)).sqrt();

                                found = ops::cvttsd2si(change * eased + start);
                            }
                        }
                        3 => {
                            let mut run_start = index - 1;

                            loop {
                                if run_start <= 0 {
                                    run_start = 0;
                                    break;
                                }

                                run_start -= 1;

                                if keyframe(run_start)?[2] != 3 {
                                    run_start = (run_start as i32).wrapping_add(1) as u32 as i64;
                                    break;
                                }
                            }

                            let run_start = run_start as i32;
                            let mut run_end = index - 1;

                            if (keyframe_count as i32) > (index as i32) {
                                let mut probe = index;

                                loop {
                                    if (last_index as u32 as i64) == probe
                                        || keyframe(probe)?[2] != 3
                                    {
                                        run_end = probe;
                                        break;
                                    }

                                    probe += 1;

                                    if keyframe_count as i32 == probe as i32 {
                                        break;
                                    }
                                }
                            }

                            let mut total = 0i64;

                            if run_start <= run_end as i32 {
                                let run_stop = (run_end as i32).wrapping_add(1);
                                let terms = run_stop.wrapping_sub(run_start);
                                let mut node = run_start as i64;
                                let mut node_term = 0i64;

                                loop {
                                    let mut weight = (keyframe(node)?[1] as i64) << 0xc;
                                    let node_frame = keyframe(node)?[0] as i64;
                                    let mut other = 0i64;

                                    loop {
                                        if node_term != other {
                                            let other_frame =
                                                keyframe(run_start as i64 + other)?[0] as i64;

                                            weight = weight.wrapping_mul(
                                                (time as i64).wrapping_sub(other_frame),
                                            );

                                            let gap = node_frame.wrapping_sub(other_frame);

                                            weight = ops::div_wide(weight, gap)
                                                .ok_or(Fault::divide(gap))?;
                                        }

                                        other += 1;

                                        if terms == other as i32 {
                                            break;
                                        }
                                    }

                                    total = total.wrapping_add(weight);
                                    node += 1;
                                    node_term += 1;

                                    if run_stop == node as i32 {
                                        break;
                                    }
                                }
                            }

                            let rounded = if total < 0 {
                                total.wrapping_add(0xfff)
                            } else {
                                total
                            };

                            found = (rounded as u64 >> 0xc) as i32;
                        }
                        _ => {}
                    }

                    break;
                }

                value = found;
            }

            let target = track.header[0] as i64;

            if part_count <= target as u64 {
                continue;
            }

            let part = model
                .parts
                .get_mut(target as usize)
                .ok_or(Fault::index_out_of_range(target, part_count as i64))?;

            match track.header[1] as u32 {
                0x0 => part.set_i32_at(0x20, value.wrapping_sub(part.i32_at(0x1c))),
                0x1 => part.set_i32_at(0x28, value.wrapping_sub(part.i32_at(0x24))),
                0x2 => part.set_i32_at(0x30, value.wrapping_sub(part.i32_at(0x2c))),
                0x3 => part.set_i32_at(0x38, value.wrapping_sub(part.i32_at(0x34))),
                0x4 => part.set_i32_at(0x44, value),
                0x5 => part.set_i32_at(0x48, value),
                0x6 => part.set_i32_at(0x54, value),
                0x7 => part.set_i32_at(0x58, value),
                0x8 => {
                    part.set_i32_at(0x70, value);
                    part.set_i32_at(0x64, value);
                }
                0x9 => part.set_i32_at(0x64, value),
                0xa => part.set_i32_at(0x70, value),
                0xb => part.set_i32_at(0x78, value),
                0xc => part.set_i32_at(0x84, value),
                0xd => part.set_u8_at(0x88, (value != 0) as u8),
                0xe => part.set_u8_at(0x8a, (value != 0) as u8),
                _ => {}
            }
        }
    }

    if flags & 2 != 0 {
        return Ok(());
    }

    let mut deployed: Vec<i32> = Vec::new();
    let mut level: Vec<i32> = Vec::new();
    let mut next: Vec<i32> = vec![-1];

    loop {
        std_vector_int_assign(&mut level, &next);
        next.clear();

        let mut index = 0u64;

        while index < model.parts.len() as u64 {
            let part = &model.parts[index as usize];
            let parent = part.i32_at(0x20).wrapping_add(part.i32_at(0x1c));

            if level.contains(&parent) {
                next.push(index as i32);
                deployed.push(index as i32);
            }

            index += 1;
        }

        if next.is_empty() {
            break;
        }
    }

    if !model.parts.is_empty() {
        let mut turn = 0u64;

        loop {
            let part_index = *deployed.get(turn as usize).ok_or(Fault::index_out_of_range(turn as i64, deployed.len() as i64))?;

            deploy_part(part_index, model)?;
            turn += 1;

            if model.parts.len() as u64 <= turn {
                break;
            }
        }
    }

    let part_count = model.parts.len();

    for index in 0..part_count {
        let part = &model.parts[index];
        let depth = part.i32_at(0x38).wrapping_add(part.i32_at(0x34));

        *model
            .draw_order
            .get_mut(index)
            .ok_or(Fault::index_out_of_range(index as i64, 0))? = index as i32;
        *model.draw_z.get_mut(index).ok_or(Fault::index_out_of_range(index as i64, 0))? = depth;
    }

    if part_count > 1 {
        for index in 1..part_count {
            let moving = model.draw_order[index];
            let depth = model.draw_z[index];
            let mut at = index as i64;

            loop {
                let before = (at - 1) as u32 as usize;

                if model.draw_z[before] <= depth {
                    break;
                }

                model.draw_order[at as usize] = model.draw_order[before];
                model.draw_z[at as usize] = model.draw_z[before];

                let was = at;

                at -= 1;

                if was <= 1 {
                    at = 0;
                    break;
                }
            }

            model.draw_order[at as i32 as usize] = moving;
            model.draw_z[at as i32 as usize] = depth;
        }
    }

    Ok(())
}
