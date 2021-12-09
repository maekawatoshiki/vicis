use super::super::types::{TypeId, Types};
use crate::ir::{module::name, util::spaces};
use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{char, digit1},
    combinator::map,
    error::VerboseError,
    sequence::preceded,
    IResult,
};

pub fn parse<'a>(
    source: &'a str,
    types: &Types,
) -> IResult<&'a str, TypeId, VerboseError<&'a str>> {
    let (mut source, mut base) = if let Ok((source, _)) = preceded(spaces, char('['))(source) {
        parse_array(source, types)?
    } else if let Ok((source, _)) = preceded(spaces, char('{'))(source) {
        parse_struct(source, types, false)?
    } else if let Ok((source, _)) = preceded(spaces, tag("<{"))(source) {
        parse_struct(source, types, true)?
    } else if let Ok((source, name)) = preceded(spaces, preceded(char('%'), name::parse))(source) {
        (source, types.base_mut().named_type(name))
    } else {
        preceded(
            spaces,
            alt((
                map(tag("void"), |_| types.base().void()),
                map(tag("i1"), |_| types.base().i1()),
                map(tag("i8"), |_| types.base().i8()),
                map(tag("i32"), |_| types.base().i32()),
                map(tag("i64"), |_| types.base().i64()),
                map(tag("metadata"), |_| types.base().metadata()),
            )),
        )(source)?
    };

    loop {
        if let Ok((source_, _ptr)) = preceded(spaces, char('*'))(source) {
            base = types.base_mut().pointer(base);
            source = source_;
            continue;
        }

        if let Ok((source_, _ptr)) = preceded(spaces, char('('))(source) {
            let (source_, base_) = parse_func_type(source_, types, base)?;
            base = base_;
            source = source_;
            continue;
        }

        break;
    }

    Ok((source, base))
}

fn parse_array<'a>(
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

fn parse_struct<'a>(
    mut source: &'a str,
    types: &Types,
    is_packed: bool,
) -> IResult<&'a str, TypeId, VerboseError<&'a str>> {
    if let Ok((source, _)) = preceded(spaces, tag(if is_packed { "}>" } else { "}" }))(source) {
        return Ok((source, types.base_mut().anonymous_struct(vec![], is_packed)));
    }

    let mut elems = vec![];
    loop {
        let (source_, ty) = parse(source, types)?;
        elems.push(ty);
        if let Ok((source_, _)) = preceded(spaces, char(','))(source_) {
            source = source_;
            continue;
        }
        let (source_, _) = preceded(spaces, tag(if is_packed { "}>" } else { "}" }))(source_)?;
        return Ok((source_, types.base_mut().anonymous_struct(elems, is_packed)));
    }
}

fn parse_func_type<'a>(
    mut source: &'a str,
    types: &Types,
    ret: TypeId,
) -> IResult<&'a str, TypeId, VerboseError<&'a str>> {
    if let Ok((source, _)) = preceded(spaces, char(')'))(source) {
        let func_ty = types.base_mut().function(ret, vec![], false);
        return Ok((source, func_ty));
    }

    let mut params = vec![];
    let mut is_var_arg = false;

    loop {
        if let Ok((source_, _)) = preceded(spaces, tag("..."))(source) {
            is_var_arg = true;
            source = source_;
            break;
        }

        let (source_, param) = parse(source, types)?;
        source = source_;
        params.push(param);

        if let Ok((source_, _)) = preceded(spaces, char(','))(source) {
            source = source_;
            continue;
        }

        break;
    }

    let (source, _) = preceded(spaces, char(')'))(source)?;
    let func_ty = types.base_mut().function(ret, params, is_var_arg);
    Ok((source, func_ty))
}

#[test]
fn test_metadata() {
    let types = Types::default();
    let source = "  metadata ";
    let (_, ty) = parse(source, &types).unwrap();
    assert!(types.base().metadata() == ty)
}
