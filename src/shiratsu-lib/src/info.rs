use crate::region::Region;

/// Describes a single file that is a part of a GameEntry
pub trait RomEntry {
    /// The MD5 Hash of the ROM
    fn hash_md5(&self) -> Option<&str>;
    /// The SHA1 Hash of the ROM
    fn hash_sha1(&self) -> Option<&str>;
    /// The CRC hash of the ROM
    fn hash_crc(&self) -> Option<&str>;
    /// The canonical file name of the ROM
    fn file_name(&self) -> &str;
    /// The size of the ROM
    fn size(&self) -> u64;
}

/// A single entry that describes a game, which may hold a collection of RomEntries
pub trait GameEntry {
    /// The name of the game entry, as is.
    fn entry_name(&self) -> Option<&str>;
    /// Any ROM entries that are part of this game entry.
    fn rom_entries(&self) -> &Vec<&dyn RomEntry>;
    /// Any serials this game was released under.
    fn serials(&self) -> &Vec<&str>;
    /// The source of the game.
    fn source(&self) -> &str;
}

/// Extra props for a GameEntry that follows a naming convention.
pub trait StructuredlyNamed: GameEntry {
    /// The region of the game.
    fn region(&self) -> &Vec<Region>;
    /// If this entry is split into multiple parts, the part number of this entry.
    fn part_number(&self) -> Option<i32>;
    /// Whether or not this game is unlicensed.
    fn is_unlicensed(&self) -> bool;
    /// Whether or not this game is a sample or a demo version of a full game.
    fn is_demo(&self) -> bool;
    /// The version of the game entry.
    fn version(&self) -> Option<&str>;
    /// The name of the release, with all tags removed, and articles at the beginning of the title.
    fn release_name(&self) -> Option<&str>;
    /// The development status of the game entry.
    fn development_status(&self) -> DevelopmentStatus;
    /// The naming convention of the structuredly named filename.
    fn naming_convention(&self) -> NamingConvention;
}

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
#[derive(Debug)]
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
impl PartialEq for dyn RomEntry {
    fn eq(&self, other: &dyn RomEntry) -> bool {
         (self.hash_md5().is_some() && other.hash_md5().is_some() && self.hash_md5().unwrap() == other.hash_md5().unwrap()) &&
         (self.hash_sha1().is_some() && other.hash_sha1().is_some() && self.hash_sha1().unwrap() == other.hash_sha1().unwrap())  &&
         (self.hash_crc().is_some() && other.hash_crc().is_some() && self.hash_crc().unwrap() == other.hash_crc().unwrap()) 
    }
}

