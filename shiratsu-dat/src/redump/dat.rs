use crate::error::*;

use quick_xml::de::DeError as XmlError;
use serde::Deserialize;
use std::convert::{TryFrom, TryInto};

use shiratsu_naming::naming::nointro::NoIntroName;
use shiratsu_naming::naming::TokenizedName;

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
    serial: Option<String>,
}

impl TryFrom<Game> for GameEntry {
    type Error = DatError;
    fn try_from(game: Game) -> Result<Self> {
        let rom = game.rom;
        let name = game.name;
        Ok(GameEntry {
            info: Some(NoIntroName::try_parse(&name)?.into()),
            entry_name: name,
            serials: game
                .serial
                .map(|s| {
                    s.split(",")
                        .map(|s| Serial::new(String::from(s.trim())))
                        .collect()
                })
                .unwrap_or(vec![]),
            rom_entries: rom.into_iter().map(|r| r.into()).collect(),
            source: "Redump",
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
    wrap RedumpParserError(XmlError) for DatError{
        fn from (err) {
            DatError::ParseError(format!("Error parsing Redump XML: {}", err.0.to_string()))
        }
    }
}
make_parse!("redump.org", Game, RedumpParserError);
make_from!("Redump", "http://redump.org/", Redump, redump);