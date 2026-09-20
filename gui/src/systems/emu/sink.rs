use std::cell::RefCell;
use std::rc::Rc;

use emu::engine::{DrawSink, Imgcut, Mamodel, Surface};

const PART_PARENT: usize = 0x1c;
const PART_CUT: usize = 0x24;
const PART_CORNERS: [usize; 4] = [0x90, 0x98, 0xa0, 0xa8];

#[derive(Clone, PartialEq, Debug)]
pub struct Quad {
    pub sheet: Option<Box<str>>,
    pub corners: [[f32; 2]; 4],
    pub source: [f32; 4],
    pub color: [f32; 4],
}

#[derive(Clone, Copy)]
struct State {
    origin: [i32; 2],
    scale: f32,
    color: [i32; 4],
    tint: [i32; 4],
    alpha: i32,
    flip: i32,
    glow: i32,
    matrix: [f32; 6],
}

impl Default for State {
    fn default() -> Self {
        Self {
            origin: [0, 0],
            scale: 1.0,
            color: [0xff; 4],
            tint: [0xff; 4],
            alpha: 0xff,
            flip: 0,
            glow: 0,
            matrix: [1.0, 0.0, 0.0, 1.0, 0.0, 0.0],
        }
    }
}

#[derive(Default)]
pub struct Frame {
    pub quads: Vec<Quad>,
    pub unsupported: Vec<&'static str>,
    pub labels: usize,
}

impl Frame {
    pub fn clear(&mut self) {
        self.quads.clear();
        self.unsupported.clear();
        self.labels = 0;
    }
}

pub struct Recorder {
    frame: Rc<RefCell<Frame>>,
    state: State,
}

impl Recorder {
    pub fn new(frame: Rc<RefCell<Frame>>) -> Self {
        Self {
            frame,
            state: State::default(),
        }
    }

    fn note(&mut self, what: &'static str) {
        let mut frame = self.frame.borrow_mut();

        if !frame.unsupported.contains(&what) {
            frame.unsupported.push(what);
        }
    }

    fn place(&self, x: f32, y: f32) -> [f32; 2] {
        let matrix = self.state.matrix;
        let local_x = x;
        let local_y = y;

        [
            matrix[0] * local_x + matrix[2] * local_y + matrix[4] + self.state.origin[0] as f32,
            matrix[1] * local_x + matrix[3] * local_y + matrix[5] + self.state.origin[1] as f32,
        ]
    }

    fn paint(&self) -> [f32; 4] {
        let color = self.state.color;
        let tint = self.state.tint;
        let alpha = self.state.alpha as f32 / 255.0;

        [
            (color[0] * tint[0]) as f32 / 65025.0,
            (color[1] * tint[1]) as f32 / 65025.0,
            (color[2] * tint[2]) as f32 / 65025.0,
            (color[3] * tint[3]) as f32 / 65025.0 * alpha,
        ]
    }

    fn box_corners(&self, x: f32, y: f32, width: f32, height: f32) -> [[f32; 2]; 4] {
        [
            self.place(x, y),
            self.place(x, y + height),
            self.place(x + width, y + height),
            self.place(x + width, y),
        ]
    }

    fn push_solid(&mut self, x: f32, y: f32, width: f32, height: f32) {
        let quad = Quad {
            sheet: None,
            corners: self.box_corners(x, y, width, height),
            source: [0.0; 4],
            color: self.paint(),
        };

        self.frame.borrow_mut().quads.push(quad);
    }

    fn push_cut(&mut self, sheet: &Imgcut, cut: i32, corners: [[f32; 2]; 4]) {
        let Some(cell) = sheet.cuts.get(cut as i64 as usize) else {
            return;
        };
        let name = String::from_utf8_lossy(&sheet.png).into_owned();
        let quad = Quad {
            sheet: Some(Box::from(name.as_str())),
            corners,
            source: [
                cell[0] as f32,
                cell[1] as f32,
                cell[2] as f32,
                cell[3] as f32,
            ],
            color: self.paint(),
        };

        self.frame.borrow_mut().quads.push(quad);
    }

    fn push_cut_box(&mut self, sheet: &Imgcut, cut: i32, x: f32, y: f32, width: f32, height: f32) {
        let corners = self.box_corners(x, y, width, height);

        self.push_cut(sheet, cut, corners);
    }

