use crate::region::Region;
use crate::naming::FlagType;

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

    /// 'ZZZ-UNK-' Unknown prefix for TOSEC v3
    ZZZUnkPrefix,
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
