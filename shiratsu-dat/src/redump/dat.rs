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
    crc: Option<String>,
    md5: Option<String>,
    sha1: Option<String>,
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
        let info = Some(NoIntroName::try_parse(&name)?.into());
        let serials = game
            .serial
            .map(|s| {
                s.split(",")
                    .map(|s| Serial::new(String::from(s.trim())))
                    .collect()
            })
            .unwrap_or(vec![]);
        let rom_entries = rom.into_iter().map(|r| r.into()).collect();
        Ok(GameEntry::new(
            name,
            None,
            rom_entries,
            serials,
            vec![],
            "Redump",
            info,
        ))
    }
}

impl From<Rom> for RomEntry {
    fn from(mut rom: Rom) -> Self {
        rom.md5
            .iter_mut()
            .for_each(|hash| hash.make_ascii_lowercase());
        rom.crc
            .iter_mut()
            .for_each(|hash| hash.make_ascii_lowercase());
        rom.sha1
            .iter_mut()
            .for_each(|hash| hash.make_ascii_lowercase());

        RomEntry {
            md5: rom.md5,
            sha1: rom.sha1,
            crc: rom.crc,
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
