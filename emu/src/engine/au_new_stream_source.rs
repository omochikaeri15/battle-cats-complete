use std::rc::Rc;

use crate::Fault;

use super::{AppContext, AudioSource};

pub fn au_new_stream_source(ctx: &mut AppContext, path: &[u8], source_type: i32) -> Result<Option<Rc<dyn AudioSource>>, Fault> {
    Ok(ctx.audio().ok_or(Fault::host_missing())?.new_stream_source(path, source_type))
}
