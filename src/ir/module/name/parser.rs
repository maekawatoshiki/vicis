use super::Name;
use crate::ir::util::spaces;
use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{alphanumeric1, char, digit1},
    combinator::map,
    error::VerboseError,
    multi::many0,
    sequence::{preceded, tuple},
    IResult,
};

pub fn parse<'a>(source: &'a str) -> IResult<&'a str, Name, VerboseError<&'a str>> {
    preceded(
        spaces,
        preceded(
            char('@'),
            preceded(
                spaces,
                alt((
                    map(digit1, |i: &'a str| Name::Number(i.parse().unwrap())),
                    map(alphanumeric1, |n: &'a str| Name::Name(n.to_string())),
                )),
            ),
        ),
    )(source)
}
