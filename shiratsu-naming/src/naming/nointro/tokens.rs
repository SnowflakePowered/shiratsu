use crate::naming::common::error::{NameError, Result};
use crate::naming::nointro::parsers::do_parse;
use crate::naming::{FlagType, NamingConvention, TokenizedName};
use crate::region::Region;
use std::fmt;
use std::fmt::{Display, Formatter};
use std::slice::Iter;

/// A token constituent within a `NoIntroName`.
///
/// Tokens are not guaranteed to have consistent semantics
/// outside of a `NoIntroName`. The order of tokens in a
/// `NoIntroName` is significant in order of appearance in
/// the input file name.
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum NoIntroToken<'a> {
    /// The title of the ROM.
    Title(&'a str),

    /// The region of the ROM.
    ///
    /// There is no one-to-one correspondence between the strings in the
    /// region flag, and the list of parsed regions, since regions may
    /// undergo expansion.
    ///
    Region {
        /// The region strings that correspond to the parsed regions.
        region_strings: Vec<&'a str>,
        /// The parsed regions.
        regions: Vec<Region>,
    },

    /// A version flag.
    ///
    /// A version flag may contain one or more versions, separated by a separator.
    ///
    /// See [`NoIntroVersion`] for the structure of each parsed version.
    ///
    /// ## Examples
    /// * `(v1.0)` parses to `Version(vec![NoIntroVersion { version_type: "v", major: "1", minor: Some("0"), prefix: None, suffixes: None, separator: None }])`
    /// * `(v1.0, PS3 v3.35 Alt)` parses to two `NoIntroVersion` values.
    Version(Vec<NoIntroVersion<'a>>),

    /// A release status flag, such as `(Sample)` or `(Beta)`
    ///
    Release {
        /// The type of release status.
        status: &'a str,
        /// The number of the release status flag, if any.
        number: Option<&'a str>,
    },

    /// A media part number flag.
    ///
    /// ## Examples
    /// * `(Disc 1)` parses to `Media { media_type: "Disc", number: "1" }`.
    Media {
        /// The media part name.
        media_type: &'a str,
        /// The number of the media part.
        number: &'a str,
    },

    /// A scene number with an optional type
    ///
    /// This appears before the title, preceding the string ` - `
    ///
    /// ## Examples
    /// * `1234` parses to `Scene { number: "1234", prefix: None }`
    /// * `z123` parses to `Scene { number: "123", prefix: Some("z") }`
    /// * `x123` parses to `Scene { number: "123", prefix: Some("x") }`
    /// * `xB123` parses to `Scene { number: "123", prefix: Some("xB") }`
    Scene {
        /// The scene number.
        number: &'a str,
        /// The letter prefix of the scene number, if any.
        prefix: Option<&'a str>,
    },

    /// A language flag containing one or more languages.
    ///
    /// See [`NoIntroLanguage`] for the structure of each parsed language.
    ///
    /// ## Examples
    /// * `(En, Zh-Hant)` parses to `Languages(vec![NoIntroLanguage { code: "En", variant: None }, NoIntroLanguage { code: "Zh", variant: Some("Hant") }])`
    Languages(Vec<NoIntroLanguage<'a>>),

    /// A generic, non-defined, or unknown flag.
    Flag {
        /// The flag's delimiter type.
        flag_type: FlagType,
        /// The text inside the flag's delimiters.
        flag: &'a str,
    },
}

/// A version parsed from a No-Intro version flag.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct NoIntroVersion<'a> {
    /// The version type, such as `Rev` or `v`, or the empty string.
    ///
    /// `v` and the empty string indicate that no space occurs between the type and
    /// the major version in the source.
    pub version_type: &'a str,
    /// The major version.
    pub major: &'a str,
    /// The minor version, separated from the major version by a dot (`.`).
    pub minor: Option<&'a str>,
    /// A prefix appearing before the version type, separated by a space, if any.
    pub prefix: Option<&'a str>,
    /// Suffixes appearing after the version number, if any.
    pub suffixes: Option<Vec<&'a str>>,
    /// The separator appearing before this version's type or prefix, if any.
    pub separator: Option<&'a str>,
}

/// A language parsed from a No-Intro language flag.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct NoIntroLanguage<'a> {
    /// The language code.
    pub code: &'a str,
    /// The language variant code, separated from the language code by a hyphen (`-`), if any.
    pub variant: Option<&'a str>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
#[repr(transparent)]
/// A No-Intro format file name.
///
/// The order of tokens in a
/// `NoIntroName` is significant in order of appearance in
/// the input file name.
pub struct NoIntroName<'a>(Vec<NoIntroToken<'a>>);

