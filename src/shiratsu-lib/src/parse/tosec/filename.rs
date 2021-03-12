use crate::parse::{NameInfo, NamingConvention, ParseError, Result};

use super::legacy_parser::tosec_parser;

pub trait TosecNameable {
    fn try_from_tosec(tosec: &str) -> Result<NameInfo>;
}

impl TosecNameable for NameInfo {
    fn try_from_tosec(name: &str) -> Result<NameInfo> {
        tosec_parser(name)
    }
}
