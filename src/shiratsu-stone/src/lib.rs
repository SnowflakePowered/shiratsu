mod formats;
mod platforms;
pub use formats::*;
pub use platforms::{PlatformId, PlatformInfo, StoneError, StonePlatforms};

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
