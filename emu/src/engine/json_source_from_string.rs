use super::json_source_new;

pub fn json_source_from_string(text: &[u8]) -> Vec<u8> {
    json_source_new(text)
}
