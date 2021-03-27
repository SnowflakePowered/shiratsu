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

use nom::bytes::complete::{take_till1, take_while1, take, take_until};
use nom::combinator::{peek, verify, map};
use nom::sequence::pair;
use nom::error::ParseError;

use crate::naming::goodtools::parse_goodtools_region;

fn parse_goodtools_region_tag(input: &str) -> IResult<&str, Vec<TOSECToken>>
{
    let (input, region_inner) = in_parens(is_not(")"))(input)?;
    let (_, (strs, regions)) = parse_goodtools_region(region_inner)?;
    Ok((input, vec![TOSECToken::Warning(TOSECWarn::GoodToolsRegionCode(region_inner)),
                    TOSECToken::Region(strs, regions)]))
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

make_parens_tag!(parse_devstatus_tag, parse_devstatus, Vec<TOSECToken>);
fn parse_devstatus(input: &str) -> IResult<&str, Vec<TOSECToken>>
{
    let known = alt((
            tag("alpha"),
            tag("beta"),
            tag("preview"),
            tag("pre-release"),
            tag("proto")
        )).map(|c| Ok(vec![TOSECToken::Development(c)]));

    let malformed = alt((
        tag("Alpha"),
        tag("Beta"),
        tag("Preview"),
        tag("Pre-Release"),
        tag("Proto"),
        tag("Prototype")
        )).map(|c| Ok(vec![TOSECToken::Warning(TOSECWarn::MalformedDevelopmentStatus(c)),
                           TOSECToken::Development(c)]));

    let (input, known) = alt((known, malformed))(input)?;
    Ok((input, known?))
}


make_parens_tag!(parse_date_tag, parse_date, Vec<TOSECToken>);
fn parse_date(input: &str) -> IResult<&str, Vec<TOSECToken>>
{
    fn parse_undelimited_date(input: &str)-> IResult<&str, Vec<TOSECToken>>
    {
        let orig_input = input;

        let (input, year) = take_year(input)?;


        let (input, month) = verify(take_while_m_n(2, 2,
                                           |c: char| c.is_ascii_digit()),
        |month: &str| month.parse::<u32>().map(|m| m <= 12).unwrap_or(false))
            (input)?;

        let (input, day) = verify(take_while_m_n(2, 2,
                                                |c: char| c.is_ascii_digit()),
                                 |month: &str| month.parse::<u32>().map(|m| m <= 31 )
                                     .unwrap_or(false))
            (input)?;
        let (_, datestr) = take(8usize)(orig_input)?;

        Ok((input, vec![
            TOSECToken::Warning(TOSECWarn::UndelimitedDate(datestr)),
            TOSECToken::Date(year, Some(month), Some(day))
        ]))
    }

    if let Ok(date) = parse_undelimited_date(input) {
        return Ok(date)
    }

    let mut parses = Vec::new();
    let (input, year) = take_year(input)?;
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
        parses.push(TOSECToken::Warning(TOSECWarn::MalformedDatePlaceholder(year)));
    }

    if let Some(month) = month
    {
        if month.contains("X")
        {
            parses.push(TOSECToken::Warning(TOSECWarn::MalformedDatePlaceholder(month)));
        }
    }

    if let Some(day) = day
    {
        if day.contains("X")
        {
            parses.push(TOSECToken::Warning(TOSECWarn::MalformedDatePlaceholder(day)));
        }
    }

    parses.push(TOSECToken::Date(year, month, day));
    Ok((input, parses))
}

