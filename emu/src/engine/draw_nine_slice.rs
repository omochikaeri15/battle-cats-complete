use crate::{Fault, ops};

use super::{DrawSink, Imgcut, draw_region_scaled, imgcut_get_sprite_cut};

pub fn draw_nine_slice(
    dc: &mut dyn DrawSink,
    sheet: &Imgcut,
    x: i32,
    y: i32,
    width: i32,
    height: i32,
    scale: f32,
    cut_index: i32,
    border_x: i32,
    border_y: i32,
    _inner_w: i32,
    _inner_h: i32,
) -> Result<(), Fault> {
    let edge_x = border_x as f32 * scale;
    let across = ops::cvttss2si(edge_x);
    let edge_y = border_y as f32 * scale;
    let down = ops::cvttss2si(edge_y);
    let cut = *imgcut_get_sprite_cut(sheet, cut_index)?;

    draw_region_scaled(dc, sheet, x, y, across, down, cut[0], cut[1], border_x, border_y);

    let x1 = ops::cvttss2si(x as f32 + edge_x);
    let twice_x = border_x.wrapping_mul(2);
    let middle_w = ops::cvttss2si(width as f32 - twice_x as f32 * scale);

    draw_region_scaled(
        dc,
        sheet,
        x1,
        y,
        middle_w,
        down,
        cut[0].wrapping_add(border_x),
        cut[1],
        cut[2].wrapping_sub(twice_x),
        border_y,
    );

    let x2 = ops::cvttss2si(width.wrapping_add(x) as f32 - edge_x);

    draw_region_scaled(
        dc,
        sheet,
        x2,
        y,
        across,
        down,
        cut[0].wrapping_sub(border_x).wrapping_add(cut[2]),
        cut[1],
        border_x,
        border_y,
    );

    let y1 = ops::cvttss2si(y as f32 + edge_y);
    let twice_y = border_y.wrapping_mul(2);
    let middle_h = ops::cvttss2si(height as f32 - twice_y as f32 * scale);

    draw_region_scaled(
        dc,
        sheet,
        x2,
        y1,
        across,
        middle_h,
        cut[0].wrapping_sub(border_x).wrapping_add(cut[2]),
        cut[1].wrapping_add(border_y),
        border_x,
        cut[3].wrapping_sub(twice_y),
    );

    let y2 = ops::cvttss2si(height.wrapping_add(y) as f32 - edge_y);

    draw_region_scaled(
        dc,
        sheet,
        x2,
        y2,
        across,
        down,
        cut[0].wrapping_sub(border_x).wrapping_add(cut[2]),
        cut[1].wrapping_sub(border_y).wrapping_add(cut[3]),
        border_x,
        border_y,
    );
    draw_region_scaled(
        dc,
        sheet,
        x1,
        y2,
        middle_w,
        down,
        cut[0].wrapping_add(border_x),
        cut[1].wrapping_sub(border_y).wrapping_add(cut[3]),
        cut[2].wrapping_sub(twice_x),
        border_y,
    );
    draw_region_scaled(
        dc,
        sheet,
        x,
        y2,
        across,
        down,
        cut[0],
        cut[1].wrapping_sub(border_y).wrapping_add(cut[3]),
        border_x,
        border_y,
    );
    draw_region_scaled(
        dc,
        sheet,
        x,
        y1,
        across,
        middle_h,
        cut[0],
        cut[1].wrapping_add(border_y),
        border_x,
        cut[3].wrapping_sub(twice_y),
    );
    draw_region_scaled(
        dc,
        sheet,
        x1,
        y1,
        middle_w,
        middle_h,
        border_x.wrapping_add(cut[0]),
        border_y.wrapping_add(cut[1]),
        cut[2].wrapping_sub(twice_x),
        cut[3].wrapping_sub(twice_y),
    );

    Ok(())
}
