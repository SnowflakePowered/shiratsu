use nom::branch::alt;
use nom::bytes::complete::take_while_m_n;
use nom::combinator::recognize;
use nom::error::{ErrorKind, ParseError};
use nom::{
    bytes::complete::tag, sequence::delimited, FindSubstring, FindToken, IResult, InputIter,
    InputLength, InputTake, InputTakeAtPosition, Parser,
};

pub(crate) fn in_parens<'a, O, E: ParseError<&'a str>, P>(
    inner: P,
) -> impl FnMut(&'a str) -> IResult<&'a str, O, E>
where
    P: Parser<&'a str, O, E>,
{
    delimited(tag("("), inner, tag(")"))
}

pub(crate) fn in_brackets<'a, O, E: ParseError<&'a str>, P>(
    inner: P,
) -> impl FnMut(&'a str) -> IResult<&'a str, O, E>
where
    P: Parser<&'a str, O, E>,
{
    delimited(tag("["), inner, tag("]"))
}

pub(crate) fn take_until_is<Arr, Tag, Input, Error: ParseError<Input>>(
    arr: Arr,
    t: Tag,
) -> impl FnMut(Input) -> IResult<Input, Input, Error>
where
    Input: InputTake + InputTakeAtPosition + FindSubstring<Tag>,
    Tag: InputLength + Clone,
    Arr: FindToken<<Input as InputTakeAtPosition>::Item>,
{
    move |i: Input| {
        let t = t.clone();

        let (_rest, test) =
            i.split_at_position1_complete(|c| arr.find_token(c), ErrorKind::IsNot)?;

        let res: IResult<_, _, Error> = match test.find_substring(t) {
            None => Err(nom::Err::Error(Error::from_error_kind(
                i,
                ErrorKind::TakeUntil,
            ))),
            // the byte offset from find_substring should be safe to split at.
            Some(index) => Ok(i.take_split(index)),
        };

        res
    }
}

/// Return the input slice up to the first occurrence of the parser,
/// and the result of the parser on match.
/// If the parser never matches, returns an error with code `ManyTill`
pub(crate) fn take_up_to<Input, Output, Error: ParseError<Input>, P>(
    mut parser: P,
) -> impl FnMut(Input) -> IResult<Input, (Input, Output), Error>
where
    P: FnMut(Input) -> IResult<Input, Output, Error>,
    Input: InputLength + InputIter + InputTake,
{
    move |i: Input| {
        let input = i;
        for (index, _) in input.iter_indices() {
            let (rest, front) = input.take_split(index);
            match parser(rest) {
                Ok((remainder, output)) => return Ok((remainder, (front, output))),
                Err(_) => continue,
            }
        }
        Err(nom::Err::Error(Error::from_error_kind(
            input,
            ErrorKind::ManyTill,
        )))
    }
}

pub(crate) fn take_year(input: &str) -> IResult<&str, &str> {
    fn take_year_inner(input: &str) -> IResult<&str, ()> {
        let (input, _) = alt((tag("19"), tag("20")))(input)?;
        let (input, _) =
            take_while_m_n(2, 2, |c: char| c == 'X' || c == 'x' || c.is_ascii_digit())(input)?;
        Ok((input, ()))
    }

    recognize(take_year_inner)(input)
}

macro_rules! make_parens_tag {
    ($fn_name:ident, $inner:ident, $token:ty) => {
        fn $fn_name<'a>(input: &'a str) -> IResult<&str, $token> {
            in_parens($inner)(input)
        }
    };
}

#[cfg(test)]
mod tests {
    use crate::naming::common::parsers::*;
    use nom::bytes::complete::tag;
    use nom::error::ErrorKind;
    use nom::Err;

    #[test]
    fn in_parens_test() {
        let mut parser = in_parens(tag("hello"));
        assert_eq!(parser("(hello)"), Ok(("", "hello")));
        assert_eq!(parser(""), Err(Err::Error(("", ErrorKind::Tag))));
    }
    #[test]
    fn in_brackets_test() {
        let mut parser = in_brackets(tag("hello"));
        assert_eq!(parser("[hello]"), Ok(("", "hello")));
        assert_eq!(parser(""), Err(Err::Error(("", ErrorKind::Tag))));
    }
}
