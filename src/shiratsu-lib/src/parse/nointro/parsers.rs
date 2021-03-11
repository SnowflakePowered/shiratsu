use super::super::common::parsers::*;

use nom::{
    bytes::complete::{
        tag,
        is_not,
    },
    character::complete::
    {
        char,
        digit1,
    },
    error::{Error, ErrorKind},
    IResult, Slice,
};

use crate::region::{Region, RegionError};
use nom::branch::alt;
use nom::combinator::opt;
use nom::sequence::preceded;
use nom::character::streaming::alphanumeric1;
use nom::multi::{many1, many_till};
use nom::bytes::complete::take_while;
use nom::character::is_alphanumeric;
use nom::character::complete::anychar;

#[derive(Debug, Eq, PartialEq)]
enum NoIntroToken<'a>
{
    Title(String),
    Region(Vec<Region>),
    Flag(&'a str),
    Version(&'a str, &'a str, Option<&'a str>),
    Beta(Option<&'a str>),
    Disc(&'a str),
}

fn parse_region(input: &str) -> IResult<&str, Vec<Region>>
{
    let regions = Region::try_from_nointro_region(input)
        .map_err(|e|
            {
                match e {
                    RegionError::BadRegionCode(_, _, idx)
                        => nom::Err::Error(Error::new(input.slice(idx..),
                                                        ErrorKind::Tag)),
                    _ => nom::Err::Error(Error::new(input, ErrorKind::Tag))
                }
            })?;
    Ok(("", regions))
}

fn parse_region_tag(input: &str) -> IResult<&str, NoIntroToken>
{
    // Hack because we don't want nom to backtrack :|
    let (input, region_inner) = in_parens(is_not(")"))(input)?;
    let (_, regions) = parse_region(region_inner)?;
    Ok((input, NoIntroToken::Region(regions)))
}

macro_rules! nointro_parens_flag_parser {
    ($fn_name:ident, $tag:literal) =>
    {
        fn $fn_name<'a>(input: &'a str) -> IResult<&'a str, NoIntroToken>
        {
            let (input, tag) = in_parens(tag($tag))(input)?;
            Ok((input, NoIntroToken::Flag(tag)))
        }
    }
}

macro_rules! nointro_brackets_flag_parser {
    ($fn_name:ident,  $tag:literal) =>
    {
        fn $fn_name<'a>(input: &'a str) -> IResult<&'a str, NoIntroToken>
        {
            let (input, tag) = in_brackets(tag($tag))(input)?;
            Ok((input, NoIntroToken::Flag(tag)))
        }
    }
}

macro_rules! make_parens_tag {
    ($fn_name:ident, $inner:ident) =>
    {
        fn $fn_name<'a>(input: &'a str) -> IResult<&str, NoIntroToken>
        {
            in_parens($inner)(input)
        }
    }
}


nointro_brackets_flag_parser!(parse_baddump_tag, "b");
nointro_brackets_flag_parser!(parse_bios_tag, "BIOS");
nointro_parens_flag_parser!(parse_prototype_tag, "Proto");
nointro_parens_flag_parser!(parse_kiosk_tag, "Kiosk");
nointro_parens_flag_parser!(parse_demo_tag, "Demo");
nointro_parens_flag_parser!(parse_sample_tag, "Sample");
nointro_parens_flag_parser!(parse_bonus_tag, "Bonus Disc");
nointro_parens_flag_parser!(parse_taikenban_tag, "Taikenban"); /* 体験版 == Demo */
nointro_parens_flag_parser!(parse_tentoutaikenban_tag, "Tentou Taikenban"); /* 店頭体験版 == Kiosk */
nointro_parens_flag_parser!(parse_unlicensed_tag, "Unl");

make_parens_tag!(parse_revision_tag, parse_revision);
fn parse_revision(input: &str) -> IResult<&str, NoIntroToken>
{
    let (input, tag) = tag("Rev")(input)?;
    let (input, _) = char(' ')(input)?;
    let (input, version) = digit1(input)?;
    Ok((input, NoIntroToken::Version(tag, version, None)))
}

make_parens_tag!(parse_version_tag, parse_version);
fn parse_version(input: &str) -> IResult<&str, NoIntroToken>
{
    let (input, ver) = alt((tag("v"), tag("Version")))(input)?;
    let (input, _) = opt(char(' '))(input)?;
    let (input, major) = digit1(input)?;
    let (input, minor) = opt(preceded(char('.'),
                                      take_while(|c: char| c.is_alphanumeric()
                                          || c == '.')))(input)?;
    Ok((input, NoIntroToken::Version(ver, major, minor)))
}

make_parens_tag!(parse_beta_tag, parse_beta);
fn parse_beta(input: &str) -> IResult<&str, NoIntroToken>
{
    let (input, _) = tag("Beta")(input)?;
    let (input, beta) = opt(preceded(char(' '), digit1))(input)?;
    Ok((input, NoIntroToken::Beta(beta)))
}

make_parens_tag!(parse_disc_tag, parse_disc);
fn parse_disc(input: &str) -> IResult<&str, NoIntroToken>
{
    let (input, _) = tag("Disc")(input)?;
    let (input, _) = char(' ')(input)?;
    let (input, number) = digit1(input)?;
    Ok((input, NoIntroToken::Disc(number)))
}

fn do_parse(input: &str) -> IResult<&str, Vec<NoIntroToken>>
{
    let mut tokens = Vec::new();
    let (input, (title, region))
        = many_till(anychar, parse_region_tag)(input)?;
    tokens.push(NoIntroToken::Title(title.into_iter().collect()));
    tokens.push(region);
    Ok((input, tokens))
}

#[cfg(test)]
mod tests
{
    use crate::parse::nointro::parsers::*;
    use crate::region::Region;
    use nom::error::{ErrorKind, Error};

