use super::super::types::{TypeId, Types};
use crate::ir::util::spaces;
use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{char, digit1},
    combinator::map,
    error::VerboseError,
    multi::many0,
    sequence::preceded,
    IResult,
};

pub fn parse<'a>(
    source: &'a str,
    types: &Types,
) -> IResult<&'a str, TypeId, VerboseError<&'a str>> {
    let (source, mut base) = if let Ok((source, _)) = preceded(spaces, char('['))(source) {
        parse_array(source, types)?
    } else {
        preceded(
            spaces,
            alt((
                map(tag("void"), |_| types.base().void()),
                map(tag("i1"), |_| types.base().i1()),
                map(tag("i8"), |_| types.base().i8()),
                map(tag("i32"), |_| types.base().i32()),
                map(tag("i64"), |_| types.base().i64()),
            )),
        )(source)?
    };

    let (source, ptrs) = many0(preceded(spaces, char('*')))(source)?;
    for _ in 0..ptrs.len() {
        base = types.base_mut().pointer(base);
    }
    Ok((source, base))
}

pub fn parse_array<'a>(
    source: &'a str,
    types: &Types,
) -> IResult<&'a str, TypeId, VerboseError<&'a str>> {
    let (source, n) = preceded(spaces, digit1)(source)?;
    let (source, _) = preceded(spaces, char('x'))(source)?;
    let (source, ty) = parse(source, types)?;
    let (source, _) = preceded(spaces, char(']'))(source)?;
    let ary_ty = types.base_mut().array(ty, n.parse::<u32>().unwrap());
    Ok((source, ary_ty))
}
