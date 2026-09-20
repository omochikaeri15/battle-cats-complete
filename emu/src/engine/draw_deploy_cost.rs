use crate::{Fault, operation};

use super::{DrawSink, Imgcut, draw_cut_scaled, draw_number_scaled};

pub fn draw_deploy_cost(
    dc: &mut dyn DrawSink,
    sheet: &Imgcut,
    alt: Option<&Imgcut>,
    value: i32,
    x: i32,
    y: i32,
    mode: i32,
    percent: i32,
    style: i32,
) -> Result<(), Fault> {
    if style == 1 {
        let alt = alt.ok_or(Fault::null_pointer())?;
        let span = 0x64i32.wrapping_sub(percent);
        let width = operation::div_100(span.wrapping_mul(0x16));
        let height = operation::div_100(span.wrapping_mul(0x1a));
        let across = x
            .wrapping_add(width)
            .wrapping_add(operation::div_2(0x16i32.wrapping_sub(width)));
        let down = y.wrapping_add(operation::div_2(0x1ai32.wrapping_sub(height)));
        let scale = percent as f32 / -100.0 + 1.0;
        let box_ = draw_number_scaled(
            dc,
            alt,
            0x3c,
            value,
            0,
            across as f32,
            down as f32,
            -3.0,
            scale,
            width,
            2,
            0,
        )?;

        draw_cut_scaled(
            dc,
            alt,
            operation::cvttss2si(box_.right),
            down,
            width,
            height,
            0x46,
        );

        return Ok(());
    }

    if style != 0 {
        return Ok(());
    }

    let span = 0x64i32.wrapping_sub(percent);
    let width = operation::div_100(span.wrapping_mul(0x16));
    let height = operation::div_100(span.wrapping_mul(0x1a));
    let down = y.wrapping_add(operation::div_2(0x1ai32.wrapping_sub(height)));
    let scale = percent as f32 / -100.0 + 1.0;

    if (mode as u32) <= 1 {
        let across = x
            .wrapping_add(width)
            .wrapping_add(operation::div_2(0x16i32.wrapping_sub(width)))
            .wrapping_add(4);
        let base = if mode == 0 { 0x23 } else { 0x2e };
        let icon = if mode == 0 { 0x2d } else { 0x38 };
        let box_ = draw_number_scaled(
            dc,
            sheet,
            base,
            value,
            0,
            across as f32,
            (down.wrapping_add(0xc)) as f32,
            -3.0,
            scale,
            width,
            2,
            0,
        )?;

        draw_cut_scaled(
            dc,
            sheet,
            operation::cvttss2si(box_.right),
            down.wrapping_add(0xc),
            width,
            height,
            icon,
        );

        return Ok(());
    }

    if (mode & -2i32) != 2 {
        return Ok(());
    }

    let base = if mode == 2 { 0x23 } else { 0x2e };
    let icon = if mode == 2 { 0x2d } else { 0x38 };
    let box_ = draw_number_scaled(
        dc,
        sheet,
        base,
        value,
        0,
        x as f32,
        down as f32,
        -3.0,
        scale,
        width,
        0,
        0,
    )?;

    draw_cut_scaled(
        dc,
        sheet,
        operation::cvttss2si(box_.right),
        down,
        width,
        height,
        icon,
    );

    Ok(())
}
