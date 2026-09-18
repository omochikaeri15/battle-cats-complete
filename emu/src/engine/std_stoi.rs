use crate::{operation, Fault};

pub fn std_stoi(text: &[u8], idx: Option<&mut usize>, base: i32) -> Result<i32, Fault> {
    let parsed = operation::strtol(text, base);

    if parsed.overflow {
        return Err(Fault::OutOfRange { site: "std_stoi" });
    }

    if parsed.end == 0 {
        return Err(Fault::InvalidArgument { site: "std_stoi" });
    }

    if let Some(idx) = idx {
        *idx = parsed.end;
    }

    if parsed.value as i32 as i64 != parsed.value {
        return Err(Fault::OutOfRange { site: "std_stoi" });
    }

    Ok(parsed.value as i32)
}
