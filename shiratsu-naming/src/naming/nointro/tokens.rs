use crate::region::Region;
use crate::naming::{FlagType, NamingConvention};
use crate::error::{NameError, Result};
use crate::naming::nointro::parsers::do_parse;
use std::slice::Iter;

/// A parsed language code.
#[derive(Debug, Clone, Eq, PartialEq)]
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
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum NoIntroToken<'a>
{
    /// The title of the game.
    Title(&'a str),

    /// A list of parsed regions.
    Region(Vec<&'a str>, Vec<Region>),

    /// An unspecified regular flag
    Flag(FlagType, &'a str),

    /// The parsed version.
    Version(Vec<(&'a str, &'a str, Option<&'a str>, Option<&'a str>, Option<Vec<&'a str>>)>),
    Release(&'a str, Option<&'a str>),

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

#[derive(Debug, Clone, Eq, PartialEq)]
#[repr(transparent)]
/// A No-Intro format file name.
pub struct NoIntroName<'a>(Vec<NoIntroToken<'a>>);
impl NoIntroName<'_>
{
    /// Tries to parse the name into a vector of tokens.
    pub fn try_parse<S: AsRef<str> + ?Sized>(input: &S) -> Result<NoIntroName> {
        let (_, value) = do_parse(input.as_ref()).map_err(|_| {
            NameError::BadFileNameError(NamingConvention::NoIntro, input.as_ref().to_string())
        })?;
        Ok(value.into())
    }

    #[inline]
    /// Returns an iterator over the tokens of this name.
    pub fn iter(&self) -> Iter<'_, NoIntroToken>
    {
        self.0.iter()
    }
}

impl <'a> From<Vec<NoIntroToken<'a>>> for NoIntroName<'a>
{
    fn from(vec: Vec<NoIntroToken<'a>>) -> Self {
        NoIntroName(vec)
    }
}

impl <'a> From<NoIntroName<'a>> for Vec<NoIntroToken<'a>>
{
    fn from(name: NoIntroName<'a>) -> Self {
        name.0
    }
}

impl <'a> AsRef<Vec<NoIntroToken<'a>>> for NoIntroName<'a>
{
    fn as_ref(&self) -> &Vec<NoIntroToken<'a>> {
        &self.0
    }
}
