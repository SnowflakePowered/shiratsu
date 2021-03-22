use crate::region::*;
use nom::{IResult, Slice};
use nom::error::{ErrorKind, Error};
use crate::naming::goodtools::tokens::{GoodToolsToken, TranslationStatus};
use crate::naming::parsers::*;
use nom::bytes::complete::{is_not, take_until, take_while1, take_while_m_n, take_while};
use nom::bytes::complete::tag;
use nom::character::complete::char;
use nom::branch::alt;
use nom::Parser;
use crate::region::RegionFormat::GoodTools;
use nom::combinator::{recognize, peek, opt};
use nom::sequence::preceded;

pub(crate) fn parse_region(input: &str) -> IResult<&str, (Vec<&str>, Vec<Region>)> {
    let (strs, regions) = Region::try_from_goodtools_region_with_strs(input)
        .map_err(|e|
            {
                match e {
                    RegionError::BadRegionCode(_, _, idx)
                    => nom::Err::Error(Error::new(input.slice(idx..),
                                                  ErrorKind::Tag)),
                    _ => nom::Err::Error(Error::new(input, ErrorKind::Tag))
                }
            })?;
    Ok(("", (strs, regions)))
}


fn parse_region_tag(input: &str) -> IResult<&str, GoodToolsToken>
{
    let (input, region_inner) = in_parens(is_not(")"))(input)?;
    let (_, (strs, region)) = parse_region(region_inner)?;
    Ok((input, (GoodToolsToken::Region(strs, region))))
}

make_parens_tag!(parse_year_tag, parse_year, GoodToolsToken);
fn parse_year(input: &str) -> IResult<&str, GoodToolsToken>
{
    let (input, year) = take_year(input)?;
    Ok((input, GoodToolsToken::Year(year)))
}

fn parse_translation_tag(input: &str) -> IResult<&str, GoodToolsToken>
{
    let (input, _) = char('[')(input)?;
    let (input, _) = char('T')(input)?;
    let (input, status) = alt((
        char('+').map(|_| TranslationStatus::Recent),
        char('-').map(|_| TranslationStatus::Outdated)
    ))(input)?;
    let (input, remain) = take_until("]")(input)?;
    Ok((input, GoodToolsToken::Translation(status, remain)))
}

make_parens_tag!(parse_revision_tag, parse_revision, GoodToolsToken);
fn parse_revision(input: &str) -> IResult<&str, GoodToolsToken>
{
    let (input, rev) = tag("REV")(input)?;
    let (input, ver) = take_while1(|c: char| c.is_ascii_alphanumeric() || c == '.')(input)?;
    Ok((input, GoodToolsToken::Version(rev, ver, None)))
}

// todo: v_underscore

make_parens_tag!(parse_version_tag, parse_version, GoodToolsToken);
fn parse_version(input: &str) -> IResult<&str, GoodToolsToken>
{
    let (input, v) = tag("V")(input)?;

    // check VWIPX
    if let Ok((input, wip)) = tag::<&str, &str, nom::error::Error<&str>>("WIP")(input) {
        let (input, number) =
            opt(take_while_m_n(1, 1, |c: char| c.is_ascii_digit()))(input)?;
        return Ok((input, GoodToolsToken::Version(v, wip, number)))
    }

    if let Ok((input, fin)) = tag::<&str, &str, nom::error::Error<&str>>("Final")(input)
    {
        let (input, ver) = opt(preceded(char('_'), take_while(
            |c: char| c.is_alphanumeric() || c == '_' || c.is_ascii_whitespace()
        )))(input)?;

        return Ok((input, GoodToolsToken::Version(v, fin, ver)))
    }
    if let Ok((input, unk)) = tag::<&str, &str, nom::error::Error<&str>>("unknown")(input)
    {
        return Ok((input, GoodToolsToken::Version(v, unk, None)))
    }

    // Quick lookahead, should fail if it is not a version
    // x.xx also possible.
    let (input, _) = peek(
        take_while1(|c: char| c.is_ascii_digit() || c == 'x'))
        (input)?;

    // Take all digits (or 'x') up to .
    let (input, major) =
        take_while1(|c: char| c.is_ascii_alphanumeric()
        || c == '-' || c == '_' //
    )(input)?;

    let (input, minor) = opt(preceded(
        char('.'),
            take_while1(|c: char| c.is_ascii_alphanumeric()
                || c == '-' || c == '_' || c == '.')
    ))(input)?;

    Ok((input, GoodToolsToken::Version(v, major, minor)))
}

make_parens_tag!(parse_version_with_space_tag, parse_version_with_space, GoodToolsToken);
fn parse_version_with_space(input: &str) -> IResult<&str, GoodToolsToken>
{
    let (input, v) = tag("V ")(input)?;
    // There are 3 possibilities
    // x.xx
    // b1/b2
    // 2 or 4 digits
    let (input, version) = alt((
            tag("x.xx").map(|_| GoodToolsToken::Version(v, "x", Some("xx"))),
            take_while_m_n(2, 4, |c: char| c.is_ascii_digit())
                .map(|ver| GoodToolsToken::Version(v, ver, None)),
            recognize(|input| {
                let (input, _) = tag("b")(input)?;
                let (input, _) = take_while1(|c: char| c.is_ascii_digit())(input)?;
                Ok((input, ()))
            }).map(|ver| GoodToolsToken::Version(v, ver, None))
        ))(input)?;
    Ok((input, version))
}


