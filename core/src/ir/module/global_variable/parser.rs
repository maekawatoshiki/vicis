use crate::ir::{
    module::{global_variable::GlobalVariable, linkage, name, unnamed_addr},
    types,
    types::Types,
    util::spaces,
    value,
};
use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{char, digit1},
    combinator::opt,
    error::VerboseError,
    sequence::preceded,
    IResult,
};

// @<GlobalVarName> = [Linkage] [PreemptionSpecifier] [Visibility]
//                    [DLLStorageClass] [ThreadLocal]
//                    [(unnamed_addr|local_unnamed_addr)] [AddrSpace]
//                    [ExternallyInitialized]
//                    <global | constant> <Type> [<InitializerConstant>]
//                    [, section "name"] [, comdat [($name)]]
//                    [, align <Alignment>] (, !name !N)*

pub fn parse<'a>(
    source: &'a str,
    types: &Types,
) -> IResult<&'a str, GlobalVariable, VerboseError<&'a str>> {
    let (source, name) = preceded(spaces, preceded(char('@'), name::parse))(source)?;
    let (source, _) = preceded(spaces, char('='))(source)?;
    let (source, linkage) = opt(preceded(spaces, linkage::parse))(source)?;
    let (source, unnamed_addr) = opt(preceded(spaces, unnamed_addr::parse))(source)?;
    let (source, kind) = preceded(spaces, alt((tag("global"), tag("constant"))))(source)?;
    let (source, ty) = types::parse(source, types)?;
    let (source, init) = parse_init(source, types, ty)?;
    let (source, align) = opt(preceded(
        spaces,
        preceded(
            char(','),
            preceded(spaces, preceded(tag("align"), preceded(spaces, digit1))),
        ),
    ))(source)?;
    Ok((
        source,
        GlobalVariable {
            name,
            linkage,
            unnamed_addr,
            is_constant: kind == "constant",
            ty,
            init,
            align: align.map_or(0, |align| align.parse::<u32>().unwrap()),
        },
    ))
}

pub fn parse_init<'a>(
    source: &'a str,
    types: &Types,
    ty: types::TypeId,
) -> IResult<&'a str, Option<value::ConstantData>, VerboseError<&'a str>> {
    if let Ok((source, _)) = preceded(spaces, tag("zeroinitializer"))(source) {
        return Ok((source, Some(value::ConstantData::AggregateZero)));
    }
    if let Ok((source, init)) = value::parser::parse_constant(source, types, ty) {
        return Ok((source, Some(init)));
    }
    Ok((source, None))
}

pub fn parse_global_type_and_const<'a>(
    source: &'a str,
    types: &Types,
) -> IResult<&'a str, (types::TypeId, value::ConstantData), VerboseError<&'a str>> {
    let (source, ty) = types::parse(source, types)?;
    let (source, konst) = value::parser::parse_constant(source, types, ty)?;
    Ok((source, (ty, konst)))
}
