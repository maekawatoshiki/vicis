use super::Name;
use crate::ir::util::{spaces, string_literal};
use nom::{
    branch::alt, bytes::complete::take_while1, character::complete::digit1, combinator::map,
    error::VerboseError, sequence::preceded, IResult,
};

pub fn parse<'a>(source: &'a str) -> IResult<&'a str, Name, VerboseError<&'a str>> {
    preceded(
        spaces,
        alt((
            map(digit1, |i: &'a str| Name::Number(i.parse().unwrap())),
            map(identifier, |n: &'a str| Name::Name(n.to_string())),
            map(string_literal, |s: String| Name::Name(format!(r#"{}"#, s))),
        )),
    )(source)
}

pub fn identifier<'a>(source: &'a str) -> IResult<&'a str, &'a str, VerboseError<&'a str>> {
    take_while1(|c: char| c.is_alphanumeric() || c == '.' || c == '_')(source)
}
