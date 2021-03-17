use crate::naming::{NameInfo, NamingConvention};
use crate::error::{ParseError, Result};

use crate::naming::tosec::TOSECToken;
use crate::naming::tosec::parsers::do_parse;

use crate::naming::tosec::legacy_parser::tosec_parser;

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