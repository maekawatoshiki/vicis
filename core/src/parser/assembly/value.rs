use super::{
    function::ParserContext,
    util::{spaces, string_literal},
};
use crate::ir::{
    types::{Type, Types, I1, I32, I64, I8},
    value::{
        ConstantArray, ConstantExpr, ConstantInt, ConstantStruct, ConstantValue, Value, ValueId,
    },
};
use nom::{
    branch::alt,
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
    ty: Type,
) -> IResult<&'a str, ConstantValue, VerboseError<&'a str>> {
    if let Ok((source, _)) = preceded(spaces, tag("undef"))(source) {
        return Ok((source, ConstantValue::Undef(ty)));
    }
    if let Ok((source, _)) = preceded(spaces, tag("null"))(source) {
        return Ok((source, ConstantValue::Null(ty)));
    }
    if let Ok((source, _)) = preceded(spaces, tag("zeroinitializer"))(source) {
        return Ok((source, ConstantValue::AggregateZero(ty)));
    }
    if let Ok((source, id)) = parse_constant_int(source, ty) {
        return Ok((source, id.into()));
    }
    if let Ok((source, id)) = parse_constant_array(source, types, ty) {
        return Ok((source, id));
    }
    if let Ok((source, id)) = parse_constant_global_ref(source, ty) {
        return Ok((source, id));
    }
    if let Ok((source, id)) = parse_constant_struct(source, types, ty) {
        return Ok((source, id));
    }
    parse_constant_expr(source, types)
}

pub fn parse_constant_int(
    source: &str,
    ty: Type,
) -> IResult<&str, ConstantInt, VerboseError<&str>> {
    let (source, num) = preceded(
        spaces,
        recognize(tuple((
            opt(char('-')),
            alt((digit1, tag("true"), tag("false"))),
        ))),
    )(source)?;
    let val = match ty {
        I1 => ConstantInt::Int1(num == "true"),
        I8 => ConstantInt::Int8(num.parse::<i8>().unwrap()),
        I32 => ConstantInt::Int32(num.parse::<i32>().unwrap()),
        I64 => ConstantInt::Int64(num.parse::<i64>().unwrap()),
        _ => todo!(),
    };
    Ok((source, val))
}

pub fn parse_constant_array<'a>(
    source: &'a str,
    types: &Types,
    ty: Type,
) -> IResult<&'a str, ConstantValue, VerboseError<&'a str>> {
    if let Ok((source, _)) = preceded(spaces, char('c'))(source) {
        let (source, s) = preceded(spaces, string_literal)(source)?;
        let val = ConstantValue::Array(ConstantArray {
            ty,
            elem_ty: I8,
            elems: s
                .as_bytes()
                .iter()
                .map(|c| ConstantValue::Int(ConstantInt::Int8(*c as i8)))
                .collect(),
            is_string: true,
        });
        return Ok((source, val));
    }

    let (mut source, _) = preceded(spaces, char('['))(source)?;
    let mut elems = vec![];
    loop {
        let (source_, ty) = super::types::parse(types)(source)?;
        let (source_, elem) = parse_constant(source_, types, ty)?;

        elems.push(elem);

        if let Ok((source_, _)) = tuple((spaces, char(',')))(source_) {
            source = source_;
            continue;
        }

        source = source_;
        break;
    }
    let (source, _) = preceded(spaces, char(']'))(source)?;
    Ok((
        source,
        ConstantValue::Array(ConstantArray {
            ty,
            elem_ty: types.get_element(ty).unwrap(),
            elems,
            is_string: false,
        }),
    ))
}

pub fn parse_constant_expr<'a>(
    source: &'a str,
    types: &Types,
) -> IResult<&'a str, ConstantValue, VerboseError<&'a str>> {
    if let Ok((source, konst)) = parse_constant_getelementptr(source, types) {
        return Ok((source, konst));
    }
    parse_constant_bitcast(source, types)
}

