use serde::Deserialize;
use std::convert::{TryFrom, TryInto};
use crate::wrap_error;
use self::xml::*;
use quick_xml::de::{DeError as ParseError};
use super::super::*;

#[derive(Debug, Deserialize, PartialEq)]
struct Rom {
    name: String,
    size: u64,
    crc: String,
    md5: String,
    sha1: String,
}

#[derive(Debug, Deserialize, PartialEq)]
struct Game {
    name: String,
    rom: Vec<Rom>,
}

impl TryFrom<Game> for GameEntry {
    type Error = DatError;
    fn try_from(game: Game) -> Result<Self> {
        let rom = game.rom;
        let name = game.name;
        Ok(GameEntry {
            entry_name: name.clone(),
            info: Some(NameInfo::try_from_tosec(name).map(|n| n.into())?),
            serials: vec![],
            rom_entries: rom.into_iter().map(|r| r.into()).collect(),
            source: "TOSEC",
        })
    }
}

impl From<Rom> for RomEntry {
    fn from(rom: Rom) -> Self {
        RomEntry {
            md5: Some(rom.md5),
            sha1: Some(rom.sha1),
            crc: Some(rom.crc),
            file_name: rom.name,
            size: rom.size,
        }
    }
}

wrap_error! {
    wrap TosecParserError(ParseError) for DatError {
        fn from (err) {
            DatError::ParseError(format!("Error parsing TOSEC XML: {}", err.0.to_string()))
        }
    }
}

/// Parses the contents of a TOSEC XML DAT into a vector of `GameEntries`
/// This function will check that the 
/// XML has the proper header for TOSEC DATs. Use
/// `parse_tosec_unchecked` if you wish to ignore the header.
pub fn parse(f: &str) -> Result<Vec<GameEntry>> {
    parse_dat::<Game, TosecParserError>(f, Some("TOSEC"))?
            .game.into_iter().map(|g| g.try_into()).collect()
}

/// Parses the contents of a TOSEC XML DAT into a vector of `GameEntries`,
/// ignoring the header element.
pub fn parse_unchecked(f: &str) -> Result<Vec<GameEntry>> {
    parse_dat_unchecked::<Game, TosecParserError>(f)?.game.into_iter().map(|g| g.try_into()).collect()
}


/// Parses the contents of a TOSEC XML DAT into a vector of `GameEntries`
/// This function will check that the 
/// XML has the proper header for TOSEC DATs. Use
/// `parse_tosec_unchecked` if you wish to ignore the header.
pub fn parse_buf<R: std::io::BufRead>(f: R) -> Result<Vec<GameEntry>> {
    parse_dat_buf::<R, Game, TosecParserError>(f, Some("TOSEC"))?
            .game.into_iter().map(|g| g.try_into()).collect()
}

/// Parses the contents of a TOSEC XML DAT into a vector of `GameEntries`,
/// ignoring the header element.
pub fn parse_unchecked_buf<R: std::io::BufRead>(f: R) -> Result<Vec<GameEntry>> {
    parse_dat_unchecked_buf::<R, Game, TosecParserError>(f)?.game.into_iter().map(|g| g.try_into()).collect()
}