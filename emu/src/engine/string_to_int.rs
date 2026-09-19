use crate::{operation, Fault};

const SITE: &str = "string_to_int";

pub fn string_to_int(text: &[u8]) -> Result<i32, Fault> {
    let parsed = operation::strtol(text, 10);

    if parsed.end == 0 {
        return Err(Fault::InvalidArgument { site: SITE });
    }

    if parsed.overflow || parsed.value < i32::MIN as i64 || parsed.value > i32::MAX as i64 {
        return Err(Fault::OutOfRange { site: SITE });
    }

    Ok(parsed.value as i32)
}
