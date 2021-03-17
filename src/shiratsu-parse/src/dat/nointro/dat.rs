use crate::naming::*;
use crate::error::*;

use serde::Deserialize;
use std::convert::{TryFrom, TryInto};

use quick_xml::de::DeError as XmlError;

use crate::naming::nointro::*;

use crate::dat::xml::*;
use crate::dat::*;

#[derive(Debug, Deserialize, PartialEq)]
struct Rom {
    name: String,
    size: i64,
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
    type Error = ParseError;
    fn try_from(game: Game) -> Result<Self> {
        let rom = game.rom;
        let name = game.name;
        Ok(GameEntry {
            info: Some(NameInfo::try_from_nointro(&name).map(|n| n.into())?),
            entry_name: name,
            serials: rom
                .iter()
                .filter_map(|r| r.serial.clone())
                .flat_map(|s| {
                    s.split(",")
                        .map(|s| Serial::new(String::from(s.trim())))
                        .collect::<Vec<_>>()
                })
                .collect(),
            rom_entries: rom.into_iter().map(|r| r.into()).collect(),
            source: "No-Intro",
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
    wrap NoIntroParserError(XmlError) for ParseError {
        fn from (err) {
            ParseError::ParseError(format!("Error parsing No-Intro XML: {}", err.0.to_string()))
        }
    }
}

fn parse(f: &str) -> Result<Vec<Result<GameEntry>>> {
    Ok(parse_dat::<Game, NoIntroParserError>(f, Some("No-Intro"))?
        .game
        .into_iter()
        .map(|g| g.try_into())
        .collect())
}

fn parse_unchecked(f: &str) -> Result<Vec<Result<GameEntry>>> {
    Ok(parse_dat_unchecked::<Game, NoIntroParserError>(f)?
        .game
        .into_iter()
        .map(|g| g.try_into())
        .collect())
}
fn parse_buf<R: std::io::BufRead>(f: R) -> Result<Vec<Result<GameEntry>>> {
    Ok(
        parse_dat_buf::<R, Game, NoIntroParserError>(f, Some("No-Intro"))?
            .game
            .into_iter()
            .map(|g| g.try_into())
            .collect(),
    )
}
fn parse_unchecked_buf<R: std::io::BufRead>(f: R) -> Result<Vec<Result<GameEntry>>> {
    Ok(parse_dat_unchecked_buf::<R, Game, NoIntroParserError>(f)?
        .game
        .into_iter()
        .map(|g| g.try_into())
        .collect())
}

/// Provides methods that parse an XML .dat files from [No-Intro](https://datomatic.no-intro.org/)
pub trait FromNoIntro {
    /// Parses the contents of a No-Intro XML DAT into a vector of `GameEntries`
    /// This function will check that the
    /// XML has the proper header for No-Intro DATs. Use
    /// `parse_nointro_unchecked` if you wish to ignore the header.
    fn try_from_nointro_str(dat: &str) -> Result<Vec<Result<GameEntry>>>;

    /// Parses the contents of a No-Intro XML DAT into a vector of `GameEntries`,
    /// ignoring the header element.
    fn try_unchecked_from_nointro_str(dat: &str) -> Result<Vec<Result<GameEntry>>>;

    /// Parses the contents of a No-Intro XML DAT into a vector of `GameEntries`
    /// This function will check that the
    /// XML has the proper header for No-Intro DATs. Use
    /// `parse_nointro_unchecked` if you wish to ignore the header.
    fn try_from_nointro_buf<R: std::io::BufRead>(buf: R) -> Result<Vec<Result<GameEntry>>>;

    /// Parses the contents of a No-Intro XML DAT into a vector of `GameEntries`,
    /// ignoring the header element
    fn try_unchecked_from_nointro_buf<R: std::io::BufRead>(
        buf: R,
    ) -> Result<Vec<Result<GameEntry>>>;
}

impl FromNoIntro for GameEntry {
    fn try_from_nointro_str(dat: &str) -> Result<Vec<Result<GameEntry>>> {
        parse(dat)
    }
    fn try_unchecked_from_nointro_str(dat: &str) -> Result<Vec<Result<GameEntry>>> {
        parse_unchecked(dat)
    }
    fn try_from_nointro_buf<R: std::io::BufRead>(buf: R) -> Result<Vec<Result<GameEntry>>> {
        parse_buf(buf)
    }
    fn try_unchecked_from_nointro_buf<R: std::io::BufRead>(
        buf: R,
    ) -> Result<Vec<Result<GameEntry>>> {
        parse_unchecked_buf(buf)
    }
}
