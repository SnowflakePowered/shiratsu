use crate::region::Region;
use crate::naming::{FlagType, NamingConvention};
use crate::naming::goodtools::parsers::do_parse;
use crate::error::{NameError, Result};
use std::slice::Iter;
use std::fmt::{Display, Formatter};
use std::fmt;

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum GoodToolsToken<'a>
{
    Title(&'a str),
    Region(Vec<&'a str>, Vec<Region>),
    Year(&'a str),
    MultiLanguage(&'a str), // (M#)
    Translation(TranslationStatus, &'a str), // [T(+/-)...]
    Version(&'a str, &'a str, Option<&'a str>), // (REV/V/V /V_ ...)
    Volume(&'a str), // (Vol #)
    NInOne(Vec<&'a str>, Option<&'a str>), // list, sep (either + or ,)

    /// A dump code in brackets
    ///  [codeNumberTypeSeparatorNumArg]
    /// `(code, number, type, separator, argnumber, args)`
    DumpCode(&'a str, Option<&'a str>, Option<&'a str>, Option<&'a str>, Option<&'a str>, Option<&'a str>),
    GameHack(Option<&'a str>), // (... Hack)
    Media(&'a str, &'a str, Option<&'a str>),
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

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum TranslationStatus
{
    Recent,
    Outdated,
}

#[derive(Debug, Clone, Eq, PartialEq)]
#[repr(transparent)]
/// A GoodTools format file name.
pub struct GoodToolsName<'a>(Vec<GoodToolsToken<'a>>);

impl GoodToolsName<'_>
{
    /// Returns an iterator over the tokens of this name.
    pub fn iter(&self) -> Iter<'_, GoodToolsToken>
    {
        self.0.iter()
    }

    pub fn try_parse<S: AsRef<str> + ?Sized>(input: &S) -> Result<GoodToolsName> {
        let (_, value) = do_parse(input.as_ref()).map_err(|_| {
            NameError::ParseError(NamingConvention::GoodTools, input.as_ref().to_string())
        })?;
        Ok(value.into())
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
                        TranslationStatus::Recent => buf.push('+'),
                        TranslationStatus::Outdated => buf.push('-'),
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