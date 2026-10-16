//! The particles the engine scatters over particular backgrounds.
//!
//! Which system runs is decided by the background's identifier alone: sparkling
//! motes, falling snow, rising bubbles or petals, a starry sky, drifting
//! bubbles, tumbling leaves, streaming lights and glowing sparks. Each keeps
//! its state in a fixed table of records, re-rolls a record from the random
//! number generator whenever it leaves the field, and draws either behind the
//! units or in front of them.

use image::RgbaImage;

use crate::chapter::stage::Rgba;
use crate::common::Rng;

use crate::graphics::rig::SpriteSheet;

use super::{cos_deg, sin_deg, Painter, Spin, Surface, World};

const MOTES: [i32; 6] = [2, 0xe, 0x1a, 0x1b, 0x22, 0x432];
const HIGH_MOTES: [i32; 4] = [2, 0x1a, 0x1b, 0x432];
const LOW_MOTES: i32 = 0xe;
const DEEP_MOTES: i32 = 0x22;
const SNOW: [i32; 3] = [3, 0x1b, 0xc5];
const RISING: [i32; 8] = [0xd, 0xf, 0x48, 0x51, 0x65, 0x7b, 0x92, 0xa9];
const PETALS: [i32; 5] = [0x51, 0x65, 0x7b, 0x92, 0xa9];
const BUBBLES: [i32; 3] = [0xd, 0xf, 0x48];
const NIGHT: [i32; 2] = [0x21, 0x3a];
const BUBBLE_STORM: i32 = 0x28;
const LEAVES: [i32; 3] = [0x29, 0x4b, 0x3f0];
const FAST_LEAVES: [i32; 2] = [0x4b, 0x3f0];
const STREAMS: [i32; 2] = [0x2e, 0x2f];
const SPARKS: i32 = 0x37;

const MOTE_COUNT: usize = 100;
const DRIFTER_COUNT: usize = 100;
const STAR_COUNT: usize = 60;
const SPRITE_COUNT: usize = 200;
const GLINT_COUNT: usize = 30;
const PETAL_COUNT: usize = 50;
const LEAF_FRONT_COUNT: usize = 30;

const MOTE_X: usize = 0;
const MOTE_Y: usize = 1;
const MOTE_PHASE: usize = 2;
const MOTE_SPEED: usize = 3;
const MOTE_KIND: usize = 4;

const DRIFTER_X: usize = 0;
const DRIFTER_Y: usize = 1;
const DRIFTER_ANGLE: usize = 2;
const DRIFTER_RADIUS: usize = 3;

const STAR_X: usize = 0;
const STAR_Y: usize = 1;
const STAR_GLOW: usize = 2;

const SPRITE_X: usize = 0;
const SPRITE_Y: usize = 1;
const SPRITE_ANGLE: usize = 2;
const SPRITE_SIZE: usize = 3;
const SPRITE_RATE: usize = 4;
const SPRITE_LIFE: usize = 5;
const SPRITE_TURN: usize = 6;

const PARTICLE_RED: [i32; 6] = [255, 180, 180, 255, 255, 180];
const PARTICLE_GREEN: [i32; 6] = [180, 255, 180, 255, 180, 255];
const PARTICLE_BLUE: [i32; 6] = [180, 180, 255, 180, 255, 255];

const GLINT_CUT: i32 = 0x1c;
const STAR_CUT: i32 = 0x1d;
const SPARK_CUT: i32 = 0x14;
const PETAL_CUT: i32 = 0x14;
const BUBBLE_SIZE: i32 = 0x13;
const DESIGN_WIDTH: i32 = 0x3c0;
const DESIGN_HEIGHT: i32 = 0x280;

const BUBBLE_IMAGE: &str = "bubble.png";
const SECOND_BUBBLE_IMAGE: &str = "bubble02.png";
const STORM_BUBBLE_IMAGE: &str = "bubble03_bg040.png";

pub(super) fn bubble_name(id: i32) -> Option<&'static str> {
    if id == BUBBLES[0] {
        Some(BUBBLE_IMAGE)
    } else if id == BUBBLES[1] || id == BUBBLES[2] {
        Some(SECOND_BUBBLE_IMAGE)
    } else if id == BUBBLE_STORM || STREAMS.contains(&id) {
        Some(STORM_BUBBLE_IMAGE)
    } else {
        None
    }
}

pub(super) fn uses_sparkles(id: i32) -> bool {
    NIGHT.contains(&id)
}

