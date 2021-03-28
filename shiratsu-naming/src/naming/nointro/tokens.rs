use crate::region::Region;
use crate::naming::{FlagType, NamingConvention, TokenizedName};
use crate::naming::common::error::{NameError, Result};
use crate::naming::nointro::parsers::do_parse;
use std::slice::Iter;
use std::fmt::{Display, Formatter};
use std::fmt;

/// A token constituent within a `NoIntroName`.
///
/// Tokens are not guaranteed to have consistent semantics
/// outside of a `NoIntroName`. The order of tokens in a
/// `NoIntroName` is significant in order of appearance in
/// the input file name.
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum NoIntroToken<'a>
{
    /// The title of the ROM.
    Title(&'a str),

    /// The region of the ROM.
    ///
    /// There is no one-to-one correspondence between the strings in the
    /// region flag, and the list of parsed regions, since regions may
    /// undergo expansion.
    ///
    /// ## Tuple elements
    /// 0. The region strings that correspond to the parsed regions.
    /// 1. The parsed regions.
    Region(Vec<&'a str>, Vec<Region>),

    /// A version flag.
    ///
    /// A version flag may contain one or more versions, separated by a separator.
    ///
    /// ## Tuple elements
    /// Each tuple in the contained vector has the following format.
    ///
    /// 0. The version type, such as `Rev` or `v`, or the empty string.
    ///    * Note that `v` and the empty string implies that there are no spaces in the source version
    ///      between the type and the major version.
    /// 1. The major version.
    /// 2. The minor version, separated by a dot (`.`).
    /// 3. The version prefix, if any.
    ///    This is not the version type, but the prefix that appears, separated by a space, before the type.
    /// 4. The version suffix, if any.
    ///    This appears after the version number, separated by a space.
    /// 5. The version separator that appears _before_ the version type or prefix.
    ///
    /// ## Examples
    /// * `(v1.0)` parses to `Version(vec![("v", "1", Some("0"), None, None, None)])`
    /// * `(v1.0, PS3 v3.35 Alt)`
    ///    parses to `Version(vec![("v", "1", Some("0"), ...),
    ///    ("v", "3", Some("35"), Some("PS3"), Some(vec!["Alt"]), Some(", ")`
    Version(Vec<(&'a str, &'a str, Option<&'a str>, Option<&'a str>, Option<Vec<&'a str>>, Option<&'a str>)>),

    /// A release status flag, such as `(Sample)` or `(Beta)`
    ///
    /// ## Tuple elements
    /// 0. The type of release status
    /// 1. The number of the release status flag, if any.
    Release(&'a str, Option<&'a str>),

    /// A media part number flag.
    ///
    /// ## Tuple elements
    /// 0. The media part name
    /// 1. The number of the media part.
    ///
    /// ## Examples
    /// * `(Disc 1)` parses to `Media("Disc", "1")`.
    Media(&'a str, &'a str),

    /// A scene number with an optional type
    ///
    /// This appears before the title, preceding the string ` - `
    ///
    /// ## Tuple elements
    /// 0. The number of the scene number.
    /// 1. The letter type of the scene number, if any.
    ///
    /// ## Examples
    /// * `1234` parses to `Scene("1234", None)`
    /// * `z123` parses to `Scene("123", Some("z"))`
    /// * `x123` parses to `Scene("123", Some("x"))`
    /// * `xB123` parses to `Scene("123", Some("xB"))`
    Scene(&'a str, Option<&'a str>),

    /// A language flag containing one or more languages.
    ///
    /// ## Tuple elements.
    /// Each tuple in the contained vector has the following format.
    ///
    /// 0. The language code of the language.
    /// 1. The variant code of the language, separated by a hyphen (`-`).
    ///
    /// ## Examples
    /// * `(En, Zh-Hant)` parses to `Languages(vec![("En", None), ("Zh", Some("Hant"))])`
    Languages(Vec<(&'a str, Option<&'a str>)>),

    /// A generic, non-defined, or unknown flag.
    Flag(FlagType, &'a str),
}

#[derive(Debug, Clone, Eq, PartialEq)]
#[repr(transparent)]
/// A No-Intro format file name.
///
/// The order of tokens in a
/// `NoIntroName` is significant in order of appearance in
/// the input file name.
pub struct NoIntroName<'a>(Vec<NoIntroToken<'a>>);

impl <'a> TokenizedName<'a, NoIntroToken<'a>> for NoIntroName<'a>
{
    fn title(&self) -> Option<&'a str> {
        self.iter()
            .find_map(|f| match f {
                NoIntroToken::Title(t) => Some(*t),
                _ => None
            })
    }

    #[inline]
    fn iter(&self) -> Iter<'_, NoIntroToken<'a>>
    {
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

impl <'a> From<Vec<NoIntroToken<'a>>> for NoIntroName<'a>
{
    fn from(vec: Vec<NoIntroToken<'a>>) -> Self {
        NoIntroName(vec)
    }
}

impl Display for NoIntroName<'_>
{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let mut buf = String::new();

        for token in self.iter()
        {
            match token {
                NoIntroToken::Title(title) => {
                    buf.push_str(title);
                }
                NoIntroToken::Region(rstrs, _) => {
                    buf.push_str(" (");
                    buf.push_str(&rstrs.join(", "));
                    buf.push(')');
                }
                NoIntroToken::Flag(FlagType::Parenthesized, f) => {
                    buf.push_str(" (");
                    buf.push_str(f);
                    buf.push(')')
                }
                NoIntroToken::Flag(FlagType::Bracketed, f) => {
                    buf.push_str(" [");
                    buf.push_str(f);
                    buf.push(')');
                }

                NoIntroToken::Version(versions) => {
                    buf.push_str(" (");

                    for (ver, major, minor, prefix,
                        suffixes, sep)
                        in versions {
                        if let Some(sep) = sep {
                            buf.push_str(sep);
                        }
                        if let Some(prefix) = prefix
                        {
                            buf.push_str(prefix);
                            buf.push(' ');
                        }
                        buf.push_str(ver);
                        if ver != &"" && ver != &"v" {
                            buf.push(' ');
                        }

                        buf.push_str(major);
                        if let Some(minor) = minor {
                            buf.push('.');
                            buf.push_str(minor);
                        }

                        if let Some(suffixes) = suffixes {
                            buf.push_str(&suffixes.join(" "));
                        }
                    }
                    buf.push(')')
                }
                NoIntroToken::Release(beta, num) => {
                    buf.push_str(" (");
                    buf.push_str(beta);
                    if let Some(num) = num {
                        buf.push(' ');
                        buf.push_str(num);
                    }
                    buf.push_str(")");
                }
                NoIntroToken::Media(part, num) => {
                    buf.push_str(" (");
                    buf.push_str(part);
                    buf.push(' ');
                    buf.push_str(num);
                    buf.push_str(")");
                }
                NoIntroToken::Scene(num, prefix) => {
                    if let Some(prefix) = prefix {
                        buf.push_str(prefix)
                    }
                    buf.push_str(num);
                    buf.push_str(" - ");
                }
                NoIntroToken::Languages(langs) => {
                    buf.push_str(" (");

                    for (lang, tag) in langs.iter() {
                        buf.push_str(lang);
                        if let Some(tag) = tag {
                            buf.push_str("-");
                            buf.push_str(tag);
                        }
                        buf.push_str(",")
                    }

                    // trim last comma
                    if buf.ends_with(",") {
                        buf.truncate(buf.len() - 1)
                    }
                    buf.push_str(")");
                }
            }
        }

        f.write_str(&buf)
    }
}
