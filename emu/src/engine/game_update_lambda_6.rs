use crate::Fault;

use super::{draw_cannon, AppContext};

pub fn game_update_lambda_6(ctx: &mut AppContext) -> Result<(), Fault> {
    draw_cannon(ctx)
}
