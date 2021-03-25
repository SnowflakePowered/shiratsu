//!
//!
mod formats;
mod platforms;
mod file_ext;

pub use formats::*;
pub use platforms::{PlatformId, PlatformInfo, StoneError, StonePlatforms};

#[cfg(test)]
mod tests {
    use crate::{StonePlatforms, formats};
    use std::convert::TryInto;

    #[test]
    fn dump_stone() {
        for platform in StonePlatforms::get().infos() {
            println!("{:?}", platform);
        };
    }

    #[test]
    fn get_mimetype() {
        let mimetype = formats::find_mimetype("NINTENDO_NES".try_into().unwrap(),
                                              "ahjdhsad.NeS", None);
        assert_eq!(mimetype, Some("application/vnd.stone-romfile.nintendo.nes-ines"));
    }
}
