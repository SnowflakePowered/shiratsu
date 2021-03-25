use crate::region::Region;
use crate::naming::{FlagType, NamingConvention, ToNameInfo, NameInfo, DevelopmentStatus};
use crate::naming::goodtools::parsers::do_parse;
use crate::error::{ParseError, Result};
use crate::naming::util::{move_default_articles_mut, replace_hyphen_mut};

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum GoodToolsToken<'a>
{
    Title(&'a str),
    Region(Vec<&'a str>, Vec<Region>),
    Year(&'a str),
    Multilanguage(&'a str), // (M#)
    Translation(TranslationStatus, &'a str), // [T(+/-)...]
    Version(&'a str, &'a str, Option<&'a str>), // (REV/V/V /V_ ...)
    Volume(&'a str), // (Vol #)
    NInOne(Vec<&'a str>, Option<&'a str>), // list, sep (either + or ,)
    DumpCode(&'a str, Option<&'a str>, Option<&'a str>, Option<&'a str>, Option<&'a str>, Option<&'a str>), // (code, number, type, sep, argnum, args)
    GameHack(Option<&'a str>), // (... Hack)
    Media(&'a str, &'a str, Option<&'a str>),
    Flag(FlagType, &'a str),
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum TranslationStatus
{
    Recent,
    Outdated,
}

#[derive(Debug, Clone, Eq, PartialEq)]
#[repr(transparent)]
pub struct GoodToolsName<'a>(Vec<GoodToolsToken<'a>>);

impl GoodToolsName<'_>
{
    pub fn try_parse<S: AsRef<str> + ?Sized>(input: &S) -> Result<GoodToolsName> {
        let (_, value) = do_parse(input.as_ref()).map_err(|_| {
            ParseError::BadFileNameError(NamingConvention::GoodTools, input.as_ref().to_string())
        })?;
        Ok(value.into())
    }
}

impl <'a> ToNameInfo for GoodToolsName<'a>
{
    fn to_name_info(&self) -> NameInfo {
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
        for token in self.0.iter() {
            match token {
                GoodToolsToken::Title(t) => name.entry_title = t.to_string(),
                GoodToolsToken::Region(_, region) => name.region = region.clone(),
                GoodToolsToken::Version(_, major, Some(minor)) =>
                    name.version = Some(format!("{}.{}", major, minor)),
                GoodToolsToken::Version(_, major, _) =>
                    name.version = Some(major.to_string()),
                GoodToolsToken::Flag(FlagType::Parenthesized, "Unl")
                    => name.is_unlicensed = true,
                GoodToolsToken::Flag(FlagType::Parenthesized, "Kiosk Demo")
                    | GoodToolsToken::Flag(FlagType::Parenthesized, "Demo")
                    => name.is_demo = true,
                GoodToolsToken::Flag(FlagType::Parenthesized, "Beta")
                    | GoodToolsToken::Flag(FlagType::Parenthesized, "Alpha")
                    | GoodToolsToken::Flag(FlagType::Parenthesized, "Pre-Release")
                => name.status = DevelopmentStatus::Prerelease,
                GoodToolsToken::Flag(FlagType::Parenthesized, "Prototype")
                    => name.status = DevelopmentStatus::Prototype,
                _ => {}
            }
        }

        let mut release_title = name.entry_title.clone();

        move_default_articles_mut(&mut release_title);
        replace_hyphen_mut(&mut release_title);
        name.release_title = release_title;
        name
    }
}

impl <'a> From<Vec<GoodToolsToken<'a>>> for GoodToolsName<'a>
{
    fn from(vec: Vec<GoodToolsToken<'a>>) -> Self {
        GoodToolsName(vec)
    }
}

impl <'a> From<GoodToolsName<'a>> for Vec<GoodToolsToken<'a>>
{
    fn from(name: GoodToolsName<'a>) -> Self {
        name.0
    }
}

impl <'a> AsRef<Vec<GoodToolsToken<'a>>> for GoodToolsName<'a>
{
    fn as_ref(&self) -> &Vec<GoodToolsToken<'a>> {
        &self.0
    }
}
