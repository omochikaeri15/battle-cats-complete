use std::error;
use std::fmt;

use crate::cat::{
    AssembleError, LevelError, NyancomboDataError, NyancomboError, NyancomboFilterError,
    NyancomboParamError, SkillAcquisitionError, SkillDescriptionsError, SkillLevelError,
    UnitBuyError, UnitEvolveError, UnitExplanationError,
};
use crate::chapter::map::{
    DropItemError, ExOptionError, LockSkipDataError, MapNameError, MapOptionError,
    ScoreBonusMapError, SpecialRulesMapError, SpecialRulesMapOptionError,
};
use crate::chapter::stage::{
    BattlegroundError, CertificationPresetError, CharaGroupError, DifficultyLevelError,
    DropCharaError, FixedFormationError, MapStageDataError, ScatCpuSettingError, StageNameError,
    StageOptionError,
};
use crate::chapter::treasure::{TreasureDataError, TreasureTextError};
use crate::combat::{DictionaryIndexError, EntityError, GlossaryError, TraitLabelError};
use crate::files::{GatyaItemBuyError, GatyaItemNameError, LocalizableError, ParamError};
use crate::enemy::{EnemyNameError, EnemyPictureBookError};

/// A single error type spanning every parser in the crate.
///
/// Aggregating a unit or a stage means calling parsers that each return their
/// own error type, which makes the `?` operator unusable across them. This enum
/// converts from all of those, so a caller can propagate any of them through one
/// signature rather than mapping each in turn.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum Error {
    /// A unit's combat statistic file could not be parsed.
    Entity(EntityError),
    /// A dictionary glossary could not be parsed.
    Glossary(GlossaryError),
    /// The Cat dictionary's ability table could not be parsed.
    DictionaryIndex(DictionaryIndexError),
    /// The orb trait label table could not be parsed.
    TraitLabel(TraitLabelError),
    /// A unit could not be aggregated from its sources.
    Assemble(AssembleError),
    /// The unit progression table could not be parsed.
    UnitBuy(UnitBuyError),
    /// The unit combo table could not be parsed.
    NyancomboData(NyancomboDataError),
    /// A localized combo text table could not be parsed.
    Nyancombo(NyancomboError),
    /// The combo magnitude table could not be parsed.
    NyancomboParam(NyancomboParamError),
    /// The combo category tab table could not be parsed.
    NyancomboFilter(NyancomboFilterError),
    /// The evolution text table could not be parsed.
    UnitEvolve(UnitEvolveError),
    /// A unit explanation file could not be parsed.
    UnitExplanation(UnitExplanationError),
    /// The level growth table could not be parsed.
    Level(LevelError),
    /// The talent acquisition table could not be parsed.
    SkillAcquisition(SkillAcquisitionError),
    /// The talent cost table could not be parsed.
    SkillLevel(SkillLevelError),
    /// The skill description table could not be parsed.
    SkillDescriptions(SkillDescriptionsError),
    /// The enemy terminology table could not be parsed.
    EnemyName(EnemyNameError),
    /// The enemy picture book table could not be parsed.
    EnemyPictureBook(EnemyPictureBookError),
    /// A stage layout file could not be parsed.
    Battleground(BattlegroundError),
    /// A fixed lineup preset could not be parsed.
    CertificationPreset(CertificationPresetError),
    /// The unit restriction group table could not be parsed.
    CharaGroup(CharaGroupError),
    /// The stage difficulty rating table could not be parsed.
    DifficultyLevel(DifficultyLevelError),
    /// The unit unlock drop table could not be parsed.
    DropChara(DropCharaError),
    /// The fixed lineup assignment table could not be parsed.
    FixedFormation(FixedFormationError),
    /// A map's stage metadata file could not be parsed.
    MapStageData(MapStageDataError),
    /// The automatic play settings file could not be parsed.
    ScatCpuSetting(ScatCpuSettingError),
    /// The localized stage name table could not be parsed.
    StageName(StageNameError),
    /// The stage lineup restriction table could not be parsed.
    StageOption(StageOptionError),
    /// A chapter's treasure set table could not be parsed.
    TreasureData(TreasureDataError),
    /// A localized treasure text table could not be parsed.
    TreasureText(TreasureTextError),
    /// The map drop table could not be parsed.
    DropItem(DropItemError),
    /// The EX map link table could not be parsed.
    ExOption(ExOptionError),
    /// The stage skip exclusion table could not be parsed.
    LockSkipData(LockSkipDataError),
    /// The localized map name table could not be parsed.
    MapName(MapNameError),
    /// The map option table could not be parsed.
    MapOption(MapOptionError),
    /// The score bonus document could not be parsed.
    ScoreBonusMap(ScoreBonusMapError),
    /// The map special rule document could not be parsed.
    SpecialRulesMap(SpecialRulesMapError),
    /// The special rule option document could not be parsed.
    SpecialRulesMapOption(SpecialRulesMapOptionError),
    /// A localization dictionary could not be parsed.
    Localizable(LocalizableError),
    /// A parameter file could not be parsed.
    Param(ParamError),
    /// The item catalogue could not be parsed.
    GatyaItemBuy(GatyaItemBuyError),
    /// A localized item text table could not be parsed.
    GatyaItemName(GatyaItemNameError),
    /// A unit's rig or animation data could not be parsed.
    #[cfg(feature = "graphics")]
    Rig(crate::graphics::rig::RigError),
    /// Resolved geometry could not be mapped back to the parts that drew it.
    #[cfg(feature = "graphics")]
    Part(crate::graphics::tools::part::PartError),
    /// An asset pack chunk or manifest could not be processed.
    #[cfg(feature = "pack")]
    Pack(crate::pack::cryptology::PackError),
    /// A Battle Cats Ultimate archive could not be processed.
    #[cfg(feature = "bcu")]
    Bcu(crate::bcu::cryptology::Error),
}

