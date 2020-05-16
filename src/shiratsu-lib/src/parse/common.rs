use crate::region::{Region, RegionError};
use std::fmt;
use std::fmt::Display;

#[derive(Debug)]
pub enum ParseError {
    ParseError(String),
    BadFileNameError(NamingConvention),
    RegionError(RegionError),
    HeaderMismatchError(&'static str, Option<String>),
}

impl From<RegionError> for ParseError {
    fn from(err: RegionError) -> Self {
        ParseError::RegionError(err)
    }
}

impl std::error::Error for ParseError {}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseError::ParseError(val) => write!(f,"{}", val),
            ParseError::BadFileNameError(convention) => 
                write!(f, "The provided file name could not be parsed properly in the {:?} naming convention", convention),
            ParseError::RegionError(region_err) =>
                write!(f, "{}", region_err),
            ParseError::HeaderMismatchError(expected, actual) =>
                write!(f, 
                    "Expected DAT to have header homepage \"{}\" but it actually was \"{}\". Use unchecked variants to ignore header checking.", 
                    expected, actual.as_deref().unwrap_or("None")),
        }
    }
}

pub type Result<T> = std::result::Result<T, ParseError>;


/// Describes a single file that is a part of a GameEntry
impl RomEntry {
    /// The MD5 Hash of the ROM
    pub fn hash_md5(&self) -> Option<&str> {
        self.md5.as_deref()
    }
    /// The SHA1 Hash of the ROM
    pub fn hash_sha1(&self) -> Option<&str> {
        self.sha1.as_deref()
    }
    /// The CRC hash of the ROM
    pub fn hash_crc(&self) -> Option<&str> {
        self.crc.as_deref()
    }
    /// The canonical file name of the ROM
    pub fn file_name(&self) -> &str {
        &self.file_name
    }
    /// The size of the ROM
    pub fn size(&self) -> u64 {
        self.size
    }
}

#[derive(Debug)]
pub struct RomEntry {
    /// The MD5 Hash of the ROM
    pub(super) md5: Option<String>,
    /// The SHA1 Hash of the ROM
    pub(super) sha1: Option<String>,
    /// The CRC hash of the ROM
    pub(super) crc: Option<String>,
    /// The canonical file name of the ROM
    pub(super) file_name: String,
    /// The size of the ROM
    pub(super) size: u64,
}

/// A single entry that describes a game, which may hold a collection of RomEntries
#[derive(Debug)]
pub struct GameEntry {
    /// The name of the game entry, as is.
    pub(super) entry_name: String,
    /// Any ROM entries that are part of this game entry.
    pub(super) rom_entries: Vec<RomEntry>,
    /// Any serials this game was released under.
    pub(super) serials: Vec<String>,
    /// The source of the game.
    pub(super) source: &'static str,
    /// Any information retrieved from the name of the game entry, if any.
    pub(super) info: Option<NameInfo>,
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
    pub fn serials(&self) -> &[String] {
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

#[derive(Debug)]
pub struct NameInfo {
    pub(super) release_name: String,
    pub(super) region: Vec<Region>,
    pub(super) part_number: Option<i32>,
    pub(super) version: Option<String>,
    pub(super) is_unlicensed: bool,
    pub(super) is_demo: bool,
    pub(super) status: DevelopmentStatus,
    pub(super) naming_convention: NamingConvention,

}

impl NameInfo {
    /// The region of the game.
    fn region(&self) -> &[Region] {
        &self.region
    }
    /// If this entry is split into multiple parts, the part number of this entry.
    fn part_number(&self) -> Option<i32> {
        self.part_number
    }
    /// Whether or not this game is unlicensed.
    fn is_unlicensed(&self) -> bool {
        self.is_unlicensed
    }
    /// Whether or not this game is a sample or a demo version of a full game.
    fn is_demo(&self) -> bool {
        self.is_demo
    }
    /// The version of the game entry.
    fn version(&self) -> Option<&str> {
        self.version.as_deref()
    }
    /// The name of the release, with all tags removed, and articles at the beginning of the title.
    fn release_name(&self) -> &str {
        &self.release_name.as_str()
    }
    /// The development status of the game entry.
    fn development_status(&self) -> DevelopmentStatus {
        self.status
    }
    /// The naming convention of the structuredly named filename.
    fn naming_convention(&self) -> NamingConvention {
        self.naming_convention
    }
}

impl Display for GameEntry {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "(game \"{}\" ", self.entry_name())?;
        write!(f, " [")?;
        if let Some(r) = self.rom_entries().iter().nth(0) {
            write!(f, "{}", r)?;
        }
        let mut iter = self.rom_entries().iter().skip(1).peekable();
        if iter.peek().is_some() {
            writeln!(f, "")?;
        }
        while let Some(r) = iter.next() {
            if iter.peek().is_some() {
                writeln!(f, "  {}", r)?;
            } else {
                write!(f, "  {}", r)?;
            }
        }
        writeln!(f, "]")?;
        writeln!(f, "  (serial {:?})", self.serials())?;
        if self.info().is_none() {
            writeln!(f, " (info None)")?;
        } else {
            writeln!(f, "  (info {})", self.info().unwrap())?;
        }
        write!(f, "  (source \"{}\"))", self.source())
    }
}

