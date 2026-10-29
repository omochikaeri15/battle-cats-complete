use std::rc::Rc;

use crate::Fault;

use super::{AppContext, SoundData};

pub fn snd_load_sound_data(ctx: &mut AppContext, path: &[u8]) -> Result<Option<Rc<dyn SoundData>>, Fault> {
    Ok(ctx.audio().ok_or(Fault::host_missing())?.load_sound_data(path))
}
