use nom::{
    bytes::complete::tag,
    sequence::delimited,
    IResult,
    Parser,
};
use nom::error::ParseError;

pub(crate) fn in_parens<'a, O, E: ParseError<&'a str>, P>(inner: P)
                                                          -> impl FnMut(&'a str) -> IResult<&'a str, O, E>
    where P: Parser<&'a str, O, E>
{
    delimited(tag("("), inner, tag(")"))
}

pub(crate) fn in_brackets<'a, O, E: ParseError<&'a str>, P>(inner: P)
    -> impl FnMut(&'a str) -> IResult<&'a str, O, E>
    where P: Parser<&'a str, O, E>
{
    delimited(tag("["), inner, tag("]"))
}

#[cfg(test)]
mod tests
{
    use nom::bytes::complete::tag;
    use nom::error::ErrorKind;
    use nom::Err;
    use crate::parse::common::parsers::*;

    #[test]
    fn in_parens_test()
    {
        let mut parser = in_parens(tag("hello"));
        assert_eq!(parser("(hello)"), Ok(("", "hello")));
        assert_eq!(parser(""), Err(Err::Error(("", ErrorKind::Tag))));
    }
    #[test]
    fn in_brackets_test()
    {
        let mut parser = in_brackets(tag("hello"));
        assert_eq!(parser("[hello]"), Ok(("", "hello")));
        assert_eq!(parser(""), Err(Err::Error(("", ErrorKind::Tag))));
    }
}

#[derive(Debug, Eq, PartialEq)]
pub enum FlagType
{
    /// The flag is parenthesized
    Parenthesized,

    /// The flag is bracketed with square brackets
    Bracketed,
}

/// A parsed version
#[derive(Debug, Eq, PartialEq)]
pub struct Version<'a>
{
    /// The version prefix.
    /// Usually 'v', 'Version', or 'Rev'
    pub version_prefix: &'a str,

    /// The major version if the version is of the form /[0-9]+\\.[a-zA-Z0-9-]/.
    /// Otherwise, if the version is not dot-separated, the entire string.
    pub major: &'a str,

    /// If the version is dot separated, everything past the dot.
    pub minor: Option<&'a str>,

    /// A prefix that appears before the version, such as
    /// (PS3 v1.40) would have prefix 'PS3'.
    pub prefix: Option<&'a str>,

    /// A suffix that appears after the version, such as
    /// (v1.40 Alt) would have suffix 'Alt'.
    pub suffix: Option<&'a Vec<&'a str>>,
}

impl <'a> From<&'a (&'a str, &'a str, Option<&'a str>, Option<&'a str>, Option<Vec<&'a str>>)> for Version<'a>
{
    fn from(tuple: &'a (&'a str, &'a str, Option<&'a str>, Option<&'a str>, Option<Vec<&'a str>>)) -> Self {
        Version {
            version_prefix: tuple.0,
            major: tuple.1,
            minor: tuple.2,
            prefix: tuple.3,
            suffix: tuple.4.as_ref()
        }
    }
}
