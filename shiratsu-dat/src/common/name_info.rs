use shiratsu_naming::region::Region;
use crate::DevelopmentStatus;
use shiratsu_naming::naming::{NamingConvention, FlagType, TokenizedName};
use shiratsu_naming::naming::nointro::*;
use crate::common::util::{move_default_articles_mut, replace_hyphen_mut};
use shiratsu_naming::naming::goodtools::*;
use shiratsu_naming::naming::tosec::*;

#[derive(Debug, Eq, PartialEq)]
pub struct NameInfo {
    pub entry_title: String,
    pub release_title: String,
    pub region: Vec<Region>,
    pub part_number: Option<i32>,
    pub version: Option<String>,
    pub is_unlicensed: bool,
    pub is_demo: bool,
    pub is_system: bool,
    pub status: DevelopmentStatus,
    pub naming_convention: NamingConvention,
}

impl NameInfo {
    /// The region of the game.
    pub fn region(&self) -> &[Region] {
        &self.region
    }
    /// If this entry is split into multiple parts, the part number of this entry.
    pub fn part_number(&self) -> Option<i32> {
        self.part_number
    }
    /// Whether or not this game is unlicensed.
    pub fn is_unlicensed(&self) -> bool {
        self.is_unlicensed
    }
    /// Whether or not this game is a sample or a demo version of a full game.
    pub fn is_demo(&self) -> bool {
        self.is_demo
    }
    /// Whether or not this game is a system update or BIOS file.
    pub fn is_system(&self) -> bool {
        self.is_system
    }
    /// The version of the game entry.
    pub fn version(&self) -> Option<&str> {
        self.version.as_deref()
    }
    /// The name of the release, with all tags removed, and articles at the beginning of the title.
    pub fn release_title(&self) -> &str {
        &self.release_title.as_str()
    }
    /// The name of the release as it appears in the filename.
    pub fn entry_title(&self) -> &str {
        &self.entry_title.as_str()
    }
    /// The development status of the game entry.
    pub fn development_status(&self) -> DevelopmentStatus {
        self.status
    }
    /// The naming convention of the structuredly named filename.
    pub fn naming_convention(&self) -> NamingConvention {
        self.naming_convention
    }
}

pub trait ToNameInfo
{
    /// Creates a new `NameInfo` object from the name data.
    fn to_name_info(&self) -> NameInfo;
}

impl<'a, T> From<T> for NameInfo
    where T: ToNameInfo
{
    fn from(name: T) -> Self {
        name.to_name_info()
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

        for token in self.iter()
        {
            match &token {
                NoIntroToken::Title(title) => {
                    name.entry_title = title.to_string()
                }
                NoIntroToken::Flag(_, "Kiosk")
                | NoIntroToken::Flag(_, "Kiosk Demo")
                | NoIntroToken::Flag(_, "Bonus Game")
                | NoIntroToken::Flag(_, "Taikenban Sample ROM")
                | NoIntroToken::Release("Demo", _)
                | NoIntroToken::Release("Sample", _) => {
                    name.is_demo = true
                }
                NoIntroToken::Release("Beta", _) => { name.status = DevelopmentStatus::Prerelease }
                NoIntroToken::Release("Proto", _) => { name.status = DevelopmentStatus::Prototype }
                NoIntroToken::Flag(_, "Unl") => { name.is_unlicensed = true }
                NoIntroToken::Version(versions) => {
                    match versions.first() {
                        Some((_, major, None, _, _, _)) => { name.version = Some(major.to_string()) }
                        Some((_, major, Some(minor), _, _, _)) => { name.version = Some(format!("{}.{}", major, minor)) }
                        _ => {}
                    }
                }
                NoIntroToken::Media(_, part) => { name.part_number = part.parse::<i32>().ok() }
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
            naming_convention: NamingConvention::GoodTools,
        };
        for token in self.iter() {
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

        for token in self.iter()
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
                TOSECToken::Version(_, major, minor) => {
                    match minor {
                        None => { name.version = Some(major.to_string()) }
                        Some(minor) => { name.version = Some(format!("{}.{}", major, minor)) }
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

