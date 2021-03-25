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
use crate::file_ext::FileExt;

type Result<T> = std::result::Result<T, StoneError>;

lazy_static_include_str!(STONE_DIST, "./stone/dist/stone.dist.json");
lazy_static! {
    pub static ref STONE: (StonePlatforms, String) = load_platform_info().unwrap();
}

pub struct StonePlatforms {
    platform_info: HashMap<PlatformId, PlatformInfo>,
}

impl StonePlatforms {
    #[doc(hidden)]
    fn new(platform_info: HashMap<PlatformId, PlatformInfo>) -> StonePlatforms {
        StonePlatforms { platform_info }
    }

    /// Gets the list of Stone platform definitions.
    pub fn get() -> &'static StonePlatforms {
        &STONE.0
    }

    /// Gets a reference to the Stone Platform ID matching the given string.
    fn get_platform_id(&self, id: &str) -> Option<&PlatformId> {
        self.platform_info
            .keys()
            .find(|&platform_id| platform_id.0 == id)
    }

    /// Gets the platform with the specified platform ID.
    /// Returns `StoneError::NoSuchPlatform` if it does not.
    pub fn platform(&self, platform_id: &PlatformId) -> Result<&PlatformInfo> {
        self.platform_info
            .get(platform_id)
            .ok_or(StoneError::NoSuchPlatform(platform_id.clone()))
    }

    /// Get an iterator of PlatformIDs in the listed definitions.
    pub fn ids(&self) -> impl Iterator<Item=&PlatformId> {
        self.platform_info.keys()
    }

    /// Get an iterator of `PlatformInfo`s in the listed definitions.
    pub fn infos(&self) -> impl Iterator<Item=&PlatformInfo> {
        self.platform_info.values()
    }

    /// Get the version of Stone.
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
/// A Stone Platform ID.
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
        if let Some(platform_id_ref) = stone.get_platform_id(&platform_id_str) {
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
        if let Some(platform_id_ref) = stone.get_platform_id(&platform_id_str) {
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
        if let Some(platform_id_ref) = stone.get_platform_id(platform_id_str.as_ref()) {
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
    file_types: HashMap<FileExt, String>,
    #[serde(rename(deserialize = "MaximumInputs"))]
    maximum_inputs: i32,
    #[serde(rename(deserialize = "BiosFiles"))]
    bios_files: Option<HashMap<String, HashSet<String>>>,
    #[serde(rename(deserialize = "FriendlyName"))]
    friendly_name: String,
    #[serde(rename(deserialize = "Metadata"))]
    metadata: HashMap<String, String>,
}

impl PlatformInfo {
    /// Gets the Platform ID
    pub fn platform_id(&self) -> &PlatformId {
        &self.platform_id
    }
    pub fn file_exts(&self) -> impl Iterator<Item=&str> {
        self.file_types.keys().map(|s| s.as_ref())
    }

    pub fn maximum_inputs(&self) -> i32 {
        self.maximum_inputs
    }

    pub fn metadata(&self) -> impl Iterator<Item=(&str, &str)>
    {
        self.metadata.iter()
            .map(|(k, v)| (k.as_str(), v.as_str()))
    }

    pub fn mimetypes(&self) -> impl Iterator<Item=&str> {
        self.file_types.values().map(|s| s.as_str())
    }
    pub fn bios_file_names(&self) -> impl Iterator<Item=&str> {
        self.bios_files
            .iter()
            .flat_map(|s| s.keys())
            .map(|s| s.as_str())
    }
    pub fn bios_file_hashes<'a, S: AsRef<str>>(&'a self, hash: &'a S) -> impl Iterator<Item=&'a str> {
        self.bios_files
            .iter()
            .flat_map(move |s| s.get(hash.as_ref()))
            .flat_map(|s| s)
            .map(|s| s.as_str())
    }
    pub fn get_mimetype_for_ext<S: AsRef<str>>(&self, ext: S) -> Option<&str> {
        self.file_types.get(&ext.as_ref().into()).map(|s| s.as_str())
    }
    pub fn friendly_name(&self) -> &str {
        &self.friendly_name
    }
    pub fn is_bios_md5<S: AsRef<str>>(&self, hash: S) -> bool {
        if let Some(bios_files) = &self.bios_files {
            bios_files.values().any(|v| v.contains(hash.as_ref()))
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
    let value =
        serde_json::from_value::<HashMap<PlatformId, PlatformInfo>>(platform_data.clone())?;

    Ok((StonePlatforms::new(value), version))
}