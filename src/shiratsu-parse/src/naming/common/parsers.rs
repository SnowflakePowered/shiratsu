use nom::{bytes::complete::tag, sequence::delimited,
          IResult, Parser, InputTakeAtPosition, InputTake, InputLength, FindSubstring, FindToken};
use nom::error::{ParseError, ErrorKind};

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


pub(crate) fn take_until_is<Arr, Tag, Input, Error: ParseError<Input>>(arr: Arr, t: Tag)
                                                            -> impl FnMut(Input) -> IResult<Input, Input, Error>
    where
        Input: InputTake + InputTakeAtPosition + FindSubstring<Tag>,
        Tag: InputLength + Clone,
        Arr: FindToken<<Input as InputTakeAtPosition>::Item>
{
    move |i: Input| {
        let t = t.clone();

        let (_rest, test)  =
            i.split_at_position1_complete(|c| arr.find_token(c),
                                          ErrorKind::IsNot)?;

        let res: IResult<_, _, Error> = match test.find_substring(t) {
            None => Err(nom::Err::Error(Error::from_error_kind(i, ErrorKind::TakeUntil))),
            Some(index) => Ok(i.take_split(index)),
        };

        res
    }
}

macro_rules! make_parens_tag {
    ($fn_name:ident, $inner:ident, $token:ty) =>
    {
        fn $fn_name<'a>(input: &'a str) -> IResult<&str, $token>
        {
            in_parens($inner)(input)
        }
    }
}

#[cfg(test)]
mod tests
{
    use nom::bytes::complete::tag;
    use nom::error::ErrorKind;
    use nom::Err;
    use crate::naming::common::parsers::*;

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