/// The sheets and images the particle systems draw from.
pub(super) struct Surfaces<'a> {
    pub(super) background: &'a SpriteSheet,
    pub(super) bubble: Option<&'a RgbaImage>,
    pub(super) sparkle: Option<&'a SpriteSheet>,
}

#[derive(Clone)]
pub(super) struct Particles {
    motes: Vec<[i32; 5]>,
    drifters: Vec<[i32; 4]>,
    stars: Vec<[i32; 3]>,
    sprites: Vec<[i32; 8]>,
}

impl Default for Particles {
    fn default() -> Self {
        Self {
            motes: vec![[0; 5]; MOTE_COUNT],
            drifters: vec![[0; 4]; DRIFTER_COUNT],
            stars: vec![[0; 3]; STAR_COUNT],
            sprites: vec![[0; 8]; SPRITE_COUNT],
        }
    }
}

fn field_width(world: &World) -> i32 {
    (world.stage_length / 10).wrapping_add(world.width.wrapping_mul(4)).wrapping_add(-0xf00)
}

fn depth(world: &World) -> i32 {
    100i32.wrapping_sub(world.min_zoom).wrapping_mul(20)
}

fn wrapped(value: i32, span: i32) -> i32 {
    value.checked_rem(span).unwrap_or(0)
}

fn floor_line(world: &World) -> i32 {
    let height = DESIGN_HEIGHT.wrapping_sub(world.zoom_y);
    let zoom = 100i32.wrapping_sub(world.min_zoom);
    let scaled = height.wrapping_mul(zoom);

    scaled.wrapping_sub((scaled / 100).wrapping_mul(100)).wrapping_sub(scaled).wrapping_add(0x1fbd0)
}

fn truncated(value: f32) -> i32 {
    if (-2147483648.0..2147483648.0).contains(&value) { value as i32 } else { i32::MIN }
}

fn scroll(world: &World) -> i32 {
    world.camera_x / -10
}

impl Particles {
    pub(super) fn animates(&self, world: &World) -> bool {
        world.is(&MOTES)
            || world.is(&SNOW)
            || world.is(&RISING)
            || world.is(&NIGHT)
            || world.id == BUBBLE_STORM
            || world.is(&LEAVES)
            || world.is(&STREAMS)
            || world.id == SPARKS
    }

