use serde::Deserialize;
use std::convert::{TryFrom, TryInto};
use crate::wrap_error;

use quick_xml::de::{from_str as from_xml, DeError as ParseError};
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
    serial: Option<Vec<String>>,
}
#[derive(Debug, Deserialize, PartialEq)]
struct Datafile {
    game: Vec<Game>
}

wrap_error! {
    wrap RedumpParserError(ParseError) for DatError {
        fn from (err) {
            DatError::ParseError(format!("Error parsing Redump XML: {}", err.0.to_string()))
        }
    }
}

pub fn parse(f: &str) -> Result<Vec<GameEntry>> {
    let d: Datafile = from_xml(f)
        .map_err::<RedumpParserError,_>(|e| e.into())?;
    d.game.into_iter().map(|g| g.try_into()).collect()
}

impl TryFrom<Game> for GameEntry {
    type Error = DatError;
    fn try_from(game: Game) -> Result<Self> {
        let rom = game.rom;
        let name = game.name;
        Ok(GameEntry {
            entry_name: name.clone(),
            info: Some(NameInfo::try_from_nointro(name).map(|n| n.into())?),
            serials: game.serial.unwrap_or(vec![]),
            rom_entries: rom.into_iter().map(|r|r.into( )).collect(),
            source: "Redump.org",
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