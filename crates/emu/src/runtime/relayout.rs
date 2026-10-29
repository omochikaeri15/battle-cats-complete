use crate::{
    Fault,
    engine::{
        AppContext, get_bottom_inset_logical, get_design_height2, get_drawable_width, get_left_inset_logical,
        get_right_inset_logical, get_top_inset_offset, option_window_build_alt, powerup_available,
    },
    ops,
};

const DESIGN_WIDTH: i32 = 0x3c0;
const RECT_STRIDE: usize = 0x10;
const ITEM_COLUMNS: i32 = 6;
const ITEM_PITCH: i32 = 0x58;
const ITEM_LEFT: i32 = -0x210;
const ITEM_TOP: i32 = 0x2b;
const BATTLE_OPTION_WINDOW: i32 = 1;

#[derive(Clone, Copy)]
enum Across {
    Inset(i32),
    Edge(i32),
    Left(i32),
    Center(i32),
}

#[derive(Clone, Copy)]
enum Down {
    Kept,
    Raised,
    Corner(i32),
    Footer(i32),
}

#[derive(Clone, Copy)]
enum Slide {
    Edge,
    Center,
}

#[derive(Clone, Copy)]
enum Lift {
    Kept,
    Raised,
    Footer,
    Centered,
}

const SHIFTED: [(i32, Slide, Lift); 5] = [
    (0xc8, Slide::Edge, Lift::Raised),
    (0xc9, Slide::Edge, Lift::Footer),
    (0xca, Slide::Center, Lift::Kept),
    (0xcb, Slide::Center, Lift::Kept),
    (0xd, Slide::Center, Lift::Centered),
];

#[derive(Clone, Copy, Debug)]
pub struct Layout {
    drawable: i32,
    right: i32,
    bottom: i32,
    shift: i32,
    pad: i32,
    design: i32,
}

pub fn layout_snapshot(ctx: &mut AppContext) -> Result<Layout, Fault> {
    Ok(Layout {
        drawable: get_drawable_width(ctx)?,
        right: get_right_inset_logical(ctx)?,
        bottom: get_bottom_inset_logical(ctx)?,
        shift: ctx.i32_at(AppContext::LETTERBOX_SHIFT)?,
        pad: ctx.i32_at(AppContext::LETTERBOX_PAD)?,
        design: get_design_height2(ctx),
    })
}

const LATCHED: [(usize, Across, Down); 26] = [
    (AppContext::CANNON_RECT, Across::Inset(-0x92), Down::Corner(0x1fe)),
    (AppContext::WORKER_RECT, Across::Left(-0x30), Down::Corner(0x207)),
    (AppContext::COMBO_SKIP_RECT, Across::Edge(-0x5c), Down::Kept),
    (AppContext::PAUSE_RECT, Across::Left(0), Down::Raised),
    (AppContext::HUD_RECTS, Across::Center(0x118), Down::Kept),
    (AppContext::HUD_RECTS + 0x20, Across::Center(0x217), Down::Kept),
    (AppContext::HUD_RECTS + 0x30, Across::Center(0x1f2), Down::Kept),
    (AppContext::HUD_RECTS + 0x40, Across::Center(0x25a), Down::Kept),
    (AppContext::HUD_RECTS + 0x50, Across::Center(0x28c), Down::Kept),
    (AppContext::HUD_RECTS + 0x60, Across::Center(0x120), Down::Kept),
    (AppContext::HUD_RECTS + 0x70, Across::Center(0x120), Down::Kept),
    (AppContext::HUD_RECTS + 0x80, Across::Center(0x135), Down::Kept),
    (AppContext::HUD_RECTS + 0x90, Across::Center(0x120), Down::Kept),
    (AppContext::HUD_RECTS + 0xa0, Across::Center(0xfb), Down::Kept),
    (AppContext::HUD_RECTS + 0xb0, Across::Center(0x21d), Down::Kept),
    (AppContext::HUD_RECTS + 0xc0, Across::Center(0x1f4), Down::Kept),
    (AppContext::CAT_GOD_MIRACLE_RECTS, Across::Center(0xf6), Down::Kept),
    (AppContext::CAT_GOD_MIRACLE_RECTS + 0x10, Across::Center(0x1aa), Down::Kept),
    (AppContext::CAT_GOD_MIRACLE_RECTS + 0x20, Across::Center(0x25e), Down::Kept),
    (AppContext::CAT_GOD_MIRACLE_RECTS + 0x30, Across::Center(0x312), Down::Kept),
    (AppContext::CAT_GOD_CONFIRM_RECT, Across::Center(0x1a6), Down::Kept),
    (AppContext::CAT_GOD_BACK_RECT, Across::Center(0x323), Down::Kept),
    (AppContext::CAT_GOD_BACK_RECT + 0x10, Across::Edge(-0x17c), Down::Kept),
    (AppContext::OPTION_RECTS, Across::Edge(-0x103), Down::Footer(0x22e)),
    (AppContext::OPTION_RECTS + 0x10, Across::Edge(-0xa1), Down::Footer(0x22e)),
    (AppContext::LOSE_SHOP_RECT, Across::Edge(-0x118), Down::Kept),
];