impl<'a> TokenizedName<'a, NoIntroToken<'a>> for NoIntroName<'a> {
    fn title(&self) -> Option<&'a str> {
        self.iter().find_map(|f| match f {
            NoIntroToken::Title(t) => Some(*t),
            _ => None,
        })
    }

    #[inline]
    fn iter(&self) -> Iter<'_, NoIntroToken<'a>> {
        self.0.iter()
    }

    fn try_parse<S: AsRef<str> + ?Sized>(input: &'a S) -> Result<NoIntroName<'a>> {
        let (_, value) = do_parse(input.as_ref()).map_err(|_| {
            NameError::ParseError(NamingConvention::NoIntro, input.as_ref().to_string())
        })?;
        Ok(value.into())
    }

    fn naming_convention() -> NamingConvention {
        NamingConvention::NoIntro
    }
}

impl<'a> From<Vec<NoIntroToken<'a>>> for NoIntroName<'a> {
    fn from(vec: Vec<NoIntroToken<'a>>) -> Self {
        NoIntroName(vec)
    }
}

impl Display for NoIntroName<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let mut buf = String::new();

        for token in self.iter() {
            match token {
                NoIntroToken::Title(title) => {
                    buf.push_str(title);
                }
                NoIntroToken::Region {
                    region_strings: rstrs,
                    regions: _,
                } => {
                    buf.push_str(" (");
                    buf.push_str(&rstrs.join(", "));
                    buf.push(')');
                }
                NoIntroToken::Flag {
                    flag_type: FlagType::Parenthesized,
                    flag: f,
                } => {
                    buf.push_str(" (");
                    buf.push_str(f);
                    buf.push(')')
                }
                NoIntroToken::Flag {
                    flag_type: FlagType::Bracketed,
                    flag: f,
                } => {
                    buf.push_str(" [");
                    buf.push_str(f);
                    buf.push(')');
                }

                NoIntroToken::Version(versions) => {
                    buf.push_str(" (");

                    for version in versions {
                        if let Some(separator) = version.separator {
                            buf.push_str(separator);
                        }
                        if let Some(prefix) = version.prefix {
                            buf.push_str(prefix);
                            buf.push(' ');
                        }
                        buf.push_str(version.version_type);
                        if version.version_type != "" && version.version_type != "v" {
                            buf.push(' ');
                        }

                        buf.push_str(version.major);
                        if let Some(minor) = version.minor {
                            buf.push('.');
                            buf.push_str(minor);
                        }

                        if let Some(suffixes) = &version.suffixes {
                            buf.push_str(&suffixes.join(" "));
                        }
                    }
                    buf.push(')')
                }
                NoIntroToken::Release {
                    status: beta,
                    number: num,
                } => {
                    buf.push_str(" (");
                    buf.push_str(beta);
                    if let Some(num) = num {
                        buf.push(' ');
                        buf.push_str(num);
                    }
                    buf.push(')');
                }
                NoIntroToken::Media {
                    media_type: part,
                    number: num,
                } => {
                    buf.push_str(" (");
                    buf.push_str(part);
                    buf.push(' ');
                    buf.push_str(num);
                    buf.push(')');
                }
                NoIntroToken::Scene {
                    number: num,
                    prefix,
                } => {
                    if let Some(prefix) = prefix {
                        buf.push_str(prefix)
                    }
                    buf.push_str(num);
                    buf.push_str(" - ");
                }
                NoIntroToken::Languages(langs) => {
                    buf.push_str(" (");

                    for language in langs {
                        buf.push_str(language.code);
                        if let Some(variant) = language.variant {
                            buf.push('-');
                            buf.push_str(variant);
                        }
                        buf.push(',')
                    }

                    // trim last comma
                    if buf.ends_with(',') {
                        buf.truncate(buf.len() - 1)
                    }
                    buf.push(')');
                }
            }
        }

        f.write_str(&buf)
    }
}
