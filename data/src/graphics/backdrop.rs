//! The battle background, composed the way the engine draws it.
//!
//! A background is a sky and a ground filled from the colors its table row
//! names, a ground tile repeated across the stage, an optional tile hung above
//! the sky, an optional fog laid over everything, and for some backgrounds a
//! set of particles or scripted effects that move every frame. The particles
//! are chosen by the background's identifier, and both they and the effects
//! draw from the engine's random number generator, so a scene is fully decided
//! by its seed.
//!
//! The scene is placed in the design space of the screen the game is designed
//! against, nine hundred and sixty by six hundred and forty pixels, and is
//! composed through a camera that may sit anywhere and zoom to any scale. A
//! frame comes back as flat quads in screen pixels any canvas implementation
//! can draw.

mod effects;
mod particles;

use std::collections::BTreeMap;
use std::sync::Arc;

use image::RgbaImage;

use crate::chapter::stage::{BackgroundEntry, Rgba};
use crate::common::Rng;

use super::animate::{resolve_frame, FrameData};
use super::rig::{Animation, Model, Rig, SpriteSheet};

pub use effects::{Base, Bound, EffectDef, EffectFile, EffectInstance, EffectsError, EffectsFile, Param, Spaced};

use effects::Frame;
use particles::Particles;

/// The width of the design screen, which is also the width of one ground tile.
const DESIGN_WIDTH: i32 = 0x3c0;

/// The height of the design screen.
const DESIGN_HEIGHT: i32 = 0x280;

/// The height of the ground cut the tile's height is measured against.
const TILE_SOURCE_HEIGHT: i32 = 510;

/// The height of the band the sky's gradient starts above the screen.
const SKY_LIFT: i32 = -0x28;

/// The height the ground's gradient ends below the design screen.
const GROUND_DROP: i32 = 0x2a8;

/// The design height the horizon sits at.
const HORIZON: i32 = 0x208;

/// The camera zoom at full scale, in hundredths of a percent.
const FULL_ZOOM: i32 = 10000;

/// The design height the deck bar's top edge sits at.
const DECK_BAR_TOP: i32 = 0x220;

/// The stage length the camera's lowest zoom is measured against.
const ZOOM_REACH: i32 = 0xea600;

/// The engine units one screen width spans, in tenths of a design pixel.
const SCREEN_SPAN: i32 = 0x2580;

/// The engine units a stage's declared width is multiplied by.
const STAGE_SCALE: i32 = 4;

/// The index of the ground tile in the background's cut list.
const GROUND_CUT: i32 = 0;

/// The index of the tile hung above the sky in the background's cut list.
const UPPER_CUT: i32 = 0x14;

/// The backgrounds the engine lays a colored gradient over.
const TINTED: [(i32, u32, u32); 8] = [
    (0xd, 0x33ffffff, 0x3300e2ff),
    (0xf, 0x3342bbff, 0x330049ad),
    (0x13, 0x33f0a936, 0x33a02120),
    (0x47, 0x33eba03c, 0x33912d05),
    (0x48, 0x33000000, 0x3300237d),
    (0x49, 0x333c8705, 0x330f1e78),
    (0xa7, 0x4c896858, 0x4caf6868),
    (0xa8, 0x4c444b4d, 0x4c314961),
];

/// The backgrounds the engine lays a white gradient over.
const WHITENED: [i32; 2] = [0x2e, 0x2f];

/// The background the engine lays a flat white wash over.
const WASHED: i32 = 0x9c;

/// The alpha of the flat white wash.
const WASH_ALPHA: i32 = 0x4c;

/// The depth layer drawn behind the units.
const BEHIND: i32 = 2;

/// The depth layer drawn in front of the units.
const IN_FRONT: i32 = 1;

/// The depth layer drawn over everything.
const OVER: i32 = 0;

/// The sprite sheet the sparkles some backgrounds scatter over the sky are cut from.
pub const SPARKLE_SHEET: &str = "000_a.png";

/// The cut list of the sparkle sheet.
pub const SPARKLE_CUTS: &str = "000_a.imgcut";

/// Returns the name of the sprite sheet image a background's tiles are read from.
///
/// # Arguments
/// * `entry` - The background's row of the table.
///
/// # Returns
/// A `String` holding the file name.
pub fn sheet_name(entry: &BackgroundEntry) -> String {
    format!("bg{:03}.png", entry.image())
}

/// Returns the name of the sprite cut list a background's tiles are read from.
///
/// # Arguments
/// * `entry` - The background's row of the table.
///
/// # Returns
/// A `String` holding the file name.
pub fn cuts_name(entry: &BackgroundEntry) -> String {
    format!("bg{:02}.imgcut", entry.model_id)
}

/// Returns the name of one of a background's own numbered models.
///
/// The engine loads the models from index zero upwards until a name is missing.
///
/// # Arguments
/// * `entry` - The background's row of the table.
/// * `index` - The model's index.
///
/// # Returns
/// A `String` holding the file name.
pub fn model_name(entry: &BackgroundEntry, index: usize) -> String {
    format!("bg{:02}_{:02}.mamodel", entry.model_id, index)
}

