use lazy_static::lazy_static;
#[allow(unused_imports)]
use lazy_static_include::{
    lazy_static_include_str, lazy_static_include_str_impl, lazy_static_include_str_inner,
};

use serde;
use serde::Deserialize;
use serde_json;
use serde_json::Value;
use std::collections::{HashMap, HashSet};
use std::convert::TryFrom;
use std::io;

type Result<T> = std::result::Result<T, StoneError>;

lazy_static_include_str!(STONE_DIST, "../../stone/dist/stone.dist.json");
lazy_static! {
    pub static ref STONE: (StonePlatforms, String) = load_platform_info().unwrap();
}

pub struct StonePlatforms {
    platform_info: HashMap<PlatformId, PlatformInfo>,
}

impl StonePlatforms {
    fn new(platform_info: HashMap<PlatformId, PlatformInfo>) -> StonePlatforms {
        StonePlatforms { platform_info }
    }

    fn get_platform_id_ref(&self, id: &str) -> Option<&PlatformId> {
        self.platform_info
            .keys()
            .find(|&platform_id| platform_id.0 == id)
    }

    /// Gets the platform with the specified platform ID.
    /// Returns StoneError::NoSuchPlatform if it does not.
    pub fn platform(&self, platform_id: &PlatformId) -> Result<&PlatformInfo> {
        self.platform_info
            .get(platform_id)
            .ok_or(StoneError::NoSuchPlatform(platform_id.clone()))
    }
    
    /// Gets a reference to the global list of Stone platforms embedded in the library.
    pub fn get() -> &'static StonePlatforms {
        &STONE.0
    }

    pub fn ids(&self) -> Vec<&PlatformId> {
        self.platform_info.keys().collect()
    }

    pub fn version() -> &'static str {
        &STONE.1
    }
}

#[derive(Debug)]
pub enum StoneError {
    Deserialization(serde_json::Error),
    Io(io::Error),
    InvalidStoneFile,
    InvalidPlatformId(String),
    NoSuchPlatform(PlatformId),
}

impl From<serde_json::Error> for StoneError {
    fn from(err: serde_json::Error) -> StoneError {
        StoneError::Deserialization(err)
    }
}

impl From<io::Error> for StoneError {
    fn from(err: io::Error) -> StoneError {
        StoneError::Io(err)
    }
}

impl std::error::Error for StoneError {}

impl std::fmt::Display for StoneError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}
#[derive(Debug, Deserialize, Eq, PartialEq, Hash, Clone)]
pub struct PlatformId(String);

impl AsRef<str> for PlatformId {
    fn as_ref(&self) -> &str{
        &self.0
    }
}

impl TryFrom<String> for &'static PlatformId {
    type Error = StoneError;
    fn try_from(platform_id_str: String) -> Result<&'static PlatformId> {
        let stone = StonePlatforms::get();
        if let Some(platform_id_ref) = stone.get_platform_id_ref(&platform_id_str) {
            Ok(platform_id_ref)
        } else {
            Err(StoneError::InvalidPlatformId(platform_id_str))
        }
    }
}

impl TryFrom<&String> for &'static PlatformId {
    type Error = StoneError;
    fn try_from(platform_id_str: &String) -> Result<&'static PlatformId> {
        let stone =StonePlatforms::get();
        if let Some(platform_id_ref) = stone.get_platform_id_ref(&platform_id_str) {
            Ok(platform_id_ref)
        } else {
            Err(StoneError::InvalidPlatformId(String::from(platform_id_str)))
        }
    }
}

impl TryFrom<&str> for &'static PlatformId {
    type Error = StoneError;
    fn try_from(platform_id_str: &str) -> Result<&'static PlatformId> {
        let result = platform_id_str;
        let stone = StonePlatforms::get();
        if let Some(platform_id_ref) = stone.get_platform_id_ref(platform_id_str.as_ref()) {
            Ok(platform_id_ref)
        } else {
            Err(StoneError::InvalidPlatformId(String::from(result)))
        }
    }
}

/// Describes a platform's ID and file types in Stone
#[derive(Debug, Deserialize)]
pub struct PlatformInfo {
    #[serde(rename(deserialize = "PlatformID"))]
    platform_id: PlatformId,
    #[serde(rename(deserialize = "FileTypes"))]
    file_types: HashMap<String, String>,

    // Don't need the rest, so we can save some space.

    // #[serde(rename(deserialize = "MaximumInputs"))]
    // maximum_inputs: i32,
    #[serde(rename(deserialize = "BiosFiles"))]
    bios_files: Option<HashMap<String, HashSet<String>>>,
    // #[serde(rename(deserialize = "FriendlyName"))]
    // friendly_name: String,
    // #[serde(rename(deserialize = "Metadata"))]
    // metadata: HashMap<String, String>,
}

impl PlatformInfo {
    /// Gets a reference to the 
    pub fn platform_id(&self) -> &PlatformId {
        &self.platform_id
    }
    pub fn file_exts(&self) -> Vec<&str> {
        self.file_types.keys().map(|s| s.as_str()).collect()
    }
    pub fn mimetypes(&self) -> Vec<&str> {
        self.file_types.values().map(|s| s.as_str()).collect()
    }
    pub fn get_mimetype_for_ext(&self, ext: &str) -> Option<&str> {
        self.file_types.get(ext).map(|s| s.as_str())
    }
    pub fn friendly_name(&self) -> Vec<&str> {
        self.file_types.values().map(|s| s.as_str()).collect()
    }
    pub fn is_bios_md5(&self, hash: &str) -> bool {
        if let Some(bios_files) = &self.bios_files {
            bios_files.values().any(|v| v.contains(hash))
        } else {
            false
        }
    }
}

fn load_platform_info() -> Result<(StonePlatforms, String)> {
    let stone_data: Value = serde_json::from_str(*STONE_DIST)?;
    let platform_data = stone_data
        .get("Platforms")
        .ok_or(StoneError::InvalidStoneFile)?;
    let version = stone_data.get("version")
        .and_then(|val|val.as_str())
        .map(|val| String::from(val))
        .ok_or(StoneError::InvalidStoneFile)?;
    let mut value =
        serde_json::from_value::<HashMap<PlatformId, PlatformInfo>>(platform_data.clone())?;
    for (_id, platform) in value.iter_mut() {
        let file_type_clone = platform.file_types.clone();
        for (ext, mime) in file_type_clone.into_iter() {
            if ext == "BIOS" || ext == "RSRC" {
                // Skip BIOS or RSRC mimetype
                continue;
            }
            // Duplicating the mimetypes once is cheap enough to easily support
            // dotless extensions.
            platform.file_types.insert(String::from(&ext[1..]), mime.clone());

            // Duplicate mimetypes twice for uppercase support, so we don't have to
            // unnecesarily duplicate a string for to_lowercase.
            platform.file_types.insert(ext[1..].to_ascii_uppercase(), mime.clone());
            platform.file_types.insert(ext.to_ascii_uppercase(), mime.clone());
        }
    }
    Ok((StonePlatforms::new(value), version))
}