impl Error {
    fn as_source(&self) -> &(dyn error::Error + 'static) {
        match self {
            Self::Entity(source) => source,
            Self::Glossary(source) => source,
            Self::DictionaryIndex(source) => source,
            Self::TraitLabel(source) => source,
            Self::Assemble(source) => source,
            Self::UnitBuy(source) => source,
            Self::NyancomboData(source) => source,
            Self::Nyancombo(source) => source,
            Self::NyancomboParam(source) => source,
            Self::NyancomboFilter(source) => source,
            Self::UnitEvolve(source) => source,
            Self::UnitExplanation(source) => source,
            Self::Level(source) => source,
            Self::SkillAcquisition(source) => source,
            Self::SkillLevel(source) => source,
            Self::SkillDescriptions(source) => source,
            Self::EnemyName(source) => source,
            Self::EnemyPictureBook(source) => source,
            Self::Battleground(source) => source,
            Self::CertificationPreset(source) => source,
            Self::CharaGroup(source) => source,
            Self::DifficultyLevel(source) => source,
            Self::DropChara(source) => source,
            Self::FixedFormation(source) => source,
            Self::MapStageData(source) => source,
            Self::ScatCpuSetting(source) => source,
            Self::StageName(source) => source,
            Self::StageOption(source) => source,
            Self::TreasureData(source) => source,
            Self::TreasureText(source) => source,
            Self::DropItem(source) => source,
            Self::ExOption(source) => source,
            Self::LockSkipData(source) => source,
            Self::MapName(source) => source,
            Self::MapOption(source) => source,
            Self::ScoreBonusMap(source) => source,
            Self::SpecialRulesMap(source) => source,
            Self::SpecialRulesMapOption(source) => source,
            Self::Localizable(source) => source,
            Self::Param(source) => source,
            Self::GatyaItemBuy(source) => source,
            Self::GatyaItemName(source) => source,
            #[cfg(feature = "graphics")]
            Self::Rig(source) => source,
            #[cfg(feature = "graphics")]
            Self::Part(source) => source,
            #[cfg(feature = "pack")]
            Self::Pack(source) => source,
            #[cfg(feature = "bcu")]
            Self::Bcu(source) => source,
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Display::fmt(self.as_source(), f)
    }
}

impl error::Error for Error {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        Some(self.as_source())
    }
}

impl From<EntityError> for Error {
    fn from(source: EntityError) -> Self {
        Self::Entity(source)
    }
}

impl From<DictionaryIndexError> for Error {
    fn from(source: DictionaryIndexError) -> Self {
        Self::DictionaryIndex(source)
    }
}

impl From<GlossaryError> for Error {
    fn from(source: GlossaryError) -> Self {
        Self::Glossary(source)
    }
}

impl From<TraitLabelError> for Error {
    fn from(source: TraitLabelError) -> Self {
        Self::TraitLabel(source)
    }
}

impl From<AssembleError> for Error {
    fn from(source: AssembleError) -> Self {
        Self::Assemble(source)
    }
}

impl From<TreasureDataError> for Error {
    fn from(source: TreasureDataError) -> Self {
        Self::TreasureData(source)
    }
}

impl From<TreasureTextError> for Error {
    fn from(source: TreasureTextError) -> Self {
        Self::TreasureText(source)
    }
}

impl From<UnitBuyError> for Error {
    fn from(source: UnitBuyError) -> Self {
        Self::UnitBuy(source)
    }
}

impl From<NyancomboDataError> for Error {
    fn from(source: NyancomboDataError) -> Self {
        Self::NyancomboData(source)
    }
}

impl From<NyancomboError> for Error {
    fn from(source: NyancomboError) -> Self {
        Self::Nyancombo(source)
    }
}

impl From<NyancomboParamError> for Error {
    fn from(source: NyancomboParamError) -> Self {
        Self::NyancomboParam(source)
    }
}

impl From<NyancomboFilterError> for Error {
    fn from(source: NyancomboFilterError) -> Self {
        Self::NyancomboFilter(source)
    }
}

impl From<UnitEvolveError> for Error {
    fn from(source: UnitEvolveError) -> Self {
        Self::UnitEvolve(source)
    }
}

