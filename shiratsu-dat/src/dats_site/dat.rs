use quick_xml::de::DeError as XmlError;
use serde::Deserialize;
use std::convert::{TryFrom, TryInto};

use crate::error::*;

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
}

impl TryFrom<Game> for GameEntry {
    type Error = DatError;
    fn try_from(game: Game) -> Result<Self> {
        let rom = game.rom;
        let name = game.name;
        let info = Some(NoIntroName::try_parse(&name)?.into());
        let rom_entries = rom.into_iter().map(|r| r.into()).collect();
        Ok(GameEntry::new(
            name,
            None,
            rom_entries,
            vec![],
            vec![],
            "dats.site",
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
    wrap DatsSiteParserError(XmlError) for DatError {
        fn from (err) {
            DatError::ParseError(format!("Error parsing The Custom DATs XML: {}", err.0.to_string()))
        }
    }
}

make_parse!(
    "Collectors Love It - The Custom Dats",
    Game,
    DatsSiteParserError
);
make_from!(
    "Collectors Love It - The Custom DATs",
    "http://dats.site/",
    DatsSite,
    dats_site
);