/// Returns the name of the animation one of a background's own numbered models plays.
///
/// # Arguments
/// * `entry` - The background's row of the table.
/// * `index` - The model's index.
///
/// # Returns
/// A `String` holding the file name.
pub fn animation_name(entry: &BackgroundEntry, index: usize) -> String {
    format!("bg{:02}_{:02}.maanim", entry.model_id, index)
}

/// Returns the name of the effect file a background spawns its effects from.
///
/// # Arguments
/// * `id` - The background identifier, or the identifier an effect file redirects to.
///
/// # Returns
/// A `String` holding the file name.
pub fn effects_name(id: u32) -> String {
    format!("bg{id:03}.json")
}

/// Returns the name of the bubble image a background floats over the scene.
///
/// # Arguments
/// * `id` - The background identifier.
///
/// # Returns
/// An `Option` containing the file name, or `None` for a background that floats no bubbles.
pub fn bubble_name(id: u32) -> Option<&'static str> {
    particles::bubble_name(id as i32)
}

/// Returns whether a background scatters sparkles from the sparkle sheet.
///
/// # Arguments
/// * `id` - The background identifier.
///
/// # Returns
/// A `bool` that is true when the sheet is drawn.
pub fn uses_sparkles(id: u32) -> bool {
    particles::uses_sparkles(id as i32)
}

/// The screen a scene is composed for, in design pixels.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Screen {
    /// The drawable width.
    pub width: i32,
    /// The drawable height.
    pub height: i32,
    /// The height of the band the engine shifts the scene down by on a tall screen.
    pub letterbox: i32,
}

impl Default for Screen {
    fn default() -> Self {
        Self { width: DESIGN_WIDTH, height: DESIGN_HEIGHT, letterbox: 0 }
    }
}

/// The camera a frame is composed through.
///
/// The engine scales the scene about the horizon and shifts it sideways to
/// follow the camera, while the sky and ground fills, the tints and the fog
/// cover the screen whatever the zoom, and the effects laid over the screen
/// are placed on it unscaled. A camera may sit anywhere and zoom to any
/// scale, and the frame it composes covers its whole screen.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Camera {
    /// The scene x shown at the horizontal center of the screen, in design pixels.
    pub x: f32,
    /// How far below its design position the horizon sits on the screen, in screen pixels.
    pub y: f32,
    /// The scale the scene is drawn at, where one is the design scale.
    pub zoom: f32,
    /// The width of the screen in pixels.
    pub width: f32,
    /// The height of the screen in pixels.
    pub height: f32,
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            x: DESIGN_WIDTH as f32 / 2.0,
            y: 0.0,
            zoom: 1.0,
            width: DESIGN_WIDTH as f32,
            height: DESIGN_HEIGHT as f32,
        }
    }
}

impl Camera {
    /// Projects a scene point onto the screen.
    ///
    /// # Arguments
    /// * `point` - The x and y of the point in design pixels.
    ///
    /// # Returns
    /// The x and y of the point in screen pixels.
    pub fn project(&self, point: [f32; 2]) -> [f32; 2] {
        [
            self.zoom * (point[0] - self.x) + self.width / 2.0,
            self.zoom * (point[1] - HORIZON as f32) + self.horizon(),
        ]
    }

    /// Returns the screen y the horizon sits at.
    ///
    /// # Returns
    /// An `f32` in screen pixels.
    pub fn horizon(&self) -> f32 {
        HORIZON as f32 + self.y
    }

    fn scene_left(&self) -> f32 {
        self.x - self.width / (2.0 * self.zoom.max(f32::EPSILON))
    }

    fn scene_right(&self) -> f32 {
        self.x + self.width / (2.0 * self.zoom.max(f32::EPSILON))
    }

    fn projected(&self, quad: &Quad) -> Quad {
        Quad { corners: quad.corners.map(|corner| self.project(corner)), ..quad.clone() }
    }
}

/// The image a quad samples.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Surface {
    /// The background's own sprite sheet.
    Background,
    /// The bubble image some backgrounds float over the scene.
    Bubble,
    /// The sparkle sheet some backgrounds scatter over the sky.
    Sparkle,
    /// One of the sprite sheets the background's effects name, by its index in the assets.
    Effect(usize),
}

/// One textured or flat quad of the composed scene.
///
/// The corners run from the top left clockwise, which is the order the
/// engine submits them in, and a flat quad leaves its surface and source empty.
/// The colors are premultiplied by their alpha, so a canvas blends them with
/// one and one minus the source alpha.
#[derive(Debug, Clone, PartialEq)]
pub struct Quad {
    /// The image the quad samples, or `None` for a flat color.
    pub surface: Option<Surface>,
    /// The corner positions, in screen pixels once composed through a camera.
    pub corners: [[f32; 2]; 4],
    /// The sampled region of the surface as its left, top, width and height in source pixels.
    pub source: [f32; 4],
    /// The premultiplied color at each corner.
    pub colors: [[f32; 4]; 4],
    /// The blending mode, where zero is ordinary alpha blending, one adds, two multiplies and three subtracts.
    pub blend: u8,
}

/// The moving parts of a frame, drawn once and composed onto any view.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Layers {
    /// The particles and effects behind and in front of the units, in draw order.
    pub scene: Vec<Quad>,
    /// The effects laid over the screen, in draw order.
    pub over: Vec<Quad>,
}

