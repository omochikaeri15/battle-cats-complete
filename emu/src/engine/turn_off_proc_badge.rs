use crate::Fault;

use super::{get_proc_badge, set_proc_badge, AppContext};

pub fn turn_off_proc_badge(ctx: &mut AppContext, faction: i32, slot: i32, badge: i32) -> Result<(), Fault> {
    if get_proc_badge(ctx, faction, slot, 0)? == badge {
        set_proc_badge(ctx, faction, slot, 0, 0)?;
    }

    if get_proc_badge(ctx, faction, slot, 1)? == badge {
        set_proc_badge(ctx, faction, slot, 1, 0)?;
    }

    if get_proc_badge(ctx, faction, slot, 2)? == badge {
        set_proc_badge(ctx, faction, slot, 2, 0)?;
    }

    if get_proc_badge(ctx, faction, slot, 3)? == badge {
        set_proc_badge(ctx, faction, slot, 3, 0)?;
    }

    if get_proc_badge(ctx, faction, slot, 4)? == badge {
        set_proc_badge(ctx, faction, slot, 4, 0)?;
    }

    Ok(())
}
