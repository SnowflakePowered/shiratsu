use crate::region::Region;
use crate::naming::{FlagType, NamingConvention, TokenizedName};
use crate::naming::goodtools::parsers::do_parse;
use crate::naming::common::error::{NameError, Result};
use std::slice::Iter;
use std::fmt::{Display, Formatter};
use std::fmt;

/// A token constituent within a `GoodToolsName`.
///
/// Tokens are not guaranteed to have consistent semantics
/// outside of a `GoodToolsName`. The order of tokens in a
/// `GoodToolsName` is significant in order of appearance in
/// the input file name.
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum GoodToolsToken<'a>
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

    /// The year the ROM was released.
    Year(&'a str),

    /// A multi-language flag in the form `(M#)`.
    MultiLanguage(&'a str), // (M#)

    /// A translation flag in the form `[T(+/-)...]`
    ///
    /// ## Tuple elements
    /// 0. The status of the translation, mapping either to a `+` or `-` in the flag.
    /// 1. The remaining arguments in the translation flag.
    Translation(GoodToolsTranslationStatus, &'a str), // [T(+/-)...]

    /// The version of the ROM.
    ///
    /// ## Tuple elements
    /// 0. The version prefix, such as `REV`, `V` etc.
    /// 1. The major version number.
    /// 2. The minor version number, separated by a dot (`.`) or underscore (`_`) if the major version is `Final`
    Version(&'a str, &'a str, Option<&'a str>), // (REV/V/V /V_ ...)

    /// The volume of the ROM, for the form `(Vol #)`
    Volume(&'a str), // (Vol #)

    /// A `(#-in-1)` flag.
    ///
    /// ## Tuple elements
    /// 0. The #-in-1 entries that appear in the flag.
    /// 1. The separator, if any, separating multiple #-in-1 entries in a single flag.
    NInOne(Vec<&'a str>, Option<&'a str>), // list, sep (either + or ,)

    /// A dump code in brackets.
    ///
    /// ## Tuple elements
    /// 0. The code letter, such as `a`, or `h`.
    /// 1. The number of the dump code.
    /// 2. The type of the dump code.
    /// 3. If present, a separator between the dump code and it's arguments.
    /// 4. The number of the dump code arguments.
    /// 5. The dump code arguments.
    ///
    /// ## Examples
    /// * `[a]` parses to `DumpCode("a", None, None, None, None, None)`.
    /// * `[a1]` parses to `DumpCode("a", Some("1"), None, None, None, None)`.
    /// * `[hIR]` parses to `DumpCode("h", None, Some("IR"), None, None, None)`.
    /// * `[h1+2C]` parses to `DumpCode("h", Some("1"), None, Some("+"), Some("2"), Some("C"))`.
    DumpCode(&'a str, Option<&'a str>, Option<&'a str>, Option<&'a str>, Option<&'a str>, Option<&'a str>),

    /// A `(Hack)` flag.
    ///
    /// If a game was specified, then it will be parsed in the first element of the tuple.
    ///
    /// ## Examples
    /// * `(Hack)` parses to `GameHack(None)`
    /// * `(Adventure Hack)` parses to `GameHack(Some("Adventure"))`
    GameHack(Option<&'a str>), // (... Hack)

    /// A media parts string.
    ///
    /// ## Tuple elements
    /// 0. The name of the media part.
    /// 1. The number of the media part.
    /// 2. The total parts of the media, if any.
    ///
    /// ## Examples
    /// * `(Disk 1 of 2)` parses to `Media("Disk", "1", Some("2"))`.
    Media(&'a str, &'a str, Option<&'a str>),

    /// A generic, non-defined, or unknown flag.
    Flag(FlagType, &'a str),
}

impl GoodToolsToken<'_> {
    fn is_bracketed_token(&self) -> bool {
        match self {
            GoodToolsToken::Translation(_, _) => true,
            GoodToolsToken::DumpCode(_, _, _, _, _, _) => true,
            GoodToolsToken::Flag(FlagType::Bracketed, _) => true,
            _ => false,
        }
    }
}

/// The status of a translation in a GoodTools file name.
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum GoodToolsTranslationStatus
{
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

impl <'a> TokenizedName<'a, GoodToolsToken<'a>> for GoodToolsName<'a>
{
    fn title(&self) -> Option<&'a str> {
        self.iter()
            .find_map(|f| match f {
                GoodToolsToken::Title(t) => Some(*t),
                _ => None
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

impl <'a> From<Vec<GoodToolsToken<'a>>> for GoodToolsName<'a>
{
    fn from(vec: Vec<GoodToolsToken<'a>>) -> Self {
        GoodToolsName(vec)
    }
}

impl Display for GoodToolsName<'_>
{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let mut buf = String::new();

        for (i, token) in self.iter().enumerate() {
            match token {
                GoodToolsToken::Title(t) => { buf.push_str(t); }
                GoodToolsToken::Region(rs, _) => {
                    buf.push_str(" (");
                    for r in rs {
                        buf.push_str(r);
                        buf.push(',');
                    }
                    if buf.ends_with(",") {
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
                GoodToolsToken::Translation(t, tags) => {
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
                GoodToolsToken::Version(ver, maj, min) => {
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
                GoodToolsToken::NInOne(ms, sep) => {
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
                GoodToolsToken::DumpCode(code, num, ty, sep, argnum, arg) => {
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
                GoodToolsToken::Media(ty, num, total) => {
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
                GoodToolsToken::Flag(FlagType::Bracketed, f) => {
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
                GoodToolsToken::Flag(FlagType::Parenthesized, f) => {
                    buf.push_str(" (");
                    buf.push_str(f);
                    buf.push_str(")");
                }
            }
        }

        f.write_str(buf.trim())
    }
}