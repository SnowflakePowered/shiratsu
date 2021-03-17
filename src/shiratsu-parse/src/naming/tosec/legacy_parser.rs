use crate::error::*;
use crate::naming::*;
use crate::region::Region;
use lazy_static::*;
use regex::Regex;

use crate::naming::util::*;
use nom::{
    bytes::complete::is_not,
    // see the "streaming/complete" paragraph lower for an explanation of these submodules
    character::complete::char,
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

fn do_parse<'a, 'b>(title: &'a str, input: &'b str) -> IResult<&'b str, NameInfo> {
    lazy_static! {
        static ref DATE: Regex = Regex::new(
            r"^((19|20)[\dxX]{2})$|^((19|20)[\dxX]{2})-[\d]{2}$|^((19|20)[\dxX]{2})-[\d]{2}-[\d][\dxX]$"
        )
        .unwrap();
        static ref EXTRACT_PART: Regex = Regex::new(r"^\d+").unwrap();
        static ref EXTRACT_DEMO: Regex = Regex::new(r"\s?\(demo(-playable|-kiosk|-rolling|-slideshow)?\)").unwrap();
        static ref REVISION: Regex = Regex::new(r"Rev ([0-9]+)$").unwrap();
        static ref VERSION: Regex = Regex::new(r"v([\.0-9]+)$").unwrap();
    }
    let mut title = title;
    let (input, _) = many0(char(' '))(input)?;
    let (input, _publisher) = strict_parens(input)?;
    let (input, tags) = many0(strict_parens)(input)?;
    // TOSEC is wobbly
    let (input, _) = many0(char(' '))(input)?;
    let (input, flags) = many0(brackets)(input)?;
    let mut part_number: Option<i32> = None;
    let mut version: Option<String> = None;
    let mut is_unlicensed = false;
    let mut is_demo = false;
    let mut status = DevelopmentStatus::Release;
    let mut region: Option<Vec<Region>> = None;
    for tag in tags {
        if region.is_none() {
            let region_candidate = Region::try_from_tosec_region(tag);
            if let Ok(good_region) = region_candidate {
                region = Some(good_region)
            }
        }

        match tag {
            "alpha" | "beta" | "preview" | "pre-release" => status = DevelopmentStatus::Prerelease,
            "proto" => status = DevelopmentStatus::Prototype,
            _ if tag.len() > 4 => match &tag[0..4] {
                "Disc" | "Disk" | "File" | "Part" | "Tape" => {
                    part_number = EXTRACT_PART
                        .find(&tag[5..])
                        .map(|m| m.as_str().parse::<i32>().ok())
                        .unwrap_or(None)
                }
                _ => continue,
            },
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
    if let Some(caps) = VERSION.captures(title) {
        let m = caps.get(0).unwrap();
        title = &title[0..m.start()];
        version = caps.get(1).map(|cap| String::from(cap.as_str()))
    } else if let Some(caps) = REVISION.captures(title) {
        let m = caps.get(0).unwrap();
        title = &title[0..m.start()];
        version = caps.get(1).map(|cap| String::from(cap.as_str()));
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
            is_system: false,
            status,
            naming_convention: NamingConvention::TOSEC,
        },
    ))
}

pub(super) fn tosec_parser<'a>(input: &str) -> Result<NameInfo> {
    lazy_static! {
        // TOSEC lies in their convention.
        // this regex matches happy path, and then
        // also matches a sad path where everything
        // goes out the window and we just match up
        // to the first parenthesis.
        static ref FIND_TITLE_WITH_DEMO_AND_DATE: fancy_regex::Regex =
            fancy_regex::Regex::new(r"^(.+)(\s?(\(demo(-playable|-kiosk|-rolling|-slideshow)?\))?(\(((19|20)[\dxX]{2})\)|\(((19|20)[0-9xX]{2})-[0-9]{2}\)|\(((19|20)[0-9xX]{2})-[0-9]{2}-[0-9][0-9xX]\))|(?=\s\())").unwrap();
    };
    let title_captures =
        FIND_TITLE_WITH_DEMO_AND_DATE
            .captures(input)
            .ok()
            .flatten()
            .ok_or(ParseError::BadFileNameError(
                NamingConvention::TOSEC,
                String::from(input),
            ))?;
    let full_match = title_captures.get(0).ok_or(ParseError::BadFileNameError(
        NamingConvention::TOSEC,
        String::from(input),
    ))?;
    let title = title_captures
        .get(1)
        .ok_or(ParseError::BadFileNameError(
            NamingConvention::TOSEC,
            String::from(input),
        ))?
        .as_str();
    let value = do_parse(title, &input[full_match.end()..])
        .map(|(_, value)| value)
        .map_err(|_| ParseError::BadFileNameError(NamingConvention::TOSEC, String::from(input)))?;
    Ok(value)
}