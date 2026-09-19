use crate::Fault;

use super::{Imgcut, Mamodel, Texture};

#[derive(Clone, Copy)]
pub enum Surface<'a> {
    Sheet(&'a Imgcut),
    Label(&'a Texture),
}

pub trait DrawSink {
    fn set_origin(&mut self, x: i32, y: i32);
    fn glow_set(&mut self, mode: i32);
    fn set_draw_scale(&mut self, scale: f32);
    fn set_tint(&mut self, red: i32, green: i32, blue: i32, alpha: i32);
    fn set_color(&mut self, red: i32, green: i32, blue: i32, alpha: i32);
    fn set_transform(&mut self, angle: f32, matrix: &[f32; 6]);
    fn set_tint_alpha(&mut self, alpha: i32);
    fn fill_rect(&mut self, x: i32, y: i32, width: i32, height: i32);
    fn fill_rect_f(&mut self, x: f32, y: f32, width: f32, height: f32);
    fn fill_polygon(&mut self, xs: &[i32], ys: &[i32], count: i32);
    fn fill_polygon_colored(&mut self, xs: &[i32], ys: &[i32], colors: &[u32], count: i32);
    fn draw_surface_aligned(&mut self, surface: Surface<'_>, x: i32, y: i32, align: i32);
    fn set_alpha(&mut self, alpha: i32);
    fn set_flip(&mut self, flip: i32);
    fn draw_surface(&mut self, surface: Surface<'_>, x: i32, y: i32);
    fn draw_cut(&mut self, sheet: &Imgcut, x: i32, y: i32, cut: i32);
    fn draw_cut_scaled(&mut self, sheet: &Imgcut, x: i32, y: i32, width: i32, height: i32, cut: i32);
    fn draw_cut_f(&mut self, sheet: &Imgcut, cut: i32, x: f32, y: f32, width: f32, height: f32);
    #[allow(clippy::too_many_arguments)]
    fn draw_region_f(&mut self, sheet: &Imgcut, src_x: i32, src_y: i32, src_w: i32, src_h: i32, x: f32, y: f32, width: f32, height: f32);
    fn draw_surface_scaled(&mut self, surface: Surface<'_>, x: i32, y: i32, width: i32, height: i32);
    #[allow(clippy::too_many_arguments)]
    fn draw_model(&mut self, model: &Mamodel, x: i32, y: i32);
    #[allow(clippy::too_many_arguments)]
    fn draw_model_scaled(&mut self, model: &Mamodel, x: i32, y: i32, pivot_x: i32, pivot_y: i32, scale: f32, alpha: i32, first: i32, second: i32);
    #[allow(clippy::too_many_arguments)]
    fn draw_cut_rotated(&mut self, sheet: &Imgcut, x: i32, y: i32, width: i32, height: i32, angle: f32, align: i32, pivot_x: i32, pivot_y: i32, pivot_align: i32, cut: i32);
    #[allow(clippy::too_many_arguments)]
    fn draw_cut_rotated_f(&mut self, sheet: &Imgcut, x: f32, y: f32, width: f32, height: f32, pivot_x: f32, pivot_y: f32, angle: f32, align: i32, pivot_align: i32, cut: i32);
    #[allow(clippy::too_many_arguments)]
    fn draw_cut_spun(&mut self, sheet: &Imgcut, x: i32, y: i32, angle: f32, align: i32, pivot_x: i32, pivot_y: i32, pivot_align: i32, cut: i32);
    #[allow(clippy::too_many_arguments)]
    fn draw_image_rotated(&mut self, sheet: &Imgcut, x: i32, y: i32, width: i32, height: i32, angle: f32, align: i32, pivot_x: i32, pivot_y: i32, pivot_align: i32);
    #[allow(clippy::too_many_arguments)]
    fn draw_quad_cut(&mut self, sheet: &Imgcut, x0: i32, y0: i32, x1: i32, y1: i32, x2: i32, y2: i32, x3: i32, y3: i32, cut: i32);
    #[allow(clippy::too_many_arguments)]
    fn draw_quad_region(&mut self, sheet: &Imgcut, x0: i32, y0: i32, x1: i32, y1: i32, x2: i32, y2: i32, x3: i32, y3: i32, src_x: i32, src_y: i32, src_w: i32, src_h: i32);
    #[allow(clippy::too_many_arguments)]
    fn draw_sprite_cut(&mut self, sheet: &Imgcut, x0: i32, y0: i32, x1: i32, y1: i32, x2: i32, y2: i32, x3: i32, y3: i32, cut: i32);
}

pub fn draw_context(sink: &mut Option<Box<dyn DrawSink>>) -> Result<&mut (dyn DrawSink + 'static), Fault> {
    sink.as_deref_mut().ok_or(Fault::HostMissing { site: "draw_context" })
}
