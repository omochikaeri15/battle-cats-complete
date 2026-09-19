use crate::{Fault, operation};

const SITE: &str = "json_string_as_int";

pub fn json_string_as_int(text: &[u8]) -> Result<i64, Fault> {
    let parsed = operation::strtol(text, 10);

    if parsed.end == 0 {
        return Err(Fault::InvalidArgument { site: SITE });
    }

    if parsed.overflow {
        return Err(Fault::OutOfRange { site: SITE });
    }

    Ok(parsed.value)
}