/// A model and animation one of a background's effects draws.
#[derive(Debug, Clone)]
pub struct EffectRig {
    /// The index of the sprite sheet in [`Assets::sheets`] the model draws from, or `None` for the background's own.
    pub sheet: Option<usize>,
    /// The model.
    pub model: Model,
    /// The animation the model plays.
    pub animation: Animation,
}

/// Everything a background needs loaded before it can be composed.
#[derive(Debug, Clone, Default)]
pub struct Assets {
    /// The background's row of the table.
    pub entry: BackgroundEntry,
    /// The background's sprite sheet, holding its ground tile and any tile hung above the sky.
    pub sheet: SpriteSheet,
    /// The bubble image, which some backgrounds float over the scene.
    pub bubble: Option<Arc<RgbaImage>>,
    /// The sparkle sheet, which some backgrounds scatter over the sky.
    pub sparkle: Option<SpriteSheet>,
    /// The effect definitions, after any redirect has been followed.
    pub defs: Vec<EffectDef>,
    /// The sprite sheets the effect definitions name, deduplicated by image.
    pub sheets: Vec<SpriteSheet>,
    /// The rig each definition draws, parallel to `defs`, holding `None` for a definition naming no files of its own.
    pub files: Vec<Option<EffectRig>>,
    /// The rigs the background's own numbered models declare, in model order.
    pub models: Vec<EffectRig>,
}

#[derive(Debug, Clone, Copy)]
pub(super) struct World {
    pub(super) id: i32,
    pub(super) stage_length: i32,
    pub(super) width: i32,
    pub(super) height: i32,
    pub(super) letterbox: i32,
    pub(super) camera_x: i32,
    pub(super) min_zoom: i32,
    pub(super) zoom_y: i32,
}

impl World {
    pub(super) fn is(&self, ids: &[i32]) -> bool {
        ids.contains(&self.id)
    }
}

pub(super) struct Painter {
    quads: Vec<Quad>,
    color: [i32; 4],
    tint: [i32; 4],
    glow: i32,
}

#[derive(Debug, Clone, Copy)]
pub(super) struct Spin {
    pub(super) x: i32,
    pub(super) y: i32,
    pub(super) width: i32,
    pub(super) height: i32,
    pub(super) angle: f32,
    pub(super) align: i32,
    pub(super) pivot_x: i32,
    pub(super) pivot_y: i32,
    pub(super) pivot_align: i32,
}

const ALIGN_CENTER: i32 = 1;
const ALIGN_RIGHT: i32 = 2;
const ALIGN_MIDDLE: i32 = 4;
const ALIGN_BOTTOM: i32 = 8;
const PIVOT_ABSOLUTE: i32 = 0x10;

fn premultiplied(color: [i32; 4]) -> [f32; 4] {
    let alpha = color[3];

    [
        color[0].wrapping_mul(alpha) as f32 / 65025.0,
        color[1].wrapping_mul(alpha) as f32 / 65025.0,
        color[2].wrapping_mul(alpha) as f32 / 65025.0,
        alpha as f32 / 255.0,
    ]
}

fn channels(color: Rgba) -> [i32; 4] {
    [color.red & 0xff, color.green & 0xff, color.blue & 0xff, color.alpha & 0xff]
}

fn argb(packed: u32) -> Rgba {
    Rgba::new(
        (packed >> 0x10 & 0xff) as i32,
        (packed >> 8 & 0xff) as i32,
        (packed & 0xff) as i32,
        (packed >> 0x18) as i32,
    )
}

fn aligned(extent: i32, align: i32, half: i32, full: i32) -> i32 {
    if align & half != 0 {
        extent / 2
    } else if align & full != 0 {
        extent
    } else {
        0
    }
}

fn truncated(value: f32) -> i32 {
    if (-2147483648.0..2147483648.0).contains(&value) { value as i32 } else { i32::MIN }
}

pub(super) fn sin_deg(degrees: f32) -> f32 {
    ((degrees as f64) * std::f64::consts::PI / 180.0).sin() as f32
}

pub(super) fn cos_deg(degrees: f32) -> f32 {
    ((degrees as f64) * std::f64::consts::PI / 180.0).cos() as f32
}

impl Painter {
    fn new() -> Self {
        Self { quads: Vec::new(), color: [0xff; 4], tint: [0xff; 4], glow: 0 }
    }

    pub(super) fn set_tint(&mut self, color: Rgba) {
        self.tint = channels(color);
    }

    pub(super) fn set_tint_alpha(&mut self, alpha: i32) {
        self.tint[3] = alpha;
    }

    pub(super) fn set_alpha(&mut self, alpha: i32) {
        self.color[3] = alpha;
    }

    pub(super) fn glow_set(&mut self, mode: i32) {
        self.glow = mode;
    }

    fn push(&mut self, surface: Option<Surface>, corners: [[f32; 2]; 4], source: [i32; 4], color: [f32; 4]) {
        self.quads.push(Quad {
            surface,
            corners,
            source: source.map(|cell| cell as f32),
            colors: [color; 4],
            blend: self.glow as u8,
        });
    }

