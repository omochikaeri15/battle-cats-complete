use super::JsonParser;

pub fn json_next_token(parser: &mut JsonParser) -> i32 {
    if parser.cursor >= parser.end {
        return 0;
    }

    let at = parser.cursor;
    let source = &parser.source;
    let (token, width) = match source[at] {
        b'-' | b'0'..=b'9' => (0xc, 1),
        b'\t' | b'\n' | b'\r' | b' ' => (7, 1),
        b'[' => (5, 1),
        b'n' => {
            if at + 3 >= parser.end || source[at + 1] != b'u' || source[at + 2] != b'l' || source[at + 3] != b'l' {
                return 0xd;
            }

            (9, 4)
        }
        b'{' => (3, 1),
        b't' => {
            if at + 3 >= parser.end || source[at + 1] != b'r' || source[at + 2] != b'u' || source[at + 3] != b'e' {
                return 0xd;
            }

            (0xa, 4)
        }
        b']' => (6, 1),
        b'"' => (8, 1),
        b',' => (1, 1),
        b'}' => (4, 1),
        b':' => (2, 1),
        b'f' => {
            if at + 4 >= parser.end || source[at + 1] != b'a' || source[at + 2] != b'l' || source[at + 3] != b's' || source[at + 4] != b'e' {
                return 0xd;
            }

            (0xb, 5)
        }
        _ => return 0xd,
    };

    parser.cursor = at + width;

    token
}
