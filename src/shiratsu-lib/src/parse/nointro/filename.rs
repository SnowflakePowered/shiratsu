use super::super::Result;
use super::super::*;
use super::legacy_parser::nointro_legacy_parser;
use crate::parse::nointro::parsers::NoIntroToken;
use crate::parse::nointro::parsers::do_parse;

pub trait NoIntroNameable {
    fn try_from_nointro(nointro: &str) -> Result<NameInfo>;
}

impl NoIntroNameable for NameInfo {
    fn try_from_nointro(name: &str) -> Result<NameInfo> {
        nointro_legacy_parser(name)
    }
}

/// Tries to parse the name into a vector of tokens.
pub fn try_parse<'a>(input: &str) -> Result<Vec<NoIntroToken>> {
    let (_, value) = do_parse(input).map_err(|_| {
        ParseError::BadFileNameError(NamingConvention::NoIntro, input.to_string())
    })?;
    Ok(value)
}