use quick_xml::de::DeError as XmlError;
use serde::Deserialize;
use std::convert::{TryFrom, TryInto};

use crate::naming::tosec::TOSECNameable;
use crate::naming::*;
use crate::error::*;

#[derive(Debug, Deserialize, PartialEq)]
struct Rom {
    name: String,
    size: i64,
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
    type Error = ParseError;
    fn try_from(game: Game) -> Result<Self> {
        let rom = game.rom;
        let name = game.name;
        Ok(GameEntry {
            info: Some(NameInfo::try_from_tosec(&name)?),
            entry_name: name,
            serials: vec![],
            rom_entries: rom.into_iter().map(|r| r.into()).collect(),
            source: "TOSEC",
        })
    }
}

impl From<Rom> for RomEntry {
    fn from(mut rom: Rom) -> Self {
        rom.md5.make_ascii_lowercase();
        rom.crc.make_ascii_lowercase();
        rom.sha1.make_ascii_lowercase();

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
    wrap TosecParserError(XmlError) for ParseError {
        fn from (err) {
            ParseError::ParseError(format!("Error parsing TOSEC XML: {}", err.0.to_string()))
        }
    }
}

make_parse!("TOSEC", Game, TosecParserError);
make_from!("TOSEC", "https://www.tosecdev.org/", TOSEC, tosec);