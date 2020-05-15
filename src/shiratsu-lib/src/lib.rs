extern crate lazy_static_include;
extern crate lazy_static;

extern crate serde;
extern crate serde_json;
extern crate phf;

pub mod stone;
pub mod region;
mod util;
pub mod parse;

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

