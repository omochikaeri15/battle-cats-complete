use crate::{Fault, ops};

pub fn std_stoi(text: &[u8], idx: Option<&mut usize>, base: i32) -> Result<i32, Fault> {
    let parsed = ops::strtol(text, base);

    if parsed.overflow {
        return Err(Fault::out_of_range());
    }

    if parsed.end == 0 {
        return Err(Fault::invalid_argument());
    }

    if let Some(idx) = idx {
        *idx = parsed.end;
    }

    if parsed.value as i32 as i64 != parsed.value {
        return Err(Fault::out_of_range());
    }

    Ok(parsed.value as i32)
}
