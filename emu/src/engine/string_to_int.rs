use crate::{Fault, ops};

pub fn string_to_int(text: &[u8]) -> Result<i32, Fault> {
    let parsed = ops::strtol(text, 10);

    if parsed.end == 0 {
        return Err(Fault::invalid_argument());
    }

    if parsed.overflow || parsed.value < i32::MIN as i64 || parsed.value > i32::MAX as i64 {
        return Err(Fault::out_of_range());
    }

    Ok(parsed.value as i32)
}
