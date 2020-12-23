use super::{
    super::{
        function::parser::ParserContext,
        module::name,
        types::{Type, TypeId},
        util::spaces,
        value::{ConstantData, ConstantInt, Value},
    },
    ValueId,
};
use nom::{
    character::complete::{char, digit1},
    error::VerboseError,
    sequence::preceded,
    IResult,
};

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

pub fn parse_global<'a, 'b>(
    source: &'a str,
    ctx: &mut ParserContext<'b>,
    _ty: TypeId,
) -> IResult<&'a str, ValueId, VerboseError<&'a str>> {
    let (source, name) = preceded(spaces, preceded(char('@'), name::parse))(source)?;
    Ok((
        source,
        ctx.data.create_value(Value::UnresolvedGlobalName(name)),
    ))
}

pub fn parse_local<'a, 'b>(
    source: &'a str,
    ctx: &mut ParserContext<'b>,
    _ty: TypeId,
) -> IResult<&'a str, ValueId, VerboseError<&'a str>> {
    let (source, name) = preceded(spaces, preceded(char('%'), name::parse))(source)?;
    Ok((source, ctx.name_to_value[&name]))
}

pub fn parse<'a, 'b>(
    source: &'a str,
    ctx: &mut ParserContext<'b>,
    ty: TypeId,
) -> IResult<&'a str, ValueId, VerboseError<&'a str>> {
    if let Ok((source, id)) = parse_constant_int(source, ctx, ty) {
        return Ok((source, id));
    }

    if let Ok((source, id)) = parse_global(source, ctx, ty) {
        return Ok((source, id));
    }

    parse_local(source, ctx, ty)
}
