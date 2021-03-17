use crate::parse::common::parsers::*;
use crate::parse::common::ARTICLES;
use crate::region::{Region, RegionError};
use nom::{
        multi::{many_till, many0, separated_list1},
        sequence::preceded,
        combinator::{opt, eof, peek},
        branch::alt,
        bytes::complete::{tag, is_not},
        character::complete::{char, digit1, alpha1, anychar, alphanumeric1},
        error::{Error, ErrorKind},
        IResult, Slice, Parser,
        bytes::complete::{take_while, take_while_m_n, take_till1},
};
use crate::parse::{trim_right_mut,
                   NameInfo, DevelopmentStatus,
                   NamingConvention, move_article,
                   replace_hyphen};
use nom::multi::separated_list0;


/// A parsed language code.
#[derive(Debug, Eq, PartialEq)]
pub struct NoIntroLanguage<'a>
{
    /// The language code
    pub code: &'a str,

    /// The language variant identifier,
    /// appearing after the hyphen
    pub variant: Option<&'a str>
}

impl <'a> From<&(&'a str, Option<&'a str>)> for NoIntroLanguage<'a>
{
    fn from(tuple: &(&'a str, Option<&'a str>)) -> Self {
        NoIntroLanguage { code: tuple.0, variant: tuple.1 }
    }
}

/// A token in a NoIntro filename.
///
/// The Tokenizer API is  lossless. The original filename is reconstructible
/// from the information in the parsed tokens.
#[derive(Debug, Eq, PartialEq)]
pub enum NoIntroToken<'a>
{
    /// The title of the game.
    Title(String),

    /// A list of parsed regions.
    Region(Vec<Region>),

