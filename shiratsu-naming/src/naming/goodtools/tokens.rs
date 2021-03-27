use crate::region::Region;
use crate::naming::{FlagType, NamingConvention};
use crate::naming::goodtools::parsers::do_parse;
use crate::error::{NameError, Result};
use std::slice::Iter;
use std::fmt::{Display, Formatter};
use std::fmt;

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
/// A GoodTools format file name.
pub struct GoodToolsName<'a>(Vec<GoodToolsToken<'a>>);

impl GoodToolsName<'_>
{
    /// Returns an iterator over the tokens of this name.
    pub fn iter(&self) -> Iter<'_, GoodToolsToken>
    {
        self.0.iter()
    }

    pub fn try_parse<S: AsRef<str> + ?Sized>(input: &S) -> Result<GoodToolsName> {
        let (_, value) = do_parse(input.as_ref()).map_err(|_| {
            NameError::ParseError(NamingConvention::GoodTools, input.as_ref().to_string())
        })?;
        Ok(value.into())
    }
}

impl <'a> From<Vec<GoodToolsToken<'a>>> for GoodToolsName<'a>
{
    fn from(vec: Vec<GoodToolsToken<'a>>) -> Self {
        GoodToolsName(vec)
    }
}

impl Display for GoodToolsName<'_>
{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let mut buf = String::new();

        unimplemented!();

        for token in self.iter() {
            match token {
                GoodToolsToken::Title(_) => {}
                GoodToolsToken::Region(_, _) => {}
                GoodToolsToken::Year(_) => {}
                GoodToolsToken::Multilanguage(_) => {}
                GoodToolsToken::Translation(_, _) => {}
                GoodToolsToken::Version(_, _, _) => {}
                GoodToolsToken::Volume(_) => {}
                GoodToolsToken::NInOne(_, _) => {}
                GoodToolsToken::DumpCode(_, _, _, _, _, _) => {}
                GoodToolsToken::GameHack(_) => {}
                GoodToolsToken::Media(_, _, _) => {}
                GoodToolsToken::Flag(_, _) => {}
            }
        }

        f.write_str(buf.trim())
    }
}