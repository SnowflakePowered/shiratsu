use crate::parse::{NameInfo, NamingConvention, ParseError, Result};
use crate::parse::tosec::parsers::{TOSECToken, do_parse};

use super::legacy_parser::tosec_parser;

pub trait TOSECNameable {
    fn try_from_tosec(tosec: &str) -> Result<NameInfo>;
}

impl TOSECNameable for NameInfo {
    fn try_from_tosec(name: &str) -> Result<NameInfo> {
        tosec_parser(name)
    }
}

/// Tries to parse the name into a vector of tokens.
pub fn try_parse<'a>(input: &str) -> Result<Vec<TOSECToken>> {
    let (_, value) = do_parse(input).map_err(|_| {
        ParseError::BadFileNameError(NamingConvention::TOSEC, input.to_string())
    })?;
    Ok(value)
}