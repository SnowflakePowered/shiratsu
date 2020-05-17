use crate::wrap_error;
use quick_xml::de::DeError as XmlError;
use serde::Deserialize;
use std::convert::{TryFrom, TryInto};

use super::super::xml::*;
use super::super::*;

#[derive(Debug, Deserialize, PartialEq)]
struct Rom {
    name: String,
    size: u32,
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
            source: "Generic",
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
    wrap GenericParserError(XmlError) for ParseError {
        fn from (err) {
            ParseError::ParseError(format!("Error parsing DAT XML: {}", err.0.to_string()))
        }
    }
}

fn parse_unchecked(f: &str) -> Result<Vec<GameEntry>> {
    parse_dat_unchecked::<Game, GenericParserError>(f)?
        .game
        .into_iter()
        .map(|g| g.try_into())
        .collect()
}

fn parse_unchecked_buf<R: std::io::BufRead>(f: R) -> Result<Vec<GameEntry>> {
    parse_dat_unchecked_buf::<R, Game, GenericParserError>(f)?
        .game
        .into_iter()
        .map(|g| g.try_into())
        .collect()
}

/// Provides methods that parse an XML .dat files from [TOSEC](https://www.tosecdev.org/)
pub trait FromGeneric {

    /// Parses the contents of a generic DAT XML
    fn try_from_str(dat: &str) -> Result<Vec<GameEntry>>;

    /// Parses the contents of a generic DAT xml
    fn try_from_buf<R: std::io::BufRead>(buf: R) -> Result<Vec<GameEntry>>;
}

impl FromGeneric for GameEntry {
    fn try_from_str(dat: &str) -> Result<Vec<GameEntry>> {
        parse_unchecked(dat)
    }
    fn try_from_buf<R: std::io::BufRead>(buf: R) -> Result<Vec<GameEntry>> {
        parse_unchecked_buf(buf)
    }
}
