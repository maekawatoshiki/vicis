use super::super::types::{TypeId, Types};
use crate::ir::util::spaces;
use nom::{
    branch::alt, bytes::complete::tag, character::complete::char, combinator::map,
    error::VerboseError, multi::many0, sequence::preceded, IResult,
};

pub fn parse<'a>(
    source: &'a str,
    types: &Types,
) -> IResult<&'a str, TypeId, VerboseError<&'a str>> {
    let (source, mut base) = preceded(
        spaces,
        alt((
            map(tag("void"), |_| types.base().void()),
            map(tag("i32"), |_| types.base().i32()),
        )),
    )(source)?;
    let (source, ptrs) = many0(preceded(spaces, char('*')))(source)?;
    for _ in 0..ptrs.len() {
        base = types.base_mut().pointer(base);
    }
    Ok((source, base))
}