    /// An unspecified regular flag
    Flag(FlagType, &'a str),

    /// The parsed version.
    /// Use Version::into to convert into a more
    /// semantically useful struct.
    Version(Vec<(&'a str, &'a str, Option<&'a str>, Option<&'a str>, Option<Vec<&'a str>>)>),
    Beta(Option<&'a str>),

    /// Part number
    Part(&'a str, &'a str),

    /// A scene number with an optional type
    ///
    /// * 1234 parses to Scene("1234", None)
    /// * z123 parses to Scene("123", Some("z"))
    /// * x123 parses to Scene("123", Some("x"))
    /// * xB123 parses to Scene("123", Some("xB"))
    Scene(&'a str, Option<&'a str>),

    /// A vector of language tuples (Code, Variant).
    Languages(Vec<(&'a str, Option<&'a str>)>),
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

macro_rules! nointro_brackets_flag_parser {
    ($fn_name:ident,  $tag:literal) =>
    {
        fn $fn_name<'a>(input: &'a str) -> IResult<&'a str, NoIntroToken>
        {
            let (input, tag) = in_brackets(tag($tag))(input)?;
            Ok((input, NoIntroToken::Flag(FlagType::Bracketed, tag)))
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

// should be handled by parse_additional_tag
// nointro_parens_flag_parser!(parse_prototype_tag, "Proto");
// nointro_parens_flag_parser!(parse_kiosk_tag, "Kiosk");
// nointro_parens_flag_parser!(parse_demo_tag, "Demo");
// nointro_parens_flag_parser!(parse_sample_tag, "Sample");
// nointro_parens_flag_parser!(parse_bonus_disc_tag, "Bonus Disc");
// nointro_parens_flag_parser!(parse_bonus_cd_tag, "Bonus CD");
// nointro_parens_flag_parser!(parse_disc_tag, "Disc");
// nointro_parens_flag_parser!(parse_update_tag, "Update");
// nointro_parens_flag_parser!(parse_dlc_tag, "DLC");
// nointro_parens_flag_parser!(parse_taikenban_tag, "Taikenban"); /* 体験版 == Demo */
// nointro_parens_flag_parser!(parse_tentoutaikenban_tag, "Tentou Taikenban"); /* 店頭体験版 == Kiosk */
// nointro_parens_flag_parser!(parse_unlicensed_tag, "Unl");
// nointro_parens_flag_parser!(parse_tool_tag, "Tool");
// nointro_parens_flag_parser!(parse_psp_the_best_tag, "PSP the Best");
// nointro_parens_flag_parser!(parse_psn_tag, "PSN");
// nointro_parens_flag_parser!(parse_eshop_tag, "eShop");
// nointro_parens_flag_parser!(parse_aftermarket_tag, "Aftermarket");

// todo: tag prefixes and suffixes ('Alt') and 'PS3 v...')
// 4 digit versions can only appear AFTER a v... tag.
make_parens_tag!(parse_version_tag, parse_version_string);
fn parse_version_string(input: &str) -> IResult<&str, NoIntroToken>
{

    fn parse_revision_version(input: &str) -> IResult<&str, (&str, &str, Option<&str>,
                                                             Option<&str>, Option<Vec<&str>>)>
    {
        let (input, tag) = tag("Rev")(input)?;
        let (input, _) = char(' ')(input)?;
        let (input, major) = alphanumeric1(input)?;
        let (input, _) = opt(char('.'))(input)?;
        let (input, minor) = opt(alphanumeric1)(input)?;

        Ok((input, (tag, major, minor, None, None)))
    }

    fn parse_unprefixed_dot_version(input: &str) -> IResult<&str, (&str, &str, Option<&str>,
                                                                   Option<&str>, Option<Vec<&str>>)>
    {
        let (input, major) = digit1(input)?;
        let (input, _) = char('.')(input)?;
        let (input, minor) = digit1(input)?;
        Ok((input, ("", major, Some(minor), None, None)))
    }


    fn parse_single_prefixed_version_with_full_tag(input: &str) -> IResult<&str, (&str, &str, Option<&str>,
                                                                                  Option<&str>, Option<Vec<&str>>)>
    {
        // Redump BIOS versions include date
        fn parse_date(input: &str) -> IResult<&str, &str>
        {
            fn parse_date_check(input: &str) -> IResult<&str, (&str, &str, &str)> {
                let (input, month) = take_while_m_n(2, 2, |c: char| c.is_ascii_digit())(input)?;
                let (input, _) = char('/')(input)?;
                let (input, day) = take_while_m_n(2, 2, |c: char| c.is_ascii_digit())(input)?;
                let (input, _) = char('/')(input)?;
                let (input, year) = take_while_m_n(2, 2, |c: char| c.is_ascii_digit())(input)?;
                Ok((input, (month, day, year)))
            }
            let (input, _) = peek(parse_date_check)(input)?;
            let (input, datestr) =  take_while_m_n(8, 8, |c: char| c.is_ascii_digit() || c == '/')(input)?;
            Ok((input, datestr))
        }

        let (input, ver) = tag("Version")(input)?;
        let (input, _) = char(' ')(input)?;

        let (input, major) = digit1(input)?;
        let (input, minor) = opt(preceded(char('.'),
                                          take_while(|c: char| c.is_ascii_alphanumeric()
                                              || c == '.' || c == '-')))(input)?;

        let mut suffixes = Vec::new();

        let (input, datestr) = opt(preceded(char(' '),parse_date))(input)?;

        let (input, suffix) = opt(
            preceded(char(' '),
                     alt((
                         tag("Alt"),
                         take_while_m_n(1,1, |c: char| c.is_ascii_uppercase() && c.is_ascii_alphabetic()),
                     ))
            ))
            (input)?;

        if datestr.is_none() && suffix.is_none() {
            return Ok((input,(ver.trim(), major, minor, None, None)));
        }

        if let Some(datestr) = datestr {
            suffixes.push(datestr);
        }

        if let Some(suffix) = suffix {
            suffixes.push(suffix);
        }

        Ok((input,(ver.trim(), major, minor, None, Some(suffixes))))
    }

    fn parse_single_prefixed_version(input: &str) -> IResult<&str, (&str, &str, Option<&str>,
                                                                    Option<&str>, Option<Vec<&str>>)>
    {
        let (input, ver) = tag("v")(input)?;

        let (input, major) = digit1(input)?;
        let (input, minor) = opt(preceded(char('.'),
                                          take_while(|c: char| c.is_alphanumeric()
                                              || c == '.' || c == '-')))(input)?;
        let (input, suffix) =
            opt(preceded(char(' '), tag("Alt")))(input)?;

        Ok((input,(ver.trim(), major, minor, None, suffix.map(|x| vec![x]))))
    }

    fn parse_playstation_version(input: &str) -> IResult<&str, (&str, &str, Option<&str>,
                                                                Option<&str>, Option<Vec<&str>>)>
    {
        let (input, prefix) = alt((tag("PS3"), tag("PSP")))(input)?;
        let (input, _) = char(' ')(input)?;
        let (input, (ver, major, minor, _, _)) = parse_single_prefixed_version(input)?;
        Ok((input, (ver, major, minor, Some(prefix), None)))
    }

    let (input, vers1) =
        alt((
             parse_playstation_version,
             parse_single_prefixed_version,
             parse_single_prefixed_version_with_full_tag,
             parse_revision_version,
             parse_unprefixed_dot_version))(input)?;

    let (input, _) = opt(alt((tag(", "), tag(","), tag(" "))))(input)?;

    let (input, mut nextvers) =
        separated_list0(
            alt((tag(", "), tag(","), tag(" "))),
                alt((
                    parse_playstation_version,
                    parse_single_prefixed_version,
                    parse_single_prefixed_version_with_full_tag,
                    parse_revision_version,
                    take_while_m_n(4, 4, |c: char| c.is_ascii_alphanumeric())
                        .map(|s| ("", s, None, None, None)
                )))
        )(input)?;

    nextvers.insert(0, vers1);
    Ok((input, NoIntroToken::Version(nextvers)))
}

make_parens_tag!(parse_beta_tag, parse_beta);
fn parse_beta(input: &str) -> IResult<&str, NoIntroToken>
{
    let (input, _) = tag("Beta")(input)?;
    let (input, beta) = opt(preceded(char(' '),
                                     take_while(|c: char| c.is_ascii_alphanumeric() || c == ' ')))(input)?;
    Ok((input, NoIntroToken::Beta(beta)))
}

make_parens_tag!(parse_disc_tag, parse_disc);
fn parse_disc(input: &str) -> IResult<&str, NoIntroToken>
{
    let (input, disc) = tag("Disc")(input)?;
    let (input, _) = char(' ')(input)?;
    let (input, number) = digit1(input)?;
    Ok((input, NoIntroToken::Part(disc, number)))
}

fn parse_scene_number(input: &str) -> IResult<&str, NoIntroToken>
{
    fn parse_regular_scene_number(input: &str) -> IResult<&str, NoIntroToken>
    {
        let (input, scene) = take_while_m_n(4, 4,
                                            |c: char| c.is_ascii_digit())(input)?;
        Ok((input, NoIntroToken::Scene(scene, None)))
    }

    fn parse_z_or_x_scene_number(input: &str) -> IResult<&str, NoIntroToken>
    {
        let (input, z) = alt((tag("z"), tag("x")))(input)?;
        let (input, scene) = take_while_m_n(3, 3, |c: char| c.is_ascii_digit())(input)?;
        Ok((input, NoIntroToken::Scene(scene, Some(z))))
    }

    fn parse_bios_scene_number(input: &str) -> IResult<&str, NoIntroToken>
    {
        let (input, b) = tag("xB")(input)?;
        let (input, scene) = take_while_m_n(2, 2, |c: char| c.is_ascii_digit())(input)?;
        Ok((input, NoIntroToken::Scene(scene, Some(b))))
    }

    let (input, scene) = alt((
        parse_regular_scene_number, // ####
        parse_bios_scene_number, // xB##
        parse_z_or_x_scene_number, // z|x###
    ))(input)?;
    Ok((input, scene))
}

fn parse_scene_tag(input: &str) -> IResult<&str, NoIntroToken>
{
    let (input, scene) = parse_scene_number(input)?;
    let (input, _) = tag(" - ")(input)?;
    Ok((input, scene))
}

make_parens_tag!(parse_language_tag, parse_language);
fn parse_language(input: &str) -> IResult<&str, NoIntroToken>
{
    fn parse_language_code(input: &str) -> IResult<&str, &str>
    {
        let (input, code) = take_while_m_n(2, 2, |c: char| c.is_ascii_alphabetic())(input)?;
        Ok((input, code))
    }

    fn parse_language_variant(input: &str) -> IResult<&str, (&str, Option<&str>)>
    {
        let (input, code) = parse_language_code(input)?;
        let (input, _) = tag("-")(input)?;
        let (input, variant) = alpha1(input)?;
        Ok((input, (code, Some(variant))))
    }

    let (input, languages) = separated_list1(
        char(','),
        alt((
            parse_language_variant,
            parse_language_code
                .map(|s| (s, None))
        )),
    )(input)?;

    Ok((input, NoIntroToken::Languages(languages)))
}

fn parse_additional_tag(input: &str) -> IResult<&str, NoIntroToken>
{
    let (input, _) = tag("(")(input)?;
    let (input, add_tag) = take_till1(|c: char| c == ')')(input)?;
    let (input, _) = tag(")")(input)?;
    Ok((input, NoIntroToken::Flag(FlagType::Parenthesized, add_tag)))
}

fn parse_known_flags(input: &str) -> IResult<&str, NoIntroToken>
{
    let (input, tag) = alt((
                                parse_language_tag,
                                parse_version_tag,
                                parse_beta_tag,
                                parse_disc_tag,
                                parse_additional_tag
    ))(input)?;
    Ok((input, tag))
}

pub(crate) fn do_parse(input: &str) -> IResult<&str, Vec<NoIntroToken>>
{
    // We need this because of "FIFA 20 - Portuguese (Brazil) In-Game Commentary"
    fn parse_region_tag_and_ensure_end(input: &str) -> IResult<&str, NoIntroToken>
    {
        let (input, code) = parse_region_tag(input)?;
        let (input, _) = alt(
            (eof,
                peek(preceded(char(' '), alt((
                    parse_additional_tag,
                    parse_baddump_tag
                    )))).map(|_| "")))(input)?;
        Ok((input, code))
    }

    let mut tokens = Vec::new();

    let (input, scene) = opt(parse_scene_tag)(input)?;
    let (input, bios) = opt(parse_bios_tag)(input)?;

    if let Some(token) = scene {
        tokens.push(token);
    }

    if let Some(token) = bios {
        tokens.push(token);
    }

    // Trim left whitespace
    let (input, _) = many0(char(' '))(input)?;

    let (input, (title, region))
        = many_till(anychar, parse_region_tag_and_ensure_end)(input)?;

    let mut title = title.into_iter().collect();

    trim_right_mut(&mut title);
    tokens.push(NoIntroToken::Title(title));
    tokens.push(region);

    let (input, mut known_tags) = many0(
        preceded(opt(char(' ')), parse_known_flags))(input)?;

    tokens.append(&mut known_tags);

    // end with [b]
    let (input, bad_dump) = opt(preceded(opt(char(' ')),
                                         parse_baddump_tag))(input)?;

    if let Some(token) = bad_dump {
        tokens.push(token);
    }

    // make sure we are EOF.
    let (input, _) = eof(input)?;

    match input {
        "" => Ok((input, tokens)),
        _ => Err(nom::Err::Error(Error::new(input, ErrorKind::NonEmpty)))
    }
}

impl<'a> From<Vec<NoIntroToken<'a>>> for NameInfo
{
    fn from(tokens: Vec<NoIntroToken<'a>>) -> Self {
        let mut name = NameInfo {
            entry_title: "".to_string(),
            release_title: "".to_string(),
            region: vec![Region::Unknown],
            part_number: None,
            version: None,
            is_unlicensed: false,
            is_demo: false,
            is_system: false,
            status: DevelopmentStatus::Release,
            naming_convention: NamingConvention::NoIntro,
        };

        for token in tokens.into_iter()
        {
            match token {
                NoIntroToken::Title(title) => {
                    name.entry_title = title
                }
                NoIntroToken::Flag(_, "Kiosk")
                | NoIntroToken::Flag(_, "Demo")
                | NoIntroToken::Flag(_, "Sample")
                | NoIntroToken::Flag(_, "Bonus Disc")
                | NoIntroToken::Flag(_, "Bonus CD")
                | NoIntroToken::Flag(_, "Taikenban")
                | NoIntroToken::Flag(_, "Tentou Taikenban") => {
                    name.is_demo = true
                }
                NoIntroToken::Beta(_) => { name.status = DevelopmentStatus::Prerelease }
                NoIntroToken::Flag(_, "Proto") => { name.status = DevelopmentStatus::Prototype }
                NoIntroToken::Flag(_, "Unl") => { name.is_unlicensed = true }
                NoIntroToken::Version(versions) => {
                    match versions.first() {
                        Some((_, major, None, _, _)) => { name.version = Some(major.to_string()) }
                        Some((_, major, Some(minor), _, _)) => { name.version = Some(format!("{}.{}", major, minor)) }
                        _ => {}
                    }
                }
                NoIntroToken::Part(_, part) => { name.part_number = part.parse::<i32>().ok() }
                NoIntroToken::Region(region) => { name.region = region }
                NoIntroToken::Flag(_, "BIOS") => { name.is_system = true }
                _ => {}
            }
        }

        let mut release_title = name.entry_title.clone();

        move_article(&mut release_title, &ARTICLES);
        replace_hyphen(&mut release_title);
        name.release_title = release_title;
        name
    }
}

#[cfg(test)]
mod tests
{
    use crate::parse::nointro::parsers::*;
    use crate::region::Region;
    use nom::error::{ErrorKind, Error};

    #[test]
    fn parse_weird_beta()
    {
        //Isle of Minno (Europe) (0.01) (Beta)
    }
    #[test]
    fn parse_scene_tags()
    {
        assert_eq!(Ok(("", NoIntroToken::Scene("1234", None))), parse_scene_number("1234"));
        assert_eq!(Ok(("", NoIntroToken::Scene("234", Some("z")))), parse_scene_number("z234"));
        assert_eq!(Ok(("", NoIntroToken::Scene("234", Some("x")))), parse_scene_number("x234"));
        assert_eq!(Ok(("", NoIntroToken::Scene("34", Some("xB")))), parse_scene_number("xB34"));
    }

    #[test]
    fn parse_language_test()
    {
        let langs = parse_language_tag("(En,Fr,Es,Zh-Hant)");
        assert_eq!(Ok(("", NoIntroToken::Languages(vec![("En", None),
                                                        ("Fr", None), ("Es", None), ("Zh", Some("Hant"))]))), langs);
    }

    #[test]
    fn parse_odekake()
    {
        let (input, stuff) = do_parse("Odekake Lester - Lelele no Le (^^; (Japan) (Unl) (Rev 1)").unwrap();
        assert_eq!("", input);
        assert_eq!(Some(&NoIntroToken::Title(String::from("Odekake Lester - Lelele no Le (^^;"))), stuff.first())
    }

    #[test]
    fn parse_additional()
    {
        let stuff = parse_additional_tag("()");
        assert_eq!(stuff, Err(nom::Err::Error(Error::new(")", ErrorKind::TakeTill1))));
    }

    #[test]
    fn parse_no_region_fail()
    {
        let err = do_parse("void tRrLM(); Void Terrarium");
        assert_eq!(Err(nom::Err::Error(Error::new("", ErrorKind::Eof))), err);
    }

    #[test]
    fn parse_void()
    {
        let (input, stuff) = do_parse("void tRrLM(); Void Terrarium (Japan)").unwrap();
        assert_eq!("", input);
        assert_eq!(Some(&NoIntroToken::Title(String::from("void tRrLM(); Void Terrarium"))), stuff.first())
    }

    #[test]
    fn parse_disc_test()
    {
        assert_eq!(parse_disc_tag("(Disc 5)"),
                   Ok(("", NoIntroToken::Part("Disc", "5"))));
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
        assert_eq!(parse_beta_tag("(Beta Phase 2)"),
                   Ok(("", NoIntroToken::Beta(Some("Phase 2")))));
    }

    #[test]
    fn parse_redump_ver_test()
    {
        assert_eq!(parse_version_tag("(Version 5.0 04/15/10 E)"),
                   Ok(("", NoIntroToken::Version(vec![
                       ("Version", "5", Some("0"), None, Some(
                           vec![
                               "04/15/10",
                               "E"
                           ]
                       ))
                   ]))));
        assert_eq!(parse_version_tag("(Version 4.5 05/25/00 A)"),
                   Ok(("", NoIntroToken::Version(vec![
                       ("Version", "4", Some("5"), None, Some(
                           vec![
                               "05/25/00",
                               "A"
                           ]
                       ))
                   ]))));
    }

    #[test]
    fn parse_ver_test()
    {
        assert_eq!(parse_version_tag("(v10.XX)"),
                   Ok(("", NoIntroToken::Version(vec![("v", "10", Some("XX"), None, None)]))));
        assert_eq!(parse_version_tag("(Version 10.5.6-10)"),
                   Ok(("", NoIntroToken::Version(vec![("Version", "10", Some("5.6-10"), None, None)]))));
        assert_eq!(parse_version_tag("(Version 9)"),
                   Ok(("", NoIntroToken::Version(vec![("Version", "9", None, None, None)]))));
        assert_eq!(parse_version_tag("(v1.0.0, v12342)"),
                   Ok(("", NoIntroToken::Version(vec![
                       ("v", "1", Some("0.0"), None, None),
                       ("v", "12342", None, None, None)
                   ]))));
        assert_eq!(parse_version_tag("(Rev 10)"),
                   Ok(("", NoIntroToken::Version(vec![("Rev", "10", None, None, None)]))));
        assert_eq!(parse_version_tag("(Rev 10.08)"),
                   Ok(("", NoIntroToken::Version(vec![("Rev", "10", Some("08"), None, None)]))));
        assert_eq!(parse_version_tag("(Rev 5C21)"),
                   Ok(("", NoIntroToken::Version(vec![("Rev", "5C21", None, None, None)]))));
        assert_eq!(parse_version_tag("(0.01)"),
                   Ok(("", NoIntroToken::Version(vec![("", "0", Some("01"), None, None)]))));
        assert_eq!(parse_version_tag("(v1.07 Rev 1)"),
                   Ok(("", NoIntroToken::Version(vec![
                       ("v", "1", Some("07"), None, None),
                       ("Rev", "1", None, None, None)
                   ]))));
        assert_eq!(parse_version_tag("(v1.07 1023)"),
                   Ok(("", NoIntroToken::Version(vec![
                       ("v", "1", Some("07"), None, None),
                       ("", "1023", None, None, None)
                   ]))));
        assert_eq!(parse_version_tag("(v1.07, 1023)"),
                   Ok(("", NoIntroToken::Version(vec![
                       ("v", "1", Some("07"),None, None),
                       ("", "1023", None, None, None)
                   ]))));
        assert_eq!(parse_version_tag("(v1.07, v1023)"),
                   Ok(("", NoIntroToken::Version(vec![
                       ("v", "1", Some("07"),None, None),
                       ("v", "1023", None, None, None)
                   ]))));
        assert_eq!(parse_version_tag("(v1.07b, v1023)"),
                   Ok(("", NoIntroToken::Version(vec![
                       ("v", "1", Some("07b"),None, None),
                       ("v", "1023", None, None, None)
                   ]))));
        assert_eq!(parse_version_tag("(1984)"),
                   Err(nom::Err::Error(Error::new(")", ErrorKind::Char))));
        assert_eq!(parse_version_tag("(v1.07, v1023)"),
                   Ok(("", NoIntroToken::Version(vec![
                       ("v", "1", Some("07"),None, None),
                       ("v", "1023", None, None, None)
                   ]))));
        assert_eq!(parse_version_tag("(v1.07, v1023, PS3 v1.70, PSP v5.51, v60 Alt)"),
                   Ok(("", NoIntroToken::Version(vec![
                       ("v", "1", Some("07"),None, None),
                       ("v", "1023", None, None, None),
                       ("v", "1", Some("70"), Some("PS3"), None),
                       ("v", "5", Some("51"), Some("PSP"), None),
                       ("v", "60", None, None, Some(vec!["Alt"]))
                   ]))));

        assert_eq!(parse_version_tag("(Version 5.0 04/15/10 E)"),
                   Ok(("", NoIntroToken::Version(vec![
                       ("Version", "5", Some("0"), None, Some(
                           vec![
                               "04/15/10",
                               "E"
                           ]
                       ))
                   ]))));
        assert_eq!(parse_version_tag("(Version 4.5 05/25/00 A)"),
                   Ok(("", NoIntroToken::Version(vec![
                       ("Version", "4", Some("5"), None, Some(
                           vec![
                               "05/25/00",
                               "A"
                           ]
                       ))
                   ]))));

        //
        // (v1.01, )
        //v1.07 Rev 1

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
    fn parse_brazil()
    {
        // FIFA 20 - Portuguese (Brazil) In-Game Commentary (World) (Pt-BR) (DLC) (eShop)
        // bruh this is dumb.
        let (input, stuff) =
            do_parse("FIFA 20 - Portuguese (Brazil) In-Game Commentary (World) (Pt-BR) (DLC) (eShop)").unwrap();
        assert_eq!("", input);
        assert_eq!(Some(
            &NoIntroToken::Title(String::from("FIFA 20 - Portuguese (Brazil) In-Game Commentary"))), stuff.first())
    }
    #[test]
    fn parse_unl()
    {
        assert_eq!(parse_additional_tag("(Unl)"), Ok(("", NoIntroToken::Flag(FlagType::Parenthesized, "Unl"))))
    }
}