    fn push_cut_native(&mut self, sheet: &Imgcut, cut: i32, x: f32, y: f32) {
        let Some(cell) = sheet.cuts.get(cut as i64 as usize) else {
            return;
        };
        let width = cell[2] as f32;
        let height = cell[3] as f32;

        self.push_cut_box(sheet, cut, x, y, width, height);
    }
}

impl DrawSink for Recorder {
    fn set_origin(&mut self, x: i32, y: i32) {
        self.state.origin = [x, y];
    }

    fn glow_set(&mut self, mode: i32) {
        self.state.glow = mode;
    }

    fn set_draw_scale(&mut self, scale: f32) {
        self.state.scale = scale;
    }

    fn set_tint(&mut self, red: i32, green: i32, blue: i32, alpha: i32) {
        self.state.tint = [red, green, blue, alpha];
    }

    fn set_color(&mut self, red: i32, green: i32, blue: i32, alpha: i32) {
        self.state.color = [red, green, blue, alpha];
    }

    fn set_transform(&mut self, _angle: f32, matrix: &[f32; 6]) {
        self.state.matrix = *matrix;
    }

    fn set_tint_alpha(&mut self, alpha: i32) {
        self.state.tint[3] = alpha;
    }

    fn color(&self) -> [i32; 4] {
        self.state.color
    }

    fn tint(&self) -> [i32; 4] {
        self.state.tint
    }

    fn fill_rect(&mut self, x: i32, y: i32, width: i32, height: i32) {
        self.push_solid(x as f32, y as f32, width as f32, height as f32);
    }

    fn fill_rect_f(&mut self, x: f32, y: f32, width: f32, height: f32) {
        self.push_solid(x, y, width, height);
    }

    fn fill_polygon(&mut self, xs: &[i32], ys: &[i32], count: i32) {
        let count = count as i64 as usize;

        for corner in 1..count.saturating_sub(1) {
            let (Some(ax), Some(ay)) = (xs.first(), ys.first()) else {
                return;
            };
            let (Some(bx), Some(by)) = (xs.get(corner), ys.get(corner)) else {
                return;
            };
            let (Some(cx), Some(cy)) = (xs.get(corner + 1), ys.get(corner + 1)) else {
                return;
            };
            let a = self.place(*ax as f32, *ay as f32);
            let b = self.place(*bx as f32, *by as f32);
            let c = self.place(*cx as f32, *cy as f32);
            let quad = Quad {
                sheet: None,
                corners: [a, b, c, c],
                source: [0.0; 4],
                color: self.paint(),
            };

            self.frame.borrow_mut().quads.push(quad);
        }
    }

    fn fill_polygon_colored(&mut self, xs: &[i32], ys: &[i32], _colors: &[u32], count: i32) {
        self.note("fill_polygon_colored uses the flat colour");
        self.fill_polygon(xs, ys, count);
    }

    fn draw_surface_aligned(&mut self, surface: Surface<'_>, x: i32, y: i32, _align: i32) {
        self.note("draw_surface_aligned ignores alignment");
        self.draw_surface(surface, x, y);
    }

    fn set_alpha(&mut self, alpha: i32) {
        self.state.alpha = alpha;
    }

    fn set_flip(&mut self, flip: i32) {
        self.state.flip = flip;
    }

    fn draw_surface(&mut self, surface: Surface<'_>, x: i32, y: i32) {
        match surface {
            Surface::Sheet(sheet) => {
                let width = sheet.width as f32;
                let height = sheet.height as f32;
                let corners = self.box_corners(x as f32, y as f32, width, height);
                let name = String::from_utf8_lossy(&sheet.png).into_owned();
                let quad = Quad {
                    sheet: Some(Box::from(name.as_str())),
                    corners,
                    source: [0.0, 0.0, width, height],
                    color: self.paint(),
                };

                self.frame.borrow_mut().quads.push(quad);
            }
            Surface::Label(_) => {
                self.frame.borrow_mut().labels += 1;
                self.note("text labels are not rasterised yet");
            }
        }
    }

    fn draw_cut(&mut self, sheet: &Imgcut, x: i32, y: i32, cut: i32) {
        self.push_cut_native(sheet, cut, x as f32, y as f32);
    }

    fn draw_cut_scaled(&mut self, sheet: &Imgcut, x: i32, y: i32, width: i32, height: i32, cut: i32) {
        self.push_cut_box(sheet, cut, x as f32, y as f32, width as f32, height as f32);
    }

