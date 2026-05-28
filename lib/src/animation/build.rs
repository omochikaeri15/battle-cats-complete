use glow;
pub use super::graphics::canvas::{GlowRenderer, CanvasError};

// We import these internally for the Rig to use
use crate::animation::data::mamodel::Model;
use crate::common::data::imgcut::SpriteSheet;
use crate::animation::graphics::{timeline, transform};

// We explicitly EXPORT Animation so the outside developer can parse and store it!
pub use crate::animation::data::maanim::Animation;

/// A static, opaque container holding the parsed skeleton and textures.
pub struct Rig {
    // pub(crate) means Nyanko can see these, but the outside world cannot!
    pub(crate) model: Model,
    pub(crate) sheet: SpriteSheet,
}

impl Rig {
    /// Ingests anything that can be viewed as bytes from the host and parses them into a mathematical rig.
    #[inline(always)]
    pub fn parse(
        png: impl AsRef<[u8]>,
        imgcut: impl AsRef<[u8]>,
        mamodel: impl AsRef<[u8]>,
    ) -> Option<Self> {
        Self::parse_inner(png.as_ref(), imgcut.as_ref(), mamodel.as_ref())
    }

    fn parse_inner(png: &[u8], imgcut: &[u8], mamodel: &[u8]) -> Option<Self> {
        let sheet = SpriteSheet::parse(png, imgcut)?;
        let model = Model::parse(mamodel)?;

        Some(Self { model, sheet })
    }

    /// Safely lets the developer check if the rig loaded successfully without exposing the raw data
    pub fn is_empty(&self) -> bool {
        self.model.parts.is_empty()
    }
}

pub fn frame(
    renderer: &mut GlowRenderer,
    gl_context: &glow::Context,
    rig: &Rig,
    anim: &Animation,
    current_frame: f32,
    viewport_width: f32,
    viewport_height: f32,
    pan_x: f32,
    pan_y: f32,
    zoom: f32,
) -> Result<(), CanvasError> {
    // Nyanko can safely access rig.model because of pub(crate)!
    let mut state_buffer = rig.model.parts.clone();
    let _ = timeline::animate(&rig.model, anim, current_frame, &mut state_buffer);
    let world_parts = transform::solve_hierarchy(&state_buffer, &rig.model);

    renderer.paint(
        gl_context,
        viewport_width,
        viewport_height,
        &world_parts,
        &rig.sheet,
        pan_x,
        pan_y,
        zoom
    )
}