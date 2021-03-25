use crate::region::*;
use nom::{IResult, Slice};
use nom::error::{ErrorKind, Error};
use crate::naming::goodtools::tokens::{GoodToolsToken, TranslationStatus};
use crate::naming::parsers::*;
use nom::bytes::complete::{is_not, take_until, take_while1, take_while_m_n, take_while, take_till1};
use nom::bytes::complete::tag;
use nom::character::complete::char;
use nom::branch::alt;
use nom::Parser;
use nom::combinator::{recognize, peek, opt, eof};
use nom::sequence::preceded;
use crate::naming::FlagType;
use nom::multi::{separated_list1, separated_list0, many0, many1};

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
    if let Ok((input, egc)) = tag::<&str, &str, nom::error::Error<&str>>("E-GC")(input) {
        let (input, _) = char(')')(input)?;
        return Ok((input, (GoodToolsToken::Region(vec![egc], vec![Region::Europe]))))
    }
    if let Ok((input, jgc)) = tag::<&str, &str, nom::error::Error<&str>>("J-GC")(input) {
        let (input, _) = char(')')(input)?;
        return Ok((input, (GoodToolsToken::Region(vec![jgc], vec![Region::Japan]))))
    }
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
    let (input, _) = char(']')(input)?;
    Ok((input, GoodToolsToken::Translation(status, remain)))
}

make_parens_tag!(parse_revision_tag, parse_revision, GoodToolsToken);
fn parse_revision(input: &str) -> IResult<&str, GoodToolsToken>
{
    let (input, rev) = tag("REV")(input)?;
    let (input, ver) = take_while1(|c: char| c.is_ascii_alphanumeric() || c == '.')(input)?;
    Ok((input, GoodToolsToken::Version(rev, ver, None)))
}

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

make_parens_tag!(parse_multilaguage_tag, parse_multilanguage, GoodToolsToken);
fn parse_multilanguage(input: &str) -> IResult<&str, GoodToolsToken>
{
    let (input, _m) = tag("M")(input)?;
    let (input, count) = take_while_m_n(1, 2, |c: char| c.is_ascii_digit())(input)?;
    Ok((input, GoodToolsToken::Multilanguage(count)))
}

make_parens_tag!(parse_vol_tag, parse_volume, GoodToolsToken);
fn parse_volume(input: &str) -> IResult<&str, GoodToolsToken>
{
    let (input, _) = tag("Vol ")(input)?;
    let (input, vol) = take_while(|c: char| c.is_ascii_digit())(input)?;
    Ok((input, GoodToolsToken::Volume(vol)))
}

