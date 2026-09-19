use crate::{Fault, operation};

const SITE: &str = "json_string_as_double";

pub fn json_string_as_double(text: &[u8]) -> Result<f64, Fault> {
    let (value, range) = operation::strtod(text).ok_or(Fault::InvalidArgument { site: SITE })?;

    if range {
        return Err(Fault::OutOfRange { site: SITE });
    }

    Ok(value)
}
