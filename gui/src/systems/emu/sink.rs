use std::cell::RefCell;
use std::rc::Rc;

use emu::engine::{DrawSink, Imgcut, Mamodel, Surface};

const IDENTITY: [f32; 6] = [1.0, 0.0, 0.0, 1.0, 0.0, 0.0];
const PART_SHEET: [usize; 2] = [0x24, 0x28];
const PART_CUT: [usize; 2] = [0x2c, 0x30];
const PART_OPACITY: usize = 0x80;
const PART_GLOW: usize = 0x8c;
const PART_CORNERS: [usize; 4] = [0x90, 0x98, 0xa0, 0xa8];
const ALIGN_CENTER: i32 = 1;
const ALIGN_RIGHT: i32 = 2;
const ALIGN_MIDDLE: i32 = 4;
const ALIGN_BOTTOM: i32 = 8;
const PIVOT_ABSOLUTE: i32 = 0x10;
const LAST_GLOW: i32 = 3;

#[derive(Clone, PartialEq, Debug)]
pub struct Quad {
    pub sheet: Option<Box<str>>,
    pub label: Option<u64>,
    pub corners: [[f32; 2]; 4],
    pub source: [f32; 4],
    pub colors: [[f32; 4]; 4],
    pub blend: u8,
}

#[derive(Clone, Copy)]
struct State {
    origin: [i32; 2],
    color: [i32; 4],
    tint: [i32; 4],
    flip: i32,
    glow: i32,
    matrix: [f32; 6],
}

impl Default for State {
    fn default() -> Self {
        Self {
            origin: [0, 0],
            color: [0xff; 4],
            tint: [0xff; 4],
            flip: 0,
            glow: 0,
            matrix: IDENTITY,
        }
    }
}

#[derive(Default)]
pub struct Frame {
    pub quads: Vec<Quad>,
}

impl Frame {
    pub fn clear(&mut self) {
        self.quads.clear();
    }
}

pub struct Recorder {
    frame: Rc<RefCell<Frame>>,
    state: State,
}

fn premultiplied(color: [i32; 4]) -> [f32; 4] {
    let alpha = color[3];

    [
        color[0].wrapping_mul(alpha) as f32 / 65025.0,
        color[1].wrapping_mul(alpha) as f32 / 65025.0,
        color[2].wrapping_mul(alpha) as f32 / 65025.0,
        alpha as f32 / 255.0,
    ]
}

fn pack(color: [i32; 4]) -> u32 {
    ((color[3] as u32 & 0xff) << 0x18)
        | ((color[0] as u32 & 0xff) << 0x10)
        | ((color[1] as u32 & 0xff) << 8)
        | (color[2] as u32 & 0xff)
}

