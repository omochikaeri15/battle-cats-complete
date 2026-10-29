use std::sync::Arc;

use iced::Rectangle;
use image::RgbaImage;

use kokoro::systems::animation::multiply_mat3;

use crate::widget::cpu::{Alpha, Layer, Pixels, Plan, Rasterize, Sheet, Vertex};

use super::canvas::Scene;
use super::pipeline::{build_vertices, Painted};

const FLOATS_PER_VERTEX: usize = 9;
pub(super) const MAX_DIMENSION: u32 = 16384;

fn plan(image: &Arc<RgbaImage>, parts: &[Painted], view_proj: &[f32; 9]) -> Plan {
    let (vertex_data, index_data, batches) = build_vertices(parts, view_proj);
    let vertices = index_data
        .iter()
        .filter_map(|index| {
            let start = *index as usize * FLOATS_PER_VERTEX;
            let floats = vertex_data.get(start..start + FLOATS_PER_VERTEX)?;

            Some(Vertex {
                position: [floats[0], floats[1]],
                uv: [floats[2], floats[3]],
                color: [floats[4]; 4],
                tint: [floats[5], floats[6], floats[7], floats[8]],
            })
        })
        .collect();
    let sheet = Sheet { width: image.width(), height: image.height(), pixels: Pixels::Image(Arc::clone(image)) };
    let layers = batches
        .iter()
        .map(|batch| Layer { sheet: Some(sheet.clone()), blend: batch.variant as u8, alpha: Alpha::Carried, range: batch.range.start as usize..batch.range.end as usize })
        .collect();

    Plan { vertices, layers }
}

impl Rasterize for Scene {
    fn plan(&self, bounds: Rectangle) -> Option<Plan> {
        let image = self.image.as_ref().filter(|_| !self.parts.is_empty())?;
        let projection = [
            2.0 / bounds.width, 0.0, 0.0,
            0.0, -2.0 / bounds.height, 0.0,
            -1.0, 1.0, 1.0,
        ];
        let camera = [
            self.zoom, 0.0, 0.0,
            0.0, self.zoom, 0.0,
            bounds.width / 2.0 + self.pan.x * self.zoom,
            bounds.height / 2.0 + self.pan.y * self.zoom,
            1.0,
        ];

        Some(plan(image, &self.parts, &multiply_mat3(&projection, &camera)))
    }
}

pub(super) fn render_parts(image: Option<&Arc<RgbaImage>>, parts: &[Painted], view_proj: &[f32; 9], background: [u8; 4], width: u32, height: u32) -> Vec<u8> {
    let drawn = image.filter(|_| !parts.is_empty()).map_or_else(
        || background.repeat(width as usize * height as usize),
        |image| plan(image, parts, view_proj).render(width, height, background),
    );

    drawn.chunks_exact(width as usize * 4).rev().flatten().copied().collect()
}