fn parse_region(input: &str) -> IResult<&str, (Vec<&str>, Vec<Region>)>
{
    let regions = Region::try_from_tosec_region_with_strs(input)
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
    let (_, (strs, regions)) = parse_region(region_inner)?;
    Ok((input, TOSECToken::Region(strs, regions)))
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


make_parens_tag!(parse_system_tag, parse_system, TOSECToken);
fn parse_system(input: &str) -> IResult<&str, TOSECToken>
{
    let (input, system) = alt((
        alt((
            tag("+2"),
            tag("+2a"),
            tag("+3"),
            tag("130XE"),
            tag("A1000"),
            tag("A1200"),
            tag("A1200-A4000"),
            tag("A2000"),
            tag("A2000-A3000"),
            tag("A2024"),
            tag("A2500-A3000UX"),
            tag("A3000"),
            tag("A4000"),
            tag("A4000T"),
            tag("A500"),
            tag("A500+"),
            tag("A500-A1000-A2000"),
            tag("A500-A1000-A2000-CDTV"),
            tag("A500-A1200"),
            tag("A500-A1200-A2000-A4000"))),
        alt((
            tag("A500-A2000"),
            tag("A500-A600-A2000"),
            tag("A570"),
            tag("A600"),
            tag("A600HD"),
            tag("AGA"),
            tag("AGA-CD32"),
            tag("Aladdin Deck Enhancer"))),
        alt((
            tag("CD32"),
            tag("CDTV"),
            tag("Computrainer"),
            tag("Doctor PC Jr."),
            tag("ECS"),
            tag("ECS-AGA"),
            tag("Executive"),
            tag("Mega ST"),
            tag("Mega-STE"),
            tag("OCS"),
            tag("OCS-AGA"),
            tag("ORCH80"),
            tag("Osbourne 1"))),
        alt((
            tag("PIANO90"),
            tag("PlayChoice-10"),
            tag("Plus4"),
            tag("Primo-A"),
            tag("Primo-A64"),
            tag("Primo-B"),
            tag("Primo-B64"),
            tag("Pro-Primo"),
            tag("ST"),
            tag("STE"),
            tag("STE-Falcon"),
            tag("TT"),
            tag("TURBO-R GT"),
            tag("TURBO-R ST"),
            tag("VS DualSystem"),
            tag("VS UniSystem"))),
    ))(input)?;
    Ok((input, TOSECToken::System(system)))
}

make_parens_tag!(parse_video_tag, parse_video, TOSECToken);
fn parse_video(input: &str) -> IResult<&str, TOSECToken>
{
    let (input, video) = alt((
            tag("CGA"),
            tag("EGA"),
            tag("HGC"),
            tag("MCGA"),
            tag("MDA"),
            tag("NTSC"),
            tag("NTSC-PAL"),
            tag("PAL"),
            tag("PAL-60"),
            tag("PAL-NTSC"),
            tag("SVGA"),
            tag("VGA"),
            tag("XGA")
        ))(input)?;
    Ok((input, TOSECToken::Video(video)))
}

make_parens_tag!(parse_copyright_tag, parse_copyright, TOSECToken);
fn parse_copyright(input: &str) -> IResult<&str, TOSECToken>
{
    let (input, copyright) = alt((
        tag("CW"),
        tag("CW-R"),
        tag("FW"),
        tag("GW"),
        tag("GW-R"),
        tag("LW"),
        tag("PD"),
        tag("SW"),
        tag("SW-R")))(input)?;
    Ok((input, TOSECToken::Copyright(copyright)))
}

fn parse_parens_tag(input: &str) -> IResult<&str, TOSECToken>
{
    let (input, _) = tag("(")(input)?;
    let (input, add_tag) = take_till1(|c: char| c == ')')(input)?;
    let (input, _) = tag(")")(input)?;
    Ok((input, TOSECToken::Flag(FlagType::Parenthesized, add_tag)))
}

fn parse_moreinfo_tag(input: &str) -> IResult<&str, TOSECToken>
{
    let (input, _) = tag("[")(input)?;
    let (input, add_tag) = take_till1(|c: char| c == ']')(input)?;
    let (input, _) = tag("]")(input)?;
    Ok((input, TOSECToken::Flag(FlagType::Bracketed, add_tag)))
}

fn parse_version_tag(input: &str) -> IResult<&str, Vec<TOSECToken>>
{
    let (input, version) = in_parens(parse_version_string)(input)?;
    Ok((input, vec![
        TOSECToken::Warning(TOSECWarn::VersionInFlag),
        version
    ]))
}

fn parse_version_string(input: &str) -> IResult<&str, TOSECToken>
{
    fn parse_revision(input: &str) -> IResult<&str, TOSECToken>
    {
        let (input, rev) = tag("Rev")(input)?;
        let (input, _) = char(' ')(input)?;
        let (input, version) = take_while(|c: char| c.is_ascii_alphanumeric())(input)?;
        Ok((input, TOSECToken::Version(rev, version, None)))
    }

    fn parse_version(input: &str) -> IResult<&str, TOSECToken>
    {
        let (input, v) = tag("v")(input)?;
        let (input, major) = take_while(|c: char| c.is_ascii_digit())(input)?;
        let (input, minor) = opt(preceded(char('.'),
                                          take_while(|c: char| c.is_ascii_alphanumeric() || c == '.')))(input)?;

        // Major version can be empty string only if minor version is not.
        // This prevents ambiguities if the title contains a v.
        if minor.is_none() && major.is_empty() {
            // fake takewhile1.
            return Err(nom::Err::Error(nom::error::Error::from_error_kind(input, ErrorKind::TakeWhile1)));
        }
        Ok((input, TOSECToken::Version(v, major, minor)))
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
                        None => Some(TOSECToken::Warning(TOSECWarn::MissingSpace))
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

fn parse_unexpected_space(input: &str) -> IResult<&str, char>
{
    if let Ok(_) = peek::<&str, &str, nom::error::Error<&str>, _>(tag(" & "))(input) {
        return Err(nom::Err::Error(nom::error::Error::from_error_kind(input, ErrorKind::Tag)))
    }
    let (input, c) = char(' ')(input)?;
    Ok((input, c))
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
            peek(alt((char('('), char('[')))).map(|_| None)))
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
    vecs.push(TOSECToken::Warning(TOSECWarn::MissingDate));
    Ok((input, vecs))
}

fn parse_zzz_unk(input: &str) -> IResult<&str, TOSECToken>
{
    let (input, _) = tag("ZZZ-UNK-")(input)?;
    Ok((input, TOSECToken::Warning(TOSECWarn::ZZZUnknown)))
}

fn parse_tosec_name(input: &str) -> IResult<&str, Vec<TOSECToken>>
{
    // string must be " by ..."
    fn parse_by_publisher(input: &str) -> IResult<&str, (TOSECToken, TOSECToken)>
    {
        let (input, title) = take_until(" by ")(input)?;
        let (input, _) = tag(" by ")(input)?;
        let (input, publisher) = take_while(|c| c != '(')(input)?;
        Ok((input, (TOSECToken::Title(title.trim()), TOSECToken::Publisher(Some(vec![publisher.trim()])))))
    }

    // TOSEC Warn: Filename may begin with ZZZ-UNK-
    let (input, zzz) = opt(parse_zzz_unk)(input)?;

    let (input, mut tokens) = alt((
            parse_title_demo_date_happy,
            // TOSEC Warn: degenerate path without date
            parse_title_degenerate_path
    ))(input).or( // This means we have title only.
                  // We need to keep this around to check for by-publisher if this is a ZZZ-UNK-
        Ok(("", vec![
            TOSECToken::Title(input),
            TOSECToken::Warning(TOSECWarn::MissingDate),
            TOSECToken::Warning(TOSECWarn::MissingPublisher)
        ])))?;

    // Error checking for the next part to prevent panic
    match tokens.first()
    {
        Some(&TOSECToken::Title(_)) => {},
        _ => return Err(nom::Err::Error(
            nom::error::Error::from_error_kind(input, ErrorKind::Tag)))
    }

    let input = match zzz {
        None => input,
        Some(zzz) => {
            // ZZZ-UNK triggers special path for old-style by-publisher
            let input = match tokens.first() {
                Some(&TOSECToken::Title(title)) => {
                    if let Ok((_, (title,
                        TOSECToken::Publisher(publisher)))) = parse_by_publisher(title)
                    {
                        tokens[0] = title; // replace title.
                        tokens.insert(1, TOSECToken::Publisher(publisher));

                        tokens.insert(1,
                                      TOSECToken::Warning(TOSECWarn::ByPublisher));
                        tokens.insert(1,
                                      TOSECToken::Warning(TOSECWarn::PublisherBeforeDate));
                        input
                    }
                    else if let Ok((input, (_, publisher))) = parse_by_publisher(input)
                    {
                        // publisher happens after tags, so we have an unexpected space.
                        tokens.push(TOSECToken::Warning(TOSECWarn::UnexpectedSpace));
                        tokens.push(TOSECToken::Warning(TOSECWarn::ByPublisher));
                        tokens.push(publisher);
                        input
                    }
                    else
                    {
                        if input == "" {
                            match tokens.last() {
                                Some(TOSECToken::Warning(TOSECWarn::MissingPublisher)) => {},
                                Some(_) => tokens.push(TOSECToken::Warning(TOSECWarn::MissingPublisher)),
                                None => unreachable!()
                            }
                        }
                        input
                    }
                }
                _ => input
            };
            tokens.insert(0, zzz);
            input
        }
    };

    // TOSEC Warn: space may occur between date and publisher
    let (input, space) = opt(parse_unexpected_space)(input)?;
    if let Some(_) = space {
        tokens.push(TOSECToken::Warning(TOSECWarn::UnexpectedSpace))
    }

    // check if next parens tag is not known tag such as region or copyright
    // if so then we are missing publisher.
    // todo: may want to only trigger this behavaiour on ZZZ?
    let input = if let Err(_) = peek(alt(
        (
            parse_known_flags,
            map(parse_moreinfo_tag, |c| vec![c])
        )))(input)
    {
        // We've already established that there is no publisher.
        if let Some(TOSECToken::Warning(TOSECWarn::MissingPublisher)) = tokens.last() {
            input
        } else {
            // publisher is otherwise required...
            let (input, publisher) = parse_publisher_tag(input)?;
            tokens.push(publisher);
            input
        }
    } else {
        // If no publisher was previously parsed...
        if !tokens.iter().any(|c|
                match c { TOSECToken::Publisher(_) => true, _ => false }) {
            tokens.push(TOSECToken::Warning(TOSECWarn::MissingPublisher));
        }
        input
    };

    // Do other flags now.
    let (input, flags)
        = many0(
        pair(opt(parse_unexpected_space),
             alt((
                 parse_known_flags,
                 // parse_parens_tag mainly for system and
                 // media label tags that are an open class
                 map(parse_parens_tag, |t| vec![t])
            ))))(input)?;

    // TOSEC Warn: Space may occur between flags
    for (space, mut flag) in flags {
        if let Some(_) = space {
            tokens.push(TOSECToken::Warning(TOSECWarn::UnexpectedSpace));
        }
        tokens.append(&mut flag)
    }

    let (input, mut dumpinfos) = parse_collect_dumpinfos(input)?;
    tokens.append(&mut dumpinfos);

    let (input, rest) = many0(
        pair(
            opt(parse_unexpected_space),
            parse_moreinfo_tag)
    )(input)?;

    for (space, flag) in rest {
        if let Some(_) = space {
            tokens.push(TOSECToken::Warning(TOSECWarn::UnexpectedSpace));
        }
        tokens.push(flag)
    }

    Ok((input, tokens))
}

fn parse_collect_dumpinfos(input: &str) -> IResult<&str, Vec<TOSECToken>>
{
    let mut tokens = Vec::new();
    let input= &["cr", "f", "h", "m", "p", "t", "tr", "o", "u", "v", "b", "a", "!"]
        .iter()
        .fold(Ok(input), |input, tag|{
            // TOSEC Warn: space may occur between flags
            let (input, space) = opt(parse_unexpected_space)(input?)?;
            if let Some(_) = space {
                tokens.push(TOSECToken::Warning(TOSECWarn::UnexpectedSpace))
            }
            let (input, info) = opt(parse_dumpinfo_tag(tag))(input)?;
            if let Some(info) =  info {
                tokens.push(info);
            }
            Ok(input)
        })?;
    Ok((input, tokens))
}

fn parse_known_flags(input: &str) -> IResult<&str, Vec<TOSECToken>>
{
    let (input, tags) = alt((
            map(alt((
                parse_region_tag,
                parse_language_tag,
                parse_system_tag,
                parse_video_tag,
                parse_copyright_tag,
                parse_media_tag)),
                |c| vec![c]),
            parse_goodtools_region_tag,
            parse_devstatus_tag,
            parse_version_tag
    ))(input)?;
    Ok((input, tags))
}

pub(crate) fn do_parse(input: &str) -> IResult<&str, Vec<TOSECToken>>
{
    let (input, mut tokens) = parse_tosec_name(input)?;
    match input {
        "" => Ok((input, tokens)),
        rest => {
            tokens.push(TOSECToken::Warning(TOSECWarn::NotEof(rest)));
            Ok((input, tokens))
        }
    }
}

pub(crate) fn do_parse_multiset(input: &str)-> IResult<&str, (Vec<Vec<TOSECToken>>, Vec<TOSECToken>)>
{
    let (input, tokens) =
        separated_list1(
            tag(" & "),
            parse_tosec_name
        )(input)?;

    let mut globals = Vec::new();

    let input = if let Ok((input, _)) = char::<&str, nom::error::Error<&str>>('-')(input) {
        let (input, flags)
            = many0(
            pair(opt(parse_unexpected_space),
                 alt((
                     parse_known_flags,
                     map(parse_parens_tag, |t| vec![t])
                 ))))(input)?;

        for (space, mut flag) in flags {
            if let Some(_) = space {
                globals.push(TOSECToken::Warning(TOSECWarn::UnexpectedSpace));
            }
            globals.append(&mut flag)
        }

        let (input, mut dumpinfos) = parse_collect_dumpinfos(input)?;
        globals.append(&mut dumpinfos);

        let (input, rest) = many0(
            pair(
                opt(parse_unexpected_space),
                parse_moreinfo_tag)
        )(input)?;

        for (space, flag) in rest {
            if let Some(_) = space {
                globals.push(TOSECToken::Warning(TOSECWarn::UnexpectedSpace));
            }
            globals.push(flag)
        }
        input
    } else { input };

    match input {
        "" => Ok((input, (tokens, globals))),
        rest => {
            globals.push(TOSECToken::Warning(TOSECWarn::NotEof(rest)));
            Ok((input, (tokens, globals)))
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
    fn test_repair()
    {
        assert_eq!(do_parse("ZZZ-UNK-Loopz (U) (Beta) (v0.06)")
                       .map(|(i, e)| (i, e.into()))
                       .map(|(i, e): (&str, TOSECName)| (i, e.into_strict())),
                   Ok(("", vec![
                       TOSECToken::Title("Loopz"),
                       TOSECToken::Version("v", "0", Some("06")),
                       TOSECToken::Date("19xx", None, None),
                       TOSECToken::Publisher(None),
                       TOSECToken::Region(vec!["US"], vec![Region::UnitedStates]),
                       TOSECToken::Development("beta")
                   ].into())));
    }
    #[test]
    fn test_parse_multi()
    {
        assert_eq!(
            do_parse_multiset("Amidar (19xx)(Devstudio) & Amigos (1987)(Mr. Tosec)-(PD)(Disk 1 of 2)[a]"),
            Ok(("",
                (
                    vec![
                        vec![
                            TOSECToken::Title("Amidar"),
                            TOSECToken::Date("19xx", None, None),
                            TOSECToken::Publisher(Some(vec!["Devstudio"]))
                        ],
                        vec![
                            TOSECToken::Title("Amigos"),
                            TOSECToken::Date("1987", None, None),
                            TOSECToken::Publisher(Some(vec!["Mr. Tosec"]))
                        ],
                    ],
                    vec![
                        TOSECToken::Copyright("PD"),
                        TOSECToken::Media(vec![("Disk", "1", Some("2"))]),
                        TOSECToken::DumpInfo("a", None, None)
                    ]
                )
            ))
        );
    }

    #[test]
    fn test_zzz_unks()
    {
        assert_eq!(do_parse("ZZZ-UNK-Raiden (U) (CES Version) (v3.0)"),
                   Ok(("",
                       vec![
                           TOSECToken::Warning(TOSECWarn::ZZZUnknown),
                           TOSECToken::Title("Raiden"),
                           TOSECToken::Warning(TOSECWarn::MissingDate),
                           TOSECToken::Warning(TOSECWarn::MissingPublisher),
                           TOSECToken::Warning(TOSECWarn::GoodToolsRegionCode("U")),
                           TOSECToken::Region(vec!["U"], vec![Region::UnitedStates]),
                           TOSECToken::Warning(TOSECWarn::UnexpectedSpace),
                           TOSECToken::Flag(FlagType::Parenthesized, "CES Version"),
                           TOSECToken::Warning(TOSECWarn::UnexpectedSpace),
                           TOSECToken::Warning(TOSECWarn::VersionInFlag),
                           TOSECToken::Version("v", "3", Some("0"))
                       ])));

        assert_eq!(do_parse("ZZZ-UNK-Space Lock (Rev 20040529)(Beta)"),
                   Ok(("",
                       vec![
                           TOSECToken::Warning(TOSECWarn::ZZZUnknown),
                           TOSECToken::Title("Space Lock"),
                           TOSECToken::Warning(TOSECWarn::MissingDate),
                           TOSECToken::Warning(TOSECWarn::MissingPublisher),
                           TOSECToken::Warning(TOSECWarn::VersionInFlag),
                           TOSECToken::Version("Rev", "20040529", None),
                           TOSECToken::Warning(TOSECWarn::MalformedDevelopmentStatus("Beta")),
                           TOSECToken::Development("Beta")
                       ]
                   ))
        );

        assert_eq!(do_parse("ZZZ-UNK-Loopz (U) (Beta) (v0.06)"),
            Ok(("", vec![
                TOSECToken::Warning(TOSECWarn::ZZZUnknown),
                TOSECToken::Title("Loopz"),
                TOSECToken::Warning(TOSECWarn::MissingDate),
                TOSECToken::Warning(TOSECWarn::MissingPublisher),
                TOSECToken::Warning(TOSECWarn::GoodToolsRegionCode("U")),
                TOSECToken::Region(vec!["U"], vec![Region::UnitedStates]),
                TOSECToken::Warning(TOSECWarn::UnexpectedSpace),
                TOSECToken::Warning(TOSECWarn::MalformedDevelopmentStatus("Beta")),
                TOSECToken::Development("Beta"),
                TOSECToken::Warning(TOSECWarn::UnexpectedSpace),
                TOSECToken::Warning(TOSECWarn::VersionInFlag),
                TOSECToken::Version("v", "0", Some("06"))])
        ));

        assert_eq!(do_parse("ZZZ-UNK-Tron 6 fun v0.15"),
        Ok(("",
            vec![
                TOSECToken::Warning(TOSECWarn::ZZZUnknown),
                TOSECToken::Title("Tron 6 fun"),
                TOSECToken::Version("v", "0", Some("15")),
                TOSECToken::Warning(TOSECWarn::MissingDate),
                TOSECToken::Warning(TOSECWarn::MissingPublisher)])
        ));

        assert_eq!(
            do_parse("ZZZ-UNK-Poker Game 512 (2005-06-20)"),
            Ok(("",
                vec![
                    TOSECToken::Warning(TOSECWarn::ZZZUnknown),
                    TOSECToken::Title("Poker Game 512"),
                    TOSECToken::Date("2005", Some("06"), Some("20")),
                    TOSECToken::Warning(TOSECWarn::MissingPublisher)])
            )
        );

        assert_eq!(
            do_parse("ZZZ-UNK-rapide_racer [lyx]"),
            Ok(("",
                vec![
                    TOSECToken::Warning(TOSECWarn::ZZZUnknown),
                    TOSECToken::Title("rapide_racer"),
                    TOSECToken::Warning(TOSECWarn::MissingDate),
                    TOSECToken::Warning(TOSECWarn::MissingPublisher),
                    TOSECToken::Flag(FlagType::Bracketed, "lyx")])
            )
        );

        assert_eq!(
            do_parse("ZZZ-UNK-Befok#Packraw (20021012) by Jum Hig (PD)"),
            Ok(("", vec![
                TOSECToken::Warning(TOSECWarn::ZZZUnknown),
                TOSECToken::Title("Befok#Packraw"),
                TOSECToken::Warning(TOSECWarn::UndelimitedDate("20021012")),
                TOSECToken::Date("2002", Some("10"), Some("12")),
                TOSECToken::Warning(TOSECWarn::UnexpectedSpace),
                TOSECToken::Warning(TOSECWarn::ByPublisher),
                TOSECToken::Publisher(Some(vec!["Jum Hig"])),
                TOSECToken::Copyright("PD")])
            ));
    }

    #[test]
    fn parse_to_string()
    {
        for string in &[
            "ZZZ-UNK-Befok#Packraw (20021012) by Jum Hig (PD)",
            "ZZZ-UNK-ATARI",
            "ZZZ-UNK-Clicks! (test) by Domin, Matthias (2001) (PD)",
            "Bombsawa (Jumpman Selected levels)(19XX)(-)(JP)(ja)(PD)[cr3 +test][h][test flag]",
            "Dune - The Battle for Arrakis Demo Hack (2009-04-03)(Ti_)[h Dune - The Battle for Arrakis]",
            "ZZZ-UNK-Raiden (U) (CES Version) (v3.0)"
        ]
        {
            assert_eq!(
                TOSECName::try_parse(string).map(|e| e.to_string()),
                Ok(String::from(*string))
            )
        }
    }


    #[test]
    fn globals_to_string()
    {
        for string in &[
            "Amidar (19xx)(Devstudio) & Amigos (1987)(Mr. Tosec)-(PD)(Disk 1 of 2)[a]"
        ]
        {
            assert_eq!(
                TOSECMultiSetName::try_parse(string).map(|e| e.to_string()),
                Ok(String::from(*string))
            )
        }
    }

    #[test]
    fn test_missing_publish_once()
    {
        assert_eq!(do_parse("ZZZ-UNK-ATARI"),
                   Ok(("",
                       vec![
                           TOSECToken::Warning(TOSECWarn::ZZZUnknown),
                           TOSECToken::Title("ATARI"),
                           TOSECToken::Warning(TOSECWarn::MissingDate),
                           TOSECToken::Warning(TOSECWarn::MissingPublisher),
                       ]
                   ))
        )
    }

    #[test]
    fn test_ambiguous_version()
    {
        assert_eq!(do_parse("ZZZ-UNK-Alien vs Predator (U) (Beta)"),
                   Ok(("",
                       vec![
                           TOSECToken::Warning(TOSECWarn::ZZZUnknown),
                           TOSECToken::Title("Alien vs Predator"),
                           TOSECToken::Warning(TOSECWarn::MissingDate),
                           TOSECToken::Warning(TOSECWarn::MissingPublisher),
                           TOSECToken::Warning(TOSECWarn::GoodToolsRegionCode("U")),
                           TOSECToken::Region(vec!["U"], vec![Region::UnitedStates]),
                           TOSECToken::Warning(TOSECWarn::UnexpectedSpace),
                           TOSECToken::Warning(TOSECWarn::MalformedDevelopmentStatus("Beta")),
                           TOSECToken::Development("Beta")
                       ]
                   ))
        )
    }


    #[test]
    fn test_by_publisher()
    {
        assert_eq!(
            do_parse("ZZZ-UNK-Micro Font Dumper by Schick, Bastian (199x) (PD)"),
            Ok(("",
                vec![
                    TOSECToken::Warning(TOSECWarn::ZZZUnknown),
                    TOSECToken::Title("Micro Font Dumper"),
                    TOSECToken::Warning(TOSECWarn::PublisherBeforeDate),
                    TOSECToken::Warning(TOSECWarn::ByPublisher),
                    TOSECToken::Publisher(Some(vec!["Schick, Bastian"])),
                    TOSECToken::Date("199x", None, None),
                    TOSECToken::Warning(TOSECWarn::UnexpectedSpace),
                    TOSECToken::Copyright("PD")
                ]
            ))
        );

        let parse = do_parse("ZZZ-UNK-Clicks! (test) by Domin, Matthias (2001) (PD)");
        assert_eq!(&parse,
            &Ok(("",
                vec![
                    TOSECToken::Warning(TOSECWarn::ZZZUnknown),
                    TOSECToken::Title("Clicks! (test)"),
                    TOSECToken::Warning(TOSECWarn::PublisherBeforeDate),
                    TOSECToken::Warning(TOSECWarn::ByPublisher),
                    TOSECToken::Publisher(Some(vec!["Domin, Matthias"])),
                    TOSECToken::Date("2001", None, None),
                    TOSECToken::Warning(TOSECWarn::UnexpectedSpace),
                    TOSECToken::Copyright("PD")
                ]
            ))
        );

        let (_, parse) = parse.unwrap();
        let parse: TOSECName = parse.into();
        println!("{:?}", parse.into_strict());

    }

    #[test]
    fn test_undelim_date()
    {
        assert_eq!(
            do_parse("ZZZ-UNK-Befok#Packraw (20021012)(Jum Hig)"),
            Ok(("",
                vec![
                    TOSECToken::Warning(TOSECWarn::ZZZUnknown),
                    TOSECToken::Title("Befok#Packraw"),
                    TOSECToken::Warning(TOSECWarn::UndelimitedDate("20021012")),
                    TOSECToken::Date("2002", Some("10"), Some("12")),
                    TOSECToken::Publisher(Some(vec!["Jum Hig"])),
                ]
            ))
        )
    }

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
                    TOSECToken::Region(vec!["FI"], vec![Region::Finland])
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
                    TOSECToken::Warning(TOSECWarn::ZZZUnknown),
                    TOSECToken::Title("Show King Tut"),
                    TOSECToken::Date("1996", None, None),
                    TOSECToken::Warning(TOSECWarn::UnexpectedSpace),
                    TOSECToken::Publisher(Some(vec!["Schick, Bastian"])),
                    TOSECToken::Warning(TOSECWarn::UnexpectedSpace),
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
                    TOSECToken::Warning(TOSECWarn::ZZZUnknown),
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
        let parse = do_parse("Escape from the Mindmaster (1982)(Starpath)(PAL)(Part 3 of 4)[Supercharger Cassette]");
        assert_eq!(
            &parse,
            &Ok(("",
                vec![
                    TOSECToken::Title("Escape from the Mindmaster"),
                    TOSECToken::Date("1982", None, None),
                    TOSECToken::Publisher(Some(vec!["Starpath"])),
                    TOSECToken::Video("PAL"),
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
                    TOSECToken::Warning(TOSECWarn::MissingSpace),
                    TOSECToken::Date("1997", Some("10"), Some("03")),
                    TOSECToken::Publisher(Some(vec!["Cracknell, Chris 'Crackers'"])),
                    TOSECToken::Video("NTSC"),
                    TOSECToken::Copyright("PD"),
                ]
            )));
        assert_eq!(
            do_parse("Cube CD 20, The (40) - Testing v1.203 (demo) (2020)(SomePublisher)"),
            Ok(("",
                vec![
                    TOSECToken::Title("Cube CD 20, The (40) - Testing"),
                    TOSECToken::Version("v", "1", Some("203")),
                    TOSECToken::Demo(None),
                    TOSECToken::Date("2020", None, None),
                    TOSECToken::Publisher(Some(vec!["SomePublisher"])),
                ])));

        assert_eq!(
            do_parse("Motocross & Pole Position Rev 1 (Starsoft - JVP)(PAL)[b1][possible unknown mode]"),
            Ok(("",
                vec![
                    TOSECToken::Title("Motocross & Pole Position"),
                    TOSECToken::Version("Rev", "1", None),
                    TOSECToken::Warning(TOSECWarn::MissingDate),
                    TOSECToken::Publisher(Some(vec!["Starsoft", "JVP"])),
                    TOSECToken::Video("PAL"),
                    TOSECToken::DumpInfo("b", Some("1"), None),
                    TOSECToken::Flag(FlagType::Bracketed, "possible unknown mode"),
                ]))
        );
        assert_eq!(
            do_parse("Motocross & Pole Position (Starsoft - JVP)(PAL)[b1][possible unknown mode]"),
            Ok(("",
                vec![
                    TOSECToken::Title("Motocross & Pole Position"),
                    TOSECToken::Warning(TOSECWarn::MissingDate),
                    TOSECToken::Publisher(Some(vec!["Starsoft", "JVP"])),
                    TOSECToken::Video("PAL"),
                    TOSECToken::DumpInfo("b", Some("1"), None),
                    TOSECToken::Flag(FlagType::Bracketed, "possible unknown mode"),
                ]))
        );
        assert_eq!(
            do_parse("Bombsawa (Jumpman Selected levels)(19XX)(-)(JP)(ja)(PD)[cr3 +test][h][test flag]"),
            Ok(("",
                vec![
                    TOSECToken::Title("Bombsawa (Jumpman Selected levels)"),
                    TOSECToken::Warning(TOSECWarn::MissingSpace),
                    TOSECToken::Warning(TOSECWarn::MalformedDatePlaceholder("19XX")),
                    TOSECToken::Date("19XX", None, None),
                    TOSECToken::Publisher(None),
                    TOSECToken::Region(vec!["JP"], vec![Region::Japan]),
                    TOSECToken::Languages(TOSECLanguage::Single("ja")),
                    TOSECToken::Copyright("PD"),
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
                    TOSECToken::Video("NTSC"),
                    TOSECToken::Region(vec!["BR"], vec![Region::Brazil]),]
            ))
        );

        assert_eq!(
            do_parse("Mega Man III - Sample version (1992)(Capcom)(US)"),
            Ok(("",
                vec![
                    TOSECToken::Title("Mega Man III - Sample version"),
                    TOSECToken::Date("1992", None, None),
                    TOSECToken::Publisher(Some(vec!["Capcom"])),
                    TOSECToken::Region(vec!["US"], vec![Region::UnitedStates]),]
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
                   Ok(((""), TOSECToken::Version("v", "1", Some("0a")))));
        assert_eq!(parse_version_string("Rev 1b"),
                   Ok(((""), TOSECToken::Version("Rev", "1b", None))));
        assert_eq!(parse_version_string("v20000101"),
                   Ok(((""), TOSECToken::Version("v", "20000101", None))));
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
        assert_eq!(parse_region("US"), Ok(("", (vec!["US"], vec![Region::UnitedStates]))));
        assert_eq!(parse_region("US-ZZ"), Ok(("", (vec!["US", "ZZ"], vec![Region::UnitedStates, Region::Unknown]))));
    }

    #[test]
    fn test_parse_region_tag()
    {
        assert_eq!(parse_region_tag("(US)"), Ok(("", TOSECToken::Region(vec!["US"],
                                                                        vec![Region::UnitedStates]))));
        assert_eq!(parse_region_tag("(US-ZZ)"), Ok(("", TOSECToken::Region(vec!["US", "ZZ"],
                                                                           vec![Region::UnitedStates, Region::Unknown]))));
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