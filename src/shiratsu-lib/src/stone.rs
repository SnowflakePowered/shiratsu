use lazy_static::lazy_static;
use lazy_static_include::{
    lazy_static_include_str, lazy_static_include_str_impl, lazy_static_include_str_inner,
};

use serde;
use serde::Deserialize;
use serde_json;
use serde_json::Value;
use std::collections::HashMap;
use std::io;

type Result<T> = std::result::Result<T, StoneError>;

lazy_static_include_str!(STONE_DIST, "../../stone/dist/stone.dist.json");
lazy_static! {
    pub static ref STONE: HashMap<PlatformId, PlatformInfo> = load_platform_info().unwrap();
}

#[derive(Debug)]
pub enum StoneError {
    Deserialization(serde_json::Error),
    Io(io::Error),
    InvalidStoneFile,
    InvalidPlatformId(String),
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
#[derive(Debug, Deserialize, Eq, PartialEq, Hash)]
pub struct PlatformId(String);

impl From<&dyn AsRef<str>> for PlatformId {
    fn from(platform_id_str: &dyn AsRef<str>) -> PlatformId {
        let result = platform_id_str.as_ref();
        PlatformId(result.to_ascii_uppercase())
    }
}

#[derive(Debug, Deserialize)]
pub struct PlatformInfo {
    #[serde(rename(deserialize = "PlatformID"))]
    platform_id: PlatformId,
    #[serde(rename(deserialize = "FileTypes"))]
    file_types: HashMap<String, String>,
    #[serde(rename(deserialize = "MaximumInputs"))]
    maximum_inputs: i32,
    #[serde(rename(deserialize = "BiosFiles"))]
    bios_files: Option<HashMap<String, Vec<String>>>,
    #[serde(rename(deserialize = "FriendlyName"))]
    friendly_name: String,
    #[serde(rename(deserialize = "Metadata"))]
    metadata: HashMap<String, String>,
}

fn load_platform_info() -> Result<HashMap<PlatformId, PlatformInfo>> {
    let stone_data: Value = serde_json::from_str(*STONE_DIST)?;
    let platform_data = stone_data
        .get("Platforms")
        .ok_or(StoneError::InvalidStoneFile)?;
    let value = serde_json::from_value::<HashMap<PlatformId, PlatformInfo>>(platform_data.clone())?;
    Ok(value)
}

pub fn get_stone() -> &'static HashMap<PlatformId, PlatformInfo> {
    &STONE
}
