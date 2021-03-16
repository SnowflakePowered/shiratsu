use crate::parse::common::parsers::*;
use crate::parse::common::ARTICLES;
use crate::region::{Region, RegionError};
use nom::{
        multi::{many_till, many0, separated_list1},
        sequence::preceded,
        combinator::{opt, eof, peek},
        branch::alt,
        bytes::complete::{tag, is_not},
        character::complete::{char, digit1, alpha1},
        error::{Error, ErrorKind},
        IResult, Slice, Parser,
        bytes::complete::{take_while, take_while_m_n, take},
        character::complete::{anychar, alphanumeric1}};
use crate::parse::{trim_right_mut,
                   NameInfo, DevelopmentStatus,
                   NamingConvention, move_article,
                   replace_hyphen};
use nom::multi::separated_list0;
use crate::region::RegionFormat::TOSEC;
use nom::bytes::complete::{take_till1, take_until, take_while1};
use nom::sequence::pair;


#[derive(Debug, Eq, PartialEq)]
pub enum TOSECToken<'a>
{
    Title(String),
    /// A list of parsed regions.
    Region(Vec<Region>),

    Publisher(Option<Vec<&'a str>>),
    Demo(Option<&'a str>),
    /// An unspecified regular flag
    Flag(FlagType, &'a str),
    Version((&'a str, &'a str, Option<&'a str>, Option<&'a str>, Option<Vec<&'a str>>)),

    Date(&'a str, Option<&'a str>, Option<&'a str>),
    DumpInfo(&'a str, Option<&'a str>, Option<&'a str>),

    /// Media parts
    Media(Vec<(&'a str, &'a str, Option<&'a str>)>),
    /// A vector of language tuples (Code, Variant).
    Languages(TOSECLanguage<'a>),
}

#[derive(Debug, Eq, PartialEq)]
pub enum TOSECLanguage<'a>
{
    /// A single language code
    Single(&'a str),

    /// A double language
    Double(&'a str, &'a str),
    /// A multi-language indicator without the leading 'M'
    Count(&'a str),
}

macro_rules! make_parens_tag {
    ($fn_name:ident, $inner:ident) =>
    {
        fn $fn_name<'a>(input: &'a str) -> IResult<&str, TOSECToken>
        {
            in_parens($inner)(input)
        }
    }
}

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

make_parens_tag!(parse_demo_tag, parse_demo);
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

make_parens_tag!(parse_date_tag, parse_date);
fn parse_date(input: &str) -> IResult<&str, TOSECToken>
{
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

    Ok((input, TOSECToken::Date(year, month, day)))
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

make_parens_tag!(parse_publisher_tag, parse_publisher);
fn parse_publisher(input: &str) -> IResult<&str, TOSECToken>
{
    if let Ok((input, _)) = char::<&str, nom::error::Error<&str>>('-')(input)
    {
        return Ok((input, TOSECToken::Publisher(None)));
    }

    let (input, publishers) = separated_list1(
        tag(" - "),
        alt((take_until(" - "),
             is_not(")"), // little bit of a hack but whatever.
             is_not("")))
    )(input)?;

    Ok((input, TOSECToken::Publisher(Some(publishers))))
}

make_parens_tag!(parse_language_tag, parse_language);
fn parse_language(input: &str) -> IResult<&str, TOSECToken>
{
    fn parse_multilang(input: &str) -> IResult<&str, TOSECToken>
    {
        let original_input = input;
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

fn parse_media_tag(input: &str) -> IResult<&str, TOSECToken>
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
        let (input, major) = take_while(|c: char| c.is_ascii_alphanumeric())(input)?;
        let (input, minor) = opt(preceded(char('.'),
                                          take_while(|c: char| c.is_ascii_alphanumeric())))(input)?;
        Ok((input, TOSECToken::Version((v, major, minor, None, None))))
    }

    let (input, version) = alt((parse_revision, parse_version))(input)?;
    Ok((input, version))
}

// Parse the happy path where a date is actually required..
fn parse_title_demo_date_happy(input: &str) -> IResult<&str, Vec<TOSECToken>>
{
    fn parse_demo_or_date(input: &str) -> IResult<&str, Vec<TOSECToken>>
    {
        let mut parses = Vec::new();
        let (input, demo) = opt(parse_demo_tag)(input)?;
        if let Some(demo) = demo {
            parses.push(demo);
        }
        let (input, date) = preceded(
            // TOSEC Wobbly Exception:
            // (demo) may occur without following space
            // 2600 Digital Clock - Demo 1 (demo)(1997-10-03)(Cracknell, Chris 'Crackers')(NTSC)(PD)
            opt(char(' ')),
                parse_date_tag
        )(input)?;
        parses.push(date);
        Ok((input, parses))
    }

    // reuse the output vec as our collector
    let (input, (title, mut tokens))
        = many_till(anychar, parse_demo_or_date)(input)?;

    let mut title = title.into_iter().collect();
    trim_right_mut(&mut title);

    tokens.insert(0, TOSECToken::Title(title));

    Ok((input, tokens))
}

// For most cases, the happy path is good.
// in some cases, the date is missing like in
// Motocross & Pole Position (Starsoft - JVP)(PAL)[b1][possible unknown mode]
fn parse_title_degenerate_path(input: &str) -> IResult<&str, Vec<TOSECToken>>
{
    let (input, title) = take_until("(")(input)?;
    let mut title = title.to_string();
    trim_right_mut(&mut title);

    Ok((input, vec![TOSECToken::Title(title)]))
}

fn parse_tosec_name(input: &str) -> IResult<&str, Vec<TOSECToken>>
{
    let (input, mut tokens) = alt(
        (parse_title_demo_date_happy,
            // TOSEC Wobbly Exception: degenerate path without date
         parse_title_degenerate_path))(input)?;

    // publisher is required
    let (input, publisher) = parse_publisher_tag(input)?;
    tokens.push(publisher);

    let (input, (mut flags, region))
        = many_till(parse_parens_tag, opt(parse_region_tag))(input)?;

    tokens.append(&mut flags);

    if let Some(region) = region {
        tokens.push(region);
    }

    let (input, langs) = opt(parse_language_tag)(input)?;
    if let Some(langs) = langs {
        tokens.push(langs);
    }

    let (input, res)
        = opt(many_till(parse_parens_tag, opt(parse_media_tag)))(input)?;

    if let Some((mut flags, parts)) = res {
        tokens.append(&mut flags);
        if let Some(region) = parts {
            tokens.push(region);
        }
    }

    let (input, mut flags) = many0(parse_parens_tag)(input)?;
    tokens.append(&mut flags);

    let (input, info) = opt(parse_dumpinfo_tag("cr"))(input)?;
    if let Some(info) =  info {
        tokens.push(info);
    }
    let (input, info) = opt(parse_dumpinfo_tag("f"))(input)?;
    if let Some(info) =  info {
        tokens.push(info);
    }
    let (input, info) = opt(parse_dumpinfo_tag("h"))(input)?;
    if let Some(info) =  info {
        tokens.push(info);
    }
    let (input, info) = opt(parse_dumpinfo_tag("m"))(input)?;
    if let Some(info) =  info {
        tokens.push(info);
    }
    let (input, info) = opt(parse_dumpinfo_tag("p"))(input)?;
    if let Some(info) =  info {
        tokens.push(info);
    }
    let (input, info) = opt(parse_dumpinfo_tag("t"))(input)?;
    if let Some(info) =  info {
        tokens.push(info);
    }
    let (input, info) = opt(parse_dumpinfo_tag("tr"))(input)?;
    if let Some(info) =  info {
        tokens.push(info);
    }
    let (input, info) = opt(parse_dumpinfo_tag("o"))(input)?;
    if let Some(info) =  info {
        tokens.push(info);
    }
    let (input, info) = opt(parse_dumpinfo_tag("u"))(input)?;
    if let Some(info) =  info {
        tokens.push(info);
    }
    let (input, info) = opt(parse_dumpinfo_tag("v"))(input)?;
    if let Some(info) =  info {
        tokens.push(info);
    }
    let (input, info) = opt(parse_dumpinfo_tag("b"))(input)?;
    if let Some(info) =  info {
        tokens.push(info);
    }
    let (input, info) = opt(parse_dumpinfo_tag("a"))(input)?;
    if let Some(info) =  info {
        tokens.push(info);
    }
    let (input, info) = opt(parse_dumpinfo_tag("!"))(input)?;
    if let Some(info) =  info {
        tokens.push(info);
    }

    let (input, mut rest) = many0(parse_additional_tag)(input)?;
    tokens.append(&mut rest);

    Ok((input, tokens))
}

pub(crate) fn do_parse(input: &str) -> IResult<&str, Vec<TOSECToken>>
{
    let (input, res) = parse_tosec_name(input)?;
    // make sure we are EOF.
    let (input, _) = eof(input)?;

    match input {
        "" => Ok((input, tokens)),
        _ => Err(nom::Err::Error(Error::new(input, ErrorKind::NonEmpty)))
    }
}

#[cfg(test)]
mod test
{
    use crate::region::Region;
    use crate::parse::tosec::parsers::*;

    #[test]
    fn test_parse_full()
    {
        assert_eq!(
            do_parse("Cube CD 20, The (40) - Testing v1.203 (demo) (2020)(SomePublisher)"),
            Ok(("",
                vec![
                    TOSECToken::Title(String::from("Cube CD 20, The (40) - Testing v1.203")),
                    TOSECToken::Demo(None),
                    TOSECToken::Date("2020", None, None),
                    TOSECToken::Publisher(Some(vec!["SomePublisher"]))]
                )));

        assert_eq!(
            do_parse("Motocross & Pole Position (Starsoft - JVP)(PAL)[b1][possible unknown mode]"),
            Ok(("",
                vec![
                    TOSECToken::Title(String::from("Motocross & Pole Position")),
                    TOSECToken::Publisher(Some(vec!["Starsoft", "JVP"])),
                    TOSECToken::Flag(FlagType::Parenthesized, "PAL"),
                    TOSECToken::DumpInfo("b", Some("1"), None),
                    TOSECToken::Flag(FlagType::Bracketed, "possible unknown mode")]
            ))
        );
        assert_eq!(
            do_parse("Bombsawa (Jumpman Selected levels)(19XX)(-)(JP)(ja)(PD)[cr3 +test][test flag]"),
            Ok(("",
                vec![
                    TOSECToken::Title(String::from("Bombsawa (Jumpman Selected levels)")),
                    TOSECToken::Date("19XX", None, None),
                    TOSECToken::Publisher(None),
                    TOSECToken::Region(vec![Region::Japan]),
                    TOSECToken::Languages(TOSECLanguage::Single("ja")),
                    TOSECToken::Flag(FlagType::Parenthesized, "PD"),
                    TOSECToken::DumpInfo("cr", Some("3"), Some("+test")),
                    TOSECToken::Flag(FlagType::Bracketed, "test flag")]
            ))
        );
        let (_, out) = do_parse("TOSEC Game (; (Weird title with parens)(19XX)(-)(JP)(en-ja)(PD)[cr3 +test][test flag]").unwrap();

        println!("{:?}", out);
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
        assert_eq!(parse_media_tag("Disc 2 of 2"), Ok(((""), TOSECToken::Media(
            vec![("Disc", "2", Some("2"))]
        ))));

        assert_eq!(parse_media_tag("Side B"), Ok(((""), TOSECToken::Media(
            vec![("Side", "B", None)]
        ))));

        assert_eq!(parse_media_tag("Disc 2 of 2 Side C"), Ok(((""), TOSECToken::Media(
            vec![("Disc", "2", Some("2")),
            ("Side", "C", None)]
        ))));

        assert_eq!(parse_media_tag("Side 2 of 2 Side C"), Ok(((""), TOSECToken::Media(
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
        assert_eq!(parse_date("1999"), Ok(("", TOSECToken::Date("1999", None, None))));
        assert_eq!(parse_date("199x"), Ok(("", TOSECToken::Date("199x", None, None))));
        assert_eq!(parse_date("199x-2x"), Ok(("", TOSECToken::Date("199x", Some("2x"), None))));
        assert_eq!(parse_date("199x-2x-10"), Ok(("", TOSECToken::Date("199x", Some("2x"), Some("10")))));
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