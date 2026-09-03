use crate::naming::common::error::{NameError, Result};
use crate::naming::goodtools::parsers::do_parse;
use crate::naming::{FlagType, NamingConvention, TokenizedName};
use crate::region::Region;
use std::fmt;
use std::fmt::{Display, Formatter};
use std::slice::Iter;

/// A token constituent within a `GoodToolsName`.
///
/// Tokens are not guaranteed to have consistent semantics
/// outside of a `GoodToolsName`. The order of tokens in a
/// `GoodToolsName` is significant in order of appearance in
/// the input file name.
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum GoodToolsToken<'a> {
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

    /// The year the ROM was released.
    Year(&'a str),

    /// A multi-language flag in the form `(M#)`.
    MultiLanguage(&'a str), // (M#)

    /// A translation flag in the form `[T(+/-)...]`
    ///
    Translation {
        /// The status of the translation, mapping either to a `+` or `-` in the flag.
        status: GoodToolsTranslationStatus,
        /// The remaining arguments in the translation flag.
        tags: &'a str,
    }, // [T(+/-)...]

    /// The version of the ROM.
    ///
    Version {
        /// The version prefix, such as `REV` or `V`.
        prefix: &'a str,
        /// The major version number.
        major: &'a str,
        /// The minor version number, separated by a dot (`.`), or by an underscore (`_`)
        /// if the major version is `Final`.
        minor: Option<&'a str>,
    }, // (REV/V/V /V_ ...)

    /// The volume of the ROM, for the form `(Vol #)`
    Volume(&'a str), // (Vol #)

    /// A `(#-in-1)` flag.
    ///
    NInOne {
        /// The #-in-1 entries that appear in the flag.
        entries: Vec<&'a str>,
        /// The separator, if any, separating multiple #-in-1 entries in a single flag.
        separator: Option<&'a str>,
    }, // list, sep (either + or ,)

    /// A dump code in brackets.
    ///
    /// ## Examples
    /// * `[a]` parses to `DumpCode { code: "a", number: None, code_type: None, separator: None, argument_number: None, arguments: None }`.
    /// * `[a1]` parses to `DumpCode { code: "a", number: Some("1"), .. }`.
    /// * `[hIR]` parses to `DumpCode { code: "h", code_type: Some("IR"), .. }`.
    /// * `[h1+2C]` parses to `DumpCode { code: "h", number: Some("1"), separator: Some("+"), argument_number: Some("2"), arguments: Some("C"), .. }`.
    DumpCode {
        /// The code letter, such as `a` or `h`.
        code: &'a str,
        /// The number of the dump code.
        number: Option<&'a str>,
        /// The type of the dump code.
        code_type: Option<&'a str>,
        /// A separator between the dump code and its arguments, if present.
        separator: Option<&'a str>,
        /// The number of the dump code arguments.
        argument_number: Option<&'a str>,
        /// The dump code arguments.
        arguments: Option<&'a str>,
    },

    /// A `(Hack)` flag.
    ///
    /// If a game was specified, then it will be parsed in the contained option.
    ///
    /// ## Examples
    /// * `(Hack)` parses to `GameHack(None)`
    /// * `(Adventure Hack)` parses to `GameHack(Some("Adventure"))`
    GameHack(Option<&'a str>), // (... Hack)

    /// A media parts string.
    ///
    /// ## Examples
    /// * `(Disk 1 of 2)` parses to `Media { media_type: "Disk", number: "1", total: Some("2") }`.
    Media {
        /// The name of the media part.
        media_type: &'a str,
        /// The number of the media part.
        number: &'a str,
        /// The total number of media parts, if any.
        total: Option<&'a str>,
    },

    /// A generic, non-defined, or unknown flag.
    Flag {
        /// The flag's delimiter type.
        flag_type: FlagType,
        /// The text inside the flag's delimiters.
        flag: &'a str,
    },
}

impl GoodToolsToken<'_> {
    fn is_bracketed_token(&self) -> bool {
        matches!(
            self,
            GoodToolsToken::Translation { status: _, tags: _ }
                | GoodToolsToken::DumpCode {
                    code: _,
                    number: _,
                    code_type: _,
                    separator: _,
                    argument_number: _,
                    arguments: _
                }
                | GoodToolsToken::Flag {
                    flag_type: FlagType::Bracketed,
                    flag: _
                }
        )
    }
}

/// The status of a translation in a GoodTools file name.
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum GoodToolsTranslationStatus {
    /// This translation is recent (`T+`)
    Recent,

    /// This translation is known to be outdated (`T-`)
    Outdated,
}

#[derive(Debug, Clone, Eq, PartialEq)]
#[repr(transparent)]
/// A GoodTools format file name.
///
/// The order of tokens in a
/// `GoodToolsName` is significant in order of appearance in
/// the input file name.
pub struct GoodToolsName<'a>(Vec<GoodToolsToken<'a>>);

impl<'a> TokenizedName<'a, GoodToolsToken<'a>> for GoodToolsName<'a> {
    fn title(&self) -> Option<&'a str> {
        self.iter().find_map(|f| match f {
            GoodToolsToken::Title(t) => Some(*t),
            _ => None,
        })
    }

    #[inline]
    fn iter(&self) -> Iter<'_, GoodToolsToken<'a>> {
        self.0.iter()
    }

    fn try_parse<S: AsRef<str> + ?Sized>(input: &'a S) -> Result<GoodToolsName<'a>> {
        let (_, value) = do_parse(input.as_ref()).map_err(|_| {
            NameError::ParseError(NamingConvention::GoodTools, input.as_ref().to_string())
        })?;
        Ok(value.into())
    }

    fn naming_convention() -> NamingConvention {
        NamingConvention::GoodTools
    }
}

impl<'a> From<Vec<GoodToolsToken<'a>>> for GoodToolsName<'a> {
    fn from(vec: Vec<GoodToolsToken<'a>>) -> Self {
        GoodToolsName(vec)
    }
}

