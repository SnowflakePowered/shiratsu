use super::super::Result;
use super::super::*;
use super::parser_old::nointro_parser;

pub trait NoIntroNameable {
    fn try_from_nointro(nointro: &str) -> Result<NameInfo>;
}

impl NoIntroNameable for NameInfo {
    fn try_from_nointro(name: &str) -> Result<NameInfo> {
        nointro_parser(name)
    }
}
