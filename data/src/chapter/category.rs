//! Identification of the stage chapter a map belongs to.

use serde::{Deserialize, Serialize};

/// Identifies one of the game's stage chapters.
///
/// The engine encodes the chapter in a map's filename prefix, but addresses the
/// same map numerically elsewhere. This sits between the two schemes.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Category {
    /// The Stories of Legend chapter.
    StoriesOfLegend,
    /// The recurring event stage chapter.
    EventStages,
    /// The collaboration event chapter.
    CollabStages,
    /// The Empire of Cats main story chapter.
    EmpireOfCats,
    /// The Into the Future main story chapter.
    IntoTheFuture,
    /// The Cats of the Cosmos main story chapter.
    CatsOfTheCosmos,
    /// The limited-time event chapter.
    ExtraStages,
    /// The Catclaw Dojo Hall of Initiates chapter.
    DojoHallOfInitiates,
    /// The Towers and Citadels chapter.
    TowersAndCitadels,
    /// The ranked Catclaw Dojo event chapter.
    DojoRankingEvents,
    /// The Challenge Battle chapter.
    ChallengeBattle,
    /// The Uncanny Legends chapter.
    UncannyLegends,
    /// The Catamin stage chapter.
    CataminStages,
    /// The Legend Quest chapter.
    LegendQuest,
    /// The Zombie Outbreak chapter overlaid on the main story maps.
    ZombieOutbreaks,
    /// The Gauntlet chapter.
    GauntletStages,
    /// The Enigma chapter.
    EnigmaStages,
    /// The collaboration Gauntlet chapter.
    CollabGauntletStages,
    /// The Aku Realms chapter.
    AkuRealms,
    /// The Behemoth Culling chapter.
    BehemothCulling,
    /// The Labyrinth chapter.
    Labyrinth,
    /// The Zero Legends chapter.
    ZeroLegends,
    /// The Otherworld Colosseum chapter.
    OtherworldColosseum,
    /// The Catclaw Championships chapter.
    CatclawChampionships,
    /// The Fever Rush chapter, whose stages fill a gauge from event points.
    FeverRush,
    /// A chapter prefix this parser does not recognize, carrying the raw prefix.
    Unknown(String),
}

impl Default for Category {
    fn default() -> Self {
        Self::Unknown(String::new())
    }
}

impl Category {
    /// Returns the filename prefix the engine uses for this chapter's map files.
    ///
    /// # Returns
    /// A `String` containing the prefix, which is empty for chapters whose files
    /// carry no prefix at all.
    pub fn map_prefix(&self) -> String {
        match self {
            Self::StoriesOfLegend      => "N".to_string(),
            Self::EventStages          => "S".to_string(),
            Self::CollabStages         => "C".to_string(),
            Self::EmpireOfCats         => "EC".to_string(),
            Self::IntoTheFuture        => "W".to_string(),
            Self::CatsOfTheCosmos      => "Space".to_string(),
            Self::ExtraStages          => "RE".to_string(),
            Self::DojoHallOfInitiates  => "T".to_string(),
            Self::TowersAndCitadels    => "V".to_string(),
            Self::DojoRankingEvents    => "R".to_string(),
            Self::ChallengeBattle      => "M".to_string(),
            Self::UncannyLegends       => "NA".to_string(),
            Self::CataminStages        => "B".to_string(),
            Self::LegendQuest          => "D".to_string(),
            Self::ZombieOutbreaks      => "Z".to_string(),
            Self::GauntletStages       => "A".to_string(),
            Self::EnigmaStages         => "H".to_string(),
            Self::CollabGauntletStages => "CA".to_string(),
            Self::AkuRealms            => "DM".to_string(),
            Self::BehemothCulling      => "Q".to_string(),
            Self::Labyrinth            => "L".to_string(),
            Self::ZeroLegends          => "ND".to_string(),
            Self::OtherworldColosseum  => "SR".to_string(),
            Self::CatclawChampionships => "G".to_string(),
            Self::FeverRush            => "PR".to_string(),
            Self::Unknown(prefix)      => prefix.clone(),
        }
    }

    /// Returns the filename prefix the engine uses for this chapter's artwork.
    ///
    /// Several chapters share their artwork with the chapter they extend, so
    /// this does not always agree with [`Category::map_prefix`].
    ///
    /// # Returns
    /// A `String` containing the artwork prefix.
    pub fn image_prefix(&self) -> String {
        self.map_prefix().to_lowercase()
    }

    /// Returns every filename prefix this chapter's stage files may carry.
    ///
    /// Stage prefixes do not always match a chapter's map prefix, and some
    /// chapters accept an extra prefix for restricted variants.
    ///
    /// # Returns
    /// A `Vec<String>` containing the candidate prefixes, most specific first.
    pub fn stage_prefix(&self) -> Vec<String> {
        let base = self.map_prefix();
        let mut prefixes = vec![base.clone()];

        match self {
            Self::StoriesOfLegend      => prefixes.push("RN".to_string()),
            Self::EventStages          => prefixes.push("RS".to_string()),
            Self::CollabStages         => prefixes.push("RC".to_string()),
            Self::EmpireOfCats         => prefixes.push("".to_string()),
            Self::ExtraStages          => prefixes.insert(0, "EX".to_string()),
            Self::DojoHallOfInitiates  => prefixes.push("RT".to_string()),
            Self::TowersAndCitadels    => prefixes.push("RV".to_string()),
            Self::DojoRankingEvents    => prefixes.push("RR".to_string()),
            Self::ChallengeBattle      => prefixes.push("RM".to_string()),
            Self::UncannyLegends       => prefixes.push("RNA".to_string()),
            Self::CataminStages        => prefixes.push("RB".to_string()),
            Self::GauntletStages       => prefixes.push("RA".to_string()),
            Self::EnigmaStages         => prefixes.push("RH".to_string()),
            Self::CollabGauntletStages => prefixes.push("RCA".to_string()),
            Self::AkuRealms            => prefixes.push("DM".to_string()),
            Self::BehemothCulling      => prefixes.push("RQ".to_string()),
            Self::ZeroLegends          => prefixes.push("RND".to_string()),
            Self::OtherworldColosseum  => prefixes.push("RSR".to_string()),
            Self::FeverRush            => prefixes.push("RPR".to_string()),
            Self::Unknown(prefix)      => {
                let upper = prefix.to_uppercase();
                if upper.starts_with('R') && upper.len() > 1 {
                    prefixes.push(upper[1..].to_string());
                }
            },
            _ => {}
        }

        prefixes
    }

