use std::fmt;

use serde::{Deserialize, Serialize};

use crate::common::columns;
use crate::common::file::{self, Separator};
use crate::common::columns::Column;

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum ScatCpuSettingError {
    EmptyFile,
}

impl fmt::Display for ScatCpuSettingError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyFile => write!(
                formatter,
                "The provided byte slice contained no valid Super Cat CPU setting data."
            ),
        }
    }
}

impl std::error::Error for ScatCpuSettingError {}

#[derive(Default, Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ScatCpuSetting {
    pub unknown_1: u32,
    pub super_cpu_daily_limit: u32,
    pub super_cpu_consume_amount: u32,
}

impl ScatCpuSetting {
    pub const COLUMNS: &'static [Column<Self>] = columns::columns! {
        unknown_1                : 0;
        super_cpu_daily_limit    : 1;
        super_cpu_consume_amount : 2;
    };

    pub fn parse<B: AsRef<[u8]>>(bytes: B, separator: Option<Separator>) -> Result<Self, ScatCpuSettingError> {
        parse_inner(bytes.as_ref(), separator)
    }
}

fn parse_inner(bytes: &[u8], separator: Option<Separator>) -> Result<ScatCpuSetting, ScatCpuSettingError> {
    let file_content = file::scrub(bytes);
    let separator_char = file::resolve(separator, &file_content);

    let mut setting = ScatCpuSetting::default();
    let mut has_content = false;

    for file_line in file_content.lines() {
        let mut clean_line = file_line;

        if let Some((before_comment, _)) = file_line.split_once("//") {
            clean_line = before_comment;
        }

        let trimmed_line = clean_line.trim();
        if trimmed_line.is_empty() {
            continue;
        }

        has_content = true;

        let parts: Vec<&str> = trimmed_line.split(separator_char).collect();
        columns::apply(&parts, ScatCpuSetting::COLUMNS, &mut setting);
        break;
    }

    if !has_content {
        return Err(ScatCpuSettingError::EmptyFile);
    }

    Ok(setting)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_column_reaches_a_field_of_its_own() {
        columns::assert_one_field_per_column(ScatCpuSetting::COLUMNS);
    }
}
