use crate::region::{Region, RegionError};
use nom::{multi::{many0, separated_list1},
          sequence::preceded, combinator::opt,
          branch::alt, bytes::complete::{tag, is_not},
          error::{Error, ErrorKind}, IResult, Slice, Parser,
          bytes::complete::{take_while, take_while_m_n},
          character::complete::char};

use crate::naming::FlagType;

use crate::naming::parsers::*;
use crate::naming::tosec::tokens::*;

use nom::bytes::complete::{take_till1, take_while1};
use nom::combinator::peek;
use nom::sequence::pair;

fn parse_dumpinfo_tag<'a>(infotag: &'static str) -> impl FnMut(&'a str) -> IResult<&'a str, TOSECToken<'a>>
{
    move |input: &'a str| {
        let (input, _) = tag("[")(input)?;
        let (input, infotag) = tag(infotag)(input)?;
        let (input, index) = opt(take_while1(|c: char| c.is_ascii_digit()))(input)?;
        let (input, params) = opt(preceded(char(' '), take_till1(|c| c ==']')))(input)?;
        let (input, _) = tag("]")(input)?;
        Ok((input, TOSECToken::DumpInfo(infotag, index, params)))
    }
}

make_parens_tag!(parse_demo_tag, parse_demo, TOSECToken);
fn parse_demo(input: &str) -> IResult<&str, TOSECToken>
{
    let (input, _) = tag("demo")(input)?;
    let (input, ty) = opt(preceded(char('-'),
    alt((
        tag("kiosk"),
        tag("rolling"),
        tag("playable"),
        tag("slideshow")
        ))
    ))(input)?;
    Ok((input, TOSECToken::Demo(ty)))
}

make_parens_tag!(parse_date_tag, parse_date, Vec<TOSECToken>);
fn parse_date(input: &str) -> IResult<&str, Vec<TOSECToken>>
{
    let mut parses = Vec::new();
    let (input, year) = take_while_m_n(4, 4,
                                       |c: char| c.is_ascii_digit()
                                           || c == 'X'|| c == 'x')(input)?;
    let (input, month) = opt(
        preceded(char('-'),
                 take_while_m_n(2, 2,
                                |c: char| c.is_ascii_digit()
                                    || c == 'X'|| c == 'x')
        ))(input)?;
    let (input, day) = opt(
        preceded(char('-'),
                 take_while_m_n(2, 2,
                                |c: char| c.is_ascii_digit()
                                    || c == 'X'|| c == 'x')
        ))(input)?;

    if year.contains("X")
    {
        parses.push(TOSECToken::Warning(TOSECParseWarning::MalformedDatePlaceholder(year)));
    }

    if let Some(month) = month
    {
        if month.contains("X")
        {
            parses.push(TOSECToken::Warning(TOSECParseWarning::MalformedDatePlaceholder(month)));
        }
    }

    if let Some(day) = day
    {
        if day.contains("X")
        {
            parses.push(TOSECToken::Warning(TOSECParseWarning::MalformedDatePlaceholder(day)));
        }
    }

    parses.push(TOSECToken::Date(year, month, day));
    Ok((input, parses))
}

