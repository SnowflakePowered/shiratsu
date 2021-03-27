#[macro_use]
pub(crate) mod common;

mod xml;

pub mod nointro;
pub mod redump;
pub mod tosec;
pub mod dats_site;
pub mod opengood;
pub mod generic;
mod error;

pub use common::*;
pub use error::*;


#[cfg(test)]
mod tests {

    use crate::NameInfo;

    use shiratsu_naming::naming::tosec::TOSECName;
    use shiratsu_naming::region::Region;
    use shiratsu_naming::naming::nointro::NoIntroName;
    use shiratsu_naming::naming::TokenizedName;

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
        let parsed: NameInfo = NoIntroName::try_parse("Cube CD 20, The (40) - Testing (Europe) (Unl)").unwrap().into();
        assert_eq!("Cube CD 20, The (40) - Testing", parsed.entry_title());
        assert_eq!("The Cube CD 20 (40): Testing", parsed.release_title());
    }

    #[test]
    fn nointro_filename_parses_2() {
        let parsed: NameInfo = NoIntroName::try_parse("Star Jacker (Japan, Europe, Australia, New Zealand) (Rev 1)").unwrap().into();
        assert_eq!("Star Jacker", parsed.entry_title());
        assert_eq!(&[Region::Japan, Region::Europe, Region::Australia, Region::NewZealand], parsed.region());
    }

    #[test]
    fn nointro_filename_parses_odekake() {
        let parsed: NameInfo = NoIntroName::try_parse("Odekake Lester - Lelele no Le (^^; (Japan)").unwrap().into();
        assert_eq!("Odekake Lester - Lelele no Le (^^;", parsed.entry_title());
        assert_eq!("Odekake Lester: Lelele no Le (^^;", parsed.release_title());
        assert_eq!(&[Region::Japan], parsed.region());
    }

    #[test]
    fn nointro_filename_parses_end() {
        let parsed: NameInfo  = NoIntroName::try_parse("Cube CD 20, The (40) - Testing (Europe) (Rev 10)").unwrap().into();
        assert_eq!("Cube CD 20, The (40) - Testing", parsed.entry_title());
        assert_eq!("The Cube CD 20 (40): Testing", parsed.release_title());
        assert_eq!(Some("10"), parsed.version());
    }

    #[test]
    fn tosec_filename_parses() {
        let parsed: NameInfo = TOSECName::try_parse("Cube CD 20, The (40) - Testing (demo) (2020)(SomePublisher)(US)").unwrap().into();
        assert_eq!("Cube CD 20, The (40) - Testing", parsed.entry_title());
        assert_eq!("The Cube CD 20 (40): Testing", parsed.release_title());
        assert_eq!(&[Region::UnitedStates], parsed.region());
    }


    #[test]
    fn tosec_filename_parses_2() {
        let parsed:  NameInfo  = TOSECName::try_parse("2600 Digital Clock - Demo 1 (demo)(1997-10-03)(Cracknell, Chris 'Crackers')(NTSC)(PD)").unwrap().into();
        assert_eq!("2600 Digital Clock - Demo 1", parsed.entry_title());
        assert_eq!("2600 Digital Clock: Demo 1", parsed.release_title());
        assert_eq!(&[Region::Unknown], parsed.region());
    }


    #[test]
    fn tosec_filename_parses_3() {
        let parsed:  NameInfo  = TOSECName::try_parse("2600 Digital Clock - Demo 1 (demo-playable)(1997-10-03)(Cracknell, Chris 'Crackers')(NTSC)(PD)").unwrap().into();
        assert_eq!("2600 Digital Clock - Demo 1", parsed.entry_title());
        assert_eq!("2600 Digital Clock: Demo 1", parsed.release_title());
        assert_eq!(&[Region::Unknown], parsed.region());
    }

    #[test]
    fn tosec_filename_parses_4() {
        let parsed:  NameInfo  = TOSECName::try_parse("Bombsawa (Jumpman Selected levels)(19XX)(-)(PD)").unwrap().into();
        assert_eq!("Bombsawa (Jumpman Selected levels)", parsed.entry_title());
        assert_eq!(&[Region::Unknown], parsed.region());
    }

    #[test]
    fn tosec_filename_parses_5() {
        let parsed: NameInfo  = TOSECName::try_parse("Motocross & Pole Position (Starsoft - JVP)(PAL)[b1][possible unknown mode]").unwrap().into();
        assert_eq!("Motocross & Pole Position", parsed.entry_title());
        assert_eq!(&[Region::Unknown], parsed.region());
    }

    #[test]
    fn tosec_filename_parses_6() {
        let parsed:  NameInfo  = TOSECName::try_parse("256 Color Demo (1997)(Schick, Bastian)(PD)[a]").unwrap().into();
        assert_eq!("256 Color Demo", parsed.entry_title());
        assert_eq!(&[Region::Unknown], parsed.region());
    }

    #[test]
    fn tosec_filename_parses_end() {
        let parsed:  NameInfo  = TOSECName::try_parse("Cube CD 20, The (40) - Testing v1.203 (demo) (2020)(SomePublisher)").unwrap().into();
        assert_eq!("Cube CD 20, The (40) - Testing", parsed.entry_title());
        assert_eq!("The Cube CD 20 (40): Testing", parsed.release_title());
        assert_eq!(&[Region::Unknown], parsed.region());
        assert_eq!(Some("1.203"), parsed.version());
    }

    #[test]
    fn tosec_filename_parses_end_rev() {
        let parsed:  NameInfo  = TOSECName::try_parse("Cube CD 20, The (40) - Testing Rev 1 (demo) (2020)(SomePublisher)").unwrap().into();
        assert_eq!("Cube CD 20, The (40) - Testing", parsed.entry_title());
        assert_eq!("The Cube CD 20 (40): Testing", parsed.release_title());
        assert_eq!(&[Region::Unknown], parsed.region());
        assert_eq!(Some("1"), parsed.version());
    }
}

