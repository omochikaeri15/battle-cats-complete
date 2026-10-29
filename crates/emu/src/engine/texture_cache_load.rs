use std::rc::Rc;

use crate::Fault;

use super::{AppContext, Imgcut, texture_load};

pub fn texture_cache_load(
    ctx: &mut AppContext,
    png: &[u8],
    cut: &[u8],
    flag: i32,
) -> Result<Option<Rc<Imgcut>>, Fault> {
    if let Some(sheet) = ctx
        .texture_cache
        .get(&[png, cut].concat())
        .and_then(|entry| entry.upgrade())
    {
        return Ok(Some(sheet));
    }

    let mut sheet = Imgcut::default();

    if !texture_load(ctx, &mut sheet, png, cut, flag)? {
        return Ok(None);
    }

    let sheet = Rc::new(sheet);

    ctx.texture_cache
        .insert([png, cut].concat(), Rc::downgrade(&sheet));

    Ok(Some(sheet))
}
