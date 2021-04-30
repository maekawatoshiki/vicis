use super::{
    super::{
        function::parser::ParserContext,
        module::name,
        types,
        types::{Type, TypeId, Types},
        util::{spaces, string_literal},
        value::{ConstantArray, ConstantData, ConstantExpr, ConstantInt, ConstantStruct, Value},
    },
    ValueId,
};
use nom::{
    bytes::complete::tag,
    character::complete::{char, digit1},
    combinator::{opt, recognize},
    error::VerboseError,
    sequence::{preceded, tuple},
    IResult,
};

pub fn parse_constant<'a>(
    source: &'a str,
    types: &Types,
    ty: TypeId,
) -> IResult<&'a str, ConstantData, VerboseError<&'a str>> {
    if let Ok((source, _)) = preceded(spaces, tag("undef"))(source) {
        return Ok((source, ConstantData::Undef));
    }
    if let Ok((source, id)) = parse_constant_int(source, types, ty) {
        return Ok((source, id));
    }
    if let Ok((source, id)) = parse_constant_array(source, types) {
        return Ok((source, id));
    }
    if let Ok((source, id)) = parse_constant_global_ref(source) {
        return Ok((source, id));
    }
    if let Ok((source, id)) = parse_constant_struct(source, types) {
        return Ok((source, id));
    }
    parse_constant_expr(source, types)
}

pub fn parse_constant_int<'a>(
    source: &'a str,
    types: &Types,
    ty: TypeId,
) -> IResult<&'a str, ConstantData, VerboseError<&'a str>> {
    let (source, num) = preceded(spaces, recognize(tuple((opt(char('-')), digit1))))(source)?;
    let val = match &*types.get(ty) {
        Type::Int(32) => ConstantData::Int(ConstantInt::Int32(num.parse::<i32>().unwrap())),
        Type::Int(64) => ConstantData::Int(ConstantInt::Int64(num.parse::<i64>().unwrap())),
        _ => todo!(),
    };
    Ok((source, val))
}

pub fn parse_constant_array<'a, 'b>(
    source: &'a str,
    types: &Types,
    // ty: TypeId,
) -> IResult<&'a str, ConstantData, VerboseError<&'a str>> {
    // TODO: Support arrays in the form of [a, b, c]
    let (source, _) = preceded(spaces, char('c'))(source)?;
    let (source, s) = preceded(spaces, string_literal)(source)?;
    let val = ConstantData::Array(ConstantArray {
        elem_ty: types.base().i8(),
        elems: s
            .as_bytes()
            .into_iter()
            .map(|c| ConstantData::Int(ConstantInt::Int8(*c as i8)))
            .collect(),
        is_string: true,
    });
    Ok((source, val))

    // let (mut source, _) = preceded(spaces, char('['))(source)?;
    // loop {
    //     let (source_, ty) = types::parse(source, ctx.types)?;
    //
    // }

    // let (source, num) = preceded(spaces, digit1)(source)?;
    // let val = match &*ctx.types.get(ty) {
    //     Type::Int(32) => Value::Constant(ConstantData::Int(ConstantInt::Int32(
    //         num.parse::<i32>().unwrap(),
    //     ))),
    //     _ => todo!(),
    // };
    // Ok((source, ctx.data.create_value(val)))
}

pub fn parse_constant_expr<'a, 'b>(
    source: &'a str,
    types: &Types,
) -> IResult<&'a str, ConstantData, VerboseError<&'a str>> {
    parse_constant_getelementptr(source, types)
}

pub fn parse_constant_getelementptr<'a, 'b>(
    source: &'a str,
    types: &Types,
) -> IResult<&'a str, ConstantData, VerboseError<&'a str>> {
    let (source, _) = preceded(spaces, tag("getelementptr"))(source)?;
    let (source, inbounds) = opt(preceded(spaces, tag("inbounds")))(source)?;
    let (source, _) = preceded(spaces, char('('))(source)?;
    let (source, ty) = types::parse(source, types)?;
    let (mut source, _) = preceded(spaces, char(','))(source)?;
    let mut args = vec![];
    let mut tys = vec![ty];
    loop {
        let (source_, ty) = types::parse(source, types)?;
        let (source_, arg) = parse_constant(source_, types, ty)?;
        tys.push(ty);
        args.push(arg);
        if let Ok((source_, _)) = preceded(spaces, char(','))(source_) {
            source = source_;
            continue;
        }
        if let Ok((source, _)) = preceded(spaces, char(')'))(source_) {
            return Ok((
                source,
                ConstantData::Expr(ConstantExpr::GetElementPtr {
                    inbounds: inbounds.is_some(),
                    tys,
                    args,
                }),
            ));
        }
    }
}

pub fn parse_constant_global_ref<'a>(
    source: &'a str,
) -> IResult<&'a str, ConstantData, VerboseError<&'a str>> {
    let (source, name) = preceded(spaces, preceded(char('@'), name::parse))(source)?;
    Ok((source, ConstantData::GlobalRef(name)))
}

pub fn parse_constant_struct<'a>(
    source: &'a str,
    types: &Types,
) -> IResult<&'a str, ConstantData, VerboseError<&'a str>> {
    let (mut source, _) = preceded(spaces, char('{'))(source)?;
    let mut elems = vec![];
    let mut elems_ty = vec![];
    loop {
        let (source_, t) = types::parse(source, types)?;
        let (source_, konst) = parse_constant(source_, types, t)?;
        elems.push(konst);
        elems_ty.push(t);
        if let Ok((source_, _)) = preceded(spaces, char(','))(source_) {
            source = source_;
            continue;
        }
        let (source_, _) = preceded(spaces, char('}'))(source_)?;
        return Ok((
            source_,
            ConstantData::Struct(ConstantStruct { elems_ty, elems }),
        ));
    }
}

pub fn parse_local<'a, 'b>(
    source: &'a str,
    ctx: &mut ParserContext<'b>,
    _ty: TypeId,
) -> IResult<&'a str, ValueId, VerboseError<&'a str>> {
    let (source, name) = preceded(spaces, preceded(char('%'), name::parse))(source)?;
    Ok((source, ctx.get_or_create_named_value(name)))
}

pub fn parse<'a, 'b>(
    source: &'a str,
    ctx: &mut ParserContext<'b>,
    ty: TypeId,
) -> IResult<&'a str, ValueId, VerboseError<&'a str>> {
    if let Ok((source, konst)) = parse_constant(source, ctx.types, ty) {
        let id = ctx.data.create_value(Value::Constant(konst));
        return Ok((source, id));
    }

    parse_local(source, ctx, ty)
}
