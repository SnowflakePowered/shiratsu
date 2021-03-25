use crate::region::Region;
use crate::naming::FlagType;

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