    fn fill_box(&mut self, left: f32, top: f32, right: f32, bottom: f32, colors: [Rgba; 4]) {
        let corners = [[left, top], [left, bottom], [right, bottom], [right, top]];
        let colors = colors.map(|color| premultiplied(channels(color)));

        self.quads.push(Quad { surface: None, corners, source: [0.0; 4], colors, blend: self.glow as u8 });
    }

    pub(super) fn fill_rect(&mut self, x: i32, y: i32, width: i32, height: i32) {
        let (x, y, width, height) = (x as f32, y as f32, width as f32, height as f32);
        let corners = [[x, y], [x, y + height], [x + width, y + height], [x + width, y]];

        self.push(None, corners, [0; 4], premultiplied(self.tint));
    }

    pub(super) fn fill_polygon(&mut self, xs: &[i32], ys: &[i32]) {
        let colors = vec![argb_of(self.tint); xs.len()];

        self.fill_polygon_colored(xs, ys, &colors);
    }

    pub(super) fn fill_polygon_colored(&mut self, xs: &[i32], ys: &[i32], colors: &[Rgba]) {
        let count = xs.len().min(ys.len()).min(colors.len());
        let corner = |at: usize| ([xs[at] as f32, ys[at] as f32], premultiplied(channels(colors[at])));

        if count == 4 {
            let (a, b, c, d) = (corner(0), corner(1), corner(2), corner(3));

            self.quads.push(Quad {
                surface: None,
                corners: [a.0, b.0, c.0, d.0],
                source: [0.0; 4],
                colors: [a.1, b.1, c.1, d.1],
                blend: self.glow as u8,
            });

            return;
        }

        for second in 1..count.saturating_sub(1) {
            let (a, b, c) = (corner(0), corner(second), corner(second + 1));

            self.quads.push(Quad {
                surface: None,
                corners: [a.0, b.0, c.0, c.0],
                source: [0.0; 4],
                colors: [a.1, b.1, c.1, c.1],
                blend: self.glow as u8,
            });
        }
    }

    fn blit(&mut self, surface: Surface, source: [i32; 4], x: f32, y: f32, width: f32, height: f32) {
        let corners = [[x, y], [x, y + height], [x + width, y + height], [x + width, y]];

        self.push(Some(surface), corners, source, premultiplied(self.color));
    }

    fn cut_of(sheet: &SpriteSheet, cut: i32) -> Option<[i32; 4]> {
        let cut = sheet.cuts.get(usize::try_from(cut).ok()?)?;

        Some([cut.x, cut.y, cut.width, cut.height])
    }

    pub(super) fn draw_cut(&mut self, surface: Surface, sheet: &SpriteSheet, x: i32, y: i32, cut: i32) {
        if let Some(source) = Self::cut_of(sheet, cut) {
            self.blit(surface, source, x as f32, y as f32, source[2] as f32, source[3] as f32);
        }
    }

    pub(super) fn draw_cut_scaled(&mut self, surface: Surface, sheet: &SpriteSheet, dest: [i32; 4], cut: i32) {
        if let Some(source) = Self::cut_of(sheet, cut) {
            let [x, y, width, height] = dest.map(|cell| cell as f32);

            self.blit(surface, source, x, y, width, height);
        }
    }

    fn spun(spin: Spin) -> [[f32; 2]; 4] {
        let Spin { x, y, width, height, angle, align, pivot_x, pivot_y, pivot_align } = spin;
        let (x, y, wide, tall) = (x as f32, y as f32, width as f32, height as f32);
        let left = x - aligned(width, align, ALIGN_CENTER, ALIGN_RIGHT) as f32;
        let top = y - aligned(height, align, ALIGN_MIDDLE, ALIGN_BOTTOM) as f32;
        let (pivot_x, pivot_y) = if pivot_align & PIVOT_ABSOLUTE != 0 {
            (pivot_x as f32, pivot_y as f32)
        } else {
            (
                pivot_x as f32 + left + aligned(width, pivot_align, ALIGN_CENTER, ALIGN_RIGHT) as f32,
                pivot_y as f32 + top + aligned(height, pivot_align, ALIGN_MIDDLE, ALIGN_BOTTOM) as f32,
            )
        };
        let radians = (f64::from(angle) * -std::f64::consts::PI / 180.0) as f32;
        let (sin, cos) = radians.sin_cos();
        let near_x = left - pivot_x;
        let near_y = top - pivot_y;
        let far_x = near_x + wide;
        let far_y = near_y + tall;
        let turn = |dx: f32, dy: f32| [dx * cos + dy * sin + pivot_x, dy * cos - dx * sin + pivot_y];

        [turn(near_x, near_y), turn(near_x, far_y), turn(far_x, far_y), turn(far_x, near_y)]
    }

    pub(super) fn draw_cut_rotated(&mut self, surface: Surface, sheet: &SpriteSheet, spin: Spin, cut: i32) {
        let Some(source) = Self::cut_of(sheet, cut) else {
            return;
        };
        let corners = Self::spun(spin).map(|corner| [corner[0].trunc(), corner[1].trunc()]);

        self.push(Some(surface), corners, source, premultiplied(self.color));
    }

