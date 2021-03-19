use crate::region::Region;
use crate::naming::{FlagType, NameInfo, ToNameInfo, DevelopmentStatus, NamingConvention};
use crate::naming::util::*;

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
    Title(&'a str),

    /// A list of parsed regions.
    Region(Vec<&'a str>, Vec<Region>),

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

#[derive(Debug, Eq, PartialEq)]
#[repr(transparent)]
pub struct NoIntroName<'a>(Vec<NoIntroToken<'a>>);

impl <'a> From<Vec<NoIntroToken<'a>>> for NoIntroName<'a>
{
    fn from(vec: Vec<NoIntroToken<'a>>) -> Self {
        NoIntroName(vec)
    }
}

impl <'a> ToNameInfo for NoIntroName<'a>
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

        for token in self.0.iter()
        {
            match &token {
                NoIntroToken::Title(title) => {
                    name.entry_title = title.to_string()
                }
                NoIntroToken::Flag(_, "Kiosk")
                | NoIntroToken::Flag(_, "Demo")
                | NoIntroToken::Flag(_, "Sample")
                | NoIntroToken::Flag(_, "Bonus Disc")
                | NoIntroToken::Flag(_, "Bonus CD")
                | NoIntroToken::Flag(_, "Taikenban")
                | NoIntroToken::Flag(_, "Tentou Taikenban") => {
                    name.is_demo = true
                }
                NoIntroToken::Beta(_) => { name.status = DevelopmentStatus::Prerelease }
                NoIntroToken::Flag(_, "Proto") => { name.status = DevelopmentStatus::Prototype }
                NoIntroToken::Flag(_, "Unl") => { name.is_unlicensed = true }
                NoIntroToken::Version(versions) => {
                    match versions.first() {
                        Some((_, major, None, _, _)) => { name.version = Some(major.to_string()) }
                        Some((_, major, Some(minor), _, _)) => { name.version = Some(format!("{}.{}", major, minor)) }
                        _ => {}
                    }
                }
                NoIntroToken::Part(_, part) => { name.part_number = part.parse::<i32>().ok() }
                NoIntroToken::Region(_, region) => { name.region = region.clone() }
                NoIntroToken::Flag(_, "BIOS") => { name.is_system = true }
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
