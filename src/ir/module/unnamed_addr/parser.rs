use super::UnnamedAddr;
use nom::{branch::alt, bytes::complete::tag, combinator::map, error::VerboseError, IResult};

pub fn parse<'a>(source: &'a str) -> IResult<&'a str, UnnamedAddr, VerboseError<&'a str>> {
    alt((
        map(tag("local_unnamed_addr"), |_| UnnamedAddr::Local),
        map(tag("unnamed_addr"), |_| UnnamedAddr::Global),
    ))(source)
}