fn unpack(color: u32) -> [f32; 4] {
    premultiplied([
        (color >> 0x10 & 0xff) as i32,
        (color >> 8 & 0xff) as i32,
        (color & 0xff) as i32,
        (color >> 0x18) as i32,
    ])
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

impl Recorder {
    pub fn new(frame: Rc<RefCell<Frame>>) -> Self {
        Self {
            frame,
            state: State::default(),
        }
    }

    fn place(&self, x: f32, y: f32) -> [f32; 2] {
        let matrix = self.state.matrix;
        let local_x = x + self.state.origin[0] as f32;
        let local_y = y + self.state.origin[1] as f32;

        [
            matrix[0] * local_x + matrix[2] * local_y + matrix[4],
            matrix[1] * local_x + matrix[3] * local_y + matrix[5],
        ]
    }

    fn flipped(&self, x: f32, y: f32, width: f32, height: f32) -> [[f32; 2]; 4] {
        let (left, right, top, bottom) = (x, x + width, y, y + height);

        let local = match self.state.flip {
            1 => [[right, top], [right, bottom], [left, bottom], [left, top]],
            2 => [[left, bottom], [left, top], [right, top], [right, bottom]],
            3 => [[right, bottom], [right, top], [left, top], [left, bottom]],
            4 => [
                [x, y + width],
                [x + height, y + width],
                [x + height, y],
                [x, y],
            ],
            5 => [
                [x + height, y],
                [x, y],
                [x, y + width],
                [x + height, y + width],
            ],
            _ => [[left, top], [left, bottom], [right, bottom], [right, top]],
        };

        local.map(|corner| self.place(corner[0], corner[1]))
    }

    fn push(&mut self, quad: Quad) {
        self.frame.borrow_mut().quads.push(quad);
    }

    fn push_solid(&mut self, x: f32, y: f32, width: f32, height: f32) {
        let quad = Quad {
            sheet: None,
            label: None,
            corners: [
                self.place(x, y),
                self.place(x, y + height),
                self.place(x + width, y + height),
                self.place(x + width, y),
            ],
            source: [0.0; 4],
            colors: [premultiplied(self.state.tint); 4],
            blend: self.state.glow as u8,
        };

        self.push(quad);
    }

    fn push_region(&mut self, sheet: &Imgcut, source: [i32; 4], corners: [[f32; 2]; 4]) {
        let quad = Quad {
            sheet: Some(Box::from(String::from_utf8_lossy(&sheet.png).as_ref())),
            label: None,
            corners,
            source: source.map(|cell| cell as f32),
            colors: [premultiplied(self.state.color); 4],
            blend: self.state.glow as u8,
        };

        self.push(quad);
    }

    fn cut_of(sheet: &Imgcut, cut: i32) -> Option<[i32; 4]> {
        sheet
            .cuts
            .get(cut as i64 as usize)
            .map(|cell| [cell[0], cell[1], cell[2], cell[3]])
    }

    fn blit(&mut self, sheet: &Imgcut, source: [i32; 4], x: f32, y: f32, width: f32, height: f32) {
        let corners = self.flipped(x, y, width, height);

        self.push_region(sheet, source, corners);
    }

    fn blit_surface(&mut self, surface: Surface<'_>, x: f32, y: f32, width: f32, height: f32) {
        match surface {
            Surface::Sheet(sheet) => {
                self.blit(sheet, [0, 0, sheet.width, sheet.height], x, y, width, height);
            }
            Surface::Label(texture) => {
                let quad = Quad {
                    sheet: None,
                    label: Some(texture.id),
                    corners: self.flipped(x, y, width, height),
                    source: [0.0, 0.0, texture.width as f32, texture.height as f32],
                    colors: [premultiplied(self.state.tint); 4],
                    blend: 0,
                };

                self.push(quad);
            }
        }
    }

    fn surface_size(surface: Surface<'_>) -> (i32, i32) {
        match surface {
            Surface::Sheet(sheet) => (sheet.width, sheet.height),
            Surface::Label(texture) => (texture.width, texture.height),
        }
    }

    fn slices(
        &mut self,
        sheet: &Imgcut,
        outer: [i32; 4],
        inner: [i32; 4],
        dest: [f32; 4],
        scale: f32,
    ) {
        let [x, y, width, height] = dest;
        let left = inner[0] - outer[0];
        let right = outer[2] - inner[2] - left;
        let top = inner[1] - outer[1];
        let bottom = outer[3] - inner[3] - top;
        let columns = [
            (outer[0], left, x, left as f32 * scale),
            (
                inner[0],
                inner[2],
                x + left as f32 * scale,
                width - (left + right) as f32 * scale,
            ),
            (
                outer[0] + outer[2] - right,
                right,
                x + width - right as f32 * scale,
                right as f32 * scale,
            ),
        ];
        let rows = [
            (outer[1], top, y, top as f32 * scale),
            (
                inner[1],
                inner[3],
                y + top as f32 * scale,
                height - (top + bottom) as f32 * scale,
            ),
            (
                inner[1] + inner[3],
                bottom,
                y + height - bottom as f32 * scale,
                bottom as f32 * scale,
            ),
        ];

        for (src_y, src_h, dest_y, dest_h) in rows {
            for (src_x, src_w, dest_x, dest_w) in columns {
                self.blit(sheet, [src_x, src_y, src_w, src_h], dest_x, dest_y, dest_w, dest_h);
            }
        }
    }

    fn spun(
        &self,
        dest: [f32; 4],
        angle: f32,
        align: i32,
        pivot: [f32; 2],
        pivot_align: i32,
    ) -> [[f32; 2]; 4] {
        let [x, y, width, height] = dest;
        let left = x - aligned(width as i32, align, ALIGN_CENTER, ALIGN_RIGHT) as f32;
        let top = y - aligned(height as i32, align, ALIGN_MIDDLE, ALIGN_BOTTOM) as f32;
        let (pivot_x, pivot_y) = if pivot_align & PIVOT_ABSOLUTE != 0 {
            (pivot[0], pivot[1])
        } else {
            (
                pivot[0] + left + aligned(width as i32, pivot_align, ALIGN_CENTER, ALIGN_RIGHT) as f32,
                pivot[1] + top + aligned(height as i32, pivot_align, ALIGN_MIDDLE, ALIGN_BOTTOM) as f32,
            )
        };
        let radians = (f64::from(angle) * -std::f64::consts::PI / 180.0) as f32;
        let (sin, cos) = radians.sin_cos();
        let near_x = left - pivot_x;
        let near_y = top - pivot_y;
        let far_x = near_x + width;
        let far_y = near_y + height;
        let turn = |dx: f32, dy: f32| [dx * cos + dy * sin + pivot_x, dy * cos - dx * sin + pivot_y];

        [
            turn(near_x, near_y),
            turn(near_x, far_y),
            turn(far_x, far_y),
            turn(far_x, near_y),
        ]
    }

    fn draw_part(&mut self, model: &Mamodel, index: i32, place: impl Fn(i32, i32) -> [i32; 2], alpha: i32) {
        let Some(part) = model.parts.get(index as i64 as usize) else {
            return;
        };
        let sprite = part.i32_at(PART_SHEET[0]).wrapping_add(part.i32_at(PART_SHEET[1]));

        if sprite == -1 {
            return;
        }

        let glow = part.i32_at(PART_GLOW);

        if (0..=LAST_GLOW).contains(&glow) {
            self.state.glow = glow;
        }

        if alpha == 0 {
            return;
        }

        self.state.color[3] = alpha;

        let table = if model.single_sheet != 0 { 0 } else { sprite as i64 as usize };
        let shared = model.sheet.as_ref().map_or_else(
            || {
                model.sheet_table.get(table).and_then(|slot| {
                    let held = slot.take();

                    slot.set(held.as_ref().map(Rc::clone));
                    held
                })
            },
            |sheet| Some(Rc::clone(sheet)),
        );
        let Some(sheet) = shared.as_deref() else {
            return;
        };
        let cut = part.i32_at(PART_CUT[0]).wrapping_add(part.i32_at(PART_CUT[1]));
        let Some(source) = Self::cut_of(sheet, cut) else {
            return;
        };
        let corners = PART_CORNERS.map(|offset| {
            let [x, y] = place(part.i32_at(offset), part.i32_at(offset + 4));

            self.place(x as f32, y as f32)
        });

        self.push_region(sheet, source, corners);
    }
}

impl DrawSink for Recorder {
    fn set_origin(&mut self, x: i32, y: i32) {
        self.state.origin = [x, y];
    }

    fn glow_set(&mut self, mode: i32) {
        self.state.glow = mode;
    }

    fn set_draw_scale(&mut self, _scale: f32) {
        self.state.matrix = IDENTITY;
    }

    fn set_tint(&mut self, red: i32, green: i32, blue: i32, alpha: i32) {
        self.state.tint = [red, green, blue, alpha];
    }

    fn set_color(&mut self, red: i32, green: i32, blue: i32, alpha: i32) {
        self.state.color = [red, green, blue, alpha];
    }

    fn set_transform(&mut self, _scale: f32, matrix: &[f32; 6]) {
        let across = (matrix[0] * matrix[0] + matrix[2] * matrix[2]).sqrt();
        let down = (matrix[1] * matrix[1] + matrix[3] * matrix[3]).sqrt();

        self.state.matrix = [across, 0.0, 0.0, down, matrix[4], matrix[5]];
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
        let colors = vec![pack(self.state.tint); count.max(0) as usize];

        self.fill_polygon_colored(xs, ys, &colors, count);
    }

    fn fill_polygon_colored(&mut self, xs: &[i32], ys: &[i32], colors: &[u32], count: i32) {
        let count = count as i64 as usize;
        let placed: Vec<Option<([f32; 2], [f32; 4])>> = (0..count)
            .map(|at| {
                let (x, y, color) = (xs.get(at)?, ys.get(at)?, colors.get(at)?);

                Some((self.place(*x as f32, *y as f32), unpack(*color)))
            })
            .collect();
        let corner = |at: usize| placed.get(at).copied().flatten();

        for second in 1..count.saturating_sub(1) {
            let (Some(a), Some(b), Some(c)) = (corner(0), corner(second), corner(second + 1)) else {
                return;
            };
            let quad = Quad {
                sheet: None,
                label: None,
                corners: [a.0, b.0, c.0, c.0],
                source: [0.0; 4],
                colors: [a.1, b.1, c.1, c.1],
                blend: self.state.glow as u8,
            };

            self.push(quad);
        }
    }

    fn draw_surface_aligned(&mut self, surface: Surface<'_>, x: i32, y: i32, align: i32) {
        let (width, height) = Self::surface_size(surface);
        let left = x.wrapping_sub(aligned(width, align, ALIGN_CENTER, ALIGN_RIGHT));
        let top = y.wrapping_sub(aligned(height, align, ALIGN_MIDDLE, ALIGN_BOTTOM));

        self.blit_surface(surface, left as f32, top as f32, width as f32, height as f32);
    }

    fn set_alpha(&mut self, alpha: i32) {
        self.state.color[3] = alpha;
    }

    fn set_flip(&mut self, flip: i32) {
        self.state.flip = flip;
    }

    fn draw_surface(&mut self, surface: Surface<'_>, x: i32, y: i32) {
        let (width, height) = Self::surface_size(surface);

        self.blit_surface(surface, x as f32, y as f32, width as f32, height as f32);
    }

    fn draw_cut(&mut self, sheet: &Imgcut, x: i32, y: i32, cut: i32) {
        if let Some(source) = Self::cut_of(sheet, cut) {
            self.blit(sheet, source, x as f32, y as f32, source[2] as f32, source[3] as f32);
        }
    }

    fn draw_cut_scaled(&mut self, sheet: &Imgcut, x: i32, y: i32, width: i32, height: i32, cut: i32) {
        if let Some(source) = Self::cut_of(sheet, cut) {
            self.blit(sheet, source, x as f32, y as f32, width as f32, height as f32);
        }
    }

    fn draw_cut_f(&mut self, sheet: &Imgcut, cut: i32, x: f32, y: f32, width: f32, height: f32) {
        if let Some(source) = Self::cut_of(sheet, cut) {
            self.blit(sheet, source, x, y, width, height);
        }
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
        self.blit(sheet, [src_x, src_y, src_w, src_h], x as f32, y as f32, src_w as f32, src_h as f32);
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
        self.blit(sheet, [src_x, src_y, src_w, src_h], x, y, width, height);
    }

    fn draw_surface_scaled(&mut self, surface: Surface<'_>, x: i32, y: i32, width: i32, height: i32) {
        self.blit_surface(surface, x as f32, y as f32, width as f32, height as f32);
    }

    fn draw_model(&mut self, model: &Mamodel, x: i32, y: i32) {
        let held = self.state;

        for index in &model.draw_order {
            let alpha = model
                .parts
                .get(*index as i64 as usize)
                .and_then(|part| part.i32_at(PART_OPACITY).wrapping_mul(0xff).checked_div(model.opacity_unit))
                .unwrap_or(0);

            self.draw_part(model, *index, |part_x, part_y| [part_x.wrapping_add(x), part_y.wrapping_add(y)], alpha);
        }

        self.state = held;
    }

    fn draw_model_scaled(
        &mut self,
        model: &Mamodel,
        x: i32,
        y: i32,
        pivot_x: i32,
        pivot_y: i32,
        scale: f32,
        alpha: i32,
        first: i32,
        second: i32,
    ) {
        let held = self.state;
        let shift_x = first.wrapping_sub(pivot_x);
        let shift_y = second.wrapping_sub(pivot_y);

        for index in &model.draw_order {
            let faded = model
                .parts
                .get(*index as i64 as usize)
                .and_then(|part| part.i32_at(PART_OPACITY).wrapping_mul(0xff).checked_div(model.opacity_unit))
                .map_or(0, |opacity| opacity.wrapping_mul(alpha) / 0xff);

            self.draw_part(
                model,
                *index,
                |part_x, part_y| {
                    [
                        (part_x.wrapping_add(shift_x) as f32 * scale + pivot_x as f32 + x as f32) as i32,
                        (part_y.wrapping_add(shift_y) as f32 * scale + pivot_y as f32 + y as f32) as i32,
                    ]
                },
                faded,
            );
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
        align: i32,
        pivot_x: i32,
        pivot_y: i32,
        pivot_align: i32,
        cut: i32,
    ) {
        let Some(source) = Self::cut_of(sheet, cut) else {
            return;
        };
        let corners = self
            .spun([x as f32, y as f32, width as f32, height as f32], angle, align, [pivot_x as f32, pivot_y as f32], pivot_align)
            .map(|corner| self.place(corner[0].trunc(), corner[1].trunc()));

        self.push_region(sheet, source, corners);
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
        align: i32,
        pivot_align: i32,
        cut: i32,
    ) {
        let Some(source) = Self::cut_of(sheet, cut) else {
            return;
        };
        let corners = self
            .spun([x, y, width, height], angle, align, [pivot_x, pivot_y], pivot_align)
            .map(|corner| self.place(corner[0], corner[1]));

        self.push_region(sheet, source, corners);
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
        let (width, height) = Self::cut_of(sheet, cut).map_or((0, 0), |cell| (cell[2], cell[3]));

        self.draw_cut_rotated(sheet, x, y, width, height, angle, align, pivot_x, pivot_y, pivot_align, cut);
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
        let corners = self
            .spun([x as f32, y as f32, width as f32, height as f32], angle, align, [pivot_x as f32, pivot_y as f32], pivot_align)
            .map(|corner| self.place(corner[0].trunc(), corner[1].trunc()));

        self.push_region(sheet, [0, 0, sheet.width, sheet.height], corners);
    }

    fn draw_panel(
        &mut self,
        sheet: &Imgcut,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        scale: f32,
        cut_a: i32,
        cut_b: i32,
    ) {
        let (Some(outer), Some(inner)) = (Self::cut_of(sheet, cut_a), Self::cut_of(sheet, cut_b)) else {
            return;
        };

        self.slices(sheet, outer, inner, [x as f32, y as f32, width as f32, height as f32], scale);
    }

    fn draw_nine_slice(
        &mut self,
        sheet: &Imgcut,
        x: i32,
        y: i32,
        width: i32,
        height: i32,
        scale: f32,
        cut: i32,
        border_x: i32,
        border_y: i32,
        _inner_w: i32,
        _inner_h: i32,
    ) {
        let Some(outer) = Self::cut_of(sheet, cut) else {
            return;
        };
        let edge_x = border_x as f32 * scale;
        let edge_y = border_y as f32 * scale;
        let corner = [edge_x as i32 as f32, edge_y as i32 as f32];
        let near = [x as f32, y as f32];
        let inner = [(x as f32 + edge_x) as i32 as f32, (y as f32 + edge_y) as i32 as f32];
        let far = [
            ((width + x) as f32 - edge_x) as i32 as f32,
            ((height + y) as f32 - edge_y) as i32 as f32,
        ];
        let span = [
            (width as f32 - (border_x * 2) as f32 * scale) as i32 as f32,
            (height as f32 - (border_y * 2) as f32 * scale) as i32 as f32,
        ];
        let columns = [
            (outer[0], border_x, near[0], corner[0]),
            (outer[0] + border_x, outer[2] - border_x * 2, inner[0], span[0]),
            (outer[0] - border_x + outer[2], border_x, far[0], corner[0]),
        ];
        let rows = [
            (outer[1], border_y, near[1], corner[1]),
            (outer[1] + border_y, outer[3] - border_y * 2, inner[1], span[1]),
            (outer[1] - border_y + outer[3], border_y, far[1], corner[1]),
        ];

        for (src_y, src_h, dest_y, dest_h) in rows {
            for (src_x, src_w, dest_x, dest_w) in columns {
                self.blit(sheet, [src_x, src_y, src_w, src_h], dest_x, dest_y, dest_w, dest_h);
            }
        }
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
        if let Some(source) = Self::cut_of(sheet, cut) {
            self.draw_quad_region(sheet, x0, y0, x1, y1, x2, y2, x3, y3, source[0], source[1], source[2], source[3]);
        }
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

        self.push_region(sheet, [src_x, src_y, src_w, src_h], corners);
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
