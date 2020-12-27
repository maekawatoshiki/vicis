use super::Name;
use crate::ir::util::spaces;
use nom::{
    branch::alt,
    bytes::complete::take_while1,
    character::complete::{alphanumeric1, anychar, digit1},
    combinator::{all_consuming, map, recognize, verify},
    error::VerboseError,
    sequence::preceded,
    IResult,
};

pub fn parse<'a>(source: &'a str) -> IResult<&'a str, Name, VerboseError<&'a str>> {
    preceded(
        spaces,
        alt((
            map(digit1, |i: &'a str| Name::Number(i.parse().unwrap())),
            map(identifier, |n: &'a str| Name::Name(n.to_string())),
        )),
    )(source)
}

pub fn identifier<'a>(source: &'a str) -> IResult<&'a str, &'a str, VerboseError<&'a str>> {
    take_while1(|c: char| c.is_alphanumeric() || c == '.')(source)
}
