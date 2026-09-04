use crate::{CueSheet, NameInfo, RomEntry, Serial};
use std::collections::HashSet;

/// A single entry that describes a game, which may hold a collection of RomEntries
#[derive(Debug)]
pub struct GameEntry {
    /// The name of the game entry, as is.
    entry_name: String,
    /// The unnormalized source title, if available.
    raw_title: Option<String>,
    /// Any ROM entries that are part of this game entry.
    rom_entries: Vec<RomEntry>,
    /// Any serials this game was released under.
    serials: Vec<Serial>,
    /// Any CUE sheets that describe the layout of this game entry's dumps.
    cue_sheets: Vec<CueSheet>,
    /// The source of the game.
    source: &'static str,
    /// Any information retrieved from the name of the game entry, if any.
    info: Option<NameInfo>,
}

/// A single entry that describes a game, which may hold a collection of RomEntries
impl GameEntry {
    /// Instantiates a GameEntry
    pub fn new(
        entry_name: String,
        raw_title: Option<String>,
        mut rom_entries: Vec<RomEntry>,
        serials: Vec<Serial>,
        cue_sheets: Vec<CueSheet>,
        source: &'static str,
        info: Option<NameInfo>,
    ) -> Self {
        let mut file_names = HashSet::new();
        rom_entries.retain(|rom| file_names.insert(rom.file_name().to_string()));
        Self {
            entry_name,
            raw_title,
            rom_entries,
            serials,
            cue_sheets,
            source,
            info,
        }
    }

    /// The name of the game entry, as is.
    pub fn entry_name(&self) -> &str {
        &self.entry_name
    }
    /// The unnormalized source title, if available.
    pub fn raw_title(&self) -> Option<&str> {
        self.raw_title.as_deref()
    }
    /// Any ROM entries that are part of this game entry.
    pub fn rom_entries(&self) -> &[RomEntry] {
        &self.rom_entries
    }
    /// Any serials this game was released under.
    pub fn serials(&self) -> &[Serial] {
        &self.serials
    }
    /// Any CUE sheets that describe the layout of this game entry's dumps.
    pub fn cue_sheets(&self) -> &[CueSheet] {
        &self.cue_sheets
    }
    /// The source of the game.
    pub fn source(&self) -> &str {
        self.source
    }
    pub fn info(&self) -> Option<&NameInfo> {
        self.info.as_ref()
    }
}