    pub(super) fn init(&mut self, world: &World, rng: &mut Rng) {
        if world.is(&MOTES) {
            for (at, mote) in self.motes.iter_mut().enumerate() {
                let roll = rng.roll(0xc0).wrapping_add((at * 0xc0) as i32);

                mote[MOTE_X] = wrapped(roll, field_width(world));

                if world.is(&HIGH_MOTES) {
                    let depth = depth(world);

                    mote[MOTE_Y] = rng.roll(depth.wrapping_add(0xc8)).wrapping_sub(depth);
                } else if world.id == LOW_MOTES {
                    let depth = depth(world);

                    mote[MOTE_Y] = rng.roll(depth.wrapping_add(0xc8)).wrapping_sub(depth).wrapping_add(-0x3c);
                } else if world.id == DEEP_MOTES {
                    let depth = depth(world);

                    mote[MOTE_Y] = rng.roll(depth.wrapping_add(0xc8)).wrapping_sub(depth).wrapping_add(-0xc8);
                }

                mote[MOTE_PHASE] = 0;
                mote[MOTE_SPEED] = rng.roll(0x14).wrapping_add(5);
                mote[MOTE_KIND] = rng.roll(7);
            }
        }

        if world.is(&SNOW) {
            for (at, drifter) in self.drifters.iter_mut().enumerate() {
                let roll = rng.roll(0x38).wrapping_add((at * 0x38) as i32);
                let x = wrapped(roll, world.stage_length / 10);

                drifter[DRIFTER_X] = x.wrapping_add(world.width.wrapping_mul(4)).wrapping_mul(100).wrapping_add(-0x5dc00);

                let depth = depth(world);

                drifter[DRIFTER_Y] = rng.roll(depth.wrapping_add(DESIGN_HEIGHT)).wrapping_sub(depth).wrapping_mul(100);
                drifter[DRIFTER_ANGLE] = 0xafi32.wrapping_sub(rng.roll(0x55));
                drifter[DRIFTER_RADIUS] = rng.roll(0x258).wrapping_add(0xc8);
            }
        }

        if world.is(&RISING) {
            for (at, mote) in self.motes.iter_mut().enumerate() {
                let rising = rng.roll(5);
                let roll = rng.roll(0xc0).wrapping_add((at * 0xc0) as i32);

                mote[MOTE_X] = wrapped(roll, field_width(world));
                mote[MOTE_Y] = if rising != 0 {
                    depth(world).wrapping_neg()
                } else {
                    let reach = DESIGN_HEIGHT.wrapping_sub(world.zoom_y);
                    let spread = 100i32.wrapping_sub(world.min_zoom).wrapping_mul(reach);

                    if spread > 0x1f3ff { 0 } else { rng.roll((spread / -100).wrapping_add(0x500)) }
                };
                mote[MOTE_PHASE] = rng.roll(0x168);

                if world.is(&PETALS) {
                    mote[MOTE_SPEED] = rng.roll(0x28).wrapping_add(100);
                    mote[MOTE_KIND] = rng.roll(2);
                }
            }

            return;
        }

        if world.id == BUBBLE_STORM {
            for sprite in self.sprites.iter_mut() {
                sprite[SPRITE_X] = rng.roll(field_width(world)).wrapping_mul(100);

                let depth = depth(world);

                sprite[SPRITE_Y] = rng.roll(depth.wrapping_add(DESIGN_HEIGHT)).wrapping_sub(depth).wrapping_mul(100);
                sprite[SPRITE_ANGLE] = rng.roll(0x168);
                sprite[SPRITE_SIZE] = rng.roll(100).wrapping_add(0x32);
                sprite[SPRITE_RATE] = rng.roll(0x96);
                sprite[SPRITE_LIFE] = rng.roll(100);
            }

            return;
        }

        if world.is(&LEAVES) {
            for pair in 0..MOTE_COUNT {
                for half in 0..2usize {
                    let zoom = world.min_zoom;
                    let (bound, base) = if half == 0 {
                        let reach = (-0x122i32).wrapping_sub(100i32.wrapping_sub(zoom).wrapping_mul(20));
                        let y = (rng.roll(100).wrapping_mul(reach) / 100).wrapping_mul(100).wrapping_add(0x7148);

                        self.sprites[pair * 2][SPRITE_Y] = y;

                        let pick = rng.roll(10);

                        self.sprites[pair * 2][SPRITE_SIZE] = 0x14i32.wrapping_add(((pick as u32) < 1) as i32);

                        (0x32, 0x32)
                    } else {
                        let lift = 0x4e20i32.checked_div(zoom).unwrap_or(0);
                        let reach = lift.wrapping_add(100i32.wrapping_sub(zoom).wrapping_mul(20)).wrapping_add(0x1e0).wrapping_neg();
                        let y = (rng.roll(100).wrapping_mul(reach) / 100).wrapping_add(lift).wrapping_add(0x1e0).wrapping_mul(100);

                        self.sprites[pair * 2 + 1][SPRITE_Y] = y;
                        self.sprites[pair * 2 + 1][SPRITE_SIZE] = 0x14;

                        (0x64, 0xc8)
                    };
                    let sprite = &mut self.sprites[pair * 2 + half];

                    sprite[SPRITE_RATE] = rng.roll(bound).wrapping_add(base);

                    if world.is(&FAST_LEAVES) {
                        sprite[SPRITE_SIZE] = sprite[SPRITE_SIZE].wrapping_add(1);
                    }

                    sprite[SPRITE_X] = rng.roll(field_width(world)).wrapping_mul(100);
                    sprite[SPRITE_ANGLE] = 0x1e;
                    sprite[SPRITE_LIFE] = rng.roll(0x3c).wrapping_add(0x46);
                    sprite[SPRITE_TURN] = rng.roll(0x168);
                }
            }

            return;
        }

        if world.is(&STREAMS) {
            for sprite in self.sprites.iter_mut() {
                sprite[SPRITE_X] = rng.roll(field_width(world)).wrapping_mul(100);

                let depth = depth(world);

                sprite[SPRITE_Y] = rng.roll(depth.wrapping_add(DESIGN_HEIGHT)).wrapping_sub(depth).wrapping_mul(100);
                sprite[SPRITE_ANGLE] = rng.roll(0x168);
                sprite[SPRITE_SIZE] = rng.roll(100).wrapping_add(0x32);
                sprite[SPRITE_RATE] = rng.roll(0x96);
                sprite[SPRITE_LIFE] = rng.roll(0xbb8).wrapping_add(0x3e8);
                sprite[SPRITE_TURN] = 0x7d0;
            }

            return;
        }

        if world.id == SPARKS {
            for mote in self.motes.iter_mut() {
                mote[MOTE_X] = !rng.roll(0x3c);
            }
        }
    }

