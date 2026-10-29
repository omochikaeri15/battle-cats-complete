#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct InvasionTarget {
    pub map_id: u32,
    pub stage: u32,
    pub crown: u8,
}

pub fn fixed_invasion_target(ex_map_id: u32) -> Option<InvasionTarget> {
    let (map_id, stage, crown) = match ex_map_id {
        4026 => (48, 0, 3),
        4042 => (30000, 29, 0),
        4068 => (13035, 5, 0),
        4069 => (13048, 0, 3),
        _ => return None,
    };

    Some(InvasionTarget { map_id, stage, crown })
}