impl Display for GoodToolsName<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let mut buf = String::new();

        for (i, token) in self.iter().enumerate() {
            match token {
                GoodToolsToken::Title(t) => {
                    buf.push_str(t);
                }
                GoodToolsToken::Region {
                    region_strings: rs,
                    regions: _,
                } => {
                    buf.push_str(" (");
                    for r in rs {
                        buf.push_str(r);
                        buf.push(',');
                    }
                    if buf.ends_with(',') {
                        buf.truncate(buf.len() - ",".len());
                    }
                    buf.push(')');
                }
                GoodToolsToken::Year(y) => {
                    buf.push_str(" (");
                    buf.push_str(y);
                    buf.push(')');
                }
                GoodToolsToken::MultiLanguage(num) => {
                    buf.push_str(" (M");
                    buf.push_str(num);
                    buf.push(')');
                }
                GoodToolsToken::Translation { status: t, tags } => {
                    if let Some(t) = self.0.get(i - 1) {
                        // space between brackets token and parens
                        if !t.is_bracketed_token() {
                            buf.push(' ');
                        }
                    }
                    buf.push_str("[T");
                    match t {
                        GoodToolsTranslationStatus::Recent => buf.push('+'),
                        GoodToolsTranslationStatus::Outdated => buf.push('-'),
                    }
                    buf.push_str(tags);
                    buf.push(']');
                }
                GoodToolsToken::Version {
                    prefix: ver,
                    major: maj,
                    minor: min,
                } => {
                    buf.push_str(" (");
                    buf.push_str(ver);
                    buf.push_str(maj);
                    if let Some(min) = min {
                        match maj {
                            &"Final" => buf.push('_'),
                            _ => buf.push('.'),
                        }
                        buf.push_str(min);
                    }
                    buf.push(')');
                }
                GoodToolsToken::Volume(v) => {
                    buf.push_str(" (Vol ");
                    buf.push_str(v);
                    buf.push(')');
                }
                GoodToolsToken::NInOne {
                    entries: ms,
                    separator: sep,
                } => {
                    buf.push_str(" (");
                    for m in ms {
                        buf.push_str(m);
                        if let Some(sep) = sep {
                            buf.push_str(sep);
                        }
                    }

                    if let Some(sep) = sep {
                        if buf.ends_with(sep) {
                            buf.truncate(buf.len() - sep.len());
                        }
                    }
                    buf.push(')');
                }
                GoodToolsToken::DumpCode {
                    code,
                    number: num,
                    code_type: ty,
                    separator: sep,
                    argument_number: argnum,
                    arguments: arg,
                } => {
                    if let Some(t) = self.0.get(i - 1) {
                        // space between brackets token and parens
                        if !t.is_bracketed_token() {
                            buf.push(' ');
                        }
                    }

                    buf.push('[');
                    buf.push_str(code);
                    if let Some(num) = num {
                        buf.push_str(num);
                    }
                    if let Some(ty) = ty {
                        buf.push_str(ty);
                    }
                    if let Some(sep) = sep {
                        buf.push_str(sep);
                    }
                    if let Some(argnum) = argnum {
                        buf.push_str(argnum);
                    }
                    if let Some(arg) = arg {
                        buf.push_str(arg);
                    }
                    buf.push(']');
                }
                GoodToolsToken::GameHack(hack) => {
                    buf.push_str(" (");
                    if let Some(hack) = hack {
                        buf.push_str(hack);
                        buf.push(' ');
                    }
                    buf.push_str("Hack");
                    buf.push(')');
                }
                GoodToolsToken::Media {
                    media_type: ty,
                    number: num,
                    total,
                } => {
                    buf.push_str(" (");
                    buf.push_str(ty);
                    buf.push(' ');
                    buf.push_str(num);
                    if let Some(total) = total {
                        buf.push_str(" of ");
                        buf.push_str(total);
                    }
                    buf.push(')');
                }
                GoodToolsToken::Flag {
                    flag_type: FlagType::Bracketed,
                    flag: f,
                } => {
                    if let Some(t) = self.0.get(i - 1) {
                        // space between brackets token and parens
                        if !t.is_bracketed_token() {
                            buf.push(' ');
                        }
                    }
                    buf.push('[');
                    buf.push_str(f);
                    buf.push(']');
                }
                GoodToolsToken::Flag {
                    flag_type: FlagType::Parenthesized,
                    flag: f,
                } => {
                    buf.push_str(" (");
                    buf.push_str(f);
                    buf.push(')');
                }
            }
        }

        f.write_str(buf.trim())
    }
}
