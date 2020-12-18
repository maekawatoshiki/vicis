use super::super::types::{TypeId, Types};
use crate::ir::util::spaces;
use nom::{
    bytes::complete::tag, combinator::map, error::VerboseError, sequence::preceded, IResult,
};

pub fn parse<'a>(
    source: &'a str,
    types: &Types,
) -> IResult<&'a str, TypeId, VerboseError<&'a str>> {
    preceded(spaces, map(tag("i32"), |_| types.base().i32()))(source)
}
