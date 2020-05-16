extern crate lazy_static_include;
extern crate lazy_static;

extern crate serde;
extern crate serde_json;
extern crate phf;

pub mod stone;
pub mod region;
mod util;
pub mod parse;

pub mod error {
    // #[derive(Debug)]
    pub enum ShiratsuError {
        StoneError(crate::stone::StoneError),
        ParseError(crate::parse::ParseError),
        IOError(std::io::Error)
    }
    impl std::error::Error for ShiratsuError {}

    impl std::fmt::Debug for ShiratsuError{
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                ShiratsuError::StoneError(err) => write!(f, "{}", err),
                ShiratsuError::ParseError(err) => write!(f, "{}", err),
                ShiratsuError::IOError(err) => write!(f, "{}", err),
            }
        }
    }
    
    impl std::fmt::Display for ShiratsuError{
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{:?}", self)
        }
    }

    impl From<crate::parse::ParseError> for ShiratsuError {
        fn from(err: crate::parse::ParseError) -> Self {
            ShiratsuError::ParseError(err)
        }
    }

    impl From<crate::stone::StoneError> for ShiratsuError {
        fn from(err: crate::stone::StoneError) -> Self {
            ShiratsuError::StoneError(err)
        }
    }

    impl From<std::io::Error> for ShiratsuError {
        fn from(err: std::io::Error) -> Self {
            ShiratsuError::IOError(err)
        }
    }

    pub type Result<T> = std::result::Result<T, ShiratsuError>;
}

#[cfg(test)]
mod tests {
    use crate::region::{Region, parse_regions};
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }

    #[test]
    fn nointro_region_parses() {
        assert_eq!(parse_regions("USA, Europe"), vec![Region::UnitedStates, Region::Europe]);
    }
    
    #[test]
    fn tosec_region_parses() {
        assert_eq!(parse_regions("US"), vec![Region::UnitedStates]);
        assert_eq!(parse_regions("US-ZZ"), vec![Region::UnitedStates, Region::Unknown]);
    }
}

