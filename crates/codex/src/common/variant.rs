use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug, Deserialize, Serialize)]
pub enum Region {
    Ja,
    En,
    Tw,
    Ko,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct RegionMetadata {
    pub internal_code: &'static str,
    pub package_suffix: &'static str,
    pub full_name: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub struct ParseRegionError;

impl fmt::Display for ParseRegionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "The provided string did not match any known region code.")
    }
}

impl std::error::Error for ParseRegionError {}

impl Region {
    pub const fn metadata(&self) -> RegionMetadata {
        match self {
            Region::Ja => RegionMetadata {
                internal_code: "ja",
                package_suffix: "",
                full_name: "Japan",
            },
            Region::En => RegionMetadata {
                internal_code: "en",
                package_suffix: "en",
                full_name: "Global",
            },
            Region::Tw => RegionMetadata {
                internal_code: "tw",
                package_suffix: "tw",
                full_name: "Taiwan",
            },
            Region::Ko => RegionMetadata {
                internal_code: "ko",
                package_suffix: "kr",
                full_name: "Korea",
            },
        }
    }
}

impl FromStr for Region {
    type Err = ParseRegionError;

    fn from_str(input_string: &str) -> Result<Self, Self::Err> {
        match input_string.to_lowercase().as_str() {
            "ja" | "jp" | "battlecats" => Ok(Region::Ja),
            "en" => Ok(Region::En),
            "tw" => Ok(Region::Tw),
            "ko" | "kr" => Ok(Region::Ko),
            _ => Err(ParseRegionError),
        }
    }
}
