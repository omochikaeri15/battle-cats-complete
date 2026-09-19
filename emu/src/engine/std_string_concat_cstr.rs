pub fn std_string_concat_cstr(prefix: &[u8], text: &[u8]) -> Vec<u8> {
    let mut joined = Vec::with_capacity(prefix.len().wrapping_add(text.len()));

    joined.extend_from_slice(prefix);
    joined.extend_from_slice(text);

    joined
}
