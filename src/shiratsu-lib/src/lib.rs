pub mod stone;
pub mod region;
mod util;
pub mod parse;

#[cfg(feature = "rusqlite")]
pub mod rusqlite;

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
    use crate::region::Region;
    use crate::parse::NameInfo;
    use crate::parse::nointro::NoIntroNameable;
    use crate::parse::tosec::TosecNameable;
    #[test]
    fn nointro_region_parses() {
        assert_eq!(Region::from_region_string("USA, Europe"), vec![Region::UnitedStates, Region::Europe]);
    }
    
    #[test]
    fn tosec_region_parses() {
        assert_eq!(Region::from_region_string("US"), vec![Region::UnitedStates]);
        assert_eq!(Region::from_region_string("US-ZZ"), vec![Region::UnitedStates, Region::Unknown]);
    }

    #[test]
    fn nointro_filename_parses() {
        let parsed = NameInfo::try_from_nointro("Cube CD 20, The (40) - Testing (Europe) (Unl)").unwrap();
        assert_eq!("Cube CD 20, The (40) - Testing", parsed.entry_title());
        assert_eq!("The Cube CD 20 (40): Testing", parsed.release_title());
    }

    #[test]
    fn nointro_filename_parses_end() {
        let parsed = NameInfo::try_from_nointro("Cube CD 20, The (40) - Testing (Europe) (Rev 10)").unwrap();
        assert_eq!("Cube CD 20, The (40) - Testing", parsed.entry_title());
        assert_eq!("The Cube CD 20 (40): Testing", parsed.release_title());
        assert_eq!(Some("10"), parsed.version());
    }

    #[test]
    #[should_panic]
    fn nointro_no_hang() {
        NameInfo::try_from_nointro("Cube CD 20, The (40) - Testing (demo) (2020)(SomePublisher)").unwrap();
    }

    #[test]
    fn tosec_filename_parses() {
        let parsed = NameInfo::try_from_tosec("Cube CD 20, The (40) - Testing (demo) (2020)(SomePublisher)(US)").unwrap();
        assert_eq!("Cube CD 20, The (40) - Testing", parsed.entry_title());
        assert_eq!("The Cube CD 20 (40): Testing", parsed.release_title());
        assert_eq!(&[Region::UnitedStates], parsed.region());
    }

    #[test]
    fn tosec_filename_parses_end() {
        let parsed = NameInfo::try_from_tosec("Cube CD 20, The (40) - Testing (demo) (2020)(SomePublisher)(v1.203)").unwrap();
        assert_eq!("Cube CD 20, The (40) - Testing", parsed.entry_title());
        assert_eq!("The Cube CD 20 (40): Testing", parsed.release_title());
        assert_eq!(&[Region::Unknown], parsed.region());
        assert_eq!(Some("1.203"), parsed.version());
    }
    #[test]
    fn tosec_filename_parses_end_rev() {
        let parsed = NameInfo::try_from_tosec("Cube CD 20, The (40) - Testing Rev 1 (demo) (2020)(SomePublisher)").unwrap();
        assert_eq!("Cube CD 20, The (40) - Testing", parsed.entry_title());
        assert_eq!("The Cube CD 20 (40): Testing", parsed.release_title());
        assert_eq!(&[Region::Unknown], parsed.region());
        assert_eq!(Some("1"), parsed.version());
    }
}

