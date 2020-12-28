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

pub fn spaces<'a>(source: &'a str) -> IResult<&'a str, (), VerboseError<&'a str>> {
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

pub fn string_literal<'a>(source: &'a str) -> IResult<&'a str, &'a str, VerboseError<&'a str>> {
    preceded(char('\"'), cut(terminated(take_until("\""), char('\"'))))(source)
}