    pub(super) fn draw_cut_spun(&mut self, surface: Surface, sheet: &SpriteSheet, spin: Spin, cut: i32) {
        let (width, height) = Self::cut_of(sheet, cut).map_or((0, 0), |cell| (cell[2], cell[3]));

        self.draw_cut_rotated(surface, sheet, Spin { width, height, ..spin }, cut);
    }

    pub(super) fn draw_image_scaled(
        &mut self,
        surface: Surface,
        image: &RgbaImage,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
    ) {
        let source = [0, 0, image.width() as i32, image.height() as i32];

        self.blit(surface, source, x as f32, y as f32, width as f32, height as f32);
    }

    pub(super) fn draw_image_rotated(&mut self, surface: Surface, image: &RgbaImage, spin: Spin) {
        let corners = Self::spun(spin).map(|corner| [corner[0].trunc(), corner[1].trunc()]);
        let source = [0, 0, image.width() as i32, image.height() as i32];

        self.push(Some(surface), corners, source, premultiplied(self.color));
    }

    fn draw_frames(&mut self, surface: Surface, sheet: &SpriteSheet, frames: &[FrameData], x: i32, y: i32) {
        let (held_glow, held_color) = (self.glow, self.color);

        for frame in frames {
            let Some(cut) = sheet.cuts.get(frame.sprite_index) else {
                continue;
            };
            let [x0, y0, x1, y1, x2, y2, x3, y3] = frame.vertices;
            let (ox, oy) = (x as f32, y as f32);
            let corners = [[x0 + ox, y0 + oy], [x1 + ox, y1 + oy], [x3 + ox, y3 + oy], [x2 + ox, y2 + oy]];
            let alpha = frame.opacity;

            self.quads.push(Quad {
                surface: Some(surface),
                corners,
                source: [cut.x as f32, cut.y as f32, cut.width as f32, cut.height as f32],
                colors: [[alpha; 4]; 4],
                blend: frame.glow,
            });
        }

        self.glow = held_glow;
        self.color = held_color;
    }
}

fn argb_of(channels: [i32; 4]) -> Rgba {
    Rgba::new(channels[0], channels[1], channels[2], channels[3])
}

#[derive(Clone)]
struct Placed {
    rig: Rig,
    animation: Animation,
    surface: Surface,
}

fn placed<'a>(files: &'a [Option<Placed>], models: &'a [Placed], key: i32) -> Option<&'a Placed> {
    if key < 0 {
        files.get(usize::try_from(!key).ok()?)?.as_ref()
    } else {
        models.get(usize::try_from(key).ok()?)
    }
}

fn anim_length(files: &[Option<Placed>], models: &[Placed], key: i32) -> i32 {
    placed(files, models, key).map_or(0, |held| held.animation.last_frame())
}

/// A background composed frame by frame.
#[derive(Clone)]
pub struct Backdrop {
    entry: BackgroundEntry,
    sheet: SpriteSheet,
    bubble: Option<Arc<RgbaImage>>,
    sparkle: Option<SpriteSheet>,
    sheets: Vec<SpriteSheet>,
    defs: Vec<EffectDef>,
    files: Vec<Option<Placed>>,
    models: Vec<Placed>,
    instances: Vec<EffectInstance>,
    origin: (i32, i32),
    particles: Particles,
    world: World,
    rng: Rng,
    screen: Screen,
    frame: i32,
}

impl Backdrop {
    /// Composes a background from its loaded assets.
    ///
    /// The scene starts where a battle starts: the camera over the player's
    /// base at full zoom, the particles scattered and every effect instance
    /// spawned from the seed.
    ///
    /// # Arguments
    /// * `assets` - The background's row, sheets and effects.
    /// * `screen` - The screen the scene is composed for.
    /// * `stage_width` - The stage's declared width, in the units its layout file declares it in.
    /// * `seed` - The state the random number generator starts from, where zero is replaced by one.
    ///
    /// # Returns
    /// A `Backdrop` positioned at its first frame.
    pub fn new(assets: Assets, screen: Screen, stage_width: i32, seed: u32) -> Self {
        let Assets { entry, sheet, bubble, sparkle, defs, sheets, files, models } = assets;
        let place = |rig: EffectRig| -> Placed {
            let (surface, cuts) = match rig.sheet.and_then(|index| sheets.get(index).map(|held| (index, held))) {
                Some((index, held)) => (Surface::Effect(index), held.clone()),
                None => (Surface::Background, sheet.clone()),
            };

            Placed { rig: Rig { model: rig.model, sheet: cuts }, animation: rig.animation, surface }
        };
        let files = files.into_iter().map(|rig| rig.map(place)).collect();
        let models = models.into_iter().map(place).collect();
        let stage_length = stage_width.wrapping_mul(STAGE_SCALE);
        let world = World {
            id: entry.id as i32,
            stage_length,
            width: screen.width,
            height: screen.height,
            letterbox: screen.letterbox,
            camera_x: stage_length.wrapping_sub(SCREEN_SPAN),
            min_zoom: ZOOM_REACH.checked_div(stage_length).unwrap_or(0).wrapping_add(1),
            zoom_y: HORIZON,
        };

        let mut backdrop = Self {
            entry,
            sheet,
            bubble,
            sparkle,
            sheets,
            defs,
            files,
            models,
            instances: Vec::new(),
            origin: (0, 0),
            particles: Particles::default(),
            world,
            rng: Rng::new(seed),
            screen,
            frame: 0,
        };

        backdrop.restart(seed);
        backdrop
    }