pub fn relatch_battle_rects(ctx: &mut AppContext, before: Layout) -> Result<(), Fault> {
    let drawable = get_drawable_width(ctx)?;
    let right = get_right_inset_logical(ctx)?;
    let left = get_left_inset_logical(ctx);
    let top = get_top_inset_offset(ctx);
    let bottom = get_bottom_inset_logical(ctx)?;
    let shift = ctx.i32_at(AppContext::LETTERBOX_SHIFT)?;

    for (rect, across, down) in LATCHED {
        let x = match across {
            Across::Inset(offset) => drawable.wrapping_sub(right).wrapping_add(offset),
            Across::Edge(offset) => drawable.wrapping_add(offset),
            Across::Left(offset) => left.wrapping_add(offset),
            Across::Center(offset) => {
                ops::div_2(drawable.wrapping_sub(DESIGN_WIDTH)).wrapping_add(offset)
            }
        };

        ctx.set_i32_at(rect, x)?;

        let y = match down {
            Down::Kept => continue,
            Down::Raised => shift.wrapping_neg(),
            Down::Corner(offset) => top.wrapping_add(shift).wrapping_add(offset),
            Down::Footer(offset) => shift.wrapping_sub(bottom).wrapping_add(offset),
        };

        ctx.set_i32_at(rect + 4, y)?;
    }

    let mut column = ITEM_COLUMNS - 1;

    for powerup in (0..ITEM_COLUMNS).rev() {
        if !powerup_available(ctx, powerup)? {
            continue;
        }

        let rect = AppContext::ITEM_RECTS + powerup as usize * RECT_STRIDE;
        let x = drawable
            .wrapping_add(column.wrapping_mul(ITEM_PITCH))
            .wrapping_sub(right)
            .wrapping_add(ITEM_LEFT);

        ctx.set_i32_at(rect, x)?;
        ctx.set_i32_at(rect + 4, ITEM_TOP.wrapping_sub(shift))?;
        column -= 1;
    }

    let shown = ctx.u8_at(AppContext::OPTION_WINDOW)? != 0
        && ctx.i32_at(AppContext::OPTION_WINDOW_KIND)? == BATTLE_OPTION_WINDOW
        && ctx.u8_at(AppContext::UNIT_INFO_OVERLAY_OPEN)? == 0;

    if shown {
        option_window_build_alt(ctx)?;
    }

    let after = layout_snapshot(ctx)?;
    let centered = ops::div_2(after.drawable).wrapping_sub(ops::div_2(before.drawable));
    let edged = after
        .drawable
        .wrapping_sub(after.right)
        .wrapping_sub(before.drawable.wrapping_sub(before.right));
    let lowered = ops::div_2(after.design)
        .wrapping_sub(ops::div_2(before.design))
        .wrapping_sub(after.shift.wrapping_sub(before.shift))
        .wrapping_sub(after.pad.wrapping_sub(before.pad));
    let raised = before.shift.wrapping_sub(after.shift);
    let footed = after
        .shift
        .wrapping_sub(after.bottom)
        .wrapping_sub(before.shift.wrapping_sub(before.bottom));

    for dialog in ctx.dialogs.objects.values_mut() {
        if dialog.flags & 2 != 0 {
            continue;
        }

        dialog.x = dialog.x.wrapping_add(centered);

        if dialog.flags & 1 == 0 {
            dialog.y = dialog.y.wrapping_add(lowered);
        }
    }

    for (id, slide, lift) in SHIFTED {
        let Some(Some(button)) = ctx.buttons.buttons.get_mut(&id) else {
            continue;
        };
        let dx = match slide {
            Slide::Edge => edged,
            Slide::Center => centered,
        };
        let dy = match lift {
            Lift::Kept => 0,
            Lift::Raised => raised,
            Lift::Footer => footed,
            Lift::Centered => lowered,
        };

        button.x = button.x.wrapping_add(dx);
        button.y = button.y.wrapping_add(dy);

        if let Some(node) = button.node.as_mut() {
            node.x += dx as f32;
            node.y += dy as f32;
        }
    }

    Ok(())
}
