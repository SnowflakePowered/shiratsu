use crate::naming::{NameInfo, NamingConvention};
use crate::error::{ParseError, Result};

use crate::naming::tosec::{TOSECName, TOSECMultiSetName};
use crate::naming::tosec::parsers::{do_parse, do_parse_multiset};

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
/// Tokens will be in the order of appearance in the resulting `Vec`.
/// Warnings occur before the associated token.
pub fn try_parse(input: &str) -> Result<TOSECName> {
   TOSECName::try_parse(input)
}

pub fn try_parse_multiset(input: &str) -> Result<TOSECMultiSetName> {
    let (_, value) = do_parse_multiset(input).map_err(|_| {
        ParseError::BadFileNameError(NamingConvention::TOSEC, input.to_string())
    })?;
    Ok(value.into())
}