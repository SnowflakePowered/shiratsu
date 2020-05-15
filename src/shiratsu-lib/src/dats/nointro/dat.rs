use crate::wrap_error;
use serde::Deserialize;
use std::convert::{TryFrom, TryInto};

use quick_xml::de::DeError as ParseError;

use super::super::nointro::NoIntroNameable;
use super::super::xml::*;
use super::super::*;
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

impl TryFrom<Game> for GameEntry {
    type Error = DatError;
    fn try_from(game: Game) -> Result<Self> {
        let rom = game.rom;
        let name = game.name;
        Ok(GameEntry {
            entry_name: name.clone(),
            info: Some(NameInfo::try_from_nointro(name).map(|n| n.into())?),
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

wrap_error! {
    wrap NoIntroParserError(ParseError) for DatError {
        fn from (err) {
            DatError::ParseError(format!("Error parsing No-Intro XML: {}", err.0.to_string()))
        }
    }
}

fn parse(f: &str) -> Result<Vec<GameEntry>> {
    parse_dat::<Game, NoIntroParserError>(f, Some("No-Intro"))?
        .game
        .into_iter()
        .map(|g| g.try_into())
        .collect()
}

fn parse_unchecked(f: &str) -> Result<Vec<GameEntry>> {
    parse_dat_unchecked::<Game, NoIntroParserError>(f)?
        .game
        .into_iter()
        .map(|g| g.try_into())
        .collect()
}
fn parse_buf<R: std::io::BufRead>(f: R) -> Result<Vec<GameEntry>> {
    parse_dat_buf::<R, Game, NoIntroParserError>(f, Some("No-Intro"))?
        .game
        .into_iter()
        .map(|g| g.try_into())
        .collect()
}
fn parse_unchecked_buf<R: std::io::BufRead>(f: R) -> Result<Vec<GameEntry>> {
    parse_dat_unchecked_buf::<R, Game, NoIntroParserError>(f)?
        .game
        .into_iter()
        .map(|g| g.try_into())
        .collect()
}

/// Provides methods that parse an XML .dat files from [No-Intro](https://datomatic.no-intro.org/)
pub trait FromNoIntro {
    /// Parses the contents of a No-Intro XML DAT into a vector of `GameEntries`
    /// This function will check that the
    /// XML has the proper header for No-Intro DATs. Use
    /// `parse_nointro_unchecked` if you wish to ignore the header.
    fn try_from_nointro(dat: &str) -> Result<Vec<GameEntry>>;

    /// Parses the contents of a No-Intro XML DAT into a vector of `GameEntries`,
    /// ignoring the header element.
    fn try_unchecked_from_nointro(dat: &str) -> Result<Vec<GameEntry>>;

    /// Parses the contents of a No-Intro XML DAT into a vector of `GameEntries`
    /// This function will check that the
    /// XML has the proper header for No-Intro DATs. Use
    /// `parse_nointro_unchecked` if you wish to ignore the header.
    fn try_from_nointro_buf<R: std::io::BufRead>(buf: R) -> Result<Vec<GameEntry>>;

    /// Parses the contents of a No-Intro XML DAT into a vector of `GameEntries`,
    /// ignoring the header element
    fn try_unchecked_from_nointro_buf<R: std::io::BufRead>(buf: R) -> Result<Vec<GameEntry>>;
}

impl FromNoIntro for GameEntry {
    fn try_from_nointro(dat: &str) -> Result<Vec<GameEntry>> {
        parse(dat)
    }
    fn try_unchecked_from_nointro(dat: &str) -> Result<Vec<GameEntry>> {
        parse_unchecked(dat)
    }
    fn try_from_nointro_buf<R: std::io::BufRead>(buf: R) -> Result<Vec<GameEntry>> {
        parse_buf(buf)
    }
    fn try_unchecked_from_nointro_buf<R: std::io::BufRead>(buf: R) -> Result<Vec<GameEntry>> {
        parse_unchecked_buf(buf)
    }
}
