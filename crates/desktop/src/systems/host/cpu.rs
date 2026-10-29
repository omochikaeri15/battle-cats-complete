use std::sync::atomic::Ordering;

use iced::Rectangle;

use crate::widget::cpu::{Alpha, Layer, Pixels, Plan, Rasterize, Sheet, Vertex};

use super::viewport::Scene;

const CLEARED: [u8; 4] = [0, 0, 0, 0xff];
const UNTINTED: [f32; 4] = [0.0; 4];

fn plan(scene: &Scene) -> Plan {
    let vertices = scene.vertices.iter().map(|vertex| Vertex { position: vertex.position, uv: vertex.uv, color: vertex.color, tint: UNTINTED }).collect();
    let layers = scene
        .runs
        .iter()
        .map(|(name, blend, start, end)| Layer {
            sheet: name.as_deref().and_then(|name| scene.uploads.iter().find(|(held, _)| held.as_ref() == name)).map(|(_, sheet)| Sheet {
                width: sheet.width,
                height: sheet.height,
                pixels: Pixels::Shared(sheet.pixels.clone()),
            }),
            blend: *blend,
            alpha: Alpha::Carried,
            range: *start as usize..*end as usize,
        })
        .collect();

    Plan { vertices, layers }
}

impl Rasterize for Scene {
    fn plan(&self, _bounds: Rectangle) -> Option<Plan> {
        if let Some(presented) = &self.presented {
            presented.shown.store(true, Ordering::Release);
        }

        Some(plan(self))
    }

    fn scale(&self) -> f32 {
        self.scale
    }

    fn padded(&self) -> bool {
        self.padded
    }
}

pub(super) fn draw(scene: &Scene, width: u32, height: u32, pixels: &mut Vec<u8>) {
    *pixels = plan(scene).render(width, height, CLEARED);
}
