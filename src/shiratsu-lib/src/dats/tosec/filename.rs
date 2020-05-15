use super::super::Result;
use super::super::*;
use crate::region::{from_nointro_region, Region};
use crate::wrap_error;
use lazy_static::*;
use regex::Regex;

use nom::{
    bytes::complete::{is_not, tag},
    bytes::streaming::take_till,
    // see the "streaming/complete" paragraph lower for an explanation of these submodules
    character::complete::char,
    combinator::{complete, opt},
    multi::many0,
    sequence::{delimited, pair},
    IResult,
};

fn parens(input: &str) -> IResult<&str, &str> {
    delimited(pair(opt(char(' ')), char('(')), is_not(")"), char(')'))(input)
}

// fn brackets(input: &str) -> IResult<&str, &str> {
//     delimited(pair(opt(char(' ')), char('[')), is_not("]"), char(']'))(input)
// }

wrap_error! {
    wrap <'a> TosecNameError(nom::Err<(&'a str, nom::error::ErrorKind)>) for DatError {
        fn from (err) {
            DatError::ParseError(format!("Error parsing Redump XML: {}", err.0.to_string()))
        }
    }
}


pub fn do_parse(input: &str) -> IResult<&str, NameInfo> {
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
            "Sample" => {
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

    // let name = move_article(
    //     String::from(title.trim()),
    //     &[
    //         article!("Eine"),
    //         article!("The"),
    //         article!("Der"),
    //         article!("Die"),
    //         article!("Das"),
    //         article!("Ein"),
    //         article!("Les"),
    //         article!("Los"),
    //         article!("Las"),
    //         article!("An"),
    //         article!("De"),
    //         article!("La"),
    //         article!("Le"),
    //         article!("El"),
    //         article!("A"),
    //     ],
    // );

    Ok((
        input,
        NameInfo {
            release_name: String::from(title.trim()),
            region: region_code,
            part_number,
            version,
            is_demo,
            is_unlicensed,
            status,
        },
    ))
    
}

fn tosec_parser<'a>(input: String) -> Result<NameInfo> {
    let value = do_parse(&input).map(|(_, value)| value)
        .map_err::<TosecNameError, _>(|err|err.into())?;
    Ok(value)
}

pub trait TosecNameable {
    fn try_from_tosec(tosec: String) -> Result<NameInfo>;
}

impl TosecNameable for NameInfo {
    fn try_from_tosec(name: String) -> Result<NameInfo> {
        tosec_parser(name)
    }
}