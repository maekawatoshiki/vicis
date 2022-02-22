use crate::{
    ir::{
        module::{global_variable::GlobalVariable, linkage::Linkage},
        types,
        types::Types,
        util::spaces,
        value,
    },
    parser::assembly::{preemption_specifier, visibility},
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
    let (source, name) = preceded(spaces, preceded(char('@'), super::name::parse))(source)?;
    let (source, _) = preceded(spaces, char('='))(source)?;
    let (source, linkage) = opt(preceded(spaces, super::linkage::parse))(source)?;
    let (source, preemption_specifier) =
        opt(preceded(spaces, preemption_specifier::parse))(source)?;
    let (source, visibility) = opt(preceded(spaces, visibility::parse))(source)?;
    let (source, unnamed_addr) = opt(preceded(spaces, super::unnamed_addr::parse))(source)?;
    let (source, kind) = preceded(spaces, alt((tag("global"), tag("constant"))))(source)?;
    let (source, ty) = super::types::parse(types)(source)?;
    let (source, init) = if matches!(
        linkage,
        Some(Linkage::External) | Some(Linkage::ExternalWeak)
    ) {
        (source, None)
    } else {
        parse_init(source, types, ty)?
    };
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
            preemption_specifier,
            visibility,
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
    ty: types::Type,
) -> IResult<&'a str, Option<value::ConstantValue>, VerboseError<&'a str>> {
    if let Ok((source, init)) = super::value::parse_constant(source, types, ty) {
        return Ok((source, Some(init)));
    }
    Ok((source, None))
}

pub fn parse_global_type_and_const<'a>(
    source: &'a str,
    types: &Types,
) -> IResult<&'a str, (types::Type, value::ConstantValue), VerboseError<&'a str>> {
    let (source, ty) = super::types::parse(types)(source)?;
    let (source, konst) = super::value::parse_constant(source, types, ty)?;
    Ok((source, (ty, konst)))
}
