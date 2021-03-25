use crate::region::Region;
use crate::naming::{FlagType, NamingConvention};

use crate::error::{NameError, Result};
use crate::naming::tosec::parsers::{do_parse, do_parse_multiset};

use std::cmp::Ordering;
use std::slice::Iter;

#[derive(Debug, Eq, Clone, Ord)]
pub enum TOSECToken<'a>
{
    Title(&'a str),
    Version(&'a str, &'a str, Option<&'a str>),
    Demo(Option<&'a str>),
    Date(&'a str, Option<&'a str>, Option<&'a str>),
    Publisher(Option<Vec<&'a str>>),
    System(&'a str),
    Video(&'a str),
    /// A list of parsed regions.
    Region(Vec<&'a str>, Vec<Region>),
    /// A vector of language tuples (Code, Variant).
    Languages(TOSECLanguage<'a>),
    Copyright(&'a str),
    Development(&'a str),
    DumpInfo(&'a str, Option<&'a str>, Option<&'a str>),
    /// Media parts
    Media(Vec<(&'a str, &'a str, Option<&'a str>)>),
    /// An unspecified regular flag
    Flag(FlagType, &'a str),
    /// A warning occurred
    Warning(TOSECWarn<'a>)
}

impl PartialEq for TOSECToken<'_>
{
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (TOSECToken::Title(t), TOSECToken::Title(o)) => t.eq(o),
            (TOSECToken::Version(a, b, c),
                TOSECToken::Version(e, f, g))
            => (a, b, c).eq(&(e, f, g)),
            (TOSECToken::Demo(t), TOSECToken::Demo(o))
            => t.eq(o),
            (TOSECToken::Date(y, m, d),
                TOSECToken::Date(y2, m2, d2))
            => (y, m, d).eq(&(y2, m2, d2)),
            (TOSECToken::Publisher(a), TOSECToken::Publisher(b))
            => a.eq(b),
            (TOSECToken::System(a), TOSECToken::System(b))
            => a.eq(b),
            (TOSECToken::Video(a), TOSECToken::Video(b))
            => a.eq(b),
            (TOSECToken::Region(a, _), TOSECToken::Region(b, _))
            // region equality depends on the string.
            => a.eq(b),
            (TOSECToken::Languages(a), TOSECToken::Languages(b))
            => a.eq(b),
            (TOSECToken::Copyright(a), TOSECToken::Copyright(b))
            => a.eq(b),
            (TOSECToken::Development(a), TOSECToken::Development(b))
            => a.eq(b),
            (TOSECToken::Media(a), TOSECToken::Media(b))
            => a.eq(b),
            // Presumably media type
            (TOSECToken::Flag(FlagType::Parenthesized, a), TOSECToken::Flag(FlagType::Parenthesized, b))
            => a.eq(b),
            (TOSECToken::DumpInfo(a, n, f),
                TOSECToken::DumpInfo(a1, n2, f2)) => {
                (a, n, f).eq(&(a1, n2, f2))
            },
            (TOSECToken::Flag(FlagType::Bracketed, a), TOSECToken::Flag(FlagType::Bracketed, b))
            => a.eq(b),
            (TOSECToken::Warning(a), TOSECToken::Warning(b)) =>
                a.eq(b),
            _ => false
        }
    }
}

impl PartialOrd for TOSECToken<'_>
{
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        fn get_priority(token: &TOSECToken) -> usize {
            match token {
                TOSECToken::Title(_) => 0,
                TOSECToken::Version(_, _, _) => 1,
                TOSECToken::Demo(_) => 2,
                TOSECToken::Date(_, _, _) => 3,
                TOSECToken::Publisher(_) => 4,
                TOSECToken::System(_) => 5,
                TOSECToken::Video(_) => 6,
                TOSECToken::Region(_, _) => 7,
                TOSECToken::Languages(_) => 8,
                TOSECToken::Copyright(_) => 9,
                TOSECToken::Development(_) => 10,
                TOSECToken::Media(_) => 11,
                // Presumably media type
                TOSECToken::Flag(FlagType::Parenthesized, _) => 12,
                TOSECToken::DumpInfo(_, _, _) => 13,
                TOSECToken::Flag(FlagType::Bracketed, _) => 14,
                TOSECToken::Warning(_) => usize::max_value()
            }
        }

        fn get_dumpinfo_priority(input: &str) -> usize {
            match input {
                "cr" => 0,
                "f" => 1,
                "h" => 2,
                "m" => 3,
                "p" => 4,
                "t" => 5,
                "tr" => 6,
                "o" => 7,
                "u" => 8,
                "v" => 9,
                "b" => 10,
                "a" => 11,
                "!" => 12,
                _ => usize::max_value()
            }
        }

        let self_priority = get_priority(&self);
        let other_priority = get_priority(&other);
        if self_priority != other_priority {
            return self_priority.partial_cmp(&other_priority);
        }
        match (self, other) {
            (TOSECToken::Title(t), TOSECToken::Title(o)) => t.partial_cmp(o),
            (TOSECToken::Version(a, b, c),
                TOSECToken::Version(e, f, g))
                => (a, b, c).partial_cmp(&(e, f, g)),
            (TOSECToken::Demo(t), TOSECToken::Demo(o))
                => t.partial_cmp(o),
            (TOSECToken::Date(y, m, d),
                TOSECToken::Date(y2, m2, d2))
                => (y, m, d).partial_cmp(&(y2, m2, d2)),
            (TOSECToken::Publisher(a), TOSECToken::Publisher(b))
                => a.partial_cmp(b),
            (TOSECToken::System(a), TOSECToken::System(b))
                => a.partial_cmp(b),
            (TOSECToken::Video(a), TOSECToken::Video(b))
                => a.partial_cmp(b),
            (TOSECToken::Region(a, _), TOSECToken::Region(b, _))
                => a.partial_cmp(b),
            (TOSECToken::Languages(a), TOSECToken::Languages(b))
                => a.partial_cmp(b),
            (TOSECToken::Copyright(a), TOSECToken::Copyright(b))
                => a.partial_cmp(b),
            (TOSECToken::Development(a), TOSECToken::Development(b))
                => a.partial_cmp(b),
            (TOSECToken::Media(a), TOSECToken::Media(b))
                => a.partial_cmp(b),
            // Presumably media type
            (TOSECToken::Flag(FlagType::Parenthesized, a), TOSECToken::Flag(FlagType::Parenthesized, b))
                => a.partial_cmp(b),
            (TOSECToken::DumpInfo(a, n, f),
                TOSECToken::DumpInfo(a1, n2, f2)) => {
                if a == a1 {
                    if n != n2 {
                        n.partial_cmp(n2)
                    } else {
                        f.partial_cmp(f2)
                    }
                } else {
                    get_dumpinfo_priority(a).partial_cmp(&get_dumpinfo_priority(a1))
                }
            },
            (TOSECToken::Flag(FlagType::Bracketed, a), TOSECToken::Flag(FlagType::Bracketed, b))
                => a.partial_cmp(b),
            (TOSECToken::Warning(a), TOSECToken::Warning(b)) =>
                a.partial_cmp(b),
            _ => unreachable!()
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, PartialOrd, Ord)]
pub enum TOSECWarn<'a>
{
    ZZZUnknown,
    MalformedDatePlaceholder(&'a str),
    MalformedDevelopmentStatus(&'a str),
    UndelimitedDate(&'a str),
    MissingDate,
    MissingPublisher,
    MissingSpace,
    UnexpectedSpace,
    ByPublisher,
    PublisherBeforeDate,
    GoodToolsRegionCode(&'a str),
    VersionInFlag,
    NotEof(&'a str)
}

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub enum TOSECLanguage<'a>
{
    /// A single language code
    Single(&'a str),
    /// A double language
    Double(&'a str, &'a str),
    /// A multi-language indicator without the leading 'M'
    Count(&'a str),
}

impl TOSECToken<'_> {
    pub fn is_warning(&self) -> bool {
        match self {
            TOSECToken::Warning(_) => true,
            _ => false,
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
#[repr(transparent)]
/// A TOSEC format file name.
pub struct TOSECName<'a>(Vec<TOSECToken<'a>>);

impl <'a> From<Vec<TOSECToken<'a>>> for TOSECName<'a>
{
    fn from(vec: Vec<TOSECToken<'a>>) -> Self {
        TOSECName(vec)
    }
}

impl <'a> From<TOSECName<'a>> for Vec<TOSECToken<'a>>
{
    fn from(name: TOSECName<'a>) -> Self {
        name.0
    }
}

impl <'a> AsRef<Vec<TOSECToken<'a>>> for TOSECName<'a>
{
    fn as_ref(&self) -> &Vec<TOSECToken<'a>> {
        &self.0
    }
}

impl TOSECName<'_> {

    #[inline]
    /// Returns an iterator over the tokens of this name.
    pub fn iter(&self) -> Iter<'_, TOSECToken>
    {
        self.0.iter()
    }

    /// Makes the name conform strictly to the TOSEC naming conventions.
    ///
    /// This removes any warning tokens and ensures the order of flags is proper.
    ///
    /// # Fixes
    /// - If there is no date, 19xx is added as the date.
    /// - If there is no publisher, the unknown publisher (-) is added
    /// - GoodTools region codes are converted into the ISO equivalent
    /// - Publishers are sorted lexicographically
    /// - Tags are put in the order
    ///    ```order
    ///   Title version (demo) (date)(publisher)(system)(video)(country)(language)
    ///   (copyright status)(development status)(media type)(media label)
    ///   [cr][f][h][m][p][t][tr][o][u][v][b][a][!][more info]
    ///   ```
    /// - The date '19XX' is changed into '19xx'
    /// - Uppercased development tags are lowercased
    pub fn into_strict(mut self) -> Self {
        if !self.0.iter().any(|e| match e { TOSECToken::Date(_, _, _) => true, _ => false })
        {
            self.0.push(TOSECToken::Date("19xx", None, None));
        }

        if !self.0.iter().any(|e| match e { TOSECToken::Publisher(_) => true, _ => false })
        {
            self.0.push(TOSECToken::Publisher(None));
        }

        self.0.sort();
        TOSECName(self.0.into_iter()
            .filter_map(|t| match t {
                TOSECToken::Publisher(Some(mut publishers)) => {
                    publishers.sort();
                    Some(TOSECToken::Publisher(Some(publishers)))
                }
                TOSECToken::Date("19XX", m, d) =>
                    Some(TOSECToken::Date("19xx", m, d)),
                TOSECToken::Development("Alpha") => Some(TOSECToken::Development("alpha")),
                TOSECToken::Development("Beta") => Some(TOSECToken::Development("beta")),
                TOSECToken::Development("Preview") => Some(TOSECToken::Development("preview")),
                TOSECToken::Development("Pre-Release") => Some(TOSECToken::Development("pre-release")),
                TOSECToken::Development("Proto")
                    |  TOSECToken::Development("Prototype") => Some(TOSECToken::Development("proto")),
                TOSECToken::Region(_, regions) => {
                    // Convert GoodTools region into TOSEC regions.
                    // The lifetime of the region strings change from 'a to 'static.

                    let region_str = regions
                        .iter()
                        .map(|r| r.into())
                        .collect::<Vec<&str>>();
                    Some(TOSECToken::Region(region_str, regions))
                }
                TOSECToken::Warning(_) => None,
                _ => Some(t)
            })
            .collect())
    }

    pub fn has_warnings(&self) -> bool {
        self.0.iter().any(|e| e.is_warning())
    }

    pub fn warnings(&self) -> impl Iterator<Item=&TOSECToken> + '_
    {
        self.0.iter().filter(|e| match e { TOSECToken::Warning(_) => true, _ => false })
    }

    pub fn try_parse<S: AsRef<str> + ?Sized>(input: &S) -> Result<TOSECName>
    {
        let (_, value) = do_parse(input.as_ref()).map_err(|_| {
            NameError::ParseError(NamingConvention::TOSEC, input.as_ref().to_string())
        })?;
        Ok(value.into())
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
/// A TOSEC format file name representing a Multi Image Set
pub struct TOSECMultiSetName<'a>
{
    tokens: Vec<Vec<TOSECToken<'a>>>,
    globals: Vec<TOSECToken<'a>>
}

impl TOSECMultiSetName<'_> {
    pub fn try_parse<S: AsRef<str> + ?Sized>(input: &S) -> Result<TOSECMultiSetName>
    {
        let (_, value) = do_parse_multiset(input.as_ref())
            .map_err(|_| {
                NameError::ParseError(NamingConvention::TOSEC, input.as_ref().to_string())
        })?;
        Ok(value.into())
    }
}
impl <'a> From<(Vec<Vec<TOSECToken<'a>>>, Vec<TOSECToken<'a>>)> for TOSECMultiSetName<'a>
{
    fn from(vecs: (Vec<Vec<TOSECToken<'a>>>, Vec<TOSECToken<'a>>)) -> Self {
        TOSECMultiSetName {
            tokens: vecs.0,
            globals: vecs.1,
        }
    }
}

impl <'a> TOSECMultiSetName<'a> {

    fn get_combined_iter(&self, index: usize) -> Option<impl Iterator<Item=&TOSECToken<'a>>>
    {
        // todo: ensure the order of global flags.
        self.tokens.get(index)
            .map(|tokens| {
                tokens.iter().chain(self.globals.iter())
            })
    }

    pub fn get_single(&self, index: usize) -> Option<TOSECName<'a>>
    {
        self.get_combined_iter(index)
            .map(|i| i.cloned().collect::<Vec<TOSECToken<'a>>>().into())
    }
}
