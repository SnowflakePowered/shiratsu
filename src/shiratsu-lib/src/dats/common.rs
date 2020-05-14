use crate::region::{Region, RegionError};
use std::fmt;
use std::fmt::Debug;

#[derive(Debug)]
pub enum DatError {
    ParseError(String),
    BadFileNameError(NamingConvention),
    RegionError(RegionError)
}

impl From<RegionError> for DatError {
    fn from(err: RegionError) -> Self {
        DatError::RegionError(err)
    }
}

impl std::error::Error for DatError {}

impl std::fmt::Display for DatError  {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

pub type Result<T> = std::result::Result<T, DatError>;

/// Describes a single file that is a part of a GameEntry
impl RomEntry {
    /// The MD5 Hash of the ROM
    pub fn hash_md5(&self) -> Option<&str>
    {
        self.md5.as_deref()
    }
    /// The SHA1 Hash of the ROM
    pub fn hash_sha1(&self) -> Option<&str>
    {
        self.sha1.as_deref()
    }
    /// The CRC hash of the ROM
    pub fn hash_crc(&self) -> Option<&str>
    {
        self.crc.as_deref()
    }
    /// The canonical file name of the ROM
    pub fn file_name(&self) -> &str
    {
        &self.file_name
    }
    /// The size of the ROM
    pub fn size(&self) -> u64
    {
        self.size
    }
}

pub struct RomEntry {
    /// The MD5 Hash of the ROM
    pub (super) md5: Option<String>,
    /// The SHA1 Hash of the ROM
    pub (super) sha1: Option<String>,
    /// The CRC hash of the ROM
    pub (super) crc: Option<String>,
    /// The canonical file name of the ROM
    pub (super) file_name: String,
    /// The size of the ROM
    pub (super) size: u64,
}

/// A single entry that describes a game, which may hold a collection of RomEntries
pub struct GameEntry {
    /// The name of the game entry, as is.
    pub (super) entry_name: String,
    /// Any ROM entries that are part of this game entry.
    pub (super) rom_entries: Vec<RomEntry>,
    /// Any serials this game was released under.
    pub (super) serials: Vec<String>,
    /// The source of the game.
    pub (super) source: &'static str,
    /// Any information retrieved from the name of the game entry, if any.
    pub (super) info: Option<Box<dyn StructuredlyNamed>>
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
    pub fn info(&self) -> Option<&Box<dyn StructuredlyNamed>> {
        self.info.as_ref()
    }
}

impl Debug for GameEntry {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "(game {:?} ", self.entry_name())?;
        write!(f, " [")?;
        if let Some(r) = self.rom_entries().iter().nth(0) {
            write!(f, "{:?}", r)?;
        }
        let mut iter = self.rom_entries().iter().skip(1).peekable();
        if iter.peek().is_some() {
            writeln!(f, "")?;
        }
        while let Some(r) = iter.next() {
            if iter.peek().is_some() {
                writeln!(f, "  {:?}", r)?;
            } else {
                write!(f, "  {:?}", r)?;
            }
        }
        writeln!(f, "]")?;
        writeln!(f, "  (serial {:?})", self.serials())?;
        if self.info().is_none() {
            writeln!(f, " (info None)")?;
        } else {
            writeln!(f, "  (info {:?})", self.info().unwrap())?;
        }
        write!(f, "  (source {}))", self.source())
    }
}

impl Debug for dyn StructuredlyNamed {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "")?;
        writeln!(f, "    (release \"{}\")", self.release_name())?;
        writeln!(f, "    (region {:?})", self.region().iter().map(|&r| r.into()).collect::<Vec<&str>>())?;
        writeln!(f, "    (part {:?})", self.part_number())?;
        writeln!(f, "    (version {:?})", self.version())?;
        writeln!(f, "    (status {:?})", self.development_status())?;
        writeln!(f, "    (is-demo? {})", self.is_demo())?;
        writeln!(f, "    (is-unlicensed? {})", self.is_unlicensed())?;
        write!(f, "    (naming {:?})", self.naming_convention())
    }
}

impl Debug for RomEntry {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "(rom \"{}\" ", self.file_name())?;
        writeln!(f, "    (crc {}) ", self.hash_crc().unwrap_or("None"))?;
        writeln!(f, "    (md5 {}) ", self.hash_md5().unwrap_or("None"))?;
        writeln!(f, "    (sha1 {}) ", self.hash_sha1().unwrap_or("None"))?;
        write!(f, "    (size {}))", self.size())
    }
}

impl <T: StructuredlyNamed + 'static> From<T> for Box<dyn StructuredlyNamed> {
    fn from(val: T) -> Self {
        Box::new(val)
    }
}

/// Extra props for a GameEntry that follows a naming convention.
pub trait StructuredlyNamed {
    /// The region of the game.
    fn region(&self) -> &[Region];
    /// If this entry is split into multiple parts, the part number of this entry.
    fn part_number(&self) -> Option<i32>;
    /// Whether or not this game is unlicensed.
    fn is_unlicensed(&self) -> bool;
    /// Whether or not this game is a sample or a demo version of a full game.
    fn is_demo(&self) -> bool;
    /// The version of the game entry.
    fn version(&self) -> Option<&str>;
    /// The name of the release, with all tags removed, and articles at the beginning of the title.
    fn release_name(&self) -> &str;
    /// The development status of the game entry.
    fn development_status(&self) -> DevelopmentStatus;
    /// The naming convention of the structuredly named filename.
    fn naming_convention(&self) -> NamingConvention;
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
         (self.hash_md5().is_some() && other.hash_md5().is_some() && self.hash_md5().unwrap() == other.hash_md5().unwrap()) &&
         (self.hash_sha1().is_some() && other.hash_sha1().is_some() && self.hash_sha1().unwrap() == other.hash_sha1().unwrap())  &&
         (self.hash_crc().is_some() && other.hash_crc().is_some() && self.hash_crc().unwrap() == other.hash_crc().unwrap()) 
    }
}