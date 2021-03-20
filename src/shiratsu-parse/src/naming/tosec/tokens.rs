use crate::region::Region;
use crate::naming::{FlagType, ToNameInfo, NameInfo, DevelopmentStatus, NamingConvention};
use crate::naming::util::*;

#[derive(Debug, Eq, Clone, PartialEq)]
pub enum TOSECToken<'a>
{
    Title(&'a str),
    /// A list of parsed regions.
    Region(Vec<&'a str>, Vec<Region>),

    Publisher(Option<Vec<&'a str>>),

    Demo(Option<&'a str>),

    Version((&'a str, &'a str, Option<&'a str>, Option<&'a str>, Option<Vec<&'a str>>)),

    Date(&'a str, Option<&'a str>, Option<&'a str>),

    Video(&'a str),
    Copyright(&'a str),
    Development(&'a str),
    DumpInfo(&'a str, Option<&'a str>, Option<&'a str>),

    /// Media parts

    Media(Vec<(&'a str, &'a str, Option<&'a str>)>),

    /// A vector of language tuples (Code, Variant).
    Languages(TOSECLanguage<'a>),

    /// An unspecified regular flag
    Flag(FlagType, &'a str),

    /// A warning occurred
    Warning(TOSECWarn<'a>)
}

#[derive(Debug, Clone, Eq, PartialEq)]
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

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum TOSECLanguage<'a>
{
    /// A single language code
    Single(&'a str),

    /// A double language
    Double(&'a str, &'a str),
    /// A multi-language indicator without the leading 'M'
    Count(&'a str),
}

#[derive(Debug, Clone, Eq, PartialEq)]
#[repr(transparent)]
pub struct TOSECName<'a>(Vec<TOSECToken<'a>>);

impl <'a> From<Vec<TOSECToken<'a>>> for TOSECName<'a>
{
    fn from(vec: Vec<TOSECToken<'a>>) -> Self {
        TOSECName(vec)
    }
}

impl TOSECName<'_> {
    pub fn has_warnings(&self) -> bool {
        self.0.iter().any(|e| match e { TOSECToken::Warning(_) => true, _ => false })
    }

    pub fn warnings(&self) -> impl Iterator<Item=&TOSECToken> + '_
    {
        self.0.iter().filter(|e| match e { TOSECToken::Warning(_) => true, _ => false })
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct TOSECMultiSetName<'a>
{
    tokens: Vec<Vec<TOSECToken<'a>>>,
    globals: Vec<TOSECToken<'a>>
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

impl <'a> ToNameInfo for TOSECName<'a>
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
            naming_convention: NamingConvention::TOSEC,
        };

        for token in self.0.iter()
        {
            match token {
                TOSECToken::Title(title) => {
                    name.entry_title = title.to_string()
                }
                TOSECToken::Region(_, regions) => {
                    name.region = regions.clone()
                }
                TOSECToken::Media(parts) => {
                    if let Some(parts) = parts.first()
                    {
                        if parts.0 != "Side" {
                            name.part_number = parts.1.parse::<i32>().ok()
                        } else {
                            // Match Side A and B
                            match parts.1 {
                                "A" => name.part_number = Some(1),
                                "B" => name.part_number = Some(2),
                                _ => {}
                            }
                        }
                    }
                }
                TOSECToken::Version(version) => {
                    match version {
                        (_, major, None, _, _) => { name.version = Some(major.to_string()) }
                        (_, major, Some(minor), _, _) => { name.version = Some(format!("{}.{}", major, minor)) }
                    }
                }
                TOSECToken::DumpInfo("p", _, _) => {
                    name.is_unlicensed = true
                }
                TOSECToken::Demo(_) => {
                    name.is_demo = true
                }
                TOSECToken::Development("proto")
                | TOSECToken::Development("Proto")
                | TOSECToken::Development("Prototype") => {
                    name.status = DevelopmentStatus::Prototype
                }
                TOSECToken::Development(_) => {
                    name.status = DevelopmentStatus::Prerelease
                }
                _ => {}

            }
        }

        if name.entry_title.ends_with("BIOS")
            || name.entry_title.ends_with("System Software")
        {
            name.is_system = true;
        }

        let mut release_title = name.entry_title.clone();

        move_default_articles_mut(&mut release_title);
        replace_hyphen_mut(&mut release_title);
        name.release_title = release_title;
        name
    }
}

