use crate::parse::common::parsers::*;
use crate::parse::common::ARTICLES;
use crate::region::{Region, RegionError};
use nom::{multi::{many_till, many0, separated_list1},
          sequence::preceded,
          combinator::{opt, eof, peek},
          branch::alt,
          bytes::complete::{tag, is_not},
          character::complete::{char, digit1, alpha1},
          error::{Error, ErrorKind},
          IResult, Slice, Parser,
          bytes::complete::{take_while, take_while_m_n},
          character::complete::{anychar, alphanumeric1}};
use crate::parse::{trim_right_mut,
                   NameInfo, DevelopmentStatus,
                   NamingConvention, move_article,
                   replace_hyphen, ParseError};

/// A token in a NoIntro filename.
///
/// The Tokenizer API is  lossless. The original filename is reconstructible
/// from the information in the parsed tokens.
#[derive(Debug, Eq, PartialEq)]
enum NoIntroToken<'a>
{
    Title(String),
    Region(Vec<Region>),

    /// An unspecified regular flag
    Flag(FlagType, &'a str),

    /// The version
    ///
    /// Version(Tag, Major, Minor, Prefix, Suffix)
    Version(Vec<(&'a str, &'a str, Option<&'a str>)>),
    Beta(Option<&'a str>),
    Disc(&'a str),

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

macro_rules! nointro_parens_flag_parser {
    ($fn_name:ident, $tag:literal) =>
    {
        fn $fn_name<'a>(input: &'a str) -> IResult<&'a str, NoIntroToken>
        {
            let (input, tag) = in_parens(tag($tag))(input)?;
            Ok((input, NoIntroToken::Flag(FlagType::Parenthesized, tag)))
        }
    }
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
nointro_parens_flag_parser!(parse_prototype_tag, "Proto");
nointro_parens_flag_parser!(parse_kiosk_tag, "Kiosk");
nointro_parens_flag_parser!(parse_demo_tag, "Demo");
nointro_parens_flag_parser!(parse_sample_tag, "Sample");
nointro_parens_flag_parser!(parse_bonus_disc_tag, "Bonus Disc");
nointro_parens_flag_parser!(parse_bonus_cd_tag, "Bonus CD");
nointro_parens_flag_parser!(parse_disc_tag, "Disc");
nointro_parens_flag_parser!(parse_update_tag, "Update");
nointro_parens_flag_parser!(parse_dlc_tag, "DLC");
nointro_parens_flag_parser!(parse_taikenban_tag, "Taikenban"); /* 体験版 == Demo */
nointro_parens_flag_parser!(parse_tentoutaikenban_tag, "Tentou Taikenban"); /* 店頭体験版 == Kiosk */
nointro_parens_flag_parser!(parse_unlicensed_tag, "Unl");
nointro_parens_flag_parser!(parse_tool_tag, "Tool");
nointro_parens_flag_parser!(parse_psp_the_best_tag, "PSP the Best");
nointro_parens_flag_parser!(parse_psn_tag, "PSN");
nointro_parens_flag_parser!(parse_eshop_tag, "eShop");

fn parse_revision(input: &str) -> IResult<&str, (&str, &str, Option<&str>)>
{
    let (input, tag) = tag("Rev")(input)?;
    let (input, _) = char(' ')(input)?;
    let (input, major) = alphanumeric1(input)?;
    let (input, _) = opt(char('.'))(input)?;
    let (input, minor) = opt(alphanumeric1)(input)?;

    Ok((input, (tag, major, minor)))
}

fn parse_atomic_version(input: &str) -> IResult<&str, (&str, &str, Option<&str>)>
{
    let (input, major) = digit1(input)?;
    let (input, _) = char('.')(input)?;
    let (input, minor) = digit1(input)?;
    Ok((input, ("", major, Some(minor))))
}

fn parse_single_version(input: &str) -> IResult<&str, (&str, &str, Option<&str>)>
{
    let (input, ver) = alt((tag("v"), tag("Version")))(input)?;
    let (input, _) = opt(char(' '))(input)?;
    let (input, major) = digit1(input)?;
    let (input, minor) = opt(preceded(char('.'),
                                      take_while(|c: char| c.is_alphanumeric()
                                                || c == '.' || c == '-')))(input)?;
    Ok((input,(ver, major, minor)))
}

// todo: make this stricter
make_parens_tag!(parse_version_tag, parse_version_string);
fn parse_version_string(input: &str) -> IResult<&str, NoIntroToken>
{
    let (input, vers) =
        alt((separated_list1(
        alt((tag(", "), tag(","), tag(" "))),
            alt((parse_single_version, parse_revision,
                 take_while_m_n(4, 4, |c: char| c.is_ascii_digit())
                     .map(|s| ("", s, None))
            )))
        , parse_atomic_version.map(|t| vec![t]))
    )(input)?;
    Ok((input, NoIntroToken::Version(vers)))
}

make_parens_tag!(parse_beta_tag, parse_beta);
fn parse_beta(input: &str) -> IResult<&str, NoIntroToken>
{
    let (input, _) = tag("Beta")(input)?;
    let (input, beta) = opt(preceded(char(' '),
                                     take_while(|c: char| c.is_ascii_alphanumeric() || c == ' ')))(input)?;
    Ok((input, NoIntroToken::Beta(beta)))
}

make_parens_tag!(parse_disc_number_tag, parse_disc);
fn parse_disc(input: &str) -> IResult<&str, NoIntroToken>
{
    let (input, _) = tag("Disc")(input)?;
    let (input, _) = char(' ')(input)?;
    let (input, number) = digit1(input)?;
    Ok((input, NoIntroToken::Disc(number)))
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
    let (input, tag) = in_parens(is_not(")"))(input)?;
    Ok((input, NoIntroToken::Flag(FlagType::Parenthesized, tag)))
}

fn parse_known_tag(input: &str) -> IResult<&str, NoIntroToken>
{
    let (input, tag) = alt((
                                parse_language_tag,
                                parse_prototype_tag,
                                parse_kiosk_tag,
                                parse_version_tag,
                                parse_unlicensed_tag,
                                parse_beta_tag,
                                parse_disc_tag,
                                parse_disc_number_tag,
                                parse_dlc_tag,
                                parse_update_tag,
                                parse_demo_tag,
                                parse_tentoutaikenban_tag,
                                parse_taikenban_tag,
                                parse_sample_tag,
                                parse_bonus_disc_tag,
                                parse_bonus_cd_tag,
                                parse_psp_the_best_tag,
                                parse_psn_tag,
                                parse_eshop_tag,
                                parse_tool_tag
                            ))(input)?;
    Ok((input, tag))
}

fn do_parse(input: &str) -> IResult<&str, Vec<NoIntroToken>>
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
        preceded(opt(char(' ')), parse_known_tag))(input)?;

    tokens.append(&mut known_tags);

    let (input, mut other_tags) = many0(preceded(opt(char(' ')),
                                                 alt((
                                                     // Beta and revision tags have been known to
                                                     // show up after some additional tags
                                                     parse_beta_tag,
                                                     parse_revision
                                                         .map(|c| NoIntroToken::Version(vec![c])),
                                                     parse_additional_tag))))(input)?;
    tokens.append(&mut other_tags);

    // end with [b]
    let (input, bad_dump) = opt(preceded(opt(char(' ')),
                                         parse_baddump_tag))(input)?;

    // make sure we are EOF.
    let (input, _) = eof(input)?;

    if let Some(token) = bad_dump {
        tokens.push(token);
    }

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
            status: DevelopmentStatus::Release,
            naming_convention: NamingConvention::NoIntro,
        };

        let mut has_bios = false;
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
                        Some((_, major, None)) => { name.version = Some(major.to_string()) }
                        Some((_, major, Some(minor))) => { name.version = Some(format!("{}.{}", major, minor)) }
                        _ => {}
                    }
                }
                NoIntroToken::Disc(disc)
                => { name.part_number = disc.parse::<i32>().ok() }
                NoIntroToken::Region(region) => { name.region = region }
                NoIntroToken::Flag(_, "BIOS") => { has_bios = true }
                _ => {}
            }
        }

        let mut release_title = name.entry_title.clone();
        if has_bios {
            release_title.push_str(" BIOS")
        }
        move_article(&mut release_title, &ARTICLES);
        replace_hyphen(&mut release_title);
        name.release_title = release_title;
        name
    }
}