    /// Resolves a chapter from the filename prefix its files carry.
    ///
    /// Matching is case-insensitive and accepts the restricted-variant prefixes
    /// the engine uses alongside the ordinary ones. The empty prefix resolves to
    /// the first main story chapter, whose files carry none.
    ///
    /// # Arguments
    /// * `prefix` - The filename prefix to resolve.
    ///
    /// # Returns
    /// The matching `Category`, or `Category::Unknown` carrying the supplied
    /// prefix when it matches no known chapter.
    pub fn from_prefix(prefix: &str) -> Self {
        match prefix.to_uppercase().as_str() {
            "N"     | "RN"  => Self::StoriesOfLegend,
            "S"     | "RS"  => Self::EventStages,
            "C"     | "RC"  => Self::CollabStages,
            "EC"    | ""    => Self::EmpireOfCats,
            "W"             => Self::IntoTheFuture,
            "SPACE"         => Self::CatsOfTheCosmos,
            "RE"    | "EX"  => Self::ExtraStages,
            "T"     | "RT"  => Self::DojoHallOfInitiates,
            "V"     | "RV"  => Self::TowersAndCitadels,
            "R"     | "RR"  => Self::DojoRankingEvents,
            "M"     | "RM"  => Self::ChallengeBattle,
            "NA"    | "RNA" => Self::UncannyLegends,
            "B"     | "RB"  => Self::CataminStages,
            "D"             => Self::LegendQuest,
            "Z"             => Self::ZombieOutbreaks,
            "A"     | "RA"  => Self::GauntletStages,
            "H"     | "RH"  => Self::EnigmaStages,
            "CA"    | "RCA" => Self::CollabGauntletStages,
            "DM"            => Self::AkuRealms,
            "Q"     | "RQ"  => Self::BehemothCulling,
            "L"             => Self::Labyrinth,
            "ND"    | "RND" => Self::ZeroLegends,
            "SR"    | "RSR" => Self::OtherworldColosseum,
            "G"             => Self::CatclawChampionships,
            "PR"    | "RPR" => Self::FeverRush,
            _               => Self::Unknown(prefix.to_string()),
        }
    }

    /// Returns the numeric base this chapter's map identifiers are offset from.
    ///
    /// # Returns
    /// An `Option` containing the chapter's base, or `None` for the main story
    /// chapters and unrecognized chapters, which are not addressed through this
    /// scheme.
    pub fn base_id(&self) -> Option<u32> {
        match self {
            Self::StoriesOfLegend      => Some(0),
            Self::EventStages          => Some(1),
            Self::CollabStages         => Some(2),
            Self::EmpireOfCats         => None,
            Self::IntoTheFuture        => None,
            Self::CatsOfTheCosmos      => None,
            Self::ExtraStages          => Some(4),
            Self::DojoHallOfInitiates  => Some(6),
            Self::TowersAndCitadels    => Some(7),
            Self::DojoRankingEvents    => Some(11),
            Self::ChallengeBattle      => Some(12),
            Self::UncannyLegends       => Some(13),
            Self::CataminStages        => Some(14),
            Self::LegendQuest          => Some(16),
            Self::ZombieOutbreaks      => None,
            Self::GauntletStages       => Some(24),
            Self::EnigmaStages         => Some(25),
            Self::CollabGauntletStages => Some(27),
            Self::AkuRealms            => Some(30),
            Self::BehemothCulling      => Some(31),
            Self::Labyrinth            => Some(33),
            Self::ZeroLegends          => Some(34),
            Self::OtherworldColosseum  => Some(36),
            Self::CatclawChampionships => Some(37),
            Self::FeverRush            => Some(39),
            Self::Unknown(_)           => None,
        }
    }

    /// Converts a map's chapter-local identifier into its global identifier.
    ///
    /// Cross-chapter tables combine the chapter's base with the map's own number,
    /// whereas a chapter's own files number its maps from zero.
    ///
    /// # Arguments
    /// * `local_map_id` - The map's number within this chapter.
    ///
    /// # Returns
    /// An `Option` containing the global identifier, or `None` when the chapter
    /// is not addressed through this scheme.
    pub fn global_map_id(&self, local_map_id: u32) -> Option<u32> {
        self.base_id().map(|base| (base * 1000) + local_map_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extra_stages_stage_prefix_includes_ex() {
        assert!(Category::ExtraStages.stage_prefix().contains(&"EX".to_string()));
    }

    #[test]
    fn extra_stages_from_prefix_accepts_re_and_ex() {
        assert_eq!(Category::from_prefix("RE"), Category::ExtraStages);
        assert_eq!(Category::from_prefix("EX"), Category::ExtraStages);
    }

    #[test]
    fn extra_stages_global_map_id_matches_map_option_scheme() {
        assert_eq!(Category::ExtraStages.global_map_id(81), Some(4081));
    }
}