fn parse_region(input: &str) -> IResult<&str, Vec<Region>>
{
    let regions = Region::try_from_tosec_region(input)
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

fn parse_region_tag(input: &str) -> IResult<&str, TOSECToken>
{
    let (input, region_inner) = in_parens(is_not(")"))(input)?;
    let (_, regions) = parse_region(region_inner)?;
    Ok((input, TOSECToken::Region(regions)))
}

make_parens_tag!(parse_publisher_tag, parse_publisher, TOSECToken);
fn parse_publisher(input: &str) -> IResult<&str, TOSECToken>
{
    if let Ok((input, _)) = char::<&str, nom::error::Error<&str>>('-')(input)
    {
        return Ok((input, TOSECToken::Publisher(None)));
    }

    let (input, publishers) = separated_list1(
        tag(" - "),
        alt((
            take_until_is(")"," - "),
                is_not(")")
        ))
    )(input)?;

    Ok((input, TOSECToken::Publisher(Some(publishers))))
}

make_parens_tag!(parse_language_tag, parse_language, TOSECToken);
fn parse_language(input: &str) -> IResult<&str, TOSECToken>
{
    fn parse_multilang(input: &str) -> IResult<&str, TOSECToken>
    {
        let (input, _m) = tag("M")(input)?;
        let (input, digits) = take_while(|c: char| c.is_ascii_digit())(input)?;
        Ok((input, TOSECToken::Languages(TOSECLanguage::Count(digits))))
    }

    fn parse_single_lang(input: &str) -> IResult<&str, TOSECToken>
    {
        let (input, lang) =
            take_while_m_n(2, 2, |c: char| c.is_lowercase() && c.is_ascii_alphabetic())(input)?;
        Ok((input, TOSECToken::Languages(TOSECLanguage::Single(lang))))
    }

    fn parse_double_lang(input: &str) -> IResult<&str, TOSECToken>
    {
        let (input, lang1) =
            take_while_m_n(2, 2, |c: char| c.is_lowercase() && c.is_ascii_alphabetic())(input)?;
        let (input, _) = char('-')(input)?;
        let (input, lang2) =
            take_while_m_n(2, 2, |c: char| c.is_lowercase() && c.is_ascii_alphabetic())(input)?;
        Ok((input, TOSECToken::Languages(TOSECLanguage::Double(lang1, lang2))))
    }

    alt((parse_multilang, parse_double_lang,parse_single_lang))(input)
}

make_parens_tag!(parse_media_tag, parse_media, TOSECToken);
fn parse_media(input: &str) -> IResult<&str, TOSECToken>
{
    fn parse_single_part(input: &str) -> IResult<&str, (&str, &str, Option<&str>)> {
        let (input, ty) = alt((
            tag("Disk"),
            tag("Disc"),
            tag("File"),
            tag("Part"),
            tag("Side")
        ))(input)?;
        let (input, _) = char(' ')(input)?;
        let (input, part) = take_while(|c: char| c.is_ascii_alphanumeric() || c == '-')(input)?;
        let (input, total) = opt(preceded(tag(" of "),
                                          take_while(|c: char| c.is_ascii_alphanumeric() || c == '-')))(input)?;
        Ok((input, (ty, part, total)))
    }

    fn parse_side(input: &str) -> IResult<&str, (&str, &str, Option<&str>)>
    {
        let (input, sidetag) = tag("Side")(input)?;
        let (input, side) = preceded(char(' '),
                                     take_while(|c: char| c.is_ascii_alphanumeric()))(input)?;
        Ok((input, (sidetag, side, None)))
    }

    let mut parts = Vec::new();
    let (input, part) = parse_single_part(input)?;
    let (input, side) = opt(preceded(char(' '),
                                     parse_side))(input)?;
    parts.push(part);
    if let Some(side) = side {
        parts.push(side);
    }
    Ok((input, TOSECToken::Media(parts)))
}

fn parse_parens_tag(input: &str) -> IResult<&str, TOSECToken>
{
    let (input, _) = tag("(")(input)?;
    let (input, add_tag) = take_till1(|c: char| c == ')')(input)?;
    let (input, _) = tag(")")(input)?;
    Ok((input, TOSECToken::Flag(FlagType::Parenthesized, add_tag)))
}

fn parse_additional_tag(input: &str) -> IResult<&str, TOSECToken>
{
    let (input, _) = tag("[")(input)?;
    let (input, add_tag) = take_till1(|c: char| c == ']')(input)?;
    let (input, _) = tag("]")(input)?;
    Ok((input, TOSECToken::Flag(FlagType::Bracketed, add_tag)))
}

fn parse_version_string(input: &str) -> IResult<&str, TOSECToken>
{
    fn parse_revision(input: &str) -> IResult<&str, TOSECToken>
    {
        let (input, rev) = tag("Rev")(input)?;
        let (input, _) = char(' ')(input)?;
        let (input, version) = take_while(|c: char| c.is_ascii_alphanumeric())(input)?;
        Ok((input, TOSECToken::Version((rev, version, None, None, None))))
    }

    fn parse_version(input: &str) -> IResult<&str, TOSECToken>
    {
        let (input, v) = tag("v")(input)?;
        let (input, major) = take_while(|c: char| c.is_ascii_digit())(input)?;
        let (input, minor) = opt(preceded(char('.'),
                                          take_while(|c: char| c.is_ascii_alphanumeric())))(input)?;
        Ok((input, TOSECToken::Version((v, major, minor, None, None))))
    }

    let (input, version) = alt((parse_revision, parse_version))(input)?;
    Ok((input, version))
}

// Parse the happy path where a date is actually required..
// invariant:
// This function returns vec![Title, Date, Warning?]
fn parse_title_demo_date_happy(input: &str) -> IResult<&str, Vec<TOSECToken>>
{
    fn parse_version_and_demo_or_date(input: &str) -> IResult<&str, Vec<TOSECToken>>
    {
        let mut parses = Vec::new();
        let (input, version) = opt(preceded(char(' '), parse_version_string))(input)?;
        if let Some(version) = version {
            parses.push(version);
        }
        let (input, demo) = opt(
            preceded(opt(char(' ')), parse_demo_tag)
        )(input)?;
        if let Some(demo) = demo {
            parses.push(demo);
        }
        let (input, (warning, mut date)) = pair(
            // TOSEC Warn:
            // (demo) may occur without following space
            // 2600 Digital Clock - Demo 1 (demo)(1997-10-03)(Cracknell, Chris 'Crackers')(NTSC)(PD)
            opt(char(' '))
                .map(|c| {
                    match c {
                        Some(_) => None,
                        None => Some(TOSECToken::Warning(TOSECParseWarning::MissingSpace))
                    }
                }),

                parse_date_tag
        )(input)?;

        // Warnings come before associated token.
        if let Some(warning) = warning {
            parses.push(warning)
        }
        parses.append(&mut date);
        Ok((input, parses))
    }

    // reuse the output vec as our collector
    let (input, (title, mut tokens))
        = take_up_to(parse_version_and_demo_or_date)(input)?;

    tokens.insert(0, TOSECToken::Title(title.trim()));

    Ok((input, tokens))
}

// For most cases, the happy path is good.
// in some cases, the date is missing like in
// Motocross & Pole Position (Starsoft - JVP)(PAL)[b1][possible unknown mode]
fn parse_title_degenerate_path(input: &str) -> IResult<&str, Vec<TOSECToken>>
{
    let mut vecs = Vec::new();
    let (input, (title, version)) = take_up_to(
        alt((
            preceded(char(' '), parse_version_string).map(|t| Some(t)),
            peek(char('(')).map(|_| None)))
    )(input)?;

    // Need to discard a space just in case we go to the version part
    let (input, _) = opt(char(' '))(input)?;

    vecs.push(TOSECToken::Title(title.trim()));
    if let Some(version) = version {
        vecs.push(version)
    }

    // 'pretend' bad date was discovered after title.
    // this means the associated token for 'Missing Date'
    // is publisher.
    vecs.push(TOSECToken::Warning(TOSECParseWarning::MissingDate));
    Ok((input, vecs))
}

fn parse_zzz_unk(input: &str) -> IResult<&str, TOSECToken>
{
    let (input, _) = tag("ZZZ-UNK-")(input)?;
    Ok((input, TOSECToken::Warning(TOSECParseWarning::ZZZUnknown)))
}

fn parse_tosec_name(input: &str) -> IResult<&str, Vec<TOSECToken>>
{
    // TOSEC Warn: Filename may begin with ZZZ-UNK-
    // ZZZ-UNK files need a totally separate parser to handle v0 names...
    let (input, zzz) = opt(parse_zzz_unk)(input)?;

    let (input, mut tokens) = alt((
            parse_title_demo_date_happy,
            // TOSEC Warn: degenerate path without date
            parse_title_degenerate_path
    ))(input)?;

    if let Some(zzz) = zzz {
        // warning comes before the associated token.
        tokens.insert(0, zzz);
    }

    // TOSEC Warn: space may occur between date and publisher
    let (input, space) = opt(char(' '))(input)?;
    if let Some(_) = space {
        tokens.push(TOSECToken::Warning(TOSECParseWarning::UnexpectedSpace))
    }

    // publisher is required
    let (input, publisher) = parse_publisher_tag(input)?;
    tokens.push(publisher);

    let (input, flags)
        = many0(
        pair(opt(char(' ')),
             alt((
                parse_region_tag,
                parse_language_tag,
                parse_media_tag,
                parse_parens_tag
            ))))(input)?;

    // TOSEC Warn: Space may occur between flags
    for (space, flag) in flags {
        if let Some(_) = space {
            tokens.push(TOSECToken::Warning(TOSECParseWarning::UnexpectedSpace));
        }
        tokens.push(flag)
    }

    let input = &["cr", "f", "h", "m", "p", "t", "tr", "o", "u", "v", "b", "a", "!"]
        .iter()
        .fold(Ok(input), |input, tag|{
            // TOSEC Warn: space may occur between flags
            let (input, space) = opt(char(' '))(input?)?;
            if let Some(_) = space {
                tokens.push(TOSECToken::Warning(TOSECParseWarning::UnexpectedSpace))
            }
            let (input, info) = opt(parse_dumpinfo_tag(tag))(input)?;
            if let Some(info) =  info {
                tokens.push(info);
            }
            Ok(input)
        })?;

    let (input, rest) = many0(
        pair(
            opt(char(' ')),
        parse_additional_tag)
    )(input)?;

    for (space, flag) in rest {
        if let Some(_) = space {
            tokens.push(TOSECToken::Warning(TOSECParseWarning::UnexpectedSpace));
        }
        tokens.push(flag)
    }

    Ok((input, tokens))
}

pub(crate) fn do_parse(input: &str) -> IResult<&str, Vec<TOSECToken>>
{
    let (input, mut tokens) = parse_tosec_name(input)?;
    match input {
        "" => Ok((input, tokens)),
        rest => {
            tokens.push(TOSECToken::Warning(TOSECParseWarning::NotEof(rest)));
            Ok((input, tokens))
        }
    }
}

#[cfg(test)]
mod test
{
    use crate::region::Region;
    use crate::naming::*;
    use crate::naming::tosec::parsers::*;

    #[test]
    fn test_parse_multibyte_char()
    {
        assert_eq!(
            do_parse("Segoin Demo Ikinä! (2015-03-28)(AirZero)(FI)"),
            Ok(("",
                vec![
                    TOSECToken::Title("Segoin Demo Ikinä!"),
                    TOSECToken::Date("2015", Some("03"), Some("28")),
                    TOSECToken::Publisher(Some(vec!["AirZero"])),
                    TOSECToken::Region(vec![Region::Finland])
                ]
            ))
        )
    }

    #[test]
    fn test_parse_spaces()
    {
        assert_eq!(
            do_parse("ZZZ-UNK-Show King Tut (1996) (Schick, Bastian) [a]"),
            Ok(("",
                vec![
                    TOSECToken::Warning(TOSECParseWarning::ZZZUnknown),
                    TOSECToken::Title("Show King Tut"),
                    TOSECToken::Date("1996", None, None),
                    TOSECToken::Warning(TOSECParseWarning::UnexpectedSpace),
                    TOSECToken::Publisher(Some(vec!["Schick, Bastian"])),
                    TOSECToken::Warning(TOSECParseWarning::UnexpectedSpace),
                    TOSECToken::DumpInfo("a", None, None)
                ]
            ))
        );
    }

    #[test]
    fn test_parse_zzz()
    {
        assert_eq!(
            do_parse("ZZZ-UNK-UNK But Ok (199x)(-)"),
            Ok(("",
                vec![
                    TOSECToken::Warning(TOSECParseWarning::ZZZUnknown),
                    TOSECToken::Title("UNK But Ok"),
                    TOSECToken::Date("199x", None, None),
                    TOSECToken::Publisher(None),
                ]
            ))
        );
    }
    #[test]
    fn test_parse_full()
    {
        assert_eq!(
            do_parse("Escape from the Mindmaster (1982)(Starpath)(PAL)(Part 3 of 4)[Supercharger Cassette]"),
            Ok(("",
                vec![
                    TOSECToken::Title("Escape from the Mindmaster"),
                    TOSECToken::Date("1982", None, None),
                    TOSECToken::Publisher(Some(vec!["Starpath"])),
                    TOSECToken::Flag(FlagType::Parenthesized, "PAL"),
                    TOSECToken::Media(vec![("Part", "3", Some("4"))]),
                    TOSECToken::Flag(FlagType::Bracketed, "Supercharger Cassette"),]
            ))
        );
        assert_eq!(
            do_parse("Dune - The Battle for Arrakis Demo Hack (2009-04-03)(Ti_)[h Dune - The Battle for Arrakis]"),
            Ok(("",
                vec![
                    TOSECToken::Title("Dune - The Battle for Arrakis Demo Hack"),
                    TOSECToken::Date("2009", Some("04"), Some("03")),
                    TOSECToken::Publisher(Some(vec!["Ti_"])),
                    TOSECToken::DumpInfo("h", None, Some("Dune - The Battle for Arrakis")),]
            ))
        );

        assert_eq!(
            do_parse("2600 Digital Clock - Demo 1 (demo)(1997-10-03)(Cracknell, Chris 'Crackers')(NTSC)(PD)"),
            Ok(("",
                vec![
                    TOSECToken::Title("2600 Digital Clock - Demo 1"),
                    TOSECToken::Demo(None),
                    TOSECToken::Warning(TOSECParseWarning::MissingSpace),
                    TOSECToken::Date("1997", Some("10"), Some("03")),
                    TOSECToken::Publisher(Some(vec!["Cracknell, Chris 'Crackers'"])),
                    TOSECToken::Flag(FlagType::Parenthesized, "NTSC"),
                    TOSECToken::Flag(FlagType::Parenthesized, "PD"),
                ]
            )));
        assert_eq!(
            do_parse("Cube CD 20, The (40) - Testing v1.203 (demo) (2020)(SomePublisher)"),
            Ok(("",
                vec![
                    TOSECToken::Title("Cube CD 20, The (40) - Testing"),
                    TOSECToken::Version(("v", "1", Some("203"), None, None)),
                    TOSECToken::Demo(None),
                    TOSECToken::Date("2020", None, None),
                    TOSECToken::Publisher(Some(vec!["SomePublisher"])),
                ])));

        assert_eq!(
            do_parse("Motocross & Pole Position Rev 1 (Starsoft - JVP)(PAL)[b1][possible unknown mode]"),
            Ok(("",
                vec![
                    TOSECToken::Title("Motocross & Pole Position"),
                    TOSECToken::Version(("Rev", "1", None, None, None)),
                    TOSECToken::Warning(TOSECParseWarning::MissingDate),
                    TOSECToken::Publisher(Some(vec!["Starsoft", "JVP"])),
                    TOSECToken::Flag(FlagType::Parenthesized, "PAL"),
                    TOSECToken::DumpInfo("b", Some("1"), None),
                    TOSECToken::Flag(FlagType::Bracketed, "possible unknown mode"),
                ]))
        );
        assert_eq!(
            do_parse("Motocross & Pole Position (Starsoft - JVP)(PAL)[b1][possible unknown mode]"),
            Ok(("",
                vec![
                    TOSECToken::Title("Motocross & Pole Position"),
                    TOSECToken::Warning(TOSECParseWarning::MissingDate),
                    TOSECToken::Publisher(Some(vec!["Starsoft", "JVP"])),
                    TOSECToken::Flag(FlagType::Parenthesized, "PAL"),
                    TOSECToken::DumpInfo("b", Some("1"), None),
                    TOSECToken::Flag(FlagType::Bracketed, "possible unknown mode"),
                ]))
        );
        assert_eq!(
            do_parse("Bombsawa (Jumpman Selected levels)(19XX)(-)(JP)(ja)(PD)[cr3 +test][h][test flag]"),
            Ok(("",
                vec![
                    TOSECToken::Title("Bombsawa (Jumpman Selected levels)"),
                    TOSECToken::Warning(TOSECParseWarning::MissingSpace),
                    TOSECToken::Warning(TOSECParseWarning::MalformedDatePlaceholder("19XX")),
                    TOSECToken::Date("19XX", None, None),
                    TOSECToken::Publisher(None),
                    TOSECToken::Region(vec![Region::Japan]),
                    TOSECToken::Languages(TOSECLanguage::Single("ja")),
                    TOSECToken::Flag(FlagType::Parenthesized, "PD"),
                    TOSECToken::DumpInfo("cr", Some("3"), Some("+test")),
                    TOSECToken::DumpInfo("h", None, None),
                    TOSECToken::Flag(FlagType::Bracketed, "test flag"),
                ]))
        );
        assert_eq!(
            do_parse("Xevious (1983)(CCE)(NTSC)(BR)"),
            Ok(("",
                vec![
                    TOSECToken::Title("Xevious"),
                    TOSECToken::Date("1983", None, None),
                    TOSECToken::Publisher(Some(vec!["CCE"])),
                    TOSECToken::Flag(FlagType::Parenthesized, "NTSC"),
                    TOSECToken::Region(vec![Region::Brazil]),]
            ))
        );

        assert_eq!(
            do_parse("Mega Man III - Sample version (1992)(Capcom)(US)"),
            Ok(("",
                vec![
                    TOSECToken::Title("Mega Man III - Sample version"),
                    TOSECToken::Date("1992", None, None),
                    TOSECToken::Publisher(Some(vec!["Capcom"])),
                    TOSECToken::Region(vec![Region::UnitedStates]),]
            ))
        );


    }
    #[test]
    fn test_parse_dumpinfo()
    {
        assert_eq!(parse_dumpinfo_tag("cr")("[cr]"),
                   Ok(("", TOSECToken::DumpInfo("cr", None, None))));

        assert_eq!(parse_dumpinfo_tag("cr")("[cr2]"),
                   Ok(("", TOSECToken::DumpInfo("cr", Some("2"), None))));

        assert_eq!(parse_dumpinfo_tag("cr")("[cr2 Crack]"),
                   Ok(("", TOSECToken::DumpInfo("cr", Some("2"), Some("Crack")))));
        assert_eq!(parse_dumpinfo_tag("cr")("[cr2Crack]"),
                   Err(nom::Err::Error(Error::new("Crack]", ErrorKind::Tag))));

        assert_eq!(parse_dumpinfo_tag("cr")("[cr2 PDX - TRSi]"),
                   Ok(("", TOSECToken::DumpInfo("cr", Some("2"), Some("PDX - TRSi")))));
    }

    #[test]
    fn test_parse_parts()
    {
        assert_eq!(parse_media("Disc 2 of 2"), Ok(((""), TOSECToken::Media(
            vec![("Disc", "2", Some("2"))]
        ))));

        assert_eq!(parse_media("Side B"), Ok(((""), TOSECToken::Media(
            vec![("Side", "B", None)]
        ))));

        assert_eq!(parse_media("Disc 2 of 2 Side C"), Ok(((""), TOSECToken::Media(
            vec![("Disc", "2", Some("2")),
            ("Side", "C", None)]
        ))));

        assert_eq!(parse_media("Side 2 of 2 Side C"), Ok(((""), TOSECToken::Media(
            vec![("Side", "2", Some("2")),
                 ("Side", "C", None)]
        ))));

    }

    #[test]
    fn test_parse_version()
    {
        assert_eq!(parse_version_string("v1.0a"),
                   Ok(((""), TOSECToken::Version(("v", "1", Some("0a"), None, None)))));
        assert_eq!(parse_version_string("Rev 1b"),
                   Ok(((""), TOSECToken::Version(("Rev", "1b",None, None, None)))));
        assert_eq!(parse_version_string("v20000101"),
                   Ok(((""), TOSECToken::Version(("v", "20000101",None, None, None)))));
    }

    #[test]
    fn test_parse_lang()
    {
        assert_eq!(parse_language_tag("(M6)"), Ok(("", TOSECToken::Languages(TOSECLanguage::Count("6")))));
        assert_eq!(parse_language_tag("(M16)"), Ok(("", TOSECToken::Languages(TOSECLanguage::Count("16")))));
        assert_eq!(parse_language_tag("(en)"), Ok(("", TOSECToken::Languages(TOSECLanguage::Single("en")))));
        assert_eq!(parse_language_tag("(en-ja)"), Ok(("", TOSECToken::Languages(TOSECLanguage::Double("en", "ja")))));
    }

    #[test]
    fn test_parse_date()
    {
        assert_eq!(parse_date("1999"), Ok(("", vec![TOSECToken::Date("1999", None, None)])));
        assert_eq!(parse_date("199x"), Ok(("", vec![TOSECToken::Date("199x", None, None)])));
        assert_eq!(parse_date("199x-2x"), Ok(("", vec![TOSECToken::Date("199x", Some("2x"), None)])));
        assert_eq!(parse_date("199x-2x-10"), Ok(("", vec![TOSECToken::Date("199x", Some("2x"), Some("10"))])));
    }

    #[test]
    fn test_parse_region()
    {
        assert_eq!(parse_region("US"), Ok(("", vec![Region::UnitedStates])));
        assert_eq!(parse_region("US-ZZ"), Ok(("", vec![Region::UnitedStates, Region::Unknown])));
    }

    #[test]
    fn test_parse_region_tag()
    {
        assert_eq!(parse_region_tag("(US)"), Ok(("", TOSECToken::Region(vec![Region::UnitedStates]))));
        assert_eq!(parse_region_tag("(US-ZZ)"), Ok(("", TOSECToken::Region(vec![Region::UnitedStates, Region::Unknown]))));
    }

    #[test]
    fn test_parse_publisher()
    {
        assert_eq!(parse_publisher_tag("(-)"),
                   Ok(("",
                       TOSECToken::Publisher(None))));

        assert_eq!(parse_publisher_tag("(Devstudio)"),
                   Ok(("",
                       TOSECToken::Publisher(Some(
                           vec![
                            "Devstudio"
                           ]
                       )))));
        assert_eq!(parse_publisher_tag("(Ultrafast Software)"),
                   Ok(("",
                       TOSECToken::Publisher(Some(
                           vec![
                               "Ultrafast Software"
                           ]
                       )))));
        assert_eq!(parse_publisher_tag("(Smith, R. - White, P.S.)"),
                   Ok(("",
                       TOSECToken::Publisher(Some(
                           vec![
                               "Smith, R.",
                               "White, P.S."
                           ]
                       )))));
        assert_eq!(parse_publisher("Smith, R. - White, P.S."),
                   Ok(("",
                       TOSECToken::Publisher(Some(
                           vec![
                               "Smith, R.",
                               "White, P.S."
                           ]
                       )))));
    }
}