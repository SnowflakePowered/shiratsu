use std::fmt::{Display, Formatter};
use std::error::Error;

use crate::region::RegionError;
use crate::naming::*;
use std::fmt;

#[derive(Debug)]
pub enum NameError {
    ParseError(String),
    BadFileNameError(NamingConvention, String),
    RegionError(RegionError),
}

impl From<RegionError> for NameError {
    fn from(err: RegionError) -> Self {
        NameError::RegionError(err)
    }
}

impl Error for NameError {}

impl Display for NameError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            NameError::ParseError(val) => write!(f, "{}", val),
            NameError::BadFileNameError(convention, string) =>
                write!(f, "The \"{}\" could not be parsed properly in the {:?} naming convention", string, convention),
            NameError::RegionError(region_err) =>
                write!(f, "{}", region_err),
        }
    }
}

pub type Result<T> = std::result::Result<T, NameError>;
