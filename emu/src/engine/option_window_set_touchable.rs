use crate::Fault;

use super::{AppContext, OptionPage, button_bank_find, new_button_set_touchable};

pub fn option_window_set_touchable(ctx: &mut AppContext, touchable: u8) -> Result<(), Fault> {
    let button = button_bank_find(&ctx.buttons, 0x3e8).ok_or(Fault::null_pointer())?;

    new_button_set_touchable(&mut ctx.buttons, button, touchable)?;

    let button = button_bank_find(&ctx.buttons, 0x3e9).ok_or(Fault::null_pointer())?;

    new_button_set_touchable(&mut ctx.buttons, button, touchable)?;

    let button = button_bank_find(&ctx.buttons, 0x3ea).ok_or(Fault::null_pointer())?;

    new_button_set_touchable(&mut ctx.buttons, button, touchable)?;

    let button = button_bank_find(&ctx.buttons, 0x3eb).ok_or(Fault::null_pointer())?;

    new_button_set_touchable(&mut ctx.buttons, button, touchable)?;

    let button = button_bank_find(&ctx.buttons, 0x3ed).ok_or(Fault::null_pointer())?;

    new_button_set_touchable(&mut ctx.buttons, button, touchable)?;

    let button = button_bank_find(&ctx.buttons, 0x3ee).ok_or(Fault::null_pointer())?;

    new_button_set_touchable(&mut ctx.buttons, button, touchable)?;

    let button = button_bank_find(&ctx.buttons, 0x3f0).ok_or(Fault::null_pointer())?;

    new_button_set_touchable(&mut ctx.buttons, button, touchable)?;

    if ctx.u8_at(AppContext::OPTION_WINDOW_PAGE + OptionPage::TALL)? != 0 {
        let button = button_bank_find(&ctx.buttons, 0x3ec).ok_or(Fault::null_pointer())?;

        new_button_set_touchable(&mut ctx.buttons, button, touchable)?;
    }

    Ok(())
}
