use quick_xml::de::DeError as XmlError;
use serde::Deserialize;
use std::convert::{TryFrom, TryInto};

use crate::error::*;

use super::super::xml::*;
use super::super::*;

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
            info: None,
            entry_name: name,
            serials: vec![],
            rom_entries: rom.into_iter().map(|r| r.into()).collect(),
            source: "dats.site",
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
    wrap DatsSiteParserError(XmlError) for ParseError {
        fn from (err) {
            ParseError::ParseError(format!("Error parsing dats.site XML: {}", err.0.to_string()))
        }
    }
}

make_parse!("Collectors Love It - The Custom Dats", Game, DatsSiteParserError);
make_from!("dats.site", "http://dats.site/", DatsSite, dats_site);