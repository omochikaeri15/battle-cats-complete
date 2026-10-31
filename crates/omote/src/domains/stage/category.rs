use codex::chapter::Category;

pub(crate) fn display_name(category: &Category) -> &str {
    match category {
        Category::StoriesOfLegend => "Stories of Legend",
        Category::EventStages => "Event Stages",
        Category::CollabStages => "Collab Stages",
        Category::EmpireOfCats => "Empire of Cats",
        Category::IntoTheFuture => "Into the Future",
        Category::CatsOfTheCosmos => "Cats of the Cosmos",
        Category::ExtraStages => "Extra Stages",
        Category::DojoHallOfInitiates => "Dojo Hall of Initiates",
        Category::TowersAndCitadels => "Towers & Citadels",
        Category::DojoRankingEvents => "Dojo Ranking Events",
        Category::ChallengeBattle => "Challenge Battle",
        Category::UncannyLegends => "Uncanny Legends",
        Category::CataminStages => "Catamin Stages",
        Category::LegendQuest => "Legend Quest",
        Category::ZombieOutbreaks => "Zombie Outbreaks",
        Category::GauntletStages => "Gauntlet Stages",
        Category::EnigmaStages => "Enigma Stages",
        Category::CollabGauntletStages => "Collab Gauntlet Stages",
        Category::AkuRealms => "Aku Realms",
        Category::BehemothCulling => "Behemoth Culling",
        Category::Labyrinth => "Labyrinth",
        Category::ZeroLegends => "Zero Legends",
        Category::OtherworldColosseum => "Otherworld Colosseum",
        Category::CatclawChampionships => "Catclaw Championships",
        Category::FeverRush => "Fever Rush",
        Category::Unknown(prefix) => {
            if prefix == "PT" {
                "Legacy Princess Punt"
            } else {
                prefix.as_str()
            }
        }
    }
}

pub(crate) fn sort_order(category: &Category) -> u32 {
    match category {
        Category::EmpireOfCats => 0,
        Category::IntoTheFuture => 1,
        Category::CatsOfTheCosmos => 2,
        Category::ZombieOutbreaks => 3,
        Category::AkuRealms => 4,
        Category::StoriesOfLegend => 10,
        Category::UncannyLegends => 11,
        Category::ZeroLegends => 12,
        Category::EventStages => 20,
        Category::ExtraStages => 21,
        Category::GauntletStages => 22,
        Category::CataminStages => 24,
        Category::ChallengeBattle => 25,
        Category::FeverRush => 26,
        Category::TowersAndCitadels => 31,
        Category::OtherworldColosseum => 32,
        Category::Labyrinth => 33,
        Category::LegendQuest => 34,
        Category::BehemothCulling => 41,
        Category::EnigmaStages => 42,
        Category::DojoHallOfInitiates => 51,
        Category::DojoRankingEvents => 52,
        Category::CatclawChampionships => 53,
        Category::CollabStages => 61,
        Category::CollabGauntletStages => 62,
        Category::Unknown(prefix) => {
            if prefix == "PT" {
                63
            } else {
                99
            }
        }
    }
}
