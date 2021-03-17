use crate::region::Region;
use crate::naming::FlagType;

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