impl From<UnitExplanationError> for Error {
    fn from(source: UnitExplanationError) -> Self {
        Self::UnitExplanation(source)
    }
}

impl From<LevelError> for Error {
    fn from(source: LevelError) -> Self {
        Self::Level(source)
    }
}

impl From<SkillAcquisitionError> for Error {
    fn from(source: SkillAcquisitionError) -> Self {
        Self::SkillAcquisition(source)
    }
}

impl From<SkillLevelError> for Error {
    fn from(source: SkillLevelError) -> Self {
        Self::SkillLevel(source)
    }
}

impl From<SkillDescriptionsError> for Error {
    fn from(source: SkillDescriptionsError) -> Self {
        Self::SkillDescriptions(source)
    }
}

impl From<EnemyNameError> for Error {
    fn from(source: EnemyNameError) -> Self {
        Self::EnemyName(source)
    }
}

impl From<EnemyPictureBookError> for Error {
    fn from(source: EnemyPictureBookError) -> Self {
        Self::EnemyPictureBook(source)
    }
}

impl From<BattlegroundError> for Error {
    fn from(source: BattlegroundError) -> Self {
        Self::Battleground(source)
    }
}

impl From<CertificationPresetError> for Error {
    fn from(source: CertificationPresetError) -> Self {
        Self::CertificationPreset(source)
    }
}

impl From<CharaGroupError> for Error {
    fn from(source: CharaGroupError) -> Self {
        Self::CharaGroup(source)
    }
}

impl From<DifficultyLevelError> for Error {
    fn from(source: DifficultyLevelError) -> Self {
        Self::DifficultyLevel(source)
    }
}

impl From<DropCharaError> for Error {
    fn from(source: DropCharaError) -> Self {
        Self::DropChara(source)
    }
}

impl From<FixedFormationError> for Error {
    fn from(source: FixedFormationError) -> Self {
        Self::FixedFormation(source)
    }
}

impl From<MapStageDataError> for Error {
    fn from(source: MapStageDataError) -> Self {
        Self::MapStageData(source)
    }
}

impl From<ScatCpuSettingError> for Error {
    fn from(source: ScatCpuSettingError) -> Self {
        Self::ScatCpuSetting(source)
    }
}

impl From<StageNameError> for Error {
    fn from(source: StageNameError) -> Self {
        Self::StageName(source)
    }
}

impl From<StageOptionError> for Error {
    fn from(source: StageOptionError) -> Self {
        Self::StageOption(source)
    }
}

impl From<DropItemError> for Error {
    fn from(source: DropItemError) -> Self {
        Self::DropItem(source)
    }
}

impl From<ExOptionError> for Error {
    fn from(source: ExOptionError) -> Self {
        Self::ExOption(source)
    }
}

impl From<LockSkipDataError> for Error {
    fn from(source: LockSkipDataError) -> Self {
        Self::LockSkipData(source)
    }
}

impl From<MapNameError> for Error {
    fn from(source: MapNameError) -> Self {
        Self::MapName(source)
    }
}

impl From<MapOptionError> for Error {
    fn from(source: MapOptionError) -> Self {
        Self::MapOption(source)
    }
}

impl From<ScoreBonusMapError> for Error {
    fn from(source: ScoreBonusMapError) -> Self {
        Self::ScoreBonusMap(source)
    }
}

impl From<SpecialRulesMapError> for Error {
    fn from(source: SpecialRulesMapError) -> Self {
        Self::SpecialRulesMap(source)
    }
}

impl From<SpecialRulesMapOptionError> for Error {
    fn from(source: SpecialRulesMapOptionError) -> Self {
        Self::SpecialRulesMapOption(source)
    }
}

impl From<LocalizableError> for Error {
    fn from(source: LocalizableError) -> Self {
        Self::Localizable(source)
    }
}

impl From<ParamError> for Error {
    fn from(source: ParamError) -> Self {
        Self::Param(source)
    }
}

impl From<GatyaItemBuyError> for Error {
    fn from(source: GatyaItemBuyError) -> Self {
        Self::GatyaItemBuy(source)
    }
}

impl From<GatyaItemNameError> for Error {
    fn from(source: GatyaItemNameError) -> Self {
        Self::GatyaItemName(source)
    }
}

#[cfg(feature = "graphics")]
impl From<crate::graphics::rig::RigError> for Error {
    fn from(source: crate::graphics::rig::RigError) -> Self {
        Self::Rig(source)
    }
}

#[cfg(feature = "graphics")]
impl From<crate::graphics::tools::part::PartError> for Error {
    fn from(source: crate::graphics::tools::part::PartError) -> Self {
        Self::Part(source)
    }
}

#[cfg(feature = "pack")]
impl From<crate::pack::cryptology::PackError> for Error {
    fn from(source: crate::pack::cryptology::PackError) -> Self {
        Self::Pack(source)
    }
}

#[cfg(feature = "bcu")]
impl From<crate::bcu::cryptology::Error> for Error {
    fn from(source: crate::bcu::cryptology::Error) -> Self {
        Self::Bcu(source)
    }
}
