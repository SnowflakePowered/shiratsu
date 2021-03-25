use crate::{StonePlatforms, PlatformId};
use std::path::Path;

pub fn find_mimetype<'a, 'b>(platform_id: &PlatformId, file_name: &'a str, md5: Option<&'a str>)
    -> Option<&'b str>
{
    StonePlatforms::get()
        .platform(platform_id).ok()
        .and_then(|platform_info| {
            if let Some(md5) = md5 {
                if platform_info.is_bios_md5(md5) {
                    return platform_info.get_mimetype_for_ext("BIOS")
                }
            }
            Path::new(file_name).extension()
                .and_then(|s| s.to_str())
                .and_then(|ext| platform_info.get_mimetype_for_ext(ext))
                .or(platform_info.get_mimetype_for_ext("RSRC"))
        })
}