    pub(super) fn tick(&mut self, world: &World, rng: &mut Rng) {
        if world.is(&MOTES) {
            for (at, mote) in self.motes.iter_mut().enumerate() {
                let timer = mote[MOTE_PHASE].wrapping_add(mote[MOTE_SPEED]);

                mote[MOTE_PHASE] = timer;

                if timer < 0x190 {
                    continue;
                }

                let spread = rng.roll(0xc0).wrapping_add((at as i32).wrapping_mul(0xc0));

                mote[MOTE_X] = wrapped(spread, field_width(world));

                if world.is(&HIGH_MOTES) {
                    let depth = depth(world);

                    mote[MOTE_Y] = rng.roll(depth.wrapping_add(0xc8)).wrapping_sub(depth);
                } else if world.id == LOW_MOTES {
                    let depth = depth(world);

                    mote[MOTE_Y] = rng.roll(depth.wrapping_add(0xc8)).wrapping_sub(depth).wrapping_add(-0x3c);
                } else if world.id == DEEP_MOTES {
                    let depth = depth(world);

                    mote[MOTE_Y] = rng.roll(depth.wrapping_add(0xc8)).wrapping_sub(depth).wrapping_add(-0xc8);
                }

                mote[MOTE_PHASE] = 0;
                mote[MOTE_SPEED] = rng.roll(0x14).wrapping_add(5);
                mote[MOTE_KIND] = rng.roll(7);
            }
        }

        if world.is(&SNOW) {
            for drifter in self.drifters.iter_mut() {
                let turn = cos_deg(drifter[DRIFTER_ANGLE] as f32);
                let x = truncated(drifter[DRIFTER_RADIUS] as f32 * turn + drifter[DRIFTER_X] as f32);

                drifter[DRIFTER_X] = x;

                let stage = world.stage_length.wrapping_mul(5);
                let moved = x
                    .wrapping_add(stage.wrapping_mul(2))
                    .wrapping_add(world.width.wrapping_mul(0x190))
                    .wrapping_add(-0x5dc00);
                let span = world.width.wrapping_mul(0x190).wrapping_add(stage.wrapping_mul(2)).wrapping_add(-0x5dc00);

                drifter[DRIFTER_X] = wrapped(moved, span);

                let lift = sin_deg(drifter[DRIFTER_ANGLE] as f32);
                let y = truncated(drifter[DRIFTER_RADIUS] as f32 * lift + drifter[DRIFTER_Y] as f32);

                drifter[DRIFTER_Y] = y;

                if floor_line(world) > y {
                    continue;
                }

                drifter[DRIFTER_Y] = 100i32.wrapping_sub(world.min_zoom).wrapping_mul(-0x7d0);
                drifter[DRIFTER_ANGLE] = 0xafi32.wrapping_sub(rng.roll(0x55));
                drifter[DRIFTER_RADIUS] = rng.roll(0x258).wrapping_add(0xc8);
            }
        }

        if world.is(&RISING) {
            for (at, mote) in self.motes.iter_mut().enumerate() {
                let y = if world.is(&PETALS) { mote[MOTE_Y].wrapping_sub(1) } else { mote[MOTE_Y].wrapping_add(-3) };

                mote[MOTE_Y] = y;
                mote[MOTE_PHASE] = mote[MOTE_PHASE].wrapping_add(1);

                if y >= depth(world).wrapping_neg() {
                    continue;
                }

                if rng.roll(0x7d0) != 0 {
                    continue;
                }

                let spread = rng.roll(0xc0).wrapping_add((at as i32).wrapping_mul(0xc0));

                mote[MOTE_X] = wrapped(spread, field_width(world));

                let height = DESIGN_HEIGHT.wrapping_sub(world.zoom_y);
                let zoom = 100i32.wrapping_sub(world.min_zoom);

                mote[MOTE_Y] = (zoom.wrapping_mul(height) / -100).wrapping_add(0x500);
                mote[MOTE_PHASE] = rng.roll(0x168);

                if world.is(&PETALS) {
                    mote[MOTE_SPEED] = rng.roll(0x28).wrapping_add(0x64);
                    mote[MOTE_KIND] = rng.roll(2);
                }
            }
        }

        if world.is(&NIGHT) {
            for drifter in self.drifters.iter_mut().take(GLINT_COUNT) {
                drifter[DRIFTER_X] = rng.roll(field_width(world));
                drifter[DRIFTER_Y] = rng.roll(0x5a).wrapping_add(0x1a4);
            }

            for star in self.stars.iter_mut() {
                star[STAR_X] = rng.roll(field_width(world));

                let depth = depth(world);

                star[STAR_Y] = rng.roll(depth.wrapping_add(DESIGN_HEIGHT)).wrapping_sub(depth);
                star[STAR_GLOW] = rng.roll(0x64).wrapping_add(0x64);
            }
        }

        if world.id == BUBBLE_STORM {
            for sprite in self.sprites.iter_mut() {
                let rise = sprite[SPRITE_LIFE].wrapping_add(0xaa).wrapping_mul(sprite[SPRITE_SIZE]) / 100;
                let y = sprite[SPRITE_Y].wrapping_add(rise);

                sprite[SPRITE_Y] = y;
                sprite[SPRITE_ANGLE] = sprite[SPRITE_ANGLE].wrapping_add(1);

                if y < floor_line(world) {
                    continue;
                }

                sprite[SPRITE_X] = rng.roll(field_width(world)).wrapping_mul(0x64);
                sprite[SPRITE_Y] = 100i32.wrapping_sub(world.min_zoom).wrapping_mul(-0x7d0);
                sprite[SPRITE_ANGLE] = rng.roll(0x168);
                sprite[SPRITE_SIZE] = rng.roll(0x96).wrapping_add(0x32);
                sprite[SPRITE_RATE] = rng.roll(0x64);
                sprite[SPRITE_LIFE] = rng.roll(0x64);
            }
        } else if world.is(&LEAVES) {
            for pair in 0..MOTE_COUNT {
                for half in 0..2usize {
                    let sprite = &mut self.sprites[pair * 2 + half];
                    let fall = sprite[SPRITE_LIFE].wrapping_mul(sprite[SPRITE_RATE]) / -100;
                    let y = sprite[SPRITE_Y].wrapping_add(fall);

                    sprite[SPRITE_Y] = y;
                    sprite[SPRITE_ANGLE] = sprite[SPRITE_ANGLE].wrapping_add(1);

                    if y >= 100i32.wrapping_sub(world.min_zoom).wrapping_mul(-0x7d0) {
                        continue;
                    }

                    sprite[SPRITE_X] = rng.roll(field_width(world)).wrapping_mul(0x64);

                    let (bound, base) = if half == 0 {
                        let y = 0x154i32.wrapping_sub(rng.roll(0x32)).wrapping_mul(0x64);

                        sprite[SPRITE_Y] = y;

                        let life = if rng.roll(0xa) < 1 { 0x15 } else { 0x14 };

                        self.sprites[pair * 2][SPRITE_SIZE] = life;

                        (0x32, 0x32)
                    } else {
                        let lift = 0x4e20i32.checked_div(world.min_zoom).unwrap_or(0);
                        let y = rng.roll(0x32).wrapping_add(lift).wrapping_mul(0x64).wrapping_add(0xbb80);

                        sprite[SPRITE_Y] = y;
                        sprite[SPRITE_SIZE] = 0x14;

                        (0x64, 0xc8)
                    };
                    let sprite = &mut self.sprites[pair * 2 + half];

                    sprite[SPRITE_RATE] = rng.roll(bound).wrapping_add(base);

                    if world.is(&FAST_LEAVES) {
                        sprite[SPRITE_SIZE] = sprite[SPRITE_SIZE].wrapping_add(1);
                    }

                    sprite[SPRITE_ANGLE] = 0;
                    sprite[SPRITE_LIFE] = rng.roll(0x3c).wrapping_add(0x46);
                    sprite[SPRITE_TURN] = rng.roll(0x168);
                }
            }
        } else if world.is(&STREAMS) {
            for sprite in self.sprites.iter_mut() {
                let speed = sprite[SPRITE_SIZE];
                let x = (sprite[SPRITE_LIFE].wrapping_add(0xaa).wrapping_mul(speed) / 100).wrapping_add(sprite[SPRITE_X]);

                sprite[SPRITE_X] = x;

                let rise = sprite[SPRITE_TURN].wrapping_add(0xaa).wrapping_mul(speed) / 100;

                sprite[SPRITE_Y] = sprite[SPRITE_Y].wrapping_add(rise);
                sprite[SPRITE_ANGLE] = sprite[SPRITE_ANGLE].wrapping_add(1);

                let stage = world.stage_length / 10;
                let edge = stage.wrapping_add(world.width.wrapping_mul(4)).wrapping_mul(0x64).wrapping_add(-0x5dc00);

                if x >= edge {
                    sprite[SPRITE_X] = 0;
                }

                if sprite[SPRITE_Y] < floor_line(world) {
                    continue;
                }

                sprite[SPRITE_X] = rng.roll(field_width(world)).wrapping_mul(0x64);
                sprite[SPRITE_Y] = 100i32.wrapping_sub(world.min_zoom).wrapping_mul(-0x7d0);
                sprite[SPRITE_ANGLE] = rng.roll(0x168);
                sprite[SPRITE_SIZE] = rng.roll(0x96).wrapping_add(0x32);
                sprite[SPRITE_RATE] = rng.roll(0x64);
                sprite[SPRITE_LIFE] = rng.roll(0x1770).wrapping_add(0x7d0);
                sprite[SPRITE_TURN] = 0xfa0;
            }
        } else if world.id == SPARKS {
            for mote in self.motes.iter_mut() {
                let old = mote[MOTE_X];

                mote[MOTE_X] = old.wrapping_add(1);

                if old.wrapping_add(1) == 0 {
                    mote[MOTE_Y] = rng.roll(field_width(world)).wrapping_mul(10);
                    mote[MOTE_PHASE] = rng.roll(0x1c2).wrapping_add(0x96);
                } else if old >= 0xa {
                    mote[MOTE_X] = !rng.roll(0x3c);
                }
            }
        }
    }

