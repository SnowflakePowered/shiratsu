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
    fn nointro_region_parses_many() {
        assert_eq!(Region::try_from_nointro_region("Japan, Europe, Australia, New Zealand").unwrap(), vec![Region::Japan, Region::Europe, Region::Australia, Region::NewZealand]);
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
    fn nointro_filename_parses_2() {
        let parsed = NameInfo::try_from_nointro("Star Jacker (Japan, Europe, Australia, New Zealand) (Rev 1)").unwrap();
        assert_eq!("Star Jacker", parsed.entry_title());
        assert_eq!(&[Region::Japan, Region::Europe, Region::Australia, Region::NewZealand], parsed.region());
    }

    #[test]
    fn nointro_filename_parses_odekake() {
        let parsed = NameInfo::try_from_nointro("Odekake Lester - Lelele no Le (^^; (Japan)").unwrap();
        assert_eq!("Odekake Lester - Lelele no Le (^^;", parsed.entry_title());
        assert_eq!("Odekake Lester: Lelele no Le (^^;", parsed.release_title());
        assert_eq!(&[Region::Japan], parsed.region());
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
    fn tosec_filename_parses_2() {
        let parsed = NameInfo::try_from_tosec("2600 Digital Clock - Demo 1 (demo)(1997-10-03)(Cracknell, Chris 'Crackers')(NTSC)(PD)").unwrap();
        assert_eq!("2600 Digital Clock - Demo 1", parsed.entry_title());
        assert_eq!("2600 Digital Clock: Demo 1", parsed.release_title());
        assert_eq!(&[Region::Unknown], parsed.region());
    }

    
    #[test]
    fn tosec_filename_parses_3() {
        let parsed = NameInfo::try_from_tosec("2600 Digital Clock - Demo 1 (demo-playable)(1997-10-03)(Cracknell, Chris 'Crackers')(NTSC)(PD)").unwrap();
        assert_eq!("2600 Digital Clock - Demo 1", parsed.entry_title());
        assert_eq!("2600 Digital Clock: Demo 1", parsed.release_title());
        assert_eq!(&[Region::Unknown], parsed.region());
    }

    #[test]
    fn tosec_filename_parses_4() {
        let parsed = NameInfo::try_from_tosec("Bombsawa (Jumpman Selected levels)(19XX)(-)(PD)").unwrap();
        assert_eq!("Bombsawa (Jumpman Selected levels)", parsed.entry_title());
        assert_eq!(&[Region::Unknown], parsed.region());
    }

    #[test]
    fn tosec_filename_parses_5() {
        let parsed = NameInfo::try_from_tosec("Motocross & Pole Position (Starsoft - JVP)(PAL)[b1][possible unknown mode]").unwrap();
        assert_eq!("Motocross & Pole Position", parsed.entry_title());
        assert_eq!(&[Region::Unknown], parsed.region());
    }

    #[test]
    fn tosec_filename_parses_6() {
        let parsed = NameInfo::try_from_tosec("256 Color Demo (1997)(Schick, Bastian)(PD)[a]").unwrap();
        assert_eq!("256 Color Demo", parsed.entry_title());
        assert_eq!(&[Region::Unknown], parsed.region());
    }

    #[test]
    fn tosec_filename_parses_end() {
        let parsed = NameInfo::try_from_tosec("Cube CD 20, The (40) - Testing v1.203 (demo) (2020)(SomePublisher)").unwrap();
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

