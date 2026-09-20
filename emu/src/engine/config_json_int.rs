use crate::Fault;

use super::AppContext;

pub fn config_json_int(
    ctx: &mut AppContext,
    section: &[u8],
    name: &[u8],
    min: i32,
    max: i32,
) -> Result<i32, Fault> {
    let value = ctx
        .platform()
        .ok_or(Fault::host_missing())?
        .config_json_int(section, name);
    let Some(value) = value else {
        return Ok(min);
    };
    let mut result = min;

    if value > result {
        result = value;
    }

    if value > max {
        result = max;
    }

    Ok(result)
}
