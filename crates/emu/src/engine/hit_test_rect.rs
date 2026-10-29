use crate::Fault;

use super::{AppContext, get_touch_x, get_touch_y};

pub fn hit_test_rect(
    ctx: &AppContext,
    x: i32,
    y: i32,
    width: i32,
    height: i32,
) -> Result<bool, Fault> {
    if get_touch_x(ctx)? < x {
        return Ok(false);
    }

    if get_touch_x(ctx)? > width.wrapping_add(x) {
        return Ok(false);
    }

    if get_touch_y(ctx)? < y {
        return Ok(false);
    }

    Ok(get_touch_y(ctx)? <= height.wrapping_add(y))
}
