//! Identification of the game's regional distributions.

use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

/// Identifies one of the four regional distributions of the game client.
///
/// Each region ships its own asset packs, localization tables, and encryption
/// parameters. The variant selects which of those parameter sets applies when
/// decrypting or interpreting a file.
#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug, Deserialize, Serialize)]
pub enum Region {
    /// The Japanese distribution, treated by the engine as the default region.
    Ja,
    /// The global English distribution.
    En,
    /// The Taiwanese distribution.
    Tw,
    /// The Korean distribution.
    Ko,
}

/// Holds the static naming conventions associated with a single [`Region`].
///
/// The engine derives asset paths and package identifiers from short region
/// codes that do not always match one another. This structure exposes each of
/// those codes separately so a caller does not have to infer one from another.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct RegionMetadata {
    /// The lowercase code used inside asset filenames and directory names.
    pub internal_code: &'static str,
    /// The suffix appended to package names, which is empty for the Japanese region.
    pub package_suffix: &'static str,
    /// The human-readable name of the distribution.
    pub full_name: &'static str,
}

/// Indicates that a string did not correspond to any known [`Region`].
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
    /// Returns the static naming conventions associated with this region.
    ///
    /// # Returns
    /// A `RegionMetadata` value containing the internal asset code, the package
    /// name suffix, and the human-readable distribution name.
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

    /// Resolves a region from one of its recognized string codes.
    ///
    /// Matching is case-insensitive and accepts the alternate codes the engine
    /// uses in different contexts, including the `battlecats` package name for
    /// the Japanese region and the divergent `kr` code for the Korean region.
    ///
    /// # Arguments
    /// * `input_string` - The region code to resolve.
    ///
    /// # Returns
    /// A `Result` containing the matching `Region` on success, or a
    /// `ParseRegionError` if the string matches no known region.
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
