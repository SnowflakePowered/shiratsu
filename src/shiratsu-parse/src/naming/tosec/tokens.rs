use crate::region::Region;
use crate::naming::{FlagType, ToNameInfo, NameInfo, DevelopmentStatus, NamingConvention};
use crate::naming::util::*;

#[derive(Debug, Eq, PartialEq)]
pub enum TOSECToken<'a>
{
    Title(&'a str),
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

    /// A warning occurred
    Warning(TOSECParseWarning<'a>)
}

#[derive(Debug, Eq, PartialEq)]
pub enum TOSECParseWarning<'a>
{
    ZZZUnknown,
    MalformedDatePlaceholder(&'a str),
    MissingDate,
    MissingSpace,
    UnexpectedSpace,
    NotEof(&'a str)
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

#[derive(Debug, Eq, PartialEq)]
#[repr(transparent)]
pub struct TOSECName<'a>(Vec<TOSECToken<'a>>);

impl <'a> From<Vec<TOSECToken<'a>>> for TOSECName<'a>
{
    fn from(vec: Vec<TOSECToken<'a>>) -> Self {
        TOSECName(vec)
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
                TOSECToken::Region(regions) => {
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
                TOSECToken::Flag(_, "proto") => {
                    name.status = DevelopmentStatus::Prototype
                }
                TOSECToken::Flag(_, "alpha")
                | TOSECToken::Flag(_, "beta")
                | TOSECToken::Flag(_, "preview")
                | TOSECToken::Flag(_, "pre-release") => {
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

