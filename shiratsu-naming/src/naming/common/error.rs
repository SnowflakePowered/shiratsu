use std::fmt::{Display, Formatter};
use std::error::Error;

use crate::region::RegionError;
use crate::naming::*;
use std::fmt;

#[derive(Debug, PartialEq)]
/// Name parsing errors.
pub enum NameError {
    /// The file name could not be parsed with the given naming conventions.
    ParseError(NamingConvention, String),

    /// An error occured when parsing a region string.
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
            NameError::ParseError(convention, string) =>
                write!(f, "The name \"{}\" could not be parsed properly in the {:?} naming convention", string, convention),
            NameError::RegionError(region_err) =>
                write!(f, "{}", region_err),
        }
    }
}

pub(crate) type Result<T> = std::result::Result<T, NameError>;
