use crate::region::Region;
use super::*;

#[derive(Debug, Eq, PartialEq)]
pub struct NameInfo {
    pub(in super::super) entry_title: String,
    pub(in super::super) release_title: String,
    pub(in super::super) region: Vec<Region>,
    pub(in super::super) part_number: Option<i32>,
    pub(in super::super) version: Option<String>,
    pub(in super::super) is_unlicensed: bool,
    pub(in super::super) is_demo: bool,
    pub(in super::super) status: DevelopmentStatus,
    pub(in super::super) naming_convention: NamingConvention,
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


