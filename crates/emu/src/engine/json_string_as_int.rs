use crate::{Fault, ops};

pub fn json_string_as_int(text: &[u8]) -> Result<i64, Fault> {
    let parsed = ops::strtol(text, 10);

    if parsed.end == 0 {
        return Err(Fault::invalid_argument());
    }

    if parsed.overflow {
        return Err(Fault::out_of_range());
    }

    Ok(parsed.value)
}
