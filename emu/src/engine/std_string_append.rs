pub fn std_string_append(target: &mut Vec<u8>, suffix: &[u8]) {
    target.extend_from_slice(suffix);
}