    #[test]
    fn parse_odekake()
    {
        let (input, stuff) = do_parse("Odekake Lester - Lelele no Le (^^; (Japan)").unwrap();
        assert_eq!("", input);
        assert_eq!(Some(&NoIntroToken::Title(String::from("Odekake Lester - Lelele no Le (^^; "))), stuff.first())
    }
    #[test]
    fn parse_disc_test()
    {
        assert_eq!(parse_disc_tag("(Disc 5)"),
                   Ok(("", NoIntroToken::Disc("5"))));
    }

    #[test]
    fn parse_beta_test()
    {
        assert_eq!(parse_beta_tag("(Beta)"),
                   Ok(("", NoIntroToken::Beta(None))));
        assert_eq!(parse_beta_tag("(Beta 3)"),
                   Ok(("", NoIntroToken::Beta(Some("3")))));
        assert_eq!(parse_beta_tag("(Beta 55)"),
                   Ok(("", NoIntroToken::Beta(Some("55")))));
    }

    #[test]
    fn parse_ver_test()
    {
        assert_eq!(parse_version_tag("(v10.XX)"),
                   Ok(("", NoIntroToken::Version("v", "10", Some("XX")))));
        assert_eq!(parse_version_tag("(Version 10.5.6)"),
                   Ok(("", NoIntroToken::Version("Version", "10", Some("5.6")))));
        assert_eq!(parse_version_tag("(Version 9)"),
                   Ok(("", NoIntroToken::Version("Version", "9", None))));

    }

    #[test]
    fn parse_rev_test()
    {
        assert_eq!(parse_revision_tag("(Rev 10)"),
                   Ok(("", NoIntroToken::Version("Rev", "10", None))));
    }
    #[test]
    fn parse_region_test()
    {
        assert_eq!(parse_region("Japan, Europe, Australia, New Zealand"),
            Ok(("", vec![Region::Japan, Region::Europe, Region::Australia, Region::NewZealand])));
    }

    #[test]
    fn parse_region_tag_test()
    {
        assert_eq!(parse_region_tag("(Japan, Europe, Australia, New Zealand)"),
                   Ok(("", NoIntroToken::Region(vec![Region::Japan, Region::Europe, Region::Australia, Region::NewZealand]))));
    }

    #[test]
    fn parse_region_test_fail()
    {
        assert_eq!(parse_region("Japan, Europe, Apustralia, New Zealand"),
            Err(nom::Err::Error(Error::new("Apustralia, New Zealand", ErrorKind::Tag))))
    }

    #[test]
    fn parse_unl()
    {
        assert_eq!(parse_unlicensed_tag("(Unl)"), Ok(("", NoIntroToken::Flag("Unl"))))
    }
}