    /// Returns the scene to its first frame with a fresh seed.
    ///
    /// # Arguments
    /// * `seed` - The state the random number generator starts from, where zero is replaced by one.
    pub fn restart(&mut self, seed: u32) {
        self.rng = Rng::new(seed);
        self.particles = Particles::default();
        self.frame = 0;
        self.particles.init(&self.world, &mut self.rng);
        self.spawn_all();
    }

    /// Returns the screen the scene is composed for.
    ///
    /// # Returns
    /// A `Screen` in design pixels.
    pub fn screen(&self) -> Screen {
        self.screen
    }

    /// Returns the background's row of the table.
    ///
    /// # Returns
    /// A reference to the `BackgroundEntry` the scene was composed from.
    pub fn entry(&self) -> &BackgroundEntry {
        &self.entry
    }

    /// Returns the frame the scene is at.
    ///
    /// # Returns
    /// An `i32` counting the frames advanced since the scene started.
    pub fn frame(&self) -> i32 {
        self.frame
    }

    /// Returns whether the scene changes from frame to frame.
    ///
    /// # Returns
    /// A `bool` that is true when the background scatters particles or spawns effects.
    pub fn animated(&self) -> bool {
        !self.instances.is_empty() || self.particles.animates(&self.world)
    }

    /// Returns the image behind a surface.
    ///
    /// # Arguments
    /// * `surface` - The surface a quad names.
    ///
    /// # Returns
    /// An `Option` containing the decoded image, or `None` when the assets hold none for the surface.
    pub fn image(&self, surface: Surface) -> Option<&Arc<RgbaImage>> {
        match surface {
            Surface::Background => self.sheet.image_data.as_ref(),
            Surface::Bubble => self.bubble.as_ref(),
            Surface::Sparkle => self.sparkle.as_ref().and_then(|sheet| sheet.image_data.as_ref()),
            Surface::Effect(index) => self.sheets.get(index).and_then(|sheet| sheet.image_data.as_ref()),
        }
    }

    /// Advances the scene by one frame.
    pub fn tick(&mut self) {
        self.particles.tick(&self.world, &mut self.rng);
        self.tick_effects();
        self.frame = self.frame.wrapping_add(1);
    }

    /// Composes the scene at its current frame.
    ///
    /// # Arguments
    /// * `camera` - The camera the frame is composed through.
    ///
    /// # Returns
    /// A `Vec<Quad>` in screen pixels and draw order, from the sky fill at the back to the fog at the front.
    pub fn render(&mut self, camera: Camera) -> Vec<Quad> {
        let layers = self.layers();

        self.compose(&layers, camera)
    }

    /// Draws the parts of the scene that move but do not depend on the view.
    ///
    /// The particles and effects are placed in the scene wherever they are,
    /// so a consumer that shows a moving view of a frame draws them once per
    /// frame and composes them onto each view with [`Backdrop::compose`].
    ///
    /// # Returns
    /// A `Layers` holding the quads behind and in front of the units in design
    /// pixels and the quads laid over the screen in screen pixels.
    pub fn layers(&mut self) -> Layers {
        let mut scene = Painter::new();

        self.particles.draw_behind(&self.world, self.surfaces(), &mut scene);
        self.draw_instances(BEHIND, &mut scene);
        self.particles.draw_in_front(&self.world, self.surfaces(), &mut scene);
        self.draw_instances(IN_FRONT, &mut scene);

        let mut over = Painter::new();

        self.draw_instances(OVER, &mut over);

        Layers { scene: scene.quads, over: over.quads }
    }

    /// Composes drawn layers through a camera.
    ///
    /// # Arguments
    /// * `layers` - The layers drawn at the frame being shown.
    /// * `camera` - The camera the frame is composed through.
    ///
    /// # Returns
    /// A `Vec<Quad>` in screen pixels and draw order, from the sky fill at the back to the fog at the front.
    pub fn compose(&self, layers: &Layers, camera: Camera) -> Vec<Quad> {
        let mut painter = Painter::new();

        self.draw_background(camera, &mut painter);
        painter.quads.extend(layers.scene.iter().map(|quad| camera.projected(quad)));
        self.draw_tints(camera, &mut painter);
        painter.quads.extend(layers.over.iter().cloned());
        self.draw_fog(camera, &mut painter);

        painter.quads
    }

