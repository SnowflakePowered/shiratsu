
use crate::naming::{NameInfo, NamingConvention};
use crate::error::{ParseError, Result};
use crate::naming::goodtools::GoodToolsName;
use crate::naming::goodtools::parsers::do_parse;


pub trait GoodToolsNameable {
    fn try_from_goodtools(nointro: &str) -> Result<NameInfo>;
}

impl GoodToolsNameable for NameInfo {
    fn try_from_goodtools(name: &str) -> Result<NameInfo> {
        unimplemented!()
    }
}

/// Tries to parse the name into a vector of tokens.
pub fn try_parse(input: &str) -> Result<GoodToolsName> {
    let (_, value) = do_parse(input).map_err(|_| {
        ParseError::BadFileNameError(NamingConvention::NoIntro, input.to_string())
    })?;
    Ok(value.into())
}