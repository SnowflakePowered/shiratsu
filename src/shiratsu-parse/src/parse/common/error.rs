use std::fmt;
use std::fmt::{Display, Formatter};
use std::error::Error;

use crate::region::RegionError;
use super::*;

#[derive(Debug)]
pub enum ParseError {
    ParseError(String),
    BadFileNameError(NamingConvention, String),
    RegionError(RegionError),
    HeaderMismatchError(&'static str, Option<String>),
}

impl From<RegionError> for ParseError {
    fn from(err: RegionError) -> Self {
        ParseError::RegionError(err)
    }
}

impl Error for ParseError {}

impl Display for ParseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            ParseError::ParseError(val) => write!(f,"{}", val),
            ParseError::BadFileNameError(convention, string) => 
                write!(f, "The \"{}\" could not be parsed properly in the {:?} naming convention", string, convention),
            ParseError::RegionError(region_err) =>
                write!(f, "{}", region_err),
            ParseError::HeaderMismatchError(expected, actual) =>
                write!(f, 
                    "Expected DAT to have header homepage \"{}\" but it actually was \"{}\". Use unchecked variants to ignore header checking.", 
                    expected, actual.as_deref().unwrap_or("None")),
        }
    }
}

pub type Result<T> = std::result::Result<T, ParseError>;
