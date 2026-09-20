use crate::engine::{DrawSink, Imgcut, Mamodel, Surface};

pub struct InertDraw {
    color: [i32; 4],
    tint: [i32; 4],
}

impl Default for InertDraw {
    fn default() -> Self {
        Self {
            color: [0xff; 4],
            tint: [0xff; 4],
        }
    }
}

impl DrawSink for InertDraw {
    fn set_origin(&mut self, _x: i32, _y: i32) {}

    fn glow_set(&mut self, _mode: i32) {}

    fn set_draw_scale(&mut self, _scale: f32) {}

    fn set_tint(&mut self, red: i32, green: i32, blue: i32, alpha: i32) {
        self.tint = [red, green, blue, alpha];
    }

    fn set_color(&mut self, red: i32, green: i32, blue: i32, alpha: i32) {
        self.color = [red, green, blue, alpha];
    }

    fn set_transform(&mut self, _angle: f32, _matrix: &[f32; 6]) {}

    fn set_tint_alpha(&mut self, alpha: i32) {
        self.tint[3] = alpha;
    }

    fn color(&self) -> [i32; 4] {
        self.color
    }

    fn tint(&self) -> [i32; 4] {
        self.tint
    }

    fn fill_rect(&mut self, _x: i32, _y: i32, _width: i32, _height: i32) {}

    fn fill_rect_f(&mut self, _x: f32, _y: f32, _width: f32, _height: f32) {}

    fn fill_polygon(&mut self, _xs: &[i32], _ys: &[i32], _count: i32) {}

    fn fill_polygon_colored(
        &mut self,
        _xs: &[i32],
        _ys: &[i32],
        _colors: &[u32],
        _count: i32,
    ) {
    }

    fn draw_surface_aligned(&mut self, _surface: Surface<'_>, _x: i32, _y: i32, _align: i32) {}

    fn set_alpha(&mut self, _alpha: i32) {}

    fn set_flip(&mut self, _flip: i32) {}

    fn draw_surface(&mut self, _surface: Surface<'_>, _x: i32, _y: i32) {}

    fn draw_cut(&mut self, _sheet: &Imgcut, _x: i32, _y: i32, _cut: i32) {}

    fn draw_cut_scaled(
        &mut self,
        _sheet: &Imgcut,
        _x: i32,
        _y: i32,
        _width: i32,
        _height: i32,
        _cut: i32,
    ) {
    }

    fn draw_cut_f(
        &mut self,
        _sheet: &Imgcut,
        _cut: i32,
        _x: f32,
        _y: f32,
        _width: f32,
        _height: f32,
    ) {
    }

    fn draw_region(
        &mut self,
        _sheet: &Imgcut,
        _x: i32,
        _y: i32,
        _src_x: i32,
        _src_y: i32,
        _src_w: i32,
        _src_h: i32,
    ) {
    }

    fn draw_region_f(
        &mut self,
        _sheet: &Imgcut,
        _src_x: i32,
        _src_y: i32,
        _src_w: i32,
        _src_h: i32,
        _x: f32,
        _y: f32,
        _width: f32,
        _height: f32,
    ) {
    }

    fn draw_surface_scaled(
        &mut self,
        _surface: Surface<'_>,
        _x: i32,
        _y: i32,
        _width: i32,
        _height: i32,
    ) {
    }

    fn draw_model(&mut self, _model: &Mamodel, _x: i32, _y: i32) {}

    fn draw_model_scaled(
        &mut self,
        _model: &Mamodel,
        _x: i32,
        _y: i32,
        _pivot_x: i32,
        _pivot_y: i32,
        _scale: f32,
        _alpha: i32,
        _first: i32,
        _second: i32,
    ) {
    }

    fn draw_cut_rotated(
        &mut self,
        _sheet: &Imgcut,
        _x: i32,
        _y: i32,
        _width: i32,
        _height: i32,
        _angle: f32,
        _align: i32,
        _pivot_x: i32,
        _pivot_y: i32,
        _pivot_align: i32,
        _cut: i32,
    ) {
    }

    fn draw_cut_rotated_f(
        &mut self,
        _sheet: &Imgcut,
        _x: f32,
        _y: f32,
        _width: f32,
        _height: f32,
        _pivot_x: f32,
        _pivot_y: f32,
        _angle: f32,
        _align: i32,
        _pivot_align: i32,
        _cut: i32,
    ) {
    }

    fn draw_cut_spun(
        &mut self,
        _sheet: &Imgcut,
        _x: i32,
        _y: i32,
        _angle: f32,
        _align: i32,
        _pivot_x: i32,
        _pivot_y: i32,
        _pivot_align: i32,
        _cut: i32,
    ) {
    }

    fn draw_image_rotated(
        &mut self,
        _sheet: &Imgcut,
        _x: i32,
        _y: i32,
        _width: i32,
        _height: i32,
        _angle: f32,
        _align: i32,
        _pivot_x: i32,
        _pivot_y: i32,
        _pivot_align: i32,
    ) {
    }

    fn draw_panel(
        &mut self,
        _sheet: &Imgcut,
        _x: i32,
        _y: i32,
        _width: i32,
        _height: i32,
        _scale: f32,
        _cut_a: i32,
        _cut_b: i32,
    ) {
    }

    fn draw_nine_slice(
        &mut self,
        _sheet: &Imgcut,
        _x: i32,
        _y: i32,
        _width: i32,
        _height: i32,
        _scale: f32,
        _cut: i32,
        _border_x: i32,
        _border_y: i32,
        _inner_w: i32,
        _inner_h: i32,
    ) {
    }

    fn draw_quad_cut(
        &mut self,
        _sheet: &Imgcut,
        _x0: i32,
        _y0: i32,
        _x1: i32,
        _y1: i32,
        _x2: i32,
        _y2: i32,
        _x3: i32,
        _y3: i32,
        _cut: i32,
    ) {
    }

    fn draw_quad_region(
        &mut self,
        _sheet: &Imgcut,
        _x0: i32,
        _y0: i32,
        _x1: i32,
        _y1: i32,
        _x2: i32,
        _y2: i32,
        _x3: i32,
        _y3: i32,
        _src_x: i32,
        _src_y: i32,
        _src_w: i32,
        _src_h: i32,
    ) {
    }

    fn draw_sprite_cut(
        &mut self,
        _sheet: &Imgcut,
        _x0: i32,
        _y0: i32,
        _x1: i32,
        _y1: i32,
        _x2: i32,
        _y2: i32,
        _x3: i32,
        _y3: i32,
        _cut: i32,
    ) {
    }
}
