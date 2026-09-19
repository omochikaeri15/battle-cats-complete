use crate::Fault;

use super::{AppContext, draw_cannon};

pub fn game_update_lambda_6(ctx: &mut AppContext) -> Result<(), Fault> {
    draw_cannon(ctx)
}
