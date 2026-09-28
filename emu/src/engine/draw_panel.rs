use crate::{Fault, ops};

use super::{DrawSink, Imgcut, draw_region_scaled, imgcut_get_sprite_cut};

pub fn draw_panel(
    dc: &mut dyn DrawSink,
    sheet: &Imgcut,
    x: i32,
    y: i32,
    width: i32,
    height: i32,
    scale: f32,
    outer_cut: i32,
    inner_cut: i32,
) -> Result<(), Fault> {
    let inner = *imgcut_get_sprite_cut(sheet, inner_cut)?;
    let outer = *imgcut_get_sprite_cut(sheet, outer_cut)?;
    let left = inner[0].wrapping_sub(outer[0]);
    let right = outer[2].wrapping_sub(inner[2].wrapping_add(left));
    let left_f = left as f32 * scale;
    let right_f = -(right as f32) * scale;
    let middle_w = ops::cvttss2si(width as f32 - left_f + right_f);
    let top = inner[1].wrapping_sub(outer[1]);
    let top_f = top as f32 * scale;
    let bottom = outer[3].wrapping_sub(inner[3].wrapping_add(top));
    let bottom_f = -(bottom as f32) * scale;
    let middle_h = ops::cvttss2si(height as f32 - top_f + bottom_f);
    let left_w = ops::cvttss2si(left_f);
    let top_h = ops::cvttss2si(top_f);

    draw_region_scaled(dc, sheet, x, y, left_w, top_h, outer[0], outer[1], left, top);

    let x1 = ops::cvttss2si(x as f32 + left_f);

    draw_region_scaled(
        dc,
        sheet,
        x1,
        y,
        middle_w,
        top_h,
        outer[0].wrapping_add(left),
        outer[1],
        inner[2],
        top,
    );

    let x2 = ops::cvttss2si(width.wrapping_add(x) as f32 + right_f);
    let right_w = ops::cvttss2si(right as f32 * scale);

    draw_region_scaled(
        dc,
        sheet,
        x2,
        y,
        right_w,
        top_h,
        outer[0].wrapping_sub(right).wrapping_add(outer[2]),
        outer[1],
        right,
        top,
    );

    let y1 = ops::cvttss2si(y as f32 + top_f);

    draw_region_scaled(
        dc,
        sheet,
        x,
        y1,
        left_w,
        middle_h,
        outer[0],
        outer[1].wrapping_add(top),
        left,
        inner[3],
    );
    draw_region_scaled(
        dc, sheet, x1, y1, middle_w, middle_h, inner[0], inner[1], inner[2], inner[3],
    );
    draw_region_scaled(
        dc,
        sheet,
        x2,
        y1,
        right_w,
        middle_h,
        outer[0].wrapping_sub(right).wrapping_add(outer[2]),
        top.wrapping_add(outer[1]),
        right,
        inner[3],
    );

    let y2 = ops::cvttss2si(height.wrapping_add(y) as f32 + bottom_f);
    let bottom_h = ops::cvttss2si(bottom as f32 * scale);

    draw_region_scaled(
        dc,
        sheet,
        x,
        y2,
        left_w,
        bottom_h,
        outer[0],
        inner[1].wrapping_add(inner[3]),
        left,
        bottom,
    );
    draw_region_scaled(
        dc,
        sheet,
        x1,
        y2,
        middle_w,
        bottom_h,
        left.wrapping_add(outer[0]),
        inner[1].wrapping_add(inner[3]),
        inner[2],
        bottom,
    );
    draw_region_scaled(
        dc,
        sheet,
        x2,
        y2,
        right_w,
        bottom_h,
        outer[0].wrapping_sub(right).wrapping_add(outer[2]),
        inner[1].wrapping_add(inner[3]),
        right,
        bottom,
    );

    Ok(())
}