make_parens_tag!(parse_version_with_underscore_tag, parse_version_with_underscore, GoodToolsToken);
fn parse_version_with_underscore(input: &str) -> IResult<&str, GoodToolsToken>
{
    let (input, v) = tag("V_")(input)?;
    let (input, major) = take_while(|c: char| c.is_ascii_alphanumeric()
    || c == '.' || c == '_' || c.is_ascii_whitespace())(input)?;
    Ok((input, GoodToolsToken::Version(v, major, None)))
}


#[cfg(test)]
mod test
{
    use crate::naming::goodtools::parsers::{parse_year_tag, parse_region_tag, parse_translation_tag, parse_version_with_space_tag, parse_version_tag};
    use crate::naming::goodtools::tokens::{GoodToolsToken, TranslationStatus};
    use crate::region::Region;
    use nom::error::ErrorKind;

    #[test]
    fn test_ver()
    {
        assert_eq!(parse_version_tag("(Vx.xx)"),
                   Ok(("", GoodToolsToken::Version("V", "x", Some("xx")))));
        assert_eq!(parse_version_tag("(Vx.x)"),
                   Ok(("", GoodToolsToken::Version("V", "x", Some("x")))));
        assert_eq!(parse_version_tag("(Vxxxx)"),
                   Ok(("", GoodToolsToken::Version("V", "xxxx", None))));
        assert_eq!(parse_version_tag("(V1.1)"),
                   Ok(("", GoodToolsToken::Version("V", "1", Some("1")))));
        assert_eq!(parse_version_tag("(V1.1.0.1)"),
                   Ok(("", GoodToolsToken::Version("V", "1", Some("1.0.1")))));
        assert_eq!(parse_version_tag("(V1.15)"),
                   Ok(("", GoodToolsToken::Version("V", "1", Some("15")))));
        assert_eq!(parse_version_tag("(V20150731)"),
                   Ok(("", GoodToolsToken::Version("V", "20150731", None))));
        assert_eq!(parse_version_tag("(V1.27_20090723)"),
                   Ok(("", GoodToolsToken::Version("V", "1", Some("27_20090723")))));
        assert_eq!(parse_version_tag("(V20050108-Rev33)"),
                   Ok(("", GoodToolsToken::Version("V", "20050108-Rev33", None))));
        assert_eq!(parse_version_tag("(VFinal)"),
                   Ok(("", GoodToolsToken::Version("V", "Final", None))));
        assert_eq!(parse_version_tag("(VFinal_20070204_for EZ4)"),
                   Ok(("", GoodToolsToken::Version("V", "Final", Some("20070204_for EZ4")))));
        assert_eq!(parse_version_tag("(V20120924PAL)"),
                   Ok(("", GoodToolsToken::Version("V", "20120924PAL", None))));
        assert_eq!(parse_version_tag("(VWIP8)"),
                   Ok(("", GoodToolsToken::Version("V", "WIP", Some("8")))));
        assert_eq!(parse_version_tag("(Vector)"),
                   Err(nom::Err::Error(nom::error::Error { input: "ector)", code: ErrorKind::TakeWhile1 })));
    }

    #[test]
    fn test_space_ver()
    {
        assert_eq!(parse_version_with_space_tag("(V b1)"),
            Ok(("", GoodToolsToken::Version("V ", "b1", None))));
        assert_eq!(parse_version_with_space_tag("(V b2)"),
                   Ok(("", GoodToolsToken::Version("V ", "b2", None))));
        assert_eq!(parse_version_with_space_tag("(V x.xx)"),
                   Ok(("", GoodToolsToken::Version("V ", "x", Some("xx")))));
        assert_eq!(parse_version_with_space_tag("(V 1502)"),
                   Ok(("", GoodToolsToken::Version("V ", "1502", None))));
        assert_eq!(parse_version_with_space_tag("(V 15)"),
                   Ok(("", GoodToolsToken::Version("V ", "15", None))));

    }

    #[test]
    fn test_translation()
    {
        assert_eq!(parse_translation_tag("[T+Eng10%]"),
                   Ok(("", GoodToolsToken::Translation(
                       TranslationStatus::Recent, "Eng10%"))));

    }

    #[test]
    fn test_region()
    {
        assert_eq!(parse_region_tag("(U)"),
                   Ok(("", GoodToolsToken::Region(vec!["U"], vec![Region::UnitedStates]))));
        assert_eq!(parse_region_tag("(W)"),
                   Ok(("", GoodToolsToken::Region(vec!["W"],
                                                  vec![Region::Japan, Region::UnitedStates, Region::Europe]))))
    }

    #[test]
    fn test_year()
    {
        assert_eq!(parse_year_tag("(1999)"), Ok(("", GoodToolsToken::Year("1999"))));
        assert_eq!(parse_year_tag("(19xx)"), Ok(("", GoodToolsToken::Year("19xx"))));
        assert_eq!(parse_year_tag("(20xx)"), Ok(("", GoodToolsToken::Year("20xx"))));
    }
}