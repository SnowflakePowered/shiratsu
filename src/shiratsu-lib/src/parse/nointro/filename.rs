use super::super::Result;
use super::super::*;
use crate::region::Region;
use lazy_static::*;
use regex::Regex;

use nom::{
    branch::alt,
    bytes::complete::{is_not, tag, take_till, take_while_m_n},
    character::complete::char,
    combinator::{complete, opt},
    error::ErrorKind,
    multi::many0,
    sequence::{delimited, pair, tuple},
    Err as NomErr, IResult,
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

fn is_digit(c: char) -> bool {
    c.is_digit(10) || c == 'z'
}

fn take_tag(input: &str) -> IResult<&str, ()>  {
    let (input, _) = take_while_m_n(4, 4, is_digit)(input)?;
    let (input, _) = tag(" - ")(input)?;
    return Ok((input, ()))
}

fn do_parse(input: &str) -> IResult<&str, NameInfo> {
    lazy_static! {
        static ref REVISION: Regex = Regex::new(r"^Rev ([0-9]+)").unwrap();
        static ref VERSION: Regex = Regex::new(r"^(v|Version )(([0-9]?)+(\.([\w\.]?)+)?)").unwrap();
        static ref BETA: Regex = Regex::new(r"^Beta\s?([0-9]?)+").unwrap();
        static ref DISC: Regex = Regex::new(r"^Disc (([0-9]?)+)").unwrap();
    };
    let (input, has_bios) = opt(tag("[BIOS] "))(input)?;
    let (input, _) = opt(take_tag)(input)?;

    let mut region_code: Option<Vec<Region>> = None;
    let (input, title) = alt((
        // Odekake Lester - Lelele no Le (^^; is an SNES game that is
        // perfectly valid according to the naming convention, but
        // effectively impossible to parse without hacky workarounds. 
        // We're just going to hard code this case.
            tag("Odekake Lester - Lelele no Le (^^; "), 
        // Empty parenthesis hangs the parser, we're just going to hack around
        // this one special case.
            tag("void tRrLM(); Void Terrarium"), 
            take_till(|c| c == '('))
        )(input)?;
    let mut entry_title = String::from(title);
    let mut input = input;

    // Greedily take from input until a region tag is found.
    // anything not a region tag is considered part of the title.
    while region_code.is_none() && input.len() > 0 {
        let (_input, (l, region_candidate, r)) = alt((parens_with, take_until_parens))(input)?;
        if let Ok(region) = Region::try_from_nointro_region(region_candidate) {
            region_code = Some(region)
        } else {
            // push remnants
            entry_title.push_str(l);
            entry_title.push_str(region_candidate);
            entry_title.push_str(r);
        }
        input = _input;
    }
    if region_code.is_none() {
        return Err(NomErr::Error((
            "Could not find valid region string by the end of the name.",
            ErrorKind::Eof,
        )));
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
            _ => {
                if let Some(caps) = VERSION.captures(flag) {
                    version = caps.get(2).map(|ver| String::from(ver.as_str()));
                } else if let Some(caps) = REVISION.captures(flag) {
                    version = caps.get(1).map(|ver| String::from(ver.as_str()));
                }
                if BETA.is_match(flag) {
                    status = DevelopmentStatus::Prerelease
                }
                if let Some(caps) = DISC.captures(flag) {
                    part_number = caps.get(1).and_then(|i| i.as_str().parse::<i32>().ok());
                }
            }
        }
    }

    trim_right_mut(&mut entry_title);
    
    let mut release_title = entry_title.clone();
    if has_bios.is_some() {
        release_title.push_str(" BIOS")
    }
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
    let value = do_parse(input).map(|(_, value)| value).map_err(|_| {
        ParseError::BadFileNameError(NamingConvention::NoIntro, String::from(input))
    })?;
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
