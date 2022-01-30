use crate::ir::module::visibility::Visibility;
use nom::{branch::alt, bytes::complete::tag, combinator::map, error::VerboseError, IResult};

pub fn parse(source: &str) -> IResult<&str, Visibility, VerboseError<&str>> {
    alt((
        map(tag("default"), |_| Visibility::Default),
        map(tag("hidden"), |_| Visibility::Hidden),
        map(tag("protected"), |_| Visibility::Protected),
    ))(source)
}
