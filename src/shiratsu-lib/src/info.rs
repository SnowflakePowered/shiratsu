use crate::stone::PlatformId;
use crate::region::Region;

#[derive(Debug)]
pub struct SerialInfo {
    platform_id: PlatformId,
    title: String,
    file_name: String,
    region_code: Vec<Region>

}