    fn draw_cut_f(&mut self, sheet: &Imgcut, cut: i32, x: f32, y: f32, width: f32, height: f32) {
        self.push_cut_box(sheet, cut, x, y, width, height);
    }

    fn draw_region(
        &mut self,
        sheet: &Imgcut,
        x: i32,
        y: i32,
        src_x: i32,
        src_y: i32,
        src_w: i32,
        src_h: i32,
    ) {
        self.draw_region_f(
            sheet,
            src_x,
            src_y,
            src_w,
            src_h,
            x as f32,
            y as f32,
            src_w as f32,
            src_h as f32,
        );
    }

    fn draw_region_f(
        &mut self,
        sheet: &Imgcut,
        src_x: i32,
        src_y: i32,
        src_w: i32,
        src_h: i32,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
    ) {
        let name = String::from_utf8_lossy(&sheet.png).into_owned();
        let quad = Quad {
            sheet: Some(Box::from(name.as_str())),
            corners: self.box_corners(x, y, width, height),
            source: [src_x as f32, src_y as f32, src_w as f32, src_h as f32],
            color: self.paint(),
        };

        self.frame.borrow_mut().quads.push(quad);
    }

    fn draw_surface_scaled(
        &mut self,
        surface: Surface<'_>,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
    ) {
        match surface {
            Surface::Sheet(sheet) => {
                let source = [0.0, 0.0, sheet.width as f32, sheet.height as f32];
                let name = String::from_utf8_lossy(&sheet.png).into_owned();
                let quad = Quad {
                    sheet: Some(Box::from(name.as_str())),
                    corners: self.box_corners(x as f32, y as f32, width as f32, height as f32),
                    source,
                    color: self.paint(),
                };

                self.frame.borrow_mut().quads.push(quad);
            }
            Surface::Label(_) => {
                self.frame.borrow_mut().labels += 1;
                self.note("text labels are not rasterised yet");
            }
        }
    }

    fn draw_model(&mut self, model: &Mamodel, x: i32, y: i32) {
        self.draw_model_scaled(model, x, y, 0, 0, 1.0, 0xff, 0, 0);
    }

    fn draw_model_scaled(
        &mut self,
        model: &Mamodel,
        x: i32,
        y: i32,
        _pivot_x: i32,
        _pivot_y: i32,
        scale: f32,
        alpha: i32,
        _first: i32,
        _second: i32,
    ) {
        let Some(sheet) = model.sheet.as_deref() else {
            return;
        };
        let held = self.state;

        self.state.origin = [
            held.origin[0].wrapping_add(x),
            held.origin[1].wrapping_add(y),
        ];
        self.state.scale = held.scale * scale;
        self.state.alpha = (held.alpha * alpha) / 255;

        for index in &model.draw_order {
            let Some(part) = model.parts.get(*index as i64 as usize) else {
                continue;
            };

            if part.i32_at(PART_PARENT) == -1 && *index != 0 {
                continue;
            }

            let cut = part.i32_at(PART_CUT);
            let mut corners = [[0.0f32; 2]; 4];

            for (slot, offset) in PART_CORNERS.iter().enumerate() {
                let corner_x = part.i32_at(*offset) as f32;
                let corner_y = part.i32_at(offset + 4) as f32;

                corners[slot] = self.place(corner_x, corner_y);
            }

            self.push_cut(sheet, cut, corners);
        }

        self.state = held;
    }

    fn draw_cut_rotated(
        &mut self,
        sheet: &Imgcut,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        angle: f32,
        _align: i32,
        _pivot_x: i32,
        _pivot_y: i32,
        _pivot_align: i32,
        cut: i32,
    ) {
        self.draw_cut_rotated_f(
            sheet,
            x as f32,
            y as f32,
            width as f32,
            height as f32,
            0.0,
            0.0,
            angle,
            0,
            0,
            cut,
        );
    }

