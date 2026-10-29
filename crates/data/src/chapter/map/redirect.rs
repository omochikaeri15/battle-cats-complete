//! Resolution of map identifiers that address a different map's data.

/// Resolves a map identifier to the identifier its data is actually stored under.
///
/// Several late-game and variant maps are addressed by one identifier in the
/// stage tables while their configuration lives under another, so a direct
/// lookup for these would find nothing. Identifiers without such an indirection
/// are returned unchanged.
///
/// # Arguments
/// * `id` - The map identifier as it appears in the referring table.
///
/// # Returns
/// A `u32` containing the identifier the map's own data is keyed by.
pub fn redirect_map_id(id: u32) -> u32 {
    match id {
        20000 => 3008, // EoC 1 Zombie
        20001 => 3009, // EoC 2 Zombie
        20002 => 3010, // EoC 3 Zombie

        21000 => 3011, // ItF 1 Zombie
        21001 => 3012, // ItF 2 Zombie
        21002 => 3013, // ItF 3 Zombie

        22000 => 3015, // CotC 1 Zombie
        22001 => 3015, // CotC 2 Zombie
        22002 => 3016, // CotC 3 Zombie

        23000 => 3007, // CotC 3 Invasion
        30000 => 3018, // Aku Realms
        38000 => 3017, // CotC 3 Invasion (Zombie)

        _ => id,
    }
}