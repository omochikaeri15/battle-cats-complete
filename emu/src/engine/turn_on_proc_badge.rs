use crate::Fault;

use super::{get_proc_badge, set_proc_badge, AppContext};

pub fn turn_on_proc_badge(ctx: &mut AppContext, faction: i32, slot: i32, badge: i32) -> Result<(), Fault> {
    if get_proc_badge(ctx, faction, slot, 0)? == badge {
        return Ok(());
    }

    if get_proc_badge(ctx, faction, slot, 1)? == badge {
        return Ok(());
    }

    if get_proc_badge(ctx, faction, slot, 2)? == badge {
        return Ok(());
    }

    if get_proc_badge(ctx, faction, slot, 3)? == badge {
        return Ok(());
    }

    if get_proc_badge(ctx, faction, slot, 4)? == badge {
        return Ok(());
    }

    let mut index = 0;

    if get_proc_badge(ctx, faction, slot, 0)? != 0 {
        index = 1;

        if get_proc_badge(ctx, faction, slot, 1)? != 0 {
            index = 2;

            if get_proc_badge(ctx, faction, slot, 2)? != 0 {
                index = 3;

                if get_proc_badge(ctx, faction, slot, 3)? != 0 {
                    index = 4;

                    if get_proc_badge(ctx, faction, slot, 4)? != 0 {
                        return Ok(());
                    }
                }
            }
        }
    }

    set_proc_badge(ctx, faction, slot, index, badge)
}
