use super::super::Result;
use super::super::*;
use crate::region::Region;
use lazy_static::*;
use regex::Regex;

use nom::{
    bytes::complete::{is_not, tag, take_till},
    character::complete::char,
    combinator::{complete, opt},
    branch::{alt},
    multi::many0,
    Err as NomErr,
    error::ErrorKind,
    sequence::{delimited, pair, tuple},
    IResult,
};

fn parens(input: &str) -> IResult<&str, &str> {
    delimited(pair(opt(char(' ')), char('(')), is_not(")"), char(')'))(input)
}

fn parens_with(input: &str) -> IResult<&str, (&str, &str, &str)> {
    tuple((alt((tag(" ("), tag("("))), is_not(")"), tag(")")))(input)
}

fn take_until_parens(input: &str) -> IResult<&str, (&str, &str, &str)> {
    tuple((tag(""), take_till(|c| c == '('), tag("")))(input)
}

fn do_parse(input: &str) -> IResult<&str, NameInfo> {
    lazy_static! {
        static ref REVISION: Regex = Regex::new(r"^Rev [0-9]").unwrap();
        static ref VERSION: Regex = Regex::new(r"^(v|Version )([0-9]?)+(\.([\w\.]?)+)?").unwrap();
        static ref BETA: Regex = Regex::new(r"^Beta\s?([0-9]?)+").unwrap();
        static ref DISC: Regex = Regex::new(r"^Disc (([0-9]?)+)").unwrap();
    };
    let (input, _) = opt(tag("[BIOS]"))(input)?;
    let mut region_code: Option<Vec<Region>> = None;
    let (input, title) = take_till(|c| c == '(')(input)?;
    let mut entry_title = String::from(title);
    let mut input = input;
    while region_code.is_none() && input.len() > 0 {
        let (_input, (l, region_candidate, r)) = alt((parens_with, take_until_parens))(input)?;
        if let Ok(region) = Region::try_from_nointro_region(region_candidate) {
            region_code = Some(region)
        } else {
            entry_title.push_str(l);
            entry_title.push_str(region_candidate);
            entry_title.push_str(r);
        }
        input = _input;
    }
    if region_code.is_none() {
        return Err(NomErr::Error(("Could not find valid region string by the end of the name.", ErrorKind::Eof)));
    }
    let (input, flags) = many0(parens)(input)?;
    let (input, _) = complete(opt(tag("[b]")))(input)?;

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
            _ if BETA.is_match(flag) => status = DevelopmentStatus::Prerelease,
            _ if DISC.is_match(flag) => {
                part_number = DISC
                    .captures(flag)
                    .and_then(|caps| caps.get(1).map(|i| i.as_str().parse::<i32>().ok()))
                    .unwrap_or(None);
            }
            _ => continue
        }
    }
    
    trim_right_mut(&mut entry_title);
    let mut release_title = entry_title.clone();
    move_article(&mut release_title, &ARTICLES);
    replace_hyphen(&mut release_title);
    Ok((
        input,
        NameInfo {
            entry_title,
            release_title,
            region: region_code.unwrap_or(vec![Region::Unknown]),
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
        .map_err(|_| ParseError::BadFileNameError(NamingConvention::NoIntro, String::from(input)))?;
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