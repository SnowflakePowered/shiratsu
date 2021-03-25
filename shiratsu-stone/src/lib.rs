mod formats;
mod platforms;
pub use formats::*;
pub use platforms::{PlatformId, PlatformInfo, StoneError, StonePlatforms};

#[cfg(test)]
mod tests {
    use crate::StonePlatforms;

    #[test]
    fn dump_stone() {
        for platform in StonePlatforms::get().ids() {
            println!("{:?}", platform);
        };
    }
}
