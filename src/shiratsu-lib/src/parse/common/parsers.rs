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