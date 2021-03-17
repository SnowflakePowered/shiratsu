use std::fmt;
use super::{GameEntry, RomEntry};

impl fmt::Display for GameEntry {
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

impl fmt::Display for RomEntry {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f, "(rom \"{}\" ", self.file_name())?;
        writeln!(f, "    (crc {}) ", self.hash_crc().unwrap_or("None"))?;
        writeln!(f, "    (md5 {}) ", self.hash_md5().unwrap_or("None"))?;
        writeln!(f, "    (sha1 {}) ", self.hash_sha1().unwrap_or("None"))?;
        write!(f, "    (size {}))", self.size())
    }
}