    fn draw_cut_rotated_f(
        &mut self,
        sheet: &Imgcut,
        x: f32,
        y: f32,
        width: f32,
        height: f32,
        pivot_x: f32,
        pivot_y: f32,
        angle: f32,
        _align: i32,
        _pivot_align: i32,
        cut: i32,
    ) {
        let radians = angle.to_radians();
        let (sin, cos) = radians.sin_cos();
        let spin = |dx: f32, dy: f32| {
            let ox = dx - pivot_x;
            let oy = dy - pivot_y;

            (pivot_x + ox * cos - oy * sin, pivot_y + ox * sin + oy * cos)
        };
        let offsets = [(0.0, 0.0), (0.0, height), (width, height), (width, 0.0)];
        let mut corners = [[0.0f32; 2]; 4];

        for (slot, (dx, dy)) in offsets.into_iter().enumerate() {
            let (rx, ry) = spin(dx, dy);

            corners[slot] = self.place(x + rx, y + ry);
        }

        self.push_cut(sheet, cut, corners);
    }

    fn draw_cut_spun(
        &mut self,
        sheet: &Imgcut,
        x: i32,
        y: i32,
        angle: f32,
        align: i32,
        pivot_x: i32,
        pivot_y: i32,
        pivot_align: i32,
        cut: i32,
    ) {
        let (width, height) = sheet
            .cuts
            .get(cut as i64 as usize)
            .map_or((0, 0), |cell| (cell[2], cell[3]));

        self.draw_cut_rotated(
            sheet, x, y, width, height, angle, align, pivot_x, pivot_y, pivot_align, cut,
        );
    }

    fn draw_image_rotated(
        &mut self,
        sheet: &Imgcut,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        angle: f32,
        align: i32,
        pivot_x: i32,
        pivot_y: i32,
        pivot_align: i32,
    ) {
        self.draw_cut_rotated(
            sheet, x, y, width, height, angle, align, pivot_x, pivot_y, pivot_align, 0,
        );
    }

    fn draw_panel(
        &mut self,
        sheet: &Imgcut,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        _scale: f32,
        cut_a: i32,
        cut_b: i32,
    ) {
        self.push_cut_box(
            sheet,
            cut_a,
            x as f32,
            y as f32,
            width as f32,
            height as f32,
        );
        self.push_cut_box(
            sheet,
            cut_b,
            x as f32,
            y as f32,
            width as f32,
            height as f32,
        );
    }

    fn draw_nine_slice(
        &mut self,
        sheet: &Imgcut,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        _scale: f32,
        cut: i32,
        _border_x: i32,
        _border_y: i32,
        _inner_w: i32,
        _inner_h: i32,
    ) {
        self.note("draw_nine_slice stretches instead of slicing");
        self.push_cut_box(sheet, cut, x as f32, y as f32, width as f32, height as f32);
    }

    fn draw_quad_cut(
        &mut self,
        sheet: &Imgcut,
        x0: i32,
        y0: i32,
        x1: i32,
        y1: i32,
        x2: i32,
        y2: i32,
        x3: i32,
        y3: i32,
        cut: i32,
    ) {
        let corners = [
            self.place(x0 as f32, y0 as f32),
            self.place(x1 as f32, y1 as f32),
            self.place(x2 as f32, y2 as f32),
            self.place(x3 as f32, y3 as f32),
        ];

        self.push_cut(sheet, cut, corners);
    }

    fn draw_quad_region(
        &mut self,
        sheet: &Imgcut,
        x0: i32,
        y0: i32,
        x1: i32,
        y1: i32,
        x2: i32,
        y2: i32,
        x3: i32,
        y3: i32,
        src_x: i32,
        src_y: i32,
        src_w: i32,
        src_h: i32,
    ) {
        let corners = [
            self.place(x0 as f32, y0 as f32),
            self.place(x1 as f32, y1 as f32),
            self.place(x2 as f32, y2 as f32),
            self.place(x3 as f32, y3 as f32),
        ];
        let name = String::from_utf8_lossy(&sheet.png).into_owned();
        let quad = Quad {
            sheet: Some(Box::from(name.as_str())),
            corners,
            source: [src_x as f32, src_y as f32, src_w as f32, src_h as f32],
            color: self.paint(),
        };

        self.frame.borrow_mut().quads.push(quad);
    }

    fn draw_sprite_cut(
        &mut self,
        sheet: &Imgcut,
        x0: i32,
        y0: i32,
        x1: i32,
        y1: i32,
        x2: i32,
        y2: i32,
        x3: i32,
        y3: i32,
        cut: i32,
    ) {
        self.draw_quad_cut(sheet, x0, y0, x1, y1, x2, y2, x3, y3, cut);
    }
}
