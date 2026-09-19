pub fn string_split(text: &[u8], separator: &[u8]) -> Vec<Vec<u8>> {
    let mut pieces = Vec::new();

    if text.is_empty() {
        return pieces;
    }

    let mut start = 0usize;

    loop {
        let found = if separator.is_empty() {
            Some(start)
        } else {
            text.get(start..).and_then(|rest| rest.windows(separator.len()).position(|window| window == separator)).map(|at| at + start)
        };

        let Some(at) = found else {
            if text.len() > start {
                pieces.push(text[start..].to_vec());
            }

            return pieces;
        };

        pieces.push(text[start..at].to_vec());
        start = at + separator.len();

        if start > text.len() {
            return pieces;
        }
    }
}
