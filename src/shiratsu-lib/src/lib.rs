extern crate lazy_static_include;
extern crate lazy_static;
extern crate serde;
extern crate serde_json;
extern crate phf;

pub mod stone;
pub mod region;
pub mod util;
pub mod info;
pub mod dats;

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
}