    pub(super) fn draw_behind(&self, world: &World, surfaces: Surfaces<'_>, painter: &mut Painter) {
        painter.set_tint(Rgba::new(0xff, 0xff, 0xff, 0xff));

        if world.is(&MOTES) {
            let mut temp = [0i32; 4];

            for mote in &self.motes {
                let depth = mote[MOTE_PHASE];

                if depth > 0x190 {
                    continue;
                }

                let kind = mote[MOTE_KIND];
                let mut lit = true;

                if kind != 0 && world.id != LOW_MOTES {
                    let shade = kind.wrapping_add(-1);

                    if (shade as u32) > 5 {
                        lit = false;
                    } else {
                        let shade = shade as usize;

                        temp[0] = PARTICLE_RED[shade];
                        temp[1] = PARTICLE_GREEN[shade];
                        temp[2] = PARTICLE_BLUE[shade];
                    }
                } else {
                    temp[0] = 0xff;
                    temp[1] = 0xff;
                    temp[2] = 0xff;
                }

                if lit {
                    temp[3] = truncated(sin_deg(depth as f32 * 0.45) * 255.0);
                }

                painter.set_tint(Rgba::new(temp[0], temp[1], temp[2], 0xff));
                painter.set_tint_alpha(temp[3]);

                let drift = scroll(world).wrapping_add(mote[MOTE_X]);
                let x = drift.wrapping_sub(world.width).wrapping_add(DESIGN_WIDTH);

                painter.fill_rect(x, mote[MOTE_Y], 4, 4);
                painter.set_tint_alpha(0xff);
            }
        }

        if world.is(&NIGHT)
            && let Some(sparkle) = surfaces.sparkle
        {
            painter.set_alpha(0x32);

            for drifter in self.drifters.iter().take(GLINT_COUNT) {
                painter.draw_cut(Surface::Sparkle, sparkle, drifter[DRIFTER_X], drifter[DRIFTER_Y], GLINT_CUT);
            }

            painter.set_alpha(0xff);
        }

        if world.is(&LEAVES) {
            for pair in 0..MOTE_COUNT {
                let sprite = &self.sprites[pair * 2];
                let fade = sprite[SPRITE_ANGLE];

                if fade <= 0x1d {
                    painter.set_alpha((fade << 8).wrapping_sub(fade) / 30);
                }

                let left = scroll(world).wrapping_add(sprite[SPRITE_X] / 100);
                let x = left.wrapping_sub(world.width).wrapping_add(0x3c4);
                let y = sprite[SPRITE_Y] / 100;
                let cut = sprite[SPRITE_SIZE];
                let (wide, tall) = Painter::cut_of(surfaces.background, cut).map_or((0, 0), |cell| (cell[2], cell[3]));
                let span = sprite[SPRITE_RATE].wrapping_mul(wide) / 100;
                let height = sprite[SPRITE_RATE].wrapping_mul(tall) / 100;
                let angle = sprite[SPRITE_TURN] as f32;

                painter.draw_cut_rotated(
                    Surface::Background,
                    surfaces.background,
                    Spin { x, y, width: span, height, angle, align: 5, pivot_x: 0, pivot_y: 0, pivot_align: 5 },
                    cut,
                );
                painter.set_alpha(0xff);
            }
        }

        if world.id == SPARKS {
            painter.glow_set(1);

            for mote in &self.motes {
                let life = mote[MOTE_X];

                if life < 0 {
                    continue;
                }

                let scale = if life > 2 {
                    let raised = life.wrapping_mul(0x64);
                    let shifted = raised.wrapping_add(-0x12c);
                    let folded = if shifted >= 0 { shifted } else { raised.wrapping_add(-0x125) };

                    0x64i32.wrapping_sub(folded >> 3)
                } else {
                    (life.wrapping_mul(0x64) as u8 as i32) / 3
                };
                let (wide, tall) = Painter::cut_of(surfaces.background, SPARK_CUT).map_or((0, 0), |cell| (cell[2], cell[3]));
                let span = wide.wrapping_mul(scale) / 100;
                let height = tall.wrapping_mul(scale) / 100;
                let x = (wide.wrapping_mul(scale) / -200)
                    .wrapping_add(mote[MOTE_Y])
                    .wrapping_add(scroll(world))
                    .wrapping_sub(world.width)
                    .wrapping_add(DESIGN_WIDTH);
                let y = (tall.wrapping_mul(scale) / -200).wrapping_add(mote[MOTE_PHASE]);

                painter.draw_cut_scaled(Surface::Background, surfaces.background, [x, y, span, height], SPARK_CUT);
            }

            painter.glow_set(0);
        }
    }

