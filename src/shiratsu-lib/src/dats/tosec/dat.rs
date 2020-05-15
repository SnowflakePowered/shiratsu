use serde::Deserialize;
use std::convert::{TryFrom, TryInto};

use super::super::Result;
use super::super::*;
use quick_xml::de::{from_str as from_xml, DeError as ParseError};

#[derive(Debug, Deserialize, PartialEq)]
struct Rom {
    name: String,
    size: u64,
    crc: String,
    md5: String,
    sha1: String,
    serial: Option<String>,
}

#[derive(Debug, Deserialize, PartialEq)]
struct Game {
    name: String,
    rom: Vec<Rom>,
}
#[derive(Debug, Deserialize, PartialEq)]
struct Datafile {
    game: Vec<Game>,
}

// Private wrapping error for ParseError
struct NoIntroParserError(ParseError);
impl From<ParseError> for NoIntroParserError {
    fn from(err: ParseError) -> Self {
        NoIntroParserError(err)
    }
}

impl From<NoIntroParserError> for DatError {
    fn from(err: NoIntroParserError) -> Self {
        DatError::ParseError(format!("Error parsing No-Intro XML: {}", err.0.to_string()))
    }
}

pub fn parse(f: &str) -> Result<Vec<GameEntry>> {
    let d: Datafile = from_xml(f).map_err::<NoIntroParserError, _>(|e| e.into())?;
    d.game.into_iter().map(|g| g.try_into()).collect()
}

impl TryFrom<Game> for GameEntry {
    type Error = DatError;
    fn try_from(game: Game) -> Result<Self> {
        let rom = game.rom;
        let name = game.name;
        Ok(GameEntry {
            entry_name: name.clone(),
            info: Some(NameInfo::try_from_tosec(name).map(|n| n.into())?),
            serials: rom.iter().filter_map(|r| r.serial.clone()).collect(),
            rom_entries: rom.into_iter().map(|r| r.into()).collect(),
            source: "No-Intro",
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
