//! Resolution of the stages the engine's fixed diversions hand over to an EX map.

/// A stage the engine swaps for an EX map once the diversion's conditions are met.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct InvasionTarget {
    /// The global identifier of the map whose stage is taken over.
    pub map_id: u32,
    /// The index of the stage within that map.
    pub stage: u32,
    /// The crown level, counted from zero, the diversion is tested at.
    pub crown: u8,
}

/// Resolves the stage an EX map takes over through one of the engine's fixed diversions.
///
/// Four stages are swapped for an EX map by a rule the engine holds itself rather than
/// reading from the EX option table. EX maps that table links are not covered here.
///
/// # Arguments
/// * `ex_map_id` - The global identifier of the EX map.
///
/// # Returns
/// An `Option` containing the `InvasionTarget` the EX map takes over, or `None` when no
/// fixed diversion leads into it.
pub fn fixed_invasion_target(ex_map_id: u32) -> Option<InvasionTarget> {
    let (map_id, stage, crown) = match ex_map_id {
        4026 => (48, 0, 3),     // Laboratory of Relics
        4042 => (30000, 29, 0), // The Aku Realms
        4068 => (13035, 5, 0),  // Ancient Forest Labyrinth
        4069 => (13048, 0, 3),  // Sacred Forest
        _ => return None,
    };

    Some(InvasionTarget { map_id, stage, crown })
}
