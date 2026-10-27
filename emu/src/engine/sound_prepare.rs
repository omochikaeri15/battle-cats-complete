use crate::{Fault, ops};

use super::{AppContext, au_active_sources, au_source_capacity, sound_manager, sound_prepare_lambda_0};

pub fn sound_prepare(ctx: &mut AppContext, priority: i32) -> Result<bool, Fault> {
    let mut found = sound_prepare_lambda_0(sound_manager(ctx)?)?;

    loop {
        let active = au_active_sources(ctx)?;
        let limit = ops::cvttss2si(au_source_capacity(ctx)? as f32 * 0.8f32);
        let Some(index) = found.filter(|_| active > limit) else {
            break;
        };
        let mgr = sound_manager(ctx)?;
        let source = mgr.sources.get(index).cloned().flatten().ok_or(Fault::null_pointer())?;

        if source.priority() < 0 {
            break;
        }

        source.stop();
        mgr.sources[index] = None;
        found = sound_prepare_lambda_0(mgr)?;
    }

    let mut ready = true;

    if let Some(index) = found {
        let active = au_active_sources(ctx)?;
        let limit = ops::cvttss2si(au_source_capacity(ctx)? as f32 * 0.8f32);

        if active >= limit {
            let mgr = sound_manager(ctx)?;
            let source = mgr.sources.get(index).cloned().flatten().ok_or(Fault::null_pointer())?;

            if source.priority() < priority {
                ready = false;
            } else {
                source.stop();
                mgr.sources[index] = None;
            }
        }
    }

    Ok(ready)
}
