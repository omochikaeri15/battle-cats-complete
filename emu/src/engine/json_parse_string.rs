use crate::{operation, Fault};

use super::{JsonNode, JsonParser};

const SITE: &str = "json_parse_string";

pub fn json_parse_string(parser: &mut JsonParser) -> Result<Option<JsonNode>, Fault> {
    let mut text: Vec<u8> = Vec::new();
    let mut closed = false;

    while parser.cursor < parser.end {
        let byte = parser.source[parser.cursor];

        if byte == b'\\' {
            parser.cursor += 1;

            let escape = *parser.source.get(parser.cursor).ok_or(Fault::IndexOutOfRange { site: SITE, index: parser.cursor as i64, limit: parser.end as i64 })?;

            match escape {
                b'b' => text.push(0x8),
                b'n' => text.push(0xa),
                b'r' => text.push(0xd),
                b't' => text.push(0x9),
                b'f' => text.push(0xc),
                b'"' | b'/' | b'\\' => text.push(escape),
                b'u' => text.extend_from_slice(b"\\u"),
                _ => {}
            }
        } else if byte == b'"' {
            closed = true;
            break;
        } else {
            text.push(byte);
        }

        parser.cursor += 1;
    }

    if !closed && *parser.source.get(parser.cursor).ok_or(Fault::IndexOutOfRange { site: SITE, index: parser.cursor as i64, limit: parser.end as i64 })? != b'"' {
        return Ok(None);
    }

    if text.len() >= 2 {
        while let Some(position) = text.windows(2).position(|pair| pair == b"\\u") {
            let digits = position + 2;
            let parsed = operation::strtol(&text[digits..(digits + 4).min(text.len())], 0x10);

            if parsed.end == 0 {
                return Err(Fault::InvalidArgument { site: SITE });
            }

            if parsed.overflow || parsed.value < i32::MIN as i64 || parsed.value > i32::MAX as i64 {
                return Err(Fault::OutOfRange { site: SITE });
            }

            let code = parsed.value as i32;
            let bytes: Vec<u8> = if code <= 0x7f {
                vec![code as u8]
            } else if code as u32 <= 0x7ff {
                vec![(code as u32 >> 6) as u8 | 0xc0, (code as u8 & 0x3f) | 0x80]
            } else if code as u32 <= 0xffff {
                vec![(code as u32 >> 0xc) as u8 | 0xe0, ((code as u32 >> 6) as u8 & 0x3f) | 0x80, (code as u8 & 0x3f) | 0x80]
            } else {
                vec![(code as u32 >> 0x12) as u8 | 0xf0, ((code as u32 >> 0xc) as u8 & 0x3f) | 0x80, ((code as u32 >> 6) as u8 & 0x3f) | 0x80, (code as u8 & 0x3f) | 0x80]
            };

            text.splice(position..(position + 6).min(text.len()), bytes);

            if text.len() <= 1 {
                break;
            }
        }
    }

    parser.cursor += 1;

    Ok(Some(JsonNode::String(text)))
}
