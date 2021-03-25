use crate::region::Region;
use crate::naming::*;

#[derive(Debug, Eq, PartialEq)]
pub struct NameInfo {
    pub(crate) entry_title: String,
    pub(crate) release_title: String,
    pub(crate) region: Vec<Region>,
    pub(crate) part_number: Option<i32>,
    pub(crate) version: Option<String>,
    pub(crate) is_unlicensed: bool,
    pub(crate) is_demo: bool,
    pub(crate) is_system: bool,
    pub(crate) status: DevelopmentStatus,
    pub(crate) naming_convention: NamingConvention,
}

impl NameInfo {
    /// The region of the game.
    pub fn region(&self) -> &[Region] {
        &self.region
    }
    /// If this entry is split into multiple parts, the part number of this entry.
    pub fn part_number(&self) -> Option<i32> {
        self.part_number
    }
    /// Whether or not this game is unlicensed.
    pub fn is_unlicensed(&self) -> bool {
        self.is_unlicensed
    }
    /// Whether or not this game is a sample or a demo version of a full game.
    pub fn is_demo(&self) -> bool {
        self.is_demo
    }
    /// Whether or not this game is a system update or BIOS file.
    pub fn is_system(&self) -> bool {
        self.is_system
    }
    /// The version of the game entry.
    pub fn version(&self) -> Option<&str> {
        self.version.as_deref()
    }
    /// The name of the release, with all tags removed, and articles at the beginning of the title.
    pub fn release_title(&self) -> &str {
        &self.release_title.as_str()
    }
    /// The name of the release as it appears in the filename.
    pub fn entry_title(&self) -> &str {
        &self.entry_title.as_str()
    }
    /// The development status of the game entry.
    pub fn development_status(&self) -> DevelopmentStatus {
        self.status
    }
    /// The naming convention of the structuredly named filename.
    pub fn naming_convention(&self) -> NamingConvention {
        self.naming_convention
    }
}

pub trait ToNameInfo
{
    /// Creates a new `NameInfo` object from the name data.
    fn to_name_info(&self) -> NameInfo;
}

impl<'a, T> From<T> for NameInfo
    where T: ToNameInfo
{
    fn from(name: T) -> Self {
        name.to_name_info()
    }
}
