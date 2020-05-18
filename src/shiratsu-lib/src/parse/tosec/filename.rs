use super::super::Result;
use super::super::*;
use crate::region::{from_tosec_region, Region};
use crate::wrap_error;
use lazy_static::*;
use regex::Regex;

use nom::{
    bytes::complete::is_not,
    // see the "streaming/complete" paragraph lower for an explanation of these submodules
    character::complete::char,
    combinator::complete,
    multi::many0,
    sequence::delimited,
    IResult,
};

fn strict_parens(input: &str) -> IResult<&str, &str> {
    delimited(char('('), is_not(")"), char(')'))(input)
}

fn brackets(input: &str) -> IResult<&str, &str> {
    delimited(char('['), is_not("]"), char(']'))(input)
}

wrap_error! {
    wrap <'a> TosecNameError(nom::Err<(&'a str, nom::error::ErrorKind)>) for ParseError {
        fn from (_) {
            ParseError::BadFileNameError(NamingConvention::TOSEC)
        }
    }
}

pub fn do_parse<'a, 'b>(title: &'a str, input: &'b str) -> IResult<&'b str, NameInfo> {
    lazy_static! {
        static ref DATE: Regex = Regex::new(r"^((19|20)[\dx]{2})$|^((19|20)[\dx]{2})-[\d]{2}$|^((19|20)[\dx]{2})-[\d]{2}-[\d][\dx]$").unwrap();
        static ref EXTRACT_PART: Regex = Regex::new(r"^\d+").unwrap();
        static ref EXTRACT_DEMO: Regex = Regex::new(r" \(demo\)").unwrap();
        static ref REVISION: Regex = Regex::new(r"Rev [0-9]$").unwrap();
        static ref VERSION: Regex = Regex::new(r"v([\.0-9]?)+$").unwrap();
    }
    let mut title = title;
    let (input, _publisher) = strict_parens(input)?;
    let (input, tags) = many0(strict_parens)(input)?;
    let (input, flags) = complete(many0(brackets))(input)?;
    let mut part_number: Option<i32> = None;
    let mut version: Option<String> = None;
    let mut is_unlicensed = false;
    let mut is_demo = false;
    let mut status = DevelopmentStatus::Release;
    let mut region: Option<Vec<Region>> = None;
    for tag in tags {
        if region.is_none() {
            let region_candidate = from_tosec_region(tag);
            if let Ok(good_region) = region_candidate{
                region = Some(good_region)
            }
        }

        match tag {
            "alpha" | "beta" | "preview" | "pre-release" => status = DevelopmentStatus::Prerelease,
            "proto" => status = DevelopmentStatus::Prototype,
            _ if tag.len() > 4 => match &tag[0..4] {
                "Disc" | "Disk" | "File" | "Part" | "Tape" => {
                    part_number = EXTRACT_PART.find(&tag[5..]).map(|m| {
                        m.as_str().parse::<i32>().ok()
                    }).unwrap_or(None)
                }
                _ => continue,
            }
            _ => continue,
        }
    }

    if flags.iter().any(|&c| c == "p") {
        is_unlicensed = true
    }
    if let Some(m) = EXTRACT_DEMO.find(title) {
        title = &title[0..m.start()];
        is_demo = true;
    }
    if let Some(m) = VERSION.find(title) {
        title = &title[0..m.start()];
        version = Some(m.as_str().to_string())
    } else if let Some(m) = REVISION.find(title) {
        title = &title[0..m.start()];
        version = Some(m.as_str().to_string())
    }

    let entry_title = String::from(title.trim());
    let mut release_title = entry_title.clone();
    move_article(&mut release_title, &ARTICLES);
    replace_hyphen(&mut release_title);

    Ok((
        input,
        NameInfo {
            entry_title,
            release_title,
            region: region.unwrap_or(vec![Region::Unknown]),
            part_number,
            version,
            is_demo,
            is_unlicensed,
            status,
            naming_convention: NamingConvention::TOSEC,
        },
    ))
    
}

fn tosec_parser<'a>(input: &str) -> Result<NameInfo> {
    lazy_static! {
        static ref FIND_TITLE_WITH_DEMO_AND_DATE: Regex = Regex::new(r"^(.+) (\(((19|20)[\dx]{2})\)|\(((19|20)[0-9x]{2})-[0-9]{2}\)|\(((19|20)[0-9x]{2})-[0-9]{2}-[0-9][0-9x]\))").unwrap();
    };
    
    let title_captures = FIND_TITLE_WITH_DEMO_AND_DATE.captures(input)
        .ok_or(ParseError::BadFileNameError(NamingConvention::TOSEC))?;
    let full_match = title_captures.get(0)
        .ok_or(ParseError::BadFileNameError(NamingConvention::TOSEC))?;
    let title = title_captures.get(1)
        .ok_or(ParseError::BadFileNameError(NamingConvention::TOSEC))?
        .as_str();
    let _ = title_captures.get(1)
        .ok_or(ParseError::BadFileNameError(NamingConvention::TOSEC))?;
    let value = do_parse(title, &input[full_match.end()..])
        .map(|(_, value)| value)
        .map_err::<TosecNameError, _>(|err|err.into())?;
        
    Ok(value)
}

pub trait TosecNameable {
    fn try_from_tosec(tosec: &str) -> Result<NameInfo>;
}

impl TosecNameable for NameInfo {
    fn try_from_tosec(name: &str) -> Result<NameInfo> {
        tosec_parser(name)
    }
}