pub fn parse_constant_getelementptr<'a>(
    source: &'a str,
    types: &Types,
) -> IResult<&'a str, ConstantValue, VerboseError<&'a str>> {
    let (source, _) = preceded(spaces, tag("getelementptr"))(source)?;
    let (source, inbounds) = opt(preceded(spaces, tag("inbounds")))(source)?;
    let (source, _) = preceded(spaces, char('('))(source)?;
    let (source, ty) = super::types::parse(types)(source)?;
    let (mut source, _) = preceded(spaces, char(','))(source)?;
    let mut args = vec![];
    let mut tys = vec![ty];
    loop {
        let (source_, ty) = super::types::parse(types)(source)?;
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
                ConstantValue::Expr(ConstantExpr::GetElementPtr {
                    inbounds: inbounds.is_some(),
                    tys,
                    args,
                }),
            ));
        }
    }
}

pub fn parse_constant_bitcast<'a>(
    source: &'a str,
    types: &Types,
) -> IResult<&'a str, ConstantValue, VerboseError<&'a str>> {
    let (source, _) = preceded(spaces, tag("bitcast"))(source)?;
    let (source, _) = preceded(spaces, char('('))(source)?;
    let (source, from) = super::types::parse(types)(source)?;
    let (source, arg) = parse_constant(source, types, from)?;
    let (source, _) = preceded(spaces, tag("to"))(source)?;
    let (source, to) = super::types::parse(types)(source)?;
    let (source, _) = preceded(spaces, char(')'))(source)?;
    Ok((
        source,
        ConstantValue::Expr(ConstantExpr::Bitcast {
            tys: [from, to],
            arg: Box::new(arg),
        }),
    ))
}

pub fn parse_constant_global_ref(
    source: &str,
    ty: Type,
) -> IResult<&str, ConstantValue, VerboseError<&str>> {
    let (source, name) = preceded(spaces, preceded(char('@'), super::name::parse))(source)?;
    Ok((source, ConstantValue::GlobalRef(name, ty)))
}

pub fn parse_constant_struct<'a>(
    source: &'a str,
    types: &Types,
    ty: Type,
) -> IResult<&'a str, ConstantValue, VerboseError<&'a str>> {
    let (mut source, is_packed) = preceded(spaces, alt((tag("{"), tag("<{"))))(source)?;
    let is_packed = is_packed == "<{";
    let mut elems = vec![];
    let mut elems_ty = vec![];
    loop {
        let (source_, t) = super::types::parse(types)(source)?;
        let (source_, konst) = parse_constant(source_, types, t)?;
        elems.push(konst);
        elems_ty.push(t);
        if let Ok((source_, _)) = preceded(spaces, char(','))(source_) {
            source = source_;
            continue;
        }
        let (source_, _) = preceded(spaces, tag(if is_packed { "}>" } else { "}" }))(source_)?;
        return Ok((
            source_,
            ConstantValue::Struct(ConstantStruct {
                ty,
                elems_ty,
                elems,
                is_packed,
            }),
        ));
    }
}

pub fn parse_local<'a, 'b>(
    source: &'a str,
    ctx: &mut ParserContext<'b>,
    _ty: Type,
) -> IResult<&'a str, ValueId, VerboseError<&'a str>> {
    let (source, name) = preceded(spaces, preceded(char('%'), super::name::parse))(source)?;
    Ok((source, ctx.get_or_create_named_value(name)))
}

pub fn parse<'a, 'b>(
    source: &'a str,
    ctx: &mut ParserContext<'b>,
    ty: Type,
) -> IResult<&'a str, ValueId, VerboseError<&'a str>> {
    if let Ok((source, konst)) = parse_constant(source, ctx.types, ty) {
        let id = ctx.data.create_value(Value::Constant(konst));
        return Ok((source, id));
    }

    parse_local(source, ctx, ty)
}
