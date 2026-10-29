use crate::{Fault, ops};

use super::{JsonNode, JsonParser};

pub fn json_parse_number(parser: &mut JsonParser) -> Result<Option<JsonNode>, Fault> {
    let start = parser.cursor;
    let mut stop = start;
    let mut fraction = false;

    if start < parser.end && (parser.source[start] == b'-' || parser.source[start].is_ascii_digit())
    {
        let mut at = start + 1;

        parser.cursor = at;
        stop = parser.end;

        while at != parser.end {
            let byte = parser.source[at];

            if byte == b'.' {
                if fraction {
                    stop = at;
                    break;
                }

                fraction = true;
            } else if !byte.is_ascii_digit() {
                stop = at;
                break;
            }

            at += 1;
            parser.cursor = at;
        }
    }

    let text = parser.source[start..stop].to_vec();

    if text.is_empty() || (text.len() == 1 && text[0] == b'-') {
        return Ok(None);
    }

    if fraction {
        let value = ops::strtof(&text).map_or(0.0, |(value, _)| value);

        return Ok(Some(JsonNode::Double(value as f64)));
    }

    if text.iter().position(|&byte| byte == b'-') == Some(0) {
        return Ok(Some(JsonNode::Int(ops::strtol(&text, 10).value)));
    }

    let value = ops::strtoull(&text, 10).map_or(0, |(value, _)| value);

    Ok(Some(JsonNode::Uint(value)))
}
