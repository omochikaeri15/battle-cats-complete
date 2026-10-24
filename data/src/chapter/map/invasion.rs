//! Resolution of the stages the engine's fixed diversions hand over to an EX map.

/// Resolves the stage an EX map takes over through one of the engine's fixed diversions.
///
/// Four stages are swapped for an EX map once their conditions are met, by a rule the
/// engine holds itself rather than reading from the EX option table. EX maps that table
/// links are not covered here.
///
/// # Arguments
/// * `ex_map_id` - The global identifier of the EX map.
///
/// # Returns
/// An `Option` containing the global map identifier and stage index the EX map takes
/// over, or `None` when no fixed diversion leads into it.
pub fn fixed_invasion_target(ex_map_id: u32) -> Option<(u32, u32)> {
    match ex_map_id {
        4026 => Some((48, 0)),     // Laboratory of Relics, four crowns
        4042 => Some((30000, 29)), // The Aku Realms
        4068 => Some((13035, 5)),  // Ancient Forest Labyrinth
        4069 => Some((13048, 0)),  // Sacred Forest, four crowns
        _ => None,
    }
}
