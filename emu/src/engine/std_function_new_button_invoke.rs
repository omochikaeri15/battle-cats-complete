use crate::Fault;

use super::{AppContext, ButtonHandler};

pub fn std_function_new_button_invoke(
    ctx: &mut AppContext,
    handler: Option<ButtonHandler>,
    button: i32,
    event: i32,
) -> Result<(), Fault> {
    handler.ok_or(Fault::bad_function_call())?(ctx, button, event)
}
