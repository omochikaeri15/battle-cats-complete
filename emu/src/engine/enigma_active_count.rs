use super::Enigma;

pub fn enigma_active_count(enigma: &Enigma) -> i64 {
    enigma.active.len() as i64
}
