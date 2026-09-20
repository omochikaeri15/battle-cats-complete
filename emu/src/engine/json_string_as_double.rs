use crate::{Fault, operation};

pub fn json_string_as_double(text: &[u8]) -> Result<f64, Fault> {
    let (value, range) = operation::strtod(text).ok_or(Fault::invalid_argument())?;

    if range {
        return Err(Fault::out_of_range());
    }

    Ok(value)
}
