use super::super::*;
use crate::region::{from_nointro_region, Region};
use super::super::Result;
use lazy_static::*;
use regex::{Regex, RegexSet};
use nom::{
    bytes::complete::{is_not, tag},
    // see the "streaming/complete" paragraph lower for an explanation of these submodules
    character::complete::char,
    sequence::{pair, delimited},
    multi::many0,
    bytes::streaming::take_till,
    IResult,
    combinator::{opt, complete},
    
};

fn parens(input: &str) -> IResult<&str, &str> {
    delimited(pair(opt(char(' ')), char('(')), is_not(")"), char(')'))(input)
}

// fn brackets(input: &str) -> IResult<&str, &str> {
//     delimited(pair(opt(char(' ')), char('[')), is_not("]"), char(']'))(input)
// }

pub struct NoIntroName {
    release_name: String,
    region: Vec<Region>,
    part_number: Option<i32>,
    version: Option<String>,
    is_unlicensed: bool,
    is_demo: bool,
    status: DevelopmentStatus
}

impl <T> From<nom::Err<T>> for DatError {
    fn from(_: nom::Err<T>) -> Self {
        DatError::BadFileNameError(NamingConvention::NoIntro)
    }
}

pub fn do_parse(input: &str) -> IResult<&str, NoIntroName> {
    lazy_static! {
        static ref REGEXES: RegexSet = RegexSet::new(&[
            r"a^", // 0: NOTHING
            r"^v([0-9]?)+(\.([0-9]?)+)?", // 1: VERSION
            r"^Rev [0-9]", // 2: REVISION
            r"^Beta\s?([0-9]?)+", // 3: BETA
            r"^Disc ([0-9]?)+", // 4: DISC
        ]).unwrap();
        static ref DISC: Regex = Regex::new( r"^Disc (([0-9]?)+)").unwrap();
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
        let match_index = REGEXES.matches(flag).into_iter().count();
        match match_index {
            0 => { // NOTHING
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
                    _ => continue
                }
            },
            1 | 2 => { // VERSION | REVISION
                version = Some(String::from(flag))
            },
            3 => { // BETA
                status = DevelopmentStatus::Prelease
            },
            4 => { // DISC
                part_number = DISC.captures(flag)
                    .map(|caps| caps.get(1).map(|i| i.as_str().parse::<i32>().ok()))
                    .unwrap_or(None).unwrap_or(None)
            },
            _ => continue
        }
    }
    Ok((input, NoIntroName {
        release_name: String::from(title.trim()),
        region: region_code,
        part_number,
        version,
        is_demo,
        is_unlicensed,
        status
    }))
}

fn nointro_parser<'a>(input: String) -> Result<NoIntroName> {
    Ok(do_parse(&input).map(|(_, value)| value)?)
}

impl NoIntroName {
    pub fn new(name: String) -> Result<NoIntroName> {
        nointro_parser(name)
    }
}

impl StructuredlyNamed for NoIntroName {
    fn region(&self) -> &[Region] {
        &self.region
    }
    fn part_number(&self) -> Option<i32> {
        self.part_number
    }
    fn is_unlicensed(&self) -> bool {
        self.is_unlicensed
    }
    fn is_demo(&self) -> bool {
        self.is_demo
    }
    fn version(&self) -> Option<&str> {
        self.version.as_deref()
    }
    fn release_name(&self) -> &str {
        &self.release_name.as_str()
    }
    fn development_status(&self) -> DevelopmentStatus {
        self.status
    }
    fn naming_convention(&self) -> NamingConvention {
        NamingConvention::NoIntro
    }
}
