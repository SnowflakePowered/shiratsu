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

struct Article(&'static str, &'static str, Regex);

impl Article {
    fn find(&self, text: &str) -> Option<usize> {
        self.2.find(text).map(|m| m.start())
    }
    const fn len_from(&self, idx: usize) -> usize {
        self.0.len() + idx
    }
}

macro_rules! article {
    ($article: expr) => {
        Article(
            concat!(", ", $article),
            concat!($article, " "),
            Regex::new(concat!(", ", $article, "($|\\s)")).unwrap(),
        )
    };
}

fn move_article(mut text: String, articles: &[Article]) -> String {
    let min_art = articles
        .iter()
        .filter_map(|art| art.find(&text).map(|idx| (art, idx)))
        .min_by_key(|(_, idx)| *idx);

    match min_art {
        None => text,
        Some((article, index)) => {
            text.replace_range(index..article.len_from(index), "");
            text.insert_str(0, article.1);
            text
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

    lazy_static! {
        static ref ARTICLES: Vec<Article> = vec![
            article!("Eine"),
            article!("The"),
            article!("Der"),
            article!("Die"),
            article!("Das"),
            article!("Ein"),
            article!("Les"),
            article!("Los"),
            article!("Las"),
            article!("An"),
            article!("De"),
            article!("La"),
            article!("Le"),
            article!("El"),
            article!("A")
        ];
    }
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

    let name = move_article(
        String::from(title.trim()),
        &[
            article!("Eine"),
            article!("The"),
            article!("Der"),
            article!("Die"),
            article!("Das"),
            article!("Ein"),
            article!("Les"),
            article!("Los"),
            article!("Las"),
            article!("An"),
            article!("De"),
            article!("La"),
            article!("Le"),
            article!("El"),
            article!("A"),
        ],
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
        },
    ))
}

fn nointro_parser<'a>(input: String) -> Result<NameInfo> {
    let value = do_parse(&input).map(|(_, value)| value)
        .map_err::<TosecNameError, _>(|err|err.into())?;
    Ok(value)
}

pub trait TosecNameable {
    fn try_from_tosec(nointro: String) -> Result<NameInfo>;
}

impl TosecNameable for NameInfo {
    fn try_from_tosec(name: String) -> Result<NameInfo> {
        nointro_parser(name)
    }
}