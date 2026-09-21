use crate::{
    Fault,
    engine::{AppContext, dialog_button_rect, get_drawable_width, get_setting},
    ops,
};

pub const LEAVE_BUTTON: i32 = 0x3ed;
pub const CONFIRM_BUTTON: i32 = 0;

const RECT_STRIDE: usize = 0x10;
const DECK_ROW: i32 = 5;
const DECK_WIDTH: i32 = 0x6e;
const DECK_HEIGHT: i32 = 0x58;
const DESIGN_WIDTH: i32 = 0x3c0;
const TWO_LINE_SHIFT: i32 = 0x5a;
const FIELD_Y: i32 = 0x12c;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Spot {
    Deck(i32),
    Item(i32),
    Worker,
    Cannon,
    Button(i32),
    DialogButton(i32),
    Field,
}

fn rect_center(ctx: &AppContext, rect: usize) -> Result<Option<(i32, i32)>, Fault> {
    let (x, y, width, height) = (ctx.i32_at(rect)?, ctx.i32_at(rect + 4)?, ctx.i32_at(rect + 8)?, ctx.i32_at(rect + 0xc)?);

    Ok((width > 0 && height > 0).then(|| (x.wrapping_add(width / 2), y.wrapping_add(height / 2))))
}

pub fn deck_row_hidden(ctx: &AppContext, slot: i32) -> Result<bool, Fault> {
    if ctx.u8_at(AppContext::DECK_TWO_LINES)? != 0 {
        return Ok(false);
    }

    Ok(ctx.i32_at(AppContext::DECK_ROW_SHOWN)? != slot / DECK_ROW)
}

pub fn deck_row_swapping(ctx: &AppContext) -> Result<bool, Fault> {
    Ok(ctx.u8_at(AppContext::DECK_ROW_SWAPPING)? != 0)
}

pub fn start_deck_row_swap(ctx: &mut AppContext) -> Result<(), Fault> {
    if deck_row_swapping(ctx)? {
        return Ok(());
    }

    ctx.set_i32_at(AppContext::DECK_ROW_SWAP_DIRECTION, 1)?;
    ctx.set_i32_at(AppContext::DECK_ROW_SWAP_TARGET, 1)?;
    ctx.set_block_at::<1>(AppContext::DECK_ROW_SWAPPING, [1])
}

pub fn option_menu_open(ctx: &AppContext) -> Result<bool, Fault> {
    Ok(ctx.u8_at(AppContext::OPTION_MENU_IS_OPEN)? != 0)
}

pub fn dialog_open(ctx: &AppContext) -> bool {
    !ctx.dialogs.active.is_empty()
}

pub fn spot_center(ctx: &mut AppContext, spot: Spot) -> Result<Option<(i32, i32)>, Fault> {
    match spot {
        Spot::Worker => rect_center(ctx, AppContext::WORKER_RECT),
        Spot::Cannon => rect_center(ctx, AppContext::CANNON_RECT),
        Spot::Item(item) => rect_center(ctx, AppContext::ITEM_RECTS + item.max(0) as usize * RECT_STRIDE),
        Spot::Field => Ok(Some((get_drawable_width(ctx)? / 2, FIELD_Y))),
        Spot::Deck(slot) => {
            if deck_row_hidden(ctx, slot)? {
                return Ok(None);
            }

            let Some(column) = usize::try_from(slot).ok().and_then(|slot| ctx.deck_button_x.get(slot)).copied() else {
                return Ok(None);
            };
            let raised = ctx.u8_at(AppContext::DECK_TWO_LINES)? != 0 && slot < DECK_ROW;
            let shift = if raised { get_setting(&ctx.settings, b"battle_slot_2lines_line", TWO_LINE_SHIFT)?.wrapping_neg() } else { 0 };
            let x = ops::cvttsd2si(f64::from(get_drawable_width(ctx)?.wrapping_sub(DESIGN_WIDTH)) * 0.5 + f64::from(column));
            let y = ctx.i32_at(AppContext::LETTERBOX_SHIFT)?.wrapping_add(ctx.deck_bar_base_y).wrapping_add(shift).wrapping_sub(1);

            Ok(Some((x.wrapping_add(DECK_WIDTH / 2), y.wrapping_add(DECK_HEIGHT / 2))))
        }
        Spot::Button(id) => Ok(ctx.buttons.buttons.get(&id).and_then(|held| held.as_deref()).filter(|button| button.enabled != 0).map(|button| {
            (
                button.x.wrapping_add(button.offset_x).wrapping_add(button.width / 2),
                button.y.wrapping_add(button.offset_y).wrapping_add(button.height / 2),
            )
        })),
        Spot::DialogButton(button) => Ok(ctx.dialogs.active.last().and_then(|id| ctx.dialogs.objects.get(id)).map(|dialog| {
            let [x, y, width, height] = dialog_button_rect(dialog, button);

            (x.wrapping_add(width / 2), y.wrapping_add(height / 2))
        })),
    }
}

pub fn queue_spot_move(ctx: &mut AppContext, x: i32, y: i32) -> Result<(), Fault> {
    ctx.set_i32_at(AppContext::TOUCH_PENDING_X, x)?;
    ctx.set_i32_at(AppContext::TOUCH_PENDING_Y, y)
}

pub fn queue_spot_press(ctx: &mut AppContext, x: i32, y: i32) -> Result<(), Fault> {
    queue_spot_move(ctx, x, y)?;
    ctx.set_block_at::<1>(AppContext::TOUCH_PENDING_BEGAN, [1])
}

pub fn queue_back(ctx: &mut AppContext, down: bool) -> Result<(), Fault> {
    ctx.set_block_at::<1>(AppContext::BACK_PRESSED, [u8::from(down)])
}
