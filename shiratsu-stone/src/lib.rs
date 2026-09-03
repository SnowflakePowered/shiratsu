//!
//!
mod file_ext;
mod formats;
mod platforms;

pub use formats::*;
pub use platforms::{PlatformId, PlatformInfo, StoneError, StonePlatforms};

#[cfg(test)]
mod tests {
    use crate::{formats, StonePlatforms};
    use std::convert::TryInto;

    #[test]
    fn dump_stone() {
        for platform in StonePlatforms::get().infos() {
            println!("{:?}", platform);
        }
    }

    #[test]
    fn get_mimetype() {
        let mimetype =
            formats::find_mimetype("NINTENDO_NES".try_into().unwrap(), "ahjdhsad.NeS", None);
        assert_eq!(
            mimetype,
            Some("application/vnd.stone-romfile.nintendo.nes-ines")
        );
    }
}
