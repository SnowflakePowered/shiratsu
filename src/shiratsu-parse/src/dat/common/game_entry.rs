use crate::dat::*;
use crate::naming::*;

/// A single entry that describes a game, which may hold a collection of RomEntries
#[derive(Debug)]
pub struct GameEntry {
    /// The name of the game entry, as is.
    pub(in super::super) entry_name: String,
    /// Any ROM entries that are part of this game entry.
    pub(in super::super) rom_entries: Vec<RomEntry>,
    /// Any serials this game was released under.
    pub(in super::super) serials: Vec<Serial>,
    /// The source of the game.
    pub(in super::super) source: &'static str,
    /// Any information retrieved from the name of the game entry, if any.
    pub(in super::super) info: Option<NameInfo>,
}

/// A single entry that describes a game, which may hold a collection of RomEntries
impl GameEntry {
    /// The name of the game entry, as is.
    pub fn entry_name(&self) -> &str {
        &self.entry_name
    }
    /// Any ROM entries that are part of this game entry.
    pub fn rom_entries(&self) -> &[RomEntry] {
        &self.rom_entries
    }
    /// Any serials this game was released under.
    pub fn serials(&self) -> &[Serial] {
        &self.serials
    }
    /// The source of the game.
    pub fn source(&self) -> &str {
        self.source
    }
    pub fn info(&self) -> Option<&NameInfo> {
        self.info.as_ref()
    }
}