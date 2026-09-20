use crate::{Fault, ops};

pub fn std_stod(text: &[u8], idx: Option<&mut usize>) -> Result<f64, Fault> {
    let Some((value, overflow)) = ops::strtod(text) else {
        return Err(Fault::invalid_argument());
    };

    if overflow {
        return Err(Fault::out_of_range());
    }

    if let Some(idx) = idx {
        *idx = text.len();
    }

    Ok(value)
}
