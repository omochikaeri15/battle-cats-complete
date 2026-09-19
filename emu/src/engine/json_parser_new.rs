use std::collections::BTreeMap;

use crate::Fault;

use super::json_parse;

#[derive(Clone, Debug, PartialEq)]
pub enum JsonNode {
    Null,
    Bool(u8),
    Int(i64),
    Uint(u64),
    Double(f64),
    String(Vec<u8>),
    Array(Vec<JsonNode>),
    Object(BTreeMap<Vec<u8>, JsonNode>),
}

pub struct JsonParser {
    pub source: Vec<u8>,
    pub begin: usize,
    pub end: usize,
    pub root: Option<JsonNode>,
    pub cursor: usize,
}

pub fn json_parser_new(source: Vec<u8>) -> Result<JsonParser, Fault> {
    let end = source.len();
    let mut parser = JsonParser { source, begin: 0, end, root: None, cursor: 0 };

    if parser.source.len() >= 4 && parser.source[0] == 0xef && parser.source[1] == 0xbb && parser.source[2] == 0xbf {
        parser.begin += 3;
    }

    json_parse(&mut parser)?;

    Ok(parser)
}
