use crate::ir::{
    module::{global_variable::GlobalVariable, linkage, name},
    types,
    types::Types,
    util::spaces,
};
use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{char, digit1},
    combinator::{map, opt},
    error::VerboseError,
    sequence::{preceded, tuple},
    Err::Error,
    IResult,
};

// @<GlobalVarName> = [Linkage] [PreemptionSpecifier] [Visibility]
//                    [DLLStorageClass] [ThreadLocal]
//                    [(unnamed_addr|local_unnamed_addr)] [AddrSpace]
//                    [ExternallyInitialized]
//                    <global | constant> <Type> [<InitializerConstant>]
//                    [, section "name"] [, comdat [($name)]]
//                    [, align <Alignment>] (, !name !N)*

pub fn parse<'a, 'b>(
    source: &'a str,
    types: &Types,
) -> IResult<&'a str, GlobalVariable, VerboseError<&'a str>> {
    let (source, name) = preceded(spaces, preceded(char('@'), name::parse))(source)?;
    let (source, _) = preceded(spaces, char('='))(source)?;
    let (source, linkage) = opt(preceded(spaces, linkage::parse))(source)?;
    let (source, unnamed_addr) = opt(preceded(
        spaces,
        alt((tag("unnamed_addr"), tag("local_unnamed_addr"))),
    ))(source)?;
    let (source, kind) = preceded(spaces, alt((tag("global"), tag("constant"))))(source)?;
    let (source, ty) = types::parse(source, types)?;
    Ok((
        source,
        GlobalVariable {
            name,
            linkage,
            is_local_unnamed_addr: unnamed_addr.unwrap_or("unnamed_addr") == "local_unnamed_addr",
            is_constant: kind == "constant",
            ty,
        },
    ))
}
