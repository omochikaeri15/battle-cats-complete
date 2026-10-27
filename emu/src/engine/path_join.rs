pub fn path_join(head: &[u8], tail: &[u8]) -> Vec<u8> {
    if head.is_empty() {
        return tail.to_vec();
    }

    let mut joined = head.to_vec();

    if head.last() != Some(&b'/') && tail.first() != Some(&b'/') {
        joined.push(b'/');
    }

    joined.extend_from_slice(tail);

    joined
}
