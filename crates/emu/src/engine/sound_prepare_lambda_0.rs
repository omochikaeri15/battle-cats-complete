use crate::Fault;

use super::SoundManager;

pub fn sound_prepare_lambda_0(mgr: &mut SoundManager) -> Result<Option<usize>, Fault> {
    let mut best: Option<usize> = None;
    let mut slot = 0;

    while slot != mgr.sources.len() {
        if let Some(source) = mgr.sources[slot].clone() {
            if source.is_stopped() {
                mgr.sources[slot] = None;
            } else {
                let replace = match best {
                    None => true,
                    Some(held) => {
                        let priority = source.priority();
                        let rival = mgr.sources[held].as_ref().ok_or(Fault::null_pointer())?.priority();

                        priority > rival
                    }
                };

                if replace {
                    best = Some(slot);
                }
            }
        }

        slot += 1;
    }

    Ok(best)
}