    fn surfaces(&self) -> particles::Surfaces<'_> {
        particles::Surfaces { background: &self.sheet, bubble: self.bubble.as_deref(), sparkle: self.sparkle.as_ref() }
    }

    fn frame_of(&self) -> Frame {
        Frame {
            origin_x: self.origin.0,
            origin_y: self.origin.1,
            stage_length: self.world.stage_length,
            letterbox: self.world.letterbox,
            min_zoom: self.world.min_zoom,
            width: self.world.width,
            height: self.world.height,
        }
    }


    fn spawn_all(&mut self) {
        let length = self.world.stage_length;
        let visible = self.world.min_zoom.wrapping_mul(length) / 1000;
        let span = self.world.width.wrapping_sub(visible).wrapping_mul(length).checked_div(visible).unwrap_or(0);
        let half = span / 2;
        let left = (span / -20) as f64;
        let left = ((self.world.width.wrapping_add(-DESIGN_WIDTH) as f64) * 0.5 + left) as f32;
        let lift = (self.world.letterbox / 2).wrapping_mul(100).wrapping_add(0xc350);
        let lift = lift.checked_div(self.world.min_zoom).unwrap_or(0);
        let top = 0x1e0i32.wrapping_sub(lift) as f32;
        let centre = ((self.world.width.wrapping_add(-DESIGN_WIDTH) as f64) * 0.5) as f32;
        let scale = self.world.min_zoom.wrapping_mul(100) as f32 / 10000.0;
        let zero = 0.0f32 * scale;
        let back = ((DESIGN_WIDTH.wrapping_sub(self.world.width) as f64) * 0.5) as f32;
        let across = back * scale + zero * -520.0 + centre;
        let down = back * zero + -520.0 * scale + 520.0;
        let x = (left * scale + zero * top + across) * 1000.0 / self.world.min_zoom as f32 + half as f32;
        let y = (-(zero * left + scale * top + down)).floor();

        self.origin = (truncated(x), truncated(y));
        self.instances.clear();

        let frame = self.frame_of();
        let Self { rng, defs, files, models, instances, .. } = self;
        let mut groups: BTreeMap<i32, i32> = BTreeMap::new();

        for (index, def) in defs.iter().enumerate() {
            let mut count = effects::roll_int(rng, &frame, &def.count, &mut groups, 0, || anim_length(files, models, -1));

            while count > 0 {
                let mut instance = EffectInstance::spawned(index);

                instance.roll(rng, &frame, def, true, |key| anim_length(files, models, key));
                instances.push(instance);
                count = count.wrapping_sub(1);
            }
        }
    }

    fn tick_effects(&mut self) {
        let frame = self.frame_of();
        let Self { rng, defs, files, models, instances, .. } = self;

        for instance in instances.iter_mut() {
            if instance.wait > 0 {
                instance.wait = instance.wait.wrapping_sub(1);
                continue;
            }

            let (v, vx, vy, heading) = (instance.v, instance.vx, instance.vy, instance.move_angle);
            let step_x = v * cos_deg(heading) + vx;
            let step_y = v * sin_deg(heading) + vy;

            instance.x += step_x;
            instance.y += step_y;
            instance.angle += instance.angular_v;

            let aged = instance.frame;

            instance.frame = aged.wrapping_add(1);

            let Some(def) = defs.get(instance.def_index) else {
                continue;
            };

            if instance.expired(def, aged) {
                instance.roll(rng, &frame, def, false, |key| anim_length(files, models, key));
            }
        }
    }

    fn draw_instances(&mut self, layer: i32, painter: &mut Painter) {
        let frame = self.frame_of();
        let camera = self.world.camera_x / 10;
        let width = self.world.width;
        let Self { defs, files, models, instances, .. } = self;

        for instance in instances.iter() {
            if instance.z != layer || instance.wait > 0 {
                continue;
            }

            let Some(def) = defs.get(instance.def_index) else {
                continue;
            };
            let spaced = def.equally_spaced;
            let spacing = frame.resolve_int(spaced.value, spaced.base, || anim_length(files, models, -1));
            let key = instance.model;
            let Some(held) = (if key < 0 {
                usize::try_from(!key).ok().and_then(|index| files.get_mut(index)).and_then(Option::as_mut)
            } else {
                usize::try_from(key).ok().and_then(|index| models.get_mut(index))
            }) else {
                continue;
            };
            let (scale_unit, angle_unit, opacity_unit) =
                (held.rig.model.scale_unit, held.rig.model.angle_unit, held.rig.model.opacity_unit);
            let Some(root) = held.rig.model.parts.first_mut() else {
                continue;
            };
            let rest = (root.scale_x, root.scale_y, root.angle, root.opacity);
            let (width_scale, height_scale) = if layer == OVER {
                (instance.scale * scale_unit as f32, scale_unit as f32 * instance.scale)
            } else {
                (
                    instance.scale * scale_unit as f32 * instance.scale_x,
                    scale_unit as f32 * instance.scale * instance.scale_y,
                )
            };

            root.scale_x = truncated(width_scale);
            root.scale_y = truncated(height_scale);
            root.angle = truncated(angle_unit as f32 * instance.angle / 360.0);
            root.opacity = truncated(opacity_unit as f32 * instance.alpha / 255.0);

            let frames = resolve_frame(&held.rig, Some(&held.animation), instance.frame, None);

            if spaced.pos1 <= spaced.pos2 {
                let mut step = spaced.pos1;
                let end = spaced.pos2.wrapping_add(1);

                loop {
                    let x = if layer == OVER {
                        truncated(instance.x + spacing.wrapping_mul(step) as f32)
                    } else {
                        let placed = instance.x + spacing.wrapping_mul(step) as f32 - camera as f32;

                        truncated((width.wrapping_add(-DESIGN_WIDTH) / 2) as f32 + placed)
                    };
                    let y = truncated(instance.y);

                    painter.draw_frames(held.surface, &held.rig.sheet, &frames, x, y);
                    step = step.wrapping_add(1);

                    if step == end {
                        break;
                    }
                }
            }

            if let Some(root) = held.rig.model.parts.first_mut() {
                (root.scale_x, root.scale_y, root.angle, root.opacity) = rest;
            }
        }
    }

    fn vertical_correction(&self) -> i32 {
        let zoom = FULL_ZOOM;
        let anchor = self.world.zoom_y.wrapping_sub(self.world.letterbox);
        let scaled = HORIZON.wrapping_mul(zoom);
        let limit = DECK_BAR_TOP.wrapping_add(self.world.letterbox);
        let slack = (100.0f64 - (zoom as f64) / 100.0f64) * (anchor as f64) / 100.0f64;
        let bottom = ((scaled as i64 / 10000) as i32 as f64 + slack) as i32;

        if bottom > limit {
            return 0i32.wrapping_sub(bottom.wrapping_sub(limit).wrapping_mul(0x2710).checked_div(zoom).unwrap_or(0));
        }

        0
    }

    fn draw_background(&self, camera: Camera, painter: &mut Painter) {
        let horizon = camera.horizon() + self.vertical_correction() as f32 * camera.zoom;
        let (width, height) = (camera.width, camera.height);

        if horizon > 0.0 {
            painter.fill_box(0.0, 0.0, width, horizon.min(height), [self.entry.sky_bottom(); 4]);
        }

        if horizon < height {
            painter.fill_box(0.0, horizon.max(0.0), width, height, [self.entry.ground_top(); 4]);
        }

        let scenery = Painter::cut_of(&self.sheet, GROUND_CUT).map_or(0, |cut| cut[3].wrapping_mul(DESIGN_HEIGHT) / TILE_SOURCE_HEIGHT);
        let base = if self.entry.has_upper_layer { scenery } else { scenery.wrapping_add(-DESIGN_HEIGHT) };
        let faded = camera.project([0.0, 0i32.wrapping_sub(base.wrapping_sub(self.vertical_correction())) as f32])[1];
        let sky_top = (SKY_LIFT as f32 + camera.y).min(0.0);

        if sky_top < faded {
            let clear = self.entry.sky_bottom().with_alpha(0);

            painter.fill_box(0.0, sky_top, width, faded, [self.entry.sky_top(), clear, clear, self.entry.sky_top()]);
        }

        let ground_top = camera.project([0.0, (self.vertical_correction().wrapping_add(DESIGN_HEIGHT)) as f32])[1];
        let ground_bottom = (GROUND_DROP as f32 + camera.y).max(height).max(ground_top);
        let clear = self.entry.ground_top().with_alpha(0);

        painter.fill_box(0.0, ground_top, width, ground_bottom, [clear, self.entry.ground_bottom(), self.entry.ground_bottom(), clear]);

        let scrolled = self.world.camera_x / 10;
        let tiles = (scrolled / DESIGN_WIDTH).wrapping_mul(DESIGN_WIDTH);
        let origin = tiles.wrapping_sub(scrolled).wrapping_add(self.world.width.wrapping_add(-DESIGN_WIDTH) / 2);
        let floor = DESIGN_HEIGHT.wrapping_sub(scenery);
        let first = ((camera.scene_left() - origin as f32) / DESIGN_WIDTH as f32).floor() as i32;
        let last = ((camera.scene_right() - origin as f32) / DESIGN_WIDTH as f32).ceil() as i32;
        let mut scene = Painter::new();

        for tile in first..=last {
            let x = origin.wrapping_add(tile.wrapping_mul(DESIGN_WIDTH));

            scene.draw_cut_scaled(Surface::Background, &self.sheet, [x, floor, DESIGN_WIDTH, scenery], GROUND_CUT);

            if self.entry.has_upper_layer {
                let dest = [x, -DESIGN_HEIGHT, DESIGN_WIDTH, DESIGN_HEIGHT];

                scene.draw_cut_scaled(Surface::Background, &self.sheet, dest, UPPER_CUT);
            }
        }

        painter.quads.extend(scene.quads.iter().map(|quad| camera.projected(quad)));
    }

    fn draw_tints(&self, camera: Camera, painter: &mut Painter) {
        let (width, height) = (camera.width, camera.height);

        if let Some((_, upper, lower)) = TINTED.iter().find(|(id, _, _)| *id == self.world.id) {
            painter.fill_box(0.0, 0.0, width, height, [argb(*upper), argb(*lower), argb(*lower), argb(*upper)]);
        } else if self.world.is(&WHITENED) {
            let (upper, lower) = (argb(0x4cffffff), argb(0x33ffffff));

            painter.fill_box(0.0, 0.0, width, height, [upper, lower, lower, upper]);
        } else if self.world.id == WASHED {
            painter.fill_box(0.0, 0.0, width, height, [Rgba::new(0xff, 0xff, 0xff, WASH_ALPHA); 4]);
        }
    }

    fn draw_fog(&self, camera: Camera, painter: &mut Painter) {
        if let (Some(upper), Some(lower)) = (self.entry.fog_top(), self.entry.fog_bottom())
            && (upper.alpha & 0xff != 0 || lower.alpha & 0xff != 0)
        {
            painter.fill_box(0.0, 0.0, camera.width, camera.height, [upper, lower, lower, upper]);
        }
    }
}
