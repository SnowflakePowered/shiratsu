#[derive(Debug)]
pub struct RomEntry {
    /// The MD5 Hash of the ROM
    pub(in super::super) md5: Option<String>,
    /// The SHA1 Hash of the ROM
    pub(in super::super) sha1: Option<String>,
    /// The CRC hash of the ROM
    pub(in super::super) crc: Option<String>,
    /// The canonical file name of the ROM
    pub(in super::super) file_name: String,
    /// The size of the ROM
    pub(in super::super) size: i64,
}

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
    pub fn size(&self) -> i64 {
        self.size
    }
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