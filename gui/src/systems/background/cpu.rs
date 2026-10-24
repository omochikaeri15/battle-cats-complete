use std::sync::Arc;

use iced::Rectangle;

use crate::widget::cpu::{Alpha, Layer, Pixels, Plan, Rasterize, Sheet, Vertex};

use super::canvas::Painted;
use super::pipeline::{self, Scene};

const CLEARED: [u8; 4] = [0; 4];
const UNTINTED: [f32; 4] = [0.0; 4];
pub(super) const MAX_DIMENSION: f32 = 16384.0;

fn plan(scene: &Scene) -> Plan {
    let vertices = scene.vertices.iter().map(|vertex| Vertex { position: vertex.position, uv: vertex.uv, color: vertex.color, tint: UNTINTED }).collect();
    let layers = scene
        .runs
        .iter()
        .map(|run| Layer {
            sheet: run.texture.and_then(|index| scene.textures.get(index)).map(|image| Sheet {
                width: image.width(),
                height: image.height(),
                pixels: Pixels::Image(Arc::clone(image)),
            }),
            blend: run.blend,
            alpha: Alpha::Blended,
            range: run.range.start as usize..run.range.end as usize,
        })
        .collect();

    Plan { vertices, layers }
}

impl Rasterize for Painted {
    fn plan(&self, bounds: Rectangle) -> Option<Plan> {
        if self.quads.is_empty() {
            return None;
        }

        let images = &self.images;
        let scene = pipeline::build(
            &self.quads,
            |surface| images.iter().find(|(held, _)| *held == surface).map(|(_, image)| Arc::clone(image)),
            |corner| [corner[0] / bounds.width * 2.0 - 1.0, 1.0 - corner[1] / bounds.height * 2.0],
        );

        Some(plan(&scene))
    }
}

pub(super) fn render(scene: &Scene, width: u32, height: u32) -> Vec<u8> {
    let drawn = plan(scene).render(width, height, CLEARED);

    drawn.chunks_exact(width as usize * 4).rev().flatten().copied().collect()
}
