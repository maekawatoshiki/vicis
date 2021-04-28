use super::Visibility;
use nom::{branch::alt, bytes::complete::tag, combinator::map, error::VerboseError, IResult};

pub fn parse_visibility<'a>(
    source: &'a str,
) -> IResult<&'a str, Visibility, VerboseError<&'a str>> {
    alt((
        map(tag("default"), |_| Visibility::Default),
        map(tag("hidden"), |_| Visibility::Hidden),
        map(tag("protected"), |_| Visibility::Protected),
    ))(source)
}
