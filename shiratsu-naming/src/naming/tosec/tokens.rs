use crate::naming::{FlagType, NamingConvention, TokenizedName};
use crate::region::Region;

use crate::naming::common::error::{NameError, Result};
use crate::naming::tosec::parsers::{do_parse, do_parse_multiset};

use std::cmp::Ordering;
use std::fmt;
use std::fmt::{Display, Formatter};
use std::slice::Iter;

/// A token constituent within a `TOSECToken`.
///
/// Tokens are not guaranteed to have consistent semantics
/// outside of a `NoIntroName`. The order of tokens in a
/// `TOSECName` is significant in order of appearance in
/// the input file name.
///
/// `TOSECToken` has a custom implementation of `PartialOrd` following the TOSEC Naming Convention.
#[derive(Debug, Eq, Clone, Ord)]
pub enum TOSECToken<'a> {
    /// The title of the ROM.
    Title(&'a str),

    /// A version flag.
    ///
    Version {
        /// The version type. If this is `Rev`, a space occurs between it and the major version.
        version_type: &'a str,
        /// The major version.
        major: &'a str,
        /// The minor version, if any.
        minor: Option<&'a str>,
    },

    /// A demo flag, preceding the `demo-` string.
    ///
    /// ## Examples
    /// * `(demo)` parses as `Demo(None)`.
    /// * `(demo-kiosk)` parses as `Demo(Some("kiosk"))`
    Demo(Option<&'a str>),

    /// A date flag
    ///
    Date {
        /// The year.
        year: &'a str,
        /// The month, if any.
        month: Option<&'a str>,
        /// The day, if any.
        day: Option<&'a str>,
    },

    /// A publisher flag, with publishers separated by ` - ` if more than one.
    ///
    /// ## Examples
    /// * `(-)` parses as `Publisher(None)`
    /// * `(Publisher A - Doe, John)` parses as `Publisher(Some(vec!["Publisher A", "Doe, John"]))`
    Publisher(Option<Vec<&'a str>>),

    /// A system flag.
    ///
    /// See the [TOSEC Naming Convention](https://www.tosecdev.org/tosec-naming-convention#_Toc302254951)
    /// for a list of valid system flags.
    System(&'a str),

    /// A video flag.
    ///
    /// See the [TOSEC Naming Convention](https://www.tosecdev.org/tosec-naming-convention#_Toc302254954)
    /// for a list of valid video flags.
    Video(&'a str),

    /// The region of the ROM.
    ///
    /// Because of the existence of GoodTools region codes in `ZZZ-UNK-` ROMs,
    /// you may not assume a one-to-one correspondence between the strings in the
    /// region flag, and the list of parsed regions, since GoodTools regions may
    /// go through expansion.
    ///
    Region {
        /// The region strings that correspond to the parsed regions.
        region_strings: Vec<&'a str>,
        /// The parsed regions.
        regions: Vec<Region>,
    },

    /// A language flag.
    ///
    /// See the `TOSECLanguage` documentation for more details.
    Languages(TOSECLanguage<'a>),

    /// A copyright status flag.
    ///
    /// See the [TOSEC Naming Convention](https://www.tosecdev.org/tosec-naming-convention#_Toc302254964)
    /// for a list of valid copyright status flags.
    Copyright(&'a str),

    /// A development status flag.
    ///
    /// See the [TOSEC Naming Convention](https://www.tosecdev.org/tosec-naming-convention#_Toc302254967)
    /// for a list of valid development status flags.
    Development(&'a str),
    /// A dump info flag.
    ///
    /// '`[more info]`' flags are parsed as
    /// `Flag { flag_type: FlagType::Bracketed, flag: &'a str }`, and not `DumpInfo`.
    ///
    /// See the [TOSEC Naming Convention](https://www.tosecdev.org/tosec-naming-convention#_Toc302254975)
    /// for a list of valid dump info flags.
    ///
    /// ## Example
    /// * `[!]` parses as `DumpInfo { code: "!", number: None, info: None }`
    /// * `[f1 Fix Fixer]` parses as `DumpInfo { code: "f", number: Some("1"), info: Some("Fix Fixer") }`
    DumpInfo {
        /// The letter or name of the dump flag.
        code: &'a str,
        /// The number of the dump flag, if any.
        number: Option<&'a str>,
        /// The arguments or additional information of the dump flag, if any.
        /// This is an opaque string, and is not specialized by dump flag type.
        info: Option<&'a str>,
    },

    /// A media part number flag.
    ///
    /// There may be multiple media parts in a flag, separated by a space.
    ///
    /// Media type flags are parsed as
    /// `Flag { flag_type: FlagType::Parenthesized, flag: &'a str }`, and not `Media`.
    ///
    /// See [`TOSECMedia`] for the structure of each parsed media part.
    ///
    /// ## Examples
    /// * `(Side A)` parses to `Media(vec![TOSECMedia { media_type: "Side", number: "A", total: None }])`.
    /// * `(Disc 1 of 2 Side B)` parses to two `TOSECMedia` values.
    Media(Vec<TOSECMedia<'a>>),

    /// A generic, non-defined, or unknown flag.
    Flag {
        /// The flag's delimiter type.
        flag_type: FlagType,
        /// The text inside the flag's delimiters.
        flag: &'a str,
    },

    /// Indicates an unexpected deviations from the
    /// TOSEC Naming convention.
    ///
    /// Warnings may be lexical (appearing in the input string) or non-lexical,
    /// depending on the warning.
    ///
    /// Warnings are always associated with the token following
    /// the warning in the resulting `TOSECName` token stream.
    ///
    /// This also means that lexical warnings occur in the order of appearance
    /// in the input string.
    Warning(TOSECWarn<'a>),
}

/// A media part parsed from a TOSEC media flag.
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct TOSECMedia<'a> {
    /// The media part type, such as `Disc`, `Part`, or `Side`.
    pub media_type: &'a str,
    /// The media part number or label.
    pub number: &'a str,
    /// The total number of media parts, if specified.
    pub total: Option<&'a str>,
}

impl PartialEq for TOSECToken<'_> {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (TOSECToken::Title(t), TOSECToken::Title(o)) => t.eq(o),
            (TOSECToken::Version { version_type: a, major: b, minor: c },
                TOSECToken::Version { version_type: e, major: f, minor: g })
            => (a, b, c).eq(&(e, f, g)),
            (TOSECToken::Demo(t), TOSECToken::Demo(o))
            => t.eq(o),
            (TOSECToken::Date { year: y, month: m, day: d },
                TOSECToken::Date { year: y2, month: m2, day: d2 })
            => (y, m, d).eq(&(y2, m2, d2)),
            (TOSECToken::Publisher(a), TOSECToken::Publisher(b))
            => a.eq(b),
            (TOSECToken::System(a), TOSECToken::System(b))
            => a.eq(b),
            (TOSECToken::Video(a), TOSECToken::Video(b))
            => a.eq(b),
            (TOSECToken::Region { region_strings: a, regions: _ }, TOSECToken::Region { region_strings: b, regions: _ })
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
            (TOSECToken::Flag { flag_type: FlagType::Parenthesized, flag: a }, TOSECToken::Flag { flag_type: FlagType::Parenthesized, flag: b })
            => a.eq(b),
            (TOSECToken::DumpInfo { code: a, number: n, info: f },
                TOSECToken::DumpInfo { code: a1, number: n2, info: f2 }) => {
                (a, n, f).eq(&(a1, n2, f2))
            },
            (TOSECToken::Flag { flag_type: FlagType::Bracketed, flag: a }, TOSECToken::Flag { flag_type: FlagType::Bracketed, flag: b })
            => a.eq(b),
            (TOSECToken::Warning(a), TOSECToken::Warning(b)) =>
                a.eq(b),
            _ => false
        }
    }
}

impl PartialOrd for TOSECToken<'_> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        fn get_priority(token: &TOSECToken) -> usize {
            match token {
                TOSECToken::Title(_) => 0,
                TOSECToken::Version {
                    version_type: _,
                    major: _,
                    minor: _,
                } => 1,
                TOSECToken::Demo(_) => 2,
                TOSECToken::Date {
                    year: _,
                    month: _,
                    day: _,
                } => 3,
                TOSECToken::Publisher(_) => 4,
                TOSECToken::System(_) => 5,
                TOSECToken::Video(_) => 6,
                TOSECToken::Region {
                    region_strings: _,
                    regions: _,
                } => 7,
                TOSECToken::Languages(_) => 8,
                TOSECToken::Copyright(_) => 9,
                TOSECToken::Development(_) => 10,
                TOSECToken::Media(_) => 11,
                // Presumably media type
                TOSECToken::Flag {
                    flag_type: FlagType::Parenthesized,
                    flag: _,
                } => 12,
                TOSECToken::DumpInfo {
                    code: _,
                    number: _,
                    info: _,
                } => 13,
                TOSECToken::Flag {
                    flag_type: FlagType::Bracketed,
                    flag: _,
                } => 14,
                TOSECToken::Warning(_) => usize::MAX,
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
                _ => usize::MAX,
            }
        }

        let self_priority = get_priority(self);
        let other_priority = get_priority(other);
        if self_priority != other_priority {
            return self_priority.partial_cmp(&other_priority);
        }
        match (self, other) {
            (TOSECToken::Title(t), TOSECToken::Title(o)) => t.partial_cmp(o),
            (
                TOSECToken::Version {
                    version_type: a,
                    major: b,
                    minor: c,
                },
                TOSECToken::Version {
                    version_type: e,
                    major: f,
                    minor: g,
                },
            ) => (a, b, c).partial_cmp(&(e, f, g)),
            (TOSECToken::Demo(t), TOSECToken::Demo(o)) => t.partial_cmp(o),
            (
                TOSECToken::Date {
                    year: y,
                    month: m,
                    day: d,
                },
                TOSECToken::Date {
                    year: y2,
                    month: m2,
                    day: d2,
                },
            ) => (y, m, d).partial_cmp(&(y2, m2, d2)),
            (TOSECToken::Publisher(a), TOSECToken::Publisher(b)) => a.partial_cmp(b),
            (TOSECToken::System(a), TOSECToken::System(b)) => a.partial_cmp(b),
            (TOSECToken::Video(a), TOSECToken::Video(b)) => a.partial_cmp(b),
            (
                TOSECToken::Region {
                    region_strings: a,
                    regions: _,
                },
                TOSECToken::Region {
                    region_strings: b,
                    regions: _,
                },
            ) => a.partial_cmp(b),
            (TOSECToken::Languages(a), TOSECToken::Languages(b)) => a.partial_cmp(b),
            (TOSECToken::Copyright(a), TOSECToken::Copyright(b)) => a.partial_cmp(b),
            (TOSECToken::Development(a), TOSECToken::Development(b)) => a.partial_cmp(b),
            (TOSECToken::Media(a), TOSECToken::Media(b)) => a.partial_cmp(b),
            // Presumably media type
            (
                TOSECToken::Flag {
                    flag_type: FlagType::Parenthesized,
                    flag: a,
                },
                TOSECToken::Flag {
                    flag_type: FlagType::Parenthesized,
                    flag: b,
                },
            ) => a.partial_cmp(b),
            (
                TOSECToken::DumpInfo {
                    code: a,
                    number: n,
                    info: f,
                },
                TOSECToken::DumpInfo {
                    code: a1,
                    number: n2,
                    info: f2,
                },
            ) => {
                if a == a1 {
                    if n != n2 {
                        n.partial_cmp(n2)
                    } else {
                        f.partial_cmp(f2)
                    }
                } else {
                    get_dumpinfo_priority(a).partial_cmp(&get_dumpinfo_priority(a1))
                }
            }
            (
                TOSECToken::Flag {
                    flag_type: FlagType::Bracketed,
                    flag: a,
                },
                TOSECToken::Flag {
                    flag_type: FlagType::Bracketed,
                    flag: b,
                },
            ) => a.partial_cmp(b),
            (TOSECToken::Warning(a), TOSECToken::Warning(b)) => a.partial_cmp(b),
            _ => unreachable!(),
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq, PartialOrd, Ord)]
/// A token that represents a warning or inconsistency in a TOSEC Naming Convention
/// file name.
///
/// Warnings may be lexical (and are emitted in the input stream), or
/// non-lexical, which serves as a warning.
///
/// Warnings may modify how a token is re-serialized depending on the order
/// they appear in the `TOSECName`. A warning always occurs before the associated token.
pub enum TOSECWarn<'a> {
    /// This file name starts with `ZZZ-UNK-`.
    ///
    /// This warning is lexical and will emit the string `ZZZ-UNK-`
    ZZZUnknown,

    /// The date placeholder in the following date token is
    /// malformed, often because it is upper cased.
    ///
    /// A date such as `19XX` will produce this warning.
    ///
    /// If multiple date segments are malformed, multiple warnings will
    /// be emitted in the order year, month, date.
    ///
    /// This warning is non-lexical.
    MalformedDatePlaceholder(&'a str),

    /// The development status flag in the following development
    /// status token is malformed, often because it is upper cased.
    ///
    /// A development status such as `Beta` will produce this warning.
    ///
    /// This warning is non-lexical.
    MalformedDevelopmentStatus(&'a str),

    /// The following date token is undelimited with hyphens.
    ///
    /// A date such as `20001231` will produce this warning.
    ///
    /// This warning is lexical and will modify the re-serialization
    /// of the following date token. This warning always occurs after all
    /// `MalformedDatePlaceholder` warnings.
    UndelimitedDate(&'a str),

    /// The required date token is missing.
    ///
    /// This warning is non-lexical.
    MissingDate,

    /// The required publisher token is missing.
    ///
    /// This warning is non-lexical.
    MissingPublisher,

    /// A space was expected between the preceding and following
    /// token of this warning.
    ///
    /// This warning is lexical and will ensure that there are
    /// no spaces between the preceding and following non-warning token.
    MissingSpace,

    /// A space occurred between the preceding and following token
    /// of this warning.
    ///
    /// This warning is lexical and will ensure that a space occurs
    /// between the preceding and following non-warning token.
    UnexpectedSpace,

    /// The following publisher token is preceded by the string 'by'.
    ///
    /// This may occur in `ZZZ-UNK-` names such as
    /// `ZZZ-UNK-Micro Font Dumper by Schick, Bastian`.
    ///
    /// This warning is lexical and will emit the string `by `.
    ByPublisher,

    /// The publisher flag occurred before the date flag.
    ///
    /// This often occurs if a `ByPublisher` warning is emitted.
    /// If so, this token is always emitted before `ByPublisher` is emitted.
    ///
    /// This warning is non-lexical.
    PublisherBeforeDate,

    /// A GoodTool region code occurred rather than an ISO region.
    ///
    /// This warning is non-lexical, however the following region token
    /// will contain the GoodTools region string, and not a valid TOSEC
    /// ISO region string.
    ///
    /// `TOSECName::into_strict()` will transform GoodTools region tokens into
    /// ISO region string tokens.
    GoodToolsRegionCode(&'a str),

    /// A version occurred wrap in parentheses as a flag.
    ///
    /// This warning is lexical and will cause the following version token
    /// to be emitted within parentheses.
    VersionInFlag,

    /// When parsing a TOSEC name, the name is expected to be complete.
    ///
    /// If not, then this warning will be emitted containing the remainder of the string.
    /// This often occurs because of unbalanced parentheses or braces.
    ///
    /// If this warning is emitted, it should be the last token in the token string, and
    /// thus have no associated token.
    ///
    /// This warning is lexical and will emit the remainder string segment.
    /// This can be removed with `TOSECName::without_trailing`.
    NotEof(&'a str),
}

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
/// A language flag parsed from a TOSEC Naming Convention file name.
pub enum TOSECLanguage<'a> {
    /// A flag with a single language.
    Single(&'a str),
    /// A flag with two languages, in the order they appear, separated by a hyphen (`-`).
    ///
    /// ## Tuple elements
    /// 0. The first language.
    /// 1. The second language.
    Double(&'a str, &'a str),
    /// A multi-language indicator, following the character `M`.
    ///
    /// ## Examples
    /// * `(M6)` parses to `Count("6")`.
    Count(&'a str),
}

#[derive(Debug, Clone, Eq, PartialEq)]
#[repr(transparent)]
/// A TOSEC format file name.
///
/// The order of tokens in a
/// `TOSECName` is significant in order of appearance in the input file name.
///
/// They are also not guaranteed to be strictly conforming to the
/// TOSEC naming convention, but can be made so
/// using `TOSECName::into_strict()`.
pub struct TOSECName<'a>(Vec<TOSECToken<'a>>);

impl<'a> From<Vec<TOSECToken<'a>>> for TOSECName<'a> {
    fn from(vec: Vec<TOSECToken<'a>>) -> Self {
        TOSECName(vec)
    }
}

impl TOSECName<'_> {
    /// Removes any trailing unparsed string segments from the name.
    pub fn without_trailing(mut self) -> Self {
        self.0
            .retain(|t| !matches!(t, TOSECToken::Warning(TOSECWarn::NotEof(_))));
        self
    }

    /// Makes the name conform strictly to the TOSEC naming conventions.
    ///
    /// This removes any warning tokens and ensures the order of flags is proper.
    ///
    /// # Fixes
    /// - If there is no date, 19xx is added as the date.
    /// - If there is no publisher, the unknown publisher (-) is added.
    /// - GoodTools region codes are converted into the ISO equivalent.
    /// - Publishers are sorted lexicographically.
    /// - Tags are put in the order
    ///    ```order
    ///   Title version (demo) (date)(publisher)(system)(video)(country)(language)
    ///   (copyright status)(development status)(media type)(media label)
    ///   [cr][f][h][m][p][t][tr][o][u][v][b][a][!][more info]
    ///   ```
    /// - The date '19XX' is changed into '19xx'.
    /// - Uppercased development tags are lowercased.
    ///
    /// # Zero-copy guarantee
    ///
    /// This method consumes the tokenized name and remains zero-copy.
    /// The zero-copy nature of the parsed tokens mean that some fixes can not be done, such
    /// as reorganizing individual publisher names into `Surname, Given Name` format.
    ///
    /// As a result, the strict name may not always conform to the strictest reading of the
    /// TOSEC naming convention, especially with regards to alphabetization or malformed
    /// flags that were not explicitly specified in listed fixes.
    ///
    /// The fixes that are done are only possible by converting the tokenized `&'a str` into
    /// a known `&'static str`.
    pub fn into_strict(mut self) -> Self {
        if !self.0.iter().any(|e| {
            matches!(
                e,
                TOSECToken::Date {
                    year: _,
                    month: _,
                    day: _
                }
            )
        }) {
            self.0.push(TOSECToken::Date {
                year: "19xx",
                month: None,
                day: None,
            });
        }

        if !self.0.iter().any(|e| matches!(e, TOSECToken::Publisher(_))) {
            self.0.push(TOSECToken::Publisher(None));
        }

        self.0.sort();
        TOSECName(
            self.0
                .into_iter()
                .filter_map(|t| match t {
                    TOSECToken::Publisher(Some(mut publishers)) => {
                        publishers.sort();
                        Some(TOSECToken::Publisher(Some(publishers)))
                    }
                    TOSECToken::Date {
                        year: "19XX",
                        month: m,
                        day: d,
                    } => Some(TOSECToken::Date {
                        year: "19xx",
                        month: m,
                        day: d,
                    }),
                    TOSECToken::Development("Alpha") => Some(TOSECToken::Development("alpha")),
                    TOSECToken::Development("Beta") => Some(TOSECToken::Development("beta")),
                    TOSECToken::Development("Preview") => Some(TOSECToken::Development("preview")),
                    TOSECToken::Development("Pre-Release") => {
                        Some(TOSECToken::Development("pre-release"))
                    }
                    TOSECToken::Development("Proto") | TOSECToken::Development("Prototype") => {
                        Some(TOSECToken::Development("proto"))
                    }
                    TOSECToken::Region {
                        region_strings: _,
                        regions,
                    } => {
                        // Convert GoodTools region into TOSEC regions.
                        // The lifetime of the region strings change from 'a to 'static.

                        let region_str = regions.iter().map(|r| r.into()).collect::<Vec<&str>>();
                        Some(TOSECToken::Region {
                            region_strings: region_str,
                            regions,
                        })
                    }
                    TOSECToken::Warning(_) => None,
                    _ => Some(t),
                })
                .collect(),
        )
    }
}

impl<'a> TokenizedName<'a, TOSECToken<'a>> for TOSECName<'a> {
    fn title(&self) -> Option<&'a str> {
        self.iter().find_map(|f| match f {
            TOSECToken::Title(t) => Some(*t),
            _ => None,
        })
    }

    fn iter(&self) -> Iter<'_, TOSECToken<'a>> {
        self.0.iter()
    }

    fn try_parse<S: AsRef<str> + ?Sized>(input: &'a S) -> Result<Self> {
        let (_, value) = do_parse(input.as_ref()).map_err(|_| {
            NameError::ParseError(NamingConvention::TOSEC, input.as_ref().to_string())
        })?;
        Ok(value.into())
    }

    fn naming_convention() -> NamingConvention {
        NamingConvention::TOSEC
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
/// A TOSEC format file name representing a multi-image set.
pub struct TOSECMultiSetName<'a> {
    tokens: Vec<Vec<TOSECToken<'a>>>,
    globals: Vec<TOSECToken<'a>>,
}

impl TOSECMultiSetName<'_> {
    /// Tries to parse a multi-image set name with the TOSEC naming convention.
    pub fn try_parse<S: AsRef<str> + ?Sized>(input: &S) -> Result<TOSECMultiSetName> {
        let (_, value) = do_parse_multiset(input.as_ref()).map_err(|_| {
            NameError::ParseError(NamingConvention::TOSEC, input.as_ref().to_string())
        })?;
        Ok(value.into())
    }

    fn get_combined_iter(&self, index: usize) -> Option<impl Iterator<Item = &TOSECToken<'_>>> {
        // todo: ensure the order of global flags.
        self.tokens
            .get(index)
            .map(|tokens| tokens.iter().chain(self.globals.iter()))
    }

    /// Gets a `TOSECName` from a multi-set name.
    ///
    /// This method will clone tokens and string slices
    /// to include global flags, but does not clone the
    /// underlying string segments.
    pub fn get_single(&self, index: usize) -> Option<TOSECName> {
        self.get_combined_iter(index)
            .map(|i| i.cloned().collect::<Vec<TOSECToken>>().into())
            .map(|i: TOSECName| i.into_strict())
    }
}
impl<'a> From<(Vec<Vec<TOSECToken<'a>>>, Vec<TOSECToken<'a>>)> for TOSECMultiSetName<'a> {
    fn from(vecs: (Vec<Vec<TOSECToken<'a>>>, Vec<TOSECToken<'a>>)) -> Self {
        TOSECMultiSetName {
            tokens: vecs.0,
            globals: vecs.1,
        }
    }
}

fn write_tosec_string(buf: &mut String, tokens: &[TOSECToken<'_>]) {
    for (i, token) in tokens.iter().enumerate() {
        match token {
            TOSECToken::Title(t) => {
                buf.push_str(t);
                buf.push(' ');
            }
            TOSECToken::Version {
                version_type: tag,
                major: maj,
                minor: min,
            } => {
                if let Some(TOSECToken::Warning(TOSECWarn::VersionInFlag)) = tokens.get(i - 1) {
                    buf.push('(');
                }
                buf.push_str(tag);
                if tag == &"Rev" {
                    buf.push(' ');
                }
                buf.push_str(maj);
                if let Some(min) = min {
                    buf.push('.');
                    buf.push_str(min);
                }
                if let Some(TOSECToken::Warning(TOSECWarn::VersionInFlag)) = tokens.get(i - 1) {
                    buf.push(')');
                }
                buf.push(' ');
            }
            TOSECToken::Demo(ty) => {
                buf.push_str("(demo");
                if let Some(ty) = ty {
                    buf.push('-');
                    buf.push_str(ty);
                }
                buf.push_str(") ");
            }
            TOSECToken::Date {
                year: y,
                month: m,
                day: d,
            } => {
                buf.push('(');
                if let Some(TOSECToken::Warning(TOSECWarn::UndelimitedDate(s))) = tokens.get(i - 1)
                {
                    buf.push_str(s);
                } else {
                    buf.push_str(y);
                    if let Some(m) = m {
                        buf.push('-');
                        buf.push_str(m);
                    }
                    if let Some(d) = d {
                        buf.push('-');
                        buf.push_str(d);
                    }
                }
                buf.push(')');
            }
            TOSECToken::Publisher(pubs) => {
                let emit_params = match tokens.get(i - 1) {
                    Some(TOSECToken::Warning(TOSECWarn::ByPublisher)) => {
                        buf.push_str("by ");
                        false
                    }
                    _ => true,
                };
                if emit_params {
                    buf.push('(');
                }
                if let Some(pubs) = pubs {
                    for pubs in pubs.iter() {
                        buf.push_str(pubs);
                        buf.push_str(" - ");
                    }

                    // trim the end
                    if buf.ends_with(" - ") {
                        buf.truncate(buf.len() - " - ".len());
                    }
                } else {
                    buf.push('-');
                }
                if emit_params {
                    buf.push(')');
                } else {
                    buf.push(' ')
                }
            }
            TOSECToken::System(s) => {
                buf.push('(');
                buf.push_str(s);
                buf.push(')');
            }
            TOSECToken::Video(v) => {
                buf.push('(');
                buf.push_str(v);
                buf.push(')');
            }
            TOSECToken::Region {
                region_strings: rs,
                regions: _,
            } => {
                buf.push('(');
                for region in rs.iter() {
                    buf.push_str(region);
                    buf.push('-');
                }

                if buf.ends_with('-') {
                    buf.truncate(buf.len() - "-".len());
                }

                buf.push(')');
            }
            TOSECToken::Languages(l) => {
                buf.push('(');
                match l {
                    TOSECLanguage::Single(s) => {
                        buf.push_str(s);
                    }
                    TOSECLanguage::Double(a, b) => {
                        buf.push_str(a);
                        buf.push('-');
                        buf.push_str(b);
                    }
                    TOSECLanguage::Count(c) => {
                        buf.push_str(c);
                    }
                }
                buf.push(')');
            }
            TOSECToken::Copyright(c) => {
                buf.push('(');
                buf.push_str(c);
                buf.push(')');
            }
            TOSECToken::Development(de) => {
                buf.push('(');
                buf.push_str(de);
                buf.push(')');
            }
            TOSECToken::DumpInfo {
                code,
                number: num,
                info,
            } => {
                buf.push('[');
                buf.push_str(code);

                if let Some(num) = num {
                    buf.push_str(num);
                }

                if let Some(info) = info {
                    buf.push(' ');
                    buf.push_str(info);
                }
                buf.push(']');
            }
            TOSECToken::Media(m) => {
                buf.push('(');
                for media in m {
                    buf.push_str(media.media_type);
                    buf.push(' ');
                    buf.push_str(media.number);
                    if let Some(total) = media.total {
                        buf.push_str(" of ");
                        buf.push_str(total);
                    }

                    buf.push(' ');
                }

                if buf.ends_with(' ') {
                    buf.truncate(buf.len() - " ".len());
                }

                buf.push(')');
            }
            TOSECToken::Flag {
                flag_type: FlagType::Parenthesized,
                flag: f,
            } => {
                buf.push('(');
                buf.push_str(f);
                buf.push(')');
            }
            TOSECToken::Flag {
                flag_type: FlagType::Bracketed,
                flag: f,
            } => {
                buf.push('[');
                buf.push_str(f);
                buf.push(']');
            }
            TOSECToken::Warning(w) => match w {
                TOSECWarn::ZZZUnknown => {
                    buf.push_str("ZZZ-UNK-");
                }
                TOSECWarn::MissingSpace => {
                    if buf.ends_with(' ') {
                        buf.truncate(buf.len() - " ".len())
                    }
                }
                TOSECWarn::UnexpectedSpace => {
                    buf.push(' ');
                }
                TOSECWarn::NotEof(remainder) => {
                    buf.push_str(remainder);
                }
                _ => {}
            },
        }
    }
}

impl Display for TOSECName<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let mut buf = String::new();
        write_tosec_string(&mut buf, &self.0);
        f.write_str(buf.trim())
    }
}

impl Display for TOSECMultiSetName<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let mut buf = String::new();
        for title in &self.tokens {
            write_tosec_string(&mut buf, title);
            buf.push_str(" & ");
        }

        if buf.ends_with(" & ") {
            buf.truncate(buf.len() - " & ".len())
        }

        if !self.globals.is_empty() {
            buf.push('-');
            write_tosec_string(&mut buf, &self.globals);
        }
        f.write_str(buf.trim())
    }
}
