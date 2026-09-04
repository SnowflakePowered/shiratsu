use crate::error::*;

use serde::Deserialize;
use std::convert::{TryFrom, TryInto};

use quick_xml::de::DeError as XmlError;

use shiratsu_naming::naming::nointro::*;
use shiratsu_naming::naming::TokenizedName;

#[derive(Debug, Deserialize, PartialEq)]
struct Rom {
    name: String,
    size: Option<i64>,
    crc: Option<String>,
    md5: Option<String>,
    sha1: Option<String>,
    serial: Option<String>,
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
        let info = Some(NoIntroName::try_parse(&name)?.into());
        let serials = rom
            .iter()
            .filter_map(|r| r.serial.clone())
            .flat_map(|s| {
                s.split(",")
                    .map(|s| Serial::new(String::from(s.trim())))
                    .collect::<Vec<_>>()
            })
            .collect();
        let rom_entries = rom.into_iter().map(|r| r.into()).collect();
        Ok(GameEntry::new(
            name,
            None,
            rom_entries,
            serials,
            vec![],
            "No-Intro",
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
            size: rom.size.unwrap_or(0),
        }
    }
}

wrap_error! {
    wrap NoIntroParserError(XmlError) for DatError {
        fn from (err) {
            DatError::ParseError(format!("Error parsing No-Intro XML: {}", err.0.to_string()))
        }
    }
}

make_parse!("No-Intro", Game, NoIntroParserError);
make_from!(
    "No-Intro",
    "https://datomatic.no-intro.org/",
    NoIntro,
    nointro
);
