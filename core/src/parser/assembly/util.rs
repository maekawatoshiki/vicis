use crate::ir::util::unescape;
use nom::{
    branch::alt,
    bytes::complete::take_until,
    character::complete::{char, multispace0},
    combinator::{cut, map},
    error::VerboseError,
    multi::many1,
    sequence::{preceded, terminated, tuple},
    IResult,
};

pub fn spaces(source: &str) -> IResult<&str, (), VerboseError<&str>> {
    alt((
        map(
            many1(tuple((
                multispace0,
                preceded(char(';'), terminated(take_until("\n"), char('\n'))),
                multispace0,
            ))),
            |_| (),
        ),
        map(multispace0, |_| ()),
    ))(source)
}

pub fn string_literal(source: &str) -> IResult<&str, String, VerboseError<&str>> {
    map(
        preceded(char('\"'), cut(terminated(take_until("\""), char('\"')))),
        |s| unescape(s).unwrap(),
    )(source)
}
