
use crate::naming::{NameInfo, NamingConvention};
use crate::error::{ParseError, Result};

use crate::naming::nointro::NoIntroToken;
use crate::naming::nointro::legacy_parser::legacy_parser;
use crate::naming::nointro::parsers::do_parse;

pub trait NoIntroNameable {
    fn try_from_nointro(nointro: &str) -> Result<NameInfo>;
}

impl NoIntroNameable for NameInfo {
    fn try_from_nointro(name: &str) -> Result<NameInfo> {
        legacy_parser(name)
    }
}

/// Tries to parse the name into a vector of tokens.
pub fn try_parse<'a>(input: &str) -> Result<Vec<NoIntroToken>> {
    let (_, value) = do_parse(input).map_err(|_| {
        ParseError::BadFileNameError(NamingConvention::NoIntro, input.to_string())
    })?;
    Ok(value)
}