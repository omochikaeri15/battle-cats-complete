use crate::Fault;

use super::{AppContext, ButtonHandler};

const SITE: &str = "std_function_new_button_invoke";

pub fn std_function_new_button_invoke(
    ctx: &mut AppContext,
    handler: Option<ButtonHandler>,
    button: i32,
    event: i32,
) -> Result<(), Fault> {
    handler.ok_or(Fault::BadFunctionCall { site: SITE })?(ctx, button, event)
}
