use super::super::Result;
use super::super::*;
use crate::region::{from_nointro_region, Region};
use crate::wrap_error;
use lazy_static::*;
use regex::Regex;
use super::super::article::move_article;

use nom::{
    bytes::complete::{is_not, tag, take_till},
    character::complete::char,
    combinator::{complete, opt},
    multi::many0,
    sequence::{delimited, pair},
    IResult,
};

fn parens(input: &str) -> IResult<&str, &str> {
    delimited(pair(opt(char(' ')), char('(')), is_not(")"), char(')'))(input)
}

wrap_error! {
    wrap <'a> NoIntroNameError(nom::Err<(&'a str, nom::error::ErrorKind)>) for ParseError {
        fn from (err) {
            ParseError::ParseError(format!("Error parsing Redump XML: {}", err.0.to_string()))
        }
    }
}

fn do_parse(input: &str) -> IResult<&str, NameInfo> {
    lazy_static! {
        static ref REVISION: Regex = Regex::new(r"^Rev [0-9]").unwrap();
        static ref VERSION: Regex = Regex::new(r"^v([0-9]?)+(\.([0-9]?)+)?").unwrap();
        static ref BETA: Regex = Regex::new(r"^Beta\s?([0-9]?)+").unwrap();
        static ref DISC: Regex = Regex::new(r"^Disc (([0-9]?)+)").unwrap();
    };
    let (input, _) = opt(tag("[BIOS]"))(input)?;
    let (input, title) = take_till(|c| c == '(')(input)?;
    let (input, region) = parens(input)?;
    let (input, flags) = many0(parens)(input)?;
    let (input, _) = complete(opt(tag("[b]")))(input)?;
    let region_code = from_nointro_region(region).unwrap_or(vec![Region::Unknown]);

    let mut part_number: Option<i32> = None;
    let mut version: Option<String> = None;
    let mut is_unlicensed = false;
    let mut is_demo = false;
    let mut status = DevelopmentStatus::Release;

    for flag in flags {
        match flag {
            "Proto" => {
                status = DevelopmentStatus::Prototype;
            }
            "Kiosk" | "Demo" | "Sample" | "Bonus Disc" | "Taikenban" /* 体験版 */ | "Tentou Taikenban" /* 店頭体験版 */=> {
                is_demo = true;
            }
            "Unl" => {
                is_unlicensed = true;
            }
            _ if VERSION.is_match(flag) || REVISION.is_match(flag) => {
                version = Some(String::from(flag));
            }
            _ if BETA.is_match(flag) => status = DevelopmentStatus::Prelease,
            _ if DISC.is_match(flag) => {
                part_number = DISC
                    .captures(flag)
                    .map(|caps| caps.get(1).map(|i| i.as_str().parse::<i32>().ok()))
                    .unwrap_or(None)
                    .unwrap_or(None);
            }
            _ => continue,
        }
    }

    let name = move_article(
        String::from(title.trim()),
        &article::ARTICLES,
    );

    Ok((
        input,
        NameInfo {
            release_name: name,
            region: region_code,
            part_number,
            version,
            is_demo,
            is_unlicensed,
            status,
            naming_convention: NamingConvention::NoIntro,
        },
    ))
}

fn nointro_parser<'a>(input: &str) -> Result<NameInfo> {
    let value = do_parse(input).map(|(_, value)| value)
        .map_err::<NoIntroNameError, _>(|err|err.into())?;
    Ok(value)
}

pub trait NoIntroNameable {
    fn try_from_nointro(nointro: &str) -> Result<NameInfo>;
}

impl NoIntroNameable for NameInfo {
    fn try_from_nointro(name: &str) -> Result<NameInfo> {
        nointro_parser(name)
    }
}