use crate::ir::module::name::Name;
use nom::{
    branch::alt, bytes::complete::take_while1, character::complete::digit1, combinator::map,
    error::VerboseError, sequence::preceded, IResult,
};

use super::util::{spaces, string_literal};

pub fn parse(source: &str) -> IResult<&str, Name, VerboseError<&str>> {
    preceded(
        spaces,
        alt((
            map(digit1, |i: &str| Name::Number(i.parse().unwrap())),
            map(identifier, |n: &str| Name::Name(n.to_string())),
            map(string_literal, Name::Name),
        )),
    )(source)
}

pub fn identifier(source: &str) -> IResult<&str, &str, VerboseError<&str>> {
    take_while1(|c: char| c.is_alphanumeric() || c == '.' || c == '_')(source)
}
