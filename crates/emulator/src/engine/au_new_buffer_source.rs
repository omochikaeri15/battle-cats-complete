use std::rc::Rc;

use crate::Fault;

use super::{AppContext, AudioSource, SoundData};

pub fn au_new_buffer_source(ctx: &mut AppContext, data: &Rc<dyn SoundData>) -> Result<Option<Rc<dyn AudioSource>>, Fault> {
    Ok(ctx.audio().ok_or(Fault::host_missing())?.new_buffer_source(data))
}
