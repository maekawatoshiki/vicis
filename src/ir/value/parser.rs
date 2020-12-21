use super::{
    super::{
        function::parser::ParserContext,
        types::{Type, TypeId},
        util::spaces,
        value::{ConstantData, ConstantInt, Value},
    },
    ValueId,
};
use nom::{character::complete::digit1, error::VerboseError, sequence::preceded, IResult};

pub fn parse_constant_int<'a, 'b>(
    source: &'a str,
    ctx: &mut ParserContext<'b>,
    ty: TypeId,
) -> IResult<&'a str, ValueId, VerboseError<&'a str>> {
    let (source, num) = preceded(spaces, digit1)(source)?;
    let val = match &*ctx.types.get(ty) {
        Type::Int(32) => Value::Constant(ConstantData::Int(ConstantInt::Int32(
            num.parse::<i32>().unwrap(),
        ))),
        _ => todo!(),
    };
    Ok((source, ctx.data.create_value(val)))
}

pub fn parse<'a, 'b>(
    source: &'a str,
    ctx: &mut ParserContext<'b>,
    ty: TypeId,
) -> IResult<&'a str, ValueId, VerboseError<&'a str>> {
    parse_constant_int(source, ctx, ty)
}