impl Display for NameInfo {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "")?;
        writeln!(f, "    (release \"{}\")", self.release_name())?;
        writeln!(
            f,
            "    (region {:?})",
            self.region()
                .iter()
                .map(|&r| r.into())
                .collect::<Vec<&str>>()
        )?;
        writeln!(f, "    (part {})", self.part_number().map(|i| format!("{}", i)).as_deref().unwrap_or("None"))?;
        writeln!(f, "    (version \"{}\")", self.version().unwrap_or("None"))?;
        writeln!(f, "    (status {:?})", self.development_status())?;
        writeln!(f, "    (is-demo? {})", self.is_demo())?;
        writeln!(f, "    (is-unlicensed? {})", self.is_unlicensed())?;
        write!(f, "    (naming {:?})", self.naming_convention())
    }
}

impl Display for RomEntry {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "(rom \"{}\" ", self.file_name())?;
        writeln!(f, "    (crc {}) ", self.hash_crc().unwrap_or("None"))?;
        writeln!(f, "    (md5 {}) ", self.hash_md5().unwrap_or("None"))?;
        writeln!(f, "    (sha1 {}) ", self.hash_sha1().unwrap_or("None"))?;
        write!(f, "    (size {}))", self.size())
    }
}

#[derive(Debug, Copy, Clone)]
/// The development status of a release.
pub enum DevelopmentStatus {
    /// A commercially released, or feature complete product.
    ///
    /// This is equivalent to a lack of a status tag in both NoIntro and TOSEC standards.
    /// If the (Sample) tag is present, the development status should be `DevelopmentStatus::Release`,
    /// with `StructuredlyName::is_demo()` returning true.
    Release,
    /// An unfinished, but mostly feature complete product, that may or may not have been intentionally released.
    ///
    /// In No-Intro, this is equivalent to the (Beta) flag. In TOSEC, (alpha), (beta), (preview), and (pre-release) all
    /// fall under this status.
    Prelease,
    /// An unreleased, unfinished product that was not released.
    ///
    /// In No-Intro, this is equivalent to the (Proto) flag. In TOSEC, this is equivalent to the (proto) flag.
    Prototype,
}

/// Naming convention commonly used by DAT producers.
#[derive(Debug, Copy, Clone)]
pub enum NamingConvention {
    /// Not a known naming convention
    Unknown,
    /// The naming convention used by The Old School Emulation Center
    ///
    /// Defined at https://www.tosecdev.org/tosec-naming-convention
    TOSEC,
    /// The naming convention used by No-Intro, redump.org, and others.
    ///
    /// Defined at https://datomatic.no-intro.org/stuff/The%20Official%20No-Intro%20Convention%20(20071030).pdf
    NoIntro,
}

/// Two RomEntries are PartialEq if they have matching hashes.
impl PartialEq for RomEntry {
    fn eq(&self, other: &RomEntry) -> bool {
        if let (Some(my_md5), Some(other_md5),
                Some(my_sha1), Some(other_sha1),
                Some(my_crc), Some(other_crc)) = 
                (self.hash_md5(), other.hash_md5(),
                self.hash_sha1(), other.hash_sha1(),
                self.hash_crc(), other.hash_crc()) {
            my_md5 == other_md5 
                    && my_sha1 == other_sha1
                    && my_crc == other_crc
        } else {
            false
        }
    }
}