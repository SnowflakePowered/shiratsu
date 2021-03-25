use std::fmt::{Display, Formatter};
use std::error::Error;

use std::fmt;
use shiratsu_naming::error::NameError;

#[derive(Debug)]
pub enum DatError {
    ParseError(String),
    NameError(NameError),
    HeaderMismatchError(&'static str, Option<String>),
}

impl From<NameError> for DatError {
    fn from(err: NameError) -> Self {
        DatError::NameError(err)
    }
}

impl Error for DatError {}

impl Display for DatError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            DatError::ParseError(val) => write!(f, "{}", val),
            DatError::HeaderMismatchError(expected, actual) =>
                write!(f,
                       "Expected DAT to have header homepage \"{}\" but it actually was \"{}\". Use unchecked variants to ignore header checking.",
                       expected, actual.as_deref().unwrap_or("None")),
            DatError::NameError(error) => write!(f, "Naming error: {}", error)
        }
    }
}

pub type Result<T> = std::result::Result<T, DatError>;