    pub(super) fn draw_in_front(&self, world: &World, surfaces: Surfaces<'_>, painter: &mut Painter) {
        if world.is(&LEAVES) {
            for pair in 0..LEAF_FRONT_COUNT {
                let sprite = &self.sprites[pair * 2 + 1];
                let fade = sprite[SPRITE_ANGLE];

                if fade <= 0x1d {
                    painter.set_alpha((fade << 8).wrapping_sub(fade) / 30);
                }

                let left = scroll(world).wrapping_add(sprite[SPRITE_X] / 100);
                let phase = sprite[SPRITE_ANGLE];
                let odd = phase.wrapping_sub(phase.wrapping_add((phase as u32 >> 31) as i32) & 0x7ffffffe);
                let x = left.wrapping_sub(world.width).wrapping_add(odd.wrapping_mul(2)).wrapping_add(0x3c4);
                let y = sprite[SPRITE_Y] / 100;
                let cut = sprite[SPRITE_SIZE];
                let (wide, tall) = Painter::cut_of(surfaces.background, cut).map_or((0, 0), |cell| (cell[2], cell[3]));
                let span = sprite[SPRITE_RATE].wrapping_mul(wide) / 100;
                let height = sprite[SPRITE_RATE].wrapping_mul(tall) / 100;
                let angle = sprite[SPRITE_TURN] as f32;

                painter.draw_cut_rotated(
                    Surface::Background,
                    surfaces.background,
                    Spin { x, y, width: span, height, angle, align: 5, pivot_x: 0, pivot_y: 0, pivot_align: 5 },
                    cut,
                );
                painter.set_alpha(0xff);
            }
        }

        if world.is(&SNOW) {
            let left = scroll(world).wrapping_add(DESIGN_WIDTH);

            for drifter in &self.drifters {
                let mut xs = [0i32; 8];
                let mut ys = [0i32; 8];

                for (corner, (x, y)) in xs.iter_mut().zip(ys.iter_mut()).enumerate() {
                    let turn = (corner * 0x2d) as i32 as f32;
                    let across = (drifter[DRIFTER_X] / 100).wrapping_add(left).wrapping_sub(world.width) as f32;

                    *x = truncated(cos_deg(turn) * 12.0 + across);

                    let down = (drifter[DRIFTER_Y] / 100) as f32;

                    *y = truncated(sin_deg(turn) * 12.0 + down);
                }

                painter.set_tint(Rgba::new(0xff, 0xff, 0xff, 0x7f));
                painter.fill_polygon(&xs, &ys);
            }
        }

        if world.is(&NIGHT)
            && let Some(sparkle) = surfaces.sparkle
        {
            for star in &self.stars {
                painter.set_alpha(star[STAR_GLOW]);
                painter.draw_cut_spun(
                    Surface::Sparkle,
                    sparkle,
                    Spin {
                        x: star[STAR_X],
                        y: star[STAR_Y],
                        width: 0,
                        height: 0,
                        angle: 60.0,
                        align: 5,
                        pivot_x: 0,
                        pivot_y: 0,
                        pivot_align: 5,
                    },
                    STAR_CUT,
                );
            }

            painter.set_alpha(0xff);
        }

        if world.is(&BUBBLES) {
            if let Some(bubble) = surfaces.bubble {
                for mote in &self.motes {
                    let across = scroll(world)
                        .wrapping_add(mote[MOTE_X])
                        .wrapping_sub(world.width)
                        .wrapping_add(DESIGN_WIDTH) as f32;
                    let sway = mote[MOTE_PHASE].wrapping_mul(0x12) as f32;
                    let x = truncated(cos_deg(sway) * 10.0 + across);

                    painter.draw_image_scaled(Surface::Bubble, bubble, x, mote[MOTE_Y], BUBBLE_SIZE, BUBBLE_SIZE);
                }
            }
        } else if world.is(&PETALS) {
            for mote in self.motes.iter().take(PETAL_COUNT) {
                let cut = mote[MOTE_KIND].wrapping_add(PETAL_CUT);
                let (cut_wide, cut_tall) = Painter::cut_of(surfaces.background, cut).map_or((0, 0), |cell| (cell[2], cell[3]));
                let wide = mote[MOTE_SPEED].wrapping_mul(cut_wide);
                let width = wide / 100;
                let tall = mote[MOTE_SPEED].wrapping_mul(cut_tall);
                let height = tall / 100;
                let across = scroll(world)
                    .wrapping_add(mote[MOTE_X])
                    .wrapping_sub(world.width)
                    .wrapping_add(DESIGN_WIDTH) as f32;
                let sway = mote[MOTE_PHASE].wrapping_mul(3) as f32;
                let x = truncated(cos_deg(sway) * 10.0 + across - (wide / 200) as f32);
                let y = (tall / -200).wrapping_add(mote[MOTE_Y]);

                painter.draw_cut_scaled(Surface::Background, surfaces.background, [x, y, width, height], cut);
            }
        }

        if world.id == BUBBLE_STORM
            && let Some(bubble) = surfaces.bubble
        {
            for sprite in &self.sprites {
                let across = (sprite[SPRITE_X] / 100)
                    .wrapping_add(scroll(world))
                    .wrapping_sub(world.width)
                    .wrapping_add(DESIGN_WIDTH) as f32;
                let size = (sprite[SPRITE_SIZE] << 2) as f32;
                let sway = (sprite[SPRITE_RATE].wrapping_mul(sprite[SPRITE_ANGLE]) / 100) as f32;
                let x = truncated(cos_deg(sway) * size / 100.0 + across);
                let y = sprite[SPRITE_Y] / 100;
                let size = sprite[SPRITE_SIZE] / 5;

                painter.draw_image_scaled(Surface::Bubble, bubble, x, y, size, size);
            }
        }

        if world.is(&STREAMS)
            && let Some(bubble) = surfaces.bubble
        {
            for sprite in &self.sprites {
                let across = (sprite[SPRITE_X] / 100)
                    .wrapping_add(scroll(world))
                    .wrapping_sub(world.width)
                    .wrapping_add(DESIGN_WIDTH) as f32;
                let size = (sprite[SPRITE_SIZE] << 2) as f32;
                let sway = (sprite[SPRITE_RATE].wrapping_mul(sprite[SPRITE_ANGLE]) / 100) as f32;
                let x = truncated(cos_deg(sway) * size / 100.0 + across);
                let y = sprite[SPRITE_Y] / 100;
                let width = sprite[SPRITE_SIZE] / 10;
                let height = sprite[SPRITE_SIZE] / 5;

                painter.draw_image_rotated(
                    Surface::Bubble,
                    bubble,
                    Spin { x, y, width, height, angle: 315.0, align: 0, pivot_x: width, pivot_y: height, pivot_align: 0 },
                );
            }
        }
    }
}
