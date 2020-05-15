extern crate lazy_static_include;
extern crate lazy_static;
extern crate serde;
extern crate serde_json;
extern crate phf;

pub mod stone;
pub mod region;
mod util;
pub mod dats;

#[cfg(test)]
mod tests {
    use crate::region::{Region, parse_regions};
    use crate::dats::GameEntry;
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }

    #[test]
    fn nointro_region_parses() {
        assert_eq!(parse_regions("USA, Europe"), vec![Region::UnitedStates, Region::Europe]);
        
    }
}

