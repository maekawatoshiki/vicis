use crate::ir::module::unnamed_addr::UnnamedAddr;
use nom::{branch::alt, bytes::complete::tag, combinator::map, error::VerboseError, IResult};

pub fn parse(source: &str) -> IResult<&str, UnnamedAddr, VerboseError<&str>> {
    alt((
        map(tag("local_unnamed_addr"), |_| UnnamedAddr::Local),
        map(tag("unnamed_addr"), |_| UnnamedAddr::Global),
    ))(source)
}