fn parse_known_parens<'a>(known: &'static str) -> impl FnMut(&'a str) -> IResult<&str, GoodToolsToken>
{
    move |input: &str| {
        let (input, info) = in_parens(tag(known))(input)?;
        Ok((input, GoodToolsToken::Flag(FlagType::Parenthesized, info)))
    }
}

fn parse_additional_parens_tag(input: &str) -> IResult<&str, GoodToolsToken>
{
    let (input, _) = tag("(")(input)?;
    let (input, add_tag) = take_till1(|c: char| c == ')')(input)?;
    let (input, _) = tag(")")(input)?;
    Ok((input, GoodToolsToken::Flag(FlagType::Parenthesized, add_tag)))
}

fn parse_additional_brackets_tag(input: &str) -> IResult<&str, GoodToolsToken>
{
    let (input, _) = tag("[")(input)?;
    let (input, add_tag) = take_till1(|c: char| c == ')')(input)?;
    let (input, _) = tag("]")(input)?;
    Ok((input, GoodToolsToken::Flag(FlagType::Bracketed, add_tag)))
}

fn parse_dump_tag<'a>(infotag: &'static str) -> impl FnMut(&'a str) -> IResult<&'a str, GoodToolsToken<'a>>
{
    move |input: &str| {
        let (input, _) = char('[')(input)?;
        let (input, itag) = tag(infotag)(input)?;
        let (input, number) = opt(take_while1(|c: char| c.is_ascii_digit()
            || c.is_ascii_lowercase()))(input)?;
        let (input, ty) = opt(take_while1(|c: char| c.is_ascii_uppercase()))(input)?;
        let (input, sep) = opt(alt((tag("+"), tag("-"))))(input)?;

        if let Some("-") = sep {
            // hack :\
            let (input, args) = take_while1(|c: char| c != ']')(input)?;
            let (input, _) = char(']')(input)?;
            return Ok((input, GoodToolsToken::DumpCode(itag, number, ty, sep, None, Some(args))))
        }
        // argnum only happens if we have '+'
        if let Some("+") = sep {
            let (input, argnum) = opt(take_while1(|c: char| c.is_ascii_digit()))(input)?;
            let (input, args) = opt(take_while1(|c: char| c.is_ascii_alphanumeric()))(input)?;
            let (input, _) = char(']')(input)?;
            return Ok((input, GoodToolsToken::DumpCode(itag, number, ty, sep, argnum, args)))
        }
        let (input, args) = opt(take_while1(|c: char| c.is_ascii_alphanumeric()))(input)?;
        let (input, _) = char(']')(input)?;
        Ok((input, GoodToolsToken::DumpCode(itag, number, ty, sep, None, args)))
    }
}

make_parens_tag!(parse_game_hack_tag, parse_game_hack, GoodToolsToken);
fn parse_game_hack(input: &str) -> IResult<&str, GoodToolsToken>
{
    let (input, game_title) = opt(take_up_to(tag(" Hack")))(input)?;

    if let Some((game_title, _)) = game_title {
        Ok((input, GoodToolsToken::GameHack(Some(game_title))))
    } else {
        let (input, _) = tag("Hack")(input)?;
        Ok((input, GoodToolsToken::GameHack(None)))
    }
}

fn parse_known_tag(input: &str) -> IResult<&str, GoodToolsToken>
{
    let (input, tokens) = alt((
        parse_region_tag,
        parse_year_tag,
        parse_vol_tag,
        parse_version_with_underscore_tag,
        parse_version_with_space_tag,
        parse_version_tag,
        parse_revision_tag,
        parse_game_hack_tag,
        parse_multilaguage_tag,
        parse_translation_tag,

        alt((
        // some known parens tag to help in heuristics
            parse_known_parens("PD"),
            parse_known_parens("NTSC"),
            parse_known_parens("PAL"),
            parse_known_parens("NTSC-PAL"),
            parse_known_parens("PAL-NTSC"),
        )),

        alt((
            parse_dump_tag("a"),
            parse_dump_tag("b"),

            // order matters
            parse_dump_tag("f_"),
            parse_dump_tag("f"),
            parse_dump_tag("o"),
            parse_dump_tag("h"),
            parse_dump_tag("p"),
            parse_dump_tag("t"),

            // order matters
            parse_dump_tag("!p"),
            parse_dump_tag("!"),
        ))
    ))(input)?;
    Ok((input, tokens))
}

fn do_parse(input: &str) -> IResult<&str, Vec<GoodToolsToken>>
{
    // two paths
    // 1. title is up to the first parens or brackets tag where
    // everything else forward is a tag
    // 2. no tags, so entire string is a title.


    fn parse_all_tags_until_end(input: &str) -> IResult<&str, Vec<GoodToolsToken>>
    {
        let (input, tokens) = many1(
            preceded(opt(tag(" ")),
                     alt((
                         parse_known_tag,
                         parse_additional_parens_tag,
                         parse_additional_brackets_tag)))
        )(input)?;
        let (input, _) = eof(input)?;
        Ok((input, tokens))
    }

    if let Ok((input, (title, mut tokens)))
        = take_up_to(parse_all_tags_until_end)(input)
    {
        tokens.insert(0, GoodToolsToken::Title(title));
        Ok((input, tokens))
    }
    else
    {
        Ok(("", vec![GoodToolsToken::Title(input)]))
    }
}


#[cfg(test)]
mod test
{
    use crate::naming::goodtools::parsers::{parse_year_tag, parse_region_tag, parse_translation_tag, parse_version_with_space_tag, parse_version_tag, parse_version_with_underscore_tag, parse_vol_tag, parse_game_hack, parse_game_hack_tag, parse_dump_tag, do_parse};
    use crate::naming::goodtools::tokens::{GoodToolsToken, TranslationStatus};
    use crate::region::Region;
    use nom::error::ErrorKind;


    #[test]
    fn test_parse()
    {
        // println!("{:?}", parse_all_tags_until_end("(2007) (PD)").unwrap());
        println!("{:?}", do_parse("(core) by wAMMA (2007) (PD)").unwrap());
        println!("{:?}", do_parse("2 Pak Special (Magenta) - Cavern Blaster, City War (1992) (HES) (PAL) [!]").unwrap());
        println!("{:?}", do_parse("Fire Emblem - Ankoku Ryuu to Hikari no Tsurugi (J) [hM04][b1]").unwrap());
    }

    #[test]
    fn test_game_hack_tag()
    {
        assert_eq!(parse_game_hack_tag("(SMB1 Hack)"),
        Ok(("", GoodToolsToken::GameHack(Some("SMB1")))))
    }

    #[test]
    fn test_vol_tag()
    {
        assert_eq!(parse_vol_tag("(Vol 6)"), Ok(("", GoodToolsToken::Volume("6"))))
    }

    #[test]
    fn test_dump_tag()
    {
        assert_eq!(parse_dump_tag("p")("[p]"),
                   Ok(("", GoodToolsToken::DumpCode("p", None, None, None, None, None)))
        );
        assert_eq!(parse_dump_tag("!")("[!]"),
                   Ok(("", GoodToolsToken::DumpCode("!", None, None, None, None, None)))
        );
        assert_eq!(parse_dump_tag("!p")("[!p]"),
                   Ok(("", GoodToolsToken::DumpCode("!p", None, None, None, None, None)))
        );
        assert_eq!(parse_dump_tag("a")("[a1]"),
                   Ok(("", GoodToolsToken::DumpCode("a", Some("1"), None, None, None, None)))
        );
        assert_eq!(parse_dump_tag("b")("[b02-Unknown Song 2]"),
                   Ok(("", GoodToolsToken::DumpCode("b", Some("02"), None, Some("-"), None, Some("Unknown Song 2"))))
        );
        assert_eq!(parse_dump_tag("h")("[h]"),
                   Ok(("", GoodToolsToken::DumpCode("h", None, None, None, None, None)))
        );
        assert_eq!(parse_dump_tag("h")("[h1]"),
            Ok(("", GoodToolsToken::DumpCode("h", Some("1"), None, None, None, None)))
        );
        assert_eq!(parse_dump_tag("h")("[h2IR]"),
                   Ok(("", GoodToolsToken::DumpCode("h", Some("2"), Some("IR"), None, None, None)))
        );
        assert_eq!(parse_dump_tag("h")("[h2IR00]"),
                   Ok(("", GoodToolsToken::DumpCode("h", Some("2"), Some("IR"), None, None, Some("00"))))
        );
        assert_eq!(parse_dump_tag("h")("[h2IRff]"),
                   Ok(("", GoodToolsToken::DumpCode("h", Some("2"), Some("IR"), None, None, Some("ff"))))
        );
        assert_eq!(parse_dump_tag("h")("[hMF6]"),
                   Ok(("", GoodToolsToken::DumpCode("h", None, Some("MF"), None, None, Some("6"))))
        );
        assert_eq!(parse_dump_tag("h")("[h1+2C]"),
                   Ok(("", GoodToolsToken::DumpCode("h", Some("1"), None, Some("+"), Some("2"), Some("C"))))
        );
    }

    #[test]
    fn test_ver_underscore()
    {
        assert_eq!(parse_version_with_underscore_tag("(V_unfinished)"),
            Ok(("", GoodToolsToken::Version("V_", "unfinished", None))));
    }

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