pub fn nointro_parser<'a>(input: &str) -> Result<NameInfo, ParseError> {
    let value = do_parse(input).map(|(_, value)| value).map_err(|_| {
        ParseError::BadFileNameError(NamingConvention::NoIntro, input.to_string())
    })?.into();
    Ok(value)
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
        assert_eq!(parse_beta_tag("(Beta Phase 2)"),
                   Ok(("", NoIntroToken::Beta(Some("Phase 2")))));
    }

    #[test]
    fn parse_ver_test()
    {
        assert_eq!(parse_version_tag("(v10.XX)"),
                   Ok(("", NoIntroToken::Version(vec![("v", "10", Some("XX"))]))));
        assert_eq!(parse_version_tag("(Version 10.5.6-10)"),
                   Ok(("", NoIntroToken::Version(vec![("Version", "10", Some("5.6-10"))]))));
        assert_eq!(parse_version_tag("(Version 9)"),
                   Ok(("", NoIntroToken::Version(vec![("Version", "9", None)]))));
        assert_eq!(parse_version_tag("(v1.0.0, v12342)"),
                   Ok(("", NoIntroToken::Version(vec![
                       ("v", "1", Some("0.0")),
                       ("v", "12342", None)
                   ]))));
        assert_eq!(parse_version_tag("(Rev 10)"),
                   Ok(("", NoIntroToken::Version(vec![("Rev", "10", None)]))));
        assert_eq!(parse_version_tag("(Rev 10.08)"),
                   Ok(("", NoIntroToken::Version(vec![("Rev", "10", Some("08"))]))));
        assert_eq!(parse_version_tag("(Rev 5C21)"),
                   Ok(("", NoIntroToken::Version(vec![("Rev", "5C21", None)]))));
        assert_eq!(parse_version_tag("(0.01)"),
                   Ok(("", NoIntroToken::Version(vec![("", "0", Some("01"))]))));
        assert_eq!(parse_version_tag("(v1.07 Rev 1)"),
                   Ok(("", NoIntroToken::Version(vec![
                       ("v", "1", Some("07")),
                       ("Rev", "1", None)
                   ]))));
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
        assert_eq!(parse_unlicensed_tag("(Unl)"), Ok(("", NoIntroToken::Flag(FlagType::Parenthesized, "Unl"))))
    }
}
