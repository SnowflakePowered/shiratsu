use super::super::common::parsers::*;

use nom::{
    bytes::complete::{
        tag,
        is_not
    },
    error::{Error, ErrorKind},
    IResult, Slice,
};

use crate::region::{Region, RegionError};

#[derive(Debug, Eq, PartialEq)]
enum NoIntroToken<'a>
{

    Region(Vec<Region>),
    Flag(&'a str),
    Version(&'a str)
}

fn parse_region(input: &str) -> IResult<&str, Vec<Region>>
{
    // let regions = Region::try_from_nointro_region(input)
    //     .map_err(|e|
    //         {
    //             match e {
    //                 RegionError::BadRegionCode(_, _, idx)
    //                     => nom::Err::Failure(Error::new(input.slice(idx..),
    //                                                     ErrorKind::Tag)),
    //                 _ => nom::Err::Error(Error::new(input, ErrorKind::Tag))
    //             }
    //         })?;
    Ok(("", vec![]))
}

fn parse_region_tag(input: &str) -> IResult<&str, NoIntroToken>
{
    // I will allow this one hack :|
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
    ($fn_name:ident, $tag:literal) =>
    {
        fn $fn_name<'a>(input: &'a str) -> IResult<&'a str, NoIntroToken>
        {
            let (input, tag) = in_brackets(tag($tag))(input)?;
            Ok((input, NoIntroToken::Flag(tag)))
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
nointro_parens_flag_parser!(parse_nil_beta_tag, "Beta");


#[cfg(test)]
mod tests
{
    use crate::parse::nointro::parsers::{parse_region_tag, parse_region, NoIntroToken, parse_unlicensed_tag};
    use crate::region::Region;
    use nom::error::{ErrorKind, Error};

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
            Err(nom::Err::Failure(Error::new("Apustralia, New Zealand", ErrorKind::Tag))))
    }

    #[test]
    fn parse_unl()
    {
        assert_eq!(parse_unlicensed_tag("(Unl)"), Ok(("", NoIntroToken::Flag("Unl"))))
    }
}
