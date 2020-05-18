use crate::stone::{StonePlatforms, PlatformId};
use crate::parse::RomEntry;
use std::path::Path;
pub trait FindRomMimetype {
    fn find_mimetype(&self, platform: &PlatformId) -> Option<&str>;
}

impl FindRomMimetype for RomEntry {
    fn find_mimetype(&self, platform_id: &PlatformId) -> Option<&str> {
        StonePlatforms::get()
            .platform(platform_id).ok()
                .and_then(|platform_info| Path::new(self.file_name()).extension()
                    .and_then(|s| s.to_str())
                    .and_then(|ext| platform_info.get_mimetype_for_ext(ext